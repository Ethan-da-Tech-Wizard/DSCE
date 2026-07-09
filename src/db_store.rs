//! Database-backed storage for DSCE vials.
//!
//! Persists vials (facts, rules, neighbors, evidence, term index) in
//! SQLite via `rusqlite`, and serves the engine's dynamic on-demand vial
//! loading during a query flood: only vials whose terms receive sand are
//! ever read into memory.
//!
//! One connection is opened per store and lives inside a `Mutex` for the
//! store's whole lifetime — no per-call connect/close churn, no leaked
//! handles (the connection closes when the store drops).
//!
//! RULE SERIALIZATION: a rule's procedural `compute` logic is stored as
//! its expression-DSL source string (e.g. `"?a = ?w * ?h"`) in the
//! `compute_body` column and re-parsed by [`crate::compute`] on load —
//! a deterministic, sandboxed replacement for the Python prototype's
//! `eval()` of lambda source. Rules whose compute is a native closure
//! cannot be serialized; `save_vial` reports them as an error rather than
//! silently dropping logic. A `compute_body` that fails to parse on load
//! degrades to "no compute" instead of failing the whole vial.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde_json::{json, Value};

use crate::compute::Compute;
use crate::facts::{Fact, Term};
use crate::vial::{Rule, Vial};

/// Errors surfaced by the store: either SQLite itself, or a
/// (de)serialization problem in our own encoding.
#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Encoding(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "sqlite error: {e}"),
            StoreError::Encoding(msg) => write!(f, "encoding error: {msg}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> StoreError {
        StoreError::Sqlite(e)
    }
}

pub type StoreResult<T> = Result<T, StoreError>;

pub struct SqliteVialStore {
    conn: Mutex<Connection>,
}

impl SqliteVialStore {
    /// Open (or create) the database at `path` and ensure the schema exists.
    pub fn open(path: impl AsRef<Path>) -> StoreResult<SqliteVialStore> {
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(SqliteVialStore { conn: Mutex::new(conn) })
    }

    /// An in-memory database — handy for tests.
    pub fn open_in_memory() -> StoreResult<SqliteVialStore> {
        let conn = Connection::open_in_memory()?;
        Self::init_schema(&conn)?;
        Ok(SqliteVialStore { conn: Mutex::new(conn) })
    }

    fn init_schema(conn: &Connection) -> StoreResult<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS vials (
                 id TEXT PRIMARY KEY,
                 concept TEXT,
                 evidence TEXT,
                 confidence REAL,
                 neighbors TEXT
             );
             CREATE TABLE IF NOT EXISTS facts (
                 vial_id TEXT,
                 subject TEXT,
                 predicate TEXT,
                 object TEXT,
                 FOREIGN KEY(vial_id) REFERENCES vials(id)
             );
             CREATE TABLE IF NOT EXISTS rules (
                 vial_id TEXT,
                 name TEXT,
                 premises TEXT,
                 conclusion TEXT,
                 compute_body TEXT,
                 confidence REAL,
                 FOREIGN KEY(vial_id) REFERENCES vials(id)
             );
             CREATE TABLE IF NOT EXISTS term_index (
                 term TEXT,
                 vial_id TEXT,
                 PRIMARY KEY(term, vial_id)
             );
             CREATE INDEX IF NOT EXISTS idx_facts_vial ON facts(vial_id);
             CREATE INDEX IF NOT EXISTS idx_rules_vial ON rules(vial_id);
             CREATE INDEX IF NOT EXISTS idx_term_index ON term_index(term);",
        )?;
        Ok(())
    }

    /// Persist one vial (metadata, facts, rules, term index) atomically.
    pub fn save_vial(&self, vial: &Vial) -> StoreResult<()> {
        let mut conn = self.conn.lock().expect("store mutex poisoned");
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT OR REPLACE INTO vials (id, concept, evidence, confidence, neighbors)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                vial.id,
                vial.concept,
                json!(vial.evidence).to_string(),
                vial.confidence,
                json!(vial.neighbors).to_string(),
            ],
        )?;

        tx.execute("DELETE FROM facts WHERE vial_id = ?1", params![vial.id])?;
        for (subject, predicate, object) in &vial.facts {
            tx.execute(
                "INSERT INTO facts (vial_id, subject, predicate, object) VALUES (?1, ?2, ?3, ?4)",
                params![vial.id, term_to_db(subject), term_to_db(predicate), term_to_db(object)],
            )?;
        }

        tx.execute("DELETE FROM rules WHERE vial_id = ?1", params![vial.id])?;
        for rule in &vial.rules {
            let compute_body: Option<String> = match &rule.compute {
                None => None,
                Some(compute) => Some(
                    compute
                        .source()
                        .ok_or_else(|| {
                            StoreError::Encoding(format!(
                                "rule {:?} has a native compute closure that cannot be serialized; \
                                 express it as Compute::expr(...) to store it",
                                rule.name
                            ))
                        })?
                        .to_string(),
                ),
            };
            let premises = Value::Array(
                rule.premises
                    .iter()
                    .map(|p| json!([term_to_json(&p.0), term_to_json(&p.1), term_to_json(&p.2)]))
                    .collect(),
            );
            let conclusion = json!([
                term_to_json(&rule.conclusion.0),
                term_to_json(&rule.conclusion.1),
                term_to_json(&rule.conclusion.2)
            ]);
            tx.execute(
                "INSERT INTO rules (vial_id, name, premises, conclusion, compute_body, confidence)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    vial.id,
                    rule.name,
                    premises.to_string(),
                    conclusion.to_string(),
                    compute_body,
                    rule.confidence
                ],
            )?;
        }

        tx.execute("DELETE FROM term_index WHERE vial_id = ?1", params![vial.id])?;
        for term in vial.terms() {
            tx.execute(
                "INSERT OR IGNORE INTO term_index (term, vial_id) VALUES (?1, ?2)",
                params![term_to_db(&term), vial.id],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// The ids of every vial indexed under `term`, sorted for determinism.
    pub fn get_vial_ids_for_term(&self, term: &Term) -> Vec<String> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        let mut stmt = match conn.prepare("SELECT vial_id FROM term_index WHERE term = ?1 ORDER BY vial_id") {
            Ok(stmt) => stmt,
            Err(_) => return Vec::new(),
        };
        stmt.query_map(params![term_to_db(term)], |row| row.get::<_, String>(0))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    /// Load one vial, fully materialized, by id.
    pub fn load_vial(&self, vial_id: &str) -> StoreResult<Vial> {
        let conn = self.conn.lock().expect("store mutex poisoned");

        let (concept, evidence_json, confidence, neighbors_json) = conn
            .query_row(
                "SELECT concept, evidence, confidence, neighbors FROM vials WHERE id = ?1",
                params![vial_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    StoreError::Encoding(format!("vial {vial_id:?} not found in database"))
                }
                other => StoreError::Sqlite(other),
            })?;
        let evidence: Vec<String> = serde_json::from_str(&evidence_json)
            .map_err(|e| StoreError::Encoding(format!("bad evidence JSON for {vial_id:?}: {e}")))?;
        let neighbors: Vec<String> = serde_json::from_str(&neighbors_json)
            .map_err(|e| StoreError::Encoding(format!("bad neighbors JSON for {vial_id:?}: {e}")))?;

        let mut facts: Vec<Fact> = Vec::new();
        {
            let mut stmt =
                conn.prepare("SELECT subject, predicate, object FROM facts WHERE vial_id = ?1")?;
            let rows = stmt.query_map(params![vial_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?;
            for row in rows {
                let (s, p, o) = row?;
                facts.push((term_from_db(&s), term_from_db(&p), term_from_db(&o)));
            }
        }

        let mut rules: Vec<Rule> = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT name, premises, conclusion, compute_body, confidence FROM rules WHERE vial_id = ?1",
            )?;
            let rows = stmt.query_map(params![vial_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, f64>(4)?,
                ))
            })?;
            for row in rows {
                let (name, premises_json, conclusion_json, compute_body, rule_confidence) = row?;
                let premises_value: Value = serde_json::from_str(&premises_json)
                    .map_err(|e| StoreError::Encoding(format!("bad premises JSON for rule {name:?}: {e}")))?;
                let premises = premises_value
                    .as_array()
                    .ok_or_else(|| StoreError::Encoding(format!("premises of rule {name:?} not an array")))?
                    .iter()
                    .map(triple_from_json)
                    .collect::<Result<Vec<_>, _>>()?;
                let conclusion_value: Value = serde_json::from_str(&conclusion_json)
                    .map_err(|e| StoreError::Encoding(format!("bad conclusion JSON for rule {name:?}: {e}")))?;
                let conclusion = triple_from_json(&conclusion_value)?;
                // A body that fails to parse (e.g. a Python lambda from the
                // legacy prototype's databases) degrades to "no compute".
                let compute = compute_body.as_deref().and_then(|body| Compute::expr(body).ok());
                rules.push(Rule {
                    name,
                    premises,
                    conclusion,
                    compute,
                    confidence: rule_confidence,
                });
            }
        }

        Ok(Vial {
            id: vial_id.to_string(),
            concept,
            facts,
            rules,
            neighbors,
            evidence,
            confidence,
        })
    }
}

/// Encode a term for the TEXT columns of `facts` and `term_index`.
///
/// Plain string form (not JSON) for compatibility with databases seeded by
/// the Python prototype's `SqliteVialStore`, which stored `str(term)`.
fn term_to_db(term: &Term) -> String {
    term.to_string()
}

/// Decode a `facts`/`term_index` TEXT column back into a typed term, using
/// the same heuristic the Python store used: bool, then int, then float,
/// and anything left stays a string.
pub fn term_from_db(value: &str) -> Term {
    match value.to_ascii_lowercase().as_str() {
        "true" => return Term::Bool(true),
        "false" => return Term::Bool(false),
        _ => {}
    }
    if let Ok(i) = value.parse::<i64>() {
        return Term::Int(i);
    }
    if let Ok(f) = value.parse::<f64>() {
        return Term::float(f);
    }
    Term::str(value)
}

/// Encode a term as a JSON value (used for rule premises/conclusions,
/// where JSON keeps the native type intact).
fn term_to_json(term: &Term) -> Value {
    match term {
        Term::Str(s) => json!(s),
        Term::Int(i) => json!(i),
        Term::Float(f) => json!(f.0),
        Term::Bool(b) => json!(b),
    }
}

pub(crate) fn term_from_json(value: &Value) -> StoreResult<Term> {
    match value {
        Value::String(s) => Ok(Term::str(s.clone())),
        Value::Bool(b) => Ok(Term::Bool(*b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Term::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Term::float(f))
            } else {
                Err(StoreError::Encoding(format!("unrepresentable number {n}")))
            }
        }
        other => Err(StoreError::Encoding(format!("cannot decode term from {other}"))),
    }
}

pub(crate) fn triple_from_json(value: &Value) -> StoreResult<(Term, Term, Term)> {
    let items = value
        .as_array()
        .filter(|a| a.len() == 3)
        .ok_or_else(|| StoreError::Encoding(format!("expected a 3-element array, got {value}")))?;
    Ok((
        term_from_json(&items[0])?,
        term_from_json(&items[1])?,
        term_from_json(&items[2])?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_db_round_trip() {
        assert_eq!(term_from_db("socrates"), Term::str("socrates"));
        assert_eq!(term_from_db("12"), Term::Int(12));
        assert_eq!(term_from_db("12.5"), Term::float(12.5));
        assert_eq!(term_from_db("true"), Term::Bool(true));
        assert_eq!(term_from_db("False"), Term::Bool(false));
    }
}

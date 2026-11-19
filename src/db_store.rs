// ==============================================================================
// Program: db_store.rs
// Programmer: Ethan-da-Tech-Wizard
// Last Modified: November 18, 2026
// Purpose: SQLite persistence layer and dynamic vial provider for DSCE Rust engine.
//          Manages thread-safe SQLite connection handles, persists vial metadata,
//          axioms, and DSL compute rules, and executes high-performance inverted
//          term lookups during spreading activation floods.
// ==============================================================================

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

// File role: persistence for vials. Called by main.rs (seeding and querying a database file), by json_vials.rs
// (import of JSON vials), and by engine.rs through the dynamic-vial provider (`get_vial_ids_for_term` and
// `load_vial`) while a query is spreading activation.
//
// Storage layout (see `init_schema`): `vials` (one row per vial), `facts` and `rules` (child rows), and
// `term_index` (term -> vial id), which is what lets the engine wake only the vials that mention a term.
//
// Encoding choices, and why: facts use plain text so the Python prototype's databases stay readable;
// rule premises/conclusions use JSON so integer/float/bool types survive exactly; evidence and neighbours are
// JSON arrays in TEXT columns because they are always read whole.
//
// Known limits: string "12" and integer 12 are indistinguishable in the `facts` table (see `term_from_db`).
// Tested below and in tests/engine_tests.rs; run `cargo test db_store`.
use std::path::Path;
// rusqlite's Connection is not Sync, so it is wrapped in a Mutex to let the store be shared across threads.
// Mutex (not RwLock): every operation needs exclusive use of the single Connection, readers included.
use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde_json::{json, Value};

use crate::compute::Compute;
use crate::facts::{Fact, Term};
use crate::vial::{Rule, Vial};

/// Errors surfaced by the store: either SQLite itself, or a
/// (de)serialization problem in our own encoding.
#[derive(Debug)]
/// Sqlite wraps driver errors; Encoding covers our own JSON/term decoding problems and "not found".
// Two variants instead of one boxed error so callers (and the CLI) can tell database failures from bad data.
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Encoding(String),
}

// Human-readable messages for CLI output.
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "sqlite error: {e}"),
            StoreError::Encoding(msg) => write!(f, "encoding error: {msg}"),
        }
    }
}

impl std::error::Error for StoreError {}

// Lets `?` convert driver errors automatically.
impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> StoreError {
        StoreError::Sqlite(e)
    }
}

/// Shorthand for results of store operations.
pub type StoreResult<T> = Result<T, StoreError>;

/// SQLite-backed vial storage. The schema mirrors the Python implementation so a
/// database file can be shared between the two engines.
// Compatibility note: column names and the plain-text fact encoding match the Python store, so either engine can open the other's file.
pub struct SqliteVialStore {
    // Single long-lived connection; lock it for the duration of each operation.
    conn: Mutex<Connection>,
}

impl SqliteVialStore {
    /// Open (or create) the database at `path` and ensure the schema exists.
    // Errors: SQLite cannot open or create the file (bad path, permissions, corrupt file). `open` never deletes existing data.
    pub fn open(path: impl AsRef<Path>) -> StoreResult<SqliteVialStore> {
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(SqliteVialStore { conn: Mutex::new(conn) })
    }

    /// An in-memory database — handy for tests.
    // Each call creates a separate private database that disappears when the store is dropped, which gives every test isolation.
    pub fn open_in_memory() -> StoreResult<SqliteVialStore> {
        let conn = Connection::open_in_memory()?;
        Self::init_schema(&conn)?;
        Ok(SqliteVialStore { conn: Mutex::new(conn) })
    }

    // Create tables and indexes if missing (idempotent). Evidence/neighbors are JSON TEXT;
    // term_index is the inverted index term -> vial used to wake vials without loading all of them.
    // Safe to call on every open: CREATE ... IF NOT EXISTS leaves existing tables and rows untouched.
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
    // Errors: any SQLite failure, or an `Encoding` error when a rule has a native closure compute. On error the
    // transaction is dropped and nothing is written. Re-saving a vial with the same id replaces the old content.
    pub fn save_vial(&self, vial: &Vial) -> StoreResult<()> {
        // A poisoned mutex means another thread panicked mid-write; panicking here is intentional rather than using possibly inconsistent state.
        let mut conn = self.conn.lock().expect("store mutex poisoned");
        // One transaction so a vial is saved all-or-nothing; dropping `tx` on an early error rolls everything back.
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT OR REPLACE INTO vials (id, concept, evidence, confidence, neighbors)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                vial.id,
                vial.concept,
                // Evidence and neighbours are stored as JSON text, which keeps them as plain TEXT columns with no extra tables.
                json!(vial.evidence).to_string(),
                vial.confidence,
                json!(vial.neighbors).to_string(),
            ],
        )?;

        // Replace semantics: delete the old child rows first so removed facts do not linger.
        tx.execute("DELETE FROM facts WHERE vial_id = ?1", params![vial.id])?;
        for (subject, predicate, object) in &vial.facts {
            tx.execute(
                "INSERT INTO facts (vial_id, subject, predicate, object) VALUES (?1, ?2, ?3, ?4)",
                params![vial.id, term_to_db(subject), term_to_db(predicate), term_to_db(object)],
            )?;
        }

        tx.execute("DELETE FROM rules WHERE vial_id = ?1", params![vial.id])?;
        for rule in &vial.rules {
            // Only expression-DSL computes have source text; a native closure is refused with an error instead of being silently dropped on save.
            // Rules with no compute store NULL; this is the column the loader reads back through `Compute::expr`.
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

            // Premises are an ordered list of triples, so they are stored as one JSON array of 3-element arrays.
            let premises = Value::Array(
                rule.premises
                    .iter()
                    .map(|p| json!([term_to_json(&p.0), term_to_json(&p.1), term_to_json(&p.2)]))
                    .collect(),
            );

            // The conclusion is a single triple in the same JSON shape as one premise.
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

        // The index is rebuilt from vial.terms() on every save so it always matches the stored content.
        tx.execute("DELETE FROM term_index WHERE vial_id = ?1", params![vial.id])?;

        // `INSERT OR IGNORE` because the same term can appear in several facts and rules but needs only one index row.
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
    // Hot path: called for each term that receives sand during a flood. It is a single indexed lookup (idx_term_index).
    pub fn get_vial_ids_for_term(&self, term: &Term) -> Vec<String> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        // ORDER BY gives a deterministic wake order. Failures yield an empty list: a lookup that fails simply wakes nothing.
        let mut stmt = match conn.prepare("SELECT vial_id FROM term_index WHERE term = ?1 ORDER BY vial_id") {
            Ok(stmt) => stmt,
            Err(_) => return Vec::new(),
        };
        stmt.query_map(params![term_to_db(term)], |row| row.get::<_, String>(0))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    /// Load one vial, fully materialized, by id.
    // Errors: unknown vial id, SQLite failures, or malformed JSON in evidence/neighbours/rules (the message names the vial or rule).
    pub fn load_vial(&self, vial_id: &str) -> StoreResult<Vial> {
        // The lock is held for the whole load, so the several SELECTs see one consistent snapshot.
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
                // A missing row is reported as a data problem with the vial id, which is clearer than SQLite's generic "no rows".
                rusqlite::Error::QueryReturnedNoRows => {
                    StoreError::Encoding(format!("vial {vial_id:?} not found in database"))
                }
                other => StoreError::Sqlite(other),
            })?;
        let evidence: Vec<String> = serde_json::from_str(&evidence_json)
            .map_err(|e| StoreError::Encoding(format!("bad evidence JSON for {vial_id:?}: {e}")))?;
        let neighbors: Vec<String> = serde_json::from_str(&neighbors_json)
            .map_err(|e| StoreError::Encoding(format!("bad neighbors JSON for {vial_id:?}: {e}")))?;

        // Inner blocks below scope each prepared statement so it is dropped before the next one is prepared.
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
                // Lenient on purpose: an unparsable body (legacy Python lambda text) just means the rule has no compute step,
                // rather than making the entire vial unloadable.
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
// Mirror of `term_from_db`; goes through Term's Display impl, which prints the same text the Python store wrote.
fn term_to_db(term: &Term) -> String {
    term.to_string()
}

/// Decode a `facts`/`term_index` TEXT column back into a typed term, using
/// the same heuristic the Python store used: bool, then int, then float,
/// and anything left stays a string.
// Order of attempts: bool, int, float, string. "12" therefore becomes Int(12) even if it was originally a string, which is inherited behaviour.
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
// Unlike `term_to_db`, JSON keeps the type tag (string vs number vs bool), so rule terms round-trip exactly.
fn term_to_json(term: &Term) -> Value {
    match term {
        Term::Str(s) => json!(s),
        Term::Int(i) => json!(i),
        Term::Float(f) => json!(f.0),
        Term::Bool(b) => json!(b),
    }
}

/// Decode a JSON value into a term; integers are tried before floats so 3 stays Int.
// `as_i64` is tried first so JSON `3` becomes Int(3) rather than Float(3.0). Arrays, objects and null are rejected.
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

/// Decode a 3-element JSON array into a (subject, predicate, object) triple, rejecting any other length.
// Shared with harvester.rs (LLM replies) and json_vials.rs, so every JSON triple goes through one validator.
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
// Note: facts are stored as plain text, so a string like "12" is read back as the integer 12 (inherited from the Python store).
// Only the pure decoding helper is unit-tested here; full save/load round trips are covered by the integration tests in tests/.
mod tests {
    use super::*;

    #[test]
    // Pins the bool -> int -> float -> string decoding order, including case-insensitive booleans ("False").
    fn term_db_round_trip() {
        assert_eq!(term_from_db("socrates"), Term::str("socrates"));
        assert_eq!(term_from_db("12"), Term::Int(12));
        assert_eq!(term_from_db("12.5"), Term::float(12.5));
        assert_eq!(term_from_db("true"), Term::Bool(true));
        assert_eq!(term_from_db("False"), Term::Bool(false));
    }
}

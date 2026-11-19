# ==============================================================================
# Program: db_store.py
# Programmer: Ethan-da-Tech-Wizard
# Last Modified: November 18, 2026
# Purpose: SQLite storage backend for DSCE knowledge vials and term indices.
#          Manages relational schema creation (vials, facts, rules, term_index),
#          vial serialization/deserialization, inverted term lookup queries,
#          and dynamic on-demand vial loading during activation floods.
# ==============================================================================

"""Database-backed storage for DSCE vials.

Allows persistence of vials (facts, rules, neighbors, terms) in SQLite
and dynamic on-demand loading of vials during the engine's query flood.
"""

from __future__ import annotations

# json encodes the list-valued columns (evidence, neighbors, premises) as TEXT,
# because SQLite has no array type.
import json
# Standard-library SQLite only: no server, no extra dependency, one local file.
import sqlite3
from typing import Optional

from dsce.facts import Fact, Pattern, Term
from dsce.vial import Rule, Vial

# Design note: every method opens its own short-lived connection and closes it
# in `finally`. That keeps the store safe to share between threads/processes
# (no connection object is ever shared) at a small per-call cost.
class SqliteVialStore:
    """
    SQLite persistence manager for DSCE vials, facts, rules, and inverted term indices.
    """
    # db_path is a filesystem path (or ":memory:", although each call opens a fresh
    # connection so an in-memory database would not persist between calls).
    def __init__(self, db_path: str):
        self.db_path = db_path
        # Create the schema eagerly so later calls can assume the tables exist.
        self._init_db()

    def _init_db(self):
        """Initializes relational tables and performance indexes in SQLite."""
        conn = sqlite3.connect(self.db_path)
        try:
            # One row per vial. evidence and neighbors are JSON-encoded lists.
            # Step 1: Create primary vial metadata table
            conn.execute("""
                CREATE TABLE IF NOT EXISTS vials (
                    id TEXT PRIMARY KEY,
                    concept TEXT,
                    evidence TEXT,
                    confidence REAL,
                    neighbors TEXT
                )
            """)
            # Facts are stored as three TEXT columns; typed values (int/float/bool) are
            # recovered on load by _parse_term.
            # Step 2: Create ground axiom facts table
            conn.execute("""
                CREATE TABLE IF NOT EXISTS facts (
                    vial_id TEXT,
                    subject TEXT,
                    predicate TEXT,
                    object TEXT,
                    FOREIGN KEY(vial_id) REFERENCES vials(id)
                )
            """)
            # premises/conclusion are JSON lists of terms. compute_body holds the source
            # text of the rule's compute function, since Python callables cannot be stored.
            # Step 3: Create inference rules table
            conn.execute("""
                CREATE TABLE IF NOT EXISTS rules (
                    vial_id TEXT,
                    name TEXT,
                    premises TEXT,
                    conclusion TEXT,
                    compute_body TEXT,
                    confidence REAL,
                    FOREIGN KEY(vial_id) REFERENCES vials(id)
                )
            """)
            # The inverted index (term -> vials mentioning it) is what lets the engine wake
            # vials without loading them all; the composite primary key prevents duplicates.
            # Step 4: Create term-to-vial inverted index table
            conn.execute("""
                CREATE TABLE IF NOT EXISTS term_index (
                    term TEXT,
                    vial_id TEXT,
                    PRIMARY KEY(term, vial_id)
                )
            """)
            # Step 5: Establish database indexes for high-speed term lookup
            conn.execute("CREATE INDEX IF NOT EXISTS idx_facts_vial ON facts(vial_id)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_rules_vial ON rules(vial_id)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_term_index ON term_index(term)")
        finally:
            conn.close()

    def save_vial(self, vial: Vial):
        """
        Persists a complete Vial object (metadata, facts, rules, and terms) into SQLite.
        """
        conn = sqlite3.connect(self.db_path)
        try:
            # INSERT OR REPLACE makes saving idempotent: re-saving a vial updates it.
            # Step 1: Save or replace vial metadata record
            conn.execute(
                "INSERT OR REPLACE INTO vials (id, concept, evidence, confidence, neighbors) VALUES (?, ?, ?, ?, ?)",
                (
                    vial.id,
                    vial.concept,
                    json.dumps(list(vial.evidence)),
                    vial.confidence,
                    json.dumps(list(vial.neighbors))
                )
            )
            
            # Child rows are deleted then re-inserted so facts removed from the vial
            # object also disappear from the database.
            # Step 2: Overwrite ground facts associated with vial
            conn.execute("DELETE FROM facts WHERE vial_id = ?", (vial.id,))
            for fact in vial.facts:
                conn.execute(
                    "INSERT INTO facts (vial_id, subject, predicate, object) VALUES (?, ?, ?, ?)",
                    (vial.id, str(fact[0]), str(fact[1]), str(fact[2]))
                )
                
            # Step 3: Overwrite inference rules associated with vial
            conn.execute("DELETE FROM rules WHERE vial_id = ?", (vial.id,))
            for rule in vial.rules:
                # _compute_body is an optional attribute attached at load time (see load_vial);
                # rules built in code without it are stored with a NULL compute body.
                compute_body = getattr(rule, "_compute_body", None)
                conn.execute(
                    "INSERT INTO rules (vial_id, name, premises, conclusion, compute_body, confidence) VALUES (?, ?, ?, ?, ?, ?)",
                    (
                        vial.id,
                        rule.name,
                        json.dumps([list(p) for p in rule.premises]),
                        json.dumps(list(rule.conclusion)),
                        compute_body,
                        rule.confidence
                    )
                )
                
            # Terms are stringified because the index column is TEXT; get_vial_ids_for_term
            # applies the same str() so lookups match.
            # Step 4: Rebuild inverted term index for this vial
            conn.execute("DELETE FROM term_index WHERE vial_id = ?", (vial.id,))
            for term in vial.terms():
                conn.execute(
                    "INSERT OR IGNORE INTO term_index (term, vial_id) VALUES (?, ?)",
                    (str(term), vial.id)
                )
            # All the writes above form one transaction; nothing is visible until this commit.
            conn.commit()
        finally:
            conn.close()

    def get_vial_ids_for_term(self, term: Term) -> tuple[str, ...]:
        """Queries inverted term index to find all vial IDs mentioning the term."""
        conn = sqlite3.connect(self.db_path)
        try:
            cursor = conn.cursor()
            # ORDER BY keeps the result deterministic, which the engine's reproducibility depends on.
            cursor.execute("SELECT vial_id FROM term_index WHERE term = ? ORDER BY vial_id", (str(term),))
            return tuple(row[0] for row in cursor.fetchall())
        finally:
            conn.close()

    def load_vial(self, vial_id: str) -> Vial:
        """Loads and reconstructs a Vial object from SQLite records."""
        conn = sqlite3.connect(self.db_path)
        try:
            cursor = conn.cursor()
            
            # Step 1: Retrieve vial metadata
            cursor.execute("SELECT concept, evidence, confidence, neighbors FROM vials WHERE id = ?", (vial_id,))
            row = cursor.fetchone()
            if not row:
                raise ValueError(f"vial {vial_id!r} not found in database")
            concept, evidence_json, confidence, neighbors_json = row
            evidence = tuple(json.loads(evidence_json))
            neighbors = tuple(json.loads(neighbors_json))
            
            # Facts come back as strings and are converted to typed terms.
            # Step 2: Retrieve and parse facts
            cursor.execute("SELECT subject, predicate, object FROM facts WHERE vial_id = ?", (vial_id,))
            facts = []
            for r in cursor.fetchall():
                facts.append((self._parse_term(r[0]), self._parse_term(r[1]), self._parse_term(r[2])))
            
            # Step 3: Retrieve and parse inference rules
            cursor.execute("SELECT name, premises, conclusion, compute_body, confidence FROM rules WHERE vial_id = ?", (vial_id,))
            rules = []
            for r in cursor.fetchall():
                name, premises_json, conclusion_json, compute_body, rule_confidence = r
                premises = tuple(tuple(self._parse_term(t) for t in p) for p in json.loads(premises_json))
                conclusion = tuple(self._parse_term(t) for t in json.loads(conclusion_json))
                
                compute = None
                # SECURITY NOTE: compute_body is executed with eval(), so the database must be
                # trusted. A tampered database could run arbitrary code when a vial is loaded.
                if compute_body:
                    try:
                        compute = eval(compute_body)
                    # A body that fails to compile leaves compute=None: the rule then fires
                    # without extra computed bindings instead of crashing the load.
                    except Exception:
                        pass
                
                rule = Rule(
                    name=name,
                    premises=premises,
                    conclusion=conclusion,
                    compute=compute,
                    confidence=rule_confidence
                )
                if compute_body:
                    # Rule is a frozen dataclass, so object.__setattr__ is used to remember the
                    # source text, letting save_vial round-trip it later.
                    object.__setattr__(rule, "_compute_body", compute_body)
                rules.append(rule)
                
            return Vial(
                id=vial_id,
                concept=concept,
                facts=tuple(facts),
                rules=tuple(rules),
                neighbors=neighbors,
                evidence=evidence,
                confidence=confidence
            )
        finally:
            conn.close()

    def _parse_term(self, val_str: str) -> Term:
        """Helper to cast serialized term strings into typed primitives."""
        if not isinstance(val_str, str):
            return val_str
        # Order matters: booleans first, then int, then float, then plain string. Trying
        # int before float keeps "3" an int and "3.5" a float.
        if val_str.lower() in ("true", "false"):
            return val_str.lower() == "true"
        try:
            return int(val_str)
        except ValueError:
            pass
        try:
            return float(val_str)
        except ValueError:
            pass
        return val_str

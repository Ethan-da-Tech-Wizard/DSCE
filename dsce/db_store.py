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

import json
import sqlite3
from typing import Optional

from dsce.facts import Fact, Pattern, Term
from dsce.vial import Rule, Vial

class SqliteVialStore:
    """
    SQLite persistence manager for DSCE vials, facts, rules, and inverted term indices.
    """
    def __init__(self, db_path: str):
        self.db_path = db_path
        self._init_db()

    def _init_db(self):
        """Initializes relational tables and performance indexes in SQLite."""
        conn = sqlite3.connect(self.db_path)
        try:
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
                
            # Step 4: Rebuild inverted term index for this vial
            conn.execute("DELETE FROM term_index WHERE vial_id = ?", (vial.id,))
            for term in vial.terms():
                conn.execute(
                    "INSERT OR IGNORE INTO term_index (term, vial_id) VALUES (?, ?)",
                    (str(term), vial.id)
                )
            conn.commit()
        finally:
            conn.close()

    def get_vial_ids_for_term(self, term: Term) -> tuple[str, ...]:
        """Queries inverted term index to find all vial IDs mentioning the term."""
        conn = sqlite3.connect(self.db_path)
        try:
            cursor = conn.cursor()
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
                if compute_body:
                    try:
                        compute = eval(compute_body)
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

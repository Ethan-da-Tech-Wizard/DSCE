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
    def __init__(self, db_path: str):
        self.db_path = db_path
        self._init_db()

    def _init_db(self):
        conn = sqlite3.connect(self.db_path)
        try:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS vials (
                    id TEXT PRIMARY KEY,
                    concept TEXT,
                    evidence TEXT,
                    confidence REAL,
                    neighbors TEXT
                )
            """)
            conn.execute("""
                CREATE TABLE IF NOT EXISTS facts (
                    vial_id TEXT,
                    subject TEXT,
                    predicate TEXT,
                    object TEXT,
                    FOREIGN KEY(vial_id) REFERENCES vials(id)
                )
            """)
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
            conn.execute("""
                CREATE TABLE IF NOT EXISTS term_index (
                    term TEXT,
                    vial_id TEXT,
                    PRIMARY KEY(term, vial_id)
                )
            """)
            # Indexes for faster lookups
            conn.execute("CREATE INDEX IF NOT EXISTS idx_facts_vial ON facts(vial_id)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_rules_vial ON rules(vial_id)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_term_index ON term_index(term)")
        finally:
            conn.close()

    def save_vial(self, vial: Vial):
        conn = sqlite3.connect(self.db_path)
        try:
            # Save vial metadata
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
            
            # Save facts
            conn.execute("DELETE FROM facts WHERE vial_id = ?", (vial.id,))
            for fact in vial.facts:
                conn.execute(
                    "INSERT INTO facts (vial_id, subject, predicate, object) VALUES (?, ?, ?, ?)",
                    (vial.id, str(fact[0]), str(fact[1]), str(fact[2]))
                )
                
            # Save rules
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
                
            # Rebuild term index for this vial
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
        conn = sqlite3.connect(self.db_path)
        try:
            cursor = conn.cursor()
            cursor.execute("SELECT vial_id FROM term_index WHERE term = ? ORDER BY vial_id", (str(term),))
            return tuple(row[0] for row in cursor.fetchall())
        finally:
            conn.close()

    def load_vial(self, vial_id: str) -> Vial:
        conn = sqlite3.connect(self.db_path)
        try:
            cursor = conn.cursor()
            
            # Get metadata
            cursor.execute("SELECT concept, evidence, confidence, neighbors FROM vials WHERE id = ?", (vial_id,))
            row = cursor.fetchone()
            if not row:
                raise ValueError(f"vial {vial_id!r} not found in database")
            concept, evidence_json, confidence, neighbors_json = row
            evidence = tuple(json.loads(evidence_json))
            neighbors = tuple(json.loads(neighbors_json))
            
            # Get facts
            cursor.execute("SELECT subject, predicate, object FROM facts WHERE vial_id = ?", (vial_id,))
            facts = []
            for r in cursor.fetchall():
                facts.append((self._parse_term(r[0]), self._parse_term(r[1]), self._parse_term(r[2])))
            
            # Get rules
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


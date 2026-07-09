import unittest
import os
import tempfile
from dsce.engine import Engine
from dsce.vial import Vial, Rule
from dsce.facts import is_more_specific
from dsce.db_store import SqliteVialStore
from dsce.demo_kb import build_engine


class TestConflicts(unittest.TestCase):
    def test_functional_predicate_conflict(self):
        engine = Engine()
        engine.register_predicate("height", functional=True)
        
        # Source A says height is 44
        engine.add_vial(Vial(
            id="source_a",
            concept="Source A survey",
            facts=(("tower of pizza in modesto ca", "height", 44),),
            evidence=("Survey A",)
        ))
        
        # Source B says height is 45
        engine.add_vial(Vial(
            id="source_b",
            concept="Source B survey",
            facts=(("tower of pizza in modesto ca", "height", 45),),
            evidence=("Survey B",)
        ))
        
        result = engine.ask(("tower of pizza in modesto ca", "height", "?h"))
        self.assertEqual(len(result.answers), 2)
        
        # Conflict should be detected
        self.assertEqual(len(result.conflicts), 1)
        conflict = result.conflicts[0]
        
        # Verify the conflict contains both facts
        fact1, fact2 = conflict
        self.assertEqual(fact1[0], "tower of pizza in modesto ca")
        self.assertEqual(fact1[1], "height")
        self.assertTrue(fact1[2] in (44, 45))
        self.assertTrue(fact2[2] in (44, 45))
        self.assertNotEqual(fact1[2], fact2[2])
        
        summary = result.summary()
        self.assertIn("!!! CONFLICT WARNING !!!", summary)
        self.assertIn("Conflict detected for functional predicate 'height'", summary)
        self.assertIn("tower of pizza in modesto ca height 44", summary)
        self.assertIn("tower of pizza in modesto ca height 45", summary)


class TestSpecificity(unittest.TestCase):
    def test_lexical_specificity(self):
        self.assertTrue(is_more_specific("tower of pie in ajo arizona", "tower of pie"))
        self.assertFalse(is_more_specific("tower of pie", "tower of pie in ajo arizona"))
        self.assertFalse(is_more_specific("tower of pie", "tower of pie"))
        self.assertFalse(is_more_specific("tower of pie", 123))

    def test_taxonomic_specificity(self):
        # Build taxonomic working memory: socrates is a human, human is a mammal
        wm = {
            ("socrates", "is_a", "human"): None,
            ("human", "is_a", "mammal"): None,
        }
        self.assertTrue(is_more_specific("socrates", "human", wm))
        self.assertTrue(is_more_specific("socrates", "mammal", wm))
        self.assertTrue(is_more_specific("human", "mammal", wm))
        self.assertFalse(is_more_specific("mammal", "socrates", wm))
        self.assertFalse(is_more_specific("socrates", "geometry", wm))

    def test_specificity_summary_output(self):
        engine = Engine()
        engine.add_vial(Vial(
            id="survey",
            concept="Location survey",
            facts=(
                ("tower of pie in ajo arizona", "height", 55),
                ("tower of pie", "height", 54),
            )
        ))
        
        result = engine.ask(("?what", "height", "?h"))
        self.assertEqual(len(result.answers), 2)
        
        summary = result.summary()
        self.assertIn("Answer 2 ('tower of pie in ajo arizona') contains more detailed/specific context than Answer 1 ('tower of pie')", summary)


class TestSqliteVialStore(unittest.TestCase):
    def setUp(self):
        self.db_fd, self.db_path = tempfile.mkstemp()
        self.store = SqliteVialStore(self.db_path)
        
    def tearDown(self):
        os.close(self.db_fd)
        os.unlink(self.db_path)

    def test_save_and_load_vial(self):
        # Get demo geometry vial (contains facts, rules with computes, evidence, neighbors)
        kb = build_engine()
        geometry_vial = kb.vials["geometry"]
        
        # Attach compute body dynamically to test serialization
        for rule in geometry_vial.rules:
            if rule.name == "rectangle-area":
                object.__setattr__(rule, "_compute_body", "lambda b: {'?a': b['?w'] * b['?h']}")
                
        self.store.save_vial(geometry_vial)
        
        # Load the vial back
        loaded = self.store.load_vial("geometry")
        
        self.assertEqual(loaded.id, "geometry")
        self.assertEqual(loaded.concept, geometry_vial.concept)
        self.assertEqual(loaded.confidence, geometry_vial.confidence)
        self.assertEqual(loaded.neighbors, geometry_vial.neighbors)
        self.assertEqual(loaded.evidence, geometry_vial.evidence)
        self.assertEqual(len(loaded.rules), len(geometry_vial.rules))
        
        # Verify compute execution
        rect_area_rule = next(r for r in loaded.rules if r.name == "rectangle-area")
        self.assertIsNotNone(rect_area_rule.compute)
        bindings = {"?w": 10, "?h": 20}
        self.assertEqual(rect_area_rule.compute(bindings), {"?a": 200})

    def test_database_backed_engine_ask(self):
        # Save all demo vials to database
        kb = build_engine()
        for rule in kb.vials["geometry"].rules:
            if rule.name == "rectangle-area":
                object.__setattr__(rule, "_compute_body", "lambda b: {'?a': b['?w'] * b['?h']}")
                
        for vial in kb.vials.values():
            self.store.save_vial(vial)
            
        # Create a database-backed engine (initially has NO vials in memory)
        db_engine = Engine(store=self.store)
        self.assertEqual(len(db_engine.vials), 0)
        
        # Ask socrates question (should activate philosophers and biology)
        result = db_engine.ask(("socrates", "is_mortal", "?answer"))
        self.assertEqual(len(result.answers), 1)
        self.assertEqual(result.answers[0].bindings["?answer"], True)
        
        # Verify that only the necessary vials were loaded into engine memory
        self.assertIn("philosophers", db_engine.vials)
        self.assertIn("biology", db_engine.vials)
        self.assertNotIn("geometry", db_engine.vials)
        self.assertNotIn("measurements", db_engine.vials)
        
        # Ask geometry question
        result2 = db_engine.ask(("courtyard", "area", "?a"))
        self.assertEqual(len(result2.answers), 1)
        self.assertEqual(result2.answers[0].bindings["?a"], 360)
        
        # Now geometry and measurements should also be loaded
        self.assertIn("geometry", db_engine.vials)
        self.assertIn("measurements", db_engine.vials)


if __name__ == "__main__":
    unittest.main()

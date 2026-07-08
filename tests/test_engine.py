import unittest

from dsce.demo_kb import build_engine
from dsce.engine import Engine
from dsce.facts import substitute, unify
from dsce.vial import Rule, Vial


class TestUnification(unittest.TestCase):
    def test_unify_binds_variables(self):
        b = unify(("?x", "is_a", "human"), ("socrates", "is_a", "human"), {})
        self.assertEqual(b, {"?x": "socrates"})

    def test_unify_respects_existing_bindings(self):
        self.assertIsNone(
            unify(("?x", "is_a", "human"), ("plato", "is_a", "human"), {"?x": "socrates"})
        )

    def test_unify_rejects_constant_mismatch(self):
        self.assertIsNone(unify(("socrates", "is_a", "god"), ("socrates", "is_a", "human"), {}))

    def test_substitute_raises_on_unbound(self):
        with self.assertRaises(ValueError):
            substitute(("?x", "area", "?a"), {"?x": "courtyard"})


class TestFlood(unittest.TestCase):
    def setUp(self):
        self.engine = build_engine()

    def test_multi_vial_inference_chain(self):
        result = self.engine.ask(("socrates", "is_mortal", "?answer"))
        self.assertEqual(len(result.answers), 1)
        answer = result.answers[0]
        self.assertEqual(answer.bindings["?answer"], True)
        proof = answer.proof.render()
        self.assertIn("mammals-are-mortal", proof)
        self.assertIn("humans-are-mammals", proof)
        self.assertIn("axiom in vial 'philosophers'", proof)

    def test_deterministic_computation(self):
        result = self.engine.ask(("courtyard", "area", "?a"))
        self.assertEqual(len(result.answers), 1)
        self.assertEqual(result.answers[0].bindings["?a"], 360)

    def test_derived_facts_feed_further_rules(self):
        # plaza is a square; area requires deriving rectangle-ness, width and
        # height first, then computing 25 * 25.
        result = self.engine.ask(("plaza", "area", "?a"))
        self.assertEqual(len(result.answers), 1)
        self.assertEqual(result.answers[0].bindings["?a"], 625)

    def test_sparse_activation(self):
        # A geometry query must not wake the philosophy or biology vials.
        result = self.engine.ask(("courtyard", "area", "?a"))
        self.assertIn("philosophers", result.dormant)
        self.assertIn("biology", result.dormant)

    def test_determinism(self):
        goal = ("?who", "is_a", "mammal")
        first = build_engine().ask(goal)
        second = build_engine().ask(goal)
        self.assertEqual(first.summary(), second.summary())

    def test_confidence_propagates(self):
        result = self.engine.ask(("socrates", "is_mortal", "?answer"))
        # 0.999 (rule) * 0.99 (philosophers vial axiom) along the chain.
        self.assertAlmostEqual(result.answers[0].confidence, 0.999 * 0.99, places=6)

    def test_no_proof_found(self):
        result = self.engine.ask(("zeus", "is_mortal", "?answer"))
        self.assertEqual(result.answers, [])

    def test_variable_subject_enumerates(self):
        result = self.engine.ask(("?who", "is_a", "mammal"))
        who = sorted(a.bindings["?who"] for a in result.answers)
        self.assertEqual(who, ["plato", "socrates"])


class TestEngineBasics(unittest.TestCase):
    def test_duplicate_vial_id_rejected(self):
        engine = Engine()
        engine.add_vial(Vial(id="a", concept="a"))
        with self.assertRaises(ValueError):
            engine.add_vial(Vial(id="a", concept="a again"))

    def test_tick_budget_halts_flood(self):
        engine = Engine(max_ticks=3)
        engine.add_vial(Vial(
            id="counter",
            concept="unbounded counting",
            facts=(("n", "value", 0),),
            rules=(Rule(
                name="successor",
                premises=(("n", "value", "?v"),),
                conclusion=("n", "value", "?next"),
                compute=lambda b: {"?next": b["?v"] + 1},
            ),),
        ))
        result = engine.ask(("n", "value", "?v"))
        self.assertEqual(result.ticks, 3)  # halted by budget, not by fixpoint


if __name__ == "__main__":
    unittest.main()

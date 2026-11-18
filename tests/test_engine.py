# Program: DSCE - test_engine.py
# Programmer: Ethan-da-Tech-Wizard (written with Claude, an AI assistant)
# Last Modified: 2026-11-18
# Purpose: Unit tests for the DSCE engine: unification, flood inference, determinism, confidence and tick limits.
"""Unit tests for the DSCE engine: unification, flood inference, determinism, confidence and tick limits."""

import unittest

from dsce.demo_kb import build_engine
from dsce.engine import Engine
from dsce.facts import substitute, unify
from dsce.vial import Rule, Vial


class TestUnification(unittest.TestCase):
    """Unit tests for pattern unification and substitution."""
    def test_unify_binds_variables(self):
        """A variable in the pattern gets bound to the fact's value."""
        b = unify(("?x", "is_a", "human"), ("socrates", "is_a", "human"), {})
        self.assertEqual(b, {"?x": "socrates"})

    def test_unify_respects_existing_bindings(self):
        """A variable already bound to a different value makes unification fail."""
        self.assertIsNone(
            unify(("?x", "is_a", "human"), ("plato", "is_a", "human"), {"?x": "socrates"})
        )

    def test_unify_rejects_constant_mismatch(self):
        """A differing constant makes unification fail."""
        self.assertIsNone(unify(("socrates", "is_a", "god"), ("socrates", "is_a", "human"), {}))

    def test_substitute_raises_on_unbound(self):
        """Substituting with a missing binding raises ValueError."""
        with self.assertRaises(ValueError):
            substitute(("?x", "area", "?a"), {"?x": "courtyard"})


class TestFlood(unittest.TestCase):
    """End-to-end tests of the flood engine against the demo knowledge base."""
    def setUp(self):
        """Build a fresh demo engine for each test."""
        self.engine = build_engine()

    def test_multi_vial_inference_chain(self):
        """The mortality answer chains across vials and cites the expected rules and axiom."""
        result = self.engine.ask(("socrates", "is_mortal", "?answer"))
        self.assertEqual(len(result.answers), 1)
        answer = result.answers[0]
        self.assertEqual(answer.bindings["?answer"], True)
        proof = answer.proof.render()
        self.assertIn("mammals-are-mortal", proof)
        self.assertIn("humans-are-mammals", proof)
        self.assertIn("axiom in vial 'philosophers'", proof)

    def test_deterministic_computation(self):
        """A compute hook yields the courtyard area of 360."""
        result = self.engine.ask(("courtyard", "area", "?a"))
        self.assertEqual(len(result.answers), 1)
        self.assertEqual(result.answers[0].bindings["?a"], 360)

    def test_derived_facts_feed_further_rules(self):
        """Derived facts feed later rules, giving the plaza area of 625."""
        # plaza is a square; area requires deriving rectangle-ness, width and
        # height first, then computing 25 * 25.
        result = self.engine.ask(("plaza", "area", "?a"))
        self.assertEqual(len(result.answers), 1)
        self.assertEqual(result.answers[0].bindings["?a"], 625)

    def test_sparse_activation(self):
        """A geometry query leaves unrelated vials dormant."""
        # A geometry query must not wake the philosophy or biology vials.
        result = self.engine.ask(("courtyard", "area", "?a"))
        self.assertIn("philosophers", result.dormant)
        self.assertIn("biology", result.dormant)

    def test_determinism(self):
        """Two separate runs of the same query produce identical summaries."""
        goal = ("?who", "is_a", "mammal")
        first = build_engine().ask(goal)
        second = build_engine().ask(goal)
        self.assertEqual(first.summary(), second.summary())

    def test_confidence_propagates(self):
        """Rule and vial confidences multiply along the proof chain."""
        result = self.engine.ask(("socrates", "is_mortal", "?answer"))
        # 0.999 (rule) * 0.99 (philosophers vial axiom) along the chain.
        self.assertAlmostEqual(result.answers[0].confidence, 0.999 * 0.99, places=6)

    def test_no_proof_found(self):
        """An unknown subject yields no answers."""
        result = self.engine.ask(("zeus", "is_mortal", "?answer"))
        self.assertEqual(result.answers, [])

    def test_variable_subject_enumerates(self):
        """A variable subject enumerates every matching entity."""
        result = self.engine.ask(("?who", "is_a", "mammal"))
        who = sorted(a.bindings["?who"] for a in result.answers)
        self.assertEqual(who, ["plato", "socrates"])


class TestEngineBasics(unittest.TestCase):
    """Tests of basic engine bookkeeping and safety limits."""
    def test_duplicate_vial_id_rejected(self):
        """Adding two vials with the same id raises ValueError."""
        engine = Engine()
        engine.add_vial(Vial(id="a", concept="a"))
        with self.assertRaises(ValueError):
            engine.add_vial(Vial(id="a", concept="a again"))

    def test_tick_budget_halts_flood(self):
        """A never-ending rule is stopped by the tick budget."""
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

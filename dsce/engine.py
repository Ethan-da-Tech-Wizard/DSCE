"""The DSCE flood engine.

Reasoning proceeds in discrete ticks:

  1. A query is parsed into a goal pattern. Its constants become the first
     sand grains.
  2. Each tick, grains wake up every dormant vial indexed under their term,
     and firing vials wake their declared neighbors.
  3. Activated vials pour their axiom facts into shared working memory and
     fire their rules against everything derived so far. Each new fact
     emits fresh grains carrying its terms.
  4. The flood stops at fixpoint (no new facts, no new vials) or when the
     tick budget runs out. Answers are all working-memory facts matching
     the goal, each with a full proof tree.

Everything is iterated in sorted order and no randomness is used, so the
same knowledge base and query always produce the identical result — the
"deterministic" in DSCE.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from dsce.facts import Bindings, Fact, Pattern, constants, sort_key, substitute, unify
from dsce.proof import Derivation, Proof, fact_str
from dsce.sand import Grain
from dsce.vial import Rule, Vial


@dataclass(frozen=True)
class Answer:
    fact: Fact
    bindings: dict
    proof: Proof

    @property
    def confidence(self) -> float:
        return self.proof.confidence


@dataclass
class Result:
    goal: Pattern
    answers: list
    ticks: int
    activated: tuple  # vial ids that received sand, in activation order
    dormant: tuple  # vial ids the flood never reached
    grains: int  # total sand grains emitted
    facts_derived: int  # size of working memory at fixpoint

    def summary(self) -> str:
        lines = [
            f"goal: {fact_str(self.goal)}",
            f"flood: {self.ticks} tick(s), {self.grains} grain(s) of sand, "
            f"{len(self.activated)}/{len(self.activated) + len(self.dormant)} vials activated, "
            f"{self.facts_derived} fact(s) in working memory",
            f"activated vials: {', '.join(self.activated) if self.activated else '(none)'}",
            f"dormant vials:   {', '.join(self.dormant) if self.dormant else '(none)'}",
        ]
        if not self.answers:
            lines.append("no proof found.")
        for i, answer in enumerate(self.answers, 1):
            lines.append(f"answer {i} (confidence {answer.confidence:.3f}):")
            lines.append(answer.proof.render())
        return "\n".join(lines)


class Engine:
    def __init__(self, max_ticks: int = 50):
        self.max_ticks = max_ticks
        self.vials: dict = {}  # id -> Vial
        self._term_index: dict = {}  # term -> sorted tuple of vial ids

    def add_vial(self, vial: Vial) -> None:
        if vial.id in self.vials:
            raise ValueError(f"duplicate vial id {vial.id!r}")
        self.vials[vial.id] = vial
        self._term_index = {}  # rebuilt lazily

    def _index(self) -> dict:
        if not self._term_index:
            index: dict = {}
            for vial_id in sorted(self.vials):
                for term in self.vials[vial_id].terms():
                    index.setdefault(term, []).append(vial_id)
            self._term_index = {term: tuple(ids) for term, ids in index.items()}
        return self._term_index

    def ask(self, goal: Pattern) -> Result:
        index = self._index()
        wm: dict = {}  # Fact -> Derivation (working memory, insertion-ordered)
        active: dict = {}  # vial id -> None, used as an ordered set
        grains: list = [Grain(term, "query", 0) for term in constants(goal)]
        total_grains = len(grains)
        ticks = 0

        for tick in range(1, self.max_ticks + 1):
            # Sand wakes dormant vials; firing vials wake their neighbors.
            newly_active = []
            for grain in grains:
                for vial_id in index.get(grain.term, ()):
                    if vial_id not in active:
                        active[vial_id] = None
                        newly_active.append(vial_id)
            for vial_id in list(newly_active):
                for neighbor in self.vials[vial_id].neighbors:
                    if neighbor in self.vials and neighbor not in active:
                        active[neighbor] = None
                        newly_active.append(neighbor)

            # Newly active vials pour in their axioms.
            new_facts = []
            for vial_id in newly_active:
                vial = self.vials[vial_id]
                for fact in sorted(vial.facts, key=sort_key):
                    if fact not in wm:
                        wm[fact] = Derivation(
                            fact=fact,
                            vial_id=vial_id,
                            confidence=vial.confidence,
                            evidence=vial.evidence,
                        )
                        new_facts.append(fact)

            # Every active vial fires its rules against working memory.
            for vial_id in active:
                vial = self.vials[vial_id]
                for rule in vial.rules:
                    for bindings in self._match(rule.premises, wm):
                        fact, derivation = self._conclude(rule, vial, bindings, wm)
                        if fact is not None and fact not in wm:
                            wm[fact] = derivation
                            new_facts.append(fact)

            # New facts become new sand. The predicate position is skipped:
            # predicates are relations, not entities, and generic ones like
            # "is_a" would otherwise flood every vial in the network.
            grains = []
            for fact in new_facts:
                subject, _, obj = fact
                grains.append(Grain(subject, wm[fact].vial_id, tick))
                grains.append(Grain(obj, wm[fact].vial_id, tick))
            total_grains += len(grains)
            ticks = tick

            if not new_facts and not newly_active:
                break  # fixpoint: the flood has settled

        answers = []
        for fact in sorted(wm, key=sort_key):
            bindings = unify(goal, fact, {})
            if bindings is not None:
                answers.append(Answer(fact=fact, bindings=bindings, proof=Proof(fact, wm)))
        dormant = tuple(v for v in sorted(self.vials) if v not in active)
        return Result(
            goal=goal,
            answers=answers,
            ticks=ticks,
            activated=tuple(active),
            dormant=dormant,
            grains=total_grains,
            facts_derived=len(wm),
        )

    def _match(self, premises: tuple, wm: dict):
        """All binding sets satisfying every premise, in deterministic order."""
        results = [{}]
        facts = sorted(wm, key=sort_key)
        for premise in premises:
            extended = []
            for bindings in results:
                for fact in facts:
                    unified = unify(premise, fact, bindings)
                    if unified is not None:
                        extended.append(unified)
            results = extended
            if not results:
                break
        return results

    def _conclude(self, rule: Rule, vial: Vial, bindings: Bindings, wm: dict):
        if rule.compute is not None:
            bindings = {**bindings, **rule.compute(bindings)}
        try:
            fact = substitute(rule.conclusion, bindings)
        except ValueError:
            return None, None
        premises = tuple(substitute(p, bindings) for p in rule.premises)
        premise_confidence = min(wm[p].confidence for p in premises) if premises else 1.0
        return fact, Derivation(
            fact=fact,
            vial_id=vial.id,
            confidence=rule.confidence * vial.confidence * premise_confidence,
            rule_name=rule.name,
            premises=premises,
        )

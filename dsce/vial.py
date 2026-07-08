"""Vials: the knowledge containers of the DSCE.

A vial holds a small, coherent body of knowledge about one concept:
ground facts, inference rules, evidence sources, a confidence value,
and links to neighboring vials. Vials stay dormant until sand reaches
them; only activated vials contribute to a proof.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Callable, Optional

from dsce.facts import Bindings, Fact, Pattern, constants


@dataclass(frozen=True)
class Rule:
    """An inference rule: if all premises hold, the conclusion holds.

    `compute` optionally derives extra bindings deterministically from the
    matched premises (e.g. arithmetic), letting the conclusion contain a
    variable that no premise binds — this is the "computation" in DSCE.
    """

    name: str
    premises: tuple  # tuple of Patterns
    conclusion: Pattern
    compute: Optional[Callable[[Bindings], Bindings]] = None
    confidence: float = 1.0


@dataclass
class Vial:
    id: str
    concept: str
    facts: tuple = ()  # tuple of ground Facts (axioms)
    rules: tuple = ()  # tuple of Rules
    neighbors: tuple = ()  # ids of related vials, activated when this one fires
    evidence: tuple = ()  # human-readable sources backing this vial
    confidence: float = 1.0

    def terms(self) -> set:
        """Every constant this vial knows about — used to index it, so that
        sand grains carrying one of these terms can find and activate it."""
        found = set()
        for fact in self.facts:
            found.update(fact)
        for rule in self.rules:
            for premise in rule.premises:
                found.update(constants(premise))
            found.update(constants(rule.conclusion))
        return found

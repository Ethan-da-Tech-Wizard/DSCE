"""Proof construction and rendering.

Every fact in working memory records how it got there: either it is an
axiom read out of a vial, or it was derived by a rule from earlier facts.
Walking those records backwards from the answer yields a complete,
inspectable proof tree — the DSCE's replacement for opaque hidden states.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

from dsce.facts import Fact


@dataclass(frozen=True)
class Derivation:
    fact: Fact
    vial_id: str
    confidence: float
    rule_name: Optional[str] = None  # None for axioms
    premises: tuple = ()  # the premise Facts a rule consumed
    evidence: tuple = ()  # sources, for axioms


def fact_str(fact: Fact) -> str:
    return "(" + " ".join(str(t) for t in fact) + ")"


class Proof:
    """A proof tree rooted at one derived fact."""

    def __init__(self, root: Fact, derivations: dict):
        self.root = root
        self.derivations = derivations  # Fact -> Derivation, shared working memory

    @property
    def confidence(self) -> float:
        return self.derivations[self.root].confidence

    def render(self) -> str:
        lines = []
        self._render(self.root, prefix="", is_last=True, is_root=True, lines=lines)
        return "\n".join(lines)

    def _render(self, fact: Fact, prefix: str, is_last: bool, is_root: bool, lines: list):
        d = self.derivations[fact]
        if d.rule_name is None:
            because = f"axiom in vial '{d.vial_id}'"
            if d.evidence:
                because += f" (evidence: {', '.join(d.evidence)})"
        else:
            because = f"by rule '{d.rule_name}' in vial '{d.vial_id}'"
        connector = "" if is_root else ("└─ " if is_last else "├─ ")
        lines.append(f"{prefix}{connector}{fact_str(fact)}  [{because}, confidence {d.confidence:.3f}]")
        child_prefix = prefix if is_root else prefix + ("   " if is_last else "│  ")
        for i, premise in enumerate(d.premises):
            self._render(premise, child_prefix, i == len(d.premises) - 1, False, lines)

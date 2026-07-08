"""Proof construction and rendering — how the DSCE shows its work.

THE KEY DESIGN DECISION in this file: proofs are not built as a separate
step after reasoning. They fall out of bookkeeping the engine does anyway.
Every fact that enters working memory is stored alongside a Derivation
record answering "how do I know this?":

    - AXIOM:   "vial V asserted it directly" (+ V's evidence sources), or
    - DERIVED: "rule R in vial V concluded it from these premise facts".

Because each derived fact's premises are themselves facts in working
memory (with their own Derivation records), walking the records backwards
from any answer reconstructs the complete reasoning chain, ending at cited
axioms. That walk IS the proof tree — the DSCE's replacement for a neural
network's opaque hidden states.

Rendering example (from `python -m dsce socrates is_mortal ?x`):

    (socrates is_mortal True)  [by rule 'mammals-are-mortal' in vial 'biology', confidence 0.989]
    └─ (socrates is_a mammal)  [by rule 'humans-are-mammals' in vial 'biology', confidence 0.990]
       └─ (socrates is_a human)  [axiom in vial 'philosophers' (evidence: Plato, Apology, ...), confidence 0.990]

Read bottom-up: an axiom with its source, each rule that consumed it, and
the final answer — every step attributable, every step checkable.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

from dsce.facts import Fact


@dataclass(frozen=True)
class Derivation:
    """The "how do I know this?" record attached to every working-memory fact.

    Fields:
        fact        the ground triple this record explains.
        vial_id     the vial responsible: home of the axiom, or home of the
                    rule that fired.
        confidence  ALREADY-COMBINED confidence of the whole chain below
                    this fact (rule x vial x weakest premise), computed at
                    derivation time by Engine._conclude. Nothing needs to
                    re-walk the tree to know how trustworthy a fact is.
        rule_name   name of the rule that derived it, or None -> axiom.
        premises    the ground premise facts the rule consumed. These are
                    the proof tree's child edges. Empty for axioms.
        evidence    human-readable sources, carried only by axioms (derived
                    facts point at their premises instead).
    """

    fact: Fact
    vial_id: str
    confidence: float
    rule_name: Optional[str] = None
    premises: tuple = ()
    evidence: tuple = ()


def fact_str(fact: Fact) -> str:
    """Render a triple as '(subject predicate object)'."""
    return "(" + " ".join(str(t) for t in fact) + ")"


class Proof:
    """A proof tree rooted at one derived fact.

    Deliberately lightweight: it does NOT copy the reasoning chain. It
    holds the root fact plus a reference to the engine's working memory
    (the Fact -> Derivation dict), and materializes the tree only when
    rendered, by following premises recursively. Many Proof objects for
    many answers all share the same underlying dict.
    """

    def __init__(self, root: Fact, derivations: dict):
        self.root = root
        self.derivations = derivations  # Fact -> Derivation, shared working memory

    @property
    def confidence(self) -> float:
        """Confidence of the root (already includes the whole chain)."""
        return self.derivations[self.root].confidence

    def render(self) -> str:
        """The proof tree as indented text with box-drawing connectors."""
        lines = []
        self._render(self.root, prefix="", is_last=True, is_root=True, lines=lines)
        return "\n".join(lines)

    def _render(self, fact: Fact, prefix: str, is_last: bool, is_root: bool, lines: list):
        """Recursive worker for render(); one call renders one node.

        Layout bookkeeping (standard tree-drawing technique):
          prefix   the accumulated indentation inherited from ancestors —
                   either spaces (under a last child) or '│  ' (under a
                   non-last child, so the vertical rail continues down to
                   its remaining siblings).
          is_last  whether this node is its parent's final premise; picks
                   '└─ ' vs '├─ ' as this node's connector.
          is_root  the root gets no connector and adds no indentation.
        """
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

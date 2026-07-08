"""Vials: the knowledge containers of the DSCE.

A vial is the unit of knowledge STORAGE, the way a parameter matrix is the
unit of storage in a neural network — except a vial is explicit, readable,
and citable. Each vial holds a small, coherent body of knowledge about one
concept:

    facts       ground axioms this vial asserts outright
    rules       "if premises then conclusion" inference steps
    neighbors   ids of related vials, woken alongside this one
    evidence    human-readable sources backing this vial's content
    confidence  how much the vial as a whole is trusted (0.0 - 1.0)

Vials stay DORMANT until sand reaches them; only activated vials contribute
facts and fire rules. This is the sparsity claim of the architecture: a
geometry question never pays for the philosophy vial.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Callable, Optional

from dsce.facts import Bindings, Fact, Pattern, constants


@dataclass(frozen=True)
class Rule:
    """An inference rule: if ALL premises hold, the conclusion holds.

    Fields:
        name        identifier shown in proof traces, e.g. "rectangle-area".
        premises    tuple of Patterns that must ALL unify against working
                    memory (sharing variables: "?x" in one premise is the
                    same "?x" in the others).
        conclusion  Pattern instantiated with the matched bindings to
                    produce the new fact.
        compute     OPTIONAL deterministic function from bindings to EXTRA
                    bindings. This is the "computation" in DSCE: it lets a
                    conclusion contain a value no premise supplied. E.g.
                    the rectangle-area rule matches ?w and ?h from premises
                    and computes {"?a": ?w * ?h}, so the conclusion
                    ("?r", "area", "?a") becomes ("courtyard", "area", 360).
                    MUST be pure and deterministic — no randomness, no I/O,
                    no reading external state — or the engine's determinism
                    guarantee is broken.
        confidence  how much this rule itself is trusted; multiplied into
                    the confidence of every fact it derives.

    Frozen (immutable) on purpose: rules are shared knowledge, and nothing
    should be able to mutate one mid-flood.
    """

    name: str
    premises: tuple  # tuple of Patterns
    conclusion: Pattern
    compute: Optional[Callable[[Bindings], Bindings]] = None
    confidence: float = 1.0


@dataclass
class Vial:
    """One knowledge container. See the module docstring for the concept.

    Fields:
        id          unique key, used in indexes, proofs, and neighbor links.
        concept     human-readable one-liner of what this vial is about.
        facts       tuple of ground Facts (axioms) poured into working
                    memory when the vial activates.
        rules       tuple of Rules fired every tick while the vial is active.
        neighbors   ids of related vials. When THIS vial wakes, its
                    neighbors wake too (transitively, within the same
                    tick). This encodes "if you're thinking about X you
                    will probably need Y" — e.g. measurements -> geometry.
        evidence    human-readable sources ("Euclid, Elements, Book I");
                    attached to every axiom this vial contributes, so they
                    appear in proof traces.
        confidence  trust in this vial overall; axioms inherit it, and it
                    discounts every rule firing from this vial.
    """

    id: str
    concept: str
    facts: tuple = ()
    rules: tuple = ()
    neighbors: tuple = ()
    evidence: tuple = ()
    confidence: float = 1.0

    def terms(self) -> set:
        """Every constant this vial mentions anywhere — its 'address'.

        The engine indexes vials under these terms; a sand grain carrying
        one of them will find and wake this vial. Collected from:
          - every position of every axiom fact, and
          - every CONSTANT (non-variable) position of every rule premise
            and conclusion (variables are placeholders, not addresses).
        """
        found = set()
        for fact in self.facts:
            found.update(fact)
        for rule in self.rules:
            for premise in rule.premises:
                found.update(constants(premise))
            found.update(constants(rule.conclusion))
        return found

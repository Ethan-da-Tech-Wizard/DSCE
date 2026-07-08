"""Sand: the activation particles of the DSCE.

Sand is NOT data — it is activation, the thing that decides which knowledge
participates in answering a given question. Where a transformer multiplies
every weight for every token, the DSCE lets activation FLOW: grains spread
outward from the question through the vial network, and only vials a grain
actually reaches wake up and do work.

Each grain carries exactly one TERM — a constant such as "socrates" or
"rectangle" — plus provenance (who emitted it, and when). Grains come from
two places:

    1. SEEDING: the constants of the goal pattern itself (origin "query",
       tick 0). Asking about socrates injects a "socrates" grain.
    2. DERIVATION: every new fact emits grains for its SUBJECT and OBJECT
       (origin = the vial whose rule/axiom produced the fact). The
       PREDICATE position is deliberately NOT emitted: predicates are
       relations, not entities, and generic ones like "is_a" appear in
       nearly every vial — letting them carry sand floods the entire
       network and destroys sparsity (this was observed during development
       and is pinned down by test_sparse_activation).

A grain does one job: its term is looked up in the engine's term->vials
index, and any dormant vial found there is woken. Grains live for exactly
one tick; facts persist, sand does not.

Frozen (immutable) dataclass: grains are records of something that
happened, and history should not be editable.
"""

from __future__ import annotations

from dataclasses import dataclass

from dsce.facts import Term


@dataclass(frozen=True)
class Grain:
    term: Term   # the single constant this grain carries
    origin: str  # "query", or the id of the vial whose derivation emitted it
    tick: int    # flood round on which this grain was created (0 = seeding)

    def __str__(self) -> str:
        return f"grain({self.term!r} from {self.origin} @ tick {self.tick})"

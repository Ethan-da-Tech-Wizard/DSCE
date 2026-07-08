"""Sand: the activation particles of the DSCE.

Sand is not data — it is activation. Each grain carries a single term
(a constant mentioned by the query or by a newly derived fact) and a
record of where it came from. Grains flow through the vial network and
wake up any dormant vial indexed under their term. Vials that never
receive sand never participate: reasoning stays sparse.
"""

from __future__ import annotations

from dataclasses import dataclass

from dsce.facts import Term


@dataclass(frozen=True)
class Grain:
    term: Term  # the constant this grain carries
    origin: str  # "query", or the id of the vial whose derivation emitted it
    tick: int  # flood cycle on which this grain was created

    def __str__(self) -> str:
        return f"grain({self.term!r} from {self.origin} @ tick {self.tick})"

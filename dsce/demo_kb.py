"""A small demonstration knowledge base.

Four vials spanning two unrelated domains, so that queries visibly
activate only the vials they need and leave the rest dormant.
"""

from __future__ import annotations

from dsce.engine import Engine
from dsce.vial import Rule, Vial


def build_engine() -> Engine:
    engine = Engine()

    engine.add_vial(Vial(
        id="philosophers",
        concept="Classical philosophers",
        facts=(
            ("socrates", "is_a", "human"),
            ("plato", "is_a", "human"),
            ("plato", "student_of", "socrates"),
        ),
        neighbors=("biology",),
        evidence=("Plato, Apology", "Diogenes Laertius, Lives"),
        confidence=0.99,
    ))

    engine.add_vial(Vial(
        id="biology",
        concept="Basic biology",
        rules=(
            Rule(
                name="humans-are-mammals",
                premises=(("?x", "is_a", "human"),),
                conclusion=("?x", "is_a", "mammal"),
            ),
            Rule(
                name="mammals-are-mortal",
                premises=(("?x", "is_a", "mammal"),),
                conclusion=("?x", "is_mortal", True),
                confidence=0.999,
            ),
        ),
        evidence=("Campbell Biology, ch. 32",),
    ))

    engine.add_vial(Vial(
        id="geometry",
        concept="Plane geometry",
        rules=(
            Rule(
                name="squares-are-rectangles",
                premises=(
                    ("?s", "is_a", "square"),
                    ("?s", "side", "?len"),
                ),
                conclusion=("?s", "is_a", "rectangle"),
            ),
            Rule(
                name="square-sides-give-width-and-height",
                premises=(
                    ("?s", "is_a", "square"),
                    ("?s", "side", "?len"),
                ),
                conclusion=("?s", "width", "?len"),
            ),
            Rule(
                name="square-sides-give-height",
                premises=(
                    ("?s", "is_a", "square"),
                    ("?s", "side", "?len"),
                ),
                conclusion=("?s", "height", "?len"),
            ),
            Rule(
                name="rectangle-area",
                premises=(
                    ("?r", "is_a", "rectangle"),
                    ("?r", "width", "?w"),
                    ("?r", "height", "?h"),
                ),
                conclusion=("?r", "area", "?a"),
                compute=lambda b: {"?a": b["?w"] * b["?h"]},
            ),
        ),
        evidence=("Euclid, Elements, Book I",),
    ))

    engine.add_vial(Vial(
        id="measurements",
        concept="Surveyed measurements",
        facts=(
            ("courtyard", "is_a", "rectangle"),
            ("courtyard", "width", 12),
            ("courtyard", "height", 30),
            ("plaza", "is_a", "square"),
            ("plaza", "side", 25),
        ),
        neighbors=("geometry",),
        evidence=("site survey 2026-03",),
        confidence=0.97,
    ))

    return engine

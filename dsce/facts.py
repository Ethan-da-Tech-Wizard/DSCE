"""Facts, patterns, and unification — the vocabulary of the whole engine.

EVERYTHING the DSCE knows or asks is expressed as a triple:

    (subject, predicate, object)

  - A FACT is a "ground" triple: all three positions are concrete values.
        ("socrates", "is_a", "human")
        ("courtyard", "width", 12)
  - A PATTERN is a triple that may contain VARIABLES — strings starting
    with "?" — standing for "any value, to be determined":
        ("?x", "is_a", "human")        "anything that is a human"
        ("courtyard", "area", "?a")    "whatever courtyard's area is"

Queries are patterns. Rule premises and conclusions are patterns. Matching
a pattern against a fact is called UNIFICATION, and the record of what each
variable turned out to equal is a BINDINGS dict, e.g. {"?x": "socrates"}.

This module is deliberately tiny and dependency-free: four pure functions
and some type aliases. Everything else in the package builds on it.
"""

from __future__ import annotations

from typing import Optional, Union

# Type aliases — for readability only; Python does not enforce them.
Term = Union[str, int, float, bool]  # one position of a triple
Fact = tuple      # ground triple of Terms (no variables)
Pattern = tuple   # triple of Terms, possibly containing variables
Bindings = dict   # variable name -> the value it is bound to


def is_variable(term: Term) -> bool:
    """A variable is any string starting with '?' (e.g. '?x', '?answer').

    Everything else — other strings, ints, floats, bools — is a constant.
    """
    return isinstance(term, str) and term.startswith("?")


def unify(pattern: Pattern, fact: Fact, bindings: Bindings) -> Optional[Bindings]:
    """Try to match `pattern` against ground `fact`, extending `bindings`.

    Walks the three positions in parallel. At each position:
      - pattern has a CONSTANT  -> it must equal the fact's value exactly,
                                   otherwise the match fails (return None).
      - pattern has a VARIABLE  -> if the variable is already bound (from a
                                   previous premise, say), its bound value
                                   must equal the fact's value; if unbound,
                                   it becomes bound to the fact's value.

    Returns the EXTENDED bindings dict on success, None on failure. The
    input `bindings` is copied, never mutated — callers rely on being able
    to try many facts against the same starting bindings.

    Examples:
        unify(("?x", "is_a", "human"), ("socrates", "is_a", "human"), {})
            -> {"?x": "socrates"}
        unify(("?x", "is_a", "human"), ("plato", "is_a", "human"),
              {"?x": "socrates"})
            -> None   (?x is already socrates; plato contradicts it)
    """
    result = dict(bindings)
    for p, f in zip(pattern, fact):
        if is_variable(p):
            if p in result:
                if result[p] != f:
                    return None  # variable already bound to something else
            else:
                result[p] = f  # bind the variable now
        elif p != f:
            return None  # constant mismatch
    return result


def substitute(pattern: Pattern, bindings: Bindings) -> Fact:
    """Turn a pattern into a ground fact by filling in every variable.

    E.g. substitute(("?r", "area", "?a"), {"?r": "courtyard", "?a": 360})
         -> ("courtyard", "area", 360)

    Raises ValueError if any variable has no binding — the engine treats
    that as "this rule cannot conclude anything for this match" and skips
    it rather than producing a half-ground fact.
    """
    out = []
    for term in pattern:
        if is_variable(term):
            if term not in bindings:
                raise ValueError(f"unbound variable {term} in {pattern}")
            out.append(bindings[term])
        else:
            out.append(term)
    return tuple(out)


def constants(pattern: Pattern) -> tuple:
    """The non-variable terms of a pattern, in position order.

    Used for seeding: the constants of a goal are what the first sand
    grains carry. constants(("socrates", "is_mortal", "?answer"))
    -> ("socrates", "is_mortal").
    """
    return tuple(term for term in pattern if not is_variable(term))


def sort_key(value) -> tuple:
    """A total order over mixed-type terms AND whole facts.

    Why this exists: determinism requires sorting collections that mix
    strings, numbers, bools, and tuples — but Python 3 refuses to compare
    across types (12 < "socrates" raises TypeError). This key maps every
    value to a comparable tuple:

        (type-rank, canonical-form)

    where numbers rank 0, strings rank 1, and tuples (facts) rank 2 and
    recurse into their elements. bool is handled before int/float because
    in Python `True == 1`, and repr() is used for numbers so 12 and 12.0
    stay distinguishable.
    """
    if isinstance(value, tuple):
        return (2, tuple(sort_key(v) for v in value))
    if isinstance(value, bool):
        return (0, "0" if not value else "1")
    if isinstance(value, (int, float)):
        return (0, repr(value))
    return (1, str(value))


def is_more_specific(term1: Term, term2: Term, wm: dict = None) -> bool:
    """Returns True if term1 contains more specific context or detail than term2.
    
    Checks both lexical sub-string containment and taxonomic is_a path in wm.
    """
    if not isinstance(term1, str) or not isinstance(term2, str):
        return False
    if term1 == term2:
        return False
    
    # 1. Lexical sub-string containment (e.g. "tower of pie in ajo arizona" contains "tower of pie")
    if term2.lower() in term1.lower():
        return True
        
    # 2. Taxonomic specialization (is_a paths in working memory)
    if wm is not None:
        visited = set()
        queue = [term1]
        while queue:
            curr = queue.pop(0)
            if curr in visited:
                continue
            visited.add(curr)
            if curr == term2:
                return True
            for fact in wm:
                if len(fact) == 3 and fact[0] == curr and fact[1] == "is_a":
                    queue.append(fact[2])
                    
    return False


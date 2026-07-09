"""Facts, patterns, and unification.

A fact is a ground triple: (subject, predicate, object).
A pattern is a triple that may contain variables, written as strings
starting with "?", e.g. ("?x", "is_a", "human").
"""

from __future__ import annotations

from typing import Optional, Union

Term = Union[str, int, float, bool]
Fact = tuple  # ground triple of Terms
Pattern = tuple  # triple of Terms, possibly containing variables
Bindings = dict


def is_variable(term: Term) -> bool:
    return isinstance(term, str) and term.startswith("?")


def unify(pattern: Pattern, fact: Fact, bindings: Bindings) -> Optional[Bindings]:
    """Match a pattern against a ground fact under existing bindings.

    Returns the extended bindings on success, or None on failure.
    Never mutates the bindings passed in.
    """
    result = dict(bindings)
    for p, f in zip(pattern, fact):
        if is_variable(p):
            if p in result:
                if result[p] != f:
                    return None
            else:
                result[p] = f
        elif p != f:
            return None
    return result


def substitute(pattern: Pattern, bindings: Bindings) -> Fact:
    """Instantiate a pattern with bindings. Raises if any variable is unbound."""
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
    """The non-variable terms of a pattern, in order."""
    return tuple(term for term in pattern if not is_variable(term))


def sort_key(value) -> tuple:
    """A total order over mixed-type terms/facts so iteration is deterministic."""
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


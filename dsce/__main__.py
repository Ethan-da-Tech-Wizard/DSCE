"""Run the DSCE demo:  python -m dsce  [subject predicate object]

With no arguments, runs a showcase of queries against the demo knowledge
base. With three arguments, asks that triple as a goal — use ?variables
for unknowns, e.g.:

    python -m dsce socrates is_mortal ?answer
    python -m dsce courtyard area ?a
    python -m dsce "?who" is_a mammal
"""

from __future__ import annotations

import sys

from dsce.demo_kb import build_engine


def parse_term(token: str):
    """Interpret one command-line token as a triple term.

    Order matters: "true"/"false" become booleans, then int is tried,
    then float, and anything left stays a string (including ?variables —
    the engine recognizes those by their leading '?', not here).
    """
    if token.lower() in ("true", "false"):
        return token.lower() == "true"
    try:
        return int(token)
    except ValueError:
        pass
    try:
        return float(token)
    except ValueError:
        pass
    return token


def main(argv: list) -> int:
    engine = build_engine()
    if len(argv) == 3:
        goals = [tuple(parse_term(t) for t in argv)]
    elif not argv:
        goals = [
            ("socrates", "is_mortal", "?answer"),
            ("courtyard", "area", "?a"),
            ("plaza", "area", "?a"),
            ("?who", "is_a", "mammal"),
        ]
    else:
        print(__doc__)
        return 2

    for i, goal in enumerate(goals):
        if i:
            print("\n" + "=" * 72 + "\n")
        print(engine.ask(goal).summary())
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

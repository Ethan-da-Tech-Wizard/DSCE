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
from dsce.engine import Engine
from dsce.db_store import SqliteVialStore


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
    db_path = None
    if argv and argv[0] == "--db":
        if len(argv) < 2:
            print("Error: --db flag requires a database path parameter.")
            return 2
        db_path = argv[1]
        argv = argv[2:]

    if db_path:
        store = SqliteVialStore(db_path)
        engine = Engine(store=store)
    else:
        engine = build_engine()

    if len(argv) == 3:
        goals = [tuple(parse_term(t) for t in argv)]
    elif not argv:
        if db_path:
            print("Error: When querying a database, you must specify a triple goal query, e.g.:")
            print("  python -m dsce --db dsce.sqlite modesto located_in ?where")
            return 2
        goals = [
            ("socrates", "is_mortal", "?answer"),
            ("courtyard", "area", "?a"),
            ("plaza", "area", "?a"),
            ("?who", "is_a", "mammal"),
        ]
    else:
        print("Run the DSCE demo or query a database:")
        print("  python -m dsce  [subject predicate object]")
        print("  python -m dsce --db <db_path> [subject predicate object]")
        return 2

    for i, goal in enumerate(goals):
        if i:
            print("\n" + "=" * 72 + "\n")
        print(engine.ask(goal).summary())
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))


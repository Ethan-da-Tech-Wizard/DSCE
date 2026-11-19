"""Regression tests for the eval() removal (db_store compute_body, gameboy_app calculator)."""
import ast
import pathlib
import sqlite3

import pytest

from dsce.safe_eval import safe_arith, safe_compute
from dsce.db_store import SqliteVialStore
from dsce.demo_kb import build_engine


def test_safe_compute_runs_arithmetic_lambda():
    """The stored-rule format used by existing vials still computes the same bindings."""
    f = safe_compute("lambda b: {'?a': b['?w'] * b['?h']}")
    assert f({"?w": 3, "?h": 4}) == {"?a": 12}


@pytest.mark.parametrize("body", [
    "__import__('os').system('true')",
    "lambda b: {'?a': __import__('os').getcwd()}",
    "lambda b: {'?a': b.keys()}",
    "lambda b: {'?a': 9**9**9}",
    "open('x','w')",
])
def test_safe_compute_rejects_code(body):
    """Anything beyond arithmetic on bindings is refused (ValueError) instead of executed."""
    with pytest.raises(ValueError):
        f = safe_compute(body)
        f({})


def test_safe_arith():
    """Calculator text evaluates; code does not."""
    assert safe_arith("2+3*4") == 14
    with pytest.raises(ValueError):
        safe_arith("__import__('os')")


def _gameboy_eval():
    """Extract safe_eval_arith from gameboy_app.py without importing it (it builds a Tk window)."""
    src = pathlib.Path(__file__).resolve().parents[1].joinpath("gameboy_app.py").read_text()
    fn = next(n for n in ast.parse(src).body
              if isinstance(n, ast.FunctionDef) and n.name == "safe_eval_arith")
    ns = {}
    exec(compile(ast.Module([fn], []), "gameboy_app", "exec"), ns)
    return ns["safe_eval_arith"]


def test_gameboy_calculator_is_safe():
    """The calculator display handler no longer calls eval()."""
    f = _gameboy_eval()
    assert f("7*6") == 42
    with pytest.raises(ValueError):
        f("__import__('os').system('true')")
    assert "eval(display" not in pathlib.Path("gameboy_app.py").read_text()


def test_load_vial_does_not_execute_stored_code(tmp_path):
    """A tampered compute_body in the database must not run when the vial is loaded."""
    marker = tmp_path / "pwned"
    db = tmp_path / "v.db"
    store = SqliteVialStore(str(db))
    store.save_vial(build_engine().vials["geometry"])
    con = sqlite3.connect(db)
    con.execute("UPDATE rules SET compute_body = ? WHERE name = 'rectangle-area'",
                (f"__import__('pathlib').Path({str(marker)!r}).write_text('x')",))
    con.commit()
    con.close()
    loaded = store.load_vial("geometry")
    assert not marker.exists()
    rule = next(r for r in loaded.rules if r.name == "rectangle-area")
    assert rule.compute is None

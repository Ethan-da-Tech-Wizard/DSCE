# ==============================================================================
# Program: safe_eval.py
# Programmer: Ethan-da-Tech-Wizard
# Purpose: Safe replacements for eval() on untrusted text. Walks a parsed `ast`
#          tree and executes only an allow-list of node types (numbers, + - * / //
#          % **, unary +/-, and one dict-returning lambda), so no names, calls,
#          attributes or imports can ever run.
# Used by: dsce/db_store.py (compute_body of stored rules).
# ==============================================================================
import ast
import operator

# Allowed binary and unary operators, mapped to the functions that implement them.
_BIN = {
    ast.Add: operator.add, ast.Sub: operator.sub, ast.Mult: operator.mul,
    ast.Div: operator.truediv, ast.FloorDiv: operator.floordiv,
    ast.Mod: operator.mod, ast.Pow: operator.pow,
}
_UNARY = {ast.UAdd: operator.pos, ast.USub: operator.neg}

# Exponents above this are refused so "9**9**9" cannot hang the process.
_MAX_EXPONENT = 1000


def _num(node, env):
    """Evaluate one arithmetic node.

    Args:
        node: an ast expression node.
        env: mapping of the lambda's single argument name to its dict value,
            or None when no lambda is in scope (plain calculator text).

    Returns:
        The numeric result.

    Raises:
        ValueError: for any node type outside the allow-list.
    """
    if isinstance(node, ast.Constant) and isinstance(node.value, (int, float)) \
            and not isinstance(node.value, bool):
        return node.value
    if isinstance(node, ast.BinOp) and type(node.op) in _BIN:
        left, right = _num(node.left, env), _num(node.right, env)
        # Cap the exponent: pow on huge ints is the one operator that can burn unbounded CPU.
        if isinstance(node.op, ast.Pow) and abs(right) > _MAX_EXPONENT:
            raise ValueError("exponent too large")
        return _BIN[type(node.op)](left, right)
    if isinstance(node, ast.UnaryOp) and type(node.op) in _UNARY:
        return _UNARY[type(node.op)](_num(node.operand, env))
    # arg['?name'] lookups are the only way a stored rule reads its bindings.
    if env is not None and isinstance(node, ast.Subscript) and isinstance(node.value, ast.Name) \
            and node.value.id == env[0] and isinstance(node.slice, ast.Constant):
        return env[1][node.slice.value]
    raise ValueError("unsupported expression: " + type(node).__name__)


def safe_arith(text):
    """Evaluate plain arithmetic text such as "2+3*4".

    Raises:
        ValueError / SyntaxError / ZeroDivisionError: on anything else.
    """
    return _num(ast.parse(text.strip(), mode="eval").body, None)


def safe_compute(body):
    """Turn stored text like "lambda b: {'?a': b['?w'] * b['?h']}" into a callable.

    The text is parsed and validated once; the returned function interprets the
    validated tree for each call, so nothing is ever compiled or exec'd.

    Returns:
        A function taking a bindings dict and returning a dict of new bindings.

    Raises:
        ValueError: if the text is not exactly one single-argument lambda returning
            a dict literal with constant keys and arithmetic values.
    """
    tree = ast.parse(body.strip(), mode="eval").body
    if not (isinstance(tree, ast.Lambda) and len(tree.args.args) == 1
            and not (tree.args.vararg or tree.args.kwarg or tree.args.kwonlyargs
                     or tree.args.defaults)
            and isinstance(tree.body, ast.Dict)):
        raise ValueError("compute_body must be a one-argument lambda returning a dict")
    arg = tree.args.args[0].arg
    keys = []
    for k in tree.body.keys:
        if not (isinstance(k, ast.Constant) and isinstance(k.value, str)):
            raise ValueError("dict keys must be string constants")
        keys.append(k.value)
    values = tree.body.values
    # Validate every value up front against a dummy binding so bad bodies fail at load time,
    # not in the middle of a query. Missing-key errors from the dummy are expected and ignored.
    for v in values:
        try:
            _num(v, (arg, {}))
        except KeyError:
            pass

    def compute(bindings):
        return {k: _num(v, (arg, bindings)) for k, v in zip(keys, values)}

    return compute

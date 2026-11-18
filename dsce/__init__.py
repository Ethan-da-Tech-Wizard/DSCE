# Program: DSCE - __init__.py
# Programmer: Ethan-da-Tech-Wizard (written with Claude, an AI assistant)
# Last Modified: 2026-11-18
# Purpose: Package entry point that re-exports the public DSCE classes (engine, vials, rules, sand, proofs and the SQLite store) and the version.
"""DSCE — Deterministic Semantic Computation Engine.

Knowledge lives in explicit containers ("vials"). Reasoning is performed by
flooding the vial network with activation particles ("sand") until enough
evidence accumulates to assemble a verifiable proof of the answer.

Original architecture proposed by The Author.
"""

from dsce.engine import Engine, Result, Answer
from dsce.vial import Vial, Rule
from dsce.sand import Grain
from dsce.proof import Proof
from dsce.db_store import SqliteVialStore

__version__ = "0.1.0"

__all__ = ["Engine", "Result", "Answer", "Vial", "Rule", "Grain", "Proof", "SqliteVialStore"]


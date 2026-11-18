# ==============================================================================
# Program: __init__.py
# Programmer: Ethan-da-Tech-Wizard
# Last Modified: November 18, 2026
# Purpose: Top-level package initialization for the Deterministic Semantic
#          Computation Engine (DSCE). Exports core engine entities: Engine,
#          Result, Answer, Vial, Rule, Grain, Proof, and SqliteVialStore.
# ==============================================================================

"""DSCE — Deterministic Semantic Computation Engine.

Knowledge lives in explicit containers ("vials"). Reasoning is performed by
flooding the vial network with activation particles ("sand") until enough
evidence accumulates to assemble a verifiable proof of the answer.
"""

from dsce.engine import Engine, Result, Answer
from dsce.vial import Vial, Rule
from dsce.sand import Grain
from dsce.proof import Proof
from dsce.db_store import SqliteVialStore

__version__ = "0.1.0"

__all__ = ["Engine", "Result", "Answer", "Vial", "Rule", "Grain", "Proof", "SqliteVialStore"]

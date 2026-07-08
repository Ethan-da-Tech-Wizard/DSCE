# DSCE Software Requirements Document (SRD)

**Version:** 0.1 · **Date:** 2026-07
**Scope:** technical/system requirements for the reference implementation.
**Traces to:** [FRD](FRD.md) behaviors, [PRD](PRD.md) goals.

## SR-A Architecture

- **SR-A1 Module decomposition.** The implementation SHALL keep the five
  concerns in separate modules with one-way dependencies:

  ```
  facts.py ← vial.py ← engine.py → proof.py
                ↑          ↑
             sand.py ──────┘
  ```
  `facts.py` (triples/unification) depends on nothing; `engine.py` is the
  only module that composes the others; demo and CLI sit on top.
- **SR-A2 Data/algorithm split.** Vials, grains, and derivations SHALL be
  passive dataclasses; all control flow lives in `Engine`.
- **SR-A3 Immutability of shared knowledge.** `Rule`, `Grain`, and
  `Derivation` SHALL be frozen dataclasses; nothing may mutate knowledge
  or history mid-flood.
- **SR-A4 Per-query isolation.** Working memory and the active set SHALL
  be local to one `ask()` call. `Engine` state between queries is limited
  to the vial registry and its derived term index.

## SR-D Determinism (implements FR-5)

- **SR-D1** Every iteration over a collection whose order can affect
  output SHALL be explicitly ordered: sorted vial ids in the index, facts
  sorted via `facts.sort_key` (which totally orders mixed types), and
  insertion-ordered dicts for WM/active-set.
- **SR-D2** `sort_key` SHALL order bools before being confused with ints,
  keep `12` and `12.0` distinguishable, and recurse into tuples.
- **SR-D3** `random`, `time`, `os`-entropy, `id()`-based ordering, and
  set iteration SHALL NOT appear in result-affecting paths.
- **SR-D4** `compute` hooks are part of the trusted computing base for
  determinism: the engine cannot verify their purity. Documentation SHALL
  state the purity contract (done: `vial.py` docstring); a lint/sandbox is
  future work (M4).

## SR-P Performance

- **SR-P1 (current, acceptable)** v0.1 targets correctness at demo scale:
  floods over tens of facts SHALL settle in milliseconds. Status: demo
  floods settle in 2 ticks, full test suite < 50 ms. ✅
- **SR-P2 (known limitation, must change at scale)** `Engine._match` is a
  naive join: for a rule with P premises over |WM| facts it explores up to
  **O(|WM|^P)** combinations, and rules are re-matched against the ENTIRE
  working memory every tick, re-deriving and discarding known facts.
  Consequences and remedy:
  - acceptable while |WM| ≤ ~10³ and P ≤ 3;
  - the M3 milestone SHALL replace it with (a) fact indexing by predicate
    and bound positions, and (b) **semi-naive evaluation** — joining each
    tick only against facts new since the previous tick — or a full
    Rete-style network with cached partial matches;
  - target after M3: 10⁵ facts, P ≤ 3, < 1 s per flood (PRD-7).
  This is risk **R-1** in the [Risk Register](RISK_REGISTER.md).
- **SR-P3 Termination.** Floods SHALL be bounded by `max_ticks`
  (default 50) so non-terminating rule sets (e.g. successor arithmetic)
  cannot hang the engine.

## SR-I Implementation constraints

- **SR-I1 Language/runtime.** Python ≥ 3.9, standard library only. No
  third-party runtime dependencies (PRD principle 5).
- **SR-I2 Packaging.** Installable via `pyproject.toml` (setuptools); also
  runnable in place with `python -m dsce`.
- **SR-I3 Testing.** Requirements marked in the FRD SHALL be covered by
  `unittest` tests runnable with `python -m unittest discover -s tests`,
  with no network or filesystem side effects.
- **SR-I4 Error behavior.** Duplicate vial ids raise `ValueError`;
  malformed conclusions (unbound variables after compute) skip the match
  rather than aborting the flood; unknown neighbor ids are ignored.

## SR-Q Quality attributes

- **SR-Q1 Legibility.** Every module SHALL carry documentation sufficient
  for a newcomer to follow it (module docstring stating its role, inline
  commentary at decision points); the repository SHALL provide a full
  walkthrough (docs/CODE_WALKTHROUGH.md) including a tick-by-tick trace.
- **SR-Q2 Observability.** Every `Result` SHALL expose flood telemetry
  (FR-4.5) so sparsity and termination behavior are measurable without a
  debugger.
- **SR-Q3 Attribution.** Distribution artifacts SHALL retain LICENSE
  (Apache-2.0), NOTICE, and CITATION.cff.

## Requirements traceability

| SRD | Implements | Verified by |
|---|---|---|
| SR-D1..3 | FR-5.1/5.2, PRD-3 | test_determinism |
| SR-P2 | PRD-7 (future) | (benchmark suite arrives with M3) |
| SR-P3 | FR-3.6 | test_tick_budget_halts_flood |
| SR-I4 | FR-1.3, FR-2.3 | test_duplicate_vial_id_rejected, test_substitute_raises_on_unbound |
| SR-Q2 | FR-4.5, PRD-4 | test_sparse_activation |

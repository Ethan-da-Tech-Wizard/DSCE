# DSCE Functional Requirements Document (FRD)

**Version:** 0.1 · **Date:** 2026-07
**Traces to:** [PRD](PRD.md) product requirements
**Verified by:** `tests/test_engine.py` (14 tests, all referenced below)

Each requirement states observable behavior: given X, the system does Y.
"WM" = working memory, the per-query pool of derived facts.

## FR-1 Knowledge representation

- **FR-1.1** The system SHALL represent all knowledge and queries as
  triples `(subject, predicate, object)`; terms are strings, ints, floats,
  or booleans. Strings beginning `?` are variables; triples containing
  variables are patterns.
- **FR-1.2** The system SHALL support vials with: unique id, concept
  description, ground facts, rules, neighbor ids, evidence strings, and a
  confidence in [0, 1].
- **FR-1.3** The system SHALL reject registration of a vial whose id is
  already registered. *(test_duplicate_vial_id_rejected)*
- **FR-1.4** Rules SHALL consist of a name, one or more premise patterns
  (sharing variables), a conclusion pattern, an optional pure compute
  function producing extra bindings, and a confidence.

## FR-2 Query and matching

- **FR-2.1** The system SHALL accept any pattern as a goal; variables may
  appear in any position. *(test_variable_subject_enumerates)*
- **FR-2.2** Unification SHALL bind unbound variables to fact values,
  SHALL fail on constant mismatch, and SHALL fail when an existing binding
  contradicts the fact. *(TestUnification, all)*
- **FR-2.3** Instantiating a conclusion with an unbound variable SHALL NOT
  produce a fact; the match is skipped. *(test_substitute_raises_on_unbound
  covers the underlying primitive)*

## FR-3 Flood semantics

- **FR-3.1** Seeding: the constants of the goal (all three positions)
  SHALL become the initial sand grains.
- **FR-3.2** Waking: a grain SHALL activate every dormant vial indexed
  under its term; a newly activated vial SHALL activate its neighbors,
  transitively, within the same tick.
- **FR-3.3** Newly activated vials SHALL contribute their axiom facts to
  WM exactly once, recorded with vial id, evidence, and vial confidence.
- **FR-3.4** Every active vial's rules SHALL be evaluated against WM every
  tick; a derived fact already present in WM SHALL be discarded (first
  derivation wins).
- **FR-3.5** Sand emission: each new fact SHALL emit grains for its
  subject and object, and SHALL NOT emit a grain for its predicate.
  *(test_sparse_activation)*
- **FR-3.6** Termination: the flood SHALL stop at fixpoint (a tick adding
  no facts and no vials) or after `max_ticks` ticks, whichever comes
  first. *(test_tick_budget_halts_flood)*
- **FR-3.7** Chaining: facts derived on a tick SHALL be available to rules
  on the same and subsequent ticks, enabling multi-rule chains
  (square → rectangle → area). *(test_derived_facts_feed_further_rules,
  test_multi_vial_inference_chain)*

## FR-4 Answers and proofs

- **FR-4.1** Answers SHALL be every WM fact at settlement that unifies
  with the goal, each carrying its variable bindings.
- **FR-4.2** Every answer SHALL expose a proof tree in which each node is
  either an axiom (vial + evidence) or a rule application (rule + vial +
  ground premises), recursively down to axioms.
  *(test_multi_vial_inference_chain checks proof content)*
- **FR-4.3** Confidence SHALL propagate as
  `rule.confidence × vial.confidence × min(premise confidences)`; axioms
  carry their vial's confidence. *(test_confidence_propagates)*
- **FR-4.4** A goal with no matching facts SHALL return an empty answer
  list (with telemetry), not an error. *(test_no_proof_found)*
- **FR-4.5** Results SHALL report: ticks used, total grains, activated
  vial ids in activation order, dormant vial ids, and WM size.

## FR-5 Determinism

- **FR-5.1** Two floods over equal knowledge bases and equal goals SHALL
  produce byte-identical rendered results — same answers, same order, same
  proofs, same telemetry. *(test_determinism)*
- **FR-5.2** No component SHALL use randomness, wall-clock time, hardware
  properties, or iteration over unordered collections.

## FR-6 Interfaces

- **FR-6.1** Library: `Engine()`, `add_vial(Vial)`, `ask(pattern) → Result`,
  `Result.summary() → str`, `Answer.proof.render() → str`.
- **FR-6.2** CLI: `python -m dsce` runs the showcase; `python -m dsce S P O`
  asks one triple, parsing tokens as bool/int/float/string, `?`-prefixed
  tokens as variables; any other argument count prints usage and exits 2.

## Future functional requirements (not in v0.1)

Tracked in [MILESTONES.md](MILESTONES.md): vial file format & loader (M1),
demand-driven backward grains (M2), incremental/indexed matching — replacing
the naive O(|WM|^P) join (M3), conflict surfacing (M4), NL front end (M5),
planner (M6).

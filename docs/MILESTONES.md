# DSCE Milestones

**Version:** 0.1 · **Date:** 2026-07
Ordering is dependency-driven, not calendar-driven: each milestone has exit
criteria and unlocks the next. Risks referenced as R-# from the
[Risk Register](RISK_REGISTER.md); requirements as PRD-#/FR-#/SR-# from the
[PRD](PRD.md)/[FRD](FRD.md)/[SRD](SRD.md).

## ✅ M0 — Reference prototype (this repository)

**Goal:** make the architecture concrete, testable, and citable.

Delivered: triple/unification core; vials with facts, rules, evidence,
confidence, neighbors; sand-grain flooding with sparse activation;
computed conclusions; confidence propagation; proof trees; deterministic
end-to-end (tested); CLI + demo KB; 14 tests; full documentation set
(walkthrough, problem statement, PRD, FRD, SRD, risk register, design doc);
attribution scaffolding (Apache-2.0, NOTICE, CITATION.cff).

## M1 — Vials as data

**Goal:** knowledge authoring without writing Python. *(PRD-8, mitigates R-3, R-6)*

- Define a vial file format (JSON or TOML): facts, rules, neighbors,
  evidence, confidence — and a **whitelisted expression language** for
  computed conclusions (arithmetic, comparison) replacing raw lambdas in
  data vials.
- Loader with validation and clear error messages (schema violations name
  the file, vial, and field).
- Ship the demo KB in both forms; round-trip test.

**Exit criteria:** `Engine.load_directory("vials/")` reproduces the demo
showcase byte-identically; a malformed vial file fails with an actionable
message.

## M2 — Demand-driven flooding + the paper

**Goal:** sand that *pulls* what a goal needs, and a citable canonical reference. *(mitigates R-8, R-7)*

- **Demand grains (backward chaining):** when an active vial holds a rule
  whose conclusion could serve the goal, emit grains for the rule's premise
  constants — the flood reaches fact-holding vials even when no forward
  path exists, reducing reliance on hand-authored neighbor links.
- Sparsity benchmarks: activation ratio and grain counts across KB sizes,
  tracked per commit.
- Write the DSCE technical paper (architecture, semantics, comparison with
  Rete/Datalog/spreading activation/MoE, prototype results) and post it
  (arXiv or repo `paper/`).

**Exit criteria:** a KB with deliberately missing neighbor links still
answers all showcase queries; measured activation ratio does not regress;
paper published and linked from README/CITATION.cff.

## M3 — Scale: replace the naive join

**Goal:** floods over ≥10⁵ facts in under a second. *(PRD-7, SR-P2, retires R-1 — the top risk)*

The prototype's `_match` explores up to O(|WM|^P) combinations and
re-matches the entire working memory every tick. Replace with:

- **Fact indexing** by predicate and bound argument positions (hash
  lookups instead of scans).
- **Semi-naive evaluation:** each tick joins only against facts new since
  the previous tick — no re-derivation churn at fixpoint.
- Evaluate a **Rete-style network** (cached partial matches) if rule
  counts grow to where per-tick re-joining dominates.
- Benchmark suite (KB generators at 10³/10⁴/10⁵ facts; P ∈ {1,2,3}) run in CI.
- Per-rule match budgets as a hardening backstop for pathological rules.

**Exit criteria:** 10⁵ facts, 3-premise rules, < 1 s per flood on commodity
hardware; determinism test still byte-identical; benchmarks in CI.

## M4 — Trustworthy knowledge

**Goal:** deterministic handling of imperfect knowledge. *(PRD-9, mitigates R-4, R-5)*

- Conflict detection: contradictory facts (same subject/predicate,
  different object where functional) surface *in the proof* instead of
  silent first-derivation-wins.
- Vial versioning and provenance metadata (who added what, when, from
  which source revision).
- Evaluate replacement confidence algebras (corroboration should raise
  confidence; correlated sources shouldn't double-count) under the
  determinism constraint; the algebra stays isolated in one function.

**Exit criteria:** a KB with a planted contradiction yields an answer whose
proof shows both branches and the resolution rule applied; version bump of
a vial reproducibly changes (only) dependent answers.

## M5 — Language front end

**Goal:** questions in, goals out. *(PRD-10, confronts R-2 — the hard one)*

Phased on purpose:
1. Template grammars per domain ("what is the AREA of X?" → `(X, area, ?a)`).
2. Controlled natural language with vocabulary drawn from loaded vials.
3. **Hybrid AI/DE:** an LLM proposes candidate goal patterns and relevant
   vials; the DSCE verifies, floods, and answers. The proof remains the
   product — the LLM never asserts facts, only suggests where to look.

**Exit criteria:** showcase queries answerable from plain-English phrasing;
every NL answer still carries a full proof; wrong LLM proposals degrade to
"no proof found", never to wrong answers.

## M6 — Planning and the AI/DE assistant

**Goal:** from answering questions to decomposing tasks — the long-term vision.

- Architecture generator: "build me X" → component graph whose nodes
  become sub-goals the flood can prove out (the Photoshop-decomposition
  problem from the original design conversations).
- Vial libraries compiled from real documentation (per R-3 mitigations).
- An assistant loop: converse (LLM) → plan → prove (DSCE) → generate →
  verify → repair.

**Exit criteria (first concrete target):** given vials compiled from Python
stdlib docs, the system plans and emits a small working program (e.g. a
CLI note-taker with search), with every API usage in the output justified
by a proof step citing documentation.

---

### Milestone → risk/requirement map

| Milestone | Retires/reduces | Delivers |
|---|---|---|
| M0 ✅ | R-9 (docs), R-7 (attribution) | PRD-1..6 |
| M1 | R-3, R-6 | PRD-8 |
| M2 | R-8, R-7 | paper, sparsity benchmarks |
| M3 | **R-1 (top risk)** | PRD-7, SR-P2 |
| M4 | R-4, R-5 | PRD-9 |
| M5 | R-2 | PRD-10 |
| M6 | — | the AI/DE vision |

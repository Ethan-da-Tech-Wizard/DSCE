# DSCE Risk Register

**Version:** 0.1 · **Date:** 2026-07
**Scoring:** Likelihood × Impact, each 1–5. Severity = product (≥15 critical, 8–14 high, 4–7 medium, ≤3 low).

| ID | Risk | L | I | Sev | Category |
|---|---|---|---|---|---|
| R-1 | Combinatorial blow-up of rule matching | 5 | 4 | **20 critical** | technical |
| R-2 | Natural-language front end proves intractable | 4 | 5 | **20 critical** | research |
| R-3 | Knowledge acquisition doesn't scale (hand-authored vials) | 4 | 4 | **16 critical** | product |
| R-4 | Wrong/conflicting knowledge yields confidently wrong proofs | 3 | 4 | **12 high** | quality |
| R-5 | Confidence algebra too crude for real domains | 3 | 3 | **9 high** | technical |
| R-6 | `compute` hooks break determinism or safety | 2 | 4 | **8 high** | technical |
| R-7 | Idea is reinvented/absorbed without attribution | 2 | 3 | **6 medium** | strategic |
| R-8 | Sparsity collapses on richly connected KBs | 3 | 2 | **6 medium** | technical |
| R-9 | Single-maintainer bus factor | 3 | 2 | **6 medium** | project |

## R-1 — Combinatorial blow-up of rule matching (CRITICAL)

- **Description:** `Engine._match` is a naive join: a rule with P premises
  over |WM| facts explores up to O(|WM|^P) combinations, and every rule is
  re-matched against the entire WM every tick (known facts are re-derived
  and discarded). At demo scale (17 facts) this is invisible; at 10⁵ facts
  with 3-premise rules it is ~10¹⁵ combinations — the engine simply stops
  being usable well before the KB reaches interesting size.
- **Trigger:** any KB beyond ~10³ facts or rules with P ≥ 3 over broad predicates.
- **Mitigation (planned, Milestone M3):** index WM facts by predicate and
  bound argument positions; adopt **semi-naive evaluation** (each tick,
  join only against facts new since last tick, which also eliminates
  re-derivation waste); if rule sets grow large, move to a Rete network
  with cached partial matches. Add a benchmark suite so the fix is measured,
  not assumed.
- **Residual risk after M3:** low for P ≤ 3; pathological rules can still
  be expensive — mitigated by per-rule match budgets (M4 hardening).

## R-2 — Natural-language front end proves intractable (CRITICAL)

- **Description:** Mapping ambiguous human questions to goal patterns is
  the historically unsolved problem that sank classic expert systems. If
  users can only speak triples, the audience shrinks to specialists.
- **Mitigation:** phase it (M5): templates → controlled natural language →
  hybrid, where an LLM *proposes* goal patterns and vial selections but the
  DSCE *verifies* and answers, keeping the trust story intact. The hybrid
  is explicitly part of the vision (AI/DE), not a compromise of it.
- **Residual:** medium — hybrid quality depends on the proposer model.

## R-3 — Knowledge acquisition doesn't scale (CRITICAL)

- **Description:** Four hand-written vials took an afternoon; the vision
  (documentation-backed vial libraries for whole languages/APIs) needs
  thousands. Manual authoring alone will stall the project — the classic
  "knowledge engineering bottleneck".
- **Mitigation:** M1 defines a data file format for vials (making authoring
  declarative and diffable); M2+ builds compilers from structured sources
  (API docs, schemas, tables) where extraction is mechanical; accept
  LLM-assisted *drafting* of vials with human review, since provenance and
  review are exactly what the vial format is for.
- **Residual:** medium — depends on source quality and reviewer bandwidth.

## R-4 — Confidently wrong proofs (HIGH)

- **Description:** Determinism does not equal correctness: a wrong axiom
  or rule yields the same wrong answer every time, presented with an
  authoritative-looking proof. Users may over-trust rendered derivations.
- **Mitigation:** evidence fields keep sources one hop away (shipped);
  M4 adds conflict detection (contradictory facts surfaced in the proof
  rather than silently first-derivation-wins), vial versioning, and
  review tooling. Documentation states the limitation plainly
  (DESIGN.md §1).

## R-5 — Confidence algebra too crude (HIGH)

- **Description:** `rule × vial × min(premises)` ignores correlated
  evidence, independent corroboration (two sources should *raise*
  confidence), and negation. Real domains may need calibrated uncertainty.
- **Mitigation:** the algebra is isolated in one function
  (`Engine._conclude`), so it can be swapped without touching flood
  mechanics; M4 evaluates principled alternatives (Dempster–Shafer,
  probabilistic soft logic) against the determinism constraint.

## R-6 — `compute` hooks (HIGH)

- **Description:** compute functions are arbitrary Python. An impure hook
  (randomness, time, network) silently breaks the core determinism
  guarantee; a malicious one is code execution inside the engine.
- **Mitigation:** purity contract documented at the definition site
  (shipped); M1's vial file format restricts computations in *data* vials
  to a whitelisted expression language (arithmetic/comparison), reserving
  raw Python hooks for code-defined vials the operator already trusts.

## R-7 — Attribution loss (MEDIUM)

- **Description:** The architecture overlaps known fields (spreading
  activation, Datalog, blackboards); ideas travel and names fall off.
- **Mitigation:** Apache-2.0 + NOTICE naming the originator (shipped),
  CITATION.cff (shipped), README credit line (shipped), public commit
  history; write and post the technical paper (M2 exit criterion) so
  there is a citable canonical reference.

## R-8 — Sparsity collapse (MEDIUM)

- **Description:** Sparse activation is the efficiency claim, but hub
  terms (very common entities) could wake most of a large KB, as generic
  predicates already did in development (caught by test, fixed by not
  emitting predicate grains).
- **Mitigation:** telemetry makes sparsity measurable per query (shipped);
  M2's demand-driven grains pull only goal-relevant premises; hub-term
  damping (activation budgets per term) if measurement shows collapse.

## R-9 — Bus factor (MEDIUM)

- **Description:** One person holds the vision and the code.
- **Mitigation:** this documentation set (walkthrough, PRD/FRD/SRD,
  design doc) exists precisely so a newcomer can reconstruct intent;
  tests pin behavior; milestones make the roadmap explicit.

---

**Review cadence:** revisit this register at every milestone boundary
(see [MILESTONES.md](MILESTONES.md)); re-score, close retired risks, add
newly discovered ones.

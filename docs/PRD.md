# DSCE Product Requirements Document (PRD)

**Product:** DSCE — Deterministic Semantic Computation Engine
**Author/Originator:** Ethan Kilmer
**Version:** 0.1 · **Date:** 2026-07
**Related docs:** [Problem Statement](PROBLEM_STATEMENT.md) · [FRD](FRD.md) · [SRD](SRD.md) · [Risk Register](RISK_REGISTER.md) · [Milestones](MILESTONES.md)

> The PRD says **what the product is and why** (user-facing goals).
> The FRD says **what it must do** (behavior). The SRD says **how the
> software must be built** (technical requirements).

## 1. Vision

A reasoning engine where knowledge is explicit and answers are proofs.
Long-term, the DSCE is the trustworthy core of AI/DE assistant systems:
a statistical language model handles the conversation, the DSCE decides
what is actually true and shows its work.

## 2. Target users

| User | Need |
|---|---|
| Researchers / students of AI | an inspectable, hackable alternative-architecture reference |
| Developers of high-trust software | embeddable reasoning with citations (compliance, engineering checks, education) |
| Knowledge engineers | a container format (vials) for curating domain knowledge with provenance |
| The curious | a system whose every answer can be read and understood end to end |

## 3. Product principles

1. **Every answer is a proof.** No conclusion without a visible derivation
   ending at cited sources. This is the product; everything else serves it.
2. **Same question, same answer.** Determinism is non-negotiable, including
   determinism about how uncertainty is handled.
3. **Pay only for what you use.** Activation is sparse; unrelated knowledge
   costs nothing per query.
4. **Knowledge is legible.** A vial can be read, reviewed, versioned, and
   diffed by a human. No opaque blobs.
5. **Zero-friction start.** The reference implementation runs with a stock
   Python install — no dependencies, no build step, no GPU.

## 4. Product requirements

| ID | Requirement | Priority | Status (v0.1) |
|---|---|---|---|
| PRD-1 | A user can define knowledge as vials (facts, rules, evidence, confidence, neighbors) in plain Python | P0 | ✅ shipped |
| PRD-2 | A user can ask a query and receive answers with full proof trees citing evidence | P0 | ✅ shipped |
| PRD-3 | Identical KB + query ⇒ byte-identical output | P0 | ✅ shipped, tested |
| PRD-4 | Query results expose flood telemetry (ticks, grains, activated vs dormant vials) so sparsity is observable | P1 | ✅ shipped |
| PRD-5 | Rules can compute values (arithmetic etc.), not just deduce symbols | P1 | ✅ shipped |
| PRD-6 | Usable from the command line and as a library | P1 | ✅ shipped |
| PRD-7 | The engine remains responsive on knowledge bases of ≥10⁵ facts | P1 | ❌ future (M3) — current join is O(\|WM\|^P) |
| PRD-8 | Vial libraries can be loaded from data files (not only Python code) | P2 | ❌ future (M1) |
| PRD-9 | Conflicting knowledge is resolved deterministically and the conflict is surfaced in the proof | P2 | ❌ future (M4) |
| PRD-10 | Natural-language questions are mapped to goal patterns | P2 | ❌ future (M5) |

## 5. Non-goals (v0.x)

- Competing with LLMs on open-ended text generation, style, or chit-chat.
- Probabilistic inference beyond the simple confidence algebra (no Bayesian networks).
- Distributed / multi-node execution.
- A GUI. The CLI and library API are the product surface for now.

## 6. Success metrics

| Metric | Target |
|---|---|
| Mechanics demonstrable end-to-end | showcase runs all 4 query classes (chain, compute, derive-then-compute, enumerate) ✅ |
| Reproducibility | determinism test green on every commit ✅ |
| Sparsity | single-domain demo queries wake ≤ 50% of vials ✅ (2/4) |
| Comprehensibility | a newcomer can trace one query end-to-end using docs/CODE_WALKTHROUGH.md alone |
| Scale (post-M3) | 10⁵ facts, P≤3 rules: answer < 1 s on commodity hardware |

## 7. Release framing

v0.1 (this repo) is a **reference prototype**: its job is to make the
architecture concrete, testable, and citable — not to be fast. The path
from prototype to product is defined in [MILESTONES.md](MILESTONES.md);
the ordered risks to that path are in [RISK_REGISTER.md](RISK_REGISTER.md).

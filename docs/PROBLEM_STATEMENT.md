# DSCE Problem Statement

**Project:** DSCE — Deterministic Semantic Computation Engine
**Originator:** Ethan Kilmer
**Status:** v0.1 (prototype stage)

## The problem

Modern AI question-answering is dominated by large language models, which
answer by *predicting likely text* from knowledge compressed into billions
of opaque numeric weights. This design has four structural consequences
that no amount of scaling fixes, because they are properties of the
architecture itself:

1. **Unverifiable answers.** An LLM cannot show *why* an answer is correct.
   Its reasoning is distributed across weights; there is no derivation to
   inspect, so errors ("hallucinations") are only detectable by already
   knowing the answer.
2. **No source attribution.** Knowledge absorbed during training loses its
   provenance. The model cannot reliably cite which document justified
   which claim, which disqualifies it wherever auditability is mandatory
   (law, medicine, engineering, compliance).
3. **Non-reproducibility.** Sampling, hardware nondeterminism, and model
   updates mean the same question can yield different answers on different
   days. Systems of record cannot be built on answers that drift.
4. **Dense, indiscriminate computation.** Every query activates every
   parameter. Asking a geometry question pays for the model's knowledge of
   poetry. Cost scales with model size, not with question difficulty.

Existing symbolic alternatives (theorem provers, Datalog engines, expert
systems) solve 1–3 but historically failed on usability: knowledge
acquisition is manual, activation is global or query-compiled rather than
associative, and none present a mental model a non-specialist can hold.

## The proposed solution

The DSCE stores knowledge in explicit, evidence-carrying containers
(**vials**) and reasons by flooding the container network with activation
particles (**sand**) seeded from the question. Only vials the sand reaches
participate. Every derived fact records its exact derivation, so every
answer is a **proof**, not a prediction. All iteration is ordered and no
randomness is used, so identical inputs yield byte-identical outputs.

This directly inverts each failure above: answers are verifiable (1),
cited (2), reproducible (3), and computed sparsely (4).

## What success looks like

- **Near term:** a reference implementation demonstrating the mechanics
  end-to-end with tests (this repository, done), adopted as the canonical
  statement of the architecture.
- **Mid term:** vial libraries compiled from real documentation, efficient
  rule evaluation (the prototype's naive O(|WM|^P) join replaced), and
  floods over 10⁵–10⁶ facts with sub-second answers.
- **Long term:** AI/DE systems — assistants that pair a statistical
  language front end with a DSCE core, so the machine *talks* like an LLM
  but *reasons* like a proof engine, in domains where being able to check
  the answer matters.

## Out of scope (deliberately)

The DSCE does not aim to replace LLMs for open-ended generation, style,
translation, or conversation. It targets the complementary niche: answers
that must be right, traceable, and repeatable.

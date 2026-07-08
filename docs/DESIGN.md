# DSCE Design Document

*DSCE (Deterministic Semantic Computation Engine) is an original architecture
proposed by Ethan Kilmer. This document describes the architecture, the
reference prototype in this repository, and the road toward AI/DE assistant
systems built on a DSCE core.*

## 1. Motivation

Large language models answer questions by predicting likely text from
knowledge compressed into billions of opaque weights. That makes them
powerful but hard to trust: the reasoning is not inspectable, the sources
are not citable, and the whole model must be activated for every question.

DSCE inverts each of those properties:

- **Knowledge is explicit.** It lives in small, coherent containers
  (*vials*) that hold facts, rules, evidence sources, and confidence values.
- **Computation is sparse.** Activation (*sand*) flows outward from the
  question and wakes only the vials it reaches. Dormant vials cost nothing.
- **Answers are proofs.** Every conclusion records exactly which rule, in
  which vial, consumed which premises — all the way down to cited axioms.
- **Everything is deterministic.** The same knowledge base and the same
  question always produce the identical answer and the identical proof.

Determinism does not guarantee correctness — a wrong rule yields a
consistently wrong answer — but it makes errors *reproducible and
attributable*, which is what auditing, debugging, and trust require.

## 2. Core concepts

### 2.1 Facts

A fact is a ground triple `(subject, predicate, object)`, e.g.
`("socrates", "is_a", "human")` or `("courtyard", "width", 12)`. Patterns
are triples that may contain variables (`"?x"`), matched by unification.

### 2.2 Vials

A vial is the unit of knowledge storage:

```
Vial
─────────────────────────────
id:          geometry
concept:     Plane geometry
facts:       ground axioms
rules:       premises ⇒ conclusion (+ optional compute step)
neighbors:   ids of related vials
evidence:    human-readable sources ("Euclid, Elements, Book I")
confidence:  0.0 – 1.0
```

Rules may carry a `compute` function that deterministically derives new
values from matched premises (e.g. `area = width × height`). This is what
makes DSCE a *computation* engine rather than only a symbolic deduction
engine.

### 2.3 Sand

Sand is not data — it is activation. Each grain carries one constant term
plus provenance (which vial emitted it, on which tick). Vials are indexed
by the constants they mention; a grain wakes every dormant vial indexed
under its term. Grains are emitted for the subject and object of each newly
derived fact, but **not** for the predicate: predicates are relations, not
entities, and generic ones like `is_a` would otherwise flood the whole
network and destroy sparsity.

### 2.4 The flood loop

```
seed grains from the goal's constants
repeat (up to a tick budget):
    grains wake dormant vials; firing vials wake their neighbors
    newly active vials pour axioms into shared working memory
    all active vials fire rules against working memory
    each new fact emits fresh grains
until nothing new happened            ← fixpoint: the flood has settled
answers = working-memory facts unifying with the goal
```

Every collection is iterated in sorted order and no randomness is used
anywhere, which is what makes the engine deterministic end to end.

### 2.5 Proofs and confidence

Working memory maps each fact to its derivation: either *axiom in vial V*
(with evidence sources) or *derived by rule R in vial V from premises P*.
Walking derivations backwards from an answer yields the proof tree.
Confidence propagates as
`rule.confidence × vial.confidence × min(premise confidences)` — the
engine is deterministic *about how it handles uncertainty*.

## 3. What the prototype demonstrates

- Multi-vial inference chains (philosophy axioms + biology rules).
- Deterministic computation inside proofs (rectangle area = 12 × 30 = 360).
- Derived facts feeding further rules (square ⇒ rectangle ⇒ area).
- Sparse activation: a geometry query never wakes the philosophy vial.
- Bit-identical results across runs (tested).
- Bounded flooding via a tick budget, so pathological rule sets terminate.

## 4. Known limitations of the prototype

- **No natural-language front end.** Queries are triples, not sentences.
  Ambiguity resolution and intent extraction are the single hardest open
  problem for the full vision.
- **Naive rule matching.** Premise joins scan working memory; a real
  implementation needs indexed joins (Rete-style networks or semi-naive
  Datalog evaluation).
- **Flat confidence model.** min/product propagation is a placeholder for a
  principled evidence calculus, conflict resolution, and versioned knowledge.
- **Hand-authored vials.** A vial *compiler* that ingests documentation
  (language references, API docs) into vials is future work.

## 5. The road ahead

The DSCE is intended as the reasoning core, not the whole assistant. The
target pipeline for an AI/DE (assistant/deterministic-engine) system:

```
Natural language → Intent extraction → Planner → Architecture generator
    → Documentation retrieval (vial libraries) → DSCE flood reasoner
    → Proof checker → Code/answer generator → Verification & repair
```

Milestones, roughly in order:

1. **Vial libraries** — curated, versioned vials compiled from official
   documentation, with source attribution baked in.
2. **Demand-driven flooding** — backward-chaining "demand grains" so the
   flood pulls in exactly the premises a goal needs, improving sparsity on
   large knowledge bases.
3. **Efficient matching** — indexed joins and incremental rule evaluation.
4. **Conflict and version handling** — deterministic resolution when vials
   disagree, with the disagreement surfaced in the proof.
5. **Language front end** — mapping questions to goal patterns, initially
   via templates, eventually via a hybrid with statistical models (an LLM
   proposes, the DSCE verifies).
6. **Planning** — the architecture generator that turns "build me X" into
   a component graph whose pieces the flood can then prove out.

## 6. Related work

DSCE sits near, but is not identical to, several established lines of
research: knowledge graphs with inference engines, production-rule systems
(OPS5/Rete), Datalog and semi-naive evaluation, spreading-activation models
of semantic memory, blackboard architectures, sparse Mixture-of-Experts
routing, and automated theorem proving. Its distinguishing combination is
*flow-based sparse activation over explicit, evidence-carrying knowledge
containers, with determinism and proof construction as first-class
requirements*.

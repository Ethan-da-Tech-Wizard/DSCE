# DSCE Code Walkthrough

*A plain-English tour of every file in the prototype, ending with a
tick-by-tick trace of a real query. Read this alongside the source — every
module also carries heavy inline documentation explaining itself.*

Reading order (each file builds on the previous ones):

```
facts.py  →  vial.py  →  sand.py  →  proof.py  →  engine.py  →  demo_kb.py  →  __main__.py
(triples)    (storage)   (activation) (traces)    (algorithm)   (example KB)   (CLI)
```

---

## 1. `dsce/facts.py` — the vocabulary

Everything the engine knows or asks is a **triple**: `(subject, predicate,
object)`, like `("socrates", "is_a", "human")` or `("courtyard", "width",
12)`. A triple with no unknowns is a **fact**. A triple containing
**variables** — strings starting with `?`, like `("?x", "is_a", "human")` —
is a **pattern**, meaning "any x that is a human".

Four functions do all the work:

| Function | Job | Example |
|---|---|---|
| `is_variable(t)` | "is this term a `?variable`?" | `is_variable("?x")` → True |
| `unify(pattern, fact, bindings)` | try to match a pattern against a fact; on success return what each variable equals | `unify(("?x","is_a","human"), ("socrates","is_a","human"), {})` → `{"?x": "socrates"}` |
| `substitute(pattern, bindings)` | fill a pattern's variables in to make a concrete fact | `substitute(("?r","area","?a"), {"?r":"courtyard","?a":360})` → `("courtyard","area",360)` |
| `constants(pattern)` | the non-variable terms — what the first sand grains will carry | `constants(("socrates","is_mortal","?answer"))` → `("socrates","is_mortal")` |

The subtle one is `unify`'s third argument. Bindings accumulated from one
premise **constrain the next**: if premise 1 bound `?x = socrates`, then
premise 2 can only match facts where `?x` is also socrates. That is how a
rule's premises stay about the same entity.

There is also `sort_key`, a helper that lets the engine sort collections
mixing strings, numbers, booleans, and tuples (Python refuses `12 <
"socrates"` natively). Sorting everything is how the engine guarantees the
same result every run.

## 2. `dsce/vial.py` — the knowledge containers

Two dataclasses:

**`Rule`** — "if all premises hold, the conclusion holds":

```python
Rule(
    name="rectangle-area",                       # shown in proofs
    premises=(("?r", "is_a", "rectangle"),       # ALL must match,
              ("?r", "width",  "?w"),            # sharing variables
              ("?r", "height", "?h")),
    conclusion=("?r", "area", "?a"),             # the new fact's shape
    compute=lambda b: {"?a": b["?w"] * b["?h"]}, # binds ?a by arithmetic
)
```

`compute` is the "computation" in DSCE: a deterministic function that
derives *extra* bindings from the matched ones, so a conclusion can contain
a value no premise supplied (here, the area). It must be pure — no
randomness, no I/O — or determinism breaks.

**`Vial`** — one container of knowledge about one concept. It has an `id`,
ground `facts` (axioms), `rules`, `neighbors` (ids of vials to wake
alongside it), `evidence` (human-readable sources that end up in proofs),
and a `confidence` in [0, 1].

`Vial.terms()` collects every constant the vial mentions — from its facts
and from the constant positions of its rules. Think of it as the vial's
**address**: the engine indexes vials under these terms, and a sand grain
carrying one of them will find and wake the vial.

## 3. `dsce/sand.py` — the activation particles

A `Grain` is three fields: `term` (one constant, e.g. `"socrates"`),
`origin` (`"query"` or the vial that emitted it), and `tick` (when).

Grains are the engine's *routing* mechanism, not its data. A grain's only
job is: look up my term in the term→vials index, wake any dormant vial
found there. Grains live exactly one tick; facts persist, sand does not.

One deliberate asymmetry, worth understanding because it took a failing
test to find: when a new fact is derived, grains are emitted for its
**subject and object only, never its predicate**. Predicates are relations
(`is_a`, `width`), not entities, and generic ones appear in nearly every
vial — during development, an `is_a` grain from a geometry fact woke the
philosophy vial, and `test_sparse_activation` now pins the fix. The *goal's*
predicate still seeds, though: the user's own words are always followed.

## 4. `dsce/proof.py` — how the engine shows its work

The key design decision: **proofs are not reconstructed afterwards; they
are bookkeeping the engine does anyway.** Every fact entering working
memory is stored with a `Derivation` record answering "how do I know this?":

- **axiom** — `vial_id` + that vial's `evidence` sources; or
- **derived** — `rule_name` + `vial_id` + the exact ground `premises` the
  rule consumed.

Since each premise is itself a fact with its own record, following records
backwards from any answer reconstructs the entire chain down to cited
axioms. The `Proof` class does exactly that walk, and `render()` draws it:

```
(socrates is_mortal True)  [by rule 'mammals-are-mortal' in vial 'biology', confidence 0.989]
└─ (socrates is_a mammal)  [by rule 'humans-are-mammals' in vial 'biology', confidence 0.990]
   └─ (socrates is_a human)  [axiom in vial 'philosophers' (evidence: Plato, Apology, ...), confidence 0.990]
```

Each `Derivation.confidence` is already the *combined* confidence of
everything beneath it — `rule × vial × min(premise confidences)` — computed
at derivation time, so nothing ever re-walks the tree to price trust.

## 5. `dsce/engine.py` — the flood algorithm

The heart. `Engine` holds the vials, a lazily built term→vial-ids index,
and a `max_ticks` budget (safety valve against rules that generate facts
forever, e.g. `n → n+1`; tested by `test_tick_budget_halts_flood`).

`Engine.ask(goal)` runs one flood. Two data structures live for the
duration of a query:

- `wm` (working memory): dict of `Fact → Derivation` — everything known so
  far *in this query*, doubling as the proof store.
- `active`: ordered set of woken vial ids.

The loop, tick by tick (numbered STEP 1–6 in the source):

1. **Wake by sand.** Each grain looks up its term in the index; dormant
   vials found there activate.
2. **Wake by neighbors.** Newly woken vials wake their declared neighbors,
   transitively, within the same tick.
3. **Pour axioms.** Newly woken vials add their facts to working memory
   (in sorted order), each recorded as an axiom Derivation.
4. **Fire rules.** *Every* active vial's rules are matched against *all* of
   working memory (`_match`, below). Each successful match is turned into a
   concrete new fact by `_conclude`. Already-known facts are skipped —
   first derivation wins, keeping proofs stable and floods finite.
5. **Emit sand.** Each genuinely new fact emits two grains: subject and
   object (not predicate — see §3).
6. **Fixpoint check.** If a tick produced no new facts and woke no new
   vials, the next tick would be identical: the sand has settled, stop.

Afterwards, every working-memory fact that unifies with the goal becomes an
`Answer` (bindings + proof), wrapped with flood telemetry in a `Result`.

**`_match(premises, wm)`** finds every way to satisfy all premises — a
database-style join built one premise at a time. Start with one empty
candidate binding `{}`; for each premise, try to extend every surviving
candidate against every fact in memory. Bindings from earlier premises
constrain later ones automatically, because `unify` rejects contradictions.

> ⚠ **The known complexity limit.** This join is *naive*: with P premises
> and |WM| facts it can inspect up to **O(|WM|^P)** combinations — and it
> re-does the work every tick, re-deriving (and discarding) everything it
> already knows. Harmless at 17 facts; fatal at a million. The fix —
> indexing facts by predicate, joining only against facts *new since the
> last tick* (semi-naive evaluation), or a full Rete network — is
> milestone **M3** in `MILESTONES.md`, requirement **SR-P2** in `SRD.md`,
> and risk **R-1** in `RISK_REGISTER.md`.

**`_conclude(rule, vial, bindings, wm)`** turns one match into one fact:
run `compute` if present (extra bindings), substitute into the conclusion
(skip the match gracefully if a variable is still unbound), and build the
Derivation with combined confidence `rule × vial × min(premises)` — a
conclusion is never more trusted than its shakiest premise.

**Where determinism comes from**, exhaustively: vial ids sorted in the
index; facts sorted when poured, when matched, and when answers are
extracted; insertion-ordered dicts for `wm` and `active`; first-derivation-
wins on duplicates; and zero randomness anywhere. `test_determinism`
asserts byte-identical output across two fresh engines.

## 6. `dsce/demo_kb.py` — the example knowledge base

Four vials in two unrelated domains, engineered to demonstrate specific
claims:

| Vial | Contents | Demonstrates |
|---|---|---|
| `philosophers` | facts only (socrates/plato are human) | evidence citations, confidence 0.99 |
| `biology` | rules only (human→mammal→mortal) | cross-vial chaining — neither vial alone can prove mortality |
| `geometry` | rules incl. `compute` (area = w×h; square→rectangle) | computed conclusions, multi-step derivation |
| `measurements` | facts (courtyard 12×30; plaza square, side 25) | survey evidence, confidence 0.97 |

The neighbor links (`philosophers→biology`, `measurements→geometry`) encode
"if you're using my facts you'll want those rules". The plaza is described
*only* as a square, so its area proof must first derive rectangle-ness and
width/height, then compute 25 × 25 — a three-rule chain visible in its proof.

## 7. `dsce/__main__.py` — the CLI

`python -m dsce` runs four showcase queries; `python -m dsce s p o` asks
your own triple. `parse_term` interprets tokens in order: `true`/`false` →
bool, then int, then float, else string (`?variables` are just strings —
the engine spots them by the leading `?`).

## 8. `tests/test_engine.py` — what is pinned down

14 tests in three groups: unification mechanics; flood behavior
(cross-vial chains, computed area, derived-facts-feeding-rules, sparse
activation, determinism, confidence propagation, no-proof, enumeration);
and engine basics (duplicate ids rejected, tick budget halts runaway rules).

---

## Appendix: a complete trace of `("socrates", "is_mortal", "?answer")`

The knowledge base has 4 vials. Watch what actually happens:

**Seeding (tick 0).** Goal constants: `socrates`, `is_mortal`. Two grains.

**Tick 1.**
- *Wake:* grain `socrates` → index → `philosophers` (its facts mention
  socrates). Grain `is_mortal` → `biology` (its rule conclusion mentions
  is_mortal). Neighbor step: philosophers lists biology — already awake.
  Active: {philosophers, biology}. Geometry and measurements: dormant.
- *Pour:* philosophers adds 3 axioms to working memory, each recorded
  `axiom, vial=philosophers, evidence=(Plato...), confidence 0.99`.
  Biology has no facts.
- *Fire:* biology's `humans-are-mammals` matches `(socrates is_a human)`
  → derives `(socrates is_a mammal)`; matches `(plato is_a human)` →
  `(plato is_a mammal)`. Then `mammals-are-mortal` runs — but the mammal
  facts *just* entered memory this same pass, so it fires too:
  `(socrates is_mortal True)`, `(plato is_mortal True)`. WM: 7 facts.
- *Sand:* each new fact emits subject+object grains: `socrates`, `mammal`,
  `plato`, `True`…
- Not a fixpoint (new facts appeared) → continue.

**Tick 2.**
- *Wake:* grain `mammal` → biology (already active). `True` → biology
  (already active). Nothing new wakes.
- *Fire:* all rules re-match, re-derive the same 4 facts, all already in
  memory → discarded. **This wasted re-matching is exactly the O(|WM|^P)
  cost the walkthrough warns about.**
- No new facts, no new vials → **fixpoint. Flood settles.**

**Answer extraction.** Scan the 7 facts for ones unifying with
`(socrates is_mortal ?answer)` → exactly `(socrates is_mortal True)`,
bindings `{"?answer": True}`. Its proof tree (walked from Derivation
records) and confidence 0.999 × 0.99 ≈ 0.989 are what the CLI prints:

```
flood: 2 tick(s), 16 grain(s) of sand, 2/4 vials activated, 7 fact(s) in working memory
activated vials: philosophers, biology
dormant vials:   geometry, measurements
answer 1 (confidence 0.989):
(socrates is_mortal True)  [by rule 'mammals-are-mortal' in vial 'biology', confidence 0.989]
└─ (socrates is_a mammal)  [by rule 'humans-are-mammals' in vial 'biology', confidence 0.990]
   └─ (socrates is_a human)  [axiom in vial 'philosophers' (evidence: Plato, Apology, Diogenes Laertius, Lives), confidence 0.990]
```

Half the knowledge base never woke up. That is the architecture working.

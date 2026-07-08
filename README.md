# DSCE — Deterministic Semantic Computation Engine

> DSCE (Deterministic Semantic Computation Engine) is an original architecture
> proposed by **Ethan Kilmer**. Knowledge lives in explicit containers
> ("vials"); reasoning is performed by flooding the vial network with
> activation particles ("sand") until enough evidence accumulates to assemble
> a verifiable proof of the answer.

This repository contains a working reference prototype in pure Python
(no dependencies), plus the design document describing the architecture.

## The idea in one picture

```
            Question
                │
                ▼
        Seed sand grains          (constants extracted from the goal)
                │
                ▼
     ┌─────────────────────┐
     │   Knowledge vials   │      facts · rules · evidence · confidence
     │                     │      · links to neighboring vials
     │  philosophers       │
     │  biology            │      Sand wakes only the vials it reaches.
     │  geometry           │      Dormant vials cost nothing.
     │  measurements       │
     └─────────────────────┘
                │
        Sand floods the network, tick by tick
                │
                ▼
      Active vials pour in axioms and fire rules;
      every new fact emits new sand
                │
                ▼
        Flood settles (fixpoint)
                │
                ▼
       Proof tree + verified answer
```

Unlike a language model, the engine does not predict likely text. It
**computes** answers from explicit knowledge, and every answer carries a
complete proof trace back to its sources. Given the same knowledge base and
the same question, it produces the identical result, every time.

|                       | LLM                      | DSCE                        |
|-----------------------|--------------------------|-----------------------------|
| Output                | Predicted next token     | Constructed proof           |
| Computation           | Dense matrix math        | Sparse graph flooding       |
| Knowledge             | Hidden weight vectors    | Explicit knowledge objects  |
| Activation            | Entire model, always     | Only the vials sand reaches |
| Trust                 | Statistical confidence   | Inspectable derivation      |

## Quick start

Requires Python 3.9+. No installation needed:

```console
$ python -m dsce                          # run the showcase queries
$ python -m dsce socrates is_mortal ?x    # ask your own triple
$ python -m dsce courtyard area ?a
$ python -m dsce "?who" is_a mammal
$ python -m unittest discover -s tests    # run the test suite
```

Example output:

```
goal: (courtyard area ?a)
flood: 2 tick(s), 22 grain(s) of sand, 2/4 vials activated, 10 fact(s) in working memory
activated vials: measurements, geometry
dormant vials:   biology, philosophers
answer 1 (confidence 0.970):
(courtyard area 360)  [by rule 'rectangle-area' in vial 'geometry', confidence 0.970]
├─ (courtyard is_a rectangle)  [axiom in vial 'measurements' (evidence: site survey 2026-03), confidence 0.970]
├─ (courtyard width 12)  [axiom in vial 'measurements' (evidence: site survey 2026-03), confidence 0.970]
└─ (courtyard height 30)  [axiom in vial 'measurements' (evidence: site survey 2026-03), confidence 0.970]
```

Note what happened: a geometry question left the philosophy and biology vials
dormant, the answer 360 was *computed* (12 × 30), and the proof cites the
site survey the measurements came from.

## Using it as a library

```python
from dsce import Engine, Vial, Rule

engine = Engine()
engine.add_vial(Vial(
    id="philosophers",
    concept="Classical philosophers",
    facts=(("socrates", "is_a", "human"),),
    evidence=("Plato, Apology",),
))
engine.add_vial(Vial(
    id="biology",
    concept="Basic biology",
    rules=(Rule(
        name="humans-are-mortal",
        premises=(("?x", "is_a", "human"),),
        conclusion=("?x", "is_mortal", True),
    ),),
))

result = engine.ask(("socrates", "is_mortal", "?answer"))
print(result.summary())
```

## Repository layout

```
dsce/
  facts.py     triples, patterns, unification
  vial.py      Vial and Rule — the knowledge containers
  sand.py      Grain — the activation particles
  engine.py    the flood loop: seed → wake → fire → settle
  proof.py     derivation records and proof-tree rendering
  demo_kb.py   a small four-vial demonstration knowledge base
  __main__.py  CLI entry point
tests/         test suite (unification, inference, determinism, sparsity)
docs/DESIGN.md the architecture in detail, and the road ahead
```

## Status and direction

This prototype demonstrates the core mechanics: vials, sand, deterministic
flooding, computed conclusions, confidence propagation, and proof
construction. The longer-term direction — natural-language intent parsing,
a planner/architecture generator, documentation-backed vial libraries, and
eventually AI/DE assistant systems built on a DSCE core — is sketched in
[docs/DESIGN.md](docs/DESIGN.md).

## License and attribution

Licensed under the [Apache License 2.0](LICENSE). You are free to use,
modify, and build on this work, including commercially; the license and
[NOTICE](NOTICE) file must remain attached to distributions.

If you use DSCE in research or writing, please cite it — see
[CITATION.cff](CITATION.cff).

Copyright © 2026 Ethan Kilmer.

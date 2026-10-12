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

## Rust implementation

The repository also contains a high-performance Rust port of the engine
(`Cargo.toml` + `src/`), with the same sand-and-vials model plus:

- a typed `Term` enum (`Str`/`Int`/`Float`/`Bool`) instead of duck typing,
- rayon-parallel rule matching over active vials (merged in a stable order,
  so parallelism never costs determinism),
- SQLite-backed vial storage via `rusqlite` with dynamic on-demand loading —
  dormant vials never leave disk,
- a small deterministic expression DSL (`?a = ?w * ?h`) replacing Python
  lambdas for rule computations, so rules round-trip through the database
  without `eval()`,
- functional-predicate conflict warnings and specificity notes, as in the
  Python prototype.

```console
$ cargo run                                              # in-memory demo showcase
$ cargo run -- socrates is_mortal ?answer                # ask your own triple
$ cargo run -- --db dsce.sqlite --seed                   # seed a SQLite database
$ cargo run -- --db dsce.sqlite "modesto" "located_in" "?where"
$ cargo test                                             # run the Rust test suite
```

The Rust store reads databases seeded by the Python `SqliteVialStore`
(Python-lambda compute bodies degrade to "no compute"; use the expression
DSL for computed rules that must round-trip).

## Generic software assembler (`vials_synthesis/`)

The Rust engine doubles as a zero-shot software assembler: instead of
hardcoded application templates, the knowledge base holds *generic design
patterns* and *API library documentation*, and programs are assembled by
Datalog rules splicing code fragments with string concatenation
(`?code = ?code1 + ?code2` in the compute DSL).

```
vials_synthesis/
  patterns/     structural knowledge (rules + facts)
    grid_layout.json   coordinate axes, row/column flow, nesting bounds
    mvc.json           Model-View-Controller separation + the capability
                       binder and final program assembler
    fsm.json           state nodes, transition keys, loop ticks
  libraries/    normalized API documentation (pure data, zero rules)
    pygame.json        screen init, surface rendering, event polling, colors
    sqlite.json        connections, schema, execute, fetch cursors
    websockets.json    serving, listeners, packet send, closure
```

The **Semantic Harvester** (`src/harvester.rs`) bridges natural language
and the engine. It turns a request into a Datalog goal plus dynamic
vocabulary triples poured into working memory for that one query:

```console
$ cargo run -- --synthesize "Make a multiplayer grid game with a scoring system"
harvested goal: (multiplayer_grid_game code ?code)
harvested vocabulary:
  (multiplayer_grid_game is_a application)
  (multiplayer_grid_game needs grid_layout)
  (scoring_system is_a state_machine)
  ...
--- assembled program #1 (confidence 0.980) ---
# assembled by the DSCE generic software assembler
...valid Python: pygame view, sqlite model, websockets controller, FSM loop
```

An LLM-backed harvest is supported too: `harvester::PROMPT_TEMPLATE`
compiles a creative request into strict JSON once, then the engine reasons
deterministically — same knowledge base + same harvest = byte-identical
program and proof, every run. The library vials never reference one
another; the MVC pattern's `bind-capability` rule connects an app's
abstract `needs` to whichever library `provides` that capability.
Documentation predicates (`param`, `returns`, ...) are registered as
*annotations*: rules can match them but they emit no sand, so shared API
vocabulary ("None", "size") cannot build activation bridges between
unrelated libraries.

### Polyglot synthesis

The harvester detects an explicit target language and asserts a
`(app, target_language, lang_*)` triple (Python is the default when no
language is named). The `polyglot_cli_app` pattern binds that triple to
whichever library vial `implements_language` it — the pattern itself names
no concrete language, so adding a language means dropping in one JSON vial.
Every language vial cites its authoritative source (ISO standards, official
standard-library documentation) in its `evidence`:

| Language   | Vial                            | Source cited                          |
|------------|---------------------------------|---------------------------------------|
| Python     | `libraries/random.json`         | Python standard library docs           |
| Rust       | `libraries/rust_std.json`       | doc.rust-lang.org/std                  |
| C          | `libraries/c_std.json`          | ISO/IEC 9899 standard library          |
| C++        | `libraries/cpp_std.json`        | C++ standard library (`<random>`)      |
| C#         | `libraries/csharp_dotnet.json`  | .NET API browser (learn.microsoft.com) |
| Go         | `libraries/go_std.json`         | pkg.go.dev standard library            |
| SQL        | `libraries/sql_sqlite.json`     | sqlite.org / ISO/IEC 9075              |
| Assembly   | `libraries/x86_64_assembly.json`| Intel SDM, System V AMD64 ABI, NASM    |
| JavaScript | `libraries/node_js.json`        | nodejs.org docs, ECMAScript spec       |

```console
$ cargo run -- --synthesize "Make a random number generator in Rust"
$ cargo run -- --synthesize "Make a random number generator in Go"
$ cargo run -- --synthesize "Write a random number generator in SQL"
```

### Frameworks, APIs, and dependency resolution

Framework vials (`pytorch`, `cuda`, `faiss`, `react`, `electron`, `docker`,
`kubernetes`) document each framework's starter API, its `depends_on`
graph, and its documented install channel, all sourced from the official
documentation. The `framework_app` pattern assembles the starter script;
`container_app` assembles Dockerfiles and Kubernetes Deployment manifests;
and `dependency_resolution` computes the transitive dependency closure as
plain Datalog:

```console
$ cargo run -- --synthesize "Make an Electron desktop app"
resolved dependencies: chromium, libuv, node_js, v8_engine
install steps:
  $ npm install electron

--- assembled program #1 (confidence 1.000) ---
// assembled by the DSCE framework assembler (Electron)
const { app, BrowserWindow } = require('electron');
...
```

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
src/
  facts.rs     (Rust) Term enum, triples, unification, specificity
  vial.rs      (Rust) Vial and Rule structs
  sand.rs      (Rust) Grain — the activation particles
  compute.rs   (Rust) deterministic, serializable rule computations
  engine.rs    (Rust) the flood loop with rayon-parallel rule firing
  db_store.rs  (Rust) SQLite persistence and dynamic vial loading
  proof.rs     (Rust) derivation records and proof-tree rendering
  demo_kb.rs   (Rust) the demonstration knowledge base
  main.rs      (Rust) CLI entry point
tests/         Python test suite + Rust integration tests (engine_tests.rs)
docs/          full documentation set — see below
```

## Documentation

Every source file carries heavy inline documentation; start there, or with:

| Document | What it answers |
|---|---|
| [docs/CODE_WALKTHROUGH.md](docs/CODE_WALKTHROUGH.md) | *"What is every part of the code doing?"* — file-by-file tour plus a tick-by-tick trace of a real query |
| [docs/DESIGN.md](docs/DESIGN.md) | the architecture in detail, related work, the road ahead |
| [docs/PROBLEM_STATEMENT.md](docs/PROBLEM_STATEMENT.md) | why DSCE exists — the four structural problems it inverts |
| [docs/PRD.md](docs/PRD.md) | product requirements: vision, users, principles, success metrics |
| [docs/FRD.md](docs/FRD.md) | functional requirements: exact observable behavior, traced to tests |
| [docs/SRD.md](docs/SRD.md) | software requirements: architecture rules, determinism rules, performance limits |
| [docs/RISK_REGISTER.md](docs/RISK_REGISTER.md) | scored risks with mitigations (top risk: the O(\|WM\|^P) naive join) |
| [docs/MILESTONES.md](docs/MILESTONES.md) | M0–M6 roadmap with exit criteria, mapped to risks and requirements |

## Status and direction

This prototype demonstrates the core mechanics: vials, sand, deterministic
flooding, computed conclusions, confidence propagation, and proof
construction. It is deliberately built for clarity, not speed — rule
matching is a naive O(|WM|^P) join, fine at demo scale and the first thing
milestone M3 replaces. The longer-term direction — natural-language intent
parsing, a planner/architecture generator, documentation-backed vial
libraries, and eventually AI/DE assistant systems built on a DSCE core —
is laid out milestone by milestone in
[docs/MILESTONES.md](docs/MILESTONES.md).

## License and attribution

Licensed under the [Apache License 2.0](LICENSE). You are free to use,
modify, and build on this work, including commercially; the license and
[NOTICE](NOTICE) file must remain attached to distributions.

If you use DSCE in research or writing, please cite it — see
[CITATION.cff](CITATION.cff).

Copyright © 2026 Ethan Kilmer.

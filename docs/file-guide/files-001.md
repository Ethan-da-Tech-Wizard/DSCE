# Files 1 of 1 — DSCE — Deterministic Semantic Computation Engine

[All pages](../FILE_GUIDE.md) · [Project guide](../PROJECT_GUIDE.md)

| File | Role and useful details |
| --- | --- |
| [.gitignore](../../.gitignore) | Defines local data, credentials, dependencies, and generated outputs that Git should leave untracked. |
| [CITATION.cff](../../CITATION.cff) | Tracked CITATION resource in .; 901 bytes. |
| [Cargo.lock](../../Cargo.lock) | Dependency lockfile recording resolved package versions and integrity/source metadata; update through the project’s package manager. |
| [Cargo.toml](../../Cargo.toml) | Configuration or build metadata for Cargo; read alongside the application or tool that loads it. |
| [LICENSE](../../LICENSE) | Preserved license, copyright, or attribution terms; read the full text before reusing the covered component. |
| [NOTICE](../../NOTICE) | Preserved license, copyright, or attribution terms; read the full text before reusing the covered component. |
| [PROJECT_NOTES.md](../../PROJECT_NOTES.md) | Preserved detailed setup, usage, architecture, and development notes from the earlier README. |
| [README.md](../../README.md) | Project overview, current status, entry points, setup navigation, and documentation links. |
| [build.rs](../../build.rs) | Source module for build; defines `main`. |
| [docs/CODE_WALKTHROUGH.md](../../docs/CODE_WALKTHROUGH.md) | Documentation: DSCE Code Walkthrough. Covers 1. `dsce/facts.py` — the vocabulary; 2. `dsce/vial.py` — the knowledge containers; 3. `dsce/sand.py` — the activation particles. |
| [docs/DEPENDENCIES.csv](../../docs/DEPENDENCIES.csv) | Machine-readable inventory of declared and locked dependencies, versions, reported licenses, and evidence URLs. |
| [docs/DESIGN.md](../../docs/DESIGN.md) | Documentation: DSCE Design Document. Covers 1. Motivation; 2. Core concepts; 2.1 Facts. |
| [docs/FILE_GUIDE.md](../../docs/FILE_GUIDE.md) | Index of the complete tracked-file guide, with one entry for every file and links to the paginated descriptions. |
| [docs/FRD.md](../../docs/FRD.md) | Documentation: DSCE Functional Requirements Document (FRD). Covers FR-1 Knowledge representation; FR-2 Query and matching; FR-3 Flood semantics. |
| [docs/LICENSE_REVIEW.md](../../docs/LICENSE_REVIEW.md) | Original-code license, preserved component notices, dependency-license evidence, and unresolved distribution questions. |
| [docs/MILESTONES.md](../../docs/MILESTONES.md) | Documentation: DSCE Milestones. Covers ✅ M0 — Reference prototype (this repository); M1 — Vials as data; M2 — Demand-driven flooding + the paper. |
| [docs/PRD.md](../../docs/PRD.md) | Documentation: DSCE Product Requirements Document (PRD). Covers 1. Vision; 2. Target users; 3. Product principles. |
| [docs/PROBLEM_STATEMENT.md](../../docs/PROBLEM_STATEMENT.md) | Documentation: DSCE Problem Statement. Covers The problem; The proposed solution; What success looks like. |
| [docs/PROJECT_GUIDE.md](../../docs/PROJECT_GUIDE.md) | Maintainer orientation: purpose, important entry points, source layout, commands, and project-specific boundaries. |
| [docs/RISK_REGISTER.md](../../docs/RISK_REGISTER.md) | Documentation: DSCE Risk Register. Covers R-1 — Combinatorial blow-up of rule matching (CRITICAL); R-2 — Natural-language front end proves intractable (CRITICAL); R-3 — Knowledge acquisition doesn't scale (CRITICAL). |
| [docs/SRD.md](../../docs/SRD.md) | Documentation: DSCE Software Requirements Document (SRD). Covers SR-A Architecture; SR-D Determinism (implements FR-5); SR-P Performance. |
| [docs/file-guide/files-001.md](../../docs/file-guide/files-001.md) | One page of the tracked-file catalog; links each listed path to its role and source-derived details. |
| [dsce/__init__.py](../../dsce/__init__.py) | DSCE — Deterministic Semantic Computation Engine. Connects `dsce.engine`, `dsce.vial`, `dsce.sand`, `dsce.proof`. |
| [dsce/__main__.py](../../dsce/__main__.py) | Run the DSCE demo: python -m dsce [subject predicate object] Defines `parse_term`, `main`. |
| [dsce/db_store.py](../../dsce/db_store.py) | Database-backed storage for DSCE vials. Defines `SqliteVialStore`. |
| [dsce/demo_kb.py](../../dsce/demo_kb.py) | A small demonstration knowledge base. Defines `build_engine`. |
| [dsce/engine.py](../../dsce/engine.py) | The DSCE flood engine. Defines `Answer`, `Result`, `Engine`. |
| [dsce/facts.py](../../dsce/facts.py) | Facts, patterns, and unification — the vocabulary of the whole engine. Defines `is_variable`, `unify`, `substitute`, `constants`, `sort_key`, `is_more_specific`. |
| [dsce/proof.py](../../dsce/proof.py) | Proof construction and rendering — how the DSCE shows its work. Defines `Derivation`, `fact_str`, `Proof`. |
| [dsce/sand.py](../../dsce/sand.py) | Sand: the activation particles of the DSCE. Defines `Grain`. |
| [dsce/vial.py](../../dsce/vial.py) | Vials: the knowledge containers of the DSCE. Defines `Rule`, `Vial`. |
| [gameboy_app.py](../../gameboy_app.py) | Python implementation of gameboy app. Defines `move_left`, `move_right`, `game_loop`, `press_key`. |
| [pyproject.toml](../../pyproject.toml) | Configuration or build metadata for pyproject; read alongside the application or tool that loads it. |
| [scripts/build_dictionary_tsv.py](../../scripts/build_dictionary_tsv.py) | Python implementation of build dictionary tsv. Defines `export_to_tsv`. |
| [scripts/download_dictionary.py](../../scripts/download_dictionary.py) | Download and Compile English Dictionary for DSCE. Defines `download_and_compile`. |
| [scripts/seed_geonames.py](../../scripts/seed_geonames.py) | Ingestion script to seed DSCE SQLite database with GeoNames cities data. Defines `download_and_extract`, `seed_db`. |
| [scripts/seed_taxonomy_grammar.py](../../scripts/seed_taxonomy_grammar.py) | Ingestion script to seed DSCE SQLite database with taxonomy and grammar data. Defines `seed_taxonomy_and_grammar`. |
| [scripts/update_dictionary.py](../../scripts/update_dictionary.py) | Python implementation of update dictionary. Defines `download_and_format`. |
| [src/compute.rs](../../src/compute.rs) | Source module for compute; defines `Compute`, `expr`, `native`, `eval`, `source`, `fmt`, `ExprProgram` and 27 more symbols. |
| [src/db_store.rs](../../src/db_store.rs) | Source module for db store; defines `StoreError`, `fmt`, `from`, `SqliteVialStore`, `open`, `open_in_memory`, `init_schema` and 9 more symbols. |
| [src/demo_kb.rs](../../src/demo_kb.rs) | Source module for demo kb; defines `t`, `demo_vials`, `build_engine`. |
| [src/engine.rs](../../src/engine.rs) | Source module for engine; defines `PredicateInfo`, `default`, `Answer`, `QueryResult`, `proof`, `summary`, `Engine` and 13 more symbols. |
| [src/facts.rs](../../src/facts.rs) | Source module for facts; defines `Term`, `str`, `float`, `is_variable`, `as_variable`, `fmt`, `from` and 12 more symbols. |
| [src/gui_main.rs](../../src/gui_main.rs) | Source module for gui main; defines `main`. |
| [src/harvester.rs](../../src/harvester.rs) | Source module for harvester; defines `prompt_for`, `Harvest`, `parse_response`, `matches_any`, `harvest_offline`, `t`, `flagship_request_harvests_full_vocabulary` and 7 more symbols. |
| [src/json_vials.rs](../../src/json_vials.rs) | Source module for json vials; defines `default_confidence`, `RuleDoc`, `VialDoc`, `triple`, `vial_from_json`, `load_vial_file`, `load_vials_dir` and 4 more symbols. |
| [src/lib.rs](../../src/lib.rs) | Source module for lib. |
| [src/main.rs](../../src/main.rs) | Source module for main; defines `parse_term`, `usage`, `synthesize`, `main`. |
| [src/proof.rs](../../src/proof.rs) | Source module for proof; defines `Derivation`, `fact_str`, `render_proof`, `render_node`. |
| [src/sand.rs](../../src/sand.rs) | Source module for sand; defines `Grain`, `new`, `fmt`. |
| [src/ui/appwindow.slint](../../src/ui/appwindow.slint) | Tracked appwindow resource in src/ui; 12,784 bytes. |
| [src/vial.rs](../../src/vial.rs) | Source module for vial; defines `from`, `Rule`, `new`, `with_compute`, `with_confidence`, `Vial`, `with_facts` and 5 more symbols. |
| [tests/engine_tests.rs](../../tests/engine_tests.rs) | Test implementation for engine tests; defines `t`, `multi_vial_inference_chain`, `deterministic_computation`, `derived_facts_feed_further_rules`, `sparse_activation`, `determinism`, `confidence_propagates` and 10 more symbols. |
| [tests/synthesis_tests.rs](../../tests/synthesis_tests.rs) | Test implementation for synthesis tests; defines `t`, `vials_dir`, `synthesize`, `synthesis_kb_loads_completely`, `library_vials_are_decoupled`, `flagship_request_assembles_full_program`, `tick` and 71 more symbols. |
| [tests/test_engine.py](../../tests/test_engine.py) | Automated checks for engine. Defines `TestUnification`, `TestFlood`, `TestEngineBasics`. |
| [tests/test_new_features.py](../../tests/test_new_features.py) | Automated checks for new features. Defines `TestConflicts`, `TestSpecificity`, `TestSqliteVialStore`. |
| [vials_synthesis/english_dictionary.json](../../vials_synthesis/english_dictionary.json) | Structured english dictionary data or evidence; inspect its schema before changing consumers. |
| [vials_synthesis/libraries/academic_concepts.json](../../vials_synthesis/libraries/academic_concepts.json) | Structured academic concepts data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/c_std.json](../../vials_synthesis/libraries/c_std.json) | Structured c std data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/cpp_std.json](../../vials_synthesis/libraries/cpp_std.json) | Structured cpp std data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/csharp_dotnet.json](../../vials_synthesis/libraries/csharp_dotnet.json) | Structured csharp dotnet data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/cuda.json](../../vials_synthesis/libraries/cuda.json) | Structured cuda data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/ddia_concepts.json](../../vials_synthesis/libraries/ddia_concepts.json) | Structured ddia concepts data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/docker.json](../../vials_synthesis/libraries/docker.json) | Structured docker data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/electron.json](../../vials_synthesis/libraries/electron.json) | Structured electron data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/express.json](../../vials_synthesis/libraries/express.json) | Structured express data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/faiss.json](../../vials_synthesis/libraries/faiss.json) | Structured faiss data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/fastapi.json](../../vials_synthesis/libraries/fastapi.json) | Structured fastapi data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/go_std.json](../../vials_synthesis/libraries/go_std.json) | Structured go std data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/kubernetes.json](../../vials_synthesis/libraries/kubernetes.json) | Structured kubernetes data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/node_js.json](../../vials_synthesis/libraries/node_js.json) | Structured node js data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/pygame.json](../../vials_synthesis/libraries/pygame.json) | Structured pygame data; top-level fields: `id`, `concept`, `facts`, `evidence`, `confidence`. |
| [vials_synthesis/libraries/python_extended.json](../../vials_synthesis/libraries/python_extended.json) | Structured python extended data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/python_gui_libraries.json](../../vials_synthesis/libraries/python_gui_libraries.json) | Structured python gui libraries data; top-level fields: `id`, `concept`, `facts`, `evidence`, `confidence`. |
| [vials_synthesis/libraries/pytorch.json](../../vials_synthesis/libraries/pytorch.json) | Structured pytorch data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/qt_cpp.json](../../vials_synthesis/libraries/qt_cpp.json) | Structured qt cpp data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/random.json](../../vials_synthesis/libraries/random.json) | Structured random data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/react.json](../../vials_synthesis/libraries/react.json) | Structured react data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/rust_std.json](../../vials_synthesis/libraries/rust_std.json) | Structured rust std data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/sql_sqlite.json](../../vials_synthesis/libraries/sql_sqlite.json) | Structured sql sqlite data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/sqlite.json](../../vials_synthesis/libraries/sqlite.json) | Structured sqlite data; top-level fields: `id`, `concept`, `facts`, `evidence`, `confidence`. |
| [vials_synthesis/libraries/tkinter.json](../../vials_synthesis/libraries/tkinter.json) | Structured tkinter data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/vocabulary.json](../../vials_synthesis/libraries/vocabulary.json) | Structured vocabulary data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/libraries/websockets.json](../../vials_synthesis/libraries/websockets.json) | Structured websockets data; top-level fields: `id`, `concept`, `facts`, `evidence`, `confidence`. |
| [vials_synthesis/libraries/x86_64_assembly.json](../../vials_synthesis/libraries/x86_64_assembly.json) | Structured x86 64 assembly data; top-level fields: `id`, `concept`, `confidence`, `evidence`, `facts`. |
| [vials_synthesis/patterns/academic_rules.json](../../vials_synthesis/patterns/academic_rules.json) | Structured academic rules data; top-level fields: `id`, `concept`, `rules`. |
| [vials_synthesis/patterns/container_app.json](../../vials_synthesis/patterns/container_app.json) | Structured container app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/cpp_gui_app.json](../../vials_synthesis/patterns/cpp_gui_app.json) | Structured cpp gui app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/ddia_rules.json](../../vials_synthesis/patterns/ddia_rules.json) | Structured ddia rules data; top-level fields: `id`, `concept`, `rules`. |
| [vials_synthesis/patterns/dependency_resolution.json](../../vials_synthesis/patterns/dependency_resolution.json) | Structured dependency resolution data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/framework_app.json](../../vials_synthesis/patterns/framework_app.json) | Structured framework app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/fsm.json](../../vials_synthesis/patterns/fsm.json) | Structured fsm data; top-level fields: `id`, `concept`, `facts`, `rules`, `evidence`. |
| [vials_synthesis/patterns/grid_layout.json](../../vials_synthesis/patterns/grid_layout.json) | Structured grid layout data; top-level fields: `id`, `concept`, `facts`, `rules`, `evidence`. |
| [vials_synthesis/patterns/gui_random_app.json](../../vials_synthesis/patterns/gui_random_app.json) | Structured gui random app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/mvc.json](../../vials_synthesis/patterns/mvc.json) | Structured mvc data; top-level fields: `id`, `concept`, `facts`, `rules`, `evidence`. |
| [vials_synthesis/patterns/polyglot_academic_app.json](../../vials_synthesis/patterns/polyglot_academic_app.json) | Structured polyglot academic app data; top-level fields: `id`, `concept`, `rules`. |
| [vials_synthesis/patterns/polyglot_calc_app.json](../../vials_synthesis/patterns/polyglot_calc_app.json) | Structured polyglot calc app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/polyglot_cli_app.json](../../vials_synthesis/patterns/polyglot_cli_app.json) | Structured polyglot cli app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/polyglot_db_app.json](../../vials_synthesis/patterns/polyglot_db_app.json) | Structured polyglot db app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/polyglot_ddia_app.json](../../vials_synthesis/patterns/polyglot_ddia_app.json) | Structured polyglot ddia app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/polyglot_game_app.json](../../vials_synthesis/patterns/polyglot_game_app.json) | Structured polyglot game app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/polyglot_gui_app.json](../../vials_synthesis/patterns/polyglot_gui_app.json) | Structured polyglot gui app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/polyglot_hybrid_app.json](../../vials_synthesis/patterns/polyglot_hybrid_app.json) | Structured polyglot hybrid app data; top-level fields: `id`, `concept`, `rules`. |
| [vials_synthesis/patterns/polyglot_multilang_app.json](../../vials_synthesis/patterns/polyglot_multilang_app.json) | Structured polyglot multilang app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/polyglot_multilib_app.json](../../vials_synthesis/patterns/polyglot_multilib_app.json) | Structured polyglot multilib app data; top-level fields: `id`, `concept`, `rules`. |
| [vials_synthesis/patterns/polyglot_ui_app.json](../../vials_synthesis/patterns/polyglot_ui_app.json) | Structured polyglot ui app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |
| [vials_synthesis/patterns/polyglot_web_app.json](../../vials_synthesis/patterns/polyglot_web_app.json) | Structured polyglot web app data; top-level fields: `id`, `concept`, `rules`, `evidence`. |

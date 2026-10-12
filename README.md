# DSCE — Deterministic Semantic Computation Engine

Explicit knowledge containers and spreading activation assemble inspectable reasoning chains in Python and Rust prototypes.

**Status:** Reasoning-engine research prototype  
**Built with:** Python, Rust, SQLite, Slint

[Project guide](docs/PROJECT_GUIDE.md) · [Every file explained](docs/FILE_GUIDE.md) · [Licenses and dependencies](docs/LICENSE_REVIEW.md)

## Start here

- [PROJECT_NOTES.md](PROJECT_NOTES.md) — Preserved detailed setup, usage, architecture, and development notes from the earlier README.
- [dsce](dsce) — Project entry point and supporting files
- [src](src) — Application source and platform integration
- [pyproject.toml](pyproject.toml) — Configuration or build metadata for pyproject; read alongside the application or tool that loads it.

[Detailed setup, usage, and development notes](PROJECT_NOTES.md) preserve the previous project documentation.

## Repository layout

| Path | Role |
| --- | --- |
| [LICENSES/](LICENSES/) | Component license texts (2 tracked entries). |
| [docs/](docs/) | Project documentation, decisions, and reference material (14 tracked entries). |
| [dsce/](dsce/) | Project-specific dsce components (9 tracked entries). |
| [scripts/](scripts/) | Build, launch, maintenance, and verification commands (5 tracked entries). |
| [src/](src/) | Application source and platform integration (14 tracked entries). |
| [tests/](tests/) | Automated checks and test fixtures (4 tracked entries). |
| [vials_synthesis/](vials_synthesis/) | Project-specific vials synthesis components (51 tracked entries). |

## Scope and current limits

The Python reference and Rust implementation are distinct entry points. The Slint GUI has its own licensing choices; the project Apache license does not replace them. The GCIDE-derived dictionary has separate GPL terms; docs/GCIDE_LICENSE.md records the source-version question and preserved license texts.

## License

Original project code retains **Apache-2.0** terms in [LICENSE](LICENSE). Third-party libraries, media, data, models, and tools retain their own terms. See the [license review](docs/LICENSE_REVIEW.md) and [dependency inventory](docs/DEPENDENCIES.csv).

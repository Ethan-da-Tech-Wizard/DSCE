# DSCE — Deterministic Semantic Computation Engine

Explicit knowledge containers and spreading activation assemble inspectable reasoning chains in Python and Rust prototypes.

**Status:** Reasoning-engine research prototype  
**Built with:** Python, Rust, SQLite, Slint

## Reading order

1. Read the root README for project purpose and setup navigation.
2. Follow the entry points below to find the implementation.
3. Use the file guide to locate individual modules, tests, assets, and configuration.
4. Check component licenses before redistributing dependencies or assets.

## Important paths

- [PROJECT_NOTES.md](../PROJECT_NOTES.md) — Preserved detailed setup, usage, architecture, and development notes from the earlier README.
- [dsce](../dsce) — Project-specific components
- [src](../src) — Application source and platform integration
- [pyproject.toml](../pyproject.toml) — Configuration or build metadata for pyproject; read alongside the application or tool that loads it.

## Structure

| Path | Role |
| --- | --- |
| [docs/](../docs/) | Project documentation, decisions, and reference material (13 tracked entries). |
| [dsce/](../dsce/) | Project-specific dsce components (9 tracked entries). |
| [scripts/](../scripts/) | Build, launch, maintenance, and verification commands (5 tracked entries). |
| [src/](../src/) | Application source and platform integration (14 tracked entries). |
| [tests/](../tests/) | Automated checks and test fixtures (4 tracked entries). |
| [vials_synthesis/](../vials_synthesis/) | Project-specific vials synthesis components (51 tracked entries). |

## Scope

The Python reference and Rust implementation are distinct entry points. The Slint GUI has its own licensing choices; the project Apache license does not replace them.

## Maintaining the guide

The file catalog is a source-inspection snapshot, not proof that every runtime path works. Update it when moving files or changing module responsibilities. The 2026-10-12 documentation review did not run every application, deploy services, or validate all historical performance claims.

# License review — DSCE — Deterministic Semantic Computation Engine

Reviewed 2026-10-12. Original project code retains **Apache-2.0** terms in [LICENSE](../LICENSE).

The owner selected MIT for previously unlicensed original code. Existing licenses were preserved. This choice does not relicense copied code, dependencies, datasets, model weights, fonts, logos, or game assets.

## Preserved notices

- [LICENSE](../LICENSE)
- [NOTICE](../NOTICE)

## Dependency evidence

[DEPENDENCIES.csv](DEPENDENCIES.csv) lists 605 manifest and lockfile records. Repeated packages can appear under different manifests, versions, or development scopes. Registry metadata is evidence, not a replacement for the actual license text. For unpinned ranges, `metadata_version` is the version inspected and is not a claim about every install. Go and system/build tools can require notices beyond package-manager metadata.

| Component | Declared version | License evidence |
| --- | --- | --- |
| crates.io: rusqlite | 0.32 | [MIT](https://crates.io/api/v1/crates/rusqlite) |
| crates.io: serde | 1 | [MIT OR Apache-2.0](https://crates.io/api/v1/crates/serde) |
| crates.io: serde_json | 1 | [MIT OR Apache-2.0](https://crates.io/api/v1/crates/serde_json) |
| crates.io: rayon | 1.10 | [MIT OR Apache-2.0](https://crates.io/api/v1/crates/rayon/1.12.0) |
| crates.io: ordered-float | 4 | [MIT](https://crates.io/api/v1/crates/ordered-float/4.6.0) |
| crates.io: slint | 1.6 | [GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0](https://crates.io/api/v1/crates/slint) |
| crates.io: slint-build | 1.6 | [GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0](https://crates.io/api/v1/crates/slint-build/1.17.1) |
| crates.io: tempfile | 3 | [MIT OR Apache-2.0](https://crates.io/api/v1/crates/tempfile) |
| PyPI: setuptools | >=61 | [MIT](https://pypi.org/pypi/setuptools/json) |

## Project-specific considerations

Slint offers GPL and alternative licensing routes. The Rust GUI must comply with the specific selected Slint terms; the Python reference’s Apache-2.0 license does not settle that choice. See [Slint licensing](https://slint.dev/pricing).

## Dictionary data

The English dictionary vial is derived from GCIDE, which is licensed GPL-3.0-or-later (see gcide.gnu.org.ua/license). It is therefore not stored in this repository and is listed in .gitignore. Run scripts/update_dictionary.py to generate it locally. Anyone who redistributes the generated file must comply with the GPL.

## Remaining work before binary distribution

No unresolved registry-license lookup remains in the listed package records. System packages, downloaded tools/models, optional plugins, release binaries, and externally obtained assets need a build-specific inventory. This source review is not a blanket legal certification or a claim that every release artifact has been audited.

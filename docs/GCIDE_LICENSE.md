# GCIDE dictionary provenance and license notices

The data in `vials_synthesis/english_dictionary.json` is derived from the GNU Collaborative International Dictionary of English (GCIDE), through [aviaryan/gcide-dictionary-json](https://github.com/aviaryan/gcide-dictionary-json). DSCE converts word definitions into its vial/fact structure with [scripts/update_dictionary.py](../scripts/update_dictionary.py). This is third-party dictionary content, separate from DSCE's Apache-2.0 application code.

The JSON conversion repository carries a GPL version 2 license text. Its license was inspected at revision `d47f769a805c91090ead554a3048e146306a91fd` and is preserved in [GCIDE-JSON-GPL-2.0.txt](../LICENSES/GCIDE-JSON-GPL-2.0.txt). That inspection revision does not establish which revision produced the existing DSCE dictionary.

[GCIDE's current upstream license statement](https://gcide.gnu.org.ua/license) specifies GPL-3.0-or-later. Its license text is preserved in [GCIDE-UPSTREAM-GPL-3.0.txt](../LICENSES/GCIDE-UPSTREAM-GPL-3.0.txt). The older converted source and the current upstream statement differ; confirm the exact source revision and applicable GPL version before bundling or redistributing the dataset. Do not label the dataset public domain, Apache-only, or MIT-only.

The original GCIDE contributors and the contributors to aviaryan/gcide-dictionary-json retain their rights. The source conversion implementation and inputs are available in the linked upstream repository. Retain these notices and the applicable GPL text with copies.

Reviewed 2026-10-12. This review corrects the previous public-domain label and preserves upstream license evidence. It changes the dataset's provenance metadata, not its definitions.

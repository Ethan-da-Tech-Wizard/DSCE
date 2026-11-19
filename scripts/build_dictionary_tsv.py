# ==============================================================================
# Program: build_dictionary_tsv.py
# Programmer: Ethan-da-Tech-Wizard
# Last Modified: November 18, 2026
# Purpose: Exports English dictionary vial definitions to tab-separated values.
#          Reads vials_synthesis/english_dictionary.json, filters for definition
#          predicates, normalizes whitespace and tabs, and writes clean (word, defn)
#          records to scratch/dictionary.tsv.
# ==============================================================================

# Run from the repo root; paths are relative. Offline: only reads/writes local files.
import json
import os

def export_to_tsv():
    """Reads dictionary JSON vial and writes normalized definitions to TSV."""
    json_path = "vials_synthesis/english_dictionary.json"
    tsv_path = "scratch/dictionary.tsv"
    
    # Step 1: Validate input JSON existence
    # Returns quietly (no exception) so callers can continue.
    if not os.path.exists(json_path):
        print(f"Error: {json_path} does not exist.")
        return
        
    print(f"Reading {json_path}...")
    with open(json_path, "r", encoding="utf-8") as f:
        data = json.load(f)
        
    # Each fact is a [word, predicate, text] triple.
    facts = data["facts"]
    print(f"Exporting {len(facts)} words to {tsv_path}...")
    
    # Step 2: Ensure destination directory exists and write tab-delimited records
    os.makedirs(os.path.dirname(tsv_path), exist_ok=True)
    with open(tsv_path, "w", encoding="utf-8") as f:
        for term, pred, definition in facts:
            # Only definition facts are exported.
            if pred == "definition":
                # Clean up newlines and tabs
                # Tabs and newlines would break the two-column TSV format, so they become spaces.
                word = term.replace("\t", " ").replace("\n", " ").strip().lower()
                defn = definition.replace("\t", " ").replace("\n", " ").strip()
                f.write(f"{word}\t{defn}\n")
                
    print("Export complete.")

if __name__ == "__main__":
    export_to_tsv()

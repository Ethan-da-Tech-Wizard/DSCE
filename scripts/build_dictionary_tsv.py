# ==============================================================================
# Program: build_dictionary_tsv.py
# Programmer: Ethan-da-Tech-Wizard
# Last Modified: November 18, 2026
# Purpose: Exports English dictionary vial definitions to tab-separated values.
#          Reads vials_synthesis/english_dictionary.json, filters for definition
#          predicates, normalizes whitespace and tabs, and writes clean (word, defn)
#          records to scratch/dictionary.tsv.
# ==============================================================================

import json
import os

def export_to_tsv():
    """Reads dictionary JSON vial and writes normalized definitions to TSV."""
    json_path = "vials_synthesis/english_dictionary.json"
    tsv_path = "scratch/dictionary.tsv"
    
    # Step 1: Validate input JSON existence
    if not os.path.exists(json_path):
        print(f"Error: {json_path} does not exist.")
        return
        
    print(f"Reading {json_path}...")
    with open(json_path, "r", encoding="utf-8") as f:
        data = json.load(f)
        
    facts = data["facts"]
    print(f"Exporting {len(facts)} words to {tsv_path}...")
    
    # Step 2: Ensure destination directory exists and write tab-delimited records
    os.makedirs(os.path.dirname(tsv_path), exist_ok=True)
    with open(tsv_path, "w", encoding="utf-8") as f:
        for term, pred, definition in facts:
            if pred == "definition":
                # Clean up newlines and tabs
                word = term.replace("\t", " ").replace("\n", " ").strip().lower()
                defn = definition.replace("\t", " ").replace("\n", " ").strip()
                f.write(f"{word}\t{defn}\n")
                
    print("Export complete.")

if __name__ == "__main__":
    export_to_tsv()

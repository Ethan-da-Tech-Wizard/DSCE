# Program: DSCE - build_dictionary_tsv.py
# Programmer: Ethan-da-Tech-Wizard (written with Claude, an AI assistant)
# Last Modified: 2026-11-18
# Purpose: Exports the english_dictionary vial's definition facts to a two-column word/definition TSV file.
"""Convert the English dictionary vial JSON into a tab-separated word/definition file."""

import json
import os

def export_to_tsv():
    """Read the dictionary vial JSON and write one word<TAB>definition line per definition fact."""
    json_path = "vials_synthesis/english_dictionary.json"
    tsv_path = "scratch/dictionary.tsv"
    
    if not os.path.exists(json_path):
        print(f"Error: {json_path} does not exist.")
        return
        
    print(f"Reading {json_path}...")
    with open(json_path, "r") as f:
        data = json.load(f)
        
    facts = data["facts"]
    print(f"Exporting {len(facts)} words to {tsv_path}...")
    
    os.makedirs(os.path.dirname(tsv_path), exist_ok=True)
    with open(tsv_path, "w") as f:
        for term, pred, definition in facts:
            if pred == "definition":
                # Clean up newlines and tabs
                word = term.replace("\t", " ").replace("\n", " ").strip().lower()
                defn = definition.replace("\t", " ").replace("\n", " ").strip()
                f.write(f"{word}\t{defn}\n")
                
    print("Export complete.")

if __name__ == "__main__":
    export_to_tsv()

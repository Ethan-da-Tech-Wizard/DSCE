# ==============================================================================
# Program: update_dictionary.py
# Programmer: Ethan-da-Tech-Wizard
# Last Modified: November 18, 2026
# Purpose: Updates DSCE English dictionary vial from upstream GCIDE JSON.
#          Fetches dictionary_minimal.json, formats word definitions into
#          ground triples, saves the updated vial, and triggers TSV export.
# ==============================================================================

# Network use: downloads one JSON file over HTTPS. Overwrites vials_synthesis/english_dictionary.json
# and then regenerates scratch/dictionary.tsv. Paths are relative to the current directory.
import urllib.request
import json
import os
import sys

def download_and_format():
    """Downloads upstream GCIDE JSON, normalizes definitions, and exports TSV."""
    # GCIDE (a free dictionary); license text is in docs/GCIDE_LICENSE.md.
    url = "https://raw.githubusercontent.com/aviaryan/gcide-dictionary-json/master/dictionary_minimal.json"
    json_dest = "vials_synthesis/english_dictionary.json"
    
    # Step 1: Download GCIDE payload over HTTPS
    print(f"Downloading GCIDE dictionary from {url}...")
    try:
        # 30 s timeout (same idea as download_dictionary.py) so a stalled connection cannot hang the script.
        response = urllib.request.urlopen(url, timeout=30)
        raw_data = response.read().decode('utf-8')
    except Exception as e:
        print(f"Error downloading dictionary: {e}")
        sys.exit(1)
        
    print("Parsing downloaded JSON...")
    raw_dict = json.loads(raw_data)
    
    # Step 2: Format into standard DSCE vial schema
    vial = {
        "id": "english_dictionary",
        "concept": "Collaborative International Dictionary of English (GCIDE)",
        "confidence": 1.0,
        "evidence": [
            "GCIDE-derived dictionary via aviaryan/gcide-dictionary-json; see docs/GCIDE_LICENSE.md for license texts and version provenance"
        ],
        "facts": []
    }
    
    print(f"Converting {len(raw_dict)} words into DSCE triples...")
    for word, definition in raw_dict.items():
        # Entries with an empty definition are dropped.
        if definition:
            defn = definition.strip()
            vial["facts"].append([word.lower(), "definition", defn])
            
    # Step 3: Serialize JSON vial to disk
    print(f"Writing to {json_dest}...")
    with open(json_dest, "w", encoding="utf-8") as f:
        json.dump(vial, f, indent=2)
        
    # Step 4: Rebuild TSV cache
    print("Vial update complete. Rebuilding TSV...")
    # Make the sibling module build_dictionary_tsv importable, even when run from another directory.
    sys.path.append(os.path.abspath(os.path.dirname(__file__)))
    import build_dictionary_tsv
    build_dictionary_tsv.export_to_tsv()

if __name__ == "__main__":
    download_and_format()

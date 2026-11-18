# ==============================================================================
# Program: download_dictionary.py
# Programmer: Ethan-da-Tech-Wizard
# Last Modified: November 18, 2026
# Purpose: Downloads and compiles English dictionary definitions into DSCE vial format.
#          Fetches dictionary definition JSON over HTTPS, formats definitions into
#          standard triples ([word, "definition", def_clean]), and saves the
#          resulting structured vial to vials_synthesis/english_dictionary.json.
# ==============================================================================

"""Download and Compile English Dictionary for DSCE.

Downloads a public English dictionary JSON file (~2MB, containing 80,000+ definitions),
formats it into a valid DSCE Vial format, and saves it to vials_synthesis/english_dictionary.json.
"""

import json
import os
import sys
import urllib.request

def download_and_compile():
    """Downloads remote dictionary JSON, cleans entries, and serializes DSCE vial."""
    url = "https://raw.githubusercontent.com/adambom/dictionary/master/dictionary.json"
    dest_vial = os.path.join("vials_synthesis", "english_dictionary.json")
    
    # Step 1: Download raw JSON payload
    print(f"[*] Downloading English dictionary from: {url}")
    try:
        with urllib.request.urlopen(url, timeout=15) as response:
            data = json.loads(response.read().decode("utf-8"))
    except Exception as e:
        print(f"[!] Network error downloading dictionary: {e}")
        sys.exit(1)
        
    print(f"[+] Downloaded dictionary with {len(data):,} words.")
    print("[*] Formatting definitions into DSCE facts...")
    
    # Step 2: Transform raw word/definition map into ground triples
    facts = []
    for word, definition in sorted(data.items()):
        word_clean = word.lower().strip()
        def_clean = definition.strip()
        if word_clean and def_clean:
            facts.append([word_clean, "definition", def_clean])
            
    vial = {
        "id": "english_dictionary",
        "concept": "Standard English Dictionary Definitions",
        "confidence": 1.0,
        "evidence": ["Webster's Unabridged Dictionary (JSON export)"],
        "facts": facts
    }
    
    # Step 3: Serialize formatted vial to destination path
    print(f"[*] Writing {len(facts):,} dictionary facts to: {dest_vial}")
    os.makedirs(os.path.dirname(dest_vial), exist_ok=True)
    with open(dest_vial, "w", encoding="utf-8") as f:
        json.dump(vial, f, indent=2)
        
    print("[+] Successfully compiled and loaded english_dictionary.json!")

if __name__ == "__main__":
    download_and_compile()

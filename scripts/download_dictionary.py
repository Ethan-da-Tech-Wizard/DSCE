"""Download and Compile English Dictionary for DSCE.

Downloads a public English dictionary JSON file (~2MB, containing 80,000+ definitions),
formats it into a valid DSCE Vial format, and saves it to vials_synthesis/english_dictionary.json.
"""

import json
import os
import sys
import urllib.request


def download_and_compile():
    url = "https://raw.githubusercontent.com/adambom/dictionary/master/dictionary.json"
    dest_vial = os.path.join("vials_synthesis", "english_dictionary.json")
    
    print(f"[*] Downloading English dictionary from: {url}")
    try:
        with urllib.request.urlopen(url, timeout=15) as response:
            data = json.loads(response.read().decode("utf-8"))
    except Exception as e:
        print(f"[!] Network error downloading dictionary: {e}")
        sys.exit(1)
        
    print(f"[+] Downloaded dictionary with {len(data):,} words.")
    print("[*] Formatting definitions into DSCE facts...")
    
    facts = []
    for word, definition in sorted(data.items()):
        word_clean = word.lower().strip()
        def_clean = definition.strip()
        # Clean up brackets and formatting characters in definition if any
        if word_clean and def_clean:
            facts.append([word_clean, "definition", def_clean])
            
    vial = {
        "id": "english_dictionary",
        "concept": "Standard English Dictionary Definitions",
        "confidence": 1.0,
        "evidence": ["Webster's Unabridged Dictionary (JSON export)"],
        "facts": facts
    }
    
    print(f"[*] Writing {len(facts):,} dictionary facts to: {dest_vial}")
    os.makedirs(os.path.dirname(dest_vial), exist_ok=True)
    with open(dest_vial, "w", encoding="utf-8") as f:
        json.dump(vial, f, indent=2)
        
    print("[+] Successfully compiled and loaded english_dictionary.json!")


if __name__ == "__main__":
    download_and_compile()

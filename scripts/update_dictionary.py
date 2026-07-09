import urllib.request
import json
import os
import sys

def download_and_format():
    url = "https://raw.githubusercontent.com/aviaryan/gcide-dictionary-json/master/dictionary_minimal.json"
    json_dest = "vials_synthesis/english_dictionary.json"
    
    print(f"Downloading GCIDE dictionary from {url}...")
    try:
        response = urllib.request.urlopen(url)
        raw_data = response.read().decode('utf-8')
    except Exception as e:
        print(f"Error downloading dictionary: {e}")
        sys.exit(1)
        
    print("Parsing downloaded JSON...")
    raw_dict = json.loads(raw_data)
    
    # Format to DSCE vial structure
    vial = {
        "id": "english_dictionary",
        "concept": "Collaborative International Dictionary of English (GCIDE)",
        "confidence": 1.0,
        "evidence": [
            "GCIDE Public Domain Dictionary, parsed via aviaryan/gcide-dictionary-json"
        ],
        "facts": []
    }
    
    print(f"Converting {len(raw_dict)} words into DSCE triples...")
    for word, definition in raw_dict.items():
        if definition:
            defn = definition.strip()
            # Clean up potential escape/formatting quirks
            vial["facts"].append([word.lower(), "definition", defn])
            
    print(f"Writing to {json_dest}...")
    with open(json_dest, "w") as f:
        json.dump(vial, f, indent=2)
        
    print("Vial update complete. Rebuilding TSV...")
    # Import and run tsv builder
    sys.path.append(os.path.abspath(os.path.dirname(__file__)))
    import build_dictionary_tsv
    build_dictionary_tsv.export_to_tsv()

if __name__ == "__main__":
    download_and_format()

"""Ingestion script to seed DSCE SQLite database with GeoNames cities data.

Downloads cities with population > 15,000 from download.geonames.org,
unzips, parses the TSV database, groups cities by country, and saves them
as modular regional Vials in SQLite.
"""

import os
import sys
import zipfile
import urllib.request
import sqlite3

# Add project root to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from dsce.db_store import SqliteVialStore
from dsce.vial import Vial

GEONAMES_URL = "http://download.geonames.org/export/dump/cities15000.zip"
ZIP_FILE = "cities15000.zip"
TXT_FILE = "cities15000.txt"
DB_PATH = "dsce.sqlite"

# Map common country codes to full names for more readable facts
COUNTRY_MAP = {
    "US": "united states",
    "CA": "canada",
    "GB": "united kingdom",
    "FR": "france",
    "DE": "germany",
    "JP": "japan",
    "CN": "china",
    "IN": "india",
    "BR": "brazil",
    "RU": "russia",
    "AU": "australia",
    "MX": "mexico",
    "IT": "italy",
    "ES": "spain",
}

def download_and_extract():
    if not os.path.exists(TXT_FILE):
        print(f"[*] Downloading GeoNames cities15000 database from {GEONAMES_URL}...")
        urllib.request.urlretrieve(GEONAMES_URL, ZIP_FILE)
        print("[*] Unzipping dataset...")
        with zipfile.ZipFile(ZIP_FILE, 'r') as zip_ref:
            zip_ref.extractall(".")
        print("[+] Dataset extracted successfully.")
    else:
        print("[*] GeoNames dataset text file already exists. Skipping download.")

def seed_db():
    download_and_extract()
    
    store = SqliteVialStore(DB_PATH)
    vial_groups = {}
    
    print("[*] Reading cities15000 database and parsing facts...")
    count = 0
    with open(TXT_FILE, "r", encoding="utf-8") as f:
        for line in f:
            parts = line.strip().split("\t")
            if len(parts) < 15:
                continue
            
            # Extract fields (GeoNames schema)
            city_name = parts[2].strip()  # asciiname
            country_code = parts[8].strip()  # country code
            state_code = parts[10].strip()  # admin1 code
            
            if not city_name or not country_code:
                continue
                
            country_name = COUNTRY_MAP.get(country_code, country_code.lower())
            vial_id = f"geo_{country_code.lower()}"
            
            if vial_id not in vial_groups:
                vial_groups[vial_id] = {
                    "concept": f"Geographical facts for {country_name.upper()}",
                    "facts": set(),
                    "evidence": "GeoNames cities15000 Database (geonames.org)"
                }
                
            # Add geographical facts
            city_key = city_name.lower()
            state_key = f"{state_code.lower()}_{country_code.lower()}" if state_code else country_name
            
            # e.g., ("modesto", "is_a", "city")
            vial_groups[vial_id]["facts"].add((city_key, "is_a", "city"))
            # e.g., ("modesto", "located_in", "us")
            vial_groups[vial_id]["facts"].add((city_key, "located_in", country_name))
            
            count += 1
            if count % 5000 == 0:
                print(f"    Processed {count} cities...")

    print(f"[+] Processed {count} cities total.")
    print(f"[*] Compiling and bulk-saving {len(vial_groups)} regional Vials to SQLite...")
    
    for vial_id, data in vial_groups.items():
        vial = Vial(
            id=vial_id,
            concept=data["concept"],
            facts=tuple(sorted(list(data["facts"]))),
            rules=(),
            neighbors=(),
            evidence=(data["evidence"],),
            confidence=1.0
        )
        store.save_vial(vial)
        
    print("[+] Seeding complete. All cities successfully loaded and indexed in SQLite.")

    # Clean up zip and extracted txt files to keep workspace tidy
    try:
        os.remove(ZIP_FILE)
        os.remove(TXT_FILE)
        print("[*] Cleaned up temporary download files.")
    except OSError:
        pass

if __name__ == "__main__":
    seed_db()

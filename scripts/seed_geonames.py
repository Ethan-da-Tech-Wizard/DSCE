# ==============================================================================
# Program: seed_geonames.py
# Programmer: Ethan-da-Tech-Wizard
# Last Modified: November 18, 2026
# Purpose: Seeds DSCE SQLite database with GeoNames global cities data.
#          Downloads and extracts global cities with populations > 15,000,
#          parses demographic and geographical coordinates from TSV, groups
#          cities into country-specific modular Vials, and saves to SQLite.
# ==============================================================================

"""Ingestion script to seed DSCE SQLite database with GeoNames cities data.

Downloads cities with population > 15,000 from download.geonames.org,
unzips, parses the TSV database, groups cities by country, and saves them
as modular regional Vials in SQLite.
"""

# Network use: this script downloads from download.geonames.org (plain HTTP).
# Run it only when that is acceptable; the extracted .txt is cached afterwards.
import os
import sys
import zipfile
import urllib.request
# (Imported but not used in this file.)
import sqlite3

# Add project root to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from dsce.db_store import SqliteVialStore
from dsce.vial import Vial

# cities15000 = every city with population above 15,000.
GEONAMES_URL = "https://download.geonames.org/export/dump/cities15000.zip"
# Downloaded and extracted into the current working directory.
ZIP_FILE = "cities15000.zip"
TXT_FILE = "cities15000.txt"
DB_PATH = "dsce.sqlite"

# Map common country codes to full names for more readable facts
# Only these 14 countries get readable names; any other code falls back to its lowercase ISO code (e.g. "nl").
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
    """Fetches and unzips GeoNames archive if not already locally cached."""
    # Cache check: skip the download if the extracted text file is already present.
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
    """Parses extracted GeoNames cities and commits regional vials into SQLite."""
    # Step 1: Ensure dataset is available locally
    download_and_extract()
    
    store = SqliteVialStore(DB_PATH)
    vial_groups = {}
    
    print("[*] Reading cities15000 database and parsing facts...")
    total_cities = 0
    
    # Step 2: Stream TSV rows and group triples by country code
    with open(TXT_FILE, "r", encoding="utf-8") as f:
        for line in f:
            parts = line.strip().split("\t")
            # GeoNames rows have 19 tab-separated columns; we need up to index 14 (population), so shorter rows are malformed.
            if len(parts) < 15:
                continue
            
            # Column indexes used: 1 = name, 8 = country code, 14 = population.
            city_name = parts[1].strip().lower()
            country_code = parts[8].strip().upper()
            country_name = COUNTRY_MAP.get(country_code, country_code.lower())
            
            try:
                population = int(parts[14])
            except ValueError:
                population = 0
                
            # One vial per country keeps activation sparse: asking about a US city wakes only the US vial.
            group_key = f"geo_{country_name.replace(' ', '_')}"
            if group_key not in vial_groups:
                vial_groups[group_key] = []
                
            vial_groups[group_key].append((city_name, "is_a", "city"))
            vial_groups[group_key].append((city_name, "located_in", country_name))
            # Zero means unknown in GeoNames, so no population fact is recorded.
            if population > 0:
                vial_groups[group_key].append((city_name, "population", population))
            total_cities += 1

    print(f"[+] Loaded {total_cities:,} cities across {len(vial_groups)} country groups.")
    print("[*] Writing modular regional Vials into SQLite database...")
    
    # Step 3: Commit each country group as an independent vial
    for vial_id, facts in vial_groups.items():
        vial = Vial(
            id=vial_id,
            concept=f"Geographical facts for {vial_id.replace('geo_', '').replace('_', ' ').title()}",
            facts=tuple(facts),
            evidence=("download.geonames.org cities15000 dataset",),
            # Slightly below 1.0: crowd-sourced data can contain errors.
            confidence=0.98
        )
        store.save_vial(vial)
        
    print(f"[+] Successfully seeded {len(vial_groups)} vials into {DB_PATH}!")

if __name__ == "__main__":
    seed_db()

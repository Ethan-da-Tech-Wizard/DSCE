"""Ingestion script to seed DSCE SQLite database with taxonomy and grammar data.

Seeds:
1. Biological Taxonomy: kingdoms, classes, and representative species for 
   Animalia, Plantae, Fungi, and Bacteria.
2. Grammar Lexicon: common words mapped to their parts of speech.
3. Grammar Rules: syntactic rules using flat triple representation.
"""

import os
import sys

# Add project root to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from dsce.db_store import SqliteVialStore
from dsce.vial import Vial, Rule


def seed_taxonomy_and_grammar():
    db_path = "dsce.sqlite"
    store = SqliteVialStore(db_path)
    
    print("[*] Preparing biological taxonomy data...")
    
    # 1. Animals Vial (tax_animalia)
    animal_facts = (
        ("animalia", "is_a", "kingdom"),
        ("mammalia", "located_in", "animalia"),
        ("mammalia", "is_a", "class"),
        ("carnivora", "located_in", "mammalia"),
        ("carnivora", "is_a", "order"),
        ("felidae", "located_in", "carnivora"),
        ("felidae", "is_a", "family"),
        ("panthera", "located_in", "felidae"),
        ("panthera", "is_a", "genus"),
        ("lion", "located_in", "panthera"),
        ("lion", "is_a", "species"),
        ("tiger", "located_in", "panthera"),
        ("tiger", "is_a", "species"),
        ("canidae", "located_in", "carnivora"),
        ("canidae", "is_a", "family"),
        ("canis", "located_in", "canidae"),
        ("canis", "is_a", "genus"),
        ("dog", "located_in", "canis"),
        ("dog", "is_a", "species"),
        ("wolf", "located_in", "canis"),
        ("wolf", "is_a", "species"),
        ("primates", "located_in", "mammalia"),
        ("primates", "is_a", "order"),
        ("hominidae", "located_in", "primates"),
        ("hominidae", "is_a", "family"),
        ("homo", "located_in", "hominidae"),
        ("homo", "is_a", "genus"),
        ("human", "located_in", "homo"),
        ("human", "is_a", "species"),
    )
    vial_animals = Vial(
        id="tax_animalia",
        concept="Biological taxonomy for Kingdom Animalia",
        facts=animal_facts,
        evidence=("Catalogue of Life (catalogueoflife.org)",),
        confidence=1.0
    )
    store.save_vial(vial_animals)
    print("  [+] Seeded 'tax_animalia'")

    # 2. Plants Vial (tax_plantae)
    plant_facts = (
        ("plantae", "is_a", "kingdom"),
        ("coniferophyta", "located_in", "plantae"),
        ("coniferophyta", "is_a", "division"),
        ("pinopsida", "located_in", "coniferophyta"),
        ("pinopsida", "is_a", "class"),
        ("pinaceae", "located_in", "pinopsida"),
        ("pinaceae", "is_a", "family"),
        ("pinus", "located_in", "pinaceae"),
        ("pinus", "is_a", "genus"),
        ("pine tree", "located_in", "pinus"),
        ("pine tree", "is_a", "species"),
        ("magnoliophyta", "located_in", "plantae"),
        ("magnoliophyta", "is_a", "division"),
        ("magnoliopsida", "located_in", "magnoliophyta"),
        ("magnoliopsida", "is_a", "class"),
        ("rosaceae", "located_in", "magnoliopsida"),
        ("rosaceae", "is_a", "family"),
        ("rosa", "located_in", "rosaceae"),
        ("rosa", "is_a", "genus"),
        ("rose", "located_in", "rosa"),
        ("rose", "is_a", "species"),
    )
    vial_plants = Vial(
        id="tax_plantae",
        concept="Biological taxonomy for Kingdom Plantae",
        facts=plant_facts,
        evidence=("Catalogue of Life (catalogueoflife.org)",),
        confidence=1.0
    )
    store.save_vial(vial_plants)
    print("  [+] Seeded 'tax_plantae'")

    # 3. Fungi Vial (tax_fungi)
    fungi_facts = (
        ("fungi", "is_a", "kingdom"),
        ("ascomycota", "located_in", "fungi"),
        ("ascomycota", "is_a", "phylum"),
        ("saccharomycetes", "located_in", "ascomycota"),
        ("saccharomycetes", "is_a", "class"),
        ("saccharomycetales", "located_in", "saccharomycetes"),
        ("saccharomycetales", "is_a", "order"),
        ("saccharomycetaceae", "located_in", "saccharomycetes"),
        ("saccharomyces", "located_in", "saccharomycetaceae"),
        ("saccharomyces", "is_a", "genus"),
        ("yeast", "located_in", "saccharomyces"),
        ("yeast", "is_a", "species"),
    )
    vial_fungi = Vial(
        id="tax_fungi",
        concept="Biological taxonomy for Kingdom Fungi",
        facts=fungi_facts,
        evidence=("Catalogue of Life (catalogueoflife.org)",),
        confidence=1.0
    )
    store.save_vial(vial_fungi)
    print("  [+] Seeded 'tax_fungi'")

    # 4. Bacteria Vial (tax_bacteria)
    bacteria_facts = (
        ("bacteria", "is_a", "kingdom"),
        ("pseudomonadota", "located_in", "bacteria"),
        ("pseudomonadota", "is_a", "phylum"),
        ("gammaproteobacteria", "located_in", "pseudomonadota"),
        ("gammaproteobacteria", "is_a", "class"),
        ("enterobacterales", "located_in", "gammaproteobacteria"),
        ("enterobacterales", "is_a", "order"),
        ("enterobacteriaceae", "located_in", "enterobacterales"),
        ("enterobacteriaceae", "is_a", "family"),
        ("escherichia", "located_in", "enterobacteriaceae"),
        ("escherichia", "is_a", "genus"),
        ("e. coli", "located_in", "escherichia"),
        ("e. coli", "is_a", "species"),
    )
    vial_bacteria = Vial(
        id="tax_bacteria",
        concept="Biological taxonomy for Kingdom Bacteria",
        facts=bacteria_facts,
        evidence=("NCBI Taxonomy Database",),
        confidence=1.0
    )
    store.save_vial(vial_bacteria)
    print("  [+] Seeded 'tax_bacteria'")

    print("[*] Preparing English grammar and linguistics data...")
    
    # 5. Grammar Lexicon Vial (grammar_lexicon)
    lexicon_facts = (
        # Articles
        ("the", "part_of_speech", "article"),
        ("a", "part_of_speech", "article"),
        ("an", "part_of_speech", "article"),
        
        # Nouns
        ("dog", "part_of_speech", "noun"),
        ("cat", "part_of_speech", "noun"),
        ("human", "part_of_speech", "noun"),
        ("teacher", "part_of_speech", "noun"),
        ("student", "part_of_speech", "noun"),
        ("apple", "part_of_speech", "noun"),
        ("running", "part_of_speech", "noun"),
        
        # Verbs
        ("runs", "part_of_speech", "verb"),
        ("chases", "part_of_speech", "verb"),
        ("eats", "part_of_speech", "verb"),
        ("is", "part_of_speech", "verb"),
        ("studies", "part_of_speech", "verb"),
        
        # Adjectives
        ("happy", "part_of_speech", "adjective"),
        ("quick", "part_of_speech", "adjective"),
        ("lazy", "part_of_speech", "adjective"),
        ("green", "part_of_speech", "adjective"),
        ("smart", "part_of_speech", "adjective"),
        
        # Adverbs
        ("quickly", "part_of_speech", "adverb"),
        ("lazily", "part_of_speech", "adverb"),
        ("happily", "part_of_speech", "adverb"),
        
        # Pronouns
        ("he", "part_of_speech", "pronoun"),
        ("she", "part_of_speech", "pronoun"),
        ("it", "part_of_speech", "pronoun"),
        ("they", "part_of_speech", "pronoun"),
    )
    vial_lexicon = Vial(
        id="grammar_lexicon",
        concept="English grammar vocabulary and parts of speech",
        facts=lexicon_facts,
        evidence=("WordNet Lexical Database",),
        confidence=1.0
    )
    store.save_vial(vial_lexicon)
    print("  [+] Seeded 'grammar_lexicon'")

    # 6. Grammar Rules Vial (grammar_rules)
    grammar_rules = (
        # Rule: A standalone noun forms a simple noun phrase
        Rule(
            name="simple-noun-phrase",
            premises=(
                ("?noun", "part_of_speech", "noun"),
            ),
            conclusion=("?noun", "is_a", "noun_phrase")
        ),
        # Rule: An article followed by a noun forms a noun phrase
        Rule(
            name="article-noun-phrase",
            premises=(
                ("?phrase", "word_1", "?art"),
                ("?phrase", "word_2", "?noun"),
                ("?art", "part_of_speech", "article"),
                ("?noun", "part_of_speech", "noun"),
            ),
            conclusion=("?phrase", "is_a", "noun_phrase")
        ),
        # Rule: An article followed by an adjective and a noun forms a noun phrase
        Rule(
            name="article-adjective-noun-phrase",
            premises=(
                ("?phrase", "word_1", "?art"),
                ("?phrase", "word_2", "?adj"),
                ("?phrase", "word_3", "?noun"),
                ("?art", "part_of_speech", "article"),
                ("?adj", "part_of_speech", "adjective"),
                ("?noun", "part_of_speech", "noun"),
            ),
            conclusion=("?phrase", "is_a", "noun_phrase")
        ),
        # Rule: A standalone verb forms a simple verb phrase
        Rule(
            name="simple-verb-phrase",
            premises=(
                ("?verb", "part_of_speech", "verb"),
            ),
            conclusion=("?verb", "is_a", "verb_phrase")
        ),
        # Rule: A verb followed by an adverb forms a verb phrase
        Rule(
            name="verb-adverb-phrase",
            premises=(
                ("?phrase", "word_1", "?verb"),
                ("?phrase", "word_2", "?adv"),
                ("?verb", "part_of_speech", "verb"),
                ("?adv", "part_of_speech", "adverb"),
            ),
            conclusion=("?phrase", "is_a", "verb_phrase")
        ),
        # Rule: A sentence structure is NP + VP
        Rule(
            name="sentence-structure",
            premises=(
                ("?sentence", "subject", "?np"),
                ("?sentence", "predicate", "?vp"),
                ("?np", "is_a", "noun_phrase"),
                ("?vp", "is_a", "verb_phrase"),
            ),
            conclusion=("?sentence", "has_structure", "subject_predicate")
        ),
        # Rule: A verb ending in "ing" acting as a noun is a gerund
        Rule(
            name="gerund-phrase-identification",
            premises=(
                ("?word", "part_of_speech", "noun"),
            ),
            conclusion=("?word", "is_a_gerund", "?is_gerund"),
            compute=lambda b: {"?is_gerund": str(b["?word"]).endswith("ing")}
        ),
    )
    vial_rules = Vial(
        id="grammar_rules",
        concept="Syntactic rules of English grammar",
        rules=grammar_rules,
        evidence=("Chomsky, Syntactic Structures",),
        confidence=1.0
    )
    store.save_vial(vial_rules)
    print("  [+] Seeded 'grammar_rules'")
    
    print("[*] Ingestion complete. Taxonomy and grammar Vials successfully seeded in SQLite.")


if __name__ == "__main__":
    seed_taxonomy_and_grammar()

#!/usr/bin/env python3
"""
Fetch 20 curated Commander decklists from Archidekt and generate analysis files.

Usage: python fetch_decks.py [--skip-fetch] [--definitions-path PATH]

  --skip-fetch         Skip API calls, only regenerate analysis from existing JSONs
  --definitions-path   Path to definitions.rs (default: auto-detected from script location)

Output files:
  NN_commander-slug.json       Per-deck card lists (20 files)
  _summary.json                Deck-level stats
  _unique_cards.json           Deduplicated cards across all decks, sorted by frequency
  _cards_needing_definitions.json  Cards missing from definitions.rs, sorted by frequency
"""

import json
import re
import sys
import time
import urllib.request
from pathlib import Path

# ---------------------------------------------------------------------------
# Deck configuration: 20 curated Commander decks from Archidekt
# ---------------------------------------------------------------------------

DECKS = [
    {
        "number": 1,
        "commander": "Atraxa, Praetors' Voice",
        "colors": ["W", "U", "B", "G"],
        "archetype": "Superfriends/+1",
        "archidekt_id": 4752046,
    },
    {
        "number": 2,
        "commander": "Krenko, Mob Boss",
        "colors": ["R"],
        "archetype": "Tribal Goblin",
        "archidekt_id": 3880705,
    },
    {
        "number": 3,
        "commander": "Yuriko, the Tiger's Shadow",
        "colors": ["U", "B"],
        "archetype": "Ninja Tribal",
        "archidekt_id": 4386,
    },
    {
        "number": 4,
        "commander": "Meren of Clan Nel Toth",
        "colors": ["B", "G"],
        "archetype": "Aristocrats/Reanimator",
        "archidekt_id": 8063127,
    },
    {
        "number": 5,
        "commander": "Prossh, Skyraider of Kher",
        "colors": ["B", "R", "G"],
        "archetype": "Sacrifice/Combo",
        "archidekt_id": 630,
    },
    {
        "number": 6,
        "commander": "Edgar Markov",
        "colors": ["W", "B", "R"],
        "archetype": "Tribal Vampire",
        "archidekt_id": 59349,
    },
    {
        "number": 7,
        "commander": "Omnath, Locus of Creation",
        "colors": ["W", "U", "R", "G"],
        "archetype": "Landfall",
        "archidekt_id": 4973670,
    },
    {
        "number": 8,
        "commander": "Korvold, Fae-Cursed King",
        "colors": ["B", "R", "G"],
        "archetype": "Sacrifice Value",
        "archidekt_id": 4720142,
    },
    {
        "number": 9,
        "commander": "Teysa Karlov",
        "colors": ["W", "B"],
        "archetype": "Death Triggers",
        "archidekt_id": 1536180,
    },
    {
        "number": 10,
        "commander": "Winota, Joiner of Forces",
        "colors": ["W", "R"],
        "archetype": "Aggro/Cheat",
        "archidekt_id": 15585118,
    },
    {
        "number": 11,
        "commander": "Talrand, Sky Summoner",
        "colors": ["U"],
        "archetype": "Spellslinger",
        "archidekt_id": 3995,
    },
    {
        "number": 12,
        "commander": "Selvala, Heart of the Wilds",
        "colors": ["G"],
        "archetype": "Mono-Green Ramp",
        "archidekt_id": 5150813,
    },
    {
        "number": 13,
        "commander": "Syr Konrad, the Grim",
        "colors": ["B"],
        "archetype": "Graveyard/Drain",
        "archidekt_id": 1580608,
    },
    {
        "number": 14,
        "commander": "Sram, Senior Edificer",
        "colors": ["W"],
        "archetype": "Voltron/Equipment",
        "archidekt_id": 773070,
    },
    {
        "number": 15,
        "commander": "The Ur-Dragon",
        "colors": ["W", "U", "B", "R", "G"],
        "archetype": "Tribal Dragon",
        "archidekt_id": 4153743,
    },
    {
        "number": 16,
        "commander": "Niv-Mizzet, Parun",
        "colors": ["U", "R"],
        "archetype": "Spellslinger/Combo",
        "archidekt_id": 57934,
    },
    {
        "number": 17,
        "commander": "Lathril, Blade of the Elves",
        "colors": ["B", "G"],
        "archetype": "Tribal Elf",
        "archidekt_id": 2781081,
    },
    {
        "number": 18,
        "commander": "Chulane, Teller of Tales",
        "colors": ["W", "U", "G"],
        "archetype": "Value/Bounce",
        "archidekt_id": 921936,
    },
    {
        "number": 19,
        "commander": "Isshin, Two Heavens as One",
        "colors": ["W", "B", "R"],
        "archetype": "Attack Triggers",
        "archidekt_id": 2400793,
    },
    {
        "number": 20,
        "commander": "Aesi, Tyrant of Gyre Strait",
        "colors": ["U", "G"],
        "archetype": "Landfall/Draw",
        "archidekt_id": 6349098,
    },
]

API_BASE = "https://archidekt.com/api/decks"
USER_AGENT = "Mozilla/5.0"
REQUEST_DELAY = 0.5  # seconds between API calls

SCRIPT_DIR = Path(__file__).resolve().parent
DEFINITIONS_PATH_DEFAULT = (
    SCRIPT_DIR / ".." / ".." / "crates" / "engine" / "src" / "cards" / "definitions.rs"
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def slugify(name: str) -> str:
    """Convert commander name to a filename slug."""
    s = name.lower()
    s = s.replace("'", "")
    s = s.replace(",", "")
    s = re.sub(r"[^a-z0-9]+", "-", s)
    s = s.strip("-")
    return s


def fetch_json(url: str) -> dict:
    """Fetch JSON from a URL."""
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read())


def extract_card(entry: dict) -> dict:
    """Extract the fields we care about from an Archidekt card entry."""
    oracle = entry["card"]["oracleCard"]
    is_commander = "Commander" in (entry.get("categories") or [])

    card = {
        "name": oracle["name"],
        "quantity": entry.get("quantity", 1),
        "types": oracle.get("types", []),
        "mana_cost": oracle.get("manaCost", ""),
        "cmc": oracle.get("cmc", 0),
        "color_identity": oracle.get("colorIdentity", []),
        "commander": is_commander,
        "oracle_text": oracle.get("text", ""),
        "keywords": oracle.get("keywords", []),
    }

    # Add subtypes if present
    sub = oracle.get("subTypes", [])
    if sub:
        card["subtypes"] = sub

    # Add supertypes if present
    sup = oracle.get("superTypes", [])
    if sup:
        card["supertypes"] = sup

    # Add P/T for creatures
    if oracle.get("power") is not None:
        card["power"] = oracle["power"]
    if oracle.get("toughness") is not None:
        card["toughness"] = oracle["toughness"]

    # Add loyalty for planeswalkers
    if oracle.get("loyalty") is not None:
        card["loyalty"] = oracle["loyalty"]

    return card


def get_defined_card_names(definitions_path: Path) -> set[str]:
    """Parse definitions.rs to extract the set of card names already defined."""
    if not definitions_path.exists():
        print(f"  warning: {definitions_path} not found, skipping gap analysis", file=sys.stderr)
        return set()

    text = definitions_path.read_text()
    # Match: name: "Card Name".to_string()
    return set(re.findall(r'name:\s*"([^"]+)"\.to_string\(\)', text))


# ---------------------------------------------------------------------------
# Fetch phase
# ---------------------------------------------------------------------------


def fetch_deck(deck_config: dict) -> dict | None:
    """Fetch a single deck from Archidekt and return our normalized format."""
    deck_id = deck_config["archidekt_id"]
    url = f"{API_BASE}/{deck_id}/"
    print(f"  fetching {url}", file=sys.stderr)

    try:
        data = fetch_json(url)
    except Exception as e:
        print(f"  ERROR fetching deck {deck_id}: {e}", file=sys.stderr)
        return None

    if "cards" not in data:
        print(f"  ERROR: no 'cards' key in response for deck {deck_id}", file=sys.stderr)
        return None

    # Filter out maybeboard and sideboard
    main_entries = [
        c for c in data["cards"]
        if "Maybeboard" not in (c.get("categories") or [])
        and "Sideboard" not in (c.get("categories") or [])
    ]

    cards = [extract_card(e) for e in main_entries]
    total = sum(c["quantity"] for c in cards)

    if total != 100:
        print(f"  warning: deck {deck_id} has {total} cards (expected 100)", file=sys.stderr)

    return {
        "commander": deck_config["commander"],
        "colors": deck_config["colors"],
        "archetype": deck_config["archetype"],
        "source": "archidekt",
        "source_url": f"https://archidekt.com/decks/{deck_id}",
        "archidekt_id": deck_id,
        "fetched_date": time.strftime("%Y-%m-%d"),
        "card_count": total,
        "cards": cards,
    }


def fetch_all_decks() -> list[dict]:
    """Fetch all 20 decks, writing each to its JSON file."""
    results = []

    for i, deck_config in enumerate(DECKS):
        num = deck_config["number"]
        slug = slugify(deck_config["commander"])
        filename = f"{num:02d}_{slug}.json"
        filepath = SCRIPT_DIR / filename

        print(f"[{i+1}/{len(DECKS)}] {deck_config['commander']}", file=sys.stderr)

        deck_data = fetch_deck(deck_config)
        if deck_data is None:
            print(f"  SKIPPED (fetch failed)", file=sys.stderr)
            continue

        with open(filepath, "w") as f:
            json.dump(deck_data, f, indent=2, ensure_ascii=False)
        print(f"  -> {filename} ({deck_data['card_count']} cards)", file=sys.stderr)

        results.append(deck_data)

        if i < len(DECKS) - 1:
            time.sleep(REQUEST_DELAY)

    return results


def load_existing_decks() -> list[dict]:
    """Load previously fetched deck JSONs (for --skip-fetch mode)."""
    results = []
    for deck_config in DECKS:
        num = deck_config["number"]
        slug = slugify(deck_config["commander"])
        filename = f"{num:02d}_{slug}.json"
        filepath = SCRIPT_DIR / filename

        if filepath.exists():
            with open(filepath) as f:
                results.append(json.load(f))
        else:
            print(f"  warning: {filename} not found, skipping", file=sys.stderr)

    return results


# ---------------------------------------------------------------------------
# Analysis phase
# ---------------------------------------------------------------------------


def generate_summary(decks: list[dict]) -> dict:
    """Generate _summary.json with deck-level stats."""
    summary = {
        "total_decks": len(decks),
        "total_cards": sum(d["card_count"] for d in decks),
        "decks": [],
    }

    for d in decks:
        commander_cards = [c for c in d["cards"] if c["commander"]]
        summary["decks"].append({
            "commander": d["commander"],
            "colors": d["colors"],
            "archetype": d["archetype"],
            "card_count": d["card_count"],
            "source_url": d["source_url"],
            "creature_count": sum(
                c["quantity"] for c in d["cards"] if "Creature" in c.get("types", [])
            ),
            "land_count": sum(
                c["quantity"] for c in d["cards"] if "Land" in c.get("types", [])
            ),
            "instant_sorcery_count": sum(
                c["quantity"] for c in d["cards"]
                if "Instant" in c.get("types", []) or "Sorcery" in c.get("types", [])
            ),
        })

    return summary


def generate_unique_cards(decks: list[dict], defined_names: set[str]) -> dict:
    """Generate the unique card list with frequency counts and definition status."""
    # Aggregate card data across all decks
    card_data: dict[str, dict] = {}  # name -> accumulated info

    for deck_idx, d in enumerate(decks):
        for c in d["cards"]:
            name = c["name"]
            if name not in card_data:
                card_data[name] = {
                    "name": name,
                    "appears_in_decks": 0,
                    "deck_names": [],
                    "types": c.get("types", []),
                    "subtypes": c.get("subtypes", []),
                    "mana_cost": c.get("mana_cost", ""),
                    "cmc": c.get("cmc", 0),
                    "keywords": c.get("keywords", []),
                    "oracle_text": c.get("oracle_text", ""),
                    "has_definition": name in defined_names,
                }
            card_data[name]["appears_in_decks"] += 1
            card_data[name]["deck_names"].append(d["commander"])

    # Sort by frequency descending, then alphabetically
    sorted_cards = sorted(
        card_data.values(),
        key=lambda x: (-x["appears_in_decks"], x["name"]),
    )

    total = len(sorted_cards)
    with_defs = sum(1 for c in sorted_cards if c["has_definition"])

    return {
        "total_unique_cards": total,
        "total_with_definitions": with_defs,
        "total_needing_definitions": total - with_defs,
        "cards": sorted_cards,
    }


def generate_cards_needing_definitions(unique_cards: dict) -> dict:
    """Filter unique cards to only those missing definitions."""
    missing = [c for c in unique_cards["cards"] if not c["has_definition"]]

    # Group by tier
    tier1 = [c for c in missing if c["appears_in_decks"] >= 10]
    tier2 = [c for c in missing if 5 <= c["appears_in_decks"] <= 9]
    tier3 = [c for c in missing if 2 <= c["appears_in_decks"] <= 4]
    tier4 = [c for c in missing if c["appears_in_decks"] == 1]

    return {
        "total_needing_definitions": len(missing),
        "tier1_10plus_decks": len(tier1),
        "tier2_5to9_decks": len(tier2),
        "tier3_2to4_decks": len(tier3),
        "tier4_1_deck": len(tier4),
        "cards": missing,
    }


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main():
    skip_fetch = "--skip-fetch" in sys.argv

    # Find definitions.rs
    definitions_path = DEFINITIONS_PATH_DEFAULT
    for i, arg in enumerate(sys.argv):
        if arg == "--definitions-path" and i + 1 < len(sys.argv):
            definitions_path = Path(sys.argv[i + 1])

    definitions_path = definitions_path.resolve()
    print(f"Definitions path: {definitions_path}", file=sys.stderr)

    # Phase 1: Fetch or load decks
    if skip_fetch:
        print("Skipping fetch, loading existing deck JSONs...", file=sys.stderr)
        decks = load_existing_decks()
    else:
        print(f"Fetching {len(DECKS)} decks from Archidekt...", file=sys.stderr)
        decks = fetch_all_decks()

    if not decks:
        print("ERROR: no decks loaded", file=sys.stderr)
        sys.exit(1)

    print(f"\nLoaded {len(decks)} decks", file=sys.stderr)

    # Phase 2: Parse definitions.rs for existing card names
    defined_names = get_defined_card_names(definitions_path)
    print(f"Found {len(defined_names)} existing card definitions", file=sys.stderr)

    # Phase 3: Generate analysis files
    print("\nGenerating analysis files...", file=sys.stderr)

    summary = generate_summary(decks)
    summary_path = SCRIPT_DIR / "_summary.json"
    with open(summary_path, "w") as f:
        json.dump(summary, f, indent=2, ensure_ascii=False)
    print(f"  -> _summary.json", file=sys.stderr)

    unique = generate_unique_cards(decks, defined_names)
    unique_path = SCRIPT_DIR / "_unique_cards.json"
    with open(unique_path, "w") as f:
        json.dump(unique, f, indent=2, ensure_ascii=False)
    print(
        f"  -> _unique_cards.json ({unique['total_unique_cards']} unique, "
        f"{unique['total_with_definitions']} have defs, "
        f"{unique['total_needing_definitions']} need defs)",
        file=sys.stderr,
    )

    needing = generate_cards_needing_definitions(unique)
    needing_path = SCRIPT_DIR / "_cards_needing_definitions.json"
    with open(needing_path, "w") as f:
        json.dump(needing, f, indent=2, ensure_ascii=False)
    print(
        f"  -> _cards_needing_definitions.json ("
        f"T1: {needing['tier1_10plus_decks']}, "
        f"T2: {needing['tier2_5to9_decks']}, "
        f"T3: {needing['tier3_2to4_decks']}, "
        f"T4: {needing['tier4_1_deck']})",
        file=sys.stderr,
    )

    # Print a quick summary to stdout
    print(f"\n{'='*60}")
    print(f"  Test Decks Fetch Complete")
    print(f"{'='*60}")
    print(f"  Decks fetched:          {len(decks)}")
    print(f"  Total cards (with dups): {summary['total_cards']}")
    print(f"  Unique cards:           {unique['total_unique_cards']}")
    print(f"  Already defined:        {unique['total_with_definitions']}")
    print(f"  Needing definitions:    {unique['total_needing_definitions']}")
    print(f"")
    print(f"  Priority tiers (needing definitions):")
    print(f"    Tier 1 (10+ decks):   {needing['tier1_10plus_decks']}")
    print(f"    Tier 2 (5-9 decks):   {needing['tier2_5to9_decks']}")
    print(f"    Tier 3 (2-4 decks):   {needing['tier3_2to4_decks']}")
    print(f"    Tier 4 (1 deck):      {needing['tier4_1_deck']}")
    print(f"")

    # Print top 20 cards needing definitions
    print(f"  Top 20 cards needing definitions:")
    for c in needing["cards"][:20]:
        types_str = "/".join(c.get("types", []))
        kw_str = ", ".join(c.get("keywords", []))
        suffix = f" [{kw_str}]" if kw_str else ""
        print(f"    {c['appears_in_decks']:2d} decks | {types_str:20s} | {c['name']}{suffix}")

    print()


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
Fetch top cards for commanders from EDHREC and write combined JSON.

Usage:
    python fetch_edhrec_cards.py                    # Fetch all 20 commanders
    python fetch_edhrec_cards.py "Krenko, Mob Boss" # Fetch one commander
    python fetch_edhrec_cards.py --analyze          # Analyze existing data (no fetch)
"""

import json
import sys
import time
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

UA = "Mozilla/5.0"
BASE = "https://json.edhrec.com/pages/commanders"
SUB_PAGES = ["", "/artifacts", "/enchantments", "/lands", "/planeswalkers"]

# Our 20 commanders from the deck plan
COMMANDERS = [
    "Atraxa, Praetors' Voice",
    "Krenko, Mob Boss",
    "Yuriko, the Tiger's Shadow",
    "Meren of Clan Nel Toth",
    "Prossh, Skyraider of Kher",
    "Edgar Markov",
    "Omnath, Locus of Creation",
    "Korvold, Fae-Cursed King",
    "Teysa Karlov",
    "Winota, Joiner of Forces",
    "Talrand, Sky Summoner",
    "Selvala, Heart of the Wilds",
    "Syr Konrad, the Grim",
    "Sram, Senior Edificer",
    "The Ur-Dragon",
    "Niv-Mizzet, Parun",
    "Lathril, Blade of the Elves",
    "Chulane, Teller of Tales",
    "Isshin, Two Heavens as One",
    "Aesi, Tyrant of Gyre Strait",
]


def format_slug(name: str) -> str:
    name = name.lower()
    name = name.replace(" ", "-")
    name = name.replace("'", "")
    name = name.replace(",", "")
    return name


def fetch(url: str) -> dict | None:
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            return json.loads(resp.read())
    except Exception as e:
        print(f"  skipped {url}: {e}", file=sys.stderr)
        return None


def get_cards_for_commander(slug: str) -> dict:
    """Fetch all card recommendations for a single commander."""
    all_cards = {}
    for sub in SUB_PAGES:
        url = f"{BASE}/{slug}{sub}.json"
        print(f"  fetching {url}", file=sys.stderr)
        data = fetch(url)
        if not data:
            continue
        cardlists = data.get("container", {}).get("json_dict", {}).get("cardlists", [])
        for cl in cardlists:
            for card in cl.get("cardviews", []):
                name = card["name"]
                if name not in all_cards:
                    all_cards[name] = card
    return all_cards


def fetch_all_commanders(commanders: list[str]) -> dict:
    """Fetch cards for all commanders and build combined data."""
    per_commander = {}
    combined_cards = {}  # name -> combined card data

    for i, commander in enumerate(commanders):
        slug = format_slug(commander)
        print(f"\n[{i+1}/{len(commanders)}] {commander} (slug: {slug})", file=sys.stderr)

        cards = get_cards_for_commander(slug)
        print(f"  Found {len(cards)} cards", file=sys.stderr)

        per_commander[commander] = {
            "slug": slug,
            "card_count": len(cards),
        }

        for name, card_data in cards.items():
            if name not in combined_cards:
                combined_cards[name] = {
                    "name": name,
                    "id": card_data.get("id", ""),
                    "sanitized": card_data.get("sanitized", ""),
                    "commanders": [],
                    "total_inclusion": 0,
                    "max_inclusion": 0,
                    "max_synergy": -999,
                    "per_commander": {},
                }

            entry = combined_cards[name]
            num_decks = card_data.get("num_decks", 0)
            synergy = card_data.get("synergy", 0)

            entry["commanders"].append(commander)
            entry["total_inclusion"] += num_decks
            entry["max_inclusion"] = max(entry["max_inclusion"], num_decks)
            entry["max_synergy"] = max(entry["max_synergy"], synergy)
            entry["per_commander"][commander] = {
                "num_decks": num_decks,
                "potential_decks": card_data.get("potential_decks", 0),
                "synergy": synergy,
            }

        # Be polite to the EDHREC API
        if i < len(commanders) - 1:
            time.sleep(1)

    return {
        "commanders": per_commander,
        "cards": combined_cards,
    }


def analyze(cards_list: list[dict]):
    """Print analysis of the combined card data."""
    print(f"\n{'='*70}", file=sys.stderr)
    print(f"EDHREC Combined Card Analysis", file=sys.stderr)
    print(f"{'='*70}", file=sys.stderr)
    print(f"Total unique cards: {len(cards_list)}", file=sys.stderr)

    # Distribution by commander count
    by_commander_count = {}
    for card in cards_list:
        n = len(card.get("commanders", []))
        by_commander_count[n] = by_commander_count.get(n, 0) + 1

    print(f"\nCards by # of commanders recommending them:", file=sys.stderr)
    cumulative = 0
    for n in sorted(by_commander_count.keys(), reverse=True):
        count = by_commander_count[n]
        cumulative += count
        print(f"  {n:2d} commanders: {count:5d} cards  (cumulative: {cumulative})", file=sys.stderr)

    # Top cards
    sorted_cards = sorted(cards_list, key=lambda c: -c.get("total_inclusion", 0))
    print(f"\nTop 30 cards by total EDHREC inclusion:", file=sys.stderr)
    for card in sorted_cards[:30]:
        print(f"  {card.get('total_inclusion',0):8d} decks  ({len(card.get('commanders',[])):2d} cmdrs)  {card['name']}", file=sys.stderr)

    # Threshold by total inclusion
    print(f"\nThreshold analysis (cards with total_inclusion >= N):", file=sys.stderr)
    for threshold in [100000, 50000, 20000, 10000, 5000, 2000, 1000, 500, 100, 0]:
        count = sum(1 for c in cards_list if c.get("total_inclusion", 0) >= threshold)
        print(f"  >= {threshold:>7d}: {count:5d} cards", file=sys.stderr)

    # Threshold by commander count
    print(f"\nThreshold analysis (cards recommended by >= N commanders):", file=sys.stderr)
    for threshold in [15, 10, 8, 5, 3, 2, 1]:
        count = sum(1 for c in cards_list if len(c.get("commanders", [])) >= threshold)
        print(f"  >= {threshold:2d} commanders: {count:5d} cards", file=sys.stderr)


def main():
    out_dir = Path(__file__).parent
    combined_path = out_dir / "edhrec_all_commanders.json"

    if len(sys.argv) > 1 and sys.argv[1] == "--analyze":
        if not combined_path.exists():
            print("No combined data found. Run without --analyze first.", file=sys.stderr)
            sys.exit(1)
        with open(combined_path) as f:
            data = json.load(f)
        analyze(data["cards"])
        return

    if len(sys.argv) > 1:
        # Single commander mode
        commanders = [" ".join(sys.argv[1:])]
    else:
        commanders = COMMANDERS

    print(f"Fetching EDHREC data for {len(commanders)} commander(s)...", file=sys.stderr)
    data = fetch_all_commanders(commanders)

    # Sort cards by total_inclusion descending
    sorted_cards = sorted(data["cards"].values(), key=lambda c: -c["total_inclusion"])

    output = {
        "generated": datetime.now(timezone.utc).isoformat(),
        "commander_count": len(data["commanders"]),
        "total_unique_cards": len(data["cards"]),
        "commanders": data["commanders"],
        "cards": sorted_cards,
    }

    with open(combined_path, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\nWritten combined data to {combined_path}", file=sys.stderr)

    analyze(sorted_cards)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
Temporary script: fetch top cards for a commander from EDHREC and write to JSON.
Usage: python fetch_edhrec_cards.py [commander-name]
Default commander: Atraxa, Praetors' Voice
"""

import csv
import json
import re
import sys
import urllib.request
from pathlib import Path

UA = "Mozilla/5.0"
BASE = "https://json.edhrec.com/pages/commanders"
SUB_PAGES = ["", "/artifacts", "/enchantments", "/lands", "/planeswalkers"]


def format_slug(name: str) -> str:
    name = name.lower()
    name = name.replace(" ", "-")
    name = name.replace("'", "")
    name = name.replace(",", "")
    return name


def fetch(url: str) -> dict | None:
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
    try:
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read())
    except Exception as e:
        print(f"  skipped {url}: {e}", file=sys.stderr)
        return None


def get_cards(slug: str) -> dict:
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


def main():
    commander = " ".join(sys.argv[1:]) if len(sys.argv) > 1 else "Atraxa, Praetors' Voice"
    slug = format_slug(commander)
    print(f"Commander: {commander} (slug: {slug})", file=sys.stderr)

    cards = get_cards(slug)
    print(f"Total unique cards: {len(cards)}", file=sys.stderr)

    # Sort by inclusion count descending
    sorted_cards = sorted(cards.values(), key=lambda c: -c.get("num_decks", 0))

    out_path = Path(__file__).parent / "edhrec_top_cards.json"
    with open(out_path, "w") as f:
        json.dump({"commander": commander, "slug": slug, "cards": sorted_cards}, f, indent=2)
    print(f"Written to {out_path}", file=sys.stderr)

    csv_path = Path(__file__).parent / "edhrec_top_cards.csv"
    with open(csv_path, "w", newline="") as f:
        writer = csv.writer(f)
        for card in sorted_cards:
            writer.writerow([card["name"]])
    print(f"Written to {csv_path}", file=sys.stderr)


if __name__ == "__main__":
    main()

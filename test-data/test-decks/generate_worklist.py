#!/usr/bin/env python3
"""
Ability-gated card authoring worklist generator.

Classifies cards from _cards_needing_definitions.json as ready/blocked/deferred
based on the engine's current ability coverage in docs/mtg-engine-ability-coverage.md.

Usage:
    python3 test-data/test-decks/generate_worklist.py

Output:
    test-data/test-decks/_authoring_worklist.json
"""

import json
import os
import re
import sys
from datetime import datetime, timezone


# ---------------------------------------------------------------------------
# Category 2: Scryfall keyword name -> coverage doc name aliases
# ---------------------------------------------------------------------------
KEYWORD_ALIASES = {
    "Food": "Food tokens",
    "Treasure": "Treasure tokens",
    "First strike": "First Strike",
    "Double strike": "Double Strike",
    "Cumulative upkeep": "Cumulative Upkeep",
    "Split second": "Split Second",
    "Partner with": "Partner With",
    "Commander ninjutsu": "Ninjutsu",
    "Battle Cry": "Battle Cry",
    "Hexproof from": "Hexproof",          # Hexproof from X is a variant of Hexproof
    "For Mirrodin!": "Living Weapon",      # For Mirrodin! is Living Weapon + red token
    "Multikicker": "Kicker",               # Multikicker is kicker with is_multikicker=true
    "Forestwalk": "Landwalk",              # Landwalk covers all -walk variants
}

# ---------------------------------------------------------------------------
# Category 3: Ability words (not keywords — the engine handles them via
# TriggerCondition/Conditional effects, no keyword enum needed)
# ---------------------------------------------------------------------------
ABILITY_WORDS = {
    "Landfall", "Battalion", "Delirium", "Domain", "Eminence", "Ferocious",
    "Lieutenant", "Magecraft", "Metalcraft", "Raid", "Spell mastery",
    "Undergrowth", "Coven", "Addendum", "Imprint",
    # Additional ability words found in card data
    "Alliance",       # Trigger: whenever creature ETBs under your control
    "Mentor",         # Attacking creature with greater power puts +1/+1 on smaller
    "Exert",          # Choose not to untap — ability word pattern
    "Channel",        # Discard from hand for effect — ability word pattern
}

# ---------------------------------------------------------------------------
# Category 4: Keyword actions the engine already handles as Effect variants
# ---------------------------------------------------------------------------
KEYWORD_ACTIONS_SUPPORTED = {
    "Mill",           # Effect::Mill
    "Scry",           # Effect::Scry
    "Fight",          # Effect::Fight
    "Goad",           # Effect::Goad
    "Proliferate",    # Effect::Proliferate
    "Investigate",    # Effect::Investigate
}

# ---------------------------------------------------------------------------
# Category 5: Non-keywords (Alchemy, joke, or card-specific text Scryfall
# misclassifies as keywords)
# ---------------------------------------------------------------------------
NON_KEYWORDS = {
    "Buy Information", "Earthbend", "Harmonize", "Hire a Mercenary",
    "Leading from the Front", "Max speed", "Mobilize", "Mold Earth",
    "Sell Contraband", "Solved", "Start your engines!", "Stagger",
    "Summary Execution", "Super Nova", "Synaptic Disintegrator",
    "Three Autostubs", "Vivid", "Warp", "Endure",
    # Additional non-standard keywords found in card data
    "Compleated",     # Phyrexian mana variant (can pay 2 life)
    "Incubate",       # Create Incubator token — niche, not in engine
    "Meld",           # Two specific cards merge — very niche
    "Level Up",       # Level up creature — niche frame mechanic
    "Venture into the dungeon",  # D&D set mechanic, n/a in coverage doc
    "Demonstrate",    # Strixhaven — copy for opponent
    "Rebound",        # Cast again next upkeep from exile
    "Transmute",      # Discard to tutor by CMC
}

# ---------------------------------------------------------------------------
# Category 6: Deferred mechanics (always deferred regardless of status)
# ---------------------------------------------------------------------------
DEFERRED_KEYWORDS = {
    "Morph", "Mutate", "Phasing", "Transform", "Megamorph",
    "Disguise", "Manifest", "Cloak", "Daybound", "Nightbound",
}


def parse_ability_coverage(path):
    """Parse ability coverage doc for keyword -> (status, priority) mapping.

    Reads the markdown table rows, extracting:
      - Ability name (first column)
      - Priority (third column)
      - Status (fourth column, e.g. `validated`, `none`, `partial`, `complete`, `n/a`)

    Returns dict mapping lowercase ability name -> (status, priority).
    Also returns a case-preserving dict for display purposes.
    """
    coverage = {}       # lowercase name -> (status, priority)
    display_names = {}  # lowercase name -> original case name

    with open(path, "r") as f:
        for line in f:
            line = line.strip()
            # Match table rows: | Name | CR | Priority | `status` | ...
            # We need at least 4 pipe-delimited columns
            if not line.startswith("|"):
                continue
            cols = [c.strip() for c in line.split("|")]
            # cols[0] is empty (before first |), cols[1] is ability name, etc.
            if len(cols) < 5:
                continue

            ability_name = cols[1].strip()
            # Skip header rows
            if ability_name in ("Ability", "Pattern", "---------", "--------", ""):
                continue
            if ability_name.startswith("---"):
                continue

            priority_col = cols[3].strip()
            status_col = cols[4].strip()

            # Extract priority (P1, P2, P3, P4)
            priority_match = re.match(r"(P[1-4])", priority_col)
            priority = priority_match.group(1) if priority_match else ""

            # Extract status from backtick-wrapped value
            status_match = re.search(r"`(\w+(?:/\w+)?)`", status_col)
            status = status_match.group(1) if status_match else ""

            if not status:
                continue

            key = ability_name.lower()
            coverage[key] = (status, priority)
            display_names[key] = ability_name

    return coverage, display_names


def resolve_keyword(keyword, coverage):
    """Resolve a single keyword to its status.

    Returns (status_string, category) where category is one of:
    'coverage', 'alias', 'ability_word', 'keyword_action', 'non_keyword', 'deferred', 'unknown'
    """
    # Check deferred first (highest priority classification)
    if keyword in DEFERRED_KEYWORDS:
        return ("deferred", "deferred")

    # Check non-keywords (ignore these)
    if keyword in NON_KEYWORDS:
        return ("ignored", "non_keyword")

    # Check ability words (treated as ready)
    if keyword in ABILITY_WORDS:
        return ("ready", "ability_word")

    # Check keyword actions the engine supports
    if keyword in KEYWORD_ACTIONS_SUPPORTED:
        return ("ready", "keyword_action")

    # Check aliases
    if keyword in KEYWORD_ALIASES:
        aliased = KEYWORD_ALIASES[keyword]
        key = aliased.lower()
        if key in coverage:
            status, priority = coverage[key]
            return (f"{status} ({priority})" if priority else status, "alias")
        # Alias target not found in coverage — treat as unknown
        return ("unknown", "alias")

    # Direct lookup in coverage doc (case-insensitive)
    key = keyword.lower()
    if key in coverage:
        status, priority = coverage[key]
        return (f"{status} ({priority})" if priority else status, "coverage")

    return ("unknown", "unknown")


def is_blocking_status(status_str):
    """Check if a resolved status means the card is blocked."""
    # Extract the base status (before any priority annotation)
    base = status_str.split("(")[0].strip()
    return base in ("none", "partial")


def classify_card(card, coverage):
    """Classify a card as ready/blocked/deferred/unknown.

    Returns (classification, details_dict).
    """
    keywords = card.get("keywords", [])

    # Cards with no keywords are always ready
    if not keywords:
        return ("ready", {
            "keyword_statuses": {},
        })

    keyword_statuses = {}
    deferred_kws = []
    blocking_kws = []
    unknown_kws = []

    for kw in keywords:
        status, category = resolve_keyword(kw, coverage)
        keyword_statuses[kw] = status

        if category == "deferred":
            deferred_kws.append(kw)
        elif category == "non_keyword":
            pass  # ignored
        elif category == "unknown":
            unknown_kws.append(kw)
        elif is_blocking_status(status):
            blocking_kws.append(kw)
        # else: ready (validated, complete, n/a, ability_word, keyword_action)

    # Classification priority: deferred > unknown > blocked > ready
    if deferred_kws:
        return ("deferred", {
            "deferred_keywords": deferred_kws,
            "keyword_statuses": keyword_statuses,
        })
    if unknown_kws:
        return ("unknown", {
            "unknown_keywords": unknown_kws,
            "keyword_statuses": keyword_statuses,
        })
    if blocking_kws:
        return ("blocked", {
            "blocking_keywords": blocking_kws,
            "keyword_statuses": keyword_statuses,
        })

    return ("ready", {
        "keyword_statuses": keyword_statuses,
    })


def parse_authored_cards(project_root):
    """Extract card names from definitions.rs that already have CardDefinitions.

    Looks for lines matching: name: "Card Name".to_string(),
    Returns a set of card names.
    """
    definitions_path = os.path.join(
        project_root, "crates", "engine", "src", "cards", "definitions.rs"
    )
    authored = set()
    pattern = re.compile(r'name:\s*"([^"]+)"\.to_string\(\)')

    try:
        with open(definitions_path, "r") as f:
            for line in f:
                m = pattern.search(line)
                if m:
                    authored.add(m.group(1))
    except FileNotFoundError:
        print(f"WARNING: {definitions_path} not found", file=sys.stderr)

    return authored


def scan_all_deck_cards(deck_dir):
    """Scan all deck JSON files and build a unified card list with deck counts.

    Returns dict of card_name -> {appears_in_decks, types, keywords, ...}
    """
    cards = {}
    for filename in sorted(os.listdir(deck_dir)):
        if not filename.endswith(".json"):
            continue
        if filename.startswith("_"):
            continue
        filepath = os.path.join(deck_dir, filename)
        try:
            with open(filepath, "r") as f:
                deck = json.load(f)
        except (json.JSONDecodeError, IOError):
            continue

        for card in deck.get("cards", []):
            name = card.get("name", "")
            if not name:
                continue
            if name not in cards:
                cards[name] = {
                    "name": name,
                    "appears_in_decks": 0,
                    "types": card.get("types", []),
                    "keywords": card.get("keywords", []),
                }
            cards[name]["appears_in_decks"] += 1

    return list(cards.values())


def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(os.path.dirname(script_dir))

    coverage_path = os.path.join(project_root, "docs", "mtg-engine-ability-coverage.md")
    output_path = os.path.join(script_dir, "_authoring_worklist.json")

    # Parse ability coverage
    coverage, display_names = parse_ability_coverage(coverage_path)
    print(f"Parsed {len(coverage)} abilities from coverage doc", file=sys.stderr)

    # Parse authored card names from definitions.rs
    authored_names = parse_authored_cards(project_root)
    print(f"Found {len(authored_names)} authored CardDefinitions", file=sys.stderr)

    # Scan all deck files for the complete card universe
    all_cards = scan_all_deck_cards(script_dir)
    print(f"Found {len(all_cards)} unique cards across all decks", file=sys.stderr)

    # Classify each card
    authored = []
    ready = []
    blocked = []
    deferred = []
    unknown = []

    for card in all_cards:
        # Check if already authored first
        if card["name"] in authored_names:
            entry = {
                "name": card["name"],
                "appears_in_decks": card["appears_in_decks"],
                "types": card.get("types", []),
                "keywords": card.get("keywords", []),
                "keyword_statuses": {},
            }
            authored.append(entry)
            continue

        classification, details = classify_card(card, coverage)

        entry = {
            "name": card["name"],
            "appears_in_decks": card["appears_in_decks"],
            "types": card.get("types", []),
            "keywords": card.get("keywords", []),
        }
        entry.update(details)

        if classification == "ready":
            ready.append(entry)
        elif classification == "blocked":
            blocked.append(entry)
        elif classification == "deferred":
            deferred.append(entry)
        elif classification == "unknown":
            unknown.append(entry)

    # Sort each section by appears_in_decks descending
    authored.sort(key=lambda x: (-x["appears_in_decks"], x["name"]))
    ready.sort(key=lambda x: (-x["appears_in_decks"], x["name"]))
    blocked.sort(key=lambda x: (-x["appears_in_decks"], x["name"]))
    deferred.sort(key=lambda x: (-x["appears_in_decks"], x["name"]))
    unknown.sort(key=lambda x: (-x["appears_in_decks"], x["name"]))

    total = len(authored) + len(ready) + len(blocked) + len(deferred) + len(unknown)

    # Build output
    output = {
        "generated": datetime.now(timezone.utc).isoformat(),
        "summary": {
            "total_cards": total,
            "authored": len(authored),
            "ready": len(ready),
            "blocked": len(blocked),
            "deferred": len(deferred),
            "unknown": len(unknown),
        },
        "authored": authored,
        "ready": ready,
        "blocked": blocked,
        "deferred": deferred,
        "unknown": unknown,
    }

    # Write output
    with open(output_path, "w") as f:
        json.dump(output, f, indent=2)

    # Print summary
    print(f"\n{'='*60}", file=sys.stderr)
    print(f"Card Authoring Worklist Summary", file=sys.stderr)
    print(f"{'='*60}", file=sys.stderr)
    print(f"  Total cards:  {total}", file=sys.stderr)
    print(f"  Authored:     {len(authored)}", file=sys.stderr)
    print(f"  Ready:        {len(ready)}", file=sys.stderr)
    print(f"  Blocked:      {len(blocked)}", file=sys.stderr)
    print(f"  Deferred:     {len(deferred)}", file=sys.stderr)
    print(f"  Unknown:      {len(unknown)}", file=sys.stderr)
    print(f"{'='*60}", file=sys.stderr)

    # Print unknown keywords for manual triage
    if unknown:
        print(f"\nUnknown keywords needing manual triage:", file=sys.stderr)
        unknown_kw_set = set()
        for entry in unknown:
            for kw in entry.get("unknown_keywords", []):
                unknown_kw_set.add(kw)
        for kw in sorted(unknown_kw_set):
            # Count how many cards use this keyword
            count = sum(1 for e in unknown if kw in e.get("unknown_keywords", []))
            print(f"  - {kw} ({count} cards)", file=sys.stderr)

    # Print top blocking keywords
    if blocked:
        blocking_counts = {}
        for entry in blocked:
            for kw in entry.get("blocking_keywords", []):
                blocking_counts[kw] = blocking_counts.get(kw, 0) + 1
        print(f"\nTop blocking keywords:", file=sys.stderr)
        for kw, count in sorted(blocking_counts.items(), key=lambda x: -x[1]):
            print(f"  - {kw}: {count} cards", file=sys.stderr)

    print(f"\nOutput written to: {output_path}", file=sys.stderr)


if __name__ == "__main__":
    main()

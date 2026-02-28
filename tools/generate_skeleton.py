#!/usr/bin/env python3
"""Generate card definition skeleton .rs files from Scryfall data.

Fetches card data from the Scryfall API and generates .rs files with all
mechanical fields filled in. Only the `abilities` vec is left as TODO.

Usage:
    # Single card
    python3 tools/generate_skeleton.py "Skullclamp"

    # Multiple cards
    python3 tools/generate_skeleton.py "Skullclamp" "Ancient Tomb" "Blood Artist"

    # From worklist (ready cards)
    python3 tools/generate_skeleton.py --from-worklist test-data/test-decks/_authoring_worklist.json --status ready

    # Limit batch size
    python3 tools/generate_skeleton.py --from-worklist test-data/test-decks/_authoring_worklist.json --status ready --limit 10

    # Dry run (don't write files)
    python3 tools/generate_skeleton.py --dry-run "Skullclamp"
"""

import argparse
import json
import os
import re
import sys
import time
import urllib.request
import urllib.error

DEFS_DIR = "crates/engine/src/cards/defs"
DECKS_DIR = "test-data/test-decks"
SCRYFALL_API = "https://api.scryfall.com/cards/named"
USER_AGENT = "MTGEngine/1.0 (card skeleton generator)"


def build_local_card_index(decks_dir: str) -> dict[str, dict]:
    """Build a name → card_data index from local deck JSON files."""
    index = {}
    if not os.path.isdir(decks_dir):
        return index
    for fname in sorted(os.listdir(decks_dir)):
        if not fname.endswith(".json") or fname.startswith("_"):
            continue
        with open(os.path.join(decks_dir, fname)) as f:
            try:
                deck = json.load(f)
            except json.JSONDecodeError:
                continue
        for card in deck.get("cards", []):
            name = card.get("name", "")
            if name and name not in index:
                # Normalize to Scryfall-like structure
                index[name] = {
                    "name": name,
                    "mana_cost": card.get("mana_cost"),
                    "type_line": _reconstruct_type_line(card),
                    "oracle_text": card.get("oracle_text", ""),
                    "keywords": card.get("keywords", []),
                    "power": card.get("power"),
                    "toughness": card.get("toughness"),
                }
    return index


def _reconstruct_type_line(card: dict) -> str:
    """Reconstruct a type_line from deck card data."""
    types = card.get("types", [])
    subtypes = card.get("subtypes", [])
    supertypes = card.get("supertypes", [])

    left_parts = list(supertypes) + list(types)
    left = " ".join(left_parts)

    if subtypes:
        return f"{left} — {' '.join(subtypes)}"
    return left


def fetch_card(name: str, local_index: dict[str, dict] | None = None) -> dict | None:
    """Fetch card data, preferring local index, falling back to Scryfall API."""
    # Try local index first
    if local_index and name in local_index:
        return local_index[name]

    # Fall back to Scryfall API
    url = f"{SCRYFALL_API}?exact={urllib.request.quote(name)}"
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        if e.code == 404:
            print(f"  WARNING: Card not found on Scryfall: {name}", file=sys.stderr)
            return None
        print(f"  WARNING: Scryfall API error {e.code} for {name}", file=sys.stderr)
        return None
    except (urllib.error.URLError, OSError) as e:
        print(f"  WARNING: Network error for {name}: {e}", file=sys.stderr)
        return None


def card_name_to_slug(name: str) -> str:
    """Convert card name to kebab-case card_id."""
    slug = name.lower()
    slug = slug.replace("'", "")
    slug = slug.replace(",", "")
    slug = re.sub(r"[^a-z0-9]+", "-", slug)
    slug = slug.strip("-")
    return slug


def slug_to_filename(slug: str) -> str:
    """Convert kebab-case slug to snake_case filename."""
    return slug.replace("-", "_")


def parse_mana_cost(mana_cost_str: str | None) -> str:
    """Parse Scryfall mana cost string to ManaCost struct."""
    if not mana_cost_str:
        return "None"

    costs = {"generic": 0, "white": 0, "blue": 0, "black": 0, "red": 0, "green": 0}

    # Extract mana symbols: {W}, {U}, {B}, {R}, {G}, {1}, {2}, etc.
    symbols = re.findall(r"\{([^}]+)\}", mana_cost_str)
    for sym in symbols:
        if sym == "W":
            costs["white"] += 1
        elif sym == "U":
            costs["blue"] += 1
        elif sym == "B":
            costs["black"] += 1
        elif sym == "R":
            costs["red"] += 1
        elif sym == "G":
            costs["green"] += 1
        elif sym == "X":
            pass  # X costs need manual handling
        elif sym == "C":
            pass  # Colorless mana symbol
        elif sym.isdigit():
            costs["generic"] += int(sym)
        # Hybrid, phyrexian, etc. need manual handling

    # Build the ManaCost expression with only non-zero fields
    fields = []
    for key in ["generic", "white", "blue", "black", "red", "green"]:
        if costs[key] > 0:
            fields.append(f"{key}: {costs[key]}")

    if not fields:
        return "Some(ManaCost { ..Default::default() })"

    return f"Some(ManaCost {{ {', '.join(fields)}, ..Default::default() }})"


def format_types(card: dict) -> str:
    """Generate the types expression."""
    type_line = card.get("type_line", "")

    # Parse supertypes, types, subtypes from the type line
    # Format: "Legendary Creature — Human Wizard"
    parts = type_line.split("—")
    left = parts[0].strip() if parts else ""
    subtypes_str = parts[1].strip() if len(parts) > 1 else ""

    supertypes = []
    card_types = []

    for word in left.split():
        if word in ("Legendary", "Basic", "Snow", "World"):
            supertypes.append(f"SuperType::{word}")
        elif word in (
            "Creature",
            "Artifact",
            "Enchantment",
            "Instant",
            "Sorcery",
            "Land",
            "Planeswalker",
            "Battle",
            "Kindred",
        ):
            card_types.append(f"CardType::{word}")
        elif word == "Tribal":
            card_types.append("CardType::Kindred")

    subtypes = [s.strip() for s in subtypes_str.split() if s.strip()] if subtypes_str else []

    # Choose the appropriate helper function
    has_supers = len(supertypes) > 0
    has_subs = len(subtypes) > 0

    if has_supers and has_subs:
        types_arr = ", ".join(card_types)
        supers_arr = ", ".join(supertypes)
        subs_arr = ", ".join(f'"{s}"' for s in subtypes)
        return f"full_types(&[{supers_arr}], &[{types_arr}], &[{subs_arr}])"
    elif has_supers:
        types_arr = ", ".join(card_types)
        supers_arr = ", ".join(supertypes)
        return f"supertypes(&[{supers_arr}], &[{types_arr}])"
    elif has_subs:
        if card_types == ["CardType::Creature"] and not has_supers:
            subs_arr = ", ".join(f'"{s}"' for s in subtypes)
            return f'creature_types(&[{subs_arr}])'
        else:
            types_arr = ", ".join(card_types)
            subs_arr = ", ".join(f'"{s}"' for s in subtypes)
            return f"types_sub(&[{types_arr}], &[{subs_arr}])"
    else:
        types_arr = ", ".join(card_types)
        return f"types(&[{types_arr}])"


def format_oracle_text(oracle_text: str) -> str:
    """Format oracle text as a Rust string literal."""
    if not oracle_text:
        return '""'
    # Escape special characters
    escaped = oracle_text.replace("\\", "\\\\").replace('"', '\\"')
    # Replace newlines with \n
    escaped = escaped.replace("\n", "\\n")
    return f'"{escaped}"'


def derive_todo_comments(oracle_text: str, keywords: list[str]) -> list[str]:
    """Derive TODO comments from oracle text to guide ability authoring."""
    todos = []

    if not oracle_text:
        return todos

    lines = oracle_text.split("\n")
    for line in lines:
        line = line.strip()
        if not line:
            continue

        # Detect common patterns
        if line.startswith("{T}:") or re.match(r"^\{[^}]+\}.*:", line):
            todos.append(f"// TODO: Activated — {line[:80]}")
        elif any(
            line.startswith(kw)
            for kw in [
                "When",
                "Whenever",
                "At the beginning",
                "At end",
            ]
        ):
            todos.append(f"// TODO: Triggered — {line[:80]}")
        elif any(
            kw.lower() in line.lower()
            for kw in [
                "gets +",
                "gets -",
                "has ",
                "have ",
                "can't",
                "don't",
                "Each creature",
                "All creatures",
                "Creatures you control",
            ]
        ):
            todos.append(f"// TODO: Static — {line[:80]}")
        elif line.lower().startswith("equip"):
            todos.append(f"// TODO: Equip — {line[:80]}")
        else:
            # Check if it's a keyword ability line
            is_keyword = False
            for kw in keywords:
                if line.lower().startswith(kw.lower()):
                    is_keyword = True
                    break
            if is_keyword:
                todos.append(f"// TODO: Keyword — {line[:80]}")
            elif len(line) > 5:  # Skip very short lines like "()"
                todos.append(f"// TODO: {line[:80]}")

    return todos


def generate_skeleton(card: dict) -> str:
    """Generate a skeleton .rs file for a card."""
    name = card["name"]
    slug = card_name_to_slug(name)
    oracle_text = card.get("oracle_text", "")
    keywords = card.get("keywords", [])
    power = card.get("power")
    toughness = card.get("toughness")

    mana_cost = parse_mana_cost(card.get("mana_cost"))
    types_expr = format_types(card)
    oracle_expr = format_oracle_text(oracle_text)
    todos = derive_todo_comments(oracle_text, keywords)

    lines = []
    lines.append(f"// {name}")
    lines.append("use crate::cards::helpers::*;")
    lines.append("")
    lines.append("pub fn card() -> CardDefinition {")
    lines.append("    CardDefinition {")
    lines.append(f'        card_id: cid("{slug}"),')
    lines.append(f'        name: "{name}".to_string(),')
    lines.append(f"        mana_cost: {mana_cost},")
    lines.append(f"        types: {types_expr},")
    lines.append(f"        oracle_text: {oracle_expr}.to_string(),")

    # Abilities section with TODOs
    if todos:
        lines.append("        abilities: vec![")
        for todo in todos:
            lines.append(f"            {todo}")
        lines.append("        ],")
    else:
        lines.append("        abilities: vec![],")

    # Power/toughness for creatures
    if power and toughness:
        try:
            p = int(power)
            t = int(toughness)
            lines.append(f"        power: Some({p}),")
            lines.append(f"        toughness: Some({t}),")
        except ValueError:
            # Variable P/T like "*" — leave as comment
            lines.append(f"        // power: {power}, toughness: {toughness} (variable — needs manual handling)")

    lines.append("        ..Default::default()")
    lines.append("    }")
    lines.append("}")
    lines.append("")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Generate card definition skeletons from Scryfall")
    parser.add_argument("cards", nargs="*", help="Card names to generate skeletons for")
    parser.add_argument("--from-worklist", help="Path to _authoring_worklist.json")
    parser.add_argument("--status", default="ready", help="Filter worklist by status (default: ready)")
    parser.add_argument("--limit", type=int, help="Max number of cards to generate")
    parser.add_argument("--dry-run", action="store_true", help="Don't write files")
    parser.add_argument("--skip-existing", action="store_true", default=True,
                        help="Skip cards that already have a file (default: true)")
    args = parser.parse_args()

    card_names = list(args.cards)

    # Load from worklist if specified
    if args.from_worklist:
        with open(args.from_worklist) as f:
            worklist = json.load(f)

        cards_list = worklist.get(args.status, [])
        for c in cards_list:
            card_names.append(c["name"])

    if not card_names:
        print("No cards specified. Use card names as arguments or --from-worklist.", file=sys.stderr)
        sys.exit(1)

    if args.limit:
        card_names = card_names[: args.limit]

    os.makedirs(DEFS_DIR, exist_ok=True)

    # Build local card index from deck files (avoids network dependency)
    print("Building local card index from deck files...")
    local_index = build_local_card_index(DECKS_DIR)
    print(f"  {len(local_index)} unique cards indexed locally")

    written = 0
    skipped = 0
    failed = 0

    for i, name in enumerate(card_names):
        slug = card_name_to_slug(name)
        filename = slug_to_filename(slug)
        filepath = os.path.join(DEFS_DIR, f"{filename}.rs")

        # Skip existing
        if args.skip_existing and os.path.exists(filepath):
            skipped += 1
            continue

        if args.dry_run:
            print(f"  Would generate: {filepath} ({name})")
            written += 1
            continue

        # Only rate-limit when falling back to Scryfall (not for local lookups)
        use_api = name not in local_index
        if use_api and i > 0:
            time.sleep(0.1)

        card = fetch_card(name, local_index)
        if card is None:
            failed += 1
            continue

        skeleton = generate_skeleton(card)
        with open(filepath, "w") as f:
            f.write(skeleton)
        src = "local" if not use_api else "scryfall"
        print(f"  Generated: {filepath} ({name}) [{src}]")
        written += 1

    print(f"\nSummary: {written} generated, {skipped} skipped (existing), {failed} failed")


if __name__ == "__main__":
    main()

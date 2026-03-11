#!/usr/bin/env python3
"""Phase 1: Generate card definition .rs files for template-able groups.

Reads _authoring_plan.json to identify cards in 5 template groups:
  - body-only: creatures/spells with no abilities (abilities: vec![])
  - land-etb-tapped: lands that enter tapped + tap-for-mana
  - mana-land: untapped lands with tap-for-mana
  - mana-artifact: artifacts with tap-for-mana
  - mana-creature: creatures with tap-for-mana

Uses SQLite card data for oracle text, types, subtypes, mana cost, P/T.
Generates one .rs file per card in crates/engine/src/cards/defs/.

Usage:
    python3 test-data/test-cards/bulk_generate.py
    python3 test-data/test-cards/bulk_generate.py --dry-run
    python3 test-data/test-cards/bulk_generate.py --group land-etb-tapped
"""

import argparse
import json
import os
import re
import sqlite3
import sys
from pathlib import Path

# Paths relative to project root
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
DEFS_DIR = PROJECT_ROOT / "crates" / "engine" / "src" / "cards" / "defs"
PLAN_FILE = PROJECT_ROOT / "test-data" / "test-cards" / "_authoring_plan.json"
DB_FILE = PROJECT_ROOT / "cards.sqlite"
LOG_FILE = PROJECT_ROOT / "test-data" / "test-cards" / "_bulk_generate_log.json"

TEMPLATE_GROUPS = ["body-only", "land-etb-tapped", "mana-land", "mana-artifact", "mana-creature"]

# Mana symbol → (field_name, mana_pool position)
MANA_MAP = {
    "W": ("white", 0),
    "U": ("blue", 1),
    "B": ("black", 2),
    "R": ("red", 3),
    "G": ("green", 4),
    "C": ("colorless", 5),
}


# --- Slug / filename helpers (matches generate_skeleton.py) ---

def card_name_to_slug(name: str) -> str:
    """Convert card name to kebab-case card_id (cid argument)."""
    # For split/MDFC cards, use only the front face name
    if " // " in name:
        name = name.split(" // ")[0]
    slug = name.lower()
    slug = slug.replace("'", "")
    slug = slug.replace(",", "")
    slug = re.sub(r"[^a-z0-9]+", "-", slug)
    slug = slug.strip("-")
    return slug


def slug_to_filename(slug: str) -> str:
    """Convert kebab-case slug to snake_case filename (no .rs)."""
    return slug.replace("-", "_")


# --- Mana cost parsing ---

def parse_mana_cost(mana_cost_str: str | None) -> str:
    """Parse Scryfall mana cost string to Rust ManaCost expression."""
    if not mana_cost_str or mana_cost_str.strip() == "":
        return "None"

    costs = {"generic": 0, "white": 0, "blue": 0, "black": 0, "red": 0, "green": 0}

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
            pass  # Colorless mana symbol — TODO if needed
        elif sym.isdigit():
            costs["generic"] += int(sym)
        elif sym == "0":
            pass  # {0} cost
        # Hybrid, phyrexian, etc. need manual handling

    fields = []
    for key in ["generic", "white", "blue", "black", "red", "green"]:
        if costs[key] > 0:
            fields.append(f"{key}: {costs[key]}")

    if not fields:
        return "Some(ManaCost { ..Default::default() })"

    return f"Some(ManaCost {{ {', '.join(fields)}, ..Default::default() }})"


# --- Type line parsing ---

def parse_subtypes(type_line: str) -> list[str]:
    """Extract subtypes from type line like 'Land — Gate Lair'."""
    if "—" not in type_line and "—" not in type_line:
        # Check for em-dash vs en-dash vs hyphen
        for sep in ["—", "–", " - "]:
            if sep in type_line:
                parts = type_line.split(sep, 1)
                return parts[1].strip().split() if len(parts) > 1 else []
        return []
    parts = type_line.split("—", 1)
    if len(parts) < 2:
        return []
    return parts[1].strip().split()


def has_supertype(type_line: str, supertype: str) -> bool:
    """Check if type line includes a supertype like 'Legendary' or 'Basic'."""
    left = type_line.split("—")[0].strip() if "—" in type_line else type_line
    return supertype in left


def format_types_expr(type_line: str, card_types: list[str]) -> str:
    """Generate the Rust types expression from type_line."""
    subtypes = parse_subtypes(type_line)
    is_creature = "Creature" in card_types or "Creature" in type_line

    if is_creature:
        if subtypes:
            quoted = ['"' + s + '"' for s in subtypes]
            return "creature_types(&[" + ", ".join(quoted) + "])"
        return "creature_types(&[])"

    # Build CardType list
    rust_types = []
    for t in card_types:
        if t in ("Creature", "Land", "Artifact", "Enchantment", "Instant", "Sorcery", "Planeswalker"):
            rust_types.append("CardType::" + t)

    # Fallback: parse from type_line
    if not rust_types:
        left = type_line.split("—")[0].strip() if "—" in type_line else type_line
        for t in ["Land", "Artifact", "Enchantment", "Instant", "Sorcery", "Planeswalker"]:
            if t in left:
                rust_types.append("CardType::" + t)

    if not rust_types:
        rust_types = ["CardType::Land"]  # fallback

    type_list = ", ".join(rust_types)

    if subtypes:
        quoted = ['"' + s + '"' for s in subtypes]
        return "types_sub(&[" + type_list + "], &[" + ", ".join(quoted) + "])"
    return "types(&[" + type_list + "])"


# --- Mana production parsing from oracle text ---

def parse_mana_production(oracle_text: str) -> list[dict]:
    """Parse mana abilities from oracle text.

    Returns list of dicts:
      {"type": "specific", "colors": ["W", "B"]}
      {"type": "any_color"}
      {"type": "colorless"}
      {"type": "unparseable", "text": "..."}
    """
    abilities = []

    # Find all {T}: Add ... lines
    for line in oracle_text.split("\n"):
        line = line.strip()
        if not re.search(r"\{T\}:?\s*Add\b", line, re.IGNORECASE):
            continue

        # "any color" variants
        if re.search(r"one mana of any color|mana of any color|any color", line, re.IGNORECASE):
            abilities.append({"type": "any_color"})
            continue

        # Specific colors: {T}: Add {W} or {B}. / {T}: Add {G}. / {T}: Add {W}, {U}, or {B}.
        colors = re.findall(r"\{([WUBRGC])\}", line)
        if colors:
            # Filter out duplicates while preserving order
            seen = set()
            unique_colors = []
            for c in colors:
                if c not in seen:
                    seen.add(c)
                    unique_colors.append(c)
            abilities.append({"type": "specific", "colors": unique_colors})
            continue

        # Colorless: {T}: Add {C} or just doesn't match specific
        if re.search(r"\{C\}", line):
            abilities.append({"type": "colorless"})
            continue

        abilities.append({"type": "unparseable", "text": line})

    return abilities


def mana_pool_args(color: str) -> str:
    """Return mana_pool(w, u, b, r, g, c) args for a single color."""
    args = [0, 0, 0, 0, 0, 0]
    pos = MANA_MAP.get(color, (None, None))[1]
    if pos is not None:
        args[pos] = 1
    return ", ".join(str(a) for a in args)


def format_mana_ability(prod: dict) -> str:
    """Generate Rust Effect expression for a mana production ability."""
    if prod["type"] == "any_color":
        return "Effect::AddManaAnyColor { player: PlayerTarget::Controller }"

    if prod["type"] == "colorless":
        return "Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 0, 0, 1) }"

    if prod["type"] == "specific":
        colors = prod["colors"]
        if len(colors) == 1:
            return f"Effect::AddMana {{ player: PlayerTarget::Controller, mana: mana_pool({mana_pool_args(colors[0])}) }}"

        # Multiple colors → Effect::Choose
        choices = []
        for c in colors:
            choices.append(
                f"                        Effect::AddMana {{ player: PlayerTarget::Controller, mana: mana_pool({mana_pool_args(c)}) }}"
            )

        # Build prompt string
        color_syms = [f"{{{c}}}" for c in colors]
        if len(color_syms) == 2:
            prompt = f"Add {color_syms[0]} or {color_syms[1]}?"
        else:
            prompt = "Add " + ", ".join(color_syms[:-1]) + f", or {color_syms[-1]}?"

        choices_str = ",\n".join(choices)
        return f"""Effect::Choose {{
                    prompt: "{prompt}".to_string(),
                    choices: vec![
{choices_str},
                    ],
                }}"""

    return None  # unparseable


# --- Card data from SQLite ---

def _build_result_from_faces(conn, card_id_val, card_name, layout):
    """Helper: build result dict from card_faces table."""
    faces = conn.execute(
        "SELECT face_index, name, mana_cost, type_line, oracle_text, power, toughness "
        "FROM card_faces WHERE card_id = ? ORDER BY face_index",
        (card_id_val,)
    ).fetchall()
    if not faces:
        return None
    f = faces[0]  # front face
    return {
        "name": card_name,
        "front_name": f[1],
        "mana_cost": f[2],
        "type_line": f[3],
        "oracle_text": f[4] or "",
        "power": f[5],
        "toughness": f[6],
        "layout": layout,
        "faces": faces,
    }


def load_card_data(conn: sqlite3.Connection, card_name: str) -> dict | None:
    """Load card data from SQLite. Handles MDFCs via card_faces."""
    # Try exact match on cards table
    row = conn.execute(
        "SELECT id, name, mana_cost, type_line, oracle_text, power, toughness, layout "
        "FROM cards WHERE name = ? AND layout NOT IN ('art_series', 'token', 'emblem')",
        (card_name,)
    ).fetchone()

    if row:
        cid, name, mana_cost, type_line, oracle_text, power, toughness, layout = row
        # For MDFCs/split cards with NULL oracle_text, get from card_faces
        if oracle_text is None and layout in ("modal_dfc", "transform", "split", "adventure", "flip"):
            result = _build_result_from_faces(conn, cid, card_name, layout)
            if result:
                return result
        return {
            "name": card_name,
            "front_name": card_name.split(" // ")[0] if " // " in card_name else card_name,
            "mana_cost": mana_cost,
            "type_line": type_line,
            "oracle_text": oracle_text or "",
            "power": power,
            "toughness": toughness,
            "layout": layout,
            "faces": None,
        }

    # Plan may have front-face-only name (e.g. "Boggart Trawler") but DB has "Boggart Trawler // Boggart Bog"
    front = card_name.split(" // ")[0] if " // " in card_name else card_name

    # Strategy 1: search card_faces by front face name
    face_row = conn.execute(
        "SELECT cf.card_id FROM card_faces cf WHERE cf.name = ? AND cf.face_index = 0",
        (front,)
    ).fetchone()
    if face_row:
        parent = conn.execute("SELECT name, layout FROM cards WHERE id = ?", (face_row[0],)).fetchone()
        if parent:
            return _build_result_from_faces(conn, face_row[0], card_name, parent[1])

    # Strategy 2: LIKE match on cards table
    like_row = conn.execute(
        "SELECT id, layout FROM cards WHERE name LIKE ? AND layout NOT IN ('art_series', 'token', 'emblem')",
        (front + " // %",)
    ).fetchone()
    if like_row:
        return _build_result_from_faces(conn, like_row[0], card_name, like_row[1])

    return None


# --- Skip logic ---

def should_skip_mana_card(oracle_text: str) -> str | None:
    """Return skip reason if oracle text has more than just mana abilities."""
    lines = [l.strip() for l in oracle_text.strip().split("\n") if l.strip()]

    for line in lines:
        # Pure mana ability line: must end with mana symbols + period (no trailing text)
        if re.match(r"^\{T\}:?\s*Add\b", line):
            # Check it's ONLY a mana ability (no extra clauses like "deals 1 damage")
            if re.match(r"^\{T\}:?\s*Add\s+(?:one mana of any color[^.]*|\{[WUBRGC]\}(?:[,\s]*(?:or\s+)?\{[WUBRGC]\})*)\.\s*$", line):
                continue
            return f"complex mana ability: {line[:60]}"
        # ETB tapped line
        if re.match(r"^(This land |.*) enters (the battlefield )?tapped", line, re.IGNORECASE):
            continue
        # Reminder text only
        if re.match(r"^\(", line):
            continue
        # Empty
        if not line:
            continue
        # Has additional oracle text beyond mana — skip for Phase 2
        return f"additional oracle text: {line[:60]}"

    return None


def should_skip_etb_tapped(oracle_text: str) -> str | None:
    """Return skip reason if ETB tapped land has complex abilities beyond mana."""
    lines = [l.strip() for l in oracle_text.strip().split("\n") if l.strip()]

    for line in lines:
        # ETB tapped / conditional ETB
        if re.search(r"enters (the battlefield )?tapped", line, re.IGNORECASE):
            continue
        # Mana ability
        if re.match(r"^\{T\}:?\s*Add\b", line):
            continue
        # Reminder text
        if re.match(r"^\(", line):
            continue
        if not line:
            continue
        # Has extra abilities — skip
        return f"additional oracle text: {line[:60]}"

    return None


# --- Template generators ---

def generate_body_only(card_data: dict, plan_card: dict) -> str:
    """Generate a card def with abilities: vec![] for body-only cards."""
    slug = card_name_to_slug(card_data["name"])
    front_name = card_data.get("front_name", card_data["name"])
    display_name = card_data["name"]

    mana_cost = parse_mana_cost(card_data["mana_cost"])
    type_line = card_data["type_line"]
    types_expr = format_types_expr(type_line, plan_card.get("types", []))

    power_line = ""
    toughness_line = ""
    if card_data.get("power") is not None:
        power_line = f"        power: Some({card_data['power']}),\n"
        toughness_line = f"        toughness: Some({card_data['toughness']}),\n"

    oracle = card_data.get("oracle_text", "").replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")

    comment = f"// {display_name}"
    if oracle:
        comment += f" — {oracle[:70]}"

    return f"""{comment}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {{
    CardDefinition {{
        card_id: cid("{slug}"),
        name: "{display_name}".to_string(),
        mana_cost: {mana_cost},
        types: {types_expr},
        oracle_text: "{oracle}".to_string(),
{power_line}{toughness_line}        abilities: vec![],
        ..Default::default()
    }}
}}
"""


def generate_etb_tapped_land(card_data: dict, plan_card: dict) -> str:
    """Generate ETB tapped land with replacement + mana ability."""
    slug = card_name_to_slug(card_data["name"])
    display_name = card_data["name"]
    oracle = card_data.get("oracle_text", "")

    type_line = card_data["type_line"]
    types_expr = format_types_expr(type_line, plan_card.get("types", []))

    prods = parse_mana_production(oracle)
    if not prods:
        return None  # Can't parse mana production

    mana_effects = []
    for prod in prods:
        eff = format_mana_ability(prod)
        if eff is None:
            return None  # Unparseable
        mana_effects.append(eff)

    # Build ability definitions
    abilities = []

    # ETB tapped replacement
    abilities.append("""            // Enters tapped (CR 614.1c)
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
            }""")

    # Mana abilities
    for i, eff in enumerate(mana_effects):
        # Indent multi-line effects properly
        if "\n" in eff:
            abilities.append(f"""            AbilityDefinition::Activated {{
                cost: Cost::Tap,
                effect: {eff},
                timing_restriction: None,
            }}""")
        else:
            abilities.append(f"""            AbilityDefinition::Activated {{
                cost: Cost::Tap,
                effect: {eff},
                timing_restriction: None,
            }}""")

    abilities_str = ",\n".join(abilities)
    oracle_escaped = oracle.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")

    comment_oracle = oracle.replace("\n", " ")[:80]

    return f"""// {display_name} — {comment_oracle}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {{
    CardDefinition {{
        card_id: cid("{slug}"),
        name: "{display_name}".to_string(),
        mana_cost: None,
        types: {types_expr},
        oracle_text: "{oracle_escaped}".to_string(),
        abilities: vec![
{abilities_str},
        ],
        ..Default::default()
    }}
}}
"""


def generate_mana_permanent(card_data: dict, plan_card: dict, group_id: str) -> str:
    """Generate mana-producing land/artifact/creature."""
    slug = card_name_to_slug(card_data["name"])
    display_name = card_data["name"]
    oracle = card_data.get("oracle_text", "")

    type_line = card_data["type_line"]
    types_expr = format_types_expr(type_line, plan_card.get("types", []))

    mana_cost = parse_mana_cost(card_data["mana_cost"])

    prods = parse_mana_production(oracle)
    if not prods:
        return None

    mana_effects = []
    for prod in prods:
        eff = format_mana_ability(prod)
        if eff is None:
            return None
        mana_effects.append(eff)

    abilities = []
    for eff in mana_effects:
        if "\n" in eff:
            abilities.append(f"""            AbilityDefinition::Activated {{
                cost: Cost::Tap,
                effect: {eff},
                timing_restriction: None,
            }}""")
        else:
            abilities.append(f"""            AbilityDefinition::Activated {{
                cost: Cost::Tap,
                effect: {eff},
                timing_restriction: None,
            }}""")

    abilities_str = ",\n".join(abilities)
    oracle_escaped = oracle.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")

    power_line = ""
    toughness_line = ""
    if card_data.get("power") is not None:
        power_line = f"        power: Some({card_data['power']}),\n"
        toughness_line = f"        toughness: Some({card_data['toughness']}),\n"

    comment_oracle = oracle.replace("\n", " ")[:80]

    return f"""// {display_name} — {comment_oracle}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {{
    CardDefinition {{
        card_id: cid("{slug}"),
        name: "{display_name}".to_string(),
        mana_cost: {mana_cost},
        types: {types_expr},
        oracle_text: "{oracle_escaped}".to_string(),
{power_line}{toughness_line}        abilities: vec![
{abilities_str},
        ],
        ..Default::default()
    }}
}}
"""


# --- Main ---

def main():
    parser = argparse.ArgumentParser(description="Phase 1: Generate template card defs")
    parser.add_argument("--dry-run", action="store_true", help="Don't write files, just print summary")
    parser.add_argument("--group", help="Only process this group_id")
    args = parser.parse_args()

    # Load authoring plan
    with open(PLAN_FILE) as f:
        plan = json.load(f)

    # Open SQLite
    conn = sqlite3.connect(str(DB_FILE))

    # Get existing def files
    existing_files = set()
    if DEFS_DIR.exists():
        existing_files = {f.stem for f in DEFS_DIR.glob("*.rs") if f.name != "mod.rs"}

    results = {"created": [], "skipped": [], "errors": []}
    groups_to_process = [args.group] if args.group else TEMPLATE_GROUPS

    for session in plan["sessions"]:
        group_id = session["group_id"]
        if group_id not in groups_to_process:
            continue
        if session["status"] != "ready":
            continue

        for card in session["cards"]:
            card_name = card["name"]
            slug = card_name_to_slug(card_name)
            filename = slug_to_filename(slug)

            # Skip if already exists
            if filename in existing_files:
                results["skipped"].append({
                    "name": card_name, "reason": "file already exists", "group": group_id
                })
                continue

            # Load from SQLite
            card_data = load_card_data(conn, card_name)
            if card_data is None:
                results["errors"].append({
                    "name": card_name, "reason": "not found in SQLite", "group": group_id
                })
                continue

            # Check skip conditions for mana/etb groups
            oracle = card_data.get("oracle_text", "")

            if group_id == "body-only":
                # Body-only: generate with empty abilities
                # MDFCs with no oracle text on front face are fine
                content = generate_body_only(card_data, card)

            elif group_id == "land-etb-tapped":
                skip_reason = should_skip_etb_tapped(oracle)
                if skip_reason:
                    results["skipped"].append({
                        "name": card_name, "reason": skip_reason, "group": group_id
                    })
                    continue
                content = generate_etb_tapped_land(card_data, card)

            elif group_id in ("mana-land", "mana-artifact", "mana-creature"):
                skip_reason = should_skip_mana_card(oracle)
                if skip_reason:
                    results["skipped"].append({
                        "name": card_name, "reason": skip_reason, "group": group_id
                    })
                    continue
                content = generate_mana_permanent(card_data, card, group_id)

            else:
                continue

            if content is None:
                results["errors"].append({
                    "name": card_name, "reason": "template generation failed (unparseable mana)", "group": group_id
                })
                continue

            # Write file
            filepath = DEFS_DIR / f"{filename}.rs"
            if not args.dry_run:
                filepath.write_text(content)
                existing_files.add(filename)  # prevent duplicates within run

            results["created"].append({
                "name": card_name, "file": f"{filename}.rs", "group": group_id
            })

    conn.close()

    # Print summary
    print(f"\n=== Phase 1 Bulk Generate Summary ===")
    print(f"Created: {len(results['created'])}")
    print(f"Skipped: {len(results['skipped'])}")
    print(f"Errors:  {len(results['errors'])}")

    # Group breakdown
    from collections import Counter
    created_by_group = Counter(c["group"] for c in results["created"])
    skipped_by_group = Counter(s["group"] for s in results["skipped"])
    print(f"\nCreated by group:")
    for g in TEMPLATE_GROUPS:
        print(f"  {g}: {created_by_group.get(g, 0)}")

    if results["skipped"]:
        print(f"\nSkipped by group:")
        for g in TEMPLATE_GROUPS:
            count = skipped_by_group.get(g, 0)
            if count:
                print(f"  {g}: {count}")
        print(f"\nSkip reasons (first 20):")
        for s in results["skipped"][:20]:
            print(f"  [{s['group']}] {s['name']}: {s['reason']}")

    if results["errors"]:
        print(f"\nErrors:")
        for e in results["errors"]:
            print(f"  [{e['group']}] {e['name']}: {e['reason']}")

    # Write log
    if not args.dry_run:
        with open(LOG_FILE, "w") as f:
            json.dump(results, f, indent=2)
        print(f"\nLog written to {LOG_FILE}")

    if args.dry_run:
        print("\n(DRY RUN — no files written)")


if __name__ == "__main__":
    main()

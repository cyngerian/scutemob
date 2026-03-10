#!/usr/bin/env python3
"""
Generate a grouped card authoring plan from combined deck + EDHREC data.

Reads:
  - Deck JSONs (1,174 cards with oracle/keywords)
  - EDHREC combined JSON (popularity/commander data)
  - SQLite DB (oracle text for EDHREC-only cards)
  - Existing card defs (skip already-authored)
  - Ability coverage doc (ready/blocked classification)

Outputs:
  - test-data/test-cards/_authoring_plan.json — grouped batches
  - Summary stats to stderr

Usage:
    python3 test-data/test-cards/generate_authoring_plan.py
    python3 test-data/test-cards/generate_authoring_plan.py --edhrec-threshold 10000
    python3 test-data/test-cards/generate_authoring_plan.py --max-cards 2000
"""

import argparse
import json
import os
import re
import sqlite3
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
PROJECT_ROOT = SCRIPT_DIR.parent.parent
DECK_DIR = PROJECT_ROOT / "test-data" / "test-decks"
EDHREC_PATH = SCRIPT_DIR / "edhrec_all_commanders.json"
SQLITE_PATH = PROJECT_ROOT / "cards.sqlite"
DEFS_PATH = PROJECT_ROOT / "crates" / "engine" / "src" / "cards" / "defs"
COVERAGE_PATH = PROJECT_ROOT / "docs" / "mtg-engine-ability-coverage.md"

# Import the worklist generator's DSL gap + keyword logic
sys.path.insert(0, str(DECK_DIR))
from generate_worklist import (
    DSL_GAP_PATTERNS,
    check_oracle_dsl_gaps,
    parse_ability_coverage,
    resolve_keyword,
    is_blocking_status,
    DEFERRED_KEYWORDS,
    NON_KEYWORDS,
    ABILITY_WORDS,
    KEYWORD_ACTIONS_SUPPORTED,
)


# ── Oracle text pattern groups ───────────────────────────────────────────────
# Order matters: first match wins. More specific patterns before general ones.
# classify_card_group() uses card types to further refine some groups.
ORACLE_GROUPS = [
    # Lands — search
    ("land-fetch", re.compile(
        r"search your library for .*(land|plains|island|swamp|mountain|forest)",
        re.IGNORECASE,
    ), "Lands & Land Search"),

    # Lands — ETB tapped (before generic mana tap)
    ("land-etb-tapped", re.compile(
        r"enters (the battlefield )?tapped", re.IGNORECASE,
    ), "Lands — ETB Tapped"),

    # Mana — tap for color (split by type in classify_card_group)
    ("mana-tap", re.compile(
        r"^\{T\}: Add \{[WUBRGC]\}", re.MULTILINE,
    ), None),  # label assigned dynamically by type
    ("mana-any", re.compile(
        r"\{T\}: Add .*(one mana of any|any color|\{[WUBRGC]\})", re.IGNORECASE,
    ), None),  # label assigned dynamically by type

    # Removal
    ("removal-exile", re.compile(
        r"exile target (creature|permanent|nonland|artifact|enchantment)", re.IGNORECASE,
    ), "Removal — Exile"),
    ("removal-destroy", re.compile(
        r"destroy (target|all|each) (creature|permanent|nonland|artifact|enchantment)",
        re.IGNORECASE,
    ), "Removal — Destroy"),
    ("removal-damage-each", re.compile(
        r"deals? \d+ damage to (each|all) (creature|player|opponent)", re.IGNORECASE,
    ), "Removal — Damage Sweeper"),
    ("removal-damage-target", re.compile(
        r"deals? \d+ damage to (target|any)", re.IGNORECASE,
    ), "Removal — Targeted Damage"),
    ("removal-bounce", re.compile(
        r"return target .* to (its|their) owner", re.IGNORECASE,
    ), "Removal — Bounce"),
    ("removal-minus", re.compile(
        r"target creature gets? [-−]\d+/[-−]\d+", re.IGNORECASE,
    ), "Removal — Debuff"),

    # Counterspells
    ("counter", re.compile(
        r"counter target spell", re.IGNORECASE,
    ), "Counterspells"),

    # Draw & card advantage
    ("draw", re.compile(
        r"draw (a card|two|three|\d+ card)", re.IGNORECASE,
    ), "Draw & Card Advantage"),
    ("tutor", re.compile(
        r"search your library for a card", re.IGNORECASE,
    ), "Tutors"),
    ("scry-surveil", re.compile(
        r"(scry|surveil) \d+", re.IGNORECASE,
    ), "Scry & Surveil"),

    # Tokens
    ("token-create", re.compile(
        r"create .* token", re.IGNORECASE,
    ), "Token Creators"),

    # +1/+1 counters & pump
    ("counters-plus", re.compile(
        r"put .* \+1/\+1 counter", re.IGNORECASE,
    ), "+1/+1 Counters"),
    ("pump-buff", re.compile(
        r"(creatures? you control |target creature )get[s]? \+\d+/\+\d+", re.IGNORECASE,
    ), "Pump & Buff"),

    # ETB triggers
    ("etb-trigger", re.compile(
        r"when(ever)? .* enters (the battlefield)?", re.IGNORECASE,
    ), "ETB Triggers"),

    # Death triggers
    ("death-trigger", re.compile(
        r"when(ever)? .* dies", re.IGNORECASE,
    ), "Death Triggers"),

    # Attack triggers
    ("attack-trigger", re.compile(
        r"when(ever)? .* attacks", re.IGNORECASE,
    ), "Attack Triggers"),

    # Combat keywords (handled by the engine already)
    ("combat-keyword", re.compile(
        r"\b(flying|trample|menace|first strike|double strike|deathtouch|lifelink|vigilance|reach|haste|indestructible|hexproof)\b",
        re.IGNORECASE,
    ), "Combat Keyword Creatures"),

    # Equipment & Auras
    ("equipment", re.compile(
        r"equip(ped)? (\{|creature)", re.IGNORECASE,
    ), "Equipment"),
    ("aura", re.compile(
        r"enchant (creature|permanent|player|land)", re.IGNORECASE,
    ), "Auras & Enchantments"),

    # Activated abilities
    ("activated-sacrifice", re.compile(
        r"sacrifice .+:", re.IGNORECASE,
    ), "Activated — Sacrifice Cost"),
    ("activated-tap", re.compile(
        r"\{T\}.*:", re.IGNORECASE,
    ), "Activated — Tap Cost"),

    # Graveyard
    ("graveyard-recursion", re.compile(
        r"return .* from .* graveyard", re.IGNORECASE,
    ), "Graveyard Recursion"),
    ("mill", re.compile(
        r"mill \d+|put the top \d+ cards .* into .* graveyard", re.IGNORECASE,
    ), "Mill"),

    # Life gain/drain
    ("lifegain", re.compile(
        r"(gain|gains?) \d+ life", re.IGNORECASE,
    ), "Life Gain"),
    ("lifedrain", re.compile(
        r"(lose|loses?) \d+ life", re.IGNORECASE,
    ), "Life Drain"),

    # Protection & prevention
    ("protection", re.compile(
        r"protection from", re.IGNORECASE,
    ), "Protection"),

    # Sacrifice outlet
    ("sacrifice-outlet", re.compile(
        r"sacrifice (a|another) (creature|permanent|artifact)", re.IGNORECASE,
    ), "Sacrifice Outlets"),
]

# ── Sub-patterns for "Other" catch-all ────────────────────────────────────────
OTHER_SUB_GROUPS = [
    ("modal-choice", re.compile(
        r"choose (one|two|three)|you may", re.IGNORECASE,
    ), "Modal & Choice Spells"),
    ("cost-reduction", re.compile(
        r"(costs?|spells?) .* less to cast", re.IGNORECASE,
    ), "Cost Reduction"),
    ("cant-restriction", re.compile(
        r"can't (attack|block|be (blocked|countered|targeted|sacrificed))|can't cast",
        re.IGNORECASE,
    ), "Restrictions & Stax"),
    ("copy-effect", re.compile(
        r"cop(y|ies) (target|a |that )", re.IGNORECASE,
    ), "Copy Effects"),
    ("extra-land", re.compile(
        r"(additional land|play an additional|extra land)", re.IGNORECASE,
    ), "Extra Land Drops"),
    ("discard-effect", re.compile(
        r"(discard|each player discards|each opponent discards)", re.IGNORECASE,
    ), "Discard Effects"),
    ("untap-phase", re.compile(
        r"(untap (all|target|each)|phases? (in|out)|doesn't untap)", re.IGNORECASE,
    ), "Untap & Phase Effects"),
    ("x-spell", re.compile(
        r"\{X\}", re.IGNORECASE,
    ), "X-Cost Spells"),
    ("static-enchantment", re.compile(
        r"(creatures you control (get|have)|spells you control|whenever you cast a)",
        re.IGNORECASE,
    ), "Static Enchantments & Anthems"),
    ("opponent-punish", re.compile(
        r"(each opponent|opponent (sacrifices|loses|discards))", re.IGNORECASE,
    ), "Opponent Punishment"),
    ("cast-trigger", re.compile(
        r"whenever you cast", re.IGNORECASE,
    ), "Cast Triggers"),
    ("exile-play", re.compile(
        r"exile .* (you may (cast|play)|until|from the top)", re.IGNORECASE,
    ), "Exile & Impulse Draw"),
]


def get_authored_cards() -> set[str]:
    """Get card names from existing card definition files."""
    authored = set()
    pattern = re.compile(r'name:\s*"([^"]+)"\.to_string\(\)')
    for fn in os.listdir(DEFS_PATH):
        if fn.endswith(".rs") and fn != "mod.rs":
            with open(DEFS_PATH / fn) as f:
                for line in f:
                    m = pattern.search(line)
                    if m:
                        authored.add(m.group(1))
                        break
    return authored


def get_deck_cards() -> dict[str, dict]:
    """Load all cards from the 20 deck JSON files."""
    cards = {}
    for fn in sorted(os.listdir(DECK_DIR)):
        if not fn.endswith(".json") or fn.startswith("_"):
            continue
        with open(DECK_DIR / fn) as f:
            deck = json.load(f)
        for card in deck.get("cards", []):
            name = card.get("name", "")
            if not name:
                continue
            if name not in cards:
                cards[name] = {
                    "name": name,
                    "types": card.get("types", []),
                    "keywords": card.get("keywords", []),
                    "oracle_text": card.get("oracle_text", ""),
                    "mana_cost": card.get("mana_cost", ""),
                    "cmc": card.get("cmc", 0),
                    "power": card.get("power", ""),
                    "toughness": card.get("toughness", ""),
                    "color_identity": card.get("color_identity", []),
                    "deck_count": 0,
                }
            cards[name]["deck_count"] += 1
    return cards


def get_sqlite_card_data(names: set[str]) -> dict[str, dict]:
    """Look up card data from the local SQLite DB.

    Handles // split names by searching for:
      1. Exact name match
      2. Front face match (name before //)
      3. Full name match in DB (DB uses // format for DFCs)
    """
    if not SQLITE_PATH.exists():
        print(f"WARNING: {SQLITE_PATH} not found, skipping SQLite lookup", file=sys.stderr)
        return {}

    conn = sqlite3.connect(str(SQLITE_PATH))
    cur = conn.cursor()
    cards = {}

    def parse_row(row) -> dict:
        name = row[0]
        type_line = row[2] or ""
        types = []
        for t in ["Creature", "Instant", "Sorcery", "Artifact", "Enchantment",
                   "Land", "Planeswalker", "Battle"]:
            if t in type_line:
                types.append(t)
        keywords_raw = row[3] or "[]"
        try:
            keywords = json.loads(keywords_raw)
        except json.JSONDecodeError:
            keywords = []
        return {
            "name": name,
            "oracle_text": row[1] or "",
            "type_line": type_line,
            "types": types,
            "keywords": keywords,
            "mana_cost": row[4] or "",
            "cmc": row[5] or 0,
            "power": row[6] or "",
            "toughness": row[7] or "",
            "color_identity": json.loads(row[9]) if row[9] else [],
        }

    SELECT_COLS = """name, oracle_text, type_line, keywords, mana_cost,
                     cmc, power, toughness, colors, color_identity"""

    # Pass 1: exact name match (bulk)
    name_list = list(names)
    for i in range(0, len(name_list), 500):
        chunk = name_list[i:i+500]
        placeholders = ",".join("?" * len(chunk))
        cur.execute(f"""
            SELECT {SELECT_COLS} FROM cards
            WHERE name IN ({placeholders})
            AND layout NOT IN ('art_series', 'token', 'emblem', 'double_faced_token')
        """, chunk)
        for row in cur.fetchall():
            if row[0] not in cards:
                cards[row[0]] = parse_row(row)

    # Pass 2: handle // names that weren't found
    missing = names - set(cards.keys())
    for original_name in list(missing):
        if "//" not in original_name:
            # Try front-face substring for single names that might be DFCs
            cur.execute(f"""
                SELECT {SELECT_COLS} FROM cards
                WHERE name LIKE ?
                AND layout NOT IN ('art_series', 'token', 'emblem', 'double_faced_token')
                LIMIT 1
            """, (f"{original_name} // %",))
        else:
            # Try exact match with the // name, or just the front face
            front_face = original_name.split("//")[0].strip()
            cur.execute(f"""
                SELECT {SELECT_COLS} FROM cards
                WHERE name = ? OR name = ? OR name LIKE ?
                AND layout NOT IN ('art_series', 'token', 'emblem', 'double_faced_token')
                LIMIT 1
            """, (original_name, front_face, f"{front_face} // %"))

        row = cur.fetchone()
        if row:
            data = parse_row(row)
            data["name"] = original_name  # preserve the original name
            cards[original_name] = data

    conn.close()
    return cards


def classify_card_group(oracle_text: str, types: list[str]) -> tuple[str, str]:
    """Classify a card into an authoring group by oracle text patterns and card type.
    Returns (group_id, group_label)."""
    if not oracle_text:
        return ("body-only", "Body Only (No Abilities)")

    for group_id, pattern, label in ORACLE_GROUPS:
        if pattern.search(oracle_text):
            # Dynamic label for mana producers — split by card type
            if group_id in ("mana-tap", "mana-any"):
                if "Land" in types:
                    return ("mana-land", "Mana — Lands")
                elif "Artifact" in types and "Creature" not in types:
                    return ("mana-artifact", "Mana — Artifacts (Rocks)")
                elif "Creature" in types:
                    return ("mana-creature", "Mana — Creatures (Dorks)")
                else:
                    return ("mana-other", "Mana — Other")
            return (group_id, label)

    # Nothing matched the primary groups — try sub-patterns for "Other"
    for sub_id, sub_pattern, sub_label in OTHER_SUB_GROUPS:
        if sub_pattern.search(oracle_text):
            return (sub_id, sub_label)

    return ("other", "Other / Miscellaneous")


def classify_authoring_status(card: dict, coverage: dict) -> str:
    """Classify as ready/blocked/deferred using worklist logic."""
    keywords = card.get("keywords", [])
    oracle_text = card.get("oracle_text", "")

    for kw in keywords:
        if kw in DEFERRED_KEYWORDS:
            return "deferred"
        if kw in NON_KEYWORDS or kw in ABILITY_WORDS or kw in KEYWORD_ACTIONS_SUPPORTED:
            continue
        status, _ = resolve_keyword(kw, coverage)
        if is_blocking_status(status):
            return "blocked"

    dsl_gaps = check_oracle_dsl_gaps(oracle_text)
    if dsl_gaps:
        return "blocked"

    return "ready"


def main():
    parser = argparse.ArgumentParser(description="Generate grouped card authoring plan")
    parser.add_argument("--edhrec-threshold", type=int, default=5000,
                        help="Minimum EDHREC total_inclusion to include (default: 5000)")
    parser.add_argument("--max-cards", type=int, default=0,
                        help="Max total cards (0 = no limit)")
    args = parser.parse_args()

    print(f"Loading data...", file=sys.stderr)

    # 1. Load deck cards (always included)
    deck_cards = get_deck_cards()
    print(f"  Deck cards: {len(deck_cards)}", file=sys.stderr)

    # 2. Load EDHREC data
    edhrec_data = {}
    if EDHREC_PATH.exists():
        with open(EDHREC_PATH) as f:
            edhrec_raw = json.load(f)
        for card in edhrec_raw.get("cards", []):
            edhrec_data[card["name"]] = card
        print(f"  EDHREC cards: {len(edhrec_data)}", file=sys.stderr)

    # 3. Build combined card universe
    #    All deck cards + EDHREC cards above threshold
    combined_names = set(deck_cards.keys())
    edhrec_additions = set()
    for name, edata in edhrec_data.items():
        if edata.get("total_inclusion", 0) >= args.edhrec_threshold:
            if name not in combined_names:
                edhrec_additions.add(name)
                combined_names.add(name)

    print(f"  EDHREC additions (>= {args.edhrec_threshold}): {len(edhrec_additions)}", file=sys.stderr)
    print(f"  Combined universe: {len(combined_names)}", file=sys.stderr)

    # 4. Get oracle text for EDHREC-only cards from SQLite
    need_sqlite = edhrec_additions - set(deck_cards.keys())
    sqlite_cards = get_sqlite_card_data(need_sqlite) if need_sqlite else {}
    still_missing = need_sqlite - set(sqlite_cards.keys())
    print(f"  SQLite lookups: {len(need_sqlite)}, found: {len(sqlite_cards)}, missing: {len(still_missing)}", file=sys.stderr)
    if still_missing:
        # Show a few missing for debugging
        sample = sorted(still_missing)[:10]
        print(f"  Missing sample: {sample}", file=sys.stderr)

    # 5. Remove already-authored cards
    authored = get_authored_cards()
    to_author = combined_names - authored
    print(f"  Already authored: {len(authored & combined_names)}", file=sys.stderr)
    print(f"  To author: {len(to_author)}", file=sys.stderr)

    # 6. Load ability coverage for ready/blocked classification
    coverage, _ = parse_ability_coverage(str(COVERAGE_PATH))

    # 7. Build card records with all data merged
    card_records = []
    for name in sorted(to_author):
        record = {"name": name}

        if name in deck_cards:
            d = deck_cards[name]
            record.update({
                "oracle_text": d.get("oracle_text", ""),
                "types": d.get("types", []),
                "keywords": d.get("keywords", []),
                "mana_cost": d.get("mana_cost", ""),
                "cmc": d.get("cmc", 0),
                "power": d.get("power", ""),
                "toughness": d.get("toughness", ""),
                "deck_count": d.get("deck_count", 0),
            })
        elif name in sqlite_cards:
            s = sqlite_cards[name]
            record.update({
                "oracle_text": s.get("oracle_text", ""),
                "types": s.get("types", []),
                "keywords": s.get("keywords", []),
                "mana_cost": s.get("mana_cost", ""),
                "cmc": s.get("cmc", 0),
                "power": s.get("power", ""),
                "toughness": s.get("toughness", ""),
                "deck_count": 0,
            })
        else:
            record.update({
                "oracle_text": "",
                "types": [],
                "keywords": [],
                "mana_cost": "",
                "cmc": 0,
                "power": "",
                "toughness": "",
                "deck_count": 0,
            })

        # EDHREC popularity
        if name in edhrec_data:
            e = edhrec_data[name]
            record["edhrec_inclusion"] = e.get("total_inclusion", 0)
            record["edhrec_commanders"] = len(e.get("commanders", []))
        else:
            record["edhrec_inclusion"] = 0
            record["edhrec_commanders"] = 0

        # Source
        record["source"] = "both" if name in deck_cards and name in edhrec_data else (
            "deck" if name in deck_cards else "edhrec"
        )

        # Authoring status
        record["status"] = classify_authoring_status(record, coverage)

        # Authoring group (uses types for mana producer sub-grouping)
        group_id, group_label = classify_card_group(
            record.get("oracle_text", ""),
            record.get("types", []),
        )
        record["group_id"] = group_id
        record["group_label"] = group_label

        # Priority score: deck_count * 100 + edhrec_commanders * 10 + edhrec_inclusion / 1000
        record["priority_score"] = (
            record["deck_count"] * 100
            + record["edhrec_commanders"] * 10
            + record["edhrec_inclusion"] / 1000
        )

        card_records.append(record)

    # 8. Apply max-cards limit if set (by priority score)
    if args.max_cards > 0 and len(card_records) > args.max_cards:
        card_records.sort(key=lambda c: -c["priority_score"])
        card_records = card_records[:args.max_cards]

    # 9. Group cards
    groups = defaultdict(list)
    for card in card_records:
        groups[card["group_id"]].append(card)

    # Sort within each group by priority score descending
    for group_id in groups:
        groups[group_id].sort(key=lambda c: -c["priority_score"])

    # 10. Split groups into sessions with variable batch sizes
    sessions = []
    session_id = 1

    # Batch sizes by group complexity
    BATCH_SIZES = {
        # Formulaic (16-20): repetitive patterns, minimal variation
        "body-only": 20,
        "combat-keyword": 16,
        "mana-land": 16,
        "mana-artifact": 16,
        "mana-creature": 16,
        "land-etb-tapped": 16,
        # Moderate (10-12): clear DSL pattern with some variation
        "draw": 12,
        "token-create": 12,
        "removal-destroy": 12,
        "removal-damage-each": 12,
        "removal-damage-target": 12,
        "removal-exile": 12,
        "counter": 12,
        "pump-buff": 12,
        "counters-plus": 10,
        "attack-trigger": 10,
        "death-trigger": 10,
        "scry-surveil": 12,
        # Complex (8): unique patterns, needs more agent attention
        # Everything else defaults to 8
    }
    DEFAULT_BATCH_SIZE = 8

    # Sort groups by total priority (highest-value groups first)
    sorted_groups = sorted(
        groups.items(),
        key=lambda kv: -sum(c["priority_score"] for c in kv[1]),
    )

    def make_card_entry(c: dict, include_oracle: bool = True) -> dict:
        entry = {
            "name": c["name"],
            "types": c["types"],
            "keywords": c["keywords"],
            "mana_cost": c["mana_cost"],
            "deck_count": c["deck_count"],
            "edhrec_inclusion": c["edhrec_inclusion"],
            "edhrec_commanders": c["edhrec_commanders"],
            "priority_score": round(c["priority_score"], 1),
            "source": c["source"],
        }
        if include_oracle:
            entry["oracle_text"] = c.get("oracle_text", "")
        return entry

    for group_id, cards in sorted_groups:
        if not cards:
            continue
        label = cards[0]["group_label"]
        batch_size = BATCH_SIZES.get(group_id, DEFAULT_BATCH_SIZE)

        ready_cards = [c for c in cards if c["status"] == "ready"]
        blocked_cards = [c for c in cards if c["status"] == "blocked"]
        deferred_cards = [c for c in cards if c["status"] == "deferred"]

        for i in range(0, len(ready_cards), batch_size):
            batch = ready_cards[i:i+batch_size]
            sessions.append({
                "session_id": session_id,
                "group_id": group_id,
                "group_label": label,
                "status": "ready",
                "card_count": len(batch),
                "cards": [make_card_entry(c) for c in batch],
            })
            session_id += 1

        if blocked_cards:
            for i in range(0, len(blocked_cards), batch_size):
                batch = blocked_cards[i:i+batch_size]
                sessions.append({
                    "session_id": session_id,
                    "group_id": group_id,
                    "group_label": label,
                    "status": "blocked",
                    "card_count": len(batch),
                    "cards": [make_card_entry(c) for c in batch],
                })
                session_id += 1

        if deferred_cards:
            sessions.append({
                "session_id": session_id,
                "group_id": group_id,
                "group_label": label,
                "status": "deferred",
                "card_count": len(deferred_cards),
                "cards": [make_card_entry(c, include_oracle=False) for c in deferred_cards],
            })
            session_id += 1

    # 11. Summary stats
    total_ready = sum(1 for c in card_records if c["status"] == "ready")
    total_blocked = sum(1 for c in card_records if c["status"] == "blocked")
    total_deferred = sum(1 for c in card_records if c["status"] == "deferred")
    ready_sessions = [s for s in sessions if s["status"] == "ready"]
    blocked_sessions = [s for s in sessions if s["status"] == "blocked"]

    summary = {
        "total_cards": len(card_records),
        "already_authored": len(authored & combined_names),
        "ready": total_ready,
        "blocked": total_blocked,
        "deferred": total_deferred,
        "ready_sessions": len(ready_sessions),
        "blocked_sessions": len(blocked_sessions),
        "groups": len(set(c["group_id"] for c in card_records)),
        "edhrec_threshold": args.edhrec_threshold,
        "sources": {
            "deck_only": sum(1 for c in card_records if c["source"] == "deck"),
            "edhrec_only": sum(1 for c in card_records if c["source"] == "edhrec"),
            "both": sum(1 for c in card_records if c["source"] == "both"),
        },
    }

    output = {
        "generated": datetime.now(timezone.utc).isoformat(),
        "summary": summary,
        "sessions": sessions,
    }

    out_path = SCRIPT_DIR / "_authoring_plan.json"
    with open(out_path, "w") as f:
        json.dump(output, f, indent=2)

    # ── Print summary ─────────────────────────────────────────────────────────
    print(f"\n{'='*70}", file=sys.stderr)
    print(f"Card Authoring Plan", file=sys.stderr)
    print(f"{'='*70}", file=sys.stderr)
    print(f"  Total to author:  {len(card_records)}", file=sys.stderr)
    print(f"  Already authored: {len(authored & combined_names)}", file=sys.stderr)
    print(f"  Ready:            {total_ready}", file=sys.stderr)
    print(f"  Blocked:          {total_blocked}", file=sys.stderr)
    print(f"  Deferred:         {total_deferred}", file=sys.stderr)
    print(f"  Ready sessions:   {len(ready_sessions)} (variable batch sizes 8-20)", file=sys.stderr)
    print(f"  Blocked sessions: {len(blocked_sessions)}", file=sys.stderr)
    print(f"", file=sys.stderr)

    # Group summary
    print(f"Groups (by total cards):", file=sys.stderr)
    group_counts = defaultdict(lambda: {"ready": 0, "blocked": 0, "deferred": 0})
    for card in card_records:
        group_counts[card["group_label"]][card["status"]] += 1

    for label, counts in sorted(group_counts.items(), key=lambda kv: -sum(kv[1].values())):
        total = sum(counts.values())
        parts = []
        if counts["ready"]:
            parts.append(f"{counts['ready']} ready")
        if counts["blocked"]:
            parts.append(f"{counts['blocked']} blocked")
        if counts["deferred"]:
            parts.append(f"{counts['deferred']} deferred")
        print(f"  {label:40s}  {total:4d}  ({', '.join(parts)})", file=sys.stderr)

    # First 5 ready sessions
    print(f"\nFirst 5 ready sessions:", file=sys.stderr)
    for session in ready_sessions[:5]:
        print(f"  S{session['session_id']:03d} [{session['group_label']}] ({session['card_count']} cards):", file=sys.stderr)
        for c in session["cards"]:
            deck_str = f"d:{c['deck_count']}" if c["deck_count"] else ""
            edhrec_str = f"e:{c['edhrec_inclusion']}" if c["edhrec_inclusion"] else ""
            src = f"({c['source']}, {deck_str} {edhrec_str})".strip()
            print(f"        {c['name']:40s} {src}", file=sys.stderr)

    # Cards with no oracle text (excluding deferred and body-only)
    no_oracle = sum(1 for c in card_records
                    if not c.get("oracle_text")
                    and c.get("status") != "deferred"
                    and c.get("group_id") != "body-only")
    if no_oracle:
        print(f"\n  WARNING: {no_oracle} cards have no oracle text (missing from SQLite)", file=sys.stderr)

    print(f"\nOutput written to: {out_path}", file=sys.stderr)


if __name__ == "__main__":
    main()

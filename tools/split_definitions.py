#!/usr/bin/env python3
"""One-time migration: split definitions.rs into one file per card.

Parses the monolithic definitions.rs using brace-depth tracking,
extracts each CardDefinition block, and writes individual .rs files
into crates/engine/src/cards/defs/.

Usage:
    python3 tools/split_definitions.py [--dry-run]
"""

import os
import re
import sys

DEFS_FILE = "crates/engine/src/cards/definitions.rs"
OUTPUT_DIR = "crates/engine/src/cards/defs"

# Prefixes used in definitions.rs that become bare names via helpers::*
# (super::card_definition:: is replaced by nothing since helpers re-exports them)
PREFIX_REPLACEMENTS = [
    ("super::card_definition::", ""),
]


def extract_card_blocks(content: str) -> list[dict]:
    """Extract CardDefinition blocks from the vec![...] in all_cards().

    Returns list of dicts with keys: card_id, name, block, comment.
    """
    lines = content.split("\n")
    cards = []
    i = 0

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        # Look for CardDefinition { opening
        if "CardDefinition {" in stripped or "CardDefinition{" in stripped:
            # Capture any preceding comment lines
            comment_lines = []
            j = i - 1
            while j >= 0 and lines[j].strip().startswith("//"):
                comment_lines.insert(0, lines[j].strip())
                j -= 1

            # Track brace depth to find the end of this CardDefinition
            start = i
            depth = 0
            end = i
            for k in range(start, len(lines)):
                for ch in lines[k]:
                    if ch == "{":
                        depth += 1
                    elif ch == "}":
                        depth -= 1
                if depth <= 0:
                    end = k
                    break

            # Extract the block
            block_lines = lines[start : end + 1]
            block = "\n".join(block_lines)

            # Extract card_id from cid("...")
            card_id_match = re.search(r'cid\("([^"]+)"\)', block)
            card_id = card_id_match.group(1) if card_id_match else None

            # Extract name from name: "..."
            name_match = re.search(r'name:\s*"([^"]+)"', block)
            name = name_match.group(1) if name_match else None

            if card_id:
                cards.append(
                    {
                        "card_id": card_id,
                        "name": name or card_id,
                        "block": block,
                        "comment": "\n".join(comment_lines) if comment_lines else None,
                    }
                )

            i = end + 1
        else:
            i += 1

    return cards


def card_id_to_filename(card_id: str) -> str:
    """Convert kebab-case card_id to snake_case filename."""
    return card_id.replace("-", "_").replace("'", "").replace(",", "")


def format_card_file(card: dict) -> str:
    """Generate a standalone .rs file for a single card."""
    block = card["block"]

    # Dedent: find minimum indentation of non-empty lines
    block_lines = block.split("\n")
    non_empty = [l for l in block_lines if l.strip()]
    if non_empty:
        min_indent = min(len(l) - len(l.lstrip()) for l in non_empty)
    else:
        min_indent = 0

    dedented_lines = []
    for l in block_lines:
        if len(l) >= min_indent:
            dedented_lines.append(l[min_indent:])
        else:
            dedented_lines.append(l.strip())

    body = "\n".join(dedented_lines)

    # Remove trailing comma after the closing }
    body = body.rstrip()
    if body.endswith(","):
        body = body[:-1]

    # Apply prefix replacements
    for old, new in PREFIX_REPLACEMENTS:
        body = body.replace(old, new)

    # Build the file
    parts = []
    if card["comment"]:
        # Convert // comments to /// doc comments
        for cline in card["comment"].split("\n"):
            stripped = cline.strip()
            if stripped.startswith("//"):
                # Convert // to /// for doc comments
                rest = stripped[2:].strip()
                parts.append(f"/// {rest}" if rest else "///")
    else:
        parts.append(f"/// {card['name']}")

    parts.append("use crate::cards::helpers::*;")
    parts.append("")
    parts.append("pub fn card() -> CardDefinition {")
    # Indent the body by 4 spaces
    for line in body.split("\n"):
        if line.strip():
            parts.append(f"    {line}")
        else:
            parts.append("")
    parts.append("}")
    parts.append("")  # trailing newline

    return "\n".join(parts)


def main():
    dry_run = "--dry-run" in sys.argv

    if not os.path.exists(DEFS_FILE):
        print(f"Error: {DEFS_FILE} not found. Run from project root.", file=sys.stderr)
        sys.exit(1)

    with open(DEFS_FILE, "r") as f:
        content = f.read()

    cards = extract_card_blocks(content)
    print(f"Found {len(cards)} card definitions")

    if not dry_run:
        os.makedirs(OUTPUT_DIR, exist_ok=True)

    filenames = []
    for card in cards:
        filename = card_id_to_filename(card["card_id"])
        filenames.append(filename)
        file_content = format_card_file(card)

        if dry_run:
            print(f"  Would write: {OUTPUT_DIR}/{filename}.rs ({card['name']})")
        else:
            path = os.path.join(OUTPUT_DIR, f"{filename}.rs")
            with open(path, "w") as f:
                f.write(file_content)
            print(f"  Wrote: {path} ({card['name']})")

    # Check for duplicates
    if len(filenames) != len(set(filenames)):
        dupes = [f for f in filenames if filenames.count(f) > 1]
        print(f"WARNING: duplicate filenames: {set(dupes)}", file=sys.stderr)

    print(f"\nTotal: {len(cards)} card files {'would be ' if dry_run else ''}generated")

    if not dry_run:
        # Write defs/mod.rs
        mod_path = os.path.join(OUTPUT_DIR, "mod.rs")
        with open(mod_path, "w") as f:
            f.write('include!(concat!(env!("OUT_DIR"), "/card_defs_generated.rs"));\n')
        print(f"  Wrote: {mod_path}")


if __name__ == "__main__":
    main()

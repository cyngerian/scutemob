#!/usr/bin/env python3
"""Refresh the printed-field fidelity fixture from the Scryfall card database.

This is the *Scryfall* half of the CARDS-2 field-fidelity gate (SR-37). It performs no
semantic work at all: it joins the corpus name list against `cards.sqlite` and copies the
printed strings out verbatim. Every judgement about whether a definition matches its
printed card is made in Rust, by
`crates/engine/tests/core/cards2_printed_field_fidelity.rs`, against the fixture this
script writes. Keeping the comparison in exactly one place is the point — a fixture that
had already been "normalised" by Python would encode a second, unreviewed opinion about
what a mana cost is.

    Corpus side          `cargo run -p card-field-dump`      (enumerates all_cards())
    Scryfall side        this script                          (joins cards.sqlite)
    Comparison           core::cards2_printed_field_fidelity  (the gate; CI runs this)

`cards.sqlite` is gitignored and absent in CI, which is exactly why the fixture is
committed. This script is run by a human when the corpus gains or loses definitions; the
gate then runs everywhere, forever, with no database.

Usage:

    cargo run -q -p card-field-dump > /tmp/corpus.tsv
    python3 tools/refresh-card-fidelity-fixture.py \
        --corpus /tmp/corpus.tsv \
        --db cards.sqlite \
        --out test-data/card-fidelity/printed-fields.tsv

Names the database does not know are reported on stderr and written to the fixture's
`# unmatched:` trailer, NOT silently dropped: an unmatched name is either a typo in a
definition or a card too new for the local database snapshot, and both need a human.
"""

import argparse
import csv
import sqlite3
import sys
from pathlib import Path

# CR 108.1/111: these Scryfall layouts are not cards a deck can contain. `token` and
# `double_faced_token` are game objects created by effects, `emblem` likewise; `art_series`
# is a collectible with no game text; `planar`, `scheme` and `vanguard` belong to variant
# formats the engine does not implement. Including them would let a token's printed line
# masquerade as an oracle card's when a definition shares its name (e.g. "Angel").
EXCLUDED_LAYOUTS = frozenset(
    {
        "token",
        "double_faced_token",
        "emblem",
        "art_series",
        "planar",
        "scheme",
        "vanguard",
    }
)

# Multi-face layouts: the definition's `mana_cost`/`power`/`toughness`/`types` describe the
# FRONT face only (the back lives in `CardDefinition.back_face`), so the fixture carries
# face 0. `cards.mana_cost` is empty for most of these and the combined `type_line` is
# "Front // Back", neither of which a single-face definition can match.
FACE_JOIN_LAYOUTS = frozenset(
    {
        "transform",
        "modal_dfc",
        "split",
        "adventure",
        "flip",
        "meld",
        "prototype",
        "mutate",
        "leveler",
        "class",
        "case",
        "saga",
        "augment",
        "host",
        "normal",
    }
)


def load_corpus(path):
    """Return [(name, completeness)] from the card-field-dump TSV."""
    rows = []
    with open(path, newline="", encoding="utf-8") as fh:
        reader = csv.reader(fh, delimiter="\t")
        header = next(reader)
        if header[0] != "name":
            sys.exit(f"{path}: unexpected header {header!r} — is this a card-field-dump TSV?")
        for row in reader:
            rows.append((row[0], row[1]))
    return rows


def build_index(db):
    """name -> (mana_cost, power, toughness, type_line), face 0 for multi-face cards.

    Two passes, in this order, so that a whole-card row always wins over a face row: a
    card named "X" and some *other* card's face also named "X" must not collide.
    """
    by_face = {}
    for name, cost, tl, power, toughness, layout in db.execute(
        """select f.name, f.mana_cost, f.type_line, f.power, f.toughness, c.layout
             from card_faces f join cards c on c.id = f.card_id
            where f.face_index = 0"""
    ):
        if layout in EXCLUDED_LAYOUTS:
            continue
        by_face.setdefault(name, (cost, power, toughness, tl))

    by_card = {}
    for name, cost, tl, power, toughness, layout in db.execute(
        "select name, mana_cost, type_line, power, toughness, layout from cards"
    ):
        if layout in EXCLUDED_LAYOUTS:
            continue
        if layout in FACE_JOIN_LAYOUTS and "//" in name:
            # Combined name for a multi-face card: take face 0's printed line.
            face = by_face.get(name.split("//")[0].strip())
            if face is not None:
                by_card.setdefault(name, face)
                continue
        by_card.setdefault(name, (cost, power, toughness, tl))

    merged = dict(by_face)
    merged.update(by_card)
    return merged


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", required=True, help="TSV from `cargo run -p card-field-dump`")
    ap.add_argument("--db", default="cards.sqlite", help="Scryfall SQLite database")
    ap.add_argument("--out", required=True, help="fixture path to write")
    args = ap.parse_args()

    corpus = load_corpus(args.corpus)
    db = sqlite3.connect(f"file:{args.db}?mode=ro", uri=True)
    index = build_index(db)

    matched, unmatched = [], []
    seen = set()
    for name, _completeness in corpus:
        if name in seen:
            continue
        seen.add(name)
        printed = index.get(name)
        if printed is None:
            unmatched.append(name)
            continue
        cost, power, toughness, type_line = printed
        matched.append(
            (
                name,
                cost if cost else "-",
                power if power is not None else "-",
                toughness if toughness is not None else "-",
                type_line,
            )
        )

    matched.sort()
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8", newline="") as fh:
        fh.write("# Printed-field fidelity fixture — CARDS-2 / SR-37. GENERATED, do not hand-edit.\n")
        fh.write("# Source: Scryfall `cards.sqlite` (gitignored); refresh with\n")
        fh.write("#   cargo run -q -p card-field-dump > /tmp/corpus.tsv\n")
        fh.write("#   python3 tools/refresh-card-fidelity-fixture.py \\\n")
        fh.write("#       --corpus /tmp/corpus.tsv --db cards.sqlite --out %s\n" % args.out)
        fh.write("# Columns: name  mana_cost  power  toughness  type_line   ('-' = field absent)\n")
        fh.write("# Multi-face cards carry FACE 0 only; non-game layouts are excluded at extraction.\n")
        fh.write("# The gate is crates/engine/tests/core/cards2_printed_field_fidelity.rs.\n")
        fh.write("name\tmana_cost\tpower\ttoughness\ttype_line\n")
        for row in matched:
            fh.write("\t".join(row) + "\n")
        if unmatched:
            fh.write("# unmatched: %d name(s) absent from the database snapshot:\n" % len(unmatched))
            for name in sorted(unmatched):
                fh.write("#   %s\n" % name)

    print(
        f"{len(matched)} matched / {len(unmatched)} unmatched of {len(seen)} distinct corpus names",
        file=sys.stderr,
    )
    for name in sorted(unmatched):
        print(f"  UNMATCHED {name}", file=sys.stderr)


if __name__ == "__main__":
    main()

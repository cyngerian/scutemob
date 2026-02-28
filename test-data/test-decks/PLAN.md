# Test Decks & Card Authoring Plan

## Goal

Populate `test-data/test-decks/` with 20 real Commander decklists sourced from
Archidekt, extract the unique card set across all decks, and build a pipeline to
scale card-definition authoring from 112 to 1,000+ cards while preserving the
Rust DSL's development advantages.

## Why

The engine currently has 112 hand-authored CardDefinitions. Real Commander decks
have ~65 unique nonland cards + ~35 lands = 100 cards each. Twenty real decks will
surface the cards that matter most in actual play and expose engine gaps (missing
keywords, untested interactions, under-defined card types). Real decks also serve
as future integration test inputs — we can validate that the engine can load,
shuffle, and play through opening turns of actual Commander games.

---

## Phase 1: Deck Selection (20 decks, verified) — DONE

### Selection Criteria
- Cover all 5 mono-colors, all 10 two-color pairs, several 3+ color combos
- Mix of archetypes: aggro, combo, control, midrange, aristocrats, spellslinger, tokens, tribal, voltron, reanimator, stax
- Favor commanders with high popularity on EDHREC (>5000 decks)
- Avoid commanders that rely heavily on mechanics we explicitly defer (Morph, Mutate, Phasing)
- Each deck must be exactly 100 cards (validated via API)

### Curated Deck List (all verified 100 cards)

| #  | Commander                        | Colors | Archetype           | Archidekt ID | URL |
|----|----------------------------------|--------|---------------------|-------------|-----|
| 01 | Atraxa, Praetors' Voice          | WUBG   | Superfriends/+1     | 4752046     | https://archidekt.com/decks/4752046 |
| 02 | Krenko, Mob Boss                 | R      | Tribal Goblin       | 3880705     | https://archidekt.com/decks/3880705 |
| 03 | Yuriko, the Tiger's Shadow       | UB     | Ninja tribal         | 4386        | https://archidekt.com/decks/4386 |
| 04 | Meren of Clan Nel Toth           | BG     | Aristocrats/Reani    | 8063127     | https://archidekt.com/decks/8063127 |
| 05 | Prossh, Skyraider of Kher        | BRG    | Sacrifice/Combo      | 630         | https://archidekt.com/decks/630 |
| 06 | Edgar Markov                     | WBR    | Tribal Vampire       | 59349       | https://archidekt.com/decks/59349 |
| 07 | Omnath, Locus of Creation        | WURG   | Landfall             | 4973670     | https://archidekt.com/decks/4973670 |
| 08 | Korvold, Fae-Cursed King         | BRG    | Sacrifice value      | 4720142     | https://archidekt.com/decks/4720142 |
| 09 | Teysa Karlov                     | WB     | Death triggers       | 1536180     | https://archidekt.com/decks/1536180 |
| 10 | Winota, Joiner of Forces         | WR     | Aggro/cheat          | 15585118    | https://archidekt.com/decks/15585118 |
| 11 | Talrand, Sky Summoner            | U      | Spellslinger         | 3995        | https://archidekt.com/decks/3995 |
| 12 | Selvala, Heart of the Wilds      | G      | Mono-green ramp      | 5150813     | https://archidekt.com/decks/5150813 |
| 13 | Syr Konrad, the Grim             | B      | Graveyard/drain      | 1580608     | https://archidekt.com/decks/1580608 |
| 14 | Sram, Senior Edificer            | W      | Voltron/equipment    | 773070      | https://archidekt.com/decks/773070 |
| 15 | The Ur-Dragon                    | WUBRG  | Tribal Dragon        | 4153743     | https://archidekt.com/decks/4153743 |
| 16 | Niv-Mizzet, Parun                | UR     | Spellslinger/combo   | 57934       | https://archidekt.com/decks/57934 |
| 17 | Lathril, Blade of the Elves      | BG     | Tribal Elf           | 2781081     | https://archidekt.com/decks/2781081 |
| 18 | Chulane, Teller of Tales         | WUG    | Value/bounce         | 921936      | https://archidekt.com/decks/921936 |
| 19 | Isshin, Two Heavens as One       | WBR    | Attack triggers      | 2400793     | https://archidekt.com/decks/2400793 |
| 20 | Aesi, Tyrant of Gyre Strait      | UG     | Landfall/draw        | 6349098     | https://archidekt.com/decks/6349098 |

### Color coverage

- **W**: Sram (W), Teysa (WB), Winota (WR), Edgar (WBR), Isshin (WBR), Chulane (WUG), Atraxa (WUBG), Omnath (WURG), Ur-Dragon (WUBRG)
- **U**: Talrand (U), Yuriko (UB), Niv-Mizzet (UR), Aesi (UG), Chulane (WUG), Atraxa (WUBG), Omnath (WURG), Ur-Dragon (WUBRG)
- **B**: Syr Konrad (B), Yuriko (UB), Teysa (WB), Meren (BG), Lathril (BG), Edgar (WBR), Isshin (WBR), Prossh (BRG), Korvold (BRG), Atraxa (WUBG), Ur-Dragon (WUBRG)
- **R**: Krenko (R), Winota (WR), Niv-Mizzet (UR), Edgar (WBR), Isshin (WBR), Prossh (BRG), Korvold (BRG), Omnath (WURG), Ur-Dragon (WUBRG)
- **G**: Selvala (G), Meren (BG), Lathril (BG), Aesi (UG), Chulane (WUG), Prossh (BRG), Korvold (BRG), Atraxa (WUBG), Omnath (WURG), Ur-Dragon (WUBRG)

All 5 mono-colors represented. Missing 2-color pairs (WU, WG, BR, RG) covered within 3+ color decks.

### Archetype coverage
- **Aggro/tokens**: Krenko, Edgar, Winota
- **Aristocrats/sacrifice**: Meren, Prossh, Korvold, Teysa
- **Combo/spellslinger**: Talrand, Niv-Mizzet
- **Control/value**: Yuriko, Chulane, Aesi
- **Landfall/ramp**: Omnath, Selvala, Aesi
- **Tribal**: Krenko (Goblin), Edgar (Vampire), Ur-Dragon (Dragon), Lathril (Elf), Yuriko (Ninja)
- **Voltron/equipment**: Sram
- **Graveyard**: Meren, Syr Konrad
- **Attack triggers**: Isshin

---

## Phase 2: Data Source & Format — DONE

### Source: Archidekt Public API

Each deck is fetched from the Archidekt public API. These are real player-built
decks with coherent strategies — not statistical composites like EDHREC averages.

**API endpoint:**
```
GET https://archidekt.com/api/decks/{id}/
Header: User-Agent: Mozilla/5.0
```

Response includes full card data with oracle text, types, mana cost, color identity,
keywords, and categories (Commander, Land, Creature, etc.).

### Output Format

Each deck stored as a JSON file:

```json
{
  "commander": "Krenko, Mob Boss",
  "colors": ["R"],
  "archetype": "Tribal Goblin",
  "source": "archidekt",
  "source_url": "https://archidekt.com/decks/3880705",
  "archidekt_id": 3880705,
  "fetched_date": "2026-02-28",
  "card_count": 100,
  "cards": [
    {
      "name": "Sol Ring",
      "quantity": 1,
      "types": ["Artifact"],
      "mana_cost": "{1}",
      "cmc": 1,
      "color_identity": [],
      "commander": false,
      "oracle_text": "{T}: Add {C}{C}.",
      "keywords": []
    }
  ]
}
```

**Filename convention:** `NN_commander-slug.json` (e.g., `02_krenko-mob-boss.json`)

---

## Phase 3: Fetch Script — DONE

`test-data/test-decks/fetch_decks.py` fetches all 20 decks. Run with:
```bash
python3 test-data/test-decks/fetch_decks.py
python3 test-data/test-decks/fetch_decks.py --skip-fetch  # re-analyze only
```

---

## Phase 4: Unique Card Extraction & Analysis — DONE

Results from the fetch:

| Metric | Value |
|--------|-------|
| Total unique cards | 1,174 |
| Already have definitions | 51 |
| Needing definitions | 1,123 |
| Tier 1 (10+ decks) | 0 needing defs (all staples already defined) |
| Tier 2 (5-9 decks) | 13 |
| Tier 3 (2-4 decks) | 224 |
| Tier 4 (1 deck) | 886 |
| Cards with NO keywords | 678 |
| Cards with keywords | 445 |

Top Tier 2 gaps: Skullclamp, Ancient Tomb, Blood Artist, Cavern of Souls,
Enlightened Tutor, Heroic Intervention, Viscera Seer, Worldly Tutor.

---

## Phase 5: Ability-Gated Worklist

Not all 1,123 cards can be authored today. Cards that use unimplemented keywords
must wait until the engine supports them.

### Classification

Cross-reference each card's keywords against `docs/mtg-engine-ability-coverage.md`:

- **Ready**: All keywords `validated` or `complete` in the engine. Author now.
- **Blocked**: Has >= 1 keyword with status `none`. Author after ability is implemented.
- **Deferred**: Needs Morph, Mutate, Phasing, or Transform — explicitly deferred mechanics.

### Top Blocking Keywords (by cards blocked)

| Keyword | Status | Cards Blocked | Notes |
|---------|--------|---------------|-------|
| Transform | P3 `none` | 9 | DFC support needed |
| Ninjutsu | P4 `none` | 8 | Doable, just not done |
| Mutate | P3 `none` (deferred) | 8 | Explicitly deferred |
| Channel | needs check | 5 | |
| Overload | P3 `none` | 2 | "Replace target with each" |
| Cipher | P4 `none` | 3 | |
| Toxic | P4 `none` | 3 | |

### Non-Keyword Gating

Keywords are the primary gate, but some cards need engine capabilities beyond
keywords (e.g., modal choice for "Choose one" spells, X-cost spells, loyalty
abilities). These are harder to detect automatically. The worklist script should
flag cards with complex oracle text patterns for manual review.

### Worklist Script

Extend `fetch_decks.py` (or create `generate_worklist.py`) to:

1. Parse ability coverage doc for keyword → status mapping
2. Classify each card in `_cards_needing_definitions.json` as ready/blocked/deferred
3. Output `_authoring_worklist.json` with:
   - Ready cards sorted by deck frequency (author these first)
   - Blocked cards grouped by missing keyword (unblocked when ability is implemented)
   - Deferred cards (don't touch until mechanic is added)
4. Print a summary: "N cards ready to author, M blocked, K deferred"

---

## Phase 6: Refactor Card Definitions Structure

### Problem

`definitions.rs` is 3,118 lines with all 112 cards in one `vec![]` literal. At
500+ cards this becomes unworkable: agent insertion failures, slow compilation,
merge conflicts between parallel authoring sessions.

### Solution

Create a separate `card-defs` crate with one file per card and `build.rs`
auto-discovery.

```
crates/
├── engine/              (rules, state, effects — unchanged)
├── card-defs/
│   ├── Cargo.toml       (depends on engine for DSL types)
│   ├── build.rs         (scans cards/, generates mod declarations)
│   └── src/
│       ├── lib.rs        (pub fn all_cards(), re-exports helpers)
│       ├── helpers.rs    (cid, types, types_sub, creature_types, etc.)
│       └── cards/
│           ├── mod.rs    (generated by build.rs)
│           ├── sol_ring.rs
│           ├── arcane_signet.rs
│           ├── lightning_greaves.rs
│           └── ...
```

Each card file:
```rust
use crate::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sol-ring"),
        name: "Sol Ring".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add {C}{C}.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(0, 0, 0, 0, 0, 2),
            },
            timing_restriction: None,
        }],
        ..Default::default()
    }
}
```

### Benefits

- **Agent targets one small file** — trivial to write, can't break other cards
- **Zero merge conflicts** between parallel authoring sessions
- **Incremental compilation** — adding a card doesn't recompile the engine
- **Full Rust DSL preserved** — type checking, IDE autocomplete, refactoring safety
- **Scales to thousands** without architectural changes

### Migration Steps

1. Create `crates/card-defs/` crate with `Cargo.toml`, `build.rs`, `src/lib.rs`
2. Write `build.rs` that scans `src/cards/` and generates `mod.rs`
3. Extract `helpers.rs` from current `definitions.rs` helper functions
4. Move each existing card definition to its own file in `src/cards/`
5. Update `engine/src/cards/definitions.rs` to re-export from `card-defs`
   (or update the two call sites directly: `lib.rs`, `replay_harness.rs`)
6. Verify all tests pass — zero behavioral change
7. Update `card-definition-author` agent to target individual files

### Call Site Changes

Only two places consume `all_cards()`:
- `crates/engine/src/lib.rs` (re-export)
- `crates/engine/src/testing/replay_harness.rs`

Both switch from `cards::definitions::all_cards()` to `card_defs::all_cards()`.

---

## Phase 7: Fix the Card-Definition-Author Agent

### Current Problems

The `card-definition-author` agent (Sonnet, maxTurns: 12) has ~50% failure rate
in batch sessions:

1. **Doesn't call Edit** — generates the definition mentally but never writes it
2. **Wrong struct syntax** — uses `TriggeredAbilityDef(...)` tuple syntax instead of
   `AbilityDefinition::Triggered { trigger_condition, effect, intervening_if }`
3. **Insertion point confusion** — can't reliably find the right spot in a 3k line file

### Fixes to Try

After Phase 6 refactor, most issues disappear:
- Agent creates a new file instead of editing a huge one (no insertion point problem)
- File is small enough to validate by reading it back
- Template is simpler (whole file, not splice into vec)

Additional prompt improvements:
- Add explicit "verify your edit" step (grep for card name after writing)
- Stronger language: "You MUST call Write/Edit. Generating code without writing it is a failure."
- Add compilation check guidance (though agent lacks Bash access)
- Bump maxTurns from 12 to 16

### Performance Tracking

Create `test-data/test-decks/_authoring_log.json` to track each agent run:

```json
{
  "runs": [
    {
      "card": "Skullclamp",
      "timestamp": "2026-03-01T10:00:00Z",
      "agent_version": "v2",
      "success": true,
      "failure_type": null,
      "tokens_used": 24000,
      "turns_used": 8,
      "notes": ""
    }
  ],
  "summary": {
    "total_runs": 10,
    "successes": 8,
    "failures": 2,
    "success_rate": 0.80,
    "by_version": {
      "v1": { "runs": 5, "successes": 3 },
      "v2": { "runs": 5, "successes": 5 }
    }
  }
}
```

Run batches of 5-10 cards, compare before/after agent prompt changes.

---

## Phase 8: Scryfall Skeleton Generator

### Concept

Most of a CardDefinition is mechanical data available from Scryfall: name, types,
subtypes, mana cost, P/T, oracle text, keywords. Only the `abilities` vec requires
human/agent authoring.

### Script: `generate_skeleton.py`

Given a card name:
1. Fetch from Scryfall API (`api.scryfall.com/cards/named?exact=...`)
2. Generate a `.rs` file with all mechanical fields filled in
3. Leave `abilities: vec![]` with TODO comments derived from oracle text
4. Write to `crates/card-defs/src/cards/<slug>.rs`

Example output for Skullclamp:
```rust
use crate::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skullclamp"),
        name: "Skullclamp".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +1/-1.\n\
            Whenever equipped creature dies, draw two cards.\n\
            Equip {1}".to_string(),
        abilities: vec![
            // TODO: Static — equipped creature gets +1/-1
            // TODO: Triggered — when equipped creature dies, draw 2
            // TODO: Equip {1}
        ],
        ..Default::default()
    }
}
```

### Batch Mode

```bash
# Generate skeletons for all ready cards from the worklist
python3 generate_skeleton.py --from-worklist _authoring_worklist.json --status ready
```

This pre-populates files. The agent or human then only needs to fill in `abilities`.

---

## Phase 9: Batch Authoring

### Workflow

1. Run worklist generator (Phase 5) to get current ready/blocked/deferred lists
2. Pick a batch of 5-10 ready cards, prioritized by deck frequency
3. Generate skeletons (Phase 8) for the batch
4. Author abilities — either:
   - `card-definition-author` agent (one card per invocation)
   - Manual authoring using existing definitions as templates
5. `cargo check` after each card to verify compilation
6. `cargo test --all` after the full batch
7. Log results to `_authoring_log.json` (Phase 7)
8. Commit: `W1-cards: author <card1>, <card2>, ...`

### Priority Order

1. **Tier 2 ready cards** (5-9 decks, all keywords implemented): ~13 cards
2. **Tier 3 ready cards** (2-4 decks): ~200 cards
3. **Tier 4 ready cards** (1 deck): ~700+ cards
4. **Blocked cards** as abilities are implemented

---

## Execution Order

| Step | Phase | Status | Prereqs |
|------|-------|--------|---------|
| 1 | Phase 1-4: Deck fetch & analysis | DONE | — |
| 2 | Phase 5: Build ability-gated worklist | TODO | Phase 4 data |
| 3 | Phase 6: Refactor definitions structure | TODO | — |
| 4 | Phase 7: Fix card-definition-author agent | TODO | Phase 6 (new file structure) |
| 5 | Phase 8: Scryfall skeleton generator | TODO | Phase 6 (target directory) |
| 6 | Phase 9: Batch authoring begins | TODO | Phases 5-8 |

Phase 5 (worklist) can start immediately — it only needs Phase 4 data. Phase 6
(refactor) is the critical path for everything else.

---

## Notes

- **No partner commanders.** Partners add complexity (2 commanders per deck). None
  in the list use partners.
- **Lands count.** Nonbasic lands with abilities (fetches, shocks, utility) are
  included in the priority list — many have relevant abilities (ETB tapped, fetch,
  color fixing) that the engine should model.
- **Deck swaps.** If any Archidekt deck goes private or gets deleted, the script
  will log the failure. Alternative IDs for each commander were tested during
  curation and can be substituted.
- **Scaling strategy.** See `docs/mtg-engine-card-pipeline.md` for the full
  architectural discussion of how this approach scales from 112 to 27,000+ cards.

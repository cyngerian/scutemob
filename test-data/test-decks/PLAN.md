# Test Decks Plan

## Goal

Populate `test-data/test-decks/` with 20 real Commander decklists sourced from
Archidekt. These are actual player-built decks with coherent strategies, synergy
packages, and tuned mana bases — not statistical composites. Then extract the
unique card set across all decks to prioritize card-definition authoring.

## Why

The engine currently has 112 hand-authored CardDefinitions. Real Commander decks
have ~65 unique nonland cards + ~35 lands = 100 cards each. Twenty real decks will
surface the cards that matter most in actual play and expose engine gaps (missing
keywords, untested interactions, under-defined card types). Real decks also serve
as future integration test inputs — we can validate that the engine can load,
shuffle, and play through opening turns of actual Commander games.

---

## Phase 1: Deck Selection (20 decks, verified)

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

## Phase 2: Data Source & Format

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
    },
    {
      "name": "Krenko, Mob Boss",
      "quantity": 1,
      "types": ["Creature"],
      "subtypes": ["Goblin", "Warrior"],
      "mana_cost": "{2}{R}{R}",
      "cmc": 4,
      "color_identity": ["Red"],
      "commander": true,
      "oracle_text": "{T}: Create X 1/1 red Goblin creature tokens...",
      "keywords": [],
      "power": 3,
      "toughness": 3
    }
  ]
}
```

**Filename convention:** `NN_commander-slug.json` (e.g., `02_krenko-mob-boss.json`)

---

## Phase 3: Fetch Script

Create `test-data/test-decks/fetch_decks.py` that:

1. Has the 20 commander entries hardcoded (name, archidekt_id, colors, archetype)
2. For each commander:
   - Fetches the Archidekt API JSON
   - Filters out Maybeboard/Sideboard cards
   - Extracts card data (name, quantity, types, subtypes, mana_cost, cmc, color_identity, oracle_text, keywords, power/toughness)
   - Marks the commander card
   - Validates exactly 100 cards
   - Writes `NN_slug.json`
3. After all decks are fetched, generates analysis files:
   - `_summary.json`: deck-level stats (card count, color, archetype per deck)
   - `_unique_cards.json`: deduplicated card list across all decks, sorted by frequency
   - `_cards_needing_definitions.json`: cards NOT in `definitions.rs`, sorted by frequency

### Dependencies
- Python 3 standard library only (urllib, json, pathlib, time)
- No pip installs needed
- Rate limit: 0.5s sleep between requests (polite to Archidekt)

---

## Phase 4: Unique Card Extraction & Analysis

After fetch completes, `_unique_cards.json` will contain:

```json
{
  "total_unique_cards": 842,
  "total_with_definitions": 95,
  "total_needing_definitions": 747,
  "cards": [
    {
      "name": "Sol Ring",
      "appears_in_decks": 20,
      "has_definition": true,
      "types": ["Artifact"],
      "keywords": []
    },
    {
      "name": "Cyclonic Rift",
      "appears_in_decks": 12,
      "has_definition": false,
      "types": ["Instant"],
      "keywords": ["Overload"]
    }
  ]
}
```

This gives a clear priority list: **cards appearing in the most decks that lack
definitions are the highest priority for authoring.**

---

## Phase 5: Card Authoring Prioritization

Using frequency data from Phase 4, batch cards for authoring:

1. **Tier 1 (appears in 10+ decks):** ~30-50 cards — Commander staples (Sol Ring,
   Swords, Cyclonic Rift, Rhystic Study, etc.). Many already have definitions.
2. **Tier 2 (appears in 5-9 decks):** ~50-80 cards — Widely-played support cards
3. **Tier 3 (appears in 2-4 decks):** ~100-200 cards — Archetype staples
4. **Tier 4 (appears in 1 deck):** ~300-500 cards — Niche/commander-specific cards

Focus authoring effort on Tiers 1-2 first, as they give the best coverage per card defined.

---

## Execution Order

1. Write `fetch_decks.py` with the 20 deck entries from the table above
2. Run the script to populate deck JSONs
3. Verify all 20 decks: 100 cards each, commander present, no maybeboard leakage
4. Generate `_unique_cards.json` and `_cards_needing_definitions.json`
5. Review the frequency-sorted gap list to plan card authoring batches

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

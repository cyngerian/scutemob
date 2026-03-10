# Card Authoring Plan

> **Generated**: 2026-03-10
> **Data source**: `_authoring_plan.json` (1,743 card universe from 20 decks + EDHREC >= 5k inclusion)
> **Status**: PLANNING — no execution started

---

## Overview

| Metric | Count |
|--------|-------|
| Total card universe | 1,743 |
| Already authored | 107 |
| To author | 1,636 |
| Ready (authorable now) | 1,471 |
| Blocked (DSL/keyword gaps) | 141 |
| Deferred (Morph/Mutate/etc.) | 24 |
| Ready sessions (batches of 8) | 206 |

---

## Execution Strategy

Three phases, each followed by a mandatory audit phase.

### Phase 1: Bulk Template Generation (Python script, no agents)

**Target**: ~313 cards with deterministic DSL patterns.

Write `test-data/test-cards/bulk_generate.py` that reads `_authoring_plan.json` + SQLite
card data and generates complete `.rs` card definition files for groups where the oracle
text maps to a single, known DSL pattern.

| Group | Cards | Template Pattern |
|-------|-------|------------------|
| Body Only (No Abilities) | 55 | `abilities: vec![]`, mechanical fields only |
| Lands — ETB Tapped | 122 | `enters_tapped: true` + tap-for-color mana ability |
| Mana — Lands | 84 | `AbilityDefinition::Activated { cost: Cost::Tap, effect: Effect::AddMana { ... } }` |
| Mana — Artifacts (Rocks) | 33 | Same as Mana Lands but `types: Artifact` |
| Mana — Creatures (Dorks) | 19 | Same but `creature_types(...)` with P/T |

**Template data sources**:
- Card name, mana cost, types, subtypes, P/T, oracle text → SQLite `cards` table
- Color production → parsed from oracle text regex (`\{T\}: Add \{([WUBRGC]+)\}`)
- ETB tapped condition → parsed from oracle text
- For "any color" producers: `Effect::AddManaAnyColor` (existing DSL pattern)

**Known template variations to handle**:
- Dual lands: `{T}: Add {W} or {B}` → two-color mana pool
- Tri lands: `{T}: Add {W}, {U}, or {B}` → three-color
- Any-color: `{T}: Add one mana of any color` → `AddManaAnyColor`
- Conditional ETB: "enters tapped unless you control a [type]" → `enters_tapped: true` + TODO comment for condition
- Filter lands: "Add one mana of any color that a land an opponent controls could produce" → `AddManaAnyColor` approximation + TODO
- Bounce lands: "When ~ enters, return a land you control to its owner's hand" → ETB trigger + mana ability

**Output**: One `.rs` file per card in `crates/engine/src/cards/defs/`. `build.rs` auto-discovers.

**Estimated effort**: Write the script (~300-400 lines), run it, `cargo build` to verify.

#### `bulk_generate.py` Detailed Spec

**Location**: `test-data/test-cards/bulk_generate.py`

**Inputs**:
- `_authoring_plan.json` — session groups (knows which cards are in which group)
- `cards.sqlite` — oracle text, type_line, mana_cost, power, toughness, subtypes
- `crates/engine/src/cards/defs/` — skip already-existing files

**Card ID convention**: `cid("card-name")` — lowercase, spaces→hyphens, strip apostrophes/commas.
Use same slug logic as `generate_skeleton.py`.

**File name convention**: `card_name.rs` — lowercase, spaces→underscores, strip apostrophes/commas/colons.

**All generated files use**: `use crate::cards::helpers::*;`

**Helper functions available** (from `crates/engine/src/cards/helpers.rs`):
- `cid(s: &str) -> CardId` — creates card ID from slug
- `types(card_types: &[CardType]) -> TypeLine` — basic type line
- `types_sub(card_types: &[CardType], subtypes: &[&str]) -> TypeLine` — with subtypes
- `creature_types(subtypes: &[&str]) -> TypeLine` — shorthand for creature with subtypes
- `mana_pool(w: u32, u: u32, b: u32, r: u32, g: u32, colorless: u32) -> ManaPool`
- `treasure_token_spec(count: u32) -> TokenSpec`

**ManaCost parsing** — parse `{2}{B}{G}` into `ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }`.
Lands have `mana_cost: None`. Mana cost string format: `{X}` = generic X, `{W}` = white, `{U}` = blue,
`{B}` = black, `{R}` = red, `{G}` = green, `{C}` = colorless, `{0}` = zero.

**Subtype parsing** — extract from SQLite `type_line` column. Format: `"Creature — Elf Druid"` →
subtypes are everything after `—`, split by space. For lands: `"Land — Gate"` → `&["Gate"]`.

**Template 1: Body Only** (`group_id == "body-only"`)
```rust
// Card Name
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("card-name"),
        name: "Card Name".to_string(),
        mana_cost: Some(ManaCost { ... }),  // or None for lands
        types: creature_types(&["Subtype1", "Subtype2"]),  // or types(&[...])
        oracle_text: "".to_string(),
        power: Some(N),      // creatures only
        toughness: Some(N),  // creatures only
        abilities: vec![],
        back_face: None,
    }
}
```

**Template 2: ETB Tapped Land** (`group_id == "land-etb-tapped"`)
Reference: `crates/engine/src/cards/defs/dimir_guildgate.rs`
```rust
// Card Name — Land; enters tapped. {T}: Add {X} or {Y}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("card-name"),
        name: "Card Name".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Subtype"]),  // or types(&[CardType::Land])
        oracle_text: "...".to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Choose {            // for 2+ colors
                    prompt: "Add {X} or {Y}?".to_string(),
                    choices: vec![
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(...) },
                        Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(...) },
                    ],
                },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
```
Variations:
- Single color: use `Effect::AddMana` directly (no `Choose`)
- Three colors (triomes): `Choose` with 3 options
- Any color: `Effect::AddManaAnyColor { player: PlayerTarget::Controller }`
- Conditional ETB ("unless you control..."): use same template + TODO comment for condition
- Has additional abilities beyond mana (e.g., Castle Locthwain): skip template, leave for Phase 2

**Template 3: Mana Land** (`group_id == "mana-land"`)
Reference: `crates/engine/src/cards/defs/command_tower.rs`
```rust
// Same as ETB Tapped but WITHOUT the Replacement block.
// Only the Activated { cost: Cost::Tap, effect: AddMana/AddManaAnyColor/Choose }
```

**Template 4: Mana Artifact** (`group_id == "mana-artifact"`)
Reference: `crates/engine/src/cards/defs/arcane_signet.rs`
```rust
// Same as Mana Land but types: types(&[CardType::Artifact])
// or types_sub(&[CardType::Artifact], &["Subtype"]) if it has subtypes
// Has mana_cost: Some(ManaCost { ... })
```

**Template 5: Mana Creature** (`group_id == "mana-creature"`)
Reference: `crates/engine/src/cards/defs/elvish_mystic.rs`
```rust
// Same activated ability but creature_types(&[...]) + power/toughness
// Has mana_cost, power: Some(N), toughness: Some(N), back_face: None
```

**Mana color parsing from oracle text**:
```python
# Parse "{T}: Add {W} or {B}." or "{T}: Add {G}." or "{T}: Add {W}, {U}, or {B}."
# Regex: r"\{T\}: Add (?:\{([WUBRGC])\}(?:,?\s*(?:or\s+)?)?)+"
# Map: W=white, U=blue, B=black, R=red, G=green, C=colorless
# "any color" / "one mana of any color" → AddManaAnyColor
```

**Skip rules** — do NOT template a card if:
- It has additional oracle text beyond the mana ability (e.g., "Whenever you tap..." or "When ~ enters...")
- It has a complex ETB condition not expressible as `EntersTapped`
- Its oracle text can't be fully parsed by the template regex
- A file already exists at the target path

For skipped cards, log the name and reason so they fall through to Phase 2.

**Script output**:
- Creates `.rs` files
- Prints summary: N files created, M skipped (with reasons)
- Writes `_bulk_generate_log.json` with per-card results

### Phase 2: Skeleton + Bulk Abilities Agent (agent-assisted)

**Target**: ~1,158 remaining ready cards with complex/unique oracle text.

#### Step 2a: Generate Skeletons

Use existing `tools/generate_skeleton.py` to create skeleton files for all remaining
ready cards. Skeletons have all mechanical fields filled, `abilities: vec![]` with
TODO comments derived from oracle text.

```bash
python3 tools/generate_skeleton.py --from-worklist test-data/test-cards/_authoring_plan.json --status ready --skip-existing
```

#### Step 2b: Bulk Abilities Agent

Create a new agent (or modify `card-definition-author`) that:
1. Reads a session from `_authoring_plan.json` (8 cards, same group)
2. Reads one reference card def from the same group to learn the DSL pattern
3. Fills in `abilities: vec![...]` for all 8 cards in the session
4. Runs `cargo build --lib -p mtg-engine` to verify compilation

**Agent design**:
- Name: `bulk-card-author` (or extend `card-definition-author`)
- Model: Sonnet (fast, good at pattern replication)
- Input: session ID from `_authoring_plan.json`
- Tools: Read, Write, Edit, Glob, Grep, Bash, `mcp__mtg-rules__lookup_card`
- One invocation per session (8 cards), not per card
- Reads `crates/engine/src/cards/helpers.rs` for available types
- Reads 1-2 existing card defs from the same group as reference

**Session execution order** (by group, highest-priority first):
1. Combat Keyword Creatures (163 ready, ~21 sessions) — simple: just list keywords
2. Draw & Card Advantage (161 ready, ~21 sessions) — `Effect::DrawCards` pattern
3. Token Creators (146 ready, ~19 sessions) — `Effect::CreateToken` pattern
4. Modal & Choice Spells (100 ready, ~13 sessions) — `Effect::Conditional` / modes
5. Removal — Destroy (48 ready, ~6 sessions) — `Effect::DestroyPermanent` pattern
6. +1/+1 Counters (42 ready, ~6 sessions) — `Effect::AddCounters` pattern
7. Attack Triggers (33 ready, ~5 sessions) — `TriggerCondition::WhenAttacks`
8. Death Triggers (24 ready, ~3 sessions) — `TriggerCondition::WheneverCreatureDies`
9. Remaining groups (~441 ready, ~56 sessions)

**Cards that should use `abilities: vec![]`** (per W5 policy):
- Cards with oracle text that exceeds DSL expressiveness
- Cards where a partial implementation would produce wrong behavior
- The agent must document the gap as a TODO comment in the file

### Phase 3: Blocked & Deferred Cards (future)

141 blocked cards become authorable as DSL gaps are filled. 24 deferred cards
need Morph/Mutate/Transform support (already implemented — update `DEFERRED_KEYWORDS`
in `generate_worklist.py` and re-run).

**Top blocking DSL gaps** (from `generate_worklist.py`):
| Gap | Cards Blocked | What's Needed |
|-----|---------------|---------------|
| `targeted_trigger` | 57 | `targets` field on `AbilityDefinition::Triggered` |
| `return_from_graveyard` | 17 | `Effect::ReturnFromGraveyardToHand` |
| `nonbasic_land_search` | 15 | Subtype-OR in `TargetFilter` for `SearchLibrary` |
| `count_threshold` | 14 | `Condition::YouControlAtLeastN` |
| `shock_etb` | 10 | `MayPay` replacement effect |

---

## Phase 4: Audit (mandatory after every phase)

Every card definition — whether templated or agent-written — must be audited.

### Audit Process

1. **Batch size**: 5 cards per reviewer agent invocation
2. **Agent**: `ability-impl-reviewer` (Opus) — already proven in the 20-card batch
3. **Per card, the reviewer checks**:
   - Oracle text matches Scryfall (via `mcp__mtg-rules__lookup_card`)
   - Mana cost, types, subtypes, P/T are correct
   - DSL usage is correct (right Effect variants, field names, enum values)
   - No overbroad triggers (e.g., `WheneverCreatureDies` for "another creature you control")
   - No no-op placeholders that make unimplemented cards castable (`GainLife(0)`)
   - TODO comments accurately describe what's missing
   - `abilities: vec![]` used where partial implementation would be wrong
4. **Reviewer writes findings** to `memory/abilities/ability-review-cards-batch-NNN.md`
5. **Fix agent** (`ability-impl-runner`) applies fixes, re-compiles
6. **Commit** after each fix pass

### Audit Schedule

| After Phase | Cards to Audit | Reviewer Sessions (5/batch) | Estimated |
|-------------|---------------|----------------------------|-----------|
| Phase 1 | ~313 templated | ~63 sessions | Systematic template bugs likely — high ROI |
| Phase 2 | ~1,158 agent-written | ~232 sessions | Agent errors vary — medium ROI |
| Total | ~1,471 | ~295 sessions | |

### Audit Parallelism

- Run 4 reviewer agents in parallel (proven in the 20-card batch)
- Each reviewer takes ~2 min → 4 parallel = ~1 review/30s
- 295 sessions ÷ 4 parallel = ~74 rounds × 2 min = ~2.5 hours total
- Fix sessions run sequentially after each review wave

### Known Audit Patterns (from 20-card batch review)

These are the bugs the reviewer found in the first batch. Templates and agents
should avoid these, but the audit will catch them:

| Pattern | Severity | Example |
|---------|----------|---------|
| Wrong target filter | HIGH | `TargetPermanent` instead of `TargetPermanentWithFilter(non_land)` |
| Overbroad trigger | MEDIUM | `WheneverCreatureDies` for "another creature you control" |
| No-op placeholder castable | MEDIUM | `GainLife(0)` makes spell castable when it shouldn't be |
| Wrong field name | COMPILE | `target:` instead of `player:` on `GainLife` |
| Missing constructor arg | COMPILE | `treasure_token_spec()` needs `treasure_token_spec(1)` |
| Wrong PlayerTarget | MEDIUM | `Controller` when card says "its owner" (multiplayer) |

---

## Updating the Worklist

After authoring cards, regenerate the worklist so the TUI stays current:

```bash
# Regenerate the worklist (reads defs/ for authored count)
python3 test-data/test-decks/generate_worklist.py

# Regenerate the authoring plan (reads defs/ + EDHREC + decks)
python3 test-data/test-cards/generate_authoring_plan.py
```

The TUI reads `_authoring_worklist.json` directly for the status bar.

**Note**: `generate_worklist.py` only knows about the 20-deck universe (1,174 cards).
The authoring plan (`_authoring_plan.json`) is the comprehensive source (1,743 cards).
The TUI should eventually be updated to read `_authoring_plan.json` instead, but this
is not blocking.

---

## Deferred Keyword Update

Morph, Mutate, Transform, Disguise, Manifest, Cloak, Daybound, Nightbound are all
implemented in the engine now. `generate_worklist.py` still lists them in `DEFERRED_KEYWORDS`.
Before Phase 3, update that set to remove implemented mechanics so those cards move
from `deferred` to `ready`.

---

## File Inventory

| File | Purpose |
|------|---------|
| `test-data/test-cards/AUTHORING_PLAN.md` | This plan |
| `test-data/test-cards/_authoring_plan.json` | Session data (groups, cards, oracle text) |
| `test-data/test-cards/edhrec_all_commanders.json` | EDHREC data for 20 commanders |
| `test-data/test-cards/generate_authoring_plan.py` | Generates `_authoring_plan.json` |
| `test-data/test-cards/fetch_edhrec_cards.py` | Fetches EDHREC data |
| `test-data/test-decks/generate_worklist.py` | Generates `_authoring_worklist.json` (TUI) |
| `test-data/test-decks/_authoring_worklist.json` | TUI-facing card status |
| `tools/generate_skeleton.py` | Creates skeleton `.rs` files from Scryfall data |

---

## Execution Checklist

- [ ] Phase 1: Write `bulk_generate.py` template script
- [ ] Phase 1: Run template generation (~313 cards)
- [ ] Phase 1: `cargo build --lib -p mtg-engine` — verify compilation
- [ ] Phase 1: `cargo test --all` — verify no regressions
- [ ] Phase 1 Audit: Run reviewer on all templated cards (63 sessions × 4 parallel)
- [ ] Phase 1 Audit: Apply fixes
- [ ] Phase 1: Commit
- [ ] Phase 2a: Generate skeletons for remaining ~1,158 cards
- [ ] Phase 2a: `cargo build` — verify skeletons compile
- [ ] Phase 2b: Create/modify bulk-card-author agent
- [ ] Phase 2b: Run agent sessions by group (206 - ~39 templated = ~167 sessions)
- [ ] Phase 2b: `cargo build` after each session
- [ ] Phase 2 Audit: Run reviewer on all agent-written cards (232 sessions × 4 parallel)
- [ ] Phase 2 Audit: Apply fixes
- [ ] Phase 2: Commit
- [ ] Phase 3: Update `DEFERRED_KEYWORDS` in `generate_worklist.py`
- [ ] Phase 3: Re-run `generate_authoring_plan.py` to move deferred → ready
- [ ] Phase 3: Author newly-ready cards
- [ ] Regenerate `_authoring_worklist.json` for TUI
- [ ] Update CLAUDE.md card count

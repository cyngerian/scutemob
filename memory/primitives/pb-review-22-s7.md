# Primitive Batch Review: PB-22 S7 — Adventure (CR 715) + Dual-Zone Search

**Date**: 2026-03-21
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 715.1-715.5, 715.2a-715.2c, 715.3-715.3d, 715.4, 715.5, 601.3e, 701.23
**Engine files reviewed**: `state/types.rs`, `state/stack.rs`, `state/game_object.rs`, `state/hash.rs`, `cards/card_definition.rs`, `rules/casting.rs`, `rules/resolution.rs`, `rules/copy.rs`, `effects/mod.rs`
**Card defs reviewed**: 5 (bonecrusher_giant.rs, lovestruck_beast.rs, monster_manual.rs, finale_of_devastation.rs, lozhan_dragons_legacy.rs)

## Verdict: needs-fix

The Adventure engine infrastructure is well-implemented. Casting from hand, exile-on-resolution,
cast-creature-from-exile, and the anti-re-adventure guard are all correct per CR 715.3a/3b/3d.
The fizzle and counter paths correctly do NOT exile Adventure spells (CR 715.3d). Hash coverage
is complete. Dual-zone search is clean.

However, Monster Manual's adventure face implements the WRONG oracle text (library search instead
of mill-and-return), which is a HIGH finding. The copy system does not propagate
`was_cast_as_adventure` to copies, violating CR 715.3c (MEDIUM). The LegalActionProvider in the
simulator does not offer Adventure casting or creature-from-adventure-exile (MEDIUM).
Bonecrusher Giant's trigger condition is wrong (opponent-only vs any spell, MEDIUM).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `copy.rs:207` | **CR 715.3c: Copies of Adventure spells should also be Adventures.** `was_cast_as_adventure` hardcoded to `false` for copies. **Fix:** propagate from original. |
| 2 | **MEDIUM** | `legal_actions.rs` | **LegalActionProvider missing Adventure paths.** No Adventure cast from hand or creature-from-exile. **Fix:** add LegalAction variants. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | **HIGH** | `monster_manual.rs` | **Wrong oracle text for adventure face.** Implements library search; oracle says mill+return. **Fix:** replace with mill+return effect. |
| 4 | **MEDIUM** | `bonecrusher_giant.rs:29` | **Trigger fires on opponent spells only; oracle says any spell.** Uses `WhenBecomesTargetByOpponent`. **Fix:** document TODO noting trigger should fire on ANY spell. |

### Finding Details

#### Finding 1: CR 715.3c — Copies of Adventure spells not marked as Adventure

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/copy.rs:207`
**CR Rule**: 715.3c -- "If an Adventure spell is copied, the copy is also an Adventure. It has the alternative characteristics of the spell and not the normal characteristics of the card that represents the Adventure spell."
**Issue**: `was_cast_as_adventure` is hardcoded to `false` for all copies. When an Adventure spell is copied (e.g., via Casualty, Storm), the copy resolves using the main face's Spell effect instead of the adventure face's effect (resolution.rs:245 checks this flag). The exile-on-resolution is moot for copies (they cease to exist), but the wrong effect would execute.
**Fix**: Change `copy.rs:207` to propagate from original: `was_cast_as_adventure: original.was_cast_as_adventure,`. This ensures the copy uses the adventure face's effect at resolution. The other copy sites in engine.rs (cascade, discover, etc.) should remain `false` since those are free-cast copies of new cards, not copies of the stack object.

#### Finding 2: LegalActionProvider missing Adventure casting paths

**Severity**: MEDIUM
**File**: `crates/simulator/src/legal_actions.rs`
**CR Rule**: 715.3 -- "As a player plays an adventurer card, the player chooses whether they play the card normally or as an Adventure."
**Issue**: The LegalActionProvider does not emit legal actions for: (a) casting a card as an Adventure from hand, or (b) casting a creature from adventure exile. The random bot and heuristic bot will never attempt these casts. This is consistent with other alt-cost keywords (Spectacle, Surge, etc.) where the bot always uses `alt_cost: None`, but Adventure is unique because the Adventure half may be the only castable option at a given mana cost.
**Fix**: Add a `CastAsAdventure { card: ObjectId }` variant to `LegalAction` and emit it when a card in hand has `adventure_face.is_some()` in its CardDefinition and the player has sufficient mana for the adventure cost. Add a `CastFromAdventureExile { card: ObjectId }` variant emitted for cards in exile with `adventure_exiled_by == Some(player)`. Alternatively, defer this to the W2 TUI phase (bot improvements) and document the gap. Given that other alt costs are similarly unimplemented in the bot, deferral is acceptable if documented.

#### Finding 3: Monster Manual adventure face has wrong oracle text

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/monster_manual.rs:28-63`
**Oracle**: Monster Manual // Zoological Study — Adventure face: "Mill five cards, then return a creature card from your graveyard to your hand." (confirmed by MCP: Keywords: ["Mill"])
**Issue**: The adventure face implements `Effect::SearchLibrary` (search library for creature, put onto battlefield, shuffle). The actual oracle text is "Mill five cards, then return a creature card from your graveyard to your hand." The comment at lines 25-27 incorrectly claims "Per Scryfall oracle" for the search text, but the MCP lookup confirms "Mill" is a keyword on this card, matching the original comment at line 6. The card's main face has the library-search-like ability ("{1}{G}, {T}: You may put a creature card from your hand onto the battlefield"); the implementer appears to have confused the two abilities.
**Fix**: Replace the adventure face's Spell effect with `Effect::Sequence(vec![Effect::MillCards { player: PlayerTarget::Controller, count: 5 }, Effect::MoveFromGraveyardToHand { ... }])`. Since `Effect::MoveFromGraveyardToHand` may not exist, use the available DSL for "return a creature card from your graveyard to your hand" (likely `Effect::SearchLibrary` targeting graveyard, or a dedicated move-zone effect). Update the oracle_text string and the comment. At minimum, the Spell effect must involve milling, not library search.

#### Finding 4: Bonecrusher Giant trigger fires on opponent spells only

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/bonecrusher_giant.rs:29`
**Oracle**: "Whenever Bonecrusher Giant becomes the target of a spell, Bonecrusher Giant deals 2 damage to that spell's controller."
**Issue**: The trigger uses `TriggerCondition::WhenBecomesTargetByOpponent`, which only fires when an opponent targets it. The oracle text says "becomes the target of a spell" with no restriction on who controls that spell -- the trigger should fire when ANY player (including the controller) targets it. The existing TODO at line 28 mentions the `EffectTarget::TriggeringPlayer` gap but does not note that the trigger condition itself is wrong.
**Fix**: Update the TODO comment to explicitly note that `WhenBecomesTargetByOpponent` is incorrect -- it should be `WhenBecomesTargetBySpell` (or equivalent that fires for ANY spell, not just opponent spells). This is a DSL gap: `TriggerCondition::WhenBecomesTargetBySpell` does not exist. Document both gaps (trigger condition + effect target) in the TODO.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 715.1 | N/A (flavor) | N/A | Card frame description |
| 715.2 | Yes | Partial | `adventure_face` field on CardDefinition |
| 715.2a | No | No | "has an Adventure" predicate not exposed for effects/triggers |
| 715.2b | No | No | Copiable values do not include adventure_face in copy system |
| 715.2c | N/A | N/A | "one card" — no draw/discard splitting needed |
| 715.3 | Yes | Yes | `AltCostKind::Adventure` casting choice |
| 715.3a | Yes | Yes | `test_adventure_cast_adventure_half_from_hand` |
| 715.3b | Yes | Yes | Type/cost override in casting.rs, is_permanent=false in resolution.rs |
| 715.3c | **No** | No | Finding 1: copies not marked as Adventure |
| 715.3d | Yes | Yes | `test_adventure_exile_on_resolution`, `test_adventure_cast_creature_from_exile`, `test_adventure_countered_goes_to_graveyard`, `test_adventure_cannot_recast_as_adventure_from_exile` |
| 715.4 | Yes | Yes | `test_adventure_normal_characteristics_in_hand` |
| 715.5 | N/A | N/A | Card name choice — no card naming system exists yet |
| 701.23 | Yes | Yes | `also_search_graveyard` on SearchLibrary, 3 dual-zone tests |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| bonecrusher_giant.rs | Partial | 2 (trigger condition, prevention) | Wrong trigger scope | Finding 4: trigger fires opponent-only, should fire for any spell |
| lovestruck_beast.rs | Yes | 1 (attack restriction) | Correct (adventure face) | Attack restriction is documented DSL gap |
| monster_manual.rs | **No** | 1 (activated ability) | **Wrong** | Finding 3: adventure face implements search instead of mill+return |
| finale_of_devastation.rs | Partial | 2 (X-based filter, X>=10 pump) | Partial | `also_search_graveyard: true` correct; CMC filter and conditional pump gaps documented |
| lozhan_dragons_legacy.rs | Yes | 1 (full trigger) | N/A (unimplemented) | TODOs correctly updated to note Adventure framework exists |

## Test Coverage Assessment

The 9 tests are well-structured and cover the core Adventure flow thoroughly:
- Positive: cast from hand, exile on resolution, cast creature from exile (3 tests)
- Negative: countered goes to graveyard, cannot re-adventure from exile (2 tests)
- Zone characteristics: normal characteristics in non-stack zones (1 test)
- Dual-zone search: library-only, graveyard, shuffle verification (3 tests)

Missing test coverage:
- No test for Adventure copy behavior (CR 715.3c) — blocked by Finding 1
- No test for Adventure + other alt cost mutual exclusion
- No test for Adventure casting when card has no adventure_face (error path)

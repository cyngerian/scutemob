# Primitive Batch Review: PB-M -- Panharmonicon Trigger Doubling

**Date**: 2026-04-09
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 603.2d
**Engine files reviewed**: `crates/engine/src/rules/abilities.rs`, `crates/engine/src/rules/replacement.rs`, `crates/engine/src/state/stubs.rs`, `crates/engine/src/state/hash.rs`
**Card defs reviewed**: 4 (panharmonicon.rs, drivnod_carnage_dominus.rs, elesh_norn_mother_of_machines.rs, ancient_greenwarden.rs)

## Verdict: needs-fix

Ancient Greenwarden card def has an incorrect `SuperType::Legendary` that does not match the
card's oracle text. One planned test for Bug 2 (CardDef-based ETB trigger doubling via
`queue_carddef_etb_triggers`) was not written, leaving that fix pathway untested. All engine
changes are CR-correct and well-structured. The remaining three card defs are accurate.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| (none) | -- | -- | All engine changes are correct. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **HIGH** | `ancient_greenwarden.rs:20` | **Incorrect Legendary supertype.** Oracle says "Creature -- Elemental" (no Legendary). Def has `SuperType::Legendary`. **Fix:** Change `full_types(&[SuperType::Legendary], ...)` to `full_types(&[], &[CardType::Creature], &["Elemental"])`. |
| 2 | MEDIUM | `trigger_doubling.rs` | **Missing test for Bug 2 (CardDef ETB path).** The `entering_object_id` fix in `queue_carddef_etb_triggers` is untested. All PB-M tests use ObjectSpec `with_triggered_ability` which goes through `check_triggers`, not `queue_carddef_etb_triggers`. **Fix:** Add `test_panharmonicon_doubles_carddef_etb_trigger` using a card with `AbilityDefinition::Triggered { trigger_condition: WhenEntersBattlefield, ... }` to exercise the CardDef ETB path with trigger doubling. |

### Finding Details

#### Finding 1: Incorrect Legendary supertype on Ancient Greenwarden

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/ancient_greenwarden.rs:20`
**Oracle**: "Creature -- Elemental" (MCP lookup confirms no Legendary supertype; mana cost {4}{G}{G}, P/T 5/7)
**Issue**: The card def declares `full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elemental"])` but Ancient Greenwarden is NOT a Legendary creature. This causes incorrect game state: the legend rule would apply to it (SBA 704.5j), preventing a player from controlling two copies, and it could be searched by "legendary" filters. The Legendary supertype is a game-relevant characteristic that affects multiple engine systems.
**Fix**: Change line 20 to `full_types(&[], &[CardType::Creature], &["Elemental"])` to remove the Legendary supertype.

#### Finding 2: Missing test for Bug 2 (CardDef ETB trigger doubling pathway)

**Severity**: MEDIUM
**File**: `crates/engine/tests/trigger_doubling.rs`
**CR Rule**: 603.2d -- trigger doubling must work for CardDef-based ETB triggers
**Issue**: Bug 2 fixed `entering_object_id` being `None` in `queue_carddef_etb_triggers` at two sites (lines 1185 and 1224 in replacement.rs). Without `entering_object_id`, `doubler_applies_to_trigger` cannot check the entering permanent's card types and conservatively skips doubling. However, no test exercises this specific code path. All existing tests use ObjectSpec `with_triggered_ability(TriggeredAbilityDef { trigger_on: SelfEntersBattlefield, ... })` which goes through the `check_triggers` pathway (not `queue_carddef_etb_triggers`). The fix is likely correct but has zero test coverage.
**Fix**: Add a test `test_panharmonicon_doubles_carddef_etb_trigger` that:
1. Creates a CardDefinition with `AbilityDefinition::Triggered { trigger_condition: TriggerCondition::WhenEntersBattlefield, ... }` (a CardDef-based ETB, not an ObjectSpec trigger).
2. Registers a Panharmonicon TriggerDoubler.
3. Casts the card so it resolves and enters via `queue_carddef_etb_triggers`.
4. Asserts the CardDef ETB trigger fires twice (1 baseline + 1 doubled).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 603.2d (ArtifactOrCreatureETB + SelfETB) | Yes | Yes | test_panharmonicon_doubles_self_etb_trigger |
| 603.2d (ArtifactOrCreatureETB negative) | Yes | Yes | test_panharmonicon_does_not_double_enchantment_etb |
| 603.2d (AnyPermanentETB) | Yes | Yes | test_any_permanent_etb_doubler_doubles_enchantment |
| 603.2d (LandETB positive) | Yes | Yes | test_land_etb_doubler_doubles_landfall_not_creature (Part 1) |
| 603.2d (LandETB negative) | Yes | Yes | test_land_etb_doubler_doubles_landfall_not_creature (Part 2) |
| 603.2d (multiple doublers additive) | Yes | Yes | test_two_panharmonicons_triple_triggers |
| 603.2d (removal after trigger) | Yes | Yes | test_panharmonicon_removal_doesnt_cancel_already_triggered |
| 603.2d (registration pipeline) | Yes | Yes | test_panharmonicon_registration_via_resolution |
| 603.2d (CardDef ETB path) | Yes | **No** | Bug 2 fix untested (Finding 2) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| panharmonicon | Yes | 0 | Yes | Clean |
| drivnod_carnage_dominus | Yes | 1 (activated ability DSL gap) | Yes | TODO is legitimate out-of-scope gap |
| elesh_norn_mother_of_machines | Yes | 1 (opponent ETB suppression) | Yes | TODO is legitimate out-of-scope gap |
| ancient_greenwarden | **No** | 0 | **No** | Finding 1: incorrect Legendary supertype |

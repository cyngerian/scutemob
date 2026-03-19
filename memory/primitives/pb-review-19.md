# Primitive Batch Review: PB-19 -- Mass Destroy / Board Wipes

**Date**: 2026-03-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 701.8 (Destroy), 702.12 (Indestructible), 701.19 (Regenerate), 406.2 (Exile), 903.9 (Commander zone)
**Engine files reviewed**: `effects/mod.rs`, `cards/card_definition.rs`, `state/hash.rs`
**Card defs reviewed**: 12 (wrath_of_god, damnation, supreme_verdict, path_of_peril, sublime_exhalation, final_showdown, scavenger_grounds, vanquish_the_horde, fumigate, bane_of_progress, ruinous_ultimatum, cyclonic_rift)

## Verdict: needs-fix

The engine primitives (DestroyAll, ExileAll, EffectAmount::LastEffectCount, AddCounterAmount) are well-implemented with correct CR compliance for the core destroy/exile logic. The AllPermanentsMatching controller filter fix is correct. Hash support is complete. The Fumigate integration test is thorough. Two substantive findings: (1) DestroyAll does not count commander-redirected permanents as "destroyed this way" which is incorrect per CR 903.9a (commanders go to graveyard first, then SBA moves them -- the destruction still happened), and (2) Cyclonic Rift uses `ControllerOf` where oracle text says "owner's hand" -- no `OwnerOf` PlayerTarget exists yet.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `effects/mod.rs:897-899` | **DestroyAll skips destroyed_count for commander redirect.** Commander permanents are "destroyed this way" even if redirected to command zone. **Fix:** increment `destroyed_count` in the `ZoneId::Command(_)` arm. |
| 2 | LOW | `effects/mod.rs:1017-1018,1048-1049` | **ExileAll ObjectExiled event uses ctx.controller instead of owner.** Pre-existing pattern (ExileObject does the same), but semantically the `player` field should be the owner. **Fix:** change `player: ctx.controller` to `player: owner` in both Proceed and Redirect arms of ExileAll. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | **MEDIUM** | `cyclonic_rift.rs:41,53` | **Uses ControllerOf instead of OwnerOf for "owner's hand."** Oracle says "Return ... to its owner's hand" but DSL lacks OwnerOf. **Fix:** add `PlayerTarget::OwnerOf` variant or document as known gap; current behavior is wrong when controller != owner. |
| 4 | LOW | `final_showdown.rs:40,46` | **TODOs remain for modes 0 and 1.** These require LoseAbilities/GainKeyword primitives not yet in DSL. Acceptable for PB-19 scope but should be tracked. |

### Finding Details

#### Finding 1: DestroyAll skips destroyed_count for commander redirect

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:897-899`
**CR Rule**: 903.9a -- "If a commander is in a graveyard or in exile and that object was put into that zone since the last time state-based actions were checked, its owner may put it into the command zone."
**Issue**: The `ZoneId::Command(_)` arm in DestroyAll's redirect match does not increment `destroyed_count`. Per CR 903.9a, commanders that are destroyed go to the graveyard first, then an SBA allows them to be moved to the command zone. The engine's current implementation appears to redirect directly to command zone via a replacement effect. Either way, the destruction event DID occur -- the commander WAS "destroyed this way." For cards like Fumigate, the controller should gain life even for destroyed commanders. The current code at line 898 comments "Does not count as destroyed (CR 701.8b)" but CR 701.8b says nothing about command zone redirection -- it defines what constitutes destruction.
**Fix**: Add `destroyed_count += 1;` inside the `ZoneId::Command(_) => { }` arm, and also emit a `CreatureDied` or `PermanentDestroyed` event if appropriate (the creature was destroyed, it just ended up in the command zone instead of the graveyard). Alternatively, if the engine models the commander replacement as preventing destruction entirely, document this deviation from CR and accept the inaccuracy.

#### Finding 2: ExileAll ObjectExiled uses ctx.controller instead of owner

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:1017-1018,1048-1049`
**CR Rule**: 406.2 -- exile zone is public, but the `player` field on `ObjectExiled` should identify the card's owner for correct event routing
**Issue**: The ExileAll implementation uses `player: ctx.controller` for ObjectExiled events, but the object's `owner` was already resolved at line 988. The `player` field should be `owner`, matching the DestroyAll redirect-to-exile pattern (line 891) which correctly uses `owner`. This is a pre-existing inconsistency from `ExileObject`, but ExileAll introduces more instances.
**Fix**: In both the `Redirect` non-command arm (line 1017) and the `Proceed` arm (line 1048), change `player: ctx.controller` to `player: owner`.

#### Finding 3: Cyclonic Rift uses ControllerOf for "owner's hand"

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/cyclonic_rift.rs:41,53`
**Oracle**: "Return target nonland permanent you don't control to its **owner's** hand."
**Issue**: The card def uses `PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 }))` to resolve the destination hand. `ControllerOf` resolves to the controller of the permanent, but oracle text says "owner's hand." When a player controls another player's permanent (e.g., via Control Magic), `ControllerOf` would bounce it to the controller's hand (wrong) instead of the owner's hand (correct). The DSL currently lacks a `PlayerTarget::OwnerOf` variant.
**Fix**: Add a `PlayerTarget::OwnerOf(Box<EffectTarget>)` variant to `PlayerTarget` in `card_definition.rs`, wire it in `resolve_player_target_list` to look up `obj.owner` instead of `obj.controller`, add hash support, and update this card def to use it. This is a DSL gap that affects multiple bounce spells. If deferred, add a TODO comment documenting the deviation.

#### Finding 4: Final Showdown modes 0 and 1 are no-ops with TODOs

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/final_showdown.rs:40,46`
**Oracle**: Mode 0: "All creatures lose all abilities until end of turn." Mode 1: "Choose a creature you control. It gains indestructible until end of turn."
**Issue**: Both modes are `Effect::Sequence(vec![])` no-ops with TODO comments. This is expected -- the required primitives (LoseAbilities, GainKeyword) don't exist yet. PB-19 only targeted mode 2 (DestroyAll). However, these TODOs should be tracked for future primitive batches.
**Fix**: No immediate fix needed. Ensure these are tracked in the DSL gap audit (`memory/card-authoring/dsl-gap-audit.md`).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 701.8a (Destroy) | Yes | Yes | test_destroy_all_creatures_basic |
| 701.8c (Regeneration replaces) | Yes | Yes | test_destroy_all_allows_regeneration |
| 702.12b (Indestructible) | Yes | Yes | test_destroy_all_respects_indestructible |
| 701.19c (Can't be regenerated) | Yes | Yes | test_destroy_all_cant_be_regenerated |
| 406.2 (Exile) | Yes | Yes | test_exile_all_basic, test_exile_all_count_tracking |
| 702.89a (Umbra Armor) | Yes | No | Code handles it but no test verifies umbra armor interaction with DestroyAll |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| wrath_of_god | Yes | 0 | Yes | |
| damnation | Yes | 0 | Yes | |
| supreme_verdict | Yes | 0 | Yes | cant_be_countered correctly set |
| path_of_peril | Yes | 0 | Yes | Cleave + max_cmc filter correct |
| sublime_exhalation | Yes | 0 | Yes | Undaunted keyword present |
| final_showdown | Partial | 2 | Partial | Modes 0/1 are no-ops; mode 2 correct |
| scavenger_grounds | Yes | 0 | Yes | ForEach+ExileObject pattern correct |
| vanquish_the_horde | Yes | 0 | Yes | SelfCostReduction + DestroyAll correct |
| fumigate | Yes | 0 | Yes | Sequence + LastEffectCount correct |
| bane_of_progress | Yes | 0 | Yes | ETB trigger + AddCounterAmount correct |
| ruinous_ultimatum | Yes | 0 | Yes | Opponent controller filter + non_land correct |
| cyclonic_rift | Partial | 0 | Partial | "Owner's hand" uses ControllerOf (Finding 3) |

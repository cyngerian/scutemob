# Primitive Batch Review: PB-35 -- Modal Triggers + Graveyard Conditions

**Date**: 2026-03-28
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 700.2, 700.2b, 603.3c, 602.2
**Engine files reviewed**: `card_definition.rs` (ActivationZone, TriggerZone, AbilityDefinition::Activated/Triggered fields), `hash.rs` (HashInto for ActivationZone, TriggerZone, Activated/Triggered arms), `abilities.rs` (handle_activate_ability graveyard check, collect_graveyard_carddef_triggers, flush_pending_triggers modal mode selection), `resolution.rs` (modal trigger dispatch at ~L1891-1934), `replay_harness.rs` (activation_zone propagation), `helpers.rs` (exports), `game_object.rs` (ActivatedAbility.activation_zone)
**Card defs reviewed**: 14 total (retreat_to_kazandu, retreat_to_coralhelm, felidar_retreat, junji_the_midnight_sky, shambling_ghast, tectonic_giant, hullbreaker_horror, glissa_sunslayer, goblin_cratermaker, umezawas_jitte, reassembling_skeleton, bloodghast, earthquake_dragon, cult_conscript)

## Verdict: needs-fix

One HIGH finding (Shambling Ghast wrong trigger condition), three MEDIUM findings (Shambling Ghast target filter, Bloodghast missing "may", Umezawa's Jitte trigger scope). The HIGH must be fixed before closing PB-35. The MEDIUMs are wrong game state but documented approximations may be acceptable for pre-alpha.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| - | - | - | No engine change findings. All engine changes are correct. |

Engine changes are clean:
- `ActivationZone` and `TriggerZone` enums are correctly defined with `#[serde(default)]` on fields.
- `hash.rs` correctly hashes both new enums and both new fields on Activated/Triggered.
- `handle_activate_ability` correctly checks `activation_zone` and gates graveyard activation on `obj.zone == ZoneId::Graveyard(player)`.
- `collect_graveyard_carddef_triggers` correctly scans graveyard objects matching `trigger_zone: Some(TriggerZone::Graveyard)` and fires for `WheneverPermanentEntersBattlefield` events with filter matching.
- `flush_pending_triggers` correctly sets `modes_chosen = vec![0]` for both min_modes 0 and min_modes >= 1 cases (bot fallback).
- `resolution.rs` modal dispatch correctly looks up modes from CardDef, maps `modes_chosen` to effects, falls back to base effect when no modes match.
- `helpers.rs` exports both new types.
- `ActivatedAbility` struct on `GameObject` has `activation_zone` field, propagated via `enrich_spec_from_def`.

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **HIGH** | `shambling_ghast.rs` | **Wrong trigger condition.** Oracle says "When this creature dies", def uses `WhenEntersBattlefield`. |
| 2 | MEDIUM | `shambling_ghast.rs` | **Wrong target filter on mode 1.** Oracle says "Target creature an opponent controls", def targets any creature. |
| 3 | MEDIUM | `bloodghast.rs` | **Missing "may" on return effect.** Oracle says "you may return", def always returns. |
| 4 | MEDIUM | `umezawas_jitte.rs` | **Trigger too narrow.** Oracle says "deals combat damage" (to anything), def uses `WhenEquippedCreatureDealsCombatDamageToPlayer`. |
| 5 | LOW | `tectonic_giant.rs` | **Partial trigger condition.** Only WhenAttacks; missing "or becomes the target of a spell an opponent controls". Documented TODO. |
| 6 | LOW | `hullbreaker_horror.rs` | **Missing "can't be countered".** Documented TODO; needs engine support for uncounterable creature spells. |
| 7 | LOW | `glissa_sunslayer.rs` | **Mode 2 placeholder.** Effect::Nothing for "remove up to three counters". Documented TODO; DSL gap. |
| 8 | LOW | `cult_conscript.rs` | **Missing activation condition.** "Activate only if non-Skeleton creature died" deferred. Documented TODO. |
| 9 | LOW | `bloodghast.rs` | **Missing "can't block".** No KeywordAbility::CantBlock in DSL. Documented TODO. |

### Finding Details

#### Finding 1: Shambling Ghast -- Wrong Trigger Condition

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/shambling_ghast.rs:25`
**Oracle**: "When this creature dies, choose one -- Target creature an opponent controls gets -1/-1 until end of turn. / Create a Treasure token."
**Issue**: The card def uses `TriggerCondition::WhenEntersBattlefield` but the oracle text says "When this creature **dies**". This is a death trigger, not an ETB trigger. The comment at line 1-3 also incorrectly states "When Shambling Ghast enters" -- the author appears to have confused the card's trigger. This produces completely wrong game state: the modal choice fires on ETB instead of on death, meaning the card gives a Treasure when it enters rather than when it dies.
**Fix**: Change `trigger_condition: TriggerCondition::WhenEntersBattlefield` to `trigger_condition: TriggerCondition::WhenDies`. Update the file header comment to say "When Shambling Ghast dies" instead of "When Shambling Ghast enters".

#### Finding 2: Shambling Ghast -- Wrong Target Filter

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/shambling_ghast.rs:30`
**Oracle**: "Target creature **an opponent controls** gets -1/-1 until end of turn."
**Issue**: The targets list uses `TargetRequirement::TargetCreature` which allows targeting any creature (including your own). The oracle specifies "an opponent controls" -- the target must be an opponent's creature.
**Fix**: Change `TargetRequirement::TargetCreature` to `TargetRequirement::TargetCreatureWithFilter(TargetFilter { controller: TargetController::Opponent, ..Default::default() })` (or equivalent -- verify the exact TargetRequirement variant name for opponent-controlled creature targeting).

#### Finding 3: Bloodghast -- Missing "May" on Return

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/bloodghast.rs:51-55`
**Oracle**: "you **may** return this card from your graveyard to the battlefield"
**Issue**: The effect is `Effect::MoveZone { target: Source, to: Battlefield }` which unconditionally moves Bloodghast to the battlefield. The oracle says "you may return" -- this is optional. In a real game, a player might choose not to return Bloodghast (e.g., to avoid a board wipe trigger, or because they don't want to give an opponent a death trigger). With the current implementation, Bloodghast always returns. For bot play this is acceptable but produces wrong game state when the player would prefer not to return it.
**Fix**: If a `Condition::MayChoose` or optional effect wrapper exists, use it. Otherwise, document this as a known approximation with a TODO comment referencing the "may" keyword. At minimum, add a comment: `// TODO: Oracle says "you may return" — currently non-optional (bot always returns).`

#### Finding 4: Umezawa's Jitte -- Trigger Too Narrow

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/umezawas_jitte.rs:32`
**Oracle**: "Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte."
**Issue**: The trigger uses `WhenEquippedCreatureDealsCombatDamageToPlayer` but the oracle says "deals combat damage" (to any target, including creatures). This means the Jitte won't gain charge counters when the equipped creature deals combat damage to blocking/blocked creatures, which is a significant gameplay difference. The comment at line 28-30 correctly notes this but does not include a TODO tag for tracking.
**Fix**: Add `// TODO(PB-37):` prefix to the existing comment at line 28-29 (already partially there). This is a DSL gap requiring a `WhenEquippedCreatureDealsCombatDamage` trigger condition variant (no "ToPlayer" suffix). File as a known approximation.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 700.2 | Yes | Yes | test_modal_triggered_ability_structure, test_felidar_retreat_modal_structure |
| 700.2b | Yes | Yes | Modal mode selection in flush_pending_triggers; 6 structure tests |
| 603.3c | Yes | Partial | Mode removal when no mode choosable: not explicitly tested (bot always picks mode 0) |
| 602.2 | Yes | Yes | handle_activate_ability graveyard zone check; 3 graveyard ability tests |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| retreat_to_kazandu | Yes | 0 | Yes | Clean |
| retreat_to_coralhelm | Yes | 0 | Approx | "tap or untap" approximated as untap only |
| felidar_retreat | Yes | 0 | Yes | Clean |
| junji_the_midnight_sky | Yes | 0 | Approx | Non-Dragon filter not expressible; any creature card used |
| shambling_ghast | **No** | 0 | **No** | **Wrong trigger (ETB instead of dies)** + wrong target filter |
| tectonic_giant | Partial | 1 | Partial | Dual trigger condition gap; mode 1 Effect::Nothing |
| hullbreaker_horror | Partial | 1 | Partial | "Can't be countered" not implemented |
| glissa_sunslayer | Partial | 1 | Partial | Mode 2 counter removal placeholder |
| goblin_cratermaker | Yes | 0 | Approx | Colorless filter approximated as non-land |
| umezawas_jitte | Partial | 1 | Partial | Trigger misses creature combat damage |
| reassembling_skeleton | Yes | 0 | Yes | Clean |
| bloodghast | Partial | 2 | Partial | Missing "may", missing "can't block" |
| earthquake_dragon | Yes | 0 | Yes | Clean |
| cult_conscript | Partial | 1 | Approx | Activation condition deferred |

## Test Summary

11 tests total (6 modal_triggers.rs + 5 graveyard_abilities.rs). All are structure/def checks. Two tests exercise actual game state (graveyard activation + zone check). No integration tests for modal trigger resolution through the stack. No test for Bloodghast graveyard trigger actually firing.

| Test Gap | Severity | Notes |
|----------|----------|-------|
| No modal trigger resolution test | LOW | Structure tests verify def shape but not that modes_chosen flows through stack to resolution |
| No Bloodghast integration test | LOW | Structure test only; no test that landfall from graveyard actually fires the trigger |
| Shambling Ghast test validates wrong trigger | LOW | test_modal_etb_trigger_structure checks WhenEntersBattlefield which is the bug itself |

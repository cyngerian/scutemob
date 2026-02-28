# Ability Review: Connive

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.50 (keyword action)
**Files reviewed**:
- `crates/engine/src/effects/mod.rs` (lines 1487-1654)
- `crates/engine/src/rules/events.rs` (lines 756-768)
- `crates/engine/src/state/game_object.rs` (lines 186-191)
- `crates/engine/src/rules/abilities.rs` (lines 1447-1457)
- `crates/engine/src/state/hash.rs` (lines 1075-1076, 1963-1973, 2510-2514)
- `crates/engine/src/state/stubs.rs` (full file -- no Connive-specific changes needed)
- `crates/engine/src/cards/card_definition.rs` (lines 393-403)
- `crates/engine/tests/connive.rs` (full file, 797 lines, 6 tests)

## Verdict: needs-fix

The core Connive handler is well-implemented: draw N, discard N, count nonland discards,
place counters, emit Connived event. The Madness interaction is correctly replicated from
the `discard_cards` helper. Hash coverage is complete. However, there are two MEDIUM findings:
(1) a missing test for CR 701.50c (creature left battlefield), and (2) the
`collect_triggers_for_event` zone check at `abilities.rs:1518` prevents "whenever this
creature connives" triggers from firing when the creature has left the battlefield, which
contradicts CR 701.50b and multiple Scryfall rulings (Psychic Pickpocket, Obscura
Interceptor). There is also a MEDIUM gap in the card-definition system: no
`TriggerCondition` variant exists for "whenever this creature connives," blocking the
card definition phase for connive-trigger cards.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `abilities.rs:1518` | **SourceConnives trigger blocked when creature off battlefield.** The zone check in `collect_triggers_for_event` skips objects not on the battlefield, preventing "whenever this creature connives" from firing per CR 701.50b. **Fix:** Add a zone-check bypass for SourceConnives. |
| 2 | **MEDIUM** | `tests/connive.rs` | **Missing test for CR 701.50c (creature left battlefield before connive).** The plan specified this test and the file header documents the rule, but no test exercises it. **Fix:** Add test. |
| 3 | **MEDIUM** | `card_definition.rs` / `replay_harness.rs` | **No TriggerCondition::WhenConnives variant.** Card definitions cannot express "whenever this creature connives" triggers, blocking step 5 for connive-trigger cards. **Fix:** Add variant and enrichment mapping. |
| 4 | LOW | `tests/connive.rs:648-652` | **Misleading comments in ETB test.** Comments say second `pass_all` resolves ETB trigger, but the ETB fires inline during the first pass via `fire_when_enters_triggered_effects`. **Fix:** Correct comments. |
| 5 | LOW | `effects/mod.rs:1725-1730` | **EffectTarget::Source returns empty when creature is gone.** If the conniving creature left the battlefield and the effect uses `Source`, connive silently skips entirely (no draw/discard). Per CR 701.50c, the controller should still draw and discard. This is an architecture limitation (inline triggers vs stacked triggers) not unique to Connive. **Fix:** Deferred -- requires stacked-trigger architecture. |

### Finding Details

#### Finding 1: SourceConnives trigger blocked when creature off battlefield

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:1518`
**CR Rule**: 701.50b -- "A permanent 'connives' after the process described in rule 701.50a is complete, even if some or all of those actions were impossible."
**Scryfall Rulings**: Psychic Pickpocket (2022-04-29): "If a resolving spell or ability instructs a specific creature to connive but that creature has left the battlefield, the creature still connives. Abilities that trigger 'when [that creature] connives' ... will trigger."

**Issue**: `collect_triggers_for_event` (line 1518) checks `if obj.zone != ZoneId::Battlefield { continue; }`. When `GameEvent::Connived` is processed by `check_triggers` (line 1447), it passes `Some(*object_id)` as `only_object`. If the creature changed zones (e.g., was destroyed between casting and resolution), the object either has a new ObjectId (per CR 400.7) or is in a non-battlefield zone. Either way, the zone check at line 1518 causes the trigger to be skipped. This means "whenever this creature connives" triggers will not fire when the creature has left the battlefield, contradicting CR 701.50b and the Psychic Pickpocket/Obscura Interceptor rulings.

**Fix**: In the `GameEvent::Connived` match arm at `abilities.rs:1447`, do NOT call `collect_triggers_for_event` (which enforces the zone check). Instead, manually iterate over `state.objects` to find objects with `SourceConnives` triggers, and accept objects in any zone (or use a new `collect_triggers_for_event_any_zone` helper). Alternatively, add a `skip_zone_check: bool` parameter to `collect_triggers_for_event` and set it to `true` for SourceConnives. The trigger should fire on the object_id that was the conniving permanent, regardless of its current zone.

#### Finding 2: Missing test for CR 701.50c (creature left battlefield)

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/tests/connive.rs`
**CR Rule**: 701.50c -- "If a permanent changes zones before an effect causes it to connive, its last known information is used to determine which object connived and who controlled it."

**Issue**: The plan (ability-plan-connive.md, Step 5, test 5) specified a test called `test_connive_creature_left_battlefield_no_counter` that uses a sequence effect to destroy the creature, then connive it. The file-level comment at line 12 documents this coverage ("No counter placed when permanent left the battlefield before connive resolves (CR 701.50c)"), but no test function exercises this scenario. The 6 implemented tests cover CR 701.50a, 701.50b, 701.50e, ETB pattern, and trigger wiring, but skip 701.50c entirely.

**Fix**: Add a test `test_connive_creature_left_battlefield_no_counter` that:
1. Places a creature on the battlefield.
2. Destroys or exiles the creature (via a Sequence effect: DestroyObject then Connive, or by conniving a creature that was already removed).
3. Verifies: controller still drew and discarded, no +1/+1 counter was placed, Connived event fired with `counters_placed: 0`.

Note: Due to Finding 5 (EffectTarget::Source returns empty when creature is gone), this test may need to use a spell that targets the creature (DeclaredTarget) and a sequence that removes the creature mid-resolution. Alternatively, the test could demonstrate that the Connive handler correctly handles the case where `creature_id` refers to an object no longer on the battlefield by using a builder that places the creature in the graveyard from the start (simulating a zone change). The exact approach depends on whether the engine supports mid-effect zone changes in test scenarios.

#### Finding 3: No TriggerCondition::WhenConnives variant

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs` (TriggerCondition enum, around line 643) and `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs` (enrich_spec_from_def, around line 884)
**CR Rule**: 701.50b -- triggers that reference "whenever [this creature] connives"

**Issue**: The `TriggerCondition` enum in `card_definition.rs` does not have a variant for "whenever this creature connives" (e.g., `WhenConnives`). The `enrich_spec_from_def` function in `replay_harness.rs` does not convert any such variant to the runtime `TriggerEvent::SourceConnives`. This means card definitions (like Ledger Shredder, Psychic Pickpocket, etc.) cannot express connive triggers through the standard `AbilityDefinition::Triggered` system. The test at `connive.rs:694` works around this by using `ObjectSpec::with_triggered_ability(TriggeredAbilityDef { trigger_on: TriggerEvent::SourceConnives, ... })` directly, but actual card definitions in `cards/definitions.rs` cannot use this approach.

**Fix**:
1. Add `WhenConnives` to the `TriggerCondition` enum in `card_definition.rs`.
2. Add a hash discriminant for it in `hash.rs` (TriggerCondition match).
3. Add a conversion block in `enrich_spec_from_def` that maps `TriggerCondition::WhenConnives` to `TriggerEvent::SourceConnives`, following the pattern of `WheneverYouSurveil` -> `ControllerSurveils` (around line 870-884).

#### Finding 4: Misleading ETB test comments

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/connive.rs:648-652`

**Issue**: The comments at lines 648-652 state:
```
// First pass_all: spell resolves, creature enters, ETB trigger queues.
// Second pass_all: ETB trigger resolves (connive fires).
```
However, the ETB trigger fires **inline** during resolution (via `fire_when_enters_triggered_effects` in `replacement.rs:884`), not as a stacked trigger. The connive effect executes during the first `pass_all`, not the second. The test still passes because the second `pass_all` is harmless (empty stack advances the step), but the comments create a false mental model.

**Fix**: Update comments to:
```
// First pass_all: spell resolves, creature enters, ETB fires inline (connive resolves).
// Second pass_all: advances step (nothing on stack).
```

#### Finding 5: EffectTarget::Source returns empty when creature is gone (architecture limitation)

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs:1725-1730`
**CR Rule**: 701.50c -- "If a permanent changes zones before an effect causes it to connive, its last known information is used to determine which object connived and who controlled it."

**Issue**: When `EffectTarget::Source` is used in a Connive effect and the source creature has left the battlefield (new ObjectId per CR 400.7), the `resolve_effect_target_list` function returns an empty vector (line 1729: `vec![]`). This means the entire connive process is skipped -- no draw, no discard, no Connived event. Per CR 701.50c, the controller should still draw and discard (using last known information for controller identity). This is a pre-existing architecture limitation: the engine's inline trigger execution means the creature is always present during ETB connive, so this path is effectively unreachable for the most common connive pattern. However, it could surface for delayed connive effects or non-ETB connive abilities that target a creature by source reference.

**Fix**: Deferred. Requires the stacked-trigger architecture to be in place so that connive triggers go on the stack and can resolve after the creature has left. At that point, the Connive handler would need to use `ctx.controller` as fallback (which it already does at line 1505: `.unwrap_or(ctx.controller)`), but the `resolve_effect_target_list` call at line 1494 would need to return a synthetic target or the handler would need to bypass target resolution when the source is gone.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.50a (basic connive: draw 1, discard 1, counter if nonland) | Yes | Yes | test_connive_basic_nonland_discard_adds_counter, test_connive_land_discard_no_counter |
| 701.50b (permanent "connives" even if actions impossible) | Yes | Yes | test_connive_empty_library_still_connives; Connived event always emitted |
| 701.50c (permanent left battlefield: last known info, no counter) | Partial | **No** | Handler checks creature_on_battlefield at lines 1615-1619, but no test. Finding 2. |
| 701.50d (multiple simultaneous connives: APNAP order) | No | No | Plan notes this as LOW priority deferred edge case. Acceptable for initial implementation. |
| 701.50e (Connive N: draw N, discard N, count nonland) | Yes | Yes | test_connive_n_multiple_draws_and_discards |
| Madness interaction (CR 702.35a) | Yes | No | Handler replicates Madness check from discard_cards (lines 1543-1607). No Connive+Madness test. |
| SourceConnives trigger wiring | Yes | Yes | test_connive_self_trigger_fires_on_connive; Finding 1 (off-battlefield gap). |
| ETB connive pattern (Raffine's Informant) | Yes | Yes | test_connive_etb_trigger_on_creature |
| Hash: Effect::Connive | Yes | N/A | Discriminant 35 at hash.rs:2510 |
| Hash: GameEvent::Connived | Yes | N/A | Discriminant 79 at hash.rs:1963 |
| Hash: TriggerEvent::SourceConnives | Yes | N/A | Discriminant 15 at hash.rs:1075 |

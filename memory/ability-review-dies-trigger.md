# Ability Review: Dies Trigger

**Date**: 2026-02-25
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 603.6c, 603.10a, 700.4, 603.3a, 603.3b, 603.4, 603.2g
**Files reviewed**:
- `crates/engine/src/state/game_object.rs:100-127` (TriggerEvent::SelfDies)
- `crates/engine/src/state/hash.rs:883-900` (HashInto for TriggerEvent)
- `crates/engine/src/testing/replay_harness.rs:406-425` (WhenDies enrichment)
- `crates/engine/src/rules/abilities.rs:428-463` (CreatureDied dispatch)
- `crates/engine/tests/abilities.rs:994-1518` (8 dies trigger tests)
- `crates/engine/src/rules/sba.rs:285-338` (CreatureDied emission in SBAs)
- `crates/engine/src/rules/engine.rs:328-345` (SBA -> check_triggers -> flush flow)
- `crates/engine/src/state/mod.rs:218-270` (move_object_to_zone controller reset)
- `crates/engine/src/rules/resolution.rs:277-314` (triggered ability resolution)

## Verdict: needs-fix

The implementation is architecturally sound and handles the core dies trigger flow correctly for the common case. The enum variant, hash coverage, enrichment, dispatch, and test suite are all well-structured. However, there is one HIGH finding: token dies triggers are broken due to SBA loop ordering, and this directly contradicts CR 603.6c / CR 700.4. The gotchas-rules.md file explicitly states tokens DO trigger "when this dies," yet the implementation silently fails for this case. There is also one MEDIUM finding related to the stolen-creature controller edge case (CR 603.3a). Both issues are acknowledged in comments/plan but are not acceptable as-is for a P1 ability.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `rules/engine.rs:328-332` | **Token dies triggers fail due to SBA loop ordering.** CR 603.6c requires leaves-the-battlefield triggers to fire; CR 700.4 defines "dies." Token cleanup (CR 704.5d) in the second SBA pass removes the token before `check_triggers` runs, causing `state.objects.get(new_grave_id)` to return `None`. **Fix:** see details below. |
| 2 | MEDIUM | `rules/abilities.rs:453-456` | **Stolen-creature controller incorrect (CR 603.3a).** `move_object_to_zone` resets controller to owner. Dies trigger uses graveyard object's controller (= owner), not the controller at time of death. **Fix:** see details below. |
| 3 | LOW | `testing/replay_harness.rs:415,420` | **`intervening_if` silently dropped during enrichment.** The `..` pattern ignores `intervening_if` from `AbilityDefinition::Triggered`, hardcoding `None`. No current cards are affected. **Fix:** propagate `Condition` to `InterveningIf` when variants overlap. |
| 4 | LOW | `tests/abilities.rs:1343-1385` | **Test 6 is a near-duplicate of Test 1.** `test_dies_trigger_fires_on_destruction_effect` uses the identical mechanism (lethal damage SBA) as `test_dies_trigger_fires_on_lethal_damage_sba`. It claims to test "destruction via spell effect" but does not actually use `Effect::Destroy`. Low priority -- the test still validates correct behavior. |

### Finding Details

#### Finding 1: Token Dies Triggers Fail Due to SBA Loop Ordering

**Severity**: HIGH
**File**: `crates/engine/src/rules/engine.rs:328-332`
**CR Rule**: 603.6c -- "Leaves-the-battlefield abilities trigger when a permanent moves from the battlefield to another zone." CR 700.4 -- "The term dies means 'is put into a graveyard from the battlefield.'" CR 704.5d -- "If a token is in a zone other than the battlefield, it ceases to exist."
**Issue**: The `check_and_apply_sbas` function runs the SBA fixed-point loop to completion before returning events. In pass 1, a token with lethal damage is moved to the graveyard, emitting `CreatureDied { new_grave_id }`. In pass 2, CR 704.5d removes the token from the graveyard (it ceases to exist, removing it from `state.objects`). Only after the entire SBA loop completes does `check_triggers` run at line 332. By then, `state.objects.get(new_grave_id)` returns `None`, and the trigger is silently skipped.

Per the MTG rules, the trigger should fire. The gotchas-rules.md file at line 23-24 explicitly states: "When a token leaves the battlefield, it briefly exists in the new zone -- triggering 'when this dies' etc. -- then ceases to exist as an SBA." The implementation contradicts this documented invariant.

The test `test_dies_trigger_token_creature_architecture_note` documents this as a "KNOWN LIMITATION" but does not mark it as incorrect behavior. For a P1 ability, this is a correctness gap.

**Fix**: Move trigger checking inside the SBA loop. After each pass of `apply_sbas_once`, call `check_triggers` on the events from that pass (before the next SBA pass removes tokens). Specifically, in `check_and_apply_sbas` (sba.rs:50-59), after `all_events.extend(events)` but before the next loop iteration, call `check_triggers` on `events` (the current pass's events, not `all_events`) and push results into `state.pending_triggers`. This ensures that `CreatureDied` events from pass 1 are processed while the token still exists in `state.objects`, before pass 2 removes it.

Alternative (less invasive): Have `check_and_apply_sbas` return per-pass event batches instead of a flat list, and have the caller run `check_triggers` on each batch in order. This preserves the current separation of concerns.

If neither approach is feasible in this iteration, at minimum: (a) update `gotchas-rules.md` line 23-24 to note the engine does NOT currently implement this correctly, (b) add a `// TODO(HIGH)` comment in `engine.rs` at the SBA+triggers site, and (c) rename the test to clearly indicate it asserts incorrect behavior (e.g., `test_dies_trigger_token_KNOWN_BUG_does_not_fire`).

#### Finding 2: Stolen-Creature Controller Incorrect (CR 603.3a)

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:453-456`
**CR Rule**: 603.3a -- "A triggered ability is controlled by the player who controlled its source at the time it triggered, unless it's a delayed triggered ability."
**Issue**: When a creature dies, `move_object_to_zone` (state/mod.rs:260) resets `controller` to `owner`. The `CreatureDied` dispatch at abilities.rs:435 reads `obj.controller` from the graveyard object, which is now `owner`, not the pre-death controller. If Player A controlled Player B's creature (e.g., via Control Magic), the dies trigger would be attributed to Player B (the owner), not Player A (the controller at time of death). This violates CR 603.3a.

The `CreatureDied` event already carries `object_id` (the old battlefield ObjectId), but the old object has been removed from `state.objects` by `move_object_to_zone`, so it cannot be looked up.

**Fix**: Extend the `CreatureDied` event variant to include the pre-death controller: `GameEvent::CreatureDied { object_id: ObjectId, new_grave_id: ObjectId, controller: PlayerId }`. In `sba.rs`, `abilities.rs` (sacrifice), and `effects/mod.rs` (Destroy), capture the controller before `move_object_to_zone` and pass it through the event. In the dispatch at abilities.rs:435, use the event's `controller` field instead of `obj.controller`.

This requires updating `CreatureDied` in: events.rs (definition), hash.rs (hash arm), sba.rs:305 (emit site), sba.rs:331 (emit site), abilities.rs:158 (sacrifice emit), effects/mod.rs:345 (destroy emit), effects/mod.rs:385 (sacrifice effect emit), replacement.rs:960 (zone_change_events emit), and all test assertions that destructure `CreatureDied`.

#### Finding 3: `intervening_if` Silently Dropped During Enrichment

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:415,420`
**CR Rule**: 603.4 -- "A triggered ability may read 'When/Whenever/At [trigger event], if [condition], [effect].' When the trigger event occurs, the ability checks whether the stated condition is true."
**Issue**: The enrichment at line 412-424 uses `..` to ignore the `intervening_if` field from `AbilityDefinition::Triggered`, then hardcodes `intervening_if: None` in the runtime `TriggeredAbilityDef`. The card-definition type `Condition` and the runtime type `InterveningIf` are different enums with partial overlap (only `ControllerLifeAtLeast` exists in both). No existing card definition uses `WhenDies` with a non-None `intervening_if`, so this is not currently triggered.
**Fix**: Add a conversion function `condition_to_intervening_if(cond: &Condition) -> Option<InterveningIf>` that maps overlapping variants, and propagate it: `intervening_if: intervening_if.as_ref().and_then(condition_to_intervening_if)`. This ensures future card definitions with conditional dies triggers work correctly.

#### Finding 4: Test 6 Is a Near-Duplicate of Test 1

**Severity**: LOW
**File**: `crates/engine/tests/abilities.rs:1343-1385`
**CR Rule**: 700.4 -- test claims to cover "destruction via spell effect."
**Issue**: `test_dies_trigger_fires_on_destruction_effect` sets up a creature with `.with_damage(2)` and passes priority for SBAs -- identical to `test_dies_trigger_fires_on_lethal_damage_sba`. The test description says "destruction via spell effect causes creature to die" but does not cast a Destroy spell. To truly test the destruction-via-effect path, it should cast a spell with `Effect::Destroy` targeting the creature and verify `CreatureDied` is emitted after effect resolution.
**Fix**: Rewrite the test to actually cast a destruction spell (e.g., a "Destroy target creature" instant) and verify the dies trigger fires during/after the resolution SBA check, or rename it to clarify it is testing the same SBA path as test 1.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 700.4 (dies = bf-to-gy) | Yes | Yes | test_dies_trigger_fires_on_lethal_damage_sba, test_dies_trigger_fires_on_sacrifice |
| 603.6c (leaves-bf trigger) | Yes | Yes | Dispatch inline in check_triggers. test_dies_trigger_fires_on_lethal_damage_sba |
| 603.10a (look back in time) | Yes (via characteristics cloning) | Partial | Tested implicitly -- characteristics are preserved by move_object_to_zone. No test that adds a dies trigger via continuous effect and verifies it fires after the effect ends. |
| 603.3a (controller at trigger time) | Partial -- uses graveyard controller (= owner) | No | No stolen-creature test. Finding 2. |
| 603.3b (APNAP ordering) | Yes | Yes | test_dies_trigger_multiple_creatures_simultaneous_sba verifies p1 bottom, p2 top |
| 603.4 (intervening-if) | Yes (runtime check) | No | No dies trigger test exercises intervening-if. Covered by existing ETB intervening-if tests. |
| 603.2g (prevented/replaced no trigger) | Yes | Yes | test_dies_trigger_does_not_fire_when_exiled (replacement redirects to exile) |
| 704.5d (token cleanup) | Yes (token removed) | Yes | test_dies_trigger_token_creature_architecture_note -- but asserts incorrect behavior (Finding 1) |
| 704.5f (zero toughness SBA) | Yes | Yes | test_dies_trigger_fires_on_zero_toughness_sba |
| 704.5g (lethal damage SBA) | Yes | Yes | test_dies_trigger_fires_on_lethal_damage_sba |
| 400.7 (new object identity) | Yes | Implicit | new_grave_id used as source; old object_id retired |
| Sacrifice-as-cost | Yes | Yes | test_dies_trigger_fires_on_sacrifice |
| Effect::Destroy path | Yes (emits CreatureDied) | No | test_dies_trigger_fires_on_destruction_effect uses SBA, not Destroy effect. Finding 4. |
| Commander zone redirect | Yes (no CreatureDied emitted) | No | Documented in plan; no dedicated test |
| Dies trigger resolution (effect execution) | Yes | Yes | test_dies_trigger_resolves_draws_card verifies DrawCards effect |

## Notes

### Positive Observations

1. **Architecture is well-designed.** The decision to bypass `collect_triggers_for_event` (which requires battlefield zone) and write custom inline dispatch is correct and well-documented. The inline approach mirrors the Ward trigger pattern.

2. **Hash coverage is complete.** `TriggerEvent::SelfDies => 8u8` in hash.rs, no gaps.

3. **CR citations are thorough.** Every code section cites the applicable CR rules. Comments explain the "look back in time" mechanism clearly.

4. **Test suite is solid for the common case.** 8 tests covering SBA paths (lethal damage, zero toughness), negative case (exile), resolution (DrawCards), sacrifice-as-cost, simultaneous deaths (APNAP), and the token edge case (documented limitation).

5. **Enrichment handles the Solemn Simulacrum end-to-end path correctly.** The `WhenDies -> SelfDies` conversion in replay_harness.rs produces the correct `TriggeredAbilityDef` for the only existing card with this trigger.

### Pre-existing Issues (Not Attributable to This Implementation)

- **SBA redirect to non-graveyard zones emits `CreatureDied` with misleading `new_grave_id` field** (sba.rs:330-334). If a replacement redirects to a zone other than Exile or Command, the event says "died" with `new_grave_id` pointing to a non-graveyard object. Pre-existing in M4 SBA code.

- **`Condition` vs `InterveningIf` type mismatch** is a pre-existing design gap. The dies trigger enrichment exposes it but did not create it.

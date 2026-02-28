# Ability Plan: Evolve

**Generated**: 2026-02-28
**CR**: 702.100
**Priority**: P3
**Similar abilities studied**: Dethrone (triggered on attack, +1/+1 counter on source, `builder.rs:465-487`, `abilities.rs:1039-1069`), Exploit (inline trigger generation in `check_triggers`, custom `StackObjectKind`, `abilities.rs:835-892`, `resolution.rs:934-956`), Modular (dies trigger with custom `StackObjectKind::ModularTrigger` carrying extra data, `abilities.rs:1525-1593`, `resolution.rs:958-1000`), Undying/Persist (intervening-if pattern, `builder.rs:528-553`, `game_object.rs:193-203`)

## CR Rule Text

```
702.100. Evolve

702.100a Evolve is a triggered ability. "Evolve" means "Whenever a creature you control
enters, if that creature's power is greater than this creature's power and/or that
creature's toughness is greater than this creature's toughness, put a +1/+1 counter on
this creature."

702.100b A creature "evolves" when one or more +1/+1 counters are put on it as a result
of its evolve ability resolving.

702.100c A creature can't have a greater power or toughness than a noncreature permanent.

702.100d If a creature has multiple instances of evolve, each triggers separately.
```

**Intervening-if rule (CR 603.4)**:
```
603.4 A triggered ability may read "When/Whenever/At [trigger event], if [condition],
[effect]." When the trigger event occurs, the ability checks whether the stated condition
is true. The ability triggers only if it is; otherwise it does nothing. If the ability
triggers, it checks the stated condition again as it resolves. If the condition isn't
true at that time, the ability is removed from the stack and does nothing.
```

## Key Edge Cases

- **Intervening-if (CR 603.4)**: The P/T comparison is checked BOTH when the trigger would fire AND when it resolves. If the entering creature's P/T changed between trigger and resolution (e.g., via pump spells, -X/-X effects, or removal), the condition is re-evaluated. If neither stat is greater at resolution, the ability does nothing.
- **Last known information (ruling 2013-04-15)**: "If the creature that entered the battlefield leaves the battlefield before evolve tries to resolve, use its last known power and toughness to compare the stats." This means we should use `calculate_characteristics` at resolution time, falling back to last-known info if the entering creature left the battlefield.
- **OR condition (CR 702.100a)**: The check is power OR toughness (inclusive or). If the entering creature has greater power but not toughness, or greater toughness but not power, the trigger fires. Both stats being greater also fires.
- **The stat that satisfies the condition can change (ruling 2013-04-15)**: "When comparing the stats as the evolve ability resolves, it's possible that the stat that's greater changes from power to toughness or vice versa. If this happens, the ability will still resolve and you'll put a +1/+1 counter on the creature with evolve." E.g., a 1/3 triggers on toughness, gets +2/-2 before resolution, now its power (3) is greater instead -- ability still resolves.
- **ETB with counters counts (ruling 2013-04-15)**: "If a creature enters the battlefield with +1/+1 counters on it, consider those counters when determining if evolve will trigger." The layer system via `calculate_characteristics` handles this naturally.
- **Multiple creatures entering simultaneously (ruling 2013-04-15)**: "If multiple creatures enter the battlefield at the same time, evolve may trigger multiple times, although the stat comparison will take place each time one of those abilities tries to resolve." E.g., two 3/3s enter; first evolve trigger resolves and adds a counter (making the evolve creature 3/3); second trigger checks and sees 3/3 vs 3/3 (neither greater) and does nothing. The engine handles this naturally since each trigger resolves independently.
- **Noncreature permanents (CR 702.100c)**: "A creature can't have a greater power or toughness than a noncreature permanent." Noncreature permanents entering do not trigger evolve. The entering permanent must be a creature.
- **Multiple instances (CR 702.100d)**: Each instance of evolve triggers separately. If a creature has two instances, two triggers go on the stack. The first resolves and adds a counter; the second re-checks (now the evolve creature has +1/+1, so the comparison may fail).
- **Self-entering creature (ruling implicit)**: The evolve creature itself entering the battlefield does NOT trigger its own evolve (it's comparing itself to itself, which can never satisfy "greater than"). But technically, the creature is "a creature you control" that entered -- the intervening-if comparison just fails because a creature is never greater than itself.
- **Multiplayer**: Evolve triggers on "a creature **you control** enters" -- only creatures controlled by the evolve creature's controller trigger it. Other players' creatures entering do not trigger evolve.
- **Layer-aware P/T (CR 613)**: The P/T comparison must use `calculate_characteristics` to get the true P/T after continuous effects. A 1/1 creature under the effect of a +2/+2 anthem is effectively 3/3 for evolve purposes.

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- exists at `types.rs:L496` (`KeywordAbility::Evolve`), `hash.rs` (disc 60), `view_model.rs`
- [ ] Step 2: Rule enforcement (trigger wiring + resolution)
- [ ] Step 3: Builder wiring (keyword -> TriggeredAbilityDef)
- [ ] Step 4: Unit tests

## Implementation Steps

### Step 1: Enum Variant (DONE)

Already added: `KeywordAbility::Evolve` at `/home/airbaggie/scutemob/crates/engine/src/state/types.rs:496`, hash discriminant 60 at `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs:428-429`, view model at `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs:648`.

### Step 2: Add `StackObjectKind::EvolveTrigger` variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add a new variant to `StackObjectKind`:
```rust
/// CR 702.100a: Evolve trigger on the stack.
///
/// When a creature with evolve sees another creature its controller controls
/// enter the battlefield with greater power and/or toughness, this trigger
/// fires. The `entering_creature` field carries the ObjectId of the creature
/// that entered, needed for the resolution-time intervening-if re-check
/// (CR 603.4 — compare entering creature P/T vs source P/T at resolution).
///
/// If the entering creature left the battlefield before resolution, use
/// last-known information for the P/T comparison (ruling 2013-04-15).
EvolveTrigger {
    source_object: ObjectId,
    entering_creature: ObjectId,
},
```
**Pattern**: Follow `ModularTrigger` at `stack.rs` (carries `counter_count`); `ExploitTrigger` at `stack.rs` (simpler variant).
**Hash**: Add discriminant 12 to `HashInto for StackObjectKind` in `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` (after `ModularTrigger` discriminant 11).
**Match arms**: Add `EvolveTrigger` to ALL match arms in `resolution.rs` that handle `StackObjectKind`. Grep for `ModularTrigger` in resolution.rs to find all sites:
  - `resolve_top_of_stack` (resolution logic)
  - `counter_spell` (`| StackObjectKind::EvolveTrigger { .. }`)

### Step 3: Add `PendingTrigger::is_evolve_trigger` and `evolve_entering_creature`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add two fields to `PendingTrigger` (after `modular_counter_count`):
```rust
/// CR 702.100a: If true, this pending trigger is an Evolve trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::EvolveTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The
/// `evolve_entering_creature` carries the ObjectId of the creature that
/// entered the battlefield and triggered evolve.
#[serde(default)]
pub is_evolve_trigger: bool,
/// CR 702.100a: ObjectId of the creature that entered the battlefield.
///
/// Only meaningful when `is_evolve_trigger` is true. Used at resolution
/// time for the intervening-if re-check (P/T comparison, CR 603.4).
/// If this creature left the battlefield, use last-known information.
#[serde(default)]
pub evolve_entering_creature: Option<ObjectId>,
```
**Hash**: Update `HashInto for PendingTrigger` in `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`. Add the two new fields after the `modular_counter_count` hash entry.

### Step 4: Trigger Dispatch in `check_triggers`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In the `GameEvent::PermanentEnteredBattlefield { object_id, .. }` arm of `check_triggers` (around line 781), add evolve trigger generation AFTER the exploit block (around line 892). Follow the Exploit pattern: inline trigger generation, not `collect_triggers_for_event`.

```rust
// CR 702.100a: Evolve — "Whenever a creature you control enters, if that
// creature's power is greater than this creature's power and/or that
// creature's toughness is greater than this creature's toughness, put a
// +1/+1 counter on this creature."
//
// Check ALL creatures on the battlefield with evolve (not just the entering
// permanent). The entering permanent must be a creature. The evolve creature
// must be controlled by the same player as the entering creature.
//
// CR 702.100c: Noncreature permanents cannot trigger evolve.
// CR 702.100d: Multiple instances of evolve each trigger separately.
// CR 603.4: Intervening-if — P/T comparison checked at trigger time.
{
    // First, verify the entering permanent is a creature.
    // Use calculate_characteristics for layer-aware type check.
    let entering_is_creature = crate::rules::layers::calculate_characteristics(state, *object_id)
        .or_else(|| state.objects.get(object_id).map(|o| o.characteristics.clone()))
        .map(|chars| chars.card_types.contains(&CardType::Creature))
        .unwrap_or(false);

    if entering_is_creature {
        let entering_controller = state.objects.get(object_id).map(|o| o.controller);

        if let Some(controller) = entering_controller {
            // Get the entering creature's P/T (layer-aware).
            let entering_chars = crate::rules::layers::calculate_characteristics(state, *object_id)
                .or_else(|| state.objects.get(object_id).map(|o| o.characteristics.clone()));

            let (entering_power, entering_toughness) = entering_chars
                .as_ref()
                .map(|c| (c.power.unwrap_or(0), c.toughness.unwrap_or(0)))
                .unwrap_or((0, 0));

            // Find all creatures with evolve controlled by the same player.
            let evolve_sources: Vec<(ObjectId, usize)> = state
                .objects
                .values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.controller == controller
                        && obj.id != *object_id // Cannot evolve from itself
                        && obj.characteristics.keywords.contains(&KeywordAbility::Evolve)
                })
                .flat_map(|obj| {
                    // Count evolve instances from the card definition for multiple instances
                    // (CR 702.100d). OrdSet deduplicates, so check the card definition.
                    let evolve_count = obj
                        .card_id
                        .as_ref()
                        .and_then(|cid| state.card_registry.get(cid.clone()))
                        .map(|def| {
                            def.abilities
                                .iter()
                                .filter(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Evolve)))
                                .count()
                        })
                        .unwrap_or(1)
                        .max(1);

                    (0..evolve_count).map(move |_| (obj.id, 0_usize))
                })
                .collect();

            for (evolve_id, _) in evolve_sources {
                // CR 603.4: Intervening-if check at trigger time.
                // Get the evolve creature's P/T (layer-aware).
                let evolve_chars = crate::rules::layers::calculate_characteristics(state, evolve_id)
                    .or_else(|| state.objects.get(&evolve_id).map(|o| o.characteristics.clone()));

                let (evolve_power, evolve_toughness) = evolve_chars
                    .as_ref()
                    .map(|c| (c.power.unwrap_or(0), c.toughness.unwrap_or(0)))
                    .unwrap_or((0, 0));

                // CR 702.100a: entering power > evolve power OR entering toughness > evolve toughness
                if entering_power > evolve_power || entering_toughness > evolve_toughness {
                    let evolve_controller = state.objects.get(&evolve_id)
                        .map(|o| o.controller)
                        .unwrap_or(controller);

                    triggers.push(PendingTrigger {
                        source: evolve_id,
                        ability_index: 0, // unused for evolve triggers
                        controller: evolve_controller,
                        triggering_event: Some(TriggerEvent::AnyPermanentEntersBattlefield),
                        entering_object_id: Some(*object_id),
                        targeting_stack_id: None,
                        triggering_player: None,
                        exalted_attacker_id: None,
                        defending_player_id: None,
                        is_evoke_sacrifice: false,
                        is_madness_trigger: false,
                        madness_exiled_card: None,
                        madness_cost: None,
                        is_miracle_trigger: false,
                        miracle_revealed_card: None,
                        miracle_cost: None,
                        is_unearth_trigger: false,
                        is_exploit_trigger: false,
                        is_modular_trigger: false,
                        modular_counter_count: None,
                        is_evolve_trigger: true,
                        evolve_entering_creature: Some(*object_id),
                    });
                }
            }
        }
    }
}
```

**Important notes**:
1. The self-check (`obj.id != *object_id`) prevents a creature from triggering its own evolve on entry. While the intervening-if would fail anyway (a creature is not greater than itself), this optimization avoids unnecessary triggers on the stack.
2. The multiple-instances check (CR 702.100d) follows the Exploit pattern: count instances from the card definition since `OrdSet` deduplicates.
3. `calculate_characteristics` is used for layer-aware P/T (CR 613), matching the ruling about ETB-with-counters.

### Step 5: Flush `PendingTrigger` to `StackObjectKind::EvolveTrigger`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers`, add an `else if trigger.is_evolve_trigger` branch (after the `is_modular_trigger` branch, around line 1525). Follow the Exploit pattern for simplicity.

```rust
} else if trigger.is_evolve_trigger {
    // CR 702.100a: Evolve ETB trigger -- "Whenever a creature you control
    // enters, if that creature's P > this creature's P and/or that creature's
    // T > this creature's T, put a +1/+1 counter on this creature."
    StackObjectKind::EvolveTrigger {
        source_object: trigger.source,
        entering_creature: trigger.evolve_entering_creature.unwrap_or(trigger.source),
    }
}
```

**Note**: Unlike Modular (which uses `continue` for early exit with custom targets), Evolve has no special target requirements. The `trigger_targets` will be empty (default), and the counter is placed on the source. So Evolve can fall through to the standard `StackObject` push below (no `continue`).

### Step 6: Resolution Handler for `EvolveTrigger`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: In `resolve_top_of_stack`, add a new match arm for `StackObjectKind::EvolveTrigger` after the `ModularTrigger` arm. This is the most important part -- it implements the resolution-time intervening-if re-check (CR 603.4).

```rust
// CR 702.100a: Evolve trigger resolves -- re-check the intervening-if
// condition (CR 603.4) and place a +1/+1 counter on the source creature
// if the entering creature still has greater P and/or T.
StackObjectKind::EvolveTrigger {
    source_object,
    entering_creature,
} => {
    let controller = stack_obj.controller;

    // CR 603.4: Resolution-time intervening-if re-check.
    // Compare entering creature's P/T vs evolve creature's P/T.
    //
    // Ruling 2013-04-15: "If the creature that entered the battlefield
    // leaves the battlefield before evolve tries to resolve, use its
    // last known power and toughness to compare the stats."
    //
    // Use calculate_characteristics for layer-aware P/T; fall back to
    // raw characteristics for objects that left the battlefield.
    let entering_chars = crate::rules::layers::calculate_characteristics(state, entering_creature)
        .or_else(|| state.objects.get(&entering_creature).map(|o| o.characteristics.clone()));

    let evolve_chars = crate::rules::layers::calculate_characteristics(state, source_object)
        .or_else(|| state.objects.get(&source_object).map(|o| o.characteristics.clone()));

    let condition_holds = match (entering_chars, evolve_chars) {
        (Some(entering), Some(evolve)) => {
            let ep = entering.power.unwrap_or(0);
            let et = entering.toughness.unwrap_or(0);
            let sp = evolve.power.unwrap_or(0);
            let st = evolve.toughness.unwrap_or(0);
            ep > sp || et > st
        }
        _ => false, // One or both objects no longer exist — condition fails
    };

    if condition_holds {
        // CR 702.100a: Put a +1/+1 counter on the evolve creature.
        // The source must still be on the battlefield.
        if let Some(obj) = state.objects.get_mut(&source_object) {
            if obj.zone == ZoneId::Battlefield {
                let current = obj
                    .counters
                    .get(&CounterType::PlusOnePlusOne)
                    .copied()
                    .unwrap_or(0);
                obj.counters = obj.counters.update(CounterType::PlusOnePlusOne, current + 1);

                events.push(GameEvent::CounterAdded {
                    object_id: source_object,
                    counter_type: CounterType::PlusOnePlusOne,
                    new_count: current + 1,
                });
            }
        }
    }

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Pattern**: Follow `ModularTrigger` resolution at `resolution.rs:958-1000`.
**CR**: 702.100a (counter placement), 603.4 (intervening-if re-check at resolution), ruling 2013-04-15 (last known information for departed creature).

**Also add to `counter_spell` match**:
```rust
| StackObjectKind::EvolveTrigger { .. }
```
in the abilities arm of the `counter_spell` function (around line 1101-1111).

### Step 7: Update All `PendingTrigger` Construction Sites

**File**: Multiple files that construct `PendingTrigger` literals.
**Action**: Add `is_evolve_trigger: false, evolve_entering_creature: None,` to every existing `PendingTrigger` construction site. Grep for `is_modular_trigger:` to find all sites.

Known sites (from grep):
- `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs` (multiple: `collect_triggers_for_event`, inline triggers for evoke/exploit/modular, dies triggers, exalted, etc.)
- `/home/airbaggie/scutemob/crates/engine/src/rules/turn_actions.rs` (unearth trigger, saga trigger)
- `/home/airbaggie/scutemob/crates/engine/src/rules/miracle.rs` (miracle trigger)
- `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs` (the madness trigger in check_triggers for AuraFellOff)

Run `grep -rn "is_modular_trigger:" crates/engine/src/` to find all sites. Each site needs the two new fields appended.

### Step 8: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/evolve.rs` (new file)
**Tests to write**:

```rust
//! Evolve keyword ability tests (CR 702.100).
//!
//! Evolve is a triggered ability: "Whenever a creature you control enters,
//! if that creature's power is greater than this creature's power and/or
//! that creature's toughness is greater than this creature's toughness,
//! put a +1/+1 counter on this creature."
//!
//! Key rules verified:
//! - Trigger fires when entering creature has greater P or T (CR 702.100a).
//! - Trigger does NOT fire when entering creature has equal or lesser P and T (CR 702.100a).
//! - Intervening-if re-checked at resolution (CR 603.4).
//! - Multiple instances each trigger separately (CR 702.100d).
//! - Noncreature permanents do not trigger evolve (CR 702.100c).
//! - OR condition: greater power alone, greater toughness alone, or both (CR 702.100a).
//! - Multiplayer: only creatures controlled by the same player trigger evolve.
```

1. **`test_evolve_basic_greater_power`** (CR 702.100a):
   - Setup: P1 controls a 1/1 creature with Evolve on the battlefield.
   - Action: P1 casts a 3/3 creature (or place it on the battlefield).
   - Assert: Evolve triggers (1 trigger on stack). After resolution, the evolve creature has 1 +1/+1 counter. Effective P/T is 2/2 via `calculate_characteristics`.

2. **`test_evolve_basic_greater_toughness`** (CR 702.100a):
   - Setup: P1 controls a 2/2 creature with Evolve.
   - Action: A 1/4 creature enters under P1's control (greater toughness, not greater power).
   - Assert: Trigger fires. After resolution, 1 +1/+1 counter on evolve creature.

3. **`test_evolve_no_trigger_equal_stats`** (CR 702.100a negative):
   - Setup: P1 controls a 2/2 creature with Evolve.
   - Action: A 2/2 creature enters under P1's control.
   - Assert: No trigger fires. Stack is empty after the ETB. No counter added.

4. **`test_evolve_no_trigger_smaller_creature`** (CR 702.100a negative):
   - Setup: P1 controls a 3/3 creature with Evolve.
   - Action: A 1/1 creature enters under P1's control.
   - Assert: No trigger fires.

5. **`test_evolve_noncreature_does_not_trigger`** (CR 702.100c):
   - Setup: P1 controls a 1/1 creature with Evolve.
   - Action: A noncreature artifact enters under P1's control.
   - Assert: No evolve trigger fires.

6. **`test_evolve_opponents_creature_does_not_trigger`** (CR 702.100a):
   - Setup: P1 controls a 1/1 creature with Evolve. P2 is another player.
   - Action: A 5/5 creature enters under P2's control.
   - Assert: No trigger on P1's evolve creature (different controller).

7. **`test_evolve_intervening_if_fails_at_resolution`** (CR 603.4):
   - Setup: P1 controls a 1/1 creature with Evolve and a 3/3 creature.
   - Action: The 3/3 triggers evolve. Before resolution, pump the evolve creature to 4/4 (via continuous effect or counter). Now neither P nor T of the entering creature is greater.
   - Assert: Trigger resolves but no counter is added (intervening-if fails at resolution).

8. **`test_evolve_multiple_instances`** (CR 702.100d):
   - Setup: P1 controls a creature with two instances of Evolve keyword (via card definition with two `AbilityDefinition::Keyword(KeywordAbility::Evolve)` entries).
   - Action: A creature with greater P/T enters.
   - Assert: Two triggers on the stack. First resolves, adds 1 counter. Second resolves, re-checks (now the evolve creature is larger) -- may or may not add a counter depending on stats.

9. **`test_evolve_multiple_creatures_entering`** (ruling 2013-04-15):
   - Setup: P1 controls a 2/2 evolve creature. Two 3/3 creatures enter simultaneously (or sequentially in the same event batch).
   - Assert: Two triggers. First resolves: counter added (now 3/3). Second resolves: re-check (3/3 vs 3/3) -- no counter added.

10. **`test_evolve_multiplayer`** (multiplayer consideration):
    - Setup: 4 players. P1 controls a 1/1 evolve creature. P2 has a 5/5 creature enter.
    - Assert: No trigger (P2's creature, not P1's). Then P1 plays a 2/2: trigger fires.

**Pattern**: Follow tests for Dethrone in `/home/airbaggie/scutemob/crates/engine/tests/dethrone.rs` (helper functions, `process_command`, `calculate_characteristics` assertions).

**Test structure**: Use `GameStateBuilder` with creatures placed directly on the battlefield at `Step::PrecombatMain`. To simulate a creature entering, either:
- Place a creature in P1's hand and cast it (requires mana setup), or
- Use `process_command(state, Command::PassPriority { .. })` until a step where ETB fires, or
- Directly construct the trigger scenario using `ObjectSpec::creature(...).in_zone(ZoneId::Battlefield)` and verifying triggers via events.

**Recommended approach for evolve tests**: Place the evolve creature on the battlefield, then add a second creature to P1's hand. Cast the creature from hand. When it resolves and enters the battlefield, the evolve trigger fires. This tests the full pipeline.

**Simpler alternative**: Set up the game at `Step::PrecombatMain` with a creature spell on the stack (pre-cast). Pass priority to resolve it. The `PermanentEnteredBattlefield` event fires and triggers evolve.

### Step 9: Card Definition (later phase)

**Suggested card**: Experiment One
- Oracle text: "Evolve / Remove two +1/+1 counters from Experiment One: Regenerate Experiment One."
- Type: Creature -- Human Ooze
- Mana cost: {G}
- P/T: 1/1
- Simple enough for first card: one keyword ability + one activated ability.
- Use `card-definition-author` agent.

**Alternative card**: Cloudfin Raptor
- Oracle text: "Flying / Evolve"
- Type: Creature -- Bird Mutant
- Mana cost: {U}
- P/T: 0/1
- Even simpler: two keyword abilities, no activated abilities.
- Better for a clean test card since it has no extra mechanics.

### Step 10: Game Script (later phase)

**Suggested scenario**: "Cloudfin Raptor evolves when a larger creature enters"
- P1 has Cloudfin Raptor (0/1, Flying, Evolve) on the battlefield.
- P1 casts a 2/2 creature (e.g., Grizzly Bears or another defined creature).
- Evolve triggers. Both players pass. Counter added.
- Assert: Cloudfin Raptor has 1 +1/+1 counter, effective P/T is 1/2.

**Subsystem directory**: `test-data/generated-scripts/stack/` (triggered ability resolution)

## Interactions to Watch

- **Layer system**: The P/T comparison must use `calculate_characteristics` (layer-aware P/T). Anthem effects, +1/+1 counters from other sources, P/T setting effects all affect the comparison.
- **ETB replacements**: Creatures entering with +1/+1 counters (e.g., from Riot, modular, or "enters with N +1/+1 counters" effects) should have those counters considered in the evolve comparison. The replacement happens before the ETB event, so `calculate_characteristics` should see the counters.
- **Panharmonicon (TriggerDoublerFilter::ArtifactOrCreatureETB)**: Evolve triggers are generated inline (not via `AnyPermanentEntersBattlefield` collection), so they will NOT be doubled by Panharmonicon. This is actually **correct** -- Panharmonicon doubles ETB triggers of the entering permanent, not triggered abilities on other permanents watching for the ETB. Evolve is the latter (it's on the evolve creature, not the entering creature). However, if the engine implements "doubles all creature ETB triggers" (not Panharmonicon-specific), this may need revisiting.
- **Humility interaction**: If Humility is on the battlefield, it removes all abilities (including evolve) in Layer 6. The evolve keyword is stripped from `characteristics.keywords` by `calculate_characteristics`. The `check_triggers` code checks `obj.characteristics.keywords.contains(&KeywordAbility::Evolve)` on the raw characteristics (not layer-calculated). **This is a potential bug**: the trigger dispatch should use `calculate_characteristics` to check for evolve, not raw `obj.characteristics`. However, this is a pre-existing pattern issue across all keyword-driven triggers (e.g., Exploit also checks `obj.characteristics.keywords`). Mark as a known limitation / LOW issue.
- **Copy effects**: If a creature copies the evolve creature, the copy gains evolve. The trigger fires normally.
- **Flicker/blink**: If the entering creature is blinked in response to the evolve trigger, it leaves and returns as a new object (CR 400.7). The resolution-time check uses `entering_creature` ObjectId -- the old ID will NOT find the object on the battlefield. The ruling says "use last known power and toughness" -- but `calculate_characteristics` won't find a dead ObjectId. The resolution code should handle this: if `calculate_characteristics` returns `None` and the object isn't in `state.objects`, the condition fails (conservative, but arguably incorrect per the ruling). To fully implement last-known information for departed creatures, a LKI system would be needed -- this is a pre-existing engine limitation, not specific to evolve. Mark as LOW.

## Files to Modify (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/stack.rs` | Add `StackObjectKind::EvolveTrigger` variant |
| `crates/engine/src/state/stubs.rs` | Add `is_evolve_trigger`, `evolve_entering_creature` to `PendingTrigger` |
| `crates/engine/src/state/hash.rs` | Hash new `StackObjectKind` disc 12, new `PendingTrigger` fields |
| `crates/engine/src/rules/abilities.rs` | Evolve trigger dispatch in `check_triggers`, flush in `flush_pending_triggers` |
| `crates/engine/src/rules/resolution.rs` | `EvolveTrigger` resolution arm, `counter_spell` match arm |
| `crates/engine/tests/evolve.rs` | New test file with 8-10 tests |
| All files constructing `PendingTrigger` | Add `is_evolve_trigger: false, evolve_entering_creature: None` |

## Design Decision: Why StackObjectKind and Not InterveningIf

Evolve's intervening-if condition requires context that the existing `check_intervening_if` function does not receive: the entering creature's ObjectId. Two approaches were considered:

1. **Extend `InterveningIf` enum + `check_intervening_if` signature**: Would require passing `entering_object_id` through all call sites (resolution.rs reads `entering_object_id` from... where? The `StackObject` doesn't carry it).

2. **Custom `StackObjectKind::EvolveTrigger`**: Carries `entering_creature` directly. Resolution handles the P/T comparison inline. Follows the established pattern (Modular, Exploit).

Option 2 was chosen because:
- It matches the existing project pattern for complex triggered abilities needing extra context.
- It avoids modifying the `InterveningIf` / `check_intervening_if` interface, which would affect persist/undying.
- The resolution-time P/T comparison is self-contained and clear.
- No builder wiring needed (evolve triggers are generated inline in `check_triggers`, not via `TriggeredAbilityDef`).

**Builder wiring is NOT needed**: Unlike Persist/Undying/Dethrone which register a `TriggeredAbilityDef` on the object's `triggered_abilities` list, Evolve triggers are generated inline in `check_triggers` (like Exploit). The builder only needs to recognize the keyword for `characteristics.keywords` population, which is already handled by the generic keyword processing.

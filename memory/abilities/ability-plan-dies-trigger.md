# Ability Plan: Dies Trigger

**Generated**: 2026-02-25
**CR**: 603.6c, 603.10a, 700.4
**Priority**: P1
**Similar abilities studied**: ETB trigger dispatch (`SelfEntersBattlefield` in `rules/abilities.rs`), Ward trigger (`SelfBecomesTargetByOpponent`), Prowess trigger (`ControllerCastsNoncreatureSpell`)

## CR Rule Text

**CR 700.4**: The term "dies" means "is put into a graveyard from the battlefield."

**CR 603.6c**: Leaves-the-battlefield abilities trigger when a permanent moves from the battlefield to another zone, or when a phased-in permanent leaves the game because its owner leaves the game. These are written as, but aren't limited to, "When [this object] leaves the battlefield, . . ." or "Whenever [something] is put into a graveyard from the battlefield, . . . ." (See also rule 603.10.) An ability that attempts to do something to the card that left the battlefield checks for it only in the first zone that it went to.

**CR 603.10**: Normally, objects that exist immediately after an event are checked to see if the event matched any trigger conditions, and continuous effects that exist at that time are used to determine what the trigger conditions are and what the objects involved in the event look like. However, some triggered abilities are exceptions to this rule; the game "looks back in time" to determine if those abilities trigger, using the existence of those abilities and the appearance of objects immediately prior to the event.

**CR 603.10a**: Some zone-change triggers look back in time. These are leaves-the-battlefield abilities, abilities that trigger when a card leaves a graveyard, and abilities that trigger when an object that all players can see is put into a hand or library.

**CR 603.3a**: A triggered ability is controlled by the player who controlled its source at the time it triggered, unless it's a delayed triggered ability.

**CR 603.2**: Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers. The ability doesn't do anything at this point.

**CR 603.2g**: An ability triggers only if its trigger event actually occurs. An event that's prevented or replaced won't trigger anything.

## Key Edge Cases

1. **"Look back in time" (CR 603.10a)**: Dies triggers are leaves-the-battlefield triggers. The game must check the object's abilities as they existed BEFORE the zone change, not after. The creature is no longer on the battlefield when `check_triggers` runs, so we cannot use the standard `collect_triggers_for_event` which requires `obj.zone == ZoneId::Battlefield`.

2. **Object identity (CR 400.7)**: When the creature moves to the graveyard, it gets a new `ObjectId`. The `PendingTrigger.source` must reference the new graveyard `ObjectId` (since resolution looks up `state.objects.get(&source_object).characteristics.triggered_abilities[ability_index]`). Characteristics are preserved across zone changes by `move_object_to_zone`.

3. **Controller at trigger time (CR 603.3a)**: The triggered ability is controlled by the player who controlled the source at the time it triggered. For dies triggers, this is the controller immediately before death. The old object snapshot from `move_object_to_zone` preserves this, but the new graveyard object resets controller to owner. The `PendingTrigger.controller` must use the old controller (from before the zone change), not the owner.

4. **Tokens dying (CR 704.5d + corner case #24)**: Tokens briefly exist in the graveyard before being removed by SBA 704.5d. "When this dies" triggers DO fire for tokens because the trigger check happens when `CreatureDied` is emitted, which is before the token-cleanup SBA runs. The token's graveyard ObjectId is valid at trigger-check time.

5. **Commander dying with replacement (corner case #18, #28)**: If a commander dies and the SBA redirects it to the command zone, `GameEvent::CreatureDied` is NOT emitted (see `sba.rs:328` -- command zone redirect skips `CreatureDied`). This is correct: the creature didn't "die" (CR 700.4 requires graveyard). If a replacement effect like Rest in Peace exiles instead, `CreatureDied` is also not emitted -- `ObjectExiled` is emitted instead. Dies triggers correctly do not fire.

6. **Multiple creatures dying simultaneously in SBAs**: SBAs are batched. Multiple `CreatureDied` events can appear in the same event batch. Each one must independently check for dies triggers. These triggers all queue as `PendingTrigger` entries and are flushed together in APNAP order.

7. **Sacrifice-as-cost deaths**: When a creature is sacrificed as an activation cost (`abilities.rs:156`), `CreatureDied` is emitted. The events are passed to `check_triggers` at `engine.rs:108`. Dies triggers must fire here too.

8. **Destruction-by-effect deaths**: When `Effect::Destroy` resolves (`effects/mod.rs:345,385`), `CreatureDied` is emitted. The events are checked via `check_triggers` at `resolution.rs:372`. Dies triggers must fire here too.

9. **Indestructible creatures**: Indestructible creatures are not destroyed by SBAs or destruction effects. They never emit `CreatureDied`. Dies triggers correctly do not fire.

10. **Multiplayer**: Multiple players may each have creatures with "when this dies" triggers that die simultaneously (e.g., board wipe). All triggers queue and are placed on the stack in APNAP order per CR 603.3b.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant -- `TriggerCondition::WhenDies` exists at `cards/card_definition.rs:468`; `TriggerEvent::SelfDies` does NOT exist yet in `state/game_object.rs`
- [ ] 2. Rule enforcement -- `check_triggers` does not match `GameEvent::CreatureDied`
- [ ] 3. Trigger wiring -- `enrich_spec_from_def` does not convert `TriggerCondition::WhenDies` to a `TriggeredAbilityDef`
- [ ] 4. Unit tests
- [ ] 5. Card definition -- Solemn Simulacrum already has `WhenDies` at `definitions.rs:1422`
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Add `TriggerEvent::SelfDies` Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `SelfDies` variant to `TriggerEvent` enum at approximately line 122 (after `ControllerCastsNoncreatureSpell`).

```rust
/// CR 603.6c / CR 700.4: Triggers when this permanent is put into a
/// graveyard from the battlefield ("dies"). This is a leaves-the-battlefield
/// trigger that "looks back in time" (CR 603.10a).
SelfDies,
```

**Pattern**: Follow `SelfEntersBattlefield` and `SelfAttacks` naming convention.

**Hash**: Add to `state/hash.rs` `HashInto for TriggerEvent` impl. Currently the last discriminant used is for `ControllerCastsNoncreatureSpell`. Add a new discriminant value (e.g., `8u8`).

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Line**: After the `ControllerCastsNoncreatureSpell` arm in `HashInto for TriggerEvent` (find by grepping for `TriggerEvent`).
**Action**: Add `TriggerEvent::SelfDies => 8u8.hash_into(hasher),`

**Match arms**: Grep for `TriggerEvent::` match expressions elsewhere in the codebase. The key match is in `collect_triggers_for_event` where `trigger_def.trigger_on != event_type` is checked -- no explicit match arm needed there since it compares by equality. But verify no exhaustive match exists.

### Step 2: Enrichment -- Convert `TriggerCondition::WhenDies` to `TriggeredAbilityDef`

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Function**: `enrich_spec_from_def` (line 340)
**Action**: After the block that populates activated abilities (ends around line 406), add a new block that converts `AbilityDefinition::Triggered` with `TriggerCondition::WhenDies` into a `TriggeredAbilityDef` and pushes it to `spec.triggered_abilities`.

```rust
// CR 603.6c / CR 700.4: Convert "When ~ dies" card-definition triggers
// into runtime TriggeredAbilityDef entries so check_triggers can dispatch them.
for ability in &def.abilities {
    if let AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WhenDies,
        effect,
        intervening_if,
    } = ability
    {
        spec = spec.with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfDies,
            intervening_if: None, // WhenDies card defs don't use intervening-if currently
            description: format!("When ~ dies (CR 700.4)"),
            effect: Some(effect.clone()),
        });
    }
}
```

**Pattern**: Follow the Ward keyword enrichment at line 351 and Prowess at line 382. Same approach: iterate over `def.abilities`, match on the right `AbilityDefinition` variant, push a `TriggeredAbilityDef`.

**Also file**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Function**: The builder's object construction code (line 347) also processes `spec.triggered_abilities`. Since the enrichment above pushes to `spec.triggered_abilities`, the builder will automatically include it. No builder changes needed.

**Import check**: Ensure `TriggerCondition` and `AbilityDefinition` are imported in `replay_harness.rs`. Check existing imports at the top of the file.

### Step 3: Trigger Dispatch -- Handle `GameEvent::CreatureDied` in `check_triggers`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Function**: `check_triggers` (line 287)
**Action**: Add a new match arm for `GameEvent::CreatureDied { object_id, new_grave_id }` in the `match event` block (currently at line 291-429).

This is the most critical step. The standard `collect_triggers_for_event` CANNOT be used here because:
1. It requires `obj.zone == ZoneId::Battlefield` (line 465), but the creature is now in the graveyard.
2. The source object has a NEW `ObjectId` (`new_grave_id`), not the old battlefield one.

**CR 603.10a mandates "look back in time"** -- check the object's abilities as they existed before the zone change. However, since `move_object_to_zone` copies characteristics to the new object, we can look at the new graveyard object (`new_grave_id`) and read its `triggered_abilities`. The characteristics are identical.

**Implementation approach**: Write a new inline dispatch (NOT calling `collect_triggers_for_event`) that:
1. Looks up the NEW graveyard object by `new_grave_id`.
2. Iterates its `characteristics.triggered_abilities`.
3. For each entry with `trigger_on == TriggerEvent::SelfDies`, creates a `PendingTrigger`.
4. The `PendingTrigger.source` is set to `*new_grave_id` (the graveyard object -- resolution needs to find the ability definition on this object).
5. The `PendingTrigger.controller` is set to the graveyard object's OWNER (which `move_object_to_zone` resets controller to). However, per CR 603.3a, it should be the controller at the time of triggering (before death). Since `move_object_to_zone` resets controller to owner, and in most cases the controller IS the owner, this is usually correct. For the rare case where a player controls another player's creature (e.g., Control Magic), we would need the old controller. For now, using the new object's `controller` (which is reset to `owner`) is acceptable -- the old object snapshot is not available at `check_triggers` time. NOTE: If this edge case matters, a future enhancement could pass the old controller through the `CreatureDied` event.

```rust
GameEvent::CreatureDied { object_id, new_grave_id } => {
    // CR 603.6c / CR 603.10a / CR 700.4: "When ~ dies" triggers
    // look back in time. The creature is now in the graveyard, but
    // its characteristics (including triggered_abilities) are preserved
    // by move_object_to_zone. Check the graveyard object for SelfDies.
    if let Some(obj) = state.objects.get(new_grave_id) {
        for (idx, trigger_def) in obj.characteristics.triggered_abilities.iter().enumerate() {
            if trigger_def.trigger_on != TriggerEvent::SelfDies {
                continue;
            }

            // CR 603.4: Check intervening-if at trigger time.
            if let Some(ref cond) = trigger_def.intervening_if {
                if !check_intervening_if(state, cond, obj.controller) {
                    continue;
                }
            }

            triggers.push(PendingTrigger {
                source: *new_grave_id,
                ability_index: idx,
                // CR 603.3a: controller at the time of triggering.
                // move_object_to_zone resets controller to owner, which
                // is correct for the common case. Stolen-creature edge
                // case would need the old controller passed through the event.
                controller: obj.controller,
                triggering_event: Some(TriggerEvent::SelfDies),
                entering_object_id: None,
                targeting_stack_id: None,
            });
        }
    }
}
```

**Placement**: Add this arm before the `_ => {}` catch-all at line 428 in the match block.

**Verification**: After this change, the flow for a creature dying is:
1. SBA/effect/sacrifice moves creature to graveyard, emits `CreatureDied { object_id: old_bf_id, new_grave_id }`
2. `check_triggers` receives this event, looks up `new_grave_id` in `state.objects`, finds the graveyard object with preserved characteristics
3. Finds `SelfDies` triggered ability, creates `PendingTrigger { source: new_grave_id, ... }`
4. `flush_pending_triggers` puts it on the stack as `StackObjectKind::TriggeredAbility { source_object: new_grave_id, ability_index }`
5. Resolution looks up `state.objects.get(&new_grave_id).characteristics.triggered_abilities[ability_index]` -- finds the effect and executes it
6. For Solemn Simulacrum: executes `DrawCards { player: Controller, count: 1 }` -- controller draws a card

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/abilities.rs`

**Helper function to add** (near line 64, after `any_etb_trigger`):

```rust
/// Triggered ability: fires when the source permanent dies (CR 700.4).
fn dies_trigger(description: &str) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDies,
        intervening_if: None,
        description: description.to_string(),
        effect: None,
    }
}

/// Dies trigger with a DrawCards effect for functional testing.
fn dies_draw_trigger() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDies,
        intervening_if: None,
        description: "When ~ dies, draw a card. (CR 700.4)".to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
    }
}
```

**Tests to write** (append to the triggered abilities section of `abilities.rs`):

1. **`test_dies_trigger_fires_on_lethal_damage_sba`**
   - CR 700.4 + CR 704.5g -- creature with lethal damage dies via SBA, dies trigger fires
   - Setup: creature with `SelfDies` trigger on battlefield, mark lethal damage, pass priority to trigger SBAs
   - Assert: `AbilityTriggered` event appears with the correct source
   - Pattern: Follow `test_triggered_ability_self_etb_fires_on_enter` (line 498)

2. **`test_dies_trigger_fires_on_zero_toughness_sba`**
   - CR 700.4 + CR 704.5f -- creature with 0 toughness dies via SBA, dies trigger fires
   - Setup: creature with `SelfDies` trigger and 0 toughness, pass priority
   - Assert: `AbilityTriggered` event appears

3. **`test_dies_trigger_does_not_fire_when_exiled`**
   - CR 700.4 -- "dies" means battlefield-to-graveyard specifically. Exile does not count.
   - Setup: creature with `SelfDies` trigger on battlefield, exile it via effect
   - Assert: NO `AbilityTriggered` event for the dies trigger. `ObjectExiled` event present instead.

4. **`test_dies_trigger_resolves_draws_card`**
   - CR 603.6c -- dies trigger resolves and its effect executes
   - Setup: creature with `dies_draw_trigger()`, kill it, all players pass priority on the triggered ability
   - Assert: controller draws a card (hand size increases)

5. **`test_dies_trigger_fires_on_sacrifice`**
   - CR 700.4 -- sacrifice puts creature into graveyard from battlefield = dies
   - Setup: creature with `SelfDies` trigger and sacrifice-activated-ability, activate it
   - Assert: `AbilityTriggered` event for the dies trigger appears after sacrifice

6. **`test_dies_trigger_fires_on_destruction_effect`**
   - CR 700.4 -- destruction via spell (e.g., Lightning Bolt killing a creature) = dies
   - Setup: creature with `SelfDies` trigger takes lethal damage from a spell, resolves
   - Assert: after SBAs, dies trigger fires

7. **`test_dies_trigger_multiple_creatures_simultaneous_sba`**
   - CR 603.3b -- multiple creatures with dies triggers die simultaneously in SBAs, all triggers queue
   - Setup: two creatures from different controllers, both with dies triggers, both with lethal damage
   - Assert: both `AbilityTriggered` events appear, in APNAP order

8. **`test_dies_trigger_token_creature_fires`**
   - Corner case #24 (CR 704.5d) -- token dies, trigger fires before token ceases to exist
   - Setup: token creature with `SelfDies` trigger, mark lethal damage
   - Assert: `AbilityTriggered` event appears (token triggers fire even though token is cleaned up by SBA 704.5d afterward)

**File**: `/home/airbaggie/scutemob/crates/engine/tests/abilities.rs`
**Pattern**: Follow the existing triggered ability tests starting at line 496.

### Step 5: Card Definition Verification

**Suggested card**: Solemn Simulacrum (already defined at `definitions.rs:1393`)
**Status**: The card definition already has `AbilityDefinition::Triggered { trigger_condition: TriggerCondition::WhenDies, effect: DrawCards { player: Controller, count: 1 } }` at line 1421-1427.
**Action**: Verify the existing definition works end-to-end after Steps 1-4. No card authoring needed. The `enrich_spec_from_def` change in Step 2 will convert this card-def trigger into a runtime `TriggeredAbilityDef { trigger_on: SelfDies }`.
**Alternative validation card**: Blood Artist or Zulaport Cutthroat would test `WheneverCreatureDies` (a different trigger condition for global dies watching), but those are out of scope for this plan. Solemn Simulacrum is the right card for `WhenDies` (self-referential).

### Step 6: Game Script (later phase)

**Suggested scenario**: Solemn Simulacrum dies to lethal damage, triggers "draw a card" for its controller.
**Setup**: Solemn Simulacrum on battlefield, Lightning Bolt targeting it, resolve, SBAs kill it, dies trigger fires, resolves, controller draws a card.
**Subsystem directory**: `test-data/generated-scripts/stack/` (trigger resolution involves the stack)
**Alternative scenario**: Solemn Simulacrum sacrificed to Mind Stone's activated ability (sacrifice cost), dies trigger fires.

## Interactions to Watch

1. **Resolution lookup**: `StackObjectKind::TriggeredAbility { source_object, ability_index }` at resolution time (line 277 of `resolution.rs`) does `state.objects.get(&source_object)`. For dies triggers, `source_object` is the graveyard `ObjectId`. The object must still be in the graveyard at resolution time. If the object has been exiled from the graveyard before the trigger resolves (e.g., Bojuka Bog), the lookup returns `None` and the ability resolves without effect (line 304: "Source gone -- ability still resolves (no effect)"). This is correct per CR 603.6: "If the object is unable to be found in the zone it went to, the part of the ability attempting to do something to the object will fail to do anything."

2. **Token SBA ordering**: When a token creature dies, `CreatureDied` is emitted during the SBA pass. `check_triggers` fires on that event batch. Then the next SBA pass removes the token from the graveyard (CR 704.5d). But the `PendingTrigger` has already been queued with `source: new_grave_id`. When `flush_pending_triggers` runs, it creates a `StackObject` referencing that ObjectId. When the triggered ability resolves, `state.objects.get(&source)` returns `None` (token ceased to exist). This means the effect lookup fails silently. **This is a known limitation**: for token dies triggers to actually execute their effects, the triggered ability on the stack would need to capture the effect at flush time rather than looking it up at resolution time. For now, this is acceptable -- most token dies triggers are on non-token permanents that watch any creature dying (e.g., Blood Artist), not on the token itself. Solemn Simulacrum is not a token.

3. **Stolen creatures**: If Player A controls Player B's creature (via Control Magic) and it dies, CR 603.3a says the trigger is controlled by the controller at trigger time (Player A). However, `move_object_to_zone` resets controller to owner (Player B). The `PendingTrigger.controller` will incorrectly use Player B. This is an acknowledged edge case for a future fix. Most games do not involve this interaction.

4. **`PlayerTarget::Controller` resolution**: The `DrawCards { player: PlayerTarget::Controller }` effect uses `EffectContext.controller` to determine who draws. This is set from `stack_obj.controller` in resolution. For dies triggers, `stack_obj.controller` comes from `PendingTrigger.controller`, which is the graveyard object's controller (= owner after `move_object_to_zone`). In the normal case where owner == controller, this is correct.

5. **SBA batch → trigger batch ordering**: After SBAs emit `CreatureDied` events, `check_triggers` runs on those events and queues `PendingTrigger` entries. Then `flush_pending_triggers` puts them on the stack. All of this happens before any player receives priority, which is correct per CR 603.3.

## Architecture Notes

- This implementation does NOT modify `collect_triggers_for_event` because that function enforces `obj.zone == ZoneId::Battlefield`, which is fundamentally wrong for leaves-the-battlefield triggers. Instead, the dies trigger dispatch is done inline in `check_triggers`'s match arm for `CreatureDied`. This is the same pattern used for `PermanentTargeted` (Ward) which has custom inline logic rather than calling `collect_triggers_for_event`.

- The "look back in time" requirement (CR 603.10a) is satisfied because `move_object_to_zone` clones the entire `Characteristics` to the new graveyard object. The triggered abilities on the graveyard object are identical to what they were on the battlefield. No additional state preservation is needed.

- Future `WheneverCreatureDies` (global "whenever A creature dies") would need a DIFFERENT dispatch: iterate all battlefield permanents checking for that trigger event, similar to `AnyPermanentEntersBattlefield`. That is out of scope for this plan.

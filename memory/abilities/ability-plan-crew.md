# Ability Plan: Crew

**Generated**: 2026-02-27
**CR**: 702.122
**Priority**: P2
**Similar abilities studied**: Convoke (tapping multiple creatures as cost, `casting.rs:653`), Rogue's Passage (activated ability applying `ApplyContinuousEffect` with `UntilEndOfTurn`, `definitions.rs:419-435`), Equip (activated ability on artifact with target validation, `abilities.rs:136-178`, `tests/equip.rs`)

## CR Rule Text

702.122. Crew

> 702.122a Crew is an activated ability of Vehicle cards. "Crew N" means "Tap any number of other untapped creatures you control with total power N or greater: This permanent becomes an artifact creature until end of turn."
>
> 702.122b A creature "crews a Vehicle" when it's tapped to pay the cost to activate a Vehicle's crew ability.
>
> 702.122c If an effect states that a creature "can't crew Vehicles," that creature can't be tapped to pay the crew cost of a Vehicle.
>
> 702.122d Some Vehicles have abilities that trigger when they become crewed. "Whenever [this Vehicle] becomes crewed" means "Whenever a crew ability of [this Vehicle] resolves." If that ability has an intervening "if" clause that refers to information about the creatures that crewed it, it means only creatures that were tapped to pay the cost of the crew ability that caused it to trigger.

Related rules:

> 301.7 Some artifacts have the subtype "Vehicle." Most Vehicles have a crew ability which allows them to become artifact creatures. See rule 702.122, "Crew."
>
> 301.7a Each Vehicle has a printed power and toughness, but it has these characteristics only if it's also a creature. See rule 208.3.
>
> 301.7b If a Vehicle becomes a creature, it immediately has its printed power and toughness. Other effects, including the effect that makes it a creature, may modify these values or set them to different values.
>
> 208.3 A noncreature permanent has no power or toughness, even if it's a card with a power and toughness printed on it (such as a Vehicle). A noncreature object not on the battlefield has power or toughness only if it has a power and toughness printed on it.

## Key Edge Cases

From CR rulings on Smuggler's Copter, Heart of Kiran, Peacewalker Colossus:

1. **Summoning sickness does NOT prevent crewing** (ruling): "Any untapped creature you control can be tapped to pay a crew cost, even one that just came under your control." Tapping for crew cost is NOT a {T} activated ability cost -- it's a special cost (same logic as Convoke, CR 702.51 ruling).
2. **Summoning sickness DOES apply to the crewed Vehicle**: "Once a Vehicle becomes a creature, it behaves exactly like any other artifact creature. It can't attack unless you've controlled it continuously since your turn began." The Vehicle has its own `has_summoning_sickness` flag.
3. **Crewing an already-crewed Vehicle is legal but has no effect**: "You may activate a crew ability of a Vehicle even if it's already an artifact creature. Doing so has no effect on the Vehicle. It doesn't change its power and toughness."
4. **You may tap MORE creatures than necessary** (ruling): The total power must be >= N, but there is no upper bound.
5. **Becoming a creature is NOT entering the battlefield** (ruling): "When a Vehicle becomes a creature, that doesn't count as having a creature enter the battlefield." No ETB triggers fire.
6. **Vehicle is an artifact type, not a creature type** (ruling): "A Vehicle that's crewed won't normally have any creature type." The `AddCardTypes` adds `Creature` but does NOT add any creature subtypes.
7. **P/T from printed card when creature** (CR 301.7b): "If a Vehicle becomes a creature, it immediately has its printed power and toughness." The engine already stores `power`/`toughness` on the object; they become active when `Creature` type is added.
8. **Crew timing for combat** (ruling): Must crew during beginning of combat step to attack (need creature status by declare attackers), or during declare attackers step to block.
9. **Noncreature permanent has no P/T** (CR 208.3): A Vehicle that is not a creature has `power: Some(N)` and `toughness: Some(N)` on the `GameObject` (printed values), but they should not count for game purposes unless it's a creature. This is an existing engine behavior -- `calculate_characteristics` returns `chars.power` which includes the printed value. For SBAs (zero toughness check), the engine already skips `None` toughness. Since Vehicles have `Some(N)`, the SBA check would compare it. **However**, CR 208.3 says noncreature permanents have no P/T. This is a potential issue -- the engine currently doesn't suppress P/T for noncreature permanents.

   **Resolution for CR 208.3**: For the initial Crew implementation, we can safely set `power: Some(N)` and `toughness: Some(N)` on Vehicle card definitions. The SBA for zero toughness (CR 704.5f/g) only destroys creatures, and the SBA check already verifies `CardType::Creature` membership. So P/T on a noncreature artifact is harmless for SBA purposes. A full CR 208.3 implementation (suppressing P/T for noncreature permanents in `calculate_characteristics`) is a future enhancement.

10. **Multiplayer**: No special multiplayer considerations beyond standard activated ability rules (any player can activate during their priority, but crew taps creatures "you control" -- always the controller of the Vehicle).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- `KeywordAbility::Crew(u32)` does not exist yet
- [ ] Step 2: Rule enforcement -- no crew handling in engine
- [ ] Step 3: Trigger wiring -- N/A for basic crew (702.122d "whenever crewed" is deferred)
- [ ] Step 4: Unit tests -- no crew tests exist
- [ ] Step 5: Card definition -- no Vehicle cards defined
- [ ] Step 6: Game script -- no crew scripts
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and Command

**1a. KeywordAbility::Crew(u32)**

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Crew(u32)` variant to `KeywordAbility` enum after `Evoke` (line ~302).
**Pattern**: Follow `Annihilator(u32)` at line 269 -- same pattern of keyword with numeric parameter.

```rust
/// CR 702.122: Crew N -- "Tap any number of other untapped creatures you
/// control with total power N or greater: This permanent becomes an artifact
/// creature until end of turn."
///
/// Marker keyword for quick presence-checking. The crew cost (N) is the
/// minimum total power of creatures that must be tapped.
/// The actual activated ability is auto-generated in `enrich_spec_from_def`
/// from `AbilityDefinition::Keyword(KeywordAbility::Crew(n))`.
Crew(u32),
```

**1b. Hash discriminant**

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Crew(n)` with discriminant `40u8`.
**Pattern**: Follow `Annihilator(n)` at line 352:
```rust
KeywordAbility::Crew(n) => {
    40u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**1c. New Command: CrewVehicle**

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add a new `CrewVehicle` command variant. Crew requires specifying which creatures to tap, which is analogous to `convoke_creatures` on `CastSpell` but is a standalone command (not embedded in another command's fields). We model Crew as a dedicated command rather than using `ActivateAbility` because the multi-creature tap cost cannot be expressed by the existing `ActivationCost` struct (which only has `requires_tap: bool` for tapping the source).

```rust
/// Crew a Vehicle by tapping creatures (CR 702.122a).
///
/// Tap any number of untapped creatures you control with total power >= N
/// to activate the Vehicle's crew ability. The ability goes on the stack;
/// when it resolves, the Vehicle becomes an artifact creature until end of turn.
///
/// Unlike `ActivateAbility`, this command explicitly names the creatures tapped
/// as part of the crew cost (similar to how `CastSpell` names `convoke_creatures`).
CrewVehicle {
    player: PlayerId,
    /// The Vehicle to crew.
    vehicle: ObjectId,
    /// Creatures to tap as the crew cost. Must be untapped creatures you control
    /// with total power >= the Vehicle's crew N value. The Vehicle itself cannot
    /// be in this list ("other untapped creatures").
    crew_creatures: Vec<ObjectId>,
},
```

**1d. Command handler dispatch**

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add a match arm for `Command::CrewVehicle` that calls a new `abilities::handle_crew_vehicle` function (analogous to `handle_activate_ability`).

```rust
Command::CrewVehicle {
    player,
    vehicle,
    crew_creatures,
} => {
    validate_player_active(&state, player)?;
    loop_detection::reset_loop_detection(&mut state);
    let mut events = abilities::handle_crew_vehicle(
        &mut state,
        player,
        vehicle,
        crew_creatures,
    )?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    let trigger_events = abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```

**1e. Harness action type**

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"crew_vehicle"` action type to `translate_player_action()`. Parse `crew_creatures` from a JSON array of card names (resolved to ObjectIds on the battlefield).

```rust
"crew_vehicle" => {
    let vehicle_id = find_on_battlefield(state, player, card_name?)?;
    let crew_names: Vec<String> = action.get("crew_creatures")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let mut crew_ids = Vec::new();
    for name in &crew_names {
        crew_ids.push(find_on_battlefield(state, player, name)?);
    }
    Some(Command::CrewVehicle {
        player,
        vehicle: vehicle_id,
        crew_creatures: crew_ids,
    })
}
```

### Step 2: Rule Enforcement — handle_crew_vehicle

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add `pub fn handle_crew_vehicle(...)` function.
**CR**: 702.122a -- "Tap any number of other untapped creatures you control with total power N or greater: This permanent becomes an artifact creature until end of turn."
**Pattern**: Follows `apply_convoke_reduction` in `casting.rs:653` for creature validation/tapping, and `handle_activate_ability` for stack push + effect wiring.

The function must:

1. **Validate priority** (CR 602.2): Player must have priority.
2. **Check split second** (CR 702.61a): If a split second spell is on the stack, reject.
3. **Validate the Vehicle**:
   - Must be on the battlefield.
   - Must be controlled by the player.
   - Must have `KeywordAbility::Crew(n)` in its keywords (use `calculate_characteristics` for layer correctness).
   - Extract the crew cost N from the keyword.
4. **Validate crew creatures** (following Convoke pattern at `casting.rs:662-718`):
   - No duplicates (HashSet check).
   - Each creature must be on the battlefield, controlled by the player, untapped.
   - Each creature must be a creature type (use `calculate_characteristics`).
   - None may be the Vehicle itself ("other untapped creatures" -- CR 702.122a).
   - **No summoning sickness check** (ruling: any untapped creature, even one that just entered).
5. **Validate total power >= N**:
   - Sum `calculate_characteristics(state, id).power.unwrap_or(0)` for all crew creatures.
   - If total < N, return error.
6. **Pay the cost -- tap all crew creatures**:
   - Set `obj.status.tapped = true` for each.
   - Emit `PermanentTapped` event for each.
7. **Push crew ability onto the stack**:
   - Create a `StackObject` with `StackObjectKind::ActivatedAbility`.
   - The `embedded_effect` is `Some(Effect::ApplyContinuousEffect { ... })` with:
     - `layer: EffectLayer::TypeChange` (Layer 4)
     - `modification: LayerModification::AddCardTypes(OrdSet::from([CardType::Creature]))`
     - `filter: EffectFilter::Source` (resolved to `SingleObject(vehicle_id)` at execution)
     - `duration: EffectDuration::UntilEndOfTurn`
   - `source_object: vehicle_id`, `ability_index: 0` (synthetic).
   - Set `controller`, `targets: vec![]`, etc.
8. **Reset priority** (CR 116.3b): `players_passed = OrdSet::new()`, priority to active player.
9. **Emit `AbilityActivated` event** (if one exists) or `StackObjectPushed`.

**Important design decision**: Crew's "becomes an artifact creature" effect only adds the `Creature` card type (Layer 4). It does NOT need a Layer 7 effect because CR 301.7b says the Vehicle "immediately has its printed power and toughness" -- the engine already stores `power: Some(N)` and `toughness: Some(N)` on the object, and `calculate_characteristics` returns them. When `Creature` is added to `card_types`, SBAs and combat will recognize it as a creature with those P/T values. No Layer 7 effect is needed.

**Note on CR 208.3 (noncreature P/T suppression)**: The engine does NOT currently suppress P/T for noncreature permanents in `calculate_characteristics`. This means a Vehicle's P/T is visible even when it's not a creature. This is technically incorrect per CR 208.3 but is harmless for the initial implementation because:
- SBA 704.5f (zero toughness) checks for `Creature` type membership before checking toughness.
- Combat only involves creatures.
- No existing engine code reads P/T of noncreature permanents in a rules-relevant way.

A full CR 208.3 implementation would add a post-layer check in `calculate_characteristics` that sets `chars.power = None; chars.toughness = None` if `!chars.card_types.contains(&CardType::Creature)`. This is deferred as a LOW issue.

### Step 3: Trigger Wiring

**Not applicable for initial implementation.** CR 702.122d ("Whenever this Vehicle becomes crewed") is a trigger condition on specific Vehicle cards (e.g., Shorikai). It is NOT a general trigger that fires for all crew activations. The initial implementation does not include any cards with "whenever crewed" triggers.

If needed later, add a `TriggerEvent::SelfBecomeCrewed` and emit a corresponding `GameEvent::VehicleCrewed` when the crew ability resolves. This can be wired in `resolution.rs` at the point where the `ApplyContinuousEffect` for crew resolves.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/crew.rs`
**Pattern**: Follow `tests/equip.rs` (activated ability tests) and `tests/convoke.rs` (multi-creature tapping tests).

**Tests to write**:

1. **`test_crew_basic_vehicle_becomes_creature`** (CR 702.122a)
   - Set up a Vehicle (Artifact - Vehicle, no Creature type, P/T 3/3, Crew 1) and a 1/1 creature.
   - Issue `CrewVehicle` command with the creature.
   - Pass priority to resolve.
   - Assert: Vehicle now has `Creature` in its card types (via `calculate_characteristics`).
   - Assert: Vehicle has power 3 and toughness 3.
   - Assert: Crew creature is tapped.

2. **`test_crew_insufficient_power_rejected`** (CR 702.122a)
   - Vehicle with Crew 3, creature with power 2.
   - Issue `CrewVehicle` -- expect error (total power 2 < 3).

3. **`test_crew_multiple_creatures`** (CR 702.122a)
   - Vehicle with Crew 3, three creatures with power 1 each.
   - Issue `CrewVehicle` with all three -- succeeds (total power 3 >= 3).
   - Assert all three creatures are tapped.

4. **`test_crew_excess_creatures_allowed`** (ruling)
   - Vehicle with Crew 1, two creatures with power 3 each.
   - Issue `CrewVehicle` with both -- succeeds (total power 6 >> 1).

5. **`test_crew_vehicle_cannot_crew_itself`** (CR 702.122a: "other")
   - A Vehicle that is already a creature tries to crew itself.
   - Expect error: "vehicle cannot crew itself" or similar.

6. **`test_crew_summoning_sick_creature_can_crew`** (ruling)
   - Creature with `has_summoning_sickness = true` (no haste).
   - Issue `CrewVehicle` -- succeeds. Summoning sickness only prevents {T} activated abilities, not crew cost tapping.

7. **`test_crew_tapped_creature_rejected`** (CR 702.122a: "untapped")
   - Creature that is already tapped.
   - Issue `CrewVehicle` -- expect error.

8. **`test_crew_not_a_creature_rejected`**
   - Try to use an artifact (non-creature) as a crew creature.
   - Expect error: "not a creature."

9. **`test_crew_already_crewed_vehicle_is_legal`** (ruling)
   - Vehicle already animated (has Creature type from a previous crew this turn).
   - Crew it again with a different creature -- succeeds, no functional change.

10. **`test_crew_effect_expires_at_end_of_turn`** (CR 702.122a: "until end of turn")
    - Crew a Vehicle, pass through to cleanup step.
    - Assert: Vehicle is no longer a creature after end of turn.
    - Requires 2+ players to avoid game-over with 1 player (known gotcha).

11. **`test_crew_vehicle_no_creature_type_has_p_t`** (CR 301.7b)
    - Crew a Vehicle -- it becomes a creature with its printed P/T.
    - Verify `calculate_characteristics` returns the correct power and toughness.

12. **`test_crew_no_etb_trigger`** (ruling)
    - Set up a permanent with "Whenever a creature enters the battlefield" trigger.
    - Crew a Vehicle -- the trigger should NOT fire (the Vehicle was already on the battlefield).

13. **`test_crew_duplicate_creature_rejected`**
    - Pass the same creature ObjectId twice in `crew_creatures`.
    - Expect error: "duplicate creature."

14. **`test_crew_opponent_creature_rejected`** (CR 702.122a: "you control")
    - Try to tap an opponent's creature for crew cost.
    - Expect error.

15. **`test_crew_requires_priority`** (CR 602.2)
    - Try to crew when not the priority holder.
    - Expect error.

### Step 5: Card Definition

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs`
**Suggested card**: **Smuggler's Copter** (simple: Crew 1, Flying, plus a loot trigger)

```rust
CardDefinition {
    card_id: cid("smugglers-copter"),
    name: "Smuggler's Copter".to_string(),
    mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
    types: types_sub(&[CardType::Artifact], &["Vehicle"]),
    oracle_text: "Flying\nWhenever Smuggler's Copter attacks or blocks, you may draw a card. If you do, discard a card.\nCrew 1".to_string(),
    power: Some(3),
    toughness: Some(3),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Flying),
        // Loot trigger deferred -- needs "when attacks or blocks" condition
        AbilityDefinition::Keyword(KeywordAbility::Crew(1)),
    ],
}
```

**Important Vehicle card modeling notes**:
- `types: types_sub(&[CardType::Artifact], &["Vehicle"])` -- Artifact with Vehicle subtype, no Creature type.
- `power: Some(3), toughness: Some(3)` -- printed P/T stored even though it's not a creature. These become active when crew adds Creature type.
- `AbilityDefinition::Keyword(KeywordAbility::Crew(1))` -- the keyword marker. The actual activated ability is auto-generated by `enrich_spec_from_def`.

### Step 6: Game Script

**Suggested scenario**: "Smuggler's Copter is crewed by a 1/1 creature, becomes a 3/3 artifact creature, attacks."
**Subsystem directory**: `test-data/generated-scripts/combat/`

### Step 7: enrich_spec_from_def Crew Translation

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: In `enrich_spec_from_def`, add a block that translates `KeywordAbility::Crew(n)` into an `ActivatedAbility` on the object (similar to how Ward generates a triggered ability in `builder.rs`).

The crew ability does NOT use the standard `ActivatedAbility` / `ActivationCost` system because `ActivationCost` cannot express "tap N total power of other creatures." Instead, the `CrewVehicle` command handles the entire cost+effect. But the keyword still needs to be present on the object for the command handler to find and validate.

**Two options**:

**Option A (Recommended)**: Do NOT generate an `ActivatedAbility` from `Crew(n)`. The `CrewVehicle` command reads the keyword directly from `characteristics.keywords`. No `activated_abilities` entry needed. This avoids the problem of `ActivationCost` being unable to represent the crew cost. The `Crew(n)` keyword on the object is sufficient for the command handler.

**Option B**: Generate a placeholder `ActivatedAbility` with a `description` field and `effect: None` so that UI/display can show "Crew 3" as an activated ability. The `CrewVehicle` command ignores this entry and reads the keyword instead. This adds complexity for no mechanical benefit in the initial implementation.

**Decision**: Option A. The `Crew(n)` keyword in `characteristics.keywords` is the source of truth. The `CrewVehicle` command handler looks for it there. No `ActivatedAbility` entry is generated.

## Implementation Architecture Summary

The Crew implementation follows these patterns:

| Aspect | Pattern source | Key difference |
|--------|---------------|----------------|
| Multi-creature tap as cost | Convoke (`casting.rs:653`) | Power threshold instead of mana reduction |
| Dedicated command | `CastSpell` with `convoke_creatures` field | Standalone command, not embedded in CastSpell |
| Stack-based activated ability | `ActivateAbility` handler (`abilities.rs:48`) | Custom cost validation bypasses `ActivationCost` |
| UntilEndOfTurn type change | Rogue's Passage (`definitions.rs:424`) | `AddCardTypes({Creature})` in Layer 4, not `AddKeyword` in Layer 6 |
| Keyword with parameter | `Annihilator(u32)` (`types.rs:269`) | Same `u32` parameter pattern |

## Interactions to Watch

1. **Crew + Humility**: Humility removes all abilities (Layer 6). If Humility removes the Crew keyword, the Vehicle can no longer be crewed. If the Vehicle was already crewed this turn, the `UntilEndOfTurn` continuous effect in Layer 4 (adding Creature type) persists because it's a separate effect -- Humility's Layer 6 ability removal doesn't remove Layer 4 continuous effects. The Vehicle remains a creature but loses all abilities (including Crew) and becomes 1/1.

2. **Crew + Blood Moon**: Blood Moon affects nonbasic lands in Layer 4. Vehicles are artifacts, not lands, so Blood Moon does not interact with Crew.

3. **Crew + SBA zero toughness**: If a continuous effect sets the Vehicle's toughness to 0 while it's a creature (e.g., via -X/-X effect), it will be destroyed by SBA 704.5f. When it's not a creature, the SBA does not apply (the SBA check requires Creature type).

4. **Crew + Blink effects**: If the Vehicle is blinked (exiled and returned), it returns as a new object (CR 400.7) without the `UntilEndOfTurn` creature type effect. It returns as a non-creature artifact Vehicle.

5. **Crew + Copy effects**: If a permanent becomes a copy of a Vehicle, the copy will not be a creature even if the original was crewed (ruling). Copy effects in Layer 1 set copiable values from the *printed* card, not from continuous effects.

6. **Crew + Protection from creatures**: If a crew creature has "protection from artifacts," this is irrelevant -- tapping for crew cost is not targeting or interacting with the Vehicle in a way that protection prevents.

7. **Multiplayer priority**: The `CrewVehicle` command can be issued by any player with priority who controls both the Vehicle and the crew creatures. The standard priority validation in `engine.rs` handles this.

## Files Modified (Summary)

1. `crates/engine/src/state/types.rs` -- Add `KeywordAbility::Crew(u32)`
2. `crates/engine/src/state/hash.rs` -- Add hash discriminant 40
3. `crates/engine/src/rules/command.rs` -- Add `Command::CrewVehicle`
4. `crates/engine/src/rules/engine.rs` -- Add command dispatch for `CrewVehicle`
5. `crates/engine/src/rules/abilities.rs` -- Add `pub fn handle_crew_vehicle(...)`
6. `crates/engine/src/testing/replay_harness.rs` -- Add `"crew_vehicle"` action type
7. `crates/engine/tests/crew.rs` -- New test file (15 tests)
8. `crates/engine/src/cards/definitions.rs` -- Add Smuggler's Copter definition
9. `crates/engine/src/lib.rs` -- Re-export `Command::CrewVehicle` if needed

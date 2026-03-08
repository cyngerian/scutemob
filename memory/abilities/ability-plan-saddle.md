# Ability Plan: Saddle

**Generated**: 2026-03-07
**CR**: 702.171
**Priority**: P4
**Similar abilities studied**: Crew (CR 702.122) — `rules/abilities.rs:6040-6239`, `rules/command.rs:527-546`, `rules/engine.rs:359-376`, `tests/crew.rs`

## CR Rule Text

702.171. Saddle

702.171a Saddle is an activated ability. "Saddle N" means "Tap any number of other untapped creatures you control with total power N or greater: This permanent becomes saddled until end of turn. Activate only as a sorcery."

702.171b Saddled is a designation that has no rules meaning other than to act as a marker that spells and abilities can identify. Only permanents can be or become saddled. Once a permanent has become saddled, it stays saddled until the end of the turn or it leaves the battlefield. Being saddled is not a part of the permanent's copiable values.

702.171c A creature "saddles" a permanent as it's tapped to pay the cost to activate a permanent's saddle ability.

## Key Edge Cases

- **Sorcery-speed only (CR 702.171a)**: Unlike Crew, Saddle has "Activate only as a sorcery." Must enforce: active player's main phase, empty stack. Crew does NOT have this restriction.
- **Saddled is a designation, NOT a continuous effect (CR 702.171b)**: Unlike Crew (which registers a Layer 4 continuous effect to add Creature type), Saddle just sets a boolean marker. No layer system involvement.
- **Not copiable (CR 702.171b)**: Copies of saddled Mounts are NOT saddled. Use a boolean flag on GameObject (like `is_renowned`), not copiable characteristics.
- **Cleared at end of turn OR on leaving battlefield (CR 702.171b)**: "stays saddled until the end of the turn or it leaves the battlefield." Boolean cleared in cleanup AND on zone changes.
- **Saddling an already-saddled Mount is legal (ruling 2024-04-12)**: "You may activate a permanent's saddle ability even if that permanent is already saddled."
- **Summoning sickness does NOT prevent saddling (same ruling as Crew)**: Tapping for saddle cost is not a {T} activated ability.
- **Mounts can attack/block without being saddled (ruling 2024-04-12)**: "Creatures with saddle can attack or block as normal even if they aren't saddled."
- **"Attacks while saddled" triggers (ruling 2024-04-12)**: "An ability that triggers when a creature 'attacks while saddled' will trigger only if that creature was saddled when it was declared as an attacker." This is a per-card trigger condition, not a core Saddle mechanic.
- **Multiplayer**: No special multiplayer considerations beyond normal activated ability rules.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Saddle(u32)` variant. Parameterized with N (the saddle cost). Follow `KeywordAbility::Crew(u32)` pattern at line ~397.
**Discriminant**: Use KW 140 (next available after Gift at 139).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Saddle(n)` in the `HashInto` impl. Follow `Crew(n)` at line ~396-399:
```
KeywordAbility::Saddle(n) => {
    140u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `pub is_saddled: bool` field to `GameObject`. Place near `is_renowned` (~line 480). Include `#[serde(default)]` attribute. Add doc comment citing CR 702.171b.

**File**: `crates/engine/src/state/mod.rs`
**Action**: Initialize `is_saddled: false` in BOTH `move_object_to_zone` sites (two new-object construction blocks, ~lines 344 and 510). CR 702.171b: "stays saddled until ... it leaves the battlefield."

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: Clear `is_saddled = false` for all battlefield objects in `cleanup_step()`, after `expire_end_of_turn_effects` (~line 1296). CR 702.171b: "stays saddled until the end of the turn."

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add arm for `KeywordAbility::Saddle(n)` in the keyword display match (~line 767, near Crew). Format: `format!("Saddle {n}")`.

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: No changes needed -- `KeywordAbility` is already exported.

**Match arm sweep**: Grep for all exhaustive `KeywordAbility` match expressions across the workspace and add `Saddle(n)` arm. Known locations:
- `state/hash.rs` (HashInto)
- `tools/replay-viewer/src/view_model.rs` (keyword display)
- Any other exhaustive matches found by grep

### Step 2: Command & Handler (Rule Enforcement)

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `Command::SaddleMount` variant. Follow `CrewVehicle` at line ~538:
```rust
/// Saddle a Mount by tapping creatures (CR 702.171a).
///
/// Tap any number of untapped creatures you control with total power >= N
/// to activate the Mount's saddle ability. The ability goes on the stack;
/// when it resolves, the Mount becomes saddled until end of turn.
/// Activate only as a sorcery.
SaddleMount {
    player: PlayerId,
    /// The Mount to saddle.
    mount: ObjectId,
    /// Creatures to tap as the saddle cost.
    saddle_creatures: Vec<ObjectId>,
},
```

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add handler arm for `Command::SaddleMount` in `process_command`. Follow `CrewVehicle` at line ~359-376:
```rust
Command::SaddleMount { player, mount, saddle_creatures } => {
    validate_player_active(&state, player)?;
    loop_detection::reset_loop_detection(&mut state);
    let mut events = abilities::handle_saddle_mount(&mut state, player, mount, saddle_creatures)?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    abilities::flush_pending_triggers(&mut state, &mut events);
    state.turn.players_passed = im::OrdSet::new();
    all_events.extend(events);
}
```

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add `handle_saddle_mount` function. Model closely on `handle_crew_vehicle` (line 6040) with these differences:

1. **Sorcery-speed check (CR 702.171a)**: After priority check and split-second check, enforce sorcery timing:
   - `state.turn.active_player != player` -> error "sorcery-speed"
   - Not a main phase -> error "not main phase"
   - `!state.stack_objects.is_empty()` -> error "stack not empty"

2. **Keyword lookup**: Use `KeywordAbility::Saddle(n)` instead of `Crew(n)`.

3. **Embedded effect**: Instead of `ApplyContinuousEffect` that adds Creature type, the embedded effect should set `is_saddled = true` on the source. Use `Effect::SetSaddled` (a new Effect variant) OR resolve it inline in `resolution.rs` by detecting the stack object kind.

   **Recommended approach**: Do NOT create a new Effect variant. Instead, use a new `StackObjectKind::SaddleAbility { source_object: ObjectId }` (disc 55). When resolution encounters this SOK, it sets `is_saddled = true` on `source_object`. This is cleaner than an Effect variant because:
   - Saddle's resolution is a simple flag set, not a persistent continuous effect
   - It mirrors the pattern used for other simple-resolution SOKs (GraftTrigger, etc.)

4. **All other validation is identical to Crew**: battlefield check, controller check, "other" self-exclusion, untapped check, creature type check, total power >= N, uniqueness, summoning sickness allowed.

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution arm for `StackObjectKind::SaddleAbility { source_object }`. When resolving:
```rust
StackObjectKind::SaddleAbility { source_object } => {
    // CR 702.171a: The Mount becomes saddled until end of turn.
    if let Some(obj) = state.objects.get_mut(&source_object) {
        if obj.zone == ZoneId::Battlefield {
            obj.is_saddled = true;
        }
    }
}
```
The battlefield check guards against the Mount having left the battlefield between activation and resolution (fizzle-like behavior).

**File**: `crates/engine/src/rules/stack.rs` (if `StackObjectKind` is defined here)
**Action**: Add `SaddleAbility { source_object: ObjectId }` variant to `StackObjectKind` enum with disc 55.

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add arm for `StackObjectKind::SaddleAbility { .. }` in the `stack_kind_info` match.

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add arm for `StackObjectKind::SaddleAbility { .. }` in the exhaustive match.

### Step 3: Harness Action + Builder Integration

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"saddle_mount"` action type in `translate_player_action()`. Follow `"crew_vehicle"` at line ~918:
```rust
"saddle_mount" => {
    let mount_id = find_on_battlefield(state, player, card_name?)?;
    let saddle_ids: Vec<ObjectId> = convoke_names
        .iter()
        .filter_map(|name| find_on_battlefield(state, player, name.as_str()))
        .collect();
    Some(Command::SaddleMount {
        player,
        mount: mount_id,
        saddle_creatures: saddle_ids,
    })
}
```
Reuses `convoke_names` field (same pattern as `crew_vehicle`).

**File**: `crates/engine/src/state/builder.rs`
**Action**: Initialize `is_saddled: false` in the `ObjectSpec -> GameObject` construction.

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Initialize `is_saddled: false` in token creation (CreateToken effect handler), wherever other boolean flags like `is_renowned` are initialized.

### Step 4: Unit Tests

**File**: `crates/engine/tests/saddle.rs`
**Tests to write**:

1. `test_saddle_basic_mount_becomes_saddled` -- CR 702.171a: Saddle a Mount with sufficient power, resolve, verify `is_saddled == true`.

2. `test_saddle_insufficient_power_rejected` -- CR 702.171a: Total power < N is rejected.

3. `test_saddle_multiple_creatures` -- CR 702.171a: Any number of creatures with total power >= N.

4. `test_saddle_mount_cannot_saddle_itself` -- CR 702.171a: "other untapped creatures" -- the Mount cannot be in the saddle_creatures list.

5. `test_saddle_summoning_sick_creature_can_saddle` -- Ruling: summoning sickness does NOT prevent saddling (same as Crew).

6. `test_saddle_tapped_creature_rejected` -- CR 702.171a: "untapped creatures."

7. `test_saddle_not_a_creature_rejected` -- CR 702.171a: Only creatures can be tapped for saddle cost.

8. `test_saddle_already_saddled_is_legal` -- Ruling 2024-04-12: Activating saddle on an already-saddled Mount is legal.

9. `test_saddle_expires_at_end_of_turn` -- CR 702.171b: `is_saddled` is cleared during cleanup.

10. `test_saddle_sorcery_speed_only` -- CR 702.171a: "Activate only as a sorcery." Must fail if: (a) not active player's turn, (b) not main phase, (c) stack not empty.

11. `test_saddle_requires_priority` -- CR 602.2: Must hold priority.

12. `test_saddle_duplicate_creature_rejected` -- CR 702.171a: Each creature tapped once.

13. `test_saddle_opponent_creature_rejected` -- CR 702.171a: "you control."

14. `test_saddle_cleared_on_zone_change` -- CR 702.171b: Leaving the battlefield clears the saddled designation.

**Pattern**: Follow tests in `tests/crew.rs` -- same structure of helpers (find_object, pass_all, mount_spec).

### Step 5: Card Definition (later phase)

**Suggested card**: Quilled Charger (simple: Saddle 2, "attacks while saddled" trigger grants +1/+2 and menace)
**Alternate**: Rambling Possum (Saddle 1, slightly more complex with creature-return ability)
**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: Saddle a Mount, attack while saddled, verify the saddled-attack trigger fires. Then pass to next turn and verify saddled status is cleared.
**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

- **Sorcery-speed restriction**: This is the key difference from Crew. Saddle cannot be activated at instant speed, during combat, or with spells on the stack.
- **"Attacks while saddled" is a per-card trigger condition**: The core Saddle implementation only manages the designation. Individual card definitions will use `TriggerCondition::AttacksWhileSaddled` or similar -- but that can be deferred to card authoring time. For the basic implementation, just the designation flag is sufficient.
- **Humility interaction**: If Humility removes the Saddle keyword, the Mount cannot be saddled (no ability to activate). But a Mount that was ALREADY saddled stays saddled (designation, not ability). This is automatic with the boolean flag approach.
- **Copy interaction (CR 702.171b)**: "Being saddled is not a part of the permanent's copiable values." The `copy.rs` system should NOT copy `is_saddled`. Since boolean flags on GameObject are not part of `Characteristics` (the copiable values struct), this is handled automatically.
- **No ETB concerns**: Saddle doesn't cause anything to enter the battlefield.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | `KeywordAbility::Saddle(u32)` |
| `crates/engine/src/state/hash.rs` | Hash arm for Saddle |
| `crates/engine/src/state/game_object.rs` | `is_saddled: bool` field |
| `crates/engine/src/state/mod.rs` | `is_saddled: false` in both zone-move sites |
| `crates/engine/src/state/builder.rs` | `is_saddled: false` in ObjectSpec construction |
| `crates/engine/src/rules/command.rs` | `Command::SaddleMount` variant |
| `crates/engine/src/rules/engine.rs` | Handler arm for SaddleMount |
| `crates/engine/src/rules/abilities.rs` | `handle_saddle_mount()` function |
| `crates/engine/src/rules/resolution.rs` | Resolution arm for SaddleAbility SOK |
| `crates/engine/src/rules/stack.rs` | `StackObjectKind::SaddleAbility` (disc 55) |
| `crates/engine/src/rules/turn_actions.rs` | Clear `is_saddled` in cleanup |
| `crates/engine/src/effects/mod.rs` | `is_saddled: false` in token creation |
| `crates/engine/src/testing/replay_harness.rs` | `"saddle_mount"` action type |
| `tools/replay-viewer/src/view_model.rs` | Arms for Saddle KW + SaddleAbility SOK |
| `tools/tui/src/play/panels/stack_view.rs` | Arm for SaddleAbility SOK |
| `crates/engine/tests/saddle.rs` | 14 unit tests |

## Discriminant Chain

- `KeywordAbility::Saddle(u32)` = disc 140
- `StackObjectKind::SaddleAbility` = disc 55
- No new `AbilityDefinition` variant needed (Saddle is activated via dedicated Command, not via the generic `ActivateAbility` path)
- After this ability: KW next = 141, SOK next = 56, AbilDef next = 57 (unchanged)

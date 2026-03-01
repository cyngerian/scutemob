# Ability Plan: Provoke

**Generated**: 2026-03-01
**CR**: 702.39
**Priority**: P4
**Similar abilities studied**: Annihilator (CR 702.86) -- SelfAttacks triggered ability with defending-player targeting; Goad (CR 701.38) -- forced-attack requirement on CombatState; Myriad (CR 702.116) -- SelfAttacks trigger with custom StackObjectKind and PendingTrigger flag

## CR Rule Text

702.39. Provoke

702.39a Provoke is a triggered ability. "Provoke" means "Whenever this creature attacks, you may choose to have target creature defending player controls untap and block this creature this combat if able. If you do, untap that creature."

702.39b If a creature has multiple instances of provoke, each triggers separately.

### Supporting Rules

CR 508.1m: Any abilities that trigger on attackers being declared trigger.

CR 508.5: If an ability of an attacking creature refers to a defending player, then the defending player it's referring to is the player that creature is attacking.

CR 508.5a: In a multiplayer game, any rule, object, or effect that refers to a "defending player" refers to one specific defending player.

CR 509.1c: The defending player checks each creature they control to see whether it's affected by any requirements (effects that say a creature must block, or that it must block if some condition is met). If the number of requirements that are being obeyed is fewer than the maximum possible number of requirements that could be obeyed without disobeying any restrictions, the declaration of blockers is illegal.

## Key Edge Cases

- **Provoke is "may" -- optional trigger**: The attacking player CHOOSES whether to provoke. "You may choose to have target creature..." means the player can decline. For deterministic testing, the default behavior is to always exercise the provoke (always choose "yes"). Future interactive choice can be added later.
- **Target must be a creature the defending player controls**: The provoked creature must be controlled by the defending player, not any opponent. In multiplayer, "defending player" means the player the provoking creature is attacking (CR 508.5/508.5a).
- **Untap happens as part of the trigger resolution**: "If you do, untap that creature" -- the untapping is part of the provoke trigger's resolution, BEFORE blockers are declared. This means a tapped creature CAN be provoked (it gets untapped).
- **"Block this creature if able" is a blocking requirement (CR 509.1c)**: The provoked creature must block the provoking creature during the declare blockers step IF it is able to do so. This is a requirement, not a mandate that overrides restrictions. If the provoked creature can't block (e.g., it has a restriction like "can't block", it's tapped by another effect after untapping, it was removed from the battlefield), the requirement is not met and no penalty applies.
- **Multiple instances trigger separately (CR 702.39b)**: A creature with two instances of provoke triggers twice, and can provoke two different creatures. Each trigger resolves independently.
- **Provoke vs. evasion**: The provoked creature must block "if able." If the provoking creature has flying and the provoked creature has neither flying nor reach, the provoked creature CANNOT block it (restricted by 509.1b). In that case, the untap still happens but the forced-block requirement is impossible to fulfill.
- **Provoke targets a specific creature**: The attacking player targets a specific creature when the trigger goes on the stack. If that creature leaves the battlefield before the trigger resolves, the trigger fizzles (all targets illegal).
- **Timing**: Provoke triggers at declare attackers (CR 508.1m). The trigger goes on the stack and resolves BEFORE the declare blockers step. The untap and forced-block requirement are applied at resolution time.
- **Multiplayer considerations**: In a 4-player game, provoking creature attacks P2 -- can only target creatures P2 controls. If another creature attacks P3, its provoke targets creatures P3 controls.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (forced-block requirement on CombatState)
- [ ] Step 3: Trigger wiring (SelfAttacks -> ProvokeTrigger)
- [ ] Step 4: Unit tests

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Provoke` variant after `Ingest` (line ~638)
**Pattern**: Follow `KeywordAbility::Myriad` at line 535 -- triggered keyword with SelfAttacks trigger, custom StackObjectKind resolution.

```rust
/// CR 702.39: Provoke -- triggered ability.
/// "Whenever this creature attacks, you may have target creature defending
/// player controls untap and block this creature this combat if able."
///
/// Triggered ability with targeting. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// Multiple instances each trigger separately (CR 702.39b).
///
/// Resolution: untap the provoked creature, then create a forced-block
/// requirement on the CombatState (checked in handle_declare_blockers).
Provoke,
```

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` `impl HashInto for KeywordAbility` block, after Ingest (discriminant 75):

```rust
// Provoke (discriminant 79) -- CR 702.39
KeywordAbility::Provoke => 79u8.hash_into(hasher),
```

**Match arms to update** (grep for exhaustive `match` on `KeywordAbility`):

1. `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs` in `format_keyword` (after Ingest line ~687):
   ```rust
   KeywordAbility::Provoke => "Provoke".to_string(),
   ```

### Step 2: CombatState -- Forced Block Requirements

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/combat.rs`
**Action**: Add a `forced_blocks` field to `CombatState` that tracks provoke-created blocking requirements.

```rust
/// CR 702.39a / CR 509.1c: Blocking requirements created by Provoke triggers.
///
/// Each entry maps a creature (ObjectId of the provoked creature) to the
/// attacker it must block (ObjectId of the provoking creature) "if able".
/// Populated when ProvokeTrigger resolves. Checked in `handle_declare_blockers`
/// to enforce CR 509.1c (blocking requirements).
///
/// Cleared at end of combat (or when combat state is dropped).
pub forced_blocks: OrdMap<ObjectId, ObjectId>,
```

Initialize to `OrdMap::new()` in `CombatState::new()`.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `self.forced_blocks.hash_into(hasher)` to the `HashInto for CombatState` implementation. Find it by grepping for `CombatState` in hash.rs.

### Step 3: Forced Block Enforcement in handle_declare_blockers

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs`
**Action**: After all individual blocker validations pass and just before recording blockers in combat state (~line 630), add a check for forced-block requirements:

```rust
// CR 702.39a / CR 509.1c: Provoke forced-block requirements.
// Each provoked creature must block its provoking attacker if able.
// "If able" means: the creature is still on the battlefield, is untapped,
// is a creature, can legally block that attacker (flying, shadow, menace,
// protection checks all pass), and is controlled by the defending player.
// If all conditions are met and the creature is NOT in the blocker list
// blocking its provoking attacker, the declaration is illegal.
if let Some(combat) = state.combat.as_ref() {
    for (&provoked_id, &must_block_attacker) in &combat.forced_blocks {
        // Only check if the provoked creature is controlled by this declaring player.
        let provoked_obj = match state.objects.get(&provoked_id) {
            Some(o) if o.controller == player && o.zone == ZoneId::Battlefield => o,
            _ => continue, // Not this player's creature or not on battlefield
        };

        // Check if the creature is tapped (can't block if tapped).
        if provoked_obj.status.tapped {
            continue;
        }

        // Check if the attacker is still a declared attacker.
        if !combat.attackers.contains_key(&must_block_attacker) {
            continue;
        }

        // Check evasion restrictions: can this creature legally block the attacker?
        // (flying, shadow, horsemanship, skulk, intimidate, fear, protection, landwalk,
        //  CantBeBlocked, decayed can't block)
        // If any restriction prevents blocking, the requirement is impossible to satisfy.
        let can_block = can_creature_block_attacker(state, provoked_id, must_block_attacker, player);
        if !can_block {
            continue;
        }

        // The creature CAN block the attacker. Check if it IS blocking it.
        let is_blocking_required_attacker = blockers.iter().any(|(b, a)| *b == provoked_id && *a == must_block_attacker)
            || combat.blockers.get(&provoked_id) == Some(&must_block_attacker);
        if !is_blocking_required_attacker {
            return Err(GameStateError::InvalidCommand(format!(
                "Creature {:?} must block {:?} (provoke requirement, CR 702.39a / CR 509.1c)",
                provoked_id, must_block_attacker
            )));
        }
    }
}
```

**New helper function**: `can_creature_block_attacker(state, blocker_id, attacker_id, declaring_player) -> bool`

This helper extracts the evasion checks from the existing `handle_declare_blockers` loop body (lines ~452-595) into a reusable function. It checks all the restrictions that prevent blocking (flying without flying/reach, shadow mismatch, horsemanship, skulk, intimidate, fear, protection, landwalk, CantBeBlocked, decayed). Returns `true` if the blocker can legally block the attacker, `false` otherwise.

**Design note**: Refactoring the evasion checks into a helper is preferable to duplicating them. The existing per-blocker validation loop should also call this helper (reducing the body to `if !can_creature_block_attacker(...) { return Err(...); }`). However, to keep the diff minimal for Batch 2, the runner MAY choose to duplicate the critical checks inline instead of refactoring. The plan documents both approaches; the runner should prefer the helper if time permits.

### Step 4: StackObjectKind -- ProvokeTrigger

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add a `ProvokeTrigger` variant to `StackObjectKind` after `IngestTrigger`:

```rust
/// CR 702.39a: Provoke triggered ability on the stack.
///
/// "Whenever this creature attacks, you may have target creature defending
/// player controls untap and block this creature this combat if able."
///
/// `source_object` is the creature with provoke (the attacker).
/// `provoked_creature` is the target creature (defending player controls).
///
/// When this trigger resolves:
/// 1. Check if the provoked creature is still on the battlefield and controlled
///    by the defending player (target legality check).
/// 2. Untap the provoked creature (CR 702.39a: "untap that creature").
/// 3. Add a forced-block requirement to `CombatState::forced_blocks`:
///    provoked_creature must block source_object "if able" (CR 509.1c).
///
/// If the provoked creature has left the battlefield or changed controllers,
/// the trigger fizzles (target illegal, CR 608.2b).
ProvokeTrigger {
    source_object: ObjectId,
    provoked_creature: ObjectId,
},
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash for `StackObjectKind::ProvokeTrigger` with discriminant 19:

```rust
// ProvokeTrigger (discriminant 19) -- CR 702.39a
StackObjectKind::ProvokeTrigger {
    source_object,
    provoked_creature,
} => {
    19u8.hash_into(hasher);
    source_object.hash_into(hasher);
    provoked_creature.hash_into(hasher);
}
```

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add match arm in `convert_stack_object_kind`:

```rust
StackObjectKind::ProvokeTrigger { source_object, .. } => {
    ("provoke_trigger", Some(*source_object))
}
```

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm for `ProvokeTrigger`:

```rust
StackObjectKind::ProvokeTrigger { source_object, .. } => {
    ("Provoke: ".to_string(), Some(*source_object))
}
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action 1**: Add resolution arm for `ProvokeTrigger` (after `IngestTrigger` resolution at line ~1671):

```rust
// CR 702.39a: Provoke trigger resolves -- untap the provoked creature
// and create a forced-block requirement.
StackObjectKind::ProvokeTrigger {
    source_object,
    provoked_creature,
} => {
    let controller = stack_obj.controller;

    // Target legality: provoked creature must still be on the battlefield.
    let target_valid = state.objects.get(&provoked_creature)
        .map(|obj| obj.zone == ZoneId::Battlefield)
        .unwrap_or(false);

    if target_valid {
        // 1. Untap the provoked creature (CR 702.39a: "untap that creature").
        if let Some(obj) = state.objects.get_mut(&provoked_creature) {
            if obj.status.tapped {
                obj.status.tapped = false;
                events.push(GameEvent::PermanentUntapped {
                    player: obj.controller,
                    object_id: provoked_creature,
                });
            }
        }

        // 2. Add forced-block requirement to CombatState.
        if let Some(combat) = state.combat.as_mut() {
            combat.forced_blocks.insert(provoked_creature, source_object);
        }
    }
    // If target invalid, trigger fizzles -- do nothing (CR 608.2b).

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Action 2**: Add `StackObjectKind::ProvokeTrigger { .. }` to the "countering abilities" match arm (line ~1762) alongside other trigger variants.

### Step 5: PendingTrigger -- Provoke Trigger Flag

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add new fields to `PendingTrigger` after `ingest_target_player` (line ~242):

```rust
/// CR 702.39a: If true, this pending trigger is a Provoke trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::ProvokeTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The
/// `provoke_target_creature` carries the ObjectId of the creature to
/// be provoked. The `ability_index` field is unused when this is true.
#[serde(default)]
pub is_provoke_trigger: bool,
/// CR 702.39a: The ObjectId of the creature that must block "if able".
///
/// Only meaningful when `is_provoke_trigger` is true. This is the target
/// creature the defending player controls. Set at trigger-collection time
/// in the `AttackersDeclared` handler in `abilities.rs`.
#[serde(default)]
pub provoke_target_creature: Option<ObjectId>,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add to `HashInto for PendingTrigger` (after ingest fields, line ~1092):

```rust
// CR 702.39a: is_provoke_trigger -- provoke attack trigger marker
self.is_provoke_trigger.hash_into(hasher);
self.provoke_target_creature.hash_into(hasher);
```

### Step 6: Trigger Wiring -- builder.rs + abilities.rs

#### Part 6a: Builder auto-generation of TriggeredAbilityDef

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: In the keyword-to-triggered-ability translation block (after the Myriad block at line ~685), add Provoke auto-generation:

```rust
// CR 702.39a: Provoke -- "Whenever this creature attacks, you may
// have target creature defending player controls untap and block this
// creature this combat if able."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.39b).
// The actual untap + forced-block logic is in StackObjectKind::ProvokeTrigger
// resolution. The description starts with "Provoke" so abilities.rs can
// identify and tag it as a provoke trigger at collection time.
if matches!(kw, KeywordAbility::Provoke) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacks,
        intervening_if: None,
        description: "Provoke (CR 702.39a): Whenever this creature attacks, \
                      you may have target creature defending player controls \
                      untap and block this creature this combat if able."
            .to_string(),
        effect: None, // Handled by ProvokeTrigger resolution
    });
}
```

#### Part 6b: Trigger collection + tagging in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In the `AttackersDeclared` handler (line ~1325), after the existing `SelfAttacks` trigger collection loop (which already tags `defending_player_id` and `is_myriad_trigger`), add Provoke trigger tagging.

The provoke trigger needs:
1. To be identified as a provoke trigger (`is_provoke_trigger = true`)
2. To carry the target creature to provoke (`provoke_target_creature`)

**Targeting strategy**: Provoke targets "target creature defending player controls." At trigger-collection time, we need to select which creature to provoke. For deterministic behavior (no interactive choice), select the first eligible creature controlled by the defending player (by ObjectId order). "Eligible" = on the battlefield, is a creature, controlled by the defending player.

Add this block inside the `for (attacker_id, attack_target) in attackers` loop in the `AttackersDeclared` handler, after the myriad tagging block:

```rust
// CR 702.39a: Tag provoke triggers.
// A SelfAttacks trigger is a provoke trigger if its source object has
// the Provoke keyword. Identified by the description starting with "Provoke".
// Select a target creature the defending player controls (deterministic:
// first creature by ObjectId order).
for t in &mut triggers[pre_len..] {
    if let Some(obj) = state.objects.get(&t.source) {
        if let Some(ta) =
            obj.characteristics.triggered_abilities.get(t.ability_index)
        {
            if ta.description.starts_with("Provoke") {
                t.is_provoke_trigger = true;

                // Select target: first creature controlled by the defending player.
                if let Some(dp) = defending_player {
                    let target = state
                        .objects
                        .values()
                        .filter(|o| {
                            o.zone == ZoneId::Battlefield
                                && o.controller == dp
                                && calculate_characteristics(state, o.id)
                                    .map(|c| c.card_types.contains(&CardType::Creature))
                                    .unwrap_or(false)
                        })
                        .map(|o| o.id)
                        .next(); // Deterministic: OrdMap iteration is by ObjectId
                    t.provoke_target_creature = target;
                }
            }
        }
    }
}
```

**Important**: If no eligible creature exists (defending player controls no creatures), `provoke_target_creature` is `None`. When flushing, if the target is `None`, skip creating the trigger on the stack (CR 603.3d -- triggered ability with targets that can't be legally targeted is not placed on the stack).

#### Part 6c: Flush to stack in flush_pending_triggers

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers`, add a branch for `is_provoke_trigger` (after the `is_ingest_trigger` branch, around line ~2169 based on the Ingest pattern):

```rust
} else if trigger.is_provoke_trigger {
    // CR 702.39a: Provoke SelfAttacks trigger -- "Whenever this creature
    // attacks, you may have target creature defending player controls
    // untap and block this creature this combat if able."
    //
    // If no valid target was found at trigger-collection time, skip
    // placing this trigger on the stack (CR 603.3d).
    if let Some(provoked) = trigger.provoke_target_creature {
        StackObjectKind::ProvokeTrigger {
            source_object: trigger.source,
            provoked_creature: provoked,
        }
    } else {
        // No valid target -- trigger is not placed on the stack.
        continue;
    }
}
```

Also set the trigger targets so the stack object has the provoked creature as its declared target:

In the `trigger_targets` construction section (around line ~2020-2048), add a branch:

```rust
} else if trigger.is_provoke_trigger {
    if let Some(provoked) = trigger.provoke_target_creature {
        vec![SpellTarget {
            target: Target::Object(provoked),
            zone_at_cast: None,
        }]
    } else {
        vec![]
    }
}
```

### Step 7: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/provoke.rs` (new file)
**Pattern**: Follow `/home/airbaggie/scutemob/crates/engine/tests/annihilator.rs` for structure, helpers, and test style.

**Tests to write**:

1. `test_702_39a_provoke_basic_untap_and_forced_block` -- CR 702.39a
   - Setup: P1 has a creature with `Provoke` on battlefield. P2 has a tapped creature.
   - P1 declares the provoke creature as attacker targeting P2.
   - Provoke trigger fires and resolves: P2's creature is untapped and must block.
   - P2 declares blockers with the provoked creature blocking the provoking creature.
   - Assert: P2's creature is untapped after trigger resolution.
   - Assert: Declaration succeeds (forced block is satisfied).

2. `test_702_39a_provoke_forces_block_requirement` -- CR 509.1c
   - Setup: P1 has a creature with `Provoke`. P2 has an untapped creature.
   - After provoke resolves, P2 declares NO blockers (empty declaration).
   - Assert: Error -- provoked creature must block if able.

3. `test_702_39a_provoke_tapped_creature_untapped` -- CR 702.39a
   - Setup: P2 has a tapped creature. After provoke trigger resolves, the creature should be untapped.
   - Assert: The creature's `status.tapped` is `false` after resolution.
   - Assert: `PermanentUntapped` event is emitted.

4. `test_702_39a_provoke_creature_cant_block_flying` -- CR 509.1b/c
   - Setup: P1's provoke creature has Flying. P2's creature has neither Flying nor Reach.
   - After provoke resolves, P2's creature is untapped. But it CANNOT block a flier.
   - P2 declares no blockers (or other blockers).
   - Assert: No error -- forced-block requirement is impossible to satisfy due to evasion restriction, so the requirement is dropped.

5. `test_702_39b_provoke_multiple_instances_trigger_separately` -- CR 702.39b
   - Setup: P1's creature has Provoke listed twice (two keyword instances). P2 has two creatures.
   - P1 attacks. Two provoke triggers fire. Each resolves targeting a different creature.
   - Assert: Two triggers on the stack after attack declaration.

6. `test_702_39a_provoke_no_valid_target` -- CR 603.3d
   - Setup: P2 controls no creatures.
   - P1 attacks with provoke creature.
   - Assert: No provoke trigger goes on the stack (no valid target).

7. `test_702_39a_provoke_multiplayer_correct_defender` -- CR 508.5a
   - Setup: 4-player game. P1 attacks P2 with provoke creature.
   - P3 has creatures, but they should NOT be targeted.
   - Assert: Provoke target is a creature P2 controls, not P3.

### Step 8: Card Definition (later phase)

**Suggested card**: Goblin Grappler
- Simple: {R}, 1/1, Provoke. No other abilities.
- Oracle text: "Provoke (Whenever this creature attacks, you may have target creature defending player controls untap and block it if able.)"

**Card lookup**: Use `card-definition-author` agent with name "Goblin Grappler".

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/goblin_grappler.rs`

### Step 9: Game Script (later phase)

**Suggested scenario**: 4-player Commander game. P1 controls Goblin Grappler. P2 controls a tapped 2/2 creature. P1 attacks P2 with Goblin Grappler. Provoke trigger fires and resolves: P2's creature is untapped. P2 must declare the provoked creature as a blocker. Combat damage resolves. Verify: the forced block occurred and P2's creature blocked the Goblin Grappler.

**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

- **Provoke + evasion**: If the provoking creature gains flying after provoke resolves but before blockers, the provoked creature (without flying/reach) cannot satisfy the requirement. The forced-block check must re-evaluate evasion at declare-blockers time, not at provoke-resolution time. Since `handle_declare_blockers` reads live characteristics, this is handled automatically.
- **Provoke + CantBeBlocked keyword**: If the provoking creature has CantBeBlocked, the provoked creature literally cannot block it. The forced-block requirement is impossible to satisfy and should be dropped.
- **Provoke + tapped between resolution and blockers**: If the provoked creature is tapped AFTER provoke untaps it (e.g., by another ability resolving), it cannot block (tapped creatures can't block). The forced-block check in `handle_declare_blockers` checks `status.tapped`, so this is handled.
- **Provoke + Menace**: If the provoking creature has Menace, the provoked creature alone cannot satisfy Menace's "must be blocked by two or more creatures" requirement. BUT the provoked creature IS required to be one of the blockers. If only one creature is assigned, the Menace check rejects the declaration. If two or more creatures block (including the provoked one), both Menace and Provoke requirements are satisfied.
- **Provoke + Decayed**: A creature with Decayed "can't block" (CR 702.147a). This is a restriction. The forced-block requirement from Provoke cannot override a restriction (CR 509.1c). The provoked creature with Decayed cannot satisfy the requirement.
- **CombatState.forced_blocks cleanup**: The `forced_blocks` map is part of `CombatState`, which is `None` outside of the combat phase. It is naturally cleaned up when combat ends. No additional cleanup needed.
- **Target fizzle**: If the provoked creature leaves the battlefield between trigger going on the stack and resolution, the trigger fizzles. Standard target-legality check at resolution time.
- **Deterministic targeting**: Without interactive choice, the engine picks the first eligible creature by `ObjectId` order (OrdMap iteration). This is consistent with other deterministic fallbacks in the engine (SacrificePermanents, Scry, etc.).
- **General-purpose forced-block infrastructure**: The `forced_blocks: OrdMap<ObjectId, ObjectId>` on `CombatState` is general-purpose. Future abilities that create "must block if able" requirements (e.g., Lure, "must be blocked if able") can reuse this field. Provoke is not a special case -- it just populates the same data structure.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Provoke` |
| `crates/engine/src/state/hash.rs` | Add hash discriminant 79 for Provoke; add hash for `forced_blocks` in CombatState; add hash for ProvokeTrigger (SOK disc 19); add hash for PendingTrigger provoke fields |
| `crates/engine/src/state/stubs.rs` | Add `is_provoke_trigger: bool` and `provoke_target_creature: Option<ObjectId>` to PendingTrigger |
| `crates/engine/src/state/combat.rs` | Add `forced_blocks: OrdMap<ObjectId, ObjectId>` to CombatState |
| `crates/engine/src/state/stack.rs` | Add `StackObjectKind::ProvokeTrigger` variant |
| `crates/engine/src/state/builder.rs` | Auto-generate TriggeredAbilityDef from Provoke keyword |
| `crates/engine/src/rules/abilities.rs` | Tag provoke triggers with `is_provoke_trigger` + `provoke_target_creature`; add ProvokeTrigger branch in `flush_pending_triggers`; add provoke target in `trigger_targets` |
| `crates/engine/src/rules/combat.rs` | Add forced-block requirement validation in `handle_declare_blockers`; optionally extract `can_creature_block_attacker` helper |
| `crates/engine/src/rules/resolution.rs` | Add `ProvokeTrigger` resolution arm (untap + forced-block insertion); add to countering match arm |
| `tools/replay-viewer/src/view_model.rs` | Add Provoke to `format_keyword`; add ProvokeTrigger to `convert_stack_object_kind` |
| `tools/tui/src/play/panels/stack_view.rs` | Add ProvokeTrigger match arm |
| `crates/engine/tests/provoke.rs` | New test file: 7 tests |

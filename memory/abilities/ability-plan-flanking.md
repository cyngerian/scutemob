# Ability Plan: Flanking

**Generated**: 2026-03-01
**CR**: 702.25
**Priority**: P4
**Similar abilities studied**: Ingest (CR 702.115) -- `state/types.rs:629-638`, `state/hash.rs:467-468`, `state/stubs.rs:230-242`, `rules/abilities.rs:1757-1838`, `rules/resolution.rs:1634-1671`, `tests/ingest.rs`; BattleCry (CR 702.91) -- `state/builder.rs:439-462` (TriggeredAbilityDef generation from keyword); Skulk (CR 702.118) -- `rules/combat.rs` (block restriction pattern in handle_declare_blockers)

## CR Rule Text

702.25. Flanking

702.25a Flanking is a triggered ability that triggers during the declare blockers step. (See rule 509, "Declare Blockers Step.") "Flanking" means "Whenever this creature becomes blocked by a creature without flanking, the blocking creature gets -1/-1 until end of turn."

702.25b If a creature has multiple instances of flanking, each triggers separately.

## Key Edge Cases

- **Triggered ability, not a static restriction**: Unlike Skulk, Shadow, or Horsemanship, Flanking is NOT a block restriction. Blocking is always legal; the -1/-1 is applied after blockers are declared, via a triggered ability on the stack. The blocker CAN block; it just gets penalized.
- **Per-blocker trigger (CR 509.3d pattern)**: The trigger fires once per blocking creature without flanking. If 3 creatures without flanking block a flanking creature, 3 separate triggers fire (one for each blocker). If one of those blockers HAS flanking, only 2 triggers fire (for the two without flanking).
- **Blocker must lack flanking at block-declaration time (CR 509.3f)**: The "without flanking" check is at the moment blockers are declared. If the blocker later gains flanking (e.g., from Cavalry Master entering), the trigger has already fired. Conversely, if the blocker has flanking at declaration time, the trigger does not fire, even if flanking is later removed.
- **Multiple instances trigger separately (CR 702.25b)**: A creature with two instances of flanking creates two triggers per qualifying blocker. E.g., 3 blockers without flanking on a creature with flanking x2 = 6 triggers total (3 blockers x 2 instances). Each resolves independently.
- **-1/-1 can kill the blocker**: If the blocker has 1 toughness, the -1/-1 from flanking reduces it to 0 toughness. SBAs will destroy it (CR 704.5f) after the trigger resolves. The blocker dies before combat damage is dealt. The attacker is still "blocked" (CR 509.1h) unless it has trample.
- **"Gets -1/-1 until end of turn"**: This is a continuous effect in Layer 7c (P/T modification), not counters. It expires at cleanup (CR 514.2). Uses `ModifyBoth(-1)` with `UntilEndOfTurn` duration.
- **Flanking creature blocking a flanking creature**: If a creature with flanking blocks another creature with flanking, the trigger does NOT fire (the blocker HAS flanking). Flanking only triggers when blocked by a creature WITHOUT flanking.
- **Multiplayer**: Each defending player declares blockers independently. Flanking triggers fire for each qualifying blocker across all defending players. APNAP ordering applies as usual.
- **Trigger goes on the stack**: Flanking triggers use the stack and can be responded to (Stifled, etc.). The -1/-1 only applies when the trigger resolves.
- **Ruling 2006-09-25 (Cavalry Master)**: The "multiple instances" behavior comes from the card definition's `abilities` Vec count, not the runtime `OrdSet` (which deduplicates). Same pattern as Ingest (CR 702.115b).

## Current State (from ability-wip.md)

Flanking is not the current WIP ability (Ingest was the last, now closed). All steps are fresh:

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (trigger wiring in abilities.rs)
- [ ] Step 3: FlankingTrigger StackObjectKind + PendingTrigger fields
- [ ] Step 4: Resolution in resolution.rs
- [ ] Step 5: Unit tests
- [ ] Step 6: Card definition
- [ ] Step 7: Game script
- [ ] Step 8: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Flanking` variant after `Ingest` (line ~638)
**Pattern**: Follow `KeywordAbility::Ingest` at line 629-638 -- simple unit variant, no parameters
**Doc comment**:
```rust
/// CR 702.25: Flanking -- triggered ability.
/// "Whenever this creature becomes blocked by a creature without flanking,
/// the blocking creature gets -1/-1 until end of turn."
/// CR 702.25b: Multiple instances trigger separately.
Flanking,
```

**Hash**: Add to `state/hash.rs` `HashInto` impl for `KeywordAbility` after `Ingest` (discriminant 76):
```rust
// Flanking (discriminant 76) -- CR 702.25
KeywordAbility::Flanking => 76u8.hash_into(hasher),
```
Add after the `Ingest` line at ~468.

**Replay viewer `format_keyword`**: Add arm to `tools/replay-viewer/src/view_model.rs` in the `format_keyword` match after `KeywordAbility::Ingest`:
```rust
KeywordAbility::Flanking => "Flanking".to_string(),
```

**Match arms**: Grep for `KeywordAbility::Ingest` to find all exhaustive match arms that need a new case. Known sites:
1. `state/hash.rs` -- covered above
2. `tools/replay-viewer/src/view_model.rs:format_keyword` -- covered above

### Step 2: PendingTrigger Fields

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add two new fields to `PendingTrigger` after `ingest_target_player` (line ~242):
```rust
/// CR 702.25a: If true, this pending trigger is a Flanking trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::FlankingTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`.
/// The `flanking_blocker_id` carries the blocking creature's ObjectId so
/// the resolution knows which creature to apply -1/-1 to.
#[serde(default)]
pub is_flanking_trigger: bool,
/// CR 702.25a: The blocking creature that gets -1/-1 until end of turn.
///
/// Only meaningful when `is_flanking_trigger` is true.
#[serde(default)]
pub flanking_blocker_id: Option<ObjectId>,
```

**Hash**: Add to `state/hash.rs` `HashInto` impl for `PendingTrigger` after the `ingest_target_player` line (~1092):
```rust
// CR 702.25a: is_flanking_trigger -- flanking blocker trigger marker
self.is_flanking_trigger.hash_into(hasher);
self.flanking_blocker_id.hash_into(hasher);
```

### Step 3: StackObjectKind Variant

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `FlankingTrigger` variant after `IngestTrigger` (line ~427):
```rust
/// CR 702.25a: Flanking triggered ability on the stack.
///
/// "Whenever this creature becomes blocked by a creature without flanking,
/// the blocking creature gets -1/-1 until end of turn."
///
/// `source_object` is the creature with flanking (the attacker).
/// `blocker_id` is the blocking creature that will receive -1/-1.
///
/// When this trigger resolves:
/// 1. Check if the blocker is still on the battlefield (CR 400.7).
/// 2. If yes, register a ContinuousEffect with ModifyBoth(-1) in Layer 7c
///    (PtModify) targeting SingleObject(blocker_id) with UntilEndOfTurn duration.
/// 3. If the blocker left the battlefield, do nothing (trigger fizzles).
///
/// CR 702.25b: Multiple instances trigger separately (each creates its own
/// trigger with the same blocker_id).
FlankingTrigger {
    source_object: ObjectId,
    blocker_id: ObjectId,
},
```

**Hash**: Add to `state/hash.rs` `HashInto` impl for `StackObjectKind` after `IngestTrigger` discriminant 18 (~line 1377):
```rust
// FlankingTrigger (discriminant 19) -- CR 702.25a
StackObjectKind::FlankingTrigger {
    source_object,
    blocker_id,
} => {
    19u8.hash_into(hasher);
    source_object.hash_into(hasher);
    blocker_id.hash_into(hasher);
}
```

**TUI stack_view.rs**: Add arm to `tools/tui/src/play/panels/stack_view.rs` in the exhaustive match after `IngestTrigger` (line ~82):
```rust
StackObjectKind::FlankingTrigger { source_object, .. } => {
    ("Flanking: ".to_string(), Some(*source_object))
}
```

**Replay viewer view_model.rs**: Add arm to `tools/replay-viewer/src/view_model.rs` in the StackObjectKind match after `IngestTrigger` (line ~475):
```rust
StackObjectKind::FlankingTrigger { source_object, .. } => {
    ("flanking_trigger", Some(*source_object))
}
```

### Step 4: Trigger Wiring in abilities.rs

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add flanking trigger dispatch inside the `BlockersDeclared` event handler (line ~1444), after the existing `SelfBlocks` trigger collection.

The `BlockersDeclared` event carries `blockers: Vec<(ObjectId, ObjectId)>` where each pair is `(blocker_id, attacker_id)`. For flanking:
1. For each `(blocker_id, attacker_id)` pair, check if the ATTACKER has the `Flanking` keyword.
2. If yes, check if the BLOCKER does NOT have the `Flanking` keyword (CR 702.25a: "creature without flanking").
3. If both conditions met, count flanking instances from the card definition (CR 702.25b: multiple instances trigger separately).
4. Create one `PendingTrigger` per flanking instance, with `is_flanking_trigger: true` and `flanking_blocker_id: Some(blocker_id)`.

**Pattern**: Follow Ingest trigger dispatch at lines 1757-1838 for the `keyword_count` pattern.

**CR**: 702.25a -- triggers when blocked by a creature without flanking.
**CR**: 702.25b -- multiple instances trigger separately.
**CR**: 509.3f -- characteristics checked at block-declaration time (already handled by checking at event time).

```rust
// CR 702.25a: Flanking -- "Whenever this creature becomes blocked by
// a creature without flanking, the blocking creature gets -1/-1 until
// end of turn."
// CR 702.25b: Multiple instances trigger separately.
for (blocker_id, attacker_id) in blockers {
    if let Some(attacker_obj) = state.objects.get(attacker_id) {
        if attacker_obj.zone != ZoneId::Battlefield {
            continue;
        }
        if !attacker_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Flanking)
        {
            continue;
        }

        // Check that the blocker does NOT have flanking (CR 702.25a).
        let blocker_has_flanking = state
            .objects
            .get(blocker_id)
            .map(|b| {
                b.characteristics
                    .keywords
                    .contains(&KeywordAbility::Flanking)
            })
            .unwrap_or(false);
        if blocker_has_flanking {
            continue;
        }

        // Count flanking instances from card definition (CR 702.25b).
        let flanking_count = attacker_obj
            .card_id
            .as_ref()
            .and_then(|cid| state.card_registry.get(cid.clone()))
            .map(|def| {
                def.abilities
                    .iter()
                    .filter(|a| {
                        matches!(
                            a,
                            crate::cards::card_definition::AbilityDefinition::Keyword(
                                KeywordAbility::Flanking
                            )
                        )
                    })
                    .count()
            })
            .unwrap_or(1)
            .max(1);

        let controller = attacker_obj.controller;
        let source_id = attacker_obj.id;
        for _ in 0..flanking_count {
            triggers.push(PendingTrigger {
                source: source_id,
                ability_index: 0, // unused for flanking triggers
                controller,
                triggering_event: Some(TriggerEvent::SelfBlocks), // closest existing event
                entering_object_id: None,
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
                is_evolve_trigger: false,
                evolve_entering_creature: None,
                is_myriad_trigger: false,
                is_suspend_counter_trigger: false,
                is_suspend_cast_trigger: false,
                suspend_card_id: None,
                is_hideaway_trigger: false,
                hideaway_count: None,
                is_partner_with_trigger: false,
                partner_with_name: None,
                is_ingest_trigger: false,
                ingest_target_player: None,
                is_flanking_trigger: true,
                flanking_blocker_id: Some(*blocker_id),
            });
        }
    }
}
```

**IMPORTANT**: Every existing `PendingTrigger` literal in the file must be updated to include `is_flanking_trigger: false, flanking_blocker_id: None`. Grep for `is_ingest_trigger` to find all sites (~15-20 occurrences).

### Step 5: Flush Handler in abilities.rs

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add flanking trigger flush handling in `flush_pending_triggers`, in the `if/else if` chain that builds `StackObjectKind`, after the `is_ingest_trigger` branch (line ~2226):

```rust
} else if trigger.is_flanking_trigger {
    // CR 702.25a: Flanking trigger -- "the blocking creature gets -1/-1
    // until end of turn."
    // `flanking_blocker_id` carries the blocking creature's ObjectId.
    StackObjectKind::FlankingTrigger {
        source_object: trigger.source,
        blocker_id: trigger.flanking_blocker_id.unwrap_or(trigger.source),
    }
}
```

### Step 6: Resolution in resolution.rs

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add `FlankingTrigger` resolution arm before the closing `}` of the main resolution match (before the fizzle/counter match arm at ~line 1762).

```rust
// CR 702.25a: Flanking trigger resolves -- the blocking creature gets
// -1/-1 until end of turn.
//
// The -1/-1 is a continuous effect in Layer 7c (PtModify) with
// UntilEndOfTurn duration. If the blocker has left the battlefield
// by resolution time (CR 400.7), the trigger does nothing.
StackObjectKind::FlankingTrigger {
    source_object: _,
    blocker_id,
} => {
    let controller = stack_obj.controller;

    // Check if the blocker is still on the battlefield.
    let blocker_alive = state
        .objects
        .get(&blocker_id)
        .map(|obj| obj.zone == ZoneId::Battlefield)
        .unwrap_or(false);

    if blocker_alive {
        // Register the -1/-1 continuous effect (Layer 7c, UntilEndOfTurn).
        let timestamp = state.next_timestamp();
        let effect = ContinuousEffect {
            source: blocker_id, // effect applies to the blocker
            filter: EffectFilter::SingleObject(blocker_id),
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(-1),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp,
        };
        state.continuous_effects.push_back(effect);
    }
    // If blocker left the battlefield, do nothing (CR 400.7).

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Also add `FlankingTrigger` to the counter match arm** at ~line 1779 (the `| StackObjectKind::IngestTrigger { .. }` list):
```rust
| StackObjectKind::FlankingTrigger { .. }
```

### Step 7: Unit Tests

**File**: `crates/engine/tests/flanking.rs`
**Tests to write**:

1. `test_702_25_flanking_basic_minus_one_minus_one`
   - CR 702.25a -- Basic case: 2/2 flanking creature is blocked by a 2/2 without flanking. After trigger resolves, the blocker should be 1/1.
   - Setup: p1 has a 2/2 creature with Flanking keyword, p2 has a 2/2 creature without Flanking.
   - Declare attacker (p1's creature), declare blocker (p2's creature blocks it).
   - Advance through priority so the trigger resolves.
   - Assert: blocker's calculated power/toughness is 1/1 (via `calculate_characteristics`).
   - Assert: AbilityTriggered and AbilityResolved events fire.

2. `test_702_25_flanking_does_not_trigger_on_flanking_blocker`
   - CR 702.25a -- Flanking creature blocked by another flanking creature: no trigger fires.
   - Setup: p1 has a 2/2 flanking creature, p2 has a 2/2 flanking creature.
   - Declare attacker, declare blocker. No trigger should fire.
   - Assert: stack is empty after blockers declared.

3. `test_702_25_flanking_kills_1_toughness_blocker`
   - CR 702.25a + CR 704.5f -- A 1/1 blocker blocking a flanking creature gets -1/-1, making it 0/0. SBA destroys it.
   - Setup: p1 has a 2/2 flanking creature, p2 has a 1/1 creature.
   - Declare attacker, declare blocker, resolve the flanking trigger.
   - Assert: the blocker is in the graveyard (SBAs ran after resolution).
   - Assert: the attacker is still "blocked" (no damage to player without trample).

4. `test_702_25b_flanking_multiple_instances`
   - CR 702.25b -- Multiple instances trigger separately. A creature with flanking x2 generates TWO triggers per qualifying blocker.
   - Setup: Build a CardDefinition with two `Keyword(Flanking)` entries. p2 has a 3/3 blocker without flanking.
   - Declare attacker, declare blocker.
   - Assert: 2 triggers on the stack.
   - Resolve both: blocker should be 1/1 (3/3 - 2x(-1/-1) = 1/1).

5. `test_702_25_flanking_multiple_blockers`
   - CR 702.25a + CR 509.3d -- Each qualifying blocker triggers flanking separately.
   - Setup: p1 has a 2/2 flanking creature with menace (so it CAN be blocked by 2+ creatures). p2 has two 2/2 creatures without flanking.
   - Declare both blockers on the flanking+menace creature.
   - Assert: 2 triggers on the stack (one per blocker).
   - Resolve both: each blocker should be 1/1.

6. `test_702_25_flanking_effect_expires_at_end_of_turn`
   - CR 702.25a -- The -1/-1 is "until end of turn" (UntilEndOfTurn duration).
   - This is implicitly tested by the `UntilEndOfTurn` infrastructure, but verify that after cleanup step, the blocker's stats return to normal.
   - Setup: 2-player, flanking creature attacks, blocker blocks, trigger resolves.
   - Advance through cleanup (pass all through remaining combat steps + end step + cleanup).
   - Assert: blocker's characteristics return to original P/T.

7. `test_702_25_flanking_multiplayer`
   - CR 702.25a -- In a 4-player game, flanking triggers fire correctly for blockers from different defending players.
   - Setup: p1 attacks p2 and p3 with two different flanking creatures. p2 and p3 each declare one blocker.
   - Assert: 2 flanking triggers on the stack (one for p2's blocker, one for p3's blocker).
   - Resolve both: each blocker should have -1/-1 applied.

**Pattern**: Follow tests in `tests/ingest.rs` -- same file structure, same `find_object` helper, same `pass_all` helper, same `GameStateBuilder` setup with `.at_step(Step::DeclareAttackers)`, same combat state initialization pattern.

### Step 8: Card Definition (later phase)

**Suggested card**: Suq'Ata Lancer ({2}{R}, 2/2, Creature - Human Knight, Haste + Flanking)
- Clean oracle text with two well-understood keywords (both already implemented: Haste is P1, Flanking will be newly implemented)
- 2/2 body interacts well with flanking tests (can kill 1-toughness blockers via flanking, survives combat with 2/2 blockers that become 1/1)
- Alternative: Mtenda Herder ({W}, 1/1, Creature - Human Scout, Flanking) -- even simpler, vanilla flanking creature

**Card lookup**: use `card-definition-author` agent

### Step 9: Game Script (later phase)

**Suggested scenario**: "Flanking creature attacks, blocked by non-flanking creature, trigger resolves giving -1/-1"
- Turn 1: p1 attacks with a 2/2 flanking creature
- p2 blocks with a 2/2 creature
- Flanking trigger fires and resolves: blocker becomes 1/1
- Combat damage: attacker deals 2 to 1/1 blocker (dies), blocker deals 1 to 2/2 attacker (survives at 2/1)
- Validates CR 702.25a basic behavior

**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

- **Flanking + Trample**: If a flanking creature with trample is blocked by a 1/1, the flanking trigger kills the blocker before combat damage. With trample, the attacker can assign lethal (0 -- already dead) to the dead blocker and the rest to the player. However, since the blocker was already removed from combat (SBAs), trample overflow goes to the player naturally.
- **Flanking + First Strike**: If the flanking creature has first strike, the flanking trigger resolves before first strike damage. The blocker is -1/-1, so it may die from first strike damage. Normal damage then only involves the surviving creatures.
- **Flanking + Protection**: If the blocker has protection from the flanking creature's color, the flanking trigger is NOT affected -- flanking is not a damage source, it's a triggered ability that creates a continuous effect. Protection prevents DEBT (Damage, Enchanting/Equipping, Blocking, Targeting) -- the -1/-1 continuous effect is none of these. However, the protection check prevents BLOCKING in the first place (CR 702.16b), so the blocker shouldn't be able to block the flanking creature. Edge case: protection from a different quality than what the flanking creature has.
- **Flanking + Humility**: Humility removes all abilities including flanking (Layer 6). If flanking is removed before blockers are declared, the trigger does not fire. If Humility enters after blockers are declared but before the trigger resolves, the trigger is already on the stack and still resolves (removing a keyword doesn't remove a trigger that's already on the stack).
- **Flanking creature as blocker**: Flanking only triggers when the flanking creature IS THE ATTACKER and becomes blocked. A flanking creature that is blocking something does NOT trigger flanking.
- **Flanking + Bushido (Batch 2 peer)**: If a creature has both flanking and bushido, both triggers fire when it is blocked. Flanking gives the blocker -1/-1; bushido gives the attacker +N/+N. They are independent triggers.
- **Multiplayer implications**: In a multiplayer game with multiple defending players, each may declare blockers independently. Flanking triggers fire per blocker per attacker across all declarations, collected into `pending_triggers` normally via the `BlockersDeclared` event handler.

## File Change Summary

| File | Change | Lines (approx) |
|------|--------|-----------------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Flanking` variant | +5 |
| `crates/engine/src/state/hash.rs` | Add discriminant 76 for keyword, discriminant 19 for stack, PendingTrigger hash fields | +10 |
| `crates/engine/src/state/stubs.rs` | Add `is_flanking_trigger` and `flanking_blocker_id` to PendingTrigger | +12 |
| `crates/engine/src/state/stack.rs` | Add `FlankingTrigger` variant to StackObjectKind | +20 |
| `crates/engine/src/rules/abilities.rs` | Add flanking trigger dispatch in BlockersDeclared handler; add flush handler; update ~15-20 PendingTrigger literals with new fields | +80 |
| `crates/engine/src/rules/resolution.rs` | Add FlankingTrigger resolution arm; add to counter match arm | +30 |
| `tools/replay-viewer/src/view_model.rs` | Add format_keyword arm + StackObjectKind arm | +4 |
| `tools/tui/src/play/panels/stack_view.rs` | Add StackObjectKind arm | +3 |
| `crates/engine/tests/flanking.rs` | 7 unit tests | +400 |
| **Total** | | ~+560 |

# Ability Plan: Opponent-Casts Trigger

**Generated**: 2026-02-26
**CR**: 603.2 (triggered abilities), 102.2/102.3 (opponent definition)
**Priority**: P1
**Similar abilities studied**: Prowess (`ControllerCastsNoncreatureSpell` in `abilities.rs:406-438`), Ward (`SelfBecomesTargetByOpponent` in `abilities.rs:478-503`), Dies trigger (`SelfDies` in `abilities.rs:506-544`)

## CR Rule Text

**CR 603.2**: Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers. The ability doesn't do anything at this point.

**CR 603.2a**: Because they aren't cast or activated, triggered abilities can trigger even when it isn't legal to cast spells and activate abilities.

**CR 603.2c**: An ability triggers only once each time its trigger event occurs. However, it can trigger repeatedly if one event contains multiple occurrences.

**CR 603.2g**: An ability triggers only if its trigger event actually occurs. An event that's prevented or replaced won't trigger anything.

**CR 603.3a**: A triggered ability is controlled by the player who controlled its source at the time it triggered.

**CR 102.2**: In a two-player game, a player's opponent is the other player.

**CR 102.3**: In a multiplayer game between teams, a player's teammates are the other players on their team, and the player's opponents are all players not on their team.

**CR 903.2**: Commander is Free-for-All -- all other players are opponents.

## Key Edge Cases

- **Opponent-only filtering (CR 102.2/102.3)**: In Commander (FFA), all other players are opponents. The trigger must NOT fire when the controller of the triggered permanent casts a spell -- only when a different player casts. This is the exact opposite of the Prowess pattern (which fires only when the controller casts).
- **Multiple opponents in multiplayer**: When Player B casts a spell and Players A, C, D each control a Rhystic Study, all three triggers fire independently. Each trigger has its own resolution with `target 0` pointing to Player B.
- **Casting player identity must be carried**: Rhystic Study's `MayPayOrElse` needs `PlayerTarget::DeclaredTarget { index: 0 }` to resolve to the specific opponent who cast the spell (not all opponents). The trigger must carry the casting player as a target on the stack entry.
- **Storm copies are NOT cast (CR 702.40c)**: Storm copies do not trigger "whenever an opponent casts" abilities. The engine already handles this correctly -- storm copies go through `GameEvent::SpellCast` in `copy.rs:347`, but they would need to be excluded if the copy's `is_copy: true` flag is set. However, inspecting `copy.rs`, cascade-cast IS a real cast and DOES trigger. Storm copies emit `SpellCast` events, so we may need to check `is_copy` on the stack object. Actually, re-reading `copy.rs`, storm copies DO emit `SpellCast` but are marked `is_copy: true` on the StackObject. The current AnySpellCast trigger does not check `is_copy`, so storm copies already trigger Rhystic Study (which is incorrect per CR 702.40c). This is a pre-existing issue -- do NOT fix it in this scope, but note it.
- **Trigger resolves before the spell**: Rhystic Study rulings confirm the triggered ability resolves before the spell that caused it to trigger (it goes on top of the stack). This is already handled by the engine's stack ordering.
- **Trigger fires even if spell is countered**: Per Rhystic Study rulings, the trigger resolves even if the original spell is countered. This is inherent in how triggers work -- they're independent stack objects.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant -- `TriggerCondition::WheneverOpponentCastsSpell` exists at `cards/card_definition.rs:492`; `TriggerEvent` enum does NOT have an `OpponentCastsSpell` variant yet (needs to be added)
- [ ] 2. Rule enforcement -- no dispatch in `check_triggers` for opponent-casts pattern
- [ ] 3. Trigger wiring -- no enrichment in `replay_harness.rs` for `WheneverOpponentCastsSpell -> OpponentCastsSpell`
- [ ] 4. Unit tests -- existing `test_rhystic_study_draws_card_when_opponent_casts` uses `AnySpellCast` (incorrect); needs rewrite
- [ ] 5. Card definition -- Rhystic Study already defined at `definitions.rs:1110`
- [ ] 6. Game script -- none
- [ ] 7. Coverage doc update -- listed as `partial`

## Implementation Steps

### Step 1: Add `TriggerEvent::OpponentCastsSpell` Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add a new variant `OpponentCastsSpell` to the `TriggerEvent` enum (after `SelfDealsCombatDamageToPlayer` at line 135).

```rust
/// CR 603.2 / CR 102.2: Triggers when an opponent of the source's controller
/// casts a spell. "Opponent" = any player other than the source's controller
/// (CR 102.2 two-player, CR 102.3 multiplayer FFA = Commander default).
/// The opponent check is done at trigger-collection time in `rules/abilities.rs`.
OpponentCastsSpell,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant `10u8` for `TriggerEvent::OpponentCastsSpell` in the `HashInto for TriggerEvent` impl (after line 901).

```rust
// CR 603.2 / CR 102.2: Opponent-casts trigger -- discriminant 10
TriggerEvent::OpponentCastsSpell => 10u8.hash_into(hasher),
```

**Match arms**: Grep for all `match` on `TriggerEvent` to ensure exhaustiveness. The only match is in `hash.rs` (the `HashInto` impl). The `collect_triggers_for_event` function uses `==` comparison, not `match`, so no additional arms are needed there.

### Step 2: Add `triggering_player` Field to `PendingTrigger`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add a new optional field `triggering_player: Option<PlayerId>` to `PendingTrigger` (after `targeting_stack_id` at line 56). This carries the casting opponent's PlayerId so `flush_pending_triggers` can set it as `Target::Player(opponent)` at index 0 on the stack entry.

```rust
/// CR 603.2 / CR 102.2: The player whose action triggered this ability.
///
/// Populated when an `OpponentCastsSpell` trigger fires. At flush time,
/// this is converted to `Target::Player(triggering_player)` at target
/// index 0 so `DeclaredTarget { index: 0 }` can resolve to the specific
/// opponent who cast the spell (e.g. Rhystic Study's "that player pays {1}").
/// `None` for all other trigger types.
#[serde(default)]
pub triggering_player: Option<PlayerId>,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `self.triggering_player.hash_into(hasher);` to the `HashInto for PendingTrigger` impl (after `self.targeting_stack_id.hash_into(hasher);` at line 863).

### Step 3: Trigger Dispatch in `check_triggers`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add opponent-casts trigger collection inside the `GameEvent::SpellCast` arm (after the Prowess block ending at line 438). The pattern follows Ward's approach: check the opponent condition at collection time.

```rust
// CR 603.2 / CR 102.2: "Whenever an opponent casts a spell."
// Collect triggers on all permanents whose controller is NOT the caster.
// In Commander FFA (CR 903.2, CR 102.2), all other players are opponents.
{
    let opponent_sources: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield && obj.controller != *player
        })
        .map(|obj| obj.id)
        .collect();

    let pre_len = triggers.len();
    for obj_id in opponent_sources {
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::OpponentCastsSpell,
            Some(obj_id),
            None,
        );
    }
    // Tag opponent-casts triggers with the casting player so
    // flush_pending_triggers can set Target::Player at index 0.
    for t in &mut triggers[pre_len..] {
        t.triggering_player = Some(*player);
    }
}
```

**Key design decision**: Filter permanents by `controller != player` at collection time (like Ward checks `controller != targeting_controller`). This means `collect_triggers_for_event` iterates only over permanents that could validly trigger, and the `TriggerEvent::OpponentCastsSpell` equality check on the triggered ability does the rest.

### Step 4: Wire `triggering_player` in `flush_pending_triggers`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Modify the `trigger_targets` construction in `flush_pending_triggers` (around line 695) to also handle `triggering_player`. The current code only checks `targeting_stack_id` (for Ward). Extend it to also check `triggering_player`.

Replace the existing `trigger_targets` construction (lines 695-702):

```rust
let trigger_targets: Vec<SpellTarget> = if let Some(tsid) = trigger.targeting_stack_id {
    vec![SpellTarget {
        target: Target::Object(tsid),
        zone_at_cast: None,
    }]
} else if let Some(pid) = trigger.triggering_player {
    vec![SpellTarget {
        target: Target::Player(pid),
        zone_at_cast: None,
    }]
} else {
    vec![]
};
```

This ensures Rhystic Study's `PlayerTarget::DeclaredTarget { index: 0 }` resolves to the casting opponent at effect resolution time. The `zone_at_cast: None` is correct because player targets don't have a zone.

### Step 5: Enrichment in `enrich_spec_from_def`

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add a `WheneverOpponentCastsSpell -> OpponentCastsSpell` enrichment block after the `WhenDealsCombatDamageToPlayer` block (after line 500). Follow the exact same pattern as the other enrichment blocks.

```rust
// CR 603.2 / CR 102.2: Convert "Whenever an opponent casts a spell"
// card-definition triggers into runtime TriggeredAbilityDef entries so
// check_triggers can dispatch them via SpellCast events.
// The opponent check is done at trigger-collection time in abilities.rs,
// not here -- this only wires the TriggerEvent.
for ability in &def.abilities {
    if let AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WheneverOpponentCastsSpell,
        effect,
        ..
    } = ability
    {
        spec = spec.with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::OpponentCastsSpell,
            intervening_if: None,
            description: "Whenever an opponent casts a spell (CR 603.2)".to_string(),
            effect: Some(effect.clone()),
        });
    }
}
```

### Step 6: Update Existing Rhystic Study Test

**File**: `/home/airbaggie/scutemob/crates/engine/tests/effects.rs`
**Action**: Update `test_rhystic_study_draws_card_when_opponent_casts` (line 1028) to use `TriggerEvent::OpponentCastsSpell` instead of `TriggerEvent::AnySpellCast`. Also update the `payer` from `PlayerTarget::EachOpponent` to `PlayerTarget::DeclaredTarget { index: 0 }` to match the actual card definition. Update the test's doc comments to remove the "will be wired in a future milestone" note.

Changes to the test:
1. Line 1024-1027: Remove the "Opponent-only filtering will be wired" comment.
2. Line 1037: Change `trigger_on: TriggerEvent::AnySpellCast` to `trigger_on: TriggerEvent::OpponentCastsSpell`.
3. Line 1042: Change `payer: PlayerTarget::EachOpponent` to `payer: PlayerTarget::DeclaredTarget { index: 0 }`.

### Step 7: New Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/effects.rs` (add after the existing Rhystic Study test)
**Tests to write**:

1. **`test_opponent_casts_trigger_does_not_fire_on_own_spell`**
   - CR 603.2 / CR 102.2: Rhystic Study controller casts a spell; the trigger must NOT fire.
   - Setup: p1 controls Rhystic Study, p1 is active and casts Shock.
   - Assert: no AbilityTriggered event, no card drawn, hand size unchanged.
   - Pattern: follows `test_prowess_does_not_trigger_on_opponent_spell` in `tests/prowess.rs:401`.

2. **`test_opponent_casts_trigger_multiplayer_multiple_opponents`**
   - CR 603.2 / CR 102.2 / CR 903.2: In a 4-player game, p1 controls Rhystic Study. When p3 casts a spell, p1's trigger fires (not p2's or p4's permanents triggering). Verify trigger controller is p1.
   - Setup: 4-player game, p1 controls Rhystic Study, p3 casts a spell.
   - Assert: exactly 1 AbilityTriggered event with controller p1; p1 draws 1 card.

3. **`test_opponent_casts_trigger_multiple_studies`**
   - CR 603.2c: If multiple permanents each have OpponentCastsSpell, each triggers independently.
   - Setup: p1 controls 2 enchantments each with OpponentCastsSpell trigger (e.g., two Rhystic Studies). p2 casts a spell.
   - Assert: 2 AbilityTriggered events; p1 draws 2 cards.

4. **`test_opponent_casts_trigger_carries_casting_player_as_target`**
   - CR 603.2: Verify that the stack entry for the triggered ability has the casting player as `Target::Player(p2)` at index 0.
   - Setup: p1 controls Rhystic Study, p2 casts a spell.
   - Assert: after triggers are flushed, the triggered ability's stack object has `targets[0].target == Target::Player(p2)`.

### Step 8: Card Definition (already exists)

**Card**: Rhystic Study at `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs:1110`
**Status**: Already defined with `TriggerCondition::WheneverOpponentCastsSpell` and correct `MayPayOrElse` effect structure. No changes needed to the card definition.

### Step 9: Game Script (later phase)

**Suggested scenario**: Rhystic Study in 4-player Commander. Player 2 casts a spell, Player 1's Rhystic Study triggers and draws a card (payment not interactive, always draws). Then Player 1 casts their own spell -- trigger does NOT fire.
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Script name**: `058_rhystic_study_opponent_casts.json`

### Step 10: Coverage Doc Update

**File**: `/home/airbaggie/scutemob/docs/mtg-engine-ability-coverage.md`
**Action**: Update the "Opponent-casts trigger" row from `partial` to `validated` once all tests pass. Update the "P1 Gaps" section to remove item 1.

## Interactions to Watch

- **Prowess and Opponent-Casts on the same SpellCast event**: Both Prowess (`ControllerCastsNoncreatureSpell`) and Opponent-Casts (`OpponentCastsSpell`) fire from the same `GameEvent::SpellCast` arm. Prowess fires on the caster's permanents; Opponent-Casts fires on non-caster permanents. They are complementary and do not conflict. If Player A controls a Prowess creature and Player B controls a Rhystic Study, and Player A casts a noncreature spell: Prowess triggers on Player A's creature, and Rhystic Study triggers on Player B's enchantment. Both work.
- **AnySpellCast still exists**: The `AnySpellCast` trigger event is NOT removed. It serves a different purpose (e.g., future cards like "Whenever any player casts a spell"). Opponent-Casts is a new, separate dispatch path.
- **Storm copy interaction**: As noted in Edge Cases, storm copies emit `SpellCast` events and would incorrectly trigger `OpponentCastsSpell`. This is a pre-existing AnySpellCast issue, not introduced by this change. Deferred to a future `is_copy` filter on the SpellCast event in `check_triggers`.
- **Cascade interaction**: Cascade's free cast IS a real cast and correctly triggers `OpponentCastsSpell`. The cascade spell emits its own `SpellCast` event.
- **`MayPayOrElse` resolution**: The effect currently always fires the `or_else` branch (line 991-993 of `effects/mod.rs`). Interactive payment choice is deferred. With `triggering_player` wiring, `DeclaredTarget { index: 0 }` now correctly resolves to the casting opponent, which is the correct target for when interactive payment is eventually implemented.
- **Multiplayer APNAP ordering**: If multiple players each control opponent-casts triggers and a single spell is cast, all triggers are collected and placed on the stack in APNAP order (CR 603.3b). This is already handled by `flush_pending_triggers`.

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/game_object.rs` | Add `TriggerEvent::OpponentCastsSpell` |
| `crates/engine/src/state/stubs.rs` | Add `triggering_player: Option<PlayerId>` to `PendingTrigger` |
| `crates/engine/src/state/hash.rs` | Add hash discriminants for new variant + field |
| `crates/engine/src/rules/abilities.rs` | Add dispatch in `check_triggers` SpellCast arm + wire `triggering_player` in `flush_pending_triggers` |
| `crates/engine/src/testing/replay_harness.rs` | Add `WheneverOpponentCastsSpell -> OpponentCastsSpell` enrichment |
| `crates/engine/tests/effects.rs` | Update existing test + add 4 new tests |
| `docs/mtg-engine-ability-coverage.md` | Update status to `validated` |

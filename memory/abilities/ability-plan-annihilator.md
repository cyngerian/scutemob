# Ability Plan: Annihilator

**Generated**: 2026-02-26
**CR**: 702.86
**Priority**: P2
**Similar abilities studied**: Exalted (CR 702.83) -- triggered keyword, combat-triggered, builder.rs auto-generation pattern

## CR Rule Text

702.86. Annihilator

702.86a Annihilator is a triggered ability. "Annihilator N" means "Whenever this creature attacks, defending player sacrifices N permanents."

702.86b If a creature has multiple instances of annihilator, each triggers separately.

### Supporting Rules

CR 508.5: If an ability of an attacking creature refers to a defending player, then the defending player it's referring to is the player that creature is attacking, the controller of the planeswalker that creature is attacking, or the protector of the battle that creature is attacking.

CR 508.5a: In a multiplayer game, any rule, object, or effect that refers to a "defending player" refers to one specific defending player, not to all of the defending players.

CR 508.1m: Any abilities that trigger on attackers being declared trigger.

CR 508.2b: Any abilities that triggered on attackers being declared are put onto the stack before the active player gets priority.

## Key Edge Cases

- **Multiple instances trigger separately** (CR 702.86b): A creature with "Annihilator 2" and "Annihilator 4" triggers BOTH -- defending player sacrifices 2 permanents from one trigger and 4 from the other.
- **Trigger resolves before blockers declared** (rulings on Artisan of Kozilek, Emrakul, etc.): Annihilator abilities trigger and resolve during the declare attackers step. Creatures sacrificed this way cannot block.
- **Attacking a planeswalker** (ruling on Ulamog's Crusher): If a creature with annihilator attacks a planeswalker, and the defending player sacrifices that planeswalker, the attacking creature continues to attack (it may be blocked, but if unblocked it deals no combat damage to anything).
- **Defending player in multiplayer** (CR 508.5, 508.5a): In a 4-player Commander game, "defending player" means the specific player that creature is attacking. If creature A attacks Player 2 and creature B attacks Player 3, each annihilator trigger targets its own defending player.
- **Defending player chooses which permanents to sacrifice**: The defending player chooses which of THEIR permanents to sacrifice (any permanent, not just creatures). They can sacrifice lands, artifacts, enchantments, etc.
- **Fewer permanents than N**: If the defending player controls fewer than N permanents when annihilator resolves, they sacrifice all permanents they control.
- **Player sacrifice is not "destroy"**: Sacrificing ignores indestructible. A permanent with indestructible can be sacrificed by annihilator.

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

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Annihilator(u32)` variant after `Exalted` (line ~263)
**Pattern**: Follow `KeywordAbility::Dredge(u32)` at line 230 and `KeywordAbility::Ward(u32)` at line 182 -- parameterized keywords carry a `u32` payload.

```rust
/// CR 702.86: Annihilator N -- "Whenever this creature attacks, defending
/// player sacrifices N permanents."
///
/// Implemented as a triggered ability. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// Multiple instances each trigger separately (CR 702.86b).
Annihilator(u32),
```

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` `impl HashInto for KeywordAbility` block (after line 350, Exalted discriminant 34):

```rust
// Annihilator (discriminant 35) -- CR 702.86
KeywordAbility::Annihilator(n) => {
    35u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**Match arms to update** (grep for exhaustive `match` on `KeywordAbility`):

1. `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs` line ~598 in `format_keyword`:
   ```rust
   KeywordAbility::Annihilator(n) => format!("Annihilator {n}"),
   ```

### Step 2: New Effect -- `SacrificePermanents`

**IMPORTANT**: The `Effect` enum currently has NO "target player sacrifices N permanents" variant. This must be added as a new effect.

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `SacrificePermanents` variant to the `Effect` enum (after `Goad` at line ~350):

```rust
/// CR 701.17a: A player sacrifices a permanent they control.
///
/// The specified player must sacrifice `count` permanents they control.
/// If they control fewer than `count` permanents, they sacrifice all
/// permanents they control. The player chooses which permanents to
/// sacrifice (deterministic fallback: sacrifice in ObjectId ascending
/// order). Sacrifice ignores indestructible (CR 701.17a).
SacrificePermanents {
    player: PlayerTarget,
    count: EffectAmount,
},
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Add effect execution handler for `SacrificePermanents` in `execute_effect_inner`. The implementation must:

1. Resolve `player` to a concrete `PlayerId` using `resolve_player_target_list`.
2. Resolve `count` to a concrete integer using `resolve_amount`.
3. Collect all permanents controlled by that player on the battlefield (sorted by ObjectId for determinism).
4. Take `min(count, permanents.len())` permanents (deterministic: first N by ObjectId ascending -- interactive choice deferred to M10+).
5. For each sacrificed permanent, move it to its owner's graveyard via `move_object_to_zone` and emit appropriate events (similar to `DestroyPermanent` but without the indestructible check).

**Pattern**: Follow `Effect::DestroyPermanent` execution in `effects/mod.rs` for zone movement, but skip the indestructible check (sacrifice is not destruction -- CR 701.17).

**CR**: CR 701.17a -- "To sacrifice a permanent, its controller moves it from the battlefield directly to its owner's graveyard. A player can't sacrifice something that isn't a permanent, or something they don't control."

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add the new `Effect::SacrificePermanents` variant to the `HashInto for Effect` implementation. Grep for `Effect::Goad` in hash.rs to find the insertion point and the next available discriminant.

### Step 3: Trigger Wiring (Annihilator as triggered ability)

Annihilator is a "Whenever this creature attacks" triggered ability. The engine already has `TriggerEvent::SelfAttacks` which fires for each creature declared as an attacker (in the `AttackersDeclared` event handler in `abilities.rs` at line ~656). The trigger wiring has TWO parts:

#### Part 3a: Builder auto-generation of TriggeredAbilityDef

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: In the keyword-to-triggered-ability translation block (after the Exalted block at line ~416), add Annihilator auto-generation:

```rust
// CR 702.86a: Annihilator N -- "Whenever this creature attacks, defending
// player sacrifices N permanents."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.86b).
// The effect targets the defending player (DeclaredTarget { index: 0 }).
if let KeywordAbility::Annihilator(n) = kw {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacks,
        intervening_if: None,
        description: format!(
            "Annihilator {n} (CR 702.86a): Whenever this creature attacks, \
             defending player sacrifices {n} permanents."
        ),
        effect: Some(Effect::SacrificePermanents {
            player: PlayerTarget::DeclaredTarget { index: 0 },
            count: EffectAmount::Fixed(*n as i32),
        }),
    });
}
```

**Pattern**: Follows the Exalted block at line ~402 which also auto-generates from keyword to TriggeredAbilityDef.

#### Part 3b: PendingTrigger needs to carry the defending player

**Critical design issue**: When `SelfAttacks` triggers fire (line ~658 in `abilities.rs`), the `collect_triggers_for_event` function collects triggers for each attacker. But the trigger does NOT currently carry information about which player the creature is attacking. Annihilator needs this because "defending player" varies per-attacker in multiplayer (CR 508.5a).

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add a new field to `PendingTrigger`:

```rust
/// CR 508.5 / CR 702.86a: The defending player for this attack trigger.
///
/// Populated when a SelfAttacks trigger fires. At flush time, this PlayerId
/// is set as Target::Player at index 0 so the annihilator effect's
/// PlayerTarget::DeclaredTarget { index: 0 } resolves to the correct
/// defending player. Also used by any future "whenever this creature attacks,
/// [effect on defending player]" triggers.
/// None for all non-attack trigger types.
#[serde(default)]
pub defending_player_id: Option<PlayerId>,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `defending_player_id` to the `HashInto for PendingTrigger` implementation.

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action 1**: In the `AttackersDeclared` handler (line ~652), after `collect_triggers_for_event` for `SelfAttacks`, tag the collected triggers with the defending player. The `AttackersDeclared` event contains `attackers: Vec<(ObjectId, AttackTarget)>` -- extract the AttackTarget for each attacker and resolve it to a PlayerId:

```rust
// CR 702.86a / CR 508.5: Tag SelfAttacks triggers with the defending
// player so annihilator (and future attack triggers) can resolve
// "defending player" correctly.
for (attacker_id, attack_target) in attackers {
    let pre_len = triggers.len();
    collect_triggers_for_event(
        state,
        &mut triggers,
        TriggerEvent::SelfAttacks,
        Some(*attacker_id),
        None,
    );
    // Resolve defending player from AttackTarget (CR 508.5).
    let defending_player = match attack_target {
        AttackTarget::Player(pid) => Some(*pid),
        AttackTarget::Planeswalker(pw_id) => {
            state.objects.get(pw_id).map(|obj| obj.controller)
        }
    };
    for t in &mut triggers[pre_len..] {
        t.defending_player_id = defending_player;
    }
}
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action 2**: In `flush_pending_triggers` (line ~962), add a branch for `defending_player_id` in the trigger_targets construction (after the `exalted_attacker_id` branch at line ~983). The defending player becomes `Target::Player` at index 0:

```rust
} else if let Some(dp) = trigger.defending_player_id {
    // CR 702.86a / CR 508.5: Annihilator triggers carry the defending
    // player ID. Set as Target::Player at index 0 so
    // PlayerTarget::DeclaredTarget { index: 0 } resolves correctly.
    vec![SpellTarget {
        target: Target::Player(dp),
        zone_at_cast: None,
    }]
}
```

**IMPORTANT ordering note**: This branch must come BEFORE the `exalted_attacker_id` branch, because a trigger could theoretically have both (though currently impossible). Place it after `triggering_player` and before `exalted_attacker_id`.

#### Part 3c: Replay harness enrichment

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: The replay harness `enrich_spec_from_def` function already translates `TriggerCondition::WhenAttacks` triggers at line ~558. For Annihilator, the keyword auto-generation in `builder.rs` handles this. However, we need to verify that objects constructed via `enrich_spec_from_def` also get annihilator wired correctly.

The `enrich_spec_from_def` function at line ~455 calls `spec.with_keyword(kw.clone())` for each keyword. The `ObjectSpec::build_object` in `builder.rs` then processes the keyword list and generates the TriggeredAbilityDef. So no additional replay harness change is needed for the keyword path -- `builder.rs` handles it.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/annihilator.rs` (new file)
**Pattern**: Follow `/home/airbaggie/scutemob/crates/engine/tests/exalted.rs` for structure, helpers, and test style.

**Tests to write**:

1. `test_annihilator_basic_sacrifice_on_attack` -- CR 702.86a
   - Setup: P1 has a creature with `Annihilator(2)` on battlefield. P2 has 3 permanents.
   - P1 declares the creature as attacker targeting P2.
   - Trigger fires, goes on stack. Both players pass. Trigger resolves.
   - Assert: P2 has 1 permanent remaining (2 were sacrificed -- deterministic: lowest ObjectId first).
   - Assert: AbilityTriggered event was emitted.

2. `test_annihilator_defending_player_sacrifices_not_attacker_controller` -- CR 702.86a
   - Setup: P1 has annihilator creature. P1 also has 3 permanents. P2 has 2 permanents.
   - P1 attacks P2.
   - Assert: P2's permanents are sacrificed, NOT P1's.

3. `test_annihilator_fewer_permanents_than_n` -- CR 701.17a edge case
   - P2 has only 1 permanent. Annihilator 2 resolves.
   - Assert: P2 sacrifices their only permanent (not an error -- sacrifice min(N, count)).

4. `test_annihilator_multiple_instances_trigger_separately` -- CR 702.86b
   - Creature has both `Annihilator(2)` and `Annihilator(1)`.
   - Declare as attacker. Two triggers on stack (one for each instance).
   - Resolve both. P2 loses 3 permanents total (2 + 1).

5. `test_annihilator_multiplayer_correct_defending_player` -- CR 508.5a
   - 4-player game. P1 attacks P2 with annihilator creature A, and attacks P3 with a non-annihilator creature B.
   - Assert: Annihilator trigger targets P2 specifically, NOT P3.

6. `test_annihilator_sacrifice_ignores_indestructible` -- CR 701.17a
   - P2 has an indestructible permanent. Annihilator resolves.
   - Assert: The indestructible permanent is sacrificed (goes to graveyard).

7. `test_annihilator_attacking_planeswalker_defending_player` -- CR 508.5
   - P2 controls a planeswalker. P1 attacks the planeswalker with an annihilator creature.
   - Annihilator's "defending player" is P2 (the controller of the planeswalker).
   - Assert: P2 sacrifices permanents.

8. `test_annihilator_zero_permanents_no_error` -- Edge case
   - P2 controls no permanents. Annihilator resolves.
   - Assert: No error, no sacrifice (N > 0 permanents = 0).

### Step 5: Card Definition (later phase)

**Suggested card**: Ulamog's Crusher
- Simple: 8 generic mana, 8/8, Annihilator 2, attacks each combat if able.
- No additional cast triggers or complex abilities.
- Oracle text: "Annihilator 2 (Whenever this creature attacks, defending player sacrifices two permanents of their choice.) / This creature attacks each combat if able."

**Card lookup**: Use `card-definition-author` agent with name "Ulamog's Crusher".

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs`

### Step 6: Game Script (later phase)

**Suggested scenario**: 4-player Commander game. P1 casts Ulamog's Crusher, then on next turn attacks P2. Annihilator 2 trigger fires. P2 sacrifices 2 permanents. Verify P2's battlefield count decreases by 2. Then P1 attacks P3 on a later turn to verify multiplayer defending player resolution.

**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

- **SacrificePermanents deterministic ordering**: Without interactive player choice (deferred to M10+), the engine must sacrifice in a deterministic order. Use ObjectId ascending (consistent with other deterministic fallbacks like Scry bottom-ordering).
- **"Sacrifice" vs "Destroy"**: Sacrifice is NOT destruction. It bypasses indestructible. It IS a zone move to graveyard, so "dies" triggers still fire. Commander zone-change SBA (CR 903.9a) applies to sacrificed commanders.
- **Token sacrifice**: Tokens can be sacrificed. They briefly exist in the graveyard (triggering "when dies") before ceasing to exist as an SBA (CR 704.5d).
- **The new `SacrificePermanents` effect is a general-purpose addition**: Once implemented, it can be reused for Annihilator, Fleshbag Marauder, Diabolic Edict, etc. It is not Annihilator-specific.
- **PendingTrigger.defending_player_id is a general-purpose addition**: Once implemented, any future "whenever this attacks, [effect on defending player]" ability can use it. Not Annihilator-specific.
- **Combat state must be populated**: The test setup must use `at_step(Step::DeclareAttackers)` so the combat state is initialized and attackers can be declared.
- **Priority must pass for triggers to resolve**: Annihilator triggers go on the stack during DeclareAttackers. Both (all) players must pass priority for the trigger to resolve BEFORE moving to DeclareBlockers. This is consistent with existing SelfAttacks trigger behavior.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Annihilator(u32)` |
| `crates/engine/src/state/hash.rs` | Add hash discriminant 35 for Annihilator; add `defending_player_id` to PendingTrigger hash; add SacrificePermanents to Effect hash |
| `crates/engine/src/state/stubs.rs` | Add `defending_player_id: Option<PlayerId>` to PendingTrigger |
| `crates/engine/src/state/builder.rs` | Auto-generate TriggeredAbilityDef from Annihilator keyword |
| `crates/engine/src/cards/card_definition.rs` | Add `Effect::SacrificePermanents` variant |
| `crates/engine/src/effects/mod.rs` | Implement SacrificePermanents execution |
| `crates/engine/src/rules/abilities.rs` | Tag SelfAttacks triggers with defending_player_id; add defending_player_id branch in flush_pending_triggers |
| `tools/replay-viewer/src/view_model.rs` | Add Annihilator to format_keyword |
| `crates/engine/tests/annihilator.rs` | New test file: 8 tests |

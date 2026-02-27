# Ability Plan: Battle Cry

**Generated**: 2026-02-27
**CR**: 702.91
**Priority**: P3
**Similar abilities studied**: Exalted (CR 702.83, `builder.rs:397-416`, `abilities.rs:709-738`, `tests/exalted.rs`), Annihilator (CR 702.86, `builder.rs:418-436`, `abilities.rs:680-707`, `tests/annihilator.rs`)

## CR Rule Text

```
702.91. Battle Cry

702.91a Battle cry is a triggered ability. "Battle cry" means "Whenever this
        creature attacks, each other attacking creature gets +1/+0 until end
        of turn."

702.91b If a creature has multiple instances of battle cry, each triggers
        separately.
```

## Key Edge Cases

1. **"Each other attacking creature"** -- the bonus applies to all attacking creatures
   EXCEPT the source creature with battle cry. If the battle cry creature is the only
   attacker, no creatures receive the bonus (CR 702.91a "each other").

2. **Multiple instances stack** -- CR 702.91b says each instance triggers separately.
   A creature with two instances of battle cry generates two triggers; each resolves
   independently, giving every other attacker +2/+0 total.

3. **Hero of Bladehold interaction** (ruling 2011-06-01): "If the token-creating ability
   resolves first, the tokens each get +1/+0 until end of turn from the battle cry
   ability." This confirms that battle cry applies to creatures that are attacking at
   resolution time, not just those declared as attackers. Tokens put onto the battlefield
   "tapped and attacking" ARE attacking creatures and benefit from battle cry if the
   battle cry trigger has not yet resolved.

4. **Power-only modification** -- +1/+0 is `ModifyPower(1)`, NOT `ModifyBoth(1)`.
   Toughness is unaffected.

5. **Multiplayer** -- in a 4-player Commander game, the battle cry creature and other
   attackers may attack different opponents. All other attacking creatures still get
   +1/+0 regardless of which player each is attacking. "Other attacking creature" has
   no player restriction beyond being controlled by the same player (implied by combat
   rules: only the active player's creatures can be attacking).

6. **Bonus expires at end of turn** -- the continuous effect has `UntilEndOfTurn`
   duration. Standard cleanup-step expiry (CR 514.2).

7. **Battle cry creature removed before resolution** -- if the battle cry creature is
   removed from combat or killed before its trigger resolves, the trigger still resolves
   (it's already on the stack). The effect applies to whatever creatures are currently
   attacking at resolution time. The source creature being gone doesn't matter because
   `ForEach` iterates combat state, not source state.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Design Decision: ForEach over Attacking Creatures

Battle Cry differs from Exalted and Annihilator in a critical way:

- **Exalted** targets a single creature (the lone attacker) via `DeclaredTarget { index: 0 }`.
- **Annihilator** targets a single player (the defending player) via `DeclaredTarget { index: 0 }`.
- **Battle Cry** affects ALL other attacking creatures -- a variable-size set.

The correct approach is `Effect::ForEach` with a new `ForEachTarget::EachOtherAttackingCreature` variant:

```rust
Effect::ForEach {
    over: ForEachTarget::EachOtherAttackingCreature,
    effect: Box::new(Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyPower(1),
            filter: CEFilter::DeclaredTarget { index: 0 },
            duration: CEDuration::UntilEndOfTurn,
        }),
    }),
}
```

When `ForEach` iterates, it creates a synthetic `EffectContext` with
`Target::Object(id)` at index 0 for each iterated creature. The inner
`ApplyContinuousEffect` resolves `DeclaredTarget { index: 0 }` to
`SingleObject(id)`, creating one `UntilEndOfTurn` continuous effect per other
attacker.

The `EachOtherAttackingCreature` variant queries `state.combat.attackers` and
filters out `ctx.source` (the battle cry creature). This correctly handles:
- Hero of Bladehold tokens (they're in `state.combat.attackers` if created before resolution)
- Multiplayer (all of the active player's attackers, regardless of target)
- Source creature removed before resolution (it won't be in attackers, filter is moot)

The trigger itself uses `TriggerEvent::SelfAttacks` (same as Annihilator), which
fires for each creature declared as an attacker. The `SelfAttacks` event handling
in `abilities.rs:688-706` already iterates per-attacker and tags with
`defending_player_id`. Battle Cry doesn't need `defending_player_id` -- it just
needs the source ObjectId, which is implicit in `ctx.source`.

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::BattleCry` variant after `Crew(u32)` (line ~309).
**Pattern**: Follow `KeywordAbility::Exalted` at line 256.

```rust
/// CR 702.91: Battle Cry -- "Whenever this creature attacks, each other
/// attacking creature gets +1/+0 until end of turn."
///
/// Implemented as a triggered ability. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// Multiple instances each trigger separately (CR 702.91b).
BattleCry,
```

**Hash**: `crates/engine/src/state/hash.rs`
Add after `Crew` discriminant (line ~368):
```rust
// BattleCry (discriminant 41) -- CR 702.91
KeywordAbility::BattleCry => 41u8.hash_into(hasher),
```

**Match arms to update**: Grep for exhaustive `match` on `KeywordAbility` and add
`BattleCry` arm. Known locations:
- `hash.rs` (HashInto impl) -- covered above
- `view_model.rs` (keyword_display) -- add `KeywordAbility::BattleCry => "Battle Cry".to_string()`

### Step 2: ForEachTarget Variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `ForEachTarget::EachOtherAttackingCreature` variant after
`EachCardInAllGraveyards` (line ~747).

```rust
/// Every other attacking creature (excluding the source of the effect).
///
/// Used by Battle Cry (CR 702.91a): "each other attacking creature gets
/// +1/+0 until end of turn." Queries `state.combat.attackers` and
/// excludes `ctx.source`.
EachOtherAttackingCreature,
```

**Hash**: `crates/engine/src/state/hash.rs`
Add after `EachCardInAllGraveyards` discriminant 6 (line ~1973):
```rust
ForEachTarget::EachOtherAttackingCreature => 7u8.hash_into(hasher),
```

**Collection logic**: `crates/engine/src/effects/mod.rs`
Add arm in `collect_for_each` function (after line ~1993):

```rust
// CR 702.91a: All attacking creatures except the source (battle cry source).
ForEachTarget::EachOtherAttackingCreature => {
    if let Some(ref combat) = state.combat {
        combat.attackers.keys()
            .filter(|&&id| id != ctx.source)
            .copied()
            .collect()
    } else {
        vec![]
    }
}
```

### Step 3: Builder Trigger Generation

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add battle cry trigger generation after the Annihilator block (line ~436).
**Pattern**: Follow Annihilator at line 418-436 (same `SelfAttacks` trigger event).

```rust
// CR 702.91a: Battle Cry -- "Whenever this creature attacks, each
// other attacking creature gets +1/+0 until end of turn."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.91b).
// The ForEach iterates over all other attacking creatures at resolution
// time and applies a +1/+0 ModifyPower continuous effect to each.
if matches!(kw, KeywordAbility::BattleCry) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacks,
        intervening_if: None,
        description: "Battle Cry (CR 702.91a): Whenever this creature attacks, \
                      each other attacking creature gets +1/+0 until end of turn."
            .to_string(),
        effect: Some(Effect::ForEach {
            over: ForEachTarget::EachOtherAttackingCreature,
            effect: Box::new(Effect::ApplyContinuousEffect {
                effect_def: Box::new(ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: CEFilter::DeclaredTarget { index: 0 },
                    duration: CEDuration::UntilEndOfTurn,
                }),
            }),
        }),
    });
}
```

**Note**: No changes needed to `abilities.rs` trigger collection. The `SelfAttacks`
event handling at line 688-706 already fires `collect_triggers_for_event` for each
attacker and tags with `defending_player_id`. Battle Cry triggers will be collected
via the same path. The `defending_player_id` is set on the pending trigger but is
not used by battle cry's `ForEach` effect (it doesn't reference
`DeclaredTarget { index: 0 }` at the trigger-target level -- the ForEach handles
target iteration internally).

**However**: There is a subtlety. The flush_pending_triggers function at line 1059
checks `defending_player_id` and sets `Target::Player(dp)` at index 0. This means
the Battle Cry trigger's context will have `targets[0] = Target::Player(defending_player)`.
The `ForEach` iterates and creates synthetic contexts with `Target::Object(id)` at
index 0 for each attacker, which is correct -- the outer target is overridden by
ForEach's inner context. This works correctly because `ForEach` creates a brand new
`inner_ctx` (line 1056-1064 in effects/mod.rs), not modifying the original context.

### Step 4: View Model Update

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add display arm in `keyword_display` function (after line 607).

```rust
KeywordAbility::BattleCry => "Battle Cry".to_string(),
```

### Step 5: Unit Tests

**File**: `crates/engine/tests/battle_cry.rs` (new file)
**Tests to write**:

1. **`test_battle_cry_basic_other_attackers_get_plus_one_power`**
   CR 702.91a -- Battle cry source (3/3) + two other attackers (2/2 each) attack.
   After trigger resolves, the two other attackers are 3/2 (+1 power, same toughness).
   Source creature is still 3/3 (does NOT get its own bonus).

2. **`test_battle_cry_source_only_attacker_no_bonus`**
   CR 702.91a "each other" -- If the battle cry creature is the only attacker,
   no creatures receive the bonus. Stack should have trigger, but resolution is a
   no-op (ForEach over empty set).

3. **`test_battle_cry_does_not_affect_toughness`**
   CR 702.91a "+1/+0" -- Verify toughness is unchanged after resolution.
   Attacker starts as 2/3, ends as 3/3 (power +1, toughness unchanged).

4. **`test_battle_cry_multiple_instances_stack`**
   CR 702.91b -- Creature with two battle cry instances (via two `.with_keyword` calls)
   attacks with two other creatures. Two triggers fire. After both resolve, each other
   attacker has +2/+0.

5. **`test_battle_cry_multiplayer_all_attackers_benefit`**
   Multiplayer 4-player: P1 has battle cry creature + two other creatures. One attacks
   P2, one attacks P3, battle cry creature attacks P2. After trigger resolves, both
   other attackers get +1/+0 regardless of which player they target.

6. **`test_battle_cry_bonus_expires_at_end_of_turn`**
   CR 514.2 -- After `expire_end_of_turn_effects`, the +1/+0 bonus is removed.
   Attacker returns to its printed power.

7. **`test_battle_cry_does_not_trigger_on_opponent_attack`**
   CR 702.91a "this creature attacks" -- SelfAttacks only fires on the battle cry
   creature itself. If an opponent declares attackers, P1's battle cry creature does
   NOT trigger. (This is inherent to `SelfAttacks` but worth testing explicitly.)

**Pattern**: Follow `tests/exalted.rs` structure (helpers: `find_object`, `pass_all`).
Import `calculate_characteristics` for P/T verification.

### Step 6: Card Definition (later phase)

**Suggested card**: Goblin Wardriver
- **Mana cost**: {R}{R}
- **Type**: Creature -- Goblin Warrior
- **P/T**: 2/2
- **Abilities**: Battle Cry only (simplest battle cry card)
- **Oracle text**: "Battle cry (Whenever this creature attacks, each other attacking creature gets +1/+0 until end of turn.)"
- **Card lookup**: use `card-definition-author` agent

Goblin Wardriver is preferred over Signal Pest (which also has "can't be blocked except
by flying/reach" evasion, requiring additional work) and Hero of Bladehold (which has
a token-creating triggered ability on attack, adding complexity).

### Step 7: Game Script (later phase)

**Suggested scenario**: Goblin Wardriver attacks with two other creatures. Battle cry
trigger resolves, both other creatures get +1/+0. Combat damage is dealt at increased
power. Verify life totals.
**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

1. **ForEach + ApplyContinuousEffect interaction**: The `ForEach` combinator creates a
   synthetic `EffectContext` with `Target::Object(id)` at index 0. The inner
   `ApplyContinuousEffect` resolves `CEFilter::DeclaredTarget { index: 0 }` to
   `CEFilter::SingleObject(id)`. This pattern is already used (e.g., Cyclonic Rift
   overloaded via ForEach + Bounce), so it should work correctly.

2. **SelfAttacks trigger timing**: Battle Cry fires at `DeclareAttackers` (CR 508.1m).
   The trigger goes on the stack and resolves before blockers are declared. At
   resolution time, `state.combat.attackers` is fully populated. If a creature was
   removed from combat before resolution (unlikely in the DeclareAttackers → priority
   window), it would no longer be in the attackers map and wouldn't receive the bonus.

3. **Hero of Bladehold tokens**: Tokens put onto the battlefield "tapped and attacking"
   are added to `state.combat.attackers` by the `PutTokensOntoBattlefieldAttacking`
   effect. If the token-creating ability resolves first (before battle cry), the tokens
   will be in the attackers map when `EachOtherAttackingCreature` iterates. This
   matches the 2011-06-01 ruling. (Token creation attacking is a future card feature
   -- not needed for the initial implementation with Goblin Wardriver.)

4. **Exhaustive match**: Adding `ForEachTarget::EachOtherAttackingCreature` and
   `KeywordAbility::BattleCry` will cause compile errors at every exhaustive match
   site. All sites must be updated:
   - `hash.rs`: HashInto for both enums
   - `view_model.rs`: keyword_display
   - `effects/mod.rs`: collect_for_each
   - Any other exhaustive matches on `ForEachTarget` or `KeywordAbility`

## File Change Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::BattleCry` variant |
| `crates/engine/src/state/hash.rs` | Add hash discriminant 41 for BattleCry; discriminant 7 for EachOtherAttackingCreature |
| `crates/engine/src/cards/card_definition.rs` | Add `ForEachTarget::EachOtherAttackingCreature` variant |
| `crates/engine/src/state/builder.rs` | Add battle cry trigger generation block |
| `crates/engine/src/effects/mod.rs` | Add `EachOtherAttackingCreature` arm in `collect_for_each` |
| `tools/replay-viewer/src/view_model.rs` | Add `BattleCry` arm in `keyword_display` |
| `crates/engine/tests/battle_cry.rs` | New test file with 7 tests |

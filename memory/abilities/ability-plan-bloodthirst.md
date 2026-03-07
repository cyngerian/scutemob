# Ability Plan: Bloodthirst

**Generated**: 2026-03-06
**CR**: 702.54
**Priority**: P4
**Similar abilities studied**: Amplify (CR 702.38) -- ETB counter placement in `resolution.rs:679-776` and `lands.rs:217-296`; Spectacle (CR 702.137) -- `life_lost_this_turn` per-player tracking in `player.rs:131`

## CR Rule Text

702.54. Bloodthirst

702.54a Bloodthirst is a static ability. "Bloodthirst N" means "If an opponent was dealt damage this turn, this permanent enters with N +1/+1 counters on it."

702.54b "Bloodthirst X" is a special form of bloodthirst. "Bloodthirst X" means "This permanent enters with X +1/+1 counters on it, where X is the total damage your opponents have been dealt this turn."

702.54c If an object has multiple instances of bloodthirst, each applies separately.

## Key Edge Cases

- **Damage vs. life loss (Blood Seeker ruling 2011-09-22)**: "Life loss is not the same as damage. Blood Seeker's ability will not cause creatures with bloodthirst to enter with +1/+1 counters." The existing `life_lost_this_turn` field is INSUFFICIENT -- Bloodthirst requires tracking actual damage dealt (which includes infect damage that gives poison instead of life loss).
- **Any opponent, any source (Indoraptor ruling 2023-11-10)**: "counts all damage dealt to your opponents this turn, not just damage dealt by sources you control." The check is: was ANY opponent dealt ANY damage this turn, from any source.
- **Multiple instances cumulative (CR 702.54c, Bloodlord of Vaasgoth ruling 2011-09-22)**: Each Bloodthirst N instance adds N counters independently if the condition is met.
- **Bloodthirst X special form (CR 702.54b)**: X equals the TOTAL damage all opponents have been dealt this turn. This requires tracking the actual damage amount, not just a boolean.
- **Eliminated players are not opponents**: Per CR 800.4a / CR 102.3, eliminated players (has_lost or has_conceded) are not opponents. Their damage totals should not count. Follow the Spectacle pattern at `casting.rs:1436-1437`.
- **Leyline of Lightning interaction (ruling 2006-02-01)**: Damage from Leyline's triggered ability (which fires on casting the Bloodthirst creature) resolves BEFORE the creature spell resolves, enabling Bloodthirst. This is naturally correct since the trigger resolves before the spell.
- **Put onto battlefield vs. cast (Bloodlord ruling 2011-09-22)**: Bloodthirst is a static ability that applies whenever the permanent enters, whether cast or put directly onto the battlefield. (Unlike Amplify which involves revealing from hand during casting, Bloodthirst just checks a game-state condition.)

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (n/a -- static ability, not triggered)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant + Infrastructure

#### 1a. KeywordAbility variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Bloodthirst(u32)` variant after `Amplify(u32)` at line ~1126
**Discriminant**: KW 123
**Pattern**: Follow `KeywordAbility::Amplify(u32)` at line 1126

```
/// CR 702.54: Bloodthirst N -- "If an opponent was dealt damage this turn,
/// this permanent enters with N +1/+1 counters on it."
///
/// Static ability / ETB replacement effect (CR 614.1c). Multiple instances
/// work separately (CR 702.54c).
///
/// Discriminant 123.
Bloodthirst(u32),
```

#### 1b. Hash

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm after the Amplify arm (~line 618)
**Pattern**: Follow `KeywordAbility::Amplify(n)` hash pattern

```
// Bloodthirst (discriminant 123) -- CR 702.54
KeywordAbility::Bloodthirst(n) => {
    123u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

#### 1c. Replay viewer keyword display

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add display arm after `Amplify` arm (~line 830)

```
KeywordAbility::Bloodthirst(n) => format!("Bloodthirst {n}"),
```

#### 1d. New PlayerState field: `damage_received_this_turn`

**File**: `crates/engine/src/state/player.rs`
**Action**: Add `pub damage_received_this_turn: u32` field to `PlayerState` after `life_lost_this_turn` (~line 131)
**CR**: 702.54a -- Bloodthirst checks "if an opponent was dealt damage this turn", which requires tracking actual damage (not just life loss, since infect deals damage without causing life loss)

```
/// Total damage dealt to this player this turn (CR 120.2, CR 702.54a).
///
/// Incremented whenever this player is dealt damage (combat or non-combat,
/// including infect damage that gives poison instead of life loss).
/// Reset to 0 at the start of each turn in `reset_turn_state`.
/// Used by Bloodthirst to check if an opponent was dealt damage this turn
/// and by Bloodthirst X to determine the total.
#[serde(default)]
pub damage_received_this_turn: u32,
```

#### 1e. Initialize in builder

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add `damage_received_this_turn: 0,` to `PlayerState` initialization (~line 288, after `life_lost_this_turn: 0`)

#### 1f. Hash the new field

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.damage_received_this_turn.hash_into(hasher);` after the `life_lost_this_turn` hash line (~line 831)

#### 1g. Reset in `reset_turn_state`

**File**: `crates/engine/src/rules/turn_actions.rs`
**Action**: Add `p.damage_received_this_turn = 0;` in the all-players reset loop after `p.life_lost_this_turn = 0;` (~line 1198)

#### 1h. Increment at ALL 4 damage-to-player sites

These are the sites where damage is applied to a player. The new field must be incremented at each one, BEFORE or alongside the existing `life_lost_this_turn` increment (or in the infect branch where life_lost is NOT incremented).

**Site 1 -- Non-combat damage, normal (effects/mod.rs ~line 198)**:
After `player.life_lost_this_turn += final_dmg;` add:
```
player.damage_received_this_turn += final_dmg;
```

**Site 2 -- Non-combat damage, infect (effects/mod.rs ~line 188-194)**:
In the infect branch (poison counters given instead of life loss), add after the PoisonCountersGiven event push:
```
if let Some(player) = state.players.get_mut(&p) {
    player.damage_received_this_turn += final_dmg;
}
```
Note: This site currently does NOT increment `life_lost_this_turn` (correctly, since infect doesn't cause life loss). But Bloodthirst counts infect damage too.

**Site 3 -- Combat damage, normal (combat.rs ~line 1451)**:
After `player.life_lost_this_turn += final_dmg;` add:
```
player.damage_received_this_turn += final_dmg;
```

**Site 4 -- Combat damage, infect (combat.rs ~line 1440-1447)**:
In the infect combat branch, after the infect poison counter logic, add:
```
if let Some(player) = state.players.get_mut(player_id) {
    player.damage_received_this_turn += final_dmg;
}
```

**Site 5 -- Cumulative upkeep life payment (engine.rs ~line 795-796)**:
This is LIFE PAYMENT, not damage. Do NOT increment `damage_received_this_turn` here. Life loss from paying costs is not damage.

### Step 2: Rule Enforcement (ETB counter placement)

#### 2a. Resolution.rs -- spell resolution ETB site

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add Bloodthirst ETB counter logic after the Amplify block (after line ~776)
**CR**: 702.54a -- "If an opponent was dealt damage this turn, this permanent enters with N +1/+1 counters on it."
**Pattern**: Follow the Amplify block structure at lines 679-776

Logic:
1. Collect all `Bloodthirst(n)` instances from the card definition (same pattern as Amplify).
2. Check if any opponent of the controller was dealt damage this turn: iterate `state.players`, find any player where `pid != controller && !ps.has_lost && !ps.has_conceded && ps.damage_received_this_turn > 0`.
3. If the condition is met, for each Bloodthirst(n) instance, add N +1/+1 counters.
4. For Bloodthirst X (future -- represented as `Bloodthirst(0)` with a special flag, or handled via a separate variant): sum `damage_received_this_turn` across all opponents. **For now, implement Bloodthirst N only** (the overwhelmingly common form). Bloodthirst X can be added later if a card needs it.
5. Emit `CounterAdded` event.

```rust
// CR 702.54a: Bloodthirst N -- "If an opponent was dealt damage this turn,
// this permanent enters with N +1/+1 counters on it."
// CR 702.54c: Multiple instances work separately.
{
    let bloodthirst_instances: Vec<u32> = card_id
        .as_ref()
        .and_then(|cid| registry.get(cid.clone()))
        .map(|def| {
            def.abilities
                .iter()
                .filter_map(|a| match a {
                    crate::cards::card_definition::AbilityDefinition::Keyword(
                        KeywordAbility::Bloodthirst(n),
                    ) => Some(*n),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    if !bloodthirst_instances.is_empty() {
        // Check if any opponent was dealt damage this turn.
        // CR 800.4a: eliminated/conceded players are not opponents.
        let any_opponent_damaged = state.players.iter().any(|(pid, ps)| {
            *pid != controller
                && !ps.has_lost
                && !ps.has_conceded
                && ps.damage_received_this_turn > 0
        });

        if any_opponent_damaged {
            let mut total_counters: u32 = 0;
            for n in &bloodthirst_instances {
                total_counters += n;
            }

            if total_counters > 0 {
                if let Some(obj) = state.objects.get_mut(&new_id) {
                    let current = obj
                        .counters
                        .get(&CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    obj.counters = obj
                        .counters
                        .update(CounterType::PlusOnePlusOne, current + total_counters);
                }
                events.push(GameEvent::CounterAdded {
                    object_id: new_id,
                    counter: CounterType::PlusOnePlusOne,
                    count: total_counters,
                });
            }
        }
    }
}
```

#### 2b. Lands.rs -- land ETB site

**File**: `crates/engine/src/rules/lands.rs`
**Action**: Add Bloodthirst ETB counter logic after the Amplify block (after line ~296)
**CR**: 702.54a
**Note**: Lands with Bloodthirst are extremely rare (no printed cards exist), but consistency with the ETB site pattern requires it. The comment should note this is for completeness.

Same logic as 2a but using `new_land_id` instead of `new_id`, and `player` instead of `controller`.

### Step 3: Trigger Wiring

**N/A** -- Bloodthirst is a static ability (CR 702.54a says "Bloodthirst is a static ability"). It functions as an ETB replacement effect, not a triggered ability. No trigger dispatch needed.

### Step 4: Unit Tests

**File**: `crates/engine/tests/bloodthirst.rs`
**Pattern**: Follow `crates/engine/tests/amplify.rs` structure (card definitions, helpers, cast + resolve pattern)

**Tests to write**:

1. `test_bloodthirst_basic_opponent_damaged` -- CR 702.54a: Bloodthirst 2 creature enters after an opponent was dealt damage. Setup: set `damage_received_this_turn = 3` on opponent. Expect 2 +1/+1 counters.

2. `test_bloodthirst_no_damage_dealt` -- CR 702.54a: Bloodthirst creature enters when no opponent was dealt damage this turn. Expect 0 counters, creature enters as printed P/T.

3. `test_bloodthirst_n_multiplier` -- CR 702.54a: Bloodthirst 3 creature enters after opponent took damage. Expect 3 counters.

4. `test_bloodthirst_multiple_instances` -- CR 702.54c: Creature with Bloodthirst 1 and Bloodthirst 2 enters after opponent took damage. Expect 3 counters (1 + 2).

5. `test_bloodthirst_multiple_opponents_damaged` -- CR 702.54a: In multiplayer, multiple opponents were dealt damage. Bloodthirst condition is still binary (any opponent), so still N counters.

6. `test_bloodthirst_only_controller_damaged` -- CR 702.54a: Only the controller (not an opponent) was dealt damage. Expect 0 counters.

7. `test_bloodthirst_eliminated_opponent_not_counted` -- CR 800.4a: An eliminated opponent's damage does not satisfy Bloodthirst. Setup: opponent has `has_lost = true` and `damage_received_this_turn > 0`. Expect 0 counters.

8. `test_bloodthirst_counter_added_event` -- Verify `CounterAdded` event is emitted with correct count.

**Card definitions for tests** (inline in test file):
- `Bloodthirst Test Creature` -- Bloodthirst 2, 1/1 Human Berserker, cost {1}{R}
- `Bloodthirst Three Test` -- Bloodthirst 3, 2/2, cost {3}
- `Bloodthirst Dual Test` -- Bloodthirst 1 + Bloodthirst 2, 1/1, cost {2}

**Test setup pattern**: Tests will directly set `state.players.get_mut(&opponent).unwrap().damage_received_this_turn = N` before casting the Bloodthirst creature. This avoids needing to deal actual damage (which would require combat or spell resolution), keeping tests focused on the ETB behavior.

### Step 5: Card Definition (later phase)

**Suggested card**: Stormblood Berserker
- Mana Cost: {1}{R}
- Type: Creature -- Human Berserker
- Oracle: Bloodthirst 2, Menace
- P/T: 1/1
- Both keywords (Bloodthirst, Menace) are already implemented (Menace was P1)

### Step 6: Game Script (later phase)

**Suggested scenario**: Player A casts Lightning Bolt targeting Player B, then casts Stormblood Berserker. Berserker enters with 2 +1/+1 counters (as a 3/3 with Menace).
**Subsystem directory**: `test-data/generated-scripts/stack/` or `replacement/`

## Interactions to Watch

- **Infect damage**: Infect deals damage but causes poison counters instead of life loss. `damage_received_this_turn` MUST be incremented for infect damage. `life_lost_this_turn` is NOT incremented for infect. This is the primary reason a new field is needed.
- **Prevention effects**: If damage is prevented (e.g., by protection, Fog, or damage prevention shields), the prevented damage is NOT "dealt" and should NOT increment `damage_received_this_turn`. The existing damage prevention infrastructure already handles this -- `final_dmg` is the post-prevention amount.
- **Replacement effects on damage**: If damage is replaced (e.g., "if damage would be dealt, prevent it and..."), the replacement happens before the damage event, so `damage_received_this_turn` is only incremented by actual damage dealt (post-replacement).
- **Multiplayer**: Bloodthirst checks "an opponent" -- any single opponent being damaged is sufficient. The N value is fixed (not dependent on how much damage was dealt). This is different from Bloodthirst X (CR 702.54b) which sums total damage to all opponents.
- **Self-damage**: A player can deal damage to themselves, but themselves is not "an opponent." Self-damage does not satisfy Bloodthirst.

## Files Modified Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `Bloodthirst(u32)` variant (KW 123) |
| `crates/engine/src/state/hash.rs` | Add hash arm for Bloodthirst + hash `damage_received_this_turn` |
| `crates/engine/src/state/player.rs` | Add `damage_received_this_turn: u32` field |
| `crates/engine/src/state/builder.rs` | Initialize `damage_received_this_turn: 0` |
| `crates/engine/src/rules/turn_actions.rs` | Reset `damage_received_this_turn` in `reset_turn_state` |
| `crates/engine/src/rules/resolution.rs` | Add Bloodthirst ETB counter logic (after Amplify block) |
| `crates/engine/src/rules/lands.rs` | Add Bloodthirst ETB counter logic (after Amplify block) |
| `crates/engine/src/effects/mod.rs` | Increment `damage_received_this_turn` at 2 damage sites |
| `crates/engine/src/rules/combat.rs` | Increment `damage_received_this_turn` at 2 combat damage sites |
| `tools/replay-viewer/src/view_model.rs` | Add display arm for Bloodthirst |
| `crates/engine/tests/bloodthirst.rs` | New test file with 8 tests |

## Discriminant Summary

- **KeywordAbility**: `Bloodthirst(u32)` = discriminant 123
- **AbilityDefinition**: No new variant needed (uses `AbilityDefinition::Keyword(KeywordAbility::Bloodthirst(n))`)
- **StackObjectKind**: No new variant needed (static ability, not triggered)

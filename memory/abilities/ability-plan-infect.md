# Ability Plan: Infect

**Generated**: 2026-02-28
**CR**: 702.90
**Priority**: P3
**Similar abilities studied**: Wither (CR 702.80) -- `state/types.rs:475-481`, `state/hash.rs:421-422`, `rules/combat.rs:863-1006`, `effects/mod.rs:206-241`, `tests/keywords.rs:2150-2598`

## CR Rule Text

702.90. Infect

> 702.90a Infect is a static ability.
>
> 702.90b Damage dealt to a player by a source with infect doesn't cause that player to lose life. Rather, it causes that source's controller to give the player that many poison counters. See rule 120.3.
>
> 702.90c Damage dealt to a creature by a source with infect isn't marked on that creature. Rather, it causes that source's controller to put that many -1/-1 counters on that creature. See rule 120.3.
>
> 702.90d If an object changes zones before an effect causes it to deal damage, its last known information is used to determine whether it had infect.
>
> 702.90e The infect rules function no matter what zone an object with infect deals damage from.
>
> 702.90f Multiple instances of infect on the same object are redundant.

### Supporting Rules

**CR 120.3** (Damage Results):
> 120.3a Damage dealt to a player by a source without infect causes that player to lose that much life.
> 120.3b Damage dealt to a player by a source with infect causes that source's controller to give the player that many poison counters.
> 120.3d Damage dealt to a creature by a source with wither and/or infect causes that source's controller to put that many -1/-1 counters on that creature.

**CR 704.5c** (SBA -- poison loss):
> If a player has ten or more poison counters, that player loses the game.

**CR 122.1f** (Poison counter definition):
> If a player has ten or more poison counters, that player loses the game as a state-based action. A player is "poisoned" if they have one or more poison counters.

## Key Edge Cases

- **CR 120.3d**: Infect shares the creature-damage mechanic with Wither. A source with infect (but not wither) still places -1/-1 counters on creatures. A source with BOTH infect and wither places -1/-1 counters once (not twice). The existing wither check should become `source_has_wither || source_has_infect`.
- **CR 702.90b / 120.3b**: Damage to a PLAYER from an infect source does NOT cause life loss. Instead, it gives poison counters. This is a replacement of the normal damage result, not an additional effect. The `life_total -= final_dmg` line must be replaced with `poison_counters += final_dmg` when the source has infect.
- **CR 702.90f**: Multiple instances of infect are redundant. The engine uses `OrdSet<KeywordAbility>` for keywords, so duplicate instances are automatically deduplicated. No special handling needed.
- **CR 702.90d**: Last known information for zone-changed sources. The engine already handles this for Wither via `calculate_characteristics` on the source at damage-application time. Infect should use the same mechanism.
- **CR 702.90e**: Functions from any zone (same as Wither CR 702.80c). No zone restriction in the keyword check.
- **Infect + Lifelink**: Per CR 120.3f, lifelink causes the source's controller to gain life "in addition to the damage's other results." With infect, the damage result is poison counters (to players) or -1/-1 counters (to creatures), but lifelink still applies. The controller gains life equal to the damage dealt. This already works correctly because lifelink is checked independently of the damage application path.
- **Infect + Trample**: When a creature with infect and trample deals combat damage to a player, the damage to the player becomes poison counters (not life loss). The trample assignment logic (assigning excess to the player) does not change -- only what happens when that damage is applied to the player changes.
- **Infect + Deathtouch + Trample**: Lethal damage to a blocker is 1 (deathtouch). The rest tramples through to the player as poison counters. This works without changes because the trample/deathtouch assignment is separate from the damage application.
- **Multiplayer**: Each opponent independently receives poison counters from infect damage. The existing SBA 704.5c (10+ poison = lose) already handles this per-player. No multiplayer-specific changes needed.
- **Planeswalker damage**: Infect does NOT interact with planeswalker damage. CR 120.3b and 120.3c are separate rules -- infect modifies player damage and creature damage, not planeswalker damage (loyalty counter removal). No changes needed for the planeswalker path.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A -- infect is a static ability, no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Already Implemented Infrastructure

The following infrastructure already exists and does NOT need to be added:

- **`PlayerState.poison_counters: u32`** -- `/home/airbaggie/scutemob/crates/engine/src/state/player.rs:74`
- **`GameStateBuilder::player_poison()`** -- `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs:139-145`
- **`PlayerBuilder::poison()`** -- `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs:820-822`
- **SBA 704.5c (poison loss)** -- `/home/airbaggie/scutemob/crates/engine/src/rules/sba.rs:251-260`
- **SBA tests for poison** -- `/home/airbaggie/scutemob/crates/engine/tests/sba.rs:125-166`
- **`LossReason::PoisonCounters`** -- `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs:47`
- **Hash for `LossReason::PoisonCounters`** -- `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs:481`
- **Hash for `poison_counters` on `PlayerState`** -- `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs:601`
- **Script schema `PlayerInitState.poison_counters`** -- `/home/airbaggie/scutemob/crates/engine/src/testing/script_schema.rs:100`
- **Script assertion `players.<name>.poison_counters`** -- `/home/airbaggie/scutemob/crates/engine/tests/script_replay.rs:257`
- **View model `PlayerView.poison`** -- `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs:64`

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Infect` variant after `Wither` (line ~481).
**Doc comment**:
```rust
/// CR 702.90: Infect -- "Damage dealt to a creature by a source with infect
/// isn't marked on that creature. Rather, it causes that many -1/-1 counters
/// to be put on that creature. Damage dealt to a player by a source with
/// infect doesn't cause that player to lose life. Rather, it causes that
/// player to get that many poison counters."
///
/// Static ability. Functions from any zone (CR 702.90e). Multiple instances
/// are redundant (CR 702.90f). Shares creature-damage mechanic with Wither
/// (CR 120.3d); additionally replaces player damage with poison counters
/// (CR 120.3b).
Infect,
```
**Pattern**: Follow `KeywordAbility::Wither` at line 481.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant for `KeywordAbility::Infect`. Next available discriminant is **63**.
**Location**: After `KeywordAbility::Ascend => 62u8.hash_into(hasher),` at line 433.
```rust
// Infect (discriminant 63) -- CR 702.90
KeywordAbility::Infect => 63u8.hash_into(hasher),
```

### Step 2: GameEvent for Poison Counters

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add a `PoisonCountersGiven` event variant to `GameEvent`. This event is needed for:
  (a) observable side-effect tracking (parallel to `LifeLost` for normal damage),
  (b) future trigger support (e.g., "whenever a player gets poison counters"),
  (c) script assertion validation.

**Location**: Near `LifeLost` at line ~357, add:
```rust
/// A player received poison counters (CR 120.3b, CR 702.90b).
/// Emitted when infect damage is dealt to a player.
PoisonCountersGiven {
    player: PlayerId,
    amount: u32,
    source: ObjectId,
},
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `GameEvent::PoisonCountersGiven` to the `HashInto` impl for `GameEvent` (if events are hashed -- check first; if `GameEvent` does not impl `HashInto`, skip this).

**Check**: Grep for `impl HashInto for GameEvent` in hash.rs. If absent, no hash entry needed.

### Step 3: Rule Enforcement -- Combat Damage (combat.rs)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs`

#### Step 3a: Extend DamageAppInfo tuple

**Location**: Line ~863, the `DamageAppInfo` type alias.
**Current**: `type DamageAppInfo = (bool, bool, bool, PlayerId, Option<(PlayerId, CardId)>);`
**New**: `type DamageAppInfo = (bool, bool, bool, bool, PlayerId, Option<(PlayerId, CardId)>);`
The four booleans are: `(source_deathtouch, source_lifelink, source_wither, source_infect, source_controller, commander_info)`.

**Location**: After the `source_wither` check at line ~880-883, add `source_infect`:
```rust
// CR 702.90a: Damage dealt by a source with infect to a creature
// places -1/-1 counters; to a player gives poison counters.
let source_infect = chars
    .as_ref()
    .map(|c| c.keywords.contains(&KeywordAbility::Infect))
    .unwrap_or(false);
```

**Location**: Update the tuple construction at line ~901-907 to include `source_infect`:
```rust
(
    source_deathtouch,
    source_lifelink,
    source_wither,
    source_infect,
    source_controller,
    commander_info,
)
```

**Location**: Update the destructuring at line ~943-953:
```rust
for (
    (
        assignment,
        (source_deathtouch, source_lifelink, source_wither, source_infect, source_controller, commander_info),
    ),
    &final_dmg,
) in assignments
```

#### Step 3b: Creature damage -- reuse wither logic for infect

**Location**: Line ~960-983, the `CombatDamageTarget::Creature(obj_id)` arm.
**Current check**: `if *source_wither { ... }`
**New check**: `if *source_wither || *source_infect { ... }`
**CR**: 120.3d -- "Damage dealt to a creature by a source with wither **and/or infect** causes that source's controller to put that many -1/-1 counters on that creature."

Update the comment to cite both CR 702.80a and CR 702.90c / CR 120.3d.

#### Step 3c: Player damage -- replace life loss with poison counters for infect

**Location**: Line ~985-1018, the `CombatDamageTarget::Player(player_id)` arm.
**Current**: Unconditionally does `player.life_total -= final_dmg as i32;`
**New**: Branch on `source_infect`:

```rust
CombatDamageTarget::Player(player_id) => {
    if *source_infect {
        // CR 702.90b / CR 120.3b: infect damage to a player gives
        // poison counters instead of causing life loss.
        if let Some(player) = state.players.get_mut(player_id) {
            player.poison_counters += final_dmg;
        }
        // Emit PoisonCountersGiven event.
        // Note: we still emit DamageDealt below (damage IS dealt, just
        // with a different result per CR 120.3b).
    } else {
        // CR 120.3a: normal damage causes life loss.
        if let Some(player) = state.players.get_mut(player_id) {
            player.life_total -= final_dmg as i32;
        }
    }
    // Track commander damage (CR 903.10a) -- applies regardless of infect.
    // Commander damage is combat damage, not life loss, so it still counts.
    if let Some((attacking_player, card_id)) = commander_info {
        // ... existing commander damage tracking code ...
    }
}
```

**Important**: Commander damage tracking (CR 903.10a) must still apply even when infect replaces life loss with poison. Commander damage counts COMBAT damage dealt, not life lost. An infect commander dealing 21+ combat damage to a player still causes a loss via the commander damage SBA, even though the player didn't lose life. The existing code already tracks `final_dmg` regardless of the damage result, so no change needed to the commander damage tracking itself -- just ensure the code path is reached for both infect and non-infect.

### Step 4: Rule Enforcement -- Non-Combat Damage (effects/mod.rs)

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`

#### Step 4a: Creature damage -- reuse wither logic for infect

**Location**: Line ~206-241, inside `Effect::DealDamage`, the `card_types.contains(&CardType::Creature)` branch.
**Current check**: `if source_has_wither { ... }`
**New check**: Also check for infect on the source.

```rust
let source_has_wither = crate::rules::layers::calculate_characteristics(
    state, ctx.source,
)
.map(|c| c.keywords.contains(&KeywordAbility::Wither))
.unwrap_or(false);

let source_has_infect = crate::rules::layers::calculate_characteristics(
    state, ctx.source,
)
.map(|c| c.keywords.contains(&KeywordAbility::Infect))
.unwrap_or(false);
```

**Optimization note**: Both checks call `calculate_characteristics`. Since this is potentially expensive, compute once:

```rust
let source_chars = crate::rules::layers::calculate_characteristics(
    state, ctx.source,
);
let source_has_wither = source_chars
    .as_ref()
    .map(|c| c.keywords.contains(&KeywordAbility::Wither))
    .unwrap_or(false);
let source_has_infect = source_chars
    .as_ref()
    .map(|c| c.keywords.contains(&KeywordAbility::Infect))
    .unwrap_or(false);
```

Then change `if source_has_wither {` to `if source_has_wither || source_has_infect {`.

Update the comment to cite both CR 702.80a and CR 702.90c / CR 120.3d.

#### Step 4b: Player damage -- replace life loss with poison counters for infect

**Location**: Line ~143-167, the `ResolvedTarget::Player(p)` arm inside `Effect::DealDamage`.
**Current**: Unconditionally does `player.life_total -= final_dmg as i32;` and emits `LifeLost`.
**New**: Check for infect on the source.

```rust
ResolvedTarget::Player(p) => {
    let damage_target = CombatDamageTarget::Player(p);
    let (final_dmg, prev_events) =
        crate::rules::replacement::apply_damage_prevention(
            state, ctx.source, &damage_target, dmg,
        );
    events.extend(prev_events);
    if final_dmg > 0 {
        // CR 702.90b / CR 120.3b: check source for infect.
        let source_has_infect =
            crate::rules::layers::calculate_characteristics(state, ctx.source)
                .map(|c| c.keywords.contains(&KeywordAbility::Infect))
                .unwrap_or(false);

        if source_has_infect {
            // CR 120.3b: infect damage to a player gives poison
            // counters instead of causing life loss.
            if let Some(player) = state.players.get_mut(&p) {
                player.poison_counters += final_dmg;
            }
            events.push(GameEvent::PoisonCountersGiven {
                player: p,
                amount: final_dmg,
                source: ctx.source,
            });
        } else {
            // CR 120.3a: normal damage causes life loss.
            if let Some(player) = state.players.get_mut(&p) {
                player.life_total -= final_dmg as i32;
            }
            events.push(GameEvent::LifeLost {
                player: p,
                amount: final_dmg,
            });
        }
        events.push(GameEvent::DamageDealt {
            source: ctx.source,
            target: damage_target,
            amount: final_dmg,
        });
    }
}
```

**Note**: The `DamageDealt` event is still emitted for infect damage (damage IS dealt, just with a different result). Only the `LifeLost` event is replaced by `PoisonCountersGiven`.

### Step 5: Trigger Wiring

**N/A** -- Infect is a static ability (CR 702.90a). It does not trigger. It modifies how damage results are applied. No trigger wiring needed.

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/keywords.rs`
**Location**: After the Wither tests (line ~2599), add a new section.

**Tests to write**:

1. **`test_702_90_infect_combat_damage_places_minus_counters_on_creature`**
   - CR 702.90c / CR 120.3d
   - p1's 3/3 infect creature attacks, p2's 4/4 blocks.
   - Assert: blocker has 3 -1/-1 counters, damage_marked == 0.
   - Assert: CounterAdded event emitted.
   - Assert: attacker has normal damage_marked (blocker has no infect).
   - **Pattern**: Follow `test_702_80_wither_combat_damage_places_minus_counters` at line ~2156.

2. **`test_702_90_infect_combat_damage_gives_poison_counters_to_player`**
   - CR 702.90b / CR 120.3b
   - p1's 3/3 infect creature attacks p2 (unblocked).
   - Assert: p2's poison_counters == 3.
   - Assert: p2's life_total is UNCHANGED (no life loss).
   - Assert: PoisonCountersGiven event emitted.
   - Assert: DamageDealt event emitted (damage IS dealt).
   - Assert: no LifeLost event for p2.

3. **`test_702_90_infect_noncombat_damage_creature_places_counters`**
   - CR 702.90c / CR 120.3d / CR 702.90e
   - Use Effect::DealDamage with an infect source targeting a creature.
   - Assert: target has -1/-1 counters, damage_marked == 0.
   - **Pattern**: Follow `test_702_80_wither_noncombat_damage_places_counters` at line ~2513.

4. **`test_702_90_infect_noncombat_damage_player_gives_poison`**
   - CR 702.90b / CR 120.3b / CR 702.90e
   - Use Effect::DealDamage with an infect source targeting a player.
   - Assert: player's poison_counters increased.
   - Assert: player's life_total UNCHANGED.
   - Assert: PoisonCountersGiven event emitted.

5. **`test_702_90_infect_kills_via_poison_sba`**
   - CR 702.90b + CR 704.5c
   - p2 starts with 8 poison. p1's 2/2 infect creature attacks unblocked.
   - Assert: p2 has 10 poison counters.
   - Assert: p2 has lost (PlayerLost with reason PoisonCounters).
   - Assert: p2's life_total is UNCHANGED.

6. **`test_702_90_infect_redundant_instances`**
   - CR 702.90f
   - Creature with two Infect keywords deals damage.
   - Assert: damage is NOT doubled (same poison/counters as single infect).
   - **Pattern**: Follow `test_702_80_wither_redundant_instances` at line ~2451.

7. **`test_702_90_infect_wither_overlap_creature`**
   - CR 120.3d
   - Creature with BOTH Wither and Infect deals combat damage to another creature.
   - Assert: -1/-1 counters placed ONCE (not doubled).
   - Assert: damage_marked == 0.

8. **`test_702_90_infect_does_not_affect_planeswalker_damage`**
   - CR 120.3c (separate from 120.3b)
   - Infect source deals damage to a planeswalker.
   - Assert: loyalty counters removed normally (no -1/-1 counters, no poison).

9. **`test_702_90_infect_commander_damage_still_tracks`**
   - CR 903.10a + CR 702.90b
   - An infect commander deals combat damage to a player.
   - Assert: commander_damage_received is updated.
   - Assert: poison counters given (not life lost).
   - Assert: if 21+ commander damage, player loses via CommanderDamage SBA.

### Step 7: Card Definition (later phase)

**Suggested card**: Glistener Elf
- Simple 1/1 creature with only Infect.
- Mana cost: {G}
- Type: Creature -- Phyrexian Elf Warrior
- Keywords: Infect
- Oracle text: "Infect"
- This is the simplest possible Infect card for validation.

**Alternative card**: Phyrexian Crusader (more complex: first strike, protection from red and white, infect; mana cost {1}{B}{B}). Good for interaction testing but more complex for initial validation.

**Card lookup**: use `card-definition-author` agent.

### Step 8: Game Script (later phase)

**Suggested scenario**: "Infect combat -- creature damage as -1/-1 counters, player damage as poison"
- Initial state: p1 has Glistener Elf (1/1 infect) on battlefield, p2 has a 3/3 creature.
- Step 1: Glistener Elf attacks p2.
- Step 2: p2 blocks with 3/3.
- Step 3: Combat damage resolves.
- Assert: 3/3 has 1 -1/-1 counter, Glistener Elf dies.

**Alternative scenario**: "Infect unblocked -- poison counters"
- Glistener Elf attacks p2 unblocked.
- Assert: p2 has 1 poison counter, life total unchanged.

**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

- **Wither code reuse (CR 120.3d)**: The creature-damage path for Infect is identical to Wither. Change the condition from `source_wither` to `source_wither || source_infect` in BOTH combat.rs and effects/mod.rs. Do NOT duplicate the -1/-1 counter placement code.
- **Lifelink + Infect**: Lifelink still works with infect (controller gains life equal to damage dealt). The lifelink check in combat.rs is independent of the wither/infect damage-result logic. Verify this works in a test or note for future testing.
- **Combat damage events**: The `CombatDamageDealt` event with its assignments is emitted regardless of infect. The assignments record the raw damage amounts. Only the application of that damage changes (poison vs life loss).
- **DamageDealt event**: Must still be emitted for infect damage. The event represents "damage was dealt" which is true -- infect only changes the result, not whether damage occurred. This matters for triggers like "whenever a creature deals damage" or "whenever damage is dealt to a player."
- **Commander damage**: Infect combat damage to a player still counts for commander damage tracking (CR 903.10a). Commander damage is defined as combat damage dealt, not life lost. An infect commander dealing 21 combat damage causes loss via CommanderDamage SBA even though the player also has 21 poison counters (both SBAs would fire simultaneously, but the player only loses once).
- **No effect on LoseLife/DrainLife**: The `Effect::LoseLife` and `Effect::DrainLife` paths in effects/mod.rs cause life LOSS, not damage. Infect only modifies damage results (CR 120.3). These paths must NOT be modified.

## Files to Modify Summary

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Infect` variant |
| `crates/engine/src/state/hash.rs` | Add discriminant 63 for `Infect` |
| `crates/engine/src/rules/events.rs` | Add `GameEvent::PoisonCountersGiven` variant |
| `crates/engine/src/rules/combat.rs` | Extend `DamageAppInfo` tuple; add infect check for creature damage (reuse wither path); add infect check for player damage (poison instead of life loss) |
| `crates/engine/src/effects/mod.rs` | Add infect check for creature damage (reuse wither path); add infect check for player damage (poison instead of life loss) |
| `crates/engine/tests/keywords.rs` | Add 9 unit tests for infect |

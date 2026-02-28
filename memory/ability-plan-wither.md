# Ability Plan: Wither

**Generated**: 2026-02-28
**CR**: 702.80
**Priority**: P3
**Similar abilities studied**: Deathtouch (combat damage keyword check in `combat.rs:869-872`, SBA in `sba.rs:284-288`), Lifelink (combat damage keyword check in `combat.rs:874-877`, life gain in `combat.rs:998-1002`)

## CR Rule Text

```
702.80. Wither

702.80a Wither is a static ability. Damage dealt to a creature by a source with wither
isn't marked on that creature. Rather, it causes that source's controller to put that many
-1/-1 counters on that creature. See rule 120.3.

702.80b If an object changes zones before an effect causes it to deal damage, its last
known information is used to determine whether it had wither.

702.80c The wither rules function no matter what zone an object with wither deals damage
from.

702.80d Multiple instances of wither on the same object are redundant.
```

Supporting rule (CR 120.3):

```
120.3d Damage dealt to a creature by a source with wither and/or infect causes that
source's controller to put that many -1/-1 counters on that creature.

120.3e Damage dealt to a creature by a source with neither wither nor infect causes that
much damage to be marked on that creature.
```

## Key Edge Cases

- **Wither only affects damage to creatures** (CR 702.80a, confirmed by Puncture Blast
  ruling 2008-08-01): damage to players and planeswalkers is dealt normally (life loss /
  loyalty counter removal). Only the creature case changes from damage_marked to -1/-1
  counters.
- **Non-combat damage with wither** (CR 702.80c): Puncture Blast is the only printed
  instant/sorcery with wither. When it deals damage to a creature, it places -1/-1
  counters instead of marking damage. The effects/mod.rs `DealDamage` handler must check
  the source for wither.
- **Creatures die via SBA 704.5f (toughness <= 0), NOT 704.5g (lethal damage)**: With
  wither, `damage_marked` stays at 0. The -1/-1 counters reduce effective toughness
  through Layer 7c (`layers.rs:116-129`). When toughness drops to 0 or below, 704.5f
  kills the creature. The SBA lethal damage check (704.5g) and deathtouch check (704.5h)
  are NOT involved because `damage_marked == 0`.
- **Interaction with Persist (CR 702.79)**: Persist triggers only when the creature had
  no -1/-1 counters at death. A creature that took wither damage will have -1/-1 counters
  and therefore will NOT trigger persist. This is a key mechanical interaction to test.
- **Interaction with Deathtouch**: Wither + deathtouch works: the wither source places
  -1/-1 counters, AND if the source has deathtouch, any amount > 0 is lethal. But since
  wither doesn't mark damage, the deathtouch SBA (704.5h) won't trigger on `damage_marked`.
  However, even 1 -1/-1 counter on a creature dealt damage by a deathtouch source is
  sufficient because the counter reduces toughness. For the 704.5h SBA to fire, we would
  need `deathtouch_damage == true && damage_marked > 0`. With pure wither+deathtouch,
  `damage_marked` stays 0, so the creature dies only if the counters reduce toughness to
  <= 0 via 704.5f. Note: this is correct behavior -- deathtouch is redundant on top of
  wither for creature damage (both achieve the same result of killing the creature, just
  via different SBA paths depending on the amount).
- **Wither + Infect (future)**: CR 120.3d says "wither and/or infect". Having both is
  redundant for creature damage. When Infect is later implemented, the creature-damage
  check should be `has_wither || has_infect`. No special handling needed now; just use a
  check pattern that is easy to extend.
- **Multiple instances redundant** (CR 702.80d): No special handling needed; the check is
  boolean ("does the source have wither?").
- **Last-known information** (CR 702.80b): If the source changes zones before damage is
  dealt, use its last known characteristics. For combat damage this is handled by the
  pre-extract pattern in `app_info` (line 864). For non-combat damage in effects/mod.rs,
  `ctx.source` references the spell on the stack during resolution, so the source is still
  available.
- **CounterAdded event**: When placing -1/-1 counters via wither, a `GameEvent::CounterAdded`
  event MUST be emitted (not just a `DamageDealt` event). This is because other abilities
  may trigger on counters being placed.

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- exists at `types.rs:481` (`KeywordAbility::Wither`),
  `hash.rs:422` (discriminant 58), `view_model.rs`
- [ ] Step 2: Rule enforcement (combat damage in `combat.rs`, non-combat damage in
  `effects/mod.rs`)
- [ ] Step 3: Trigger wiring -- N/A (wither is a static ability that modifies damage
  application; it does not trigger)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant (DONE)

**File**: `crates/engine/src/state/types.rs`
**Status**: Already added at line 481 as `KeywordAbility::Wither`
**Hash**: Already added at `crates/engine/src/state/hash.rs:422` with discriminant 58
**View model**: Already added at `tools/replay-viewer/src/view_model.rs`

No action needed.

### Step 2a: Rule Enforcement -- Combat Damage

**File**: `crates/engine/src/rules/combat.rs`
**Action**: Modify the `apply_combat_damage` function at two locations:

#### 2a-i: Add `source_wither` to the pre-extract `app_info` tuple (line ~863-901)

The `DamageAppInfo` type alias at line 863 is currently:
```rust
type DamageAppInfo = (bool, bool, PlayerId, Option<(PlayerId, CardId)>);
```

Change to a 5-tuple that includes `source_wither`:
```rust
type DamageAppInfo = (bool, bool, bool, PlayerId, Option<(PlayerId, CardId)>);
//                    ^dt   ^ll   ^wither ^ctrl   ^cmdr
```

Add the wither check after the lifelink check (around line 877):
```rust
// CR 702.80a: Damage dealt to a creature by a source with wither places
// -1/-1 counters instead of marking damage.
let source_wither = chars
    .as_ref()
    .map(|c| c.keywords.contains(&KeywordAbility::Wither))
    .unwrap_or(false);
```

Update the return tuple to include `source_wither` (line ~895-900):
```rust
(source_deathtouch, source_lifelink, source_wither, source_controller, commander_info)
```

**Pattern**: Follow `source_deathtouch` at line 869 and `source_lifelink` at line 874.

#### 2a-ii: Add `counter_events` collector and use `source_wither` in the damage application loop

**Important**: The `events` vec is not created until line ~1006 (after the damage
application loop). `CounterAdded` events from wither must be collected separately during
the loop and spliced into the final events afterward.

**Before the damage application loop** (before line ~933), add:
```rust
// Collect wither counter events during the damage application loop.
// These will be added to the event stream after the loop.
let mut wither_counter_events: Vec<GameEvent> = Vec::new();
```

Update the destructuring at line ~933-934:
```rust
(assignment, (source_deathtouch, source_lifelink, source_wither, source_controller, commander_info))
```

Modify the `CombatDamageTarget::Creature` arm (line ~946-952). Currently:
```rust
CombatDamageTarget::Creature(obj_id) => {
    if let Some(obj) = state.objects.get_mut(obj_id) {
        obj.damage_marked += final_dmg;
        if *source_deathtouch {
            obj.deathtouch_damage = true;
        }
    }
}
```

Change to:
```rust
CombatDamageTarget::Creature(obj_id) => {
    if let Some(obj) = state.objects.get_mut(obj_id) {
        if *source_wither {
            // CR 702.80a / CR 120.3d: wither damage to a creature places
            // -1/-1 counters instead of marking damage.
            let cur = obj
                .counters
                .get(&CounterType::MinusOneMinusOne)
                .copied()
                .unwrap_or(0);
            obj.counters
                .insert(CounterType::MinusOneMinusOne, cur + final_dmg);
            wither_counter_events.push(GameEvent::CounterAdded {
                object_id: *obj_id,
                counter: CounterType::MinusOneMinusOne,
                count: final_dmg,
            });
        } else {
            // CR 120.3e: normal damage marking.
            obj.damage_marked += final_dmg;
        }
        if *source_deathtouch {
            obj.deathtouch_damage = true;
        }
    }
}
```

**After the damage application loop** (after line ~1003, before the `events` vec creation
at line ~1006), splice the wither counter events into the event stream:

Change:
```rust
let mut events = prevention_events;
events.push(GameEvent::CombatDamageDealt { ... });
```
To:
```rust
let mut events = prevention_events;
events.extend(wither_counter_events);
events.push(GameEvent::CombatDamageDealt { ... });
```

This places `CounterAdded` events BEFORE `CombatDamageDealt`, which matches the semantic
ordering: the counters are placed as part of the damage being dealt, so the counter events
logically precede the summary event.

**CR**: 702.80a, 120.3d, 120.3e
**Note**: Deathtouch is set regardless of wither. With wither, `damage_marked` stays 0,
so the 704.5h SBA (deathtouch + damage_marked > 0) won't fire. But setting the flag is
harmless and correct for future infect+deathtouch combinations. Also, if the creature has
both marked damage from another source AND -1/-1 counters from wither, deathtouch applies
to the marked damage portion.
**Required import**: Add `CounterType` to the existing import at line 17:
```rust
use crate::state::types::{CardType, Color, CounterType, KeywordAbility, LandwalkType, SuperType};
```

### Step 2b: Rule Enforcement -- Non-Combat Damage

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Modify the `DealDamage` handler's creature-damage path (lines 206-214).

Currently:
```rust
} else {
    // CR 120.3b: damage to a creature marks damage_marked.
    if let Some(obj) = state.objects.get_mut(&id) {
        obj.damage_marked += final_dmg;
        // CR 702.2: deathtouch — mark for SBA.
        // (deathtouch_damage field exists on the source; for spell
        // damage we do not set deathtouch_damage here — spell damage
        // sources don't have the deathtouch keyword in this context.)
    }
}
```

Change to:
```rust
} else {
    // CR 120.3d/e: check source for wither keyword.
    // CR 702.80c: wither functions from any zone.
    let source_has_wither = crate::rules::layers::calculate_characteristics(
        state, ctx.source,
    )
    .map(|c| c.keywords.contains(&KeywordAbility::Wither))
    .unwrap_or(false);

    if let Some(obj) = state.objects.get_mut(&id) {
        if source_has_wither {
            // CR 702.80a / CR 120.3d: wither damage to a creature places
            // -1/-1 counters instead of marking damage.
            let cur = obj
                .counters
                .get(&crate::state::types::CounterType::MinusOneMinusOne)
                .copied()
                .unwrap_or(0);
            obj.counters
                .insert(crate::state::types::CounterType::MinusOneMinusOne, cur + final_dmg);
            events.push(GameEvent::CounterAdded {
                object_id: id,
                counter: crate::state::types::CounterType::MinusOneMinusOne,
                count: final_dmg,
            });
        } else {
            // CR 120.3e: normal damage marking.
            obj.damage_marked += final_dmg;
        }
    }
}
```

**CR**: 702.80a, 702.80c, 120.3d, 120.3e
**Note**: Uses full path `crate::state::types::CounterType` for consistency with the
existing loyalty counter code at lines 199/204. `KeywordAbility` is already imported at
line 36.
**Note**: `calculate_characteristics` is used to check the source because wither functions
from any zone (CR 702.80c). The source may be an instant on the stack (Puncture Blast) or
a creature on the battlefield. `calculate_characteristics` handles both cases as long as
the object exists. If the source no longer exists (rare edge case), `unwrap_or(false)`
defaults to no wither, which is safe.
**Note**: Unlike combat damage, the `events` vec IS available here (it's the `events`
parameter passed to `execute_effect_inner`), so we can push `CounterAdded` directly.

### Step 3: Trigger Wiring

**N/A.** Wither is a static ability that modifies damage application. It does not
generate triggers. The `CounterAdded` event emitted in Step 2 may trigger other
abilities (e.g., "whenever a counter is placed"), but that is handled by the existing
trigger dispatch infrastructure, not by wither itself.

### Step 4: Unit Tests

**File**: `crates/engine/tests/keywords.rs`
**Tests to write** (append to the end of the file):

All tests should use the existing `pass_all`, `find_object`, and builder patterns from
`keywords.rs`. Import `CounterType` and any other needed types.

#### Test 1: `test_702_80_wither_combat_damage_places_minus_counters`
- **CR**: 702.80a, 120.3d
- **Setup**: p1 has a 3/3 creature with Wither. p2 has a 4/4 creature. Start at
  DeclareAttackers step.
- **Actions**: p1 attacks with wither creature, p2 blocks, advance through combat damage.
- **Assertions**:
  - The defending creature has 3 -1/-1 counters
    (`obj.counters.get(&CounterType::MinusOneMinusOne) == Some(&3)`)
  - The defending creature has `damage_marked == 0` (wither does NOT mark damage)
  - The attacking creature (no wither on defender) has `damage_marked == 4` (normal damage)
  - A `CounterAdded` event was emitted for the -1/-1 counters

#### Test 2: `test_702_80_wither_combat_kills_creature_via_toughness_sba`
- **CR**: 702.80a, 704.5f
- **Setup**: p1 has a 3/3 creature with Wither. p2 has a 3/3 creature. Start at
  DeclareAttackers step.
- **Actions**: p1 attacks, p2 blocks, advance through combat damage, then check SBAs.
- **Assertions**:
  - After combat damage: p2's creature has 3 -1/-1 counters, 0 damage_marked
  - After SBAs: p2's creature is in graveyard (died via 704.5f: toughness 3-3=0)
  - p1's creature also dies (3 damage from blocker, 704.5g lethal damage)

#### Test 3: `test_702_80_wither_does_not_affect_player_damage`
- **CR**: 702.80a (only creatures)
- **Setup**: p1 has a 3/3 creature with Wither. p2 is the defending player. Start at
  DeclareAttackers step.
- **Actions**: p1 attacks p2 unblocked, advance through combat damage.
- **Assertions**:
  - p2's life total decreased by 3 (40 -> 37)
  - No -1/-1 counters were placed on any object
  - No `CounterAdded` event was emitted

#### Test 4: `test_702_80_wither_noncombat_damage_places_counters`
- **CR**: 702.80c, 120.3d
- **Setup**: p1 has a source object with Wither on the battlefield (use a creature with
  wither as the source object via `ctx.source`). p2 has a 4/4 creature on the
  battlefield. This test constructs a spell-damage scenario.
- **Approach**: Use `DealDamage` effect with a wither source. Create a stack object whose
  source has wither, then resolve it via the effect system. Alternatively, if the harness
  doesn't easily support this, use the lower-level `execute_effect` function directly.
- **Note**: This test requires careful construction. A simpler approach: define a card
  with both Wither and a damage effect (like Puncture Blast), register it, cast it
  targeting p2's creature, and verify -1/-1 counters instead of damage_marked.
  However, the card definition depends on Step 5. For now, test with a manually
  constructed `EffectContext` and `Effect::DealDamage`.
- **Assertions**:
  - Target creature has -1/-1 counters equal to the damage amount
  - Target creature has `damage_marked == 0`

#### Test 5: `test_702_80_wither_persist_interaction`
- **CR**: 702.80a, 702.79a
- **Setup**: p1 has a 3/3 creature with Wither. p2 has a 2/2 creature with Persist.
  Start at DeclareAttackers step.
- **Actions**: p1 attacks with wither creature, p2 blocks with persist creature. Advance
  through combat damage and SBAs.
- **Assertions**:
  - After combat damage: p2's creature has 3 -1/-1 counters (effective toughness = -1)
  - After SBAs: p2's creature dies (704.5f)
  - Persist does NOT trigger because the creature had -1/-1 counters at death
  - (Verify no trigger on the stack for the persist creature)

#### Test 6: `test_702_80_wither_redundant_instances`
- **CR**: 702.80d
- **Setup**: p1 has a 2/2 creature with Wither, Wither (two instances). p2 has a 3/3
  creature. Start at DeclareAttackers step.
- **Actions**: p1 attacks, p2 blocks, advance through combat damage.
- **Assertions**:
  - p2's creature has exactly 2 -1/-1 counters (not 4)
  - `damage_marked == 0`
  - Multiple instances don't double the effect

**Pattern**: Follow the deathtouch+trample test at `combat.rs:541-619` for combat setup
and the lifelink test at `keywords.rs:997` for keyword verification patterns.

### Step 5: Card Definition (later phase)

**Suggested cards**:
1. **Boggart Ram-Gang** ({R/G}{R/G}{R/G}, 3/3 Goblin Warrior, Haste, Wither) -- simple
   creature with wither for combat tests
2. **Puncture Blast** ({2}{R}, Instant, Wither, deals 3 damage to any target) -- the only
   instant with wither, critical for testing non-combat wither damage

**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: Boggart Ram-Gang attacks, gets blocked by a 4/4 creature. After
combat damage, verify -1/-1 counters on the blocker instead of damage_marked.
**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

### Damage Pipeline Integration
Wither modifies the damage application step, which sits between damage prevention
(replacement effects) and damage events. The modification order is:
1. Calculate raw damage amount
2. Apply damage prevention (`apply_damage_prevention`) -- this happens BEFORE wither check
3. Apply wither check on the final_dmg amount (new code)
4. Either place -1/-1 counters (wither) or mark damage (normal)
5. Emit `DamageDealt` / `CombatDamageDealt` events (wither damage is still "damage dealt")
6. Apply lifelink gains (wither damage still triggers lifelink -- CR 702.15a says
   "damage dealt by a source with lifelink")

This means:
- Prevention shields reduce the counter count (correct: if 3 damage is dealt by a wither
  source but 1 is prevented, only 2 -1/-1 counters are placed)
- Lifelink still works with wither (damage was dealt, just applied as counters)
- `DamageDealt` / `CombatDamageDealt` events are still emitted (other abilities may care
  about "damage was dealt" regardless of how it was applied)

### Layer System
-1/-1 counters are already handled in Layer 7c (`layers.rs:116-129`). No changes needed.

### SBA System
No SBA changes needed. The existing SBAs correctly handle:
- 704.5f: toughness <= 0 (catches creatures killed by -1/-1 counters from wither)
- 704.5q: counter annihilation (+1/+1 cancels -1/-1) (already implemented in `sba.rs:886-934`)

### Persist / Undying
- Persist (CR 702.79): checks for -1/-1 counters at death. Wither damage adds -1/-1
  counters, so persist won't trigger on a creature that died from wither damage.
- Undying (CR 702.93): checks for +1/+1 counters at death. Wither doesn't affect +1/+1
  counters, so undying is unaffected.

### Future: Infect (CR 702.90)
Infect shares wither's creature-damage behavior (CR 120.3d: "wither and/or infect").
When Infect is implemented later:
- The combat.rs wither check should become `*source_wither || *source_infect`
- The effects/mod.rs check should be similar
- Infect additionally changes player damage to poison counters (CR 702.90b)

The wither implementation should use a pattern that makes this extension easy. Naming
the boolean `source_wither` (not `source_wither_or_infect`) is fine for now; the infect
implementation will extend the tuple.

### Multiplayer
No special multiplayer considerations. Wither works identically regardless of player
count -- it only modifies how damage is applied to creatures.

## Required Imports to Verify

### `crates/engine/src/rules/combat.rs`
- `CounterType` -- NOT currently imported. Add to existing import at line 17:
  `use crate::state::types::{CardType, Color, CounterType, KeywordAbility, LandwalkType, SuperType};`
- `GameEvent::CounterAdded` -- already available via existing `GameEvent` import at line 22.

### `crates/engine/src/effects/mod.rs`
- `CounterType` -- NOT imported as a short name. Used via full path
  `crate::state::types::CounterType` at lines 199/204. Wither code should follow the
  same full-path pattern for consistency.
- `KeywordAbility` -- already imported at line 36 via
  `use crate::state::types::{CardType, KeywordAbility, ManaColor};`

### `crates/engine/tests/keywords.rs`
- `CounterType` -- add to the test file's import block (from `mtg_engine::CounterType`).
- `check_and_apply_sbas` -- needed for Test 2 (verify SBA kills creature after wither
  damage). Check if already imported; if not, add from `mtg_engine::check_and_apply_sbas`.

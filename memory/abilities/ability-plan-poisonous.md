# Ability Plan: Poisonous

**Generated**: 2026-03-01
**CR**: 702.70
**Priority**: P4
**Similar abilities studied**: Ingest (CR 702.115, same trigger event: `SelfDealsCombatDamageToPlayer`, same `target_player` pattern in `PendingTrigger`/`StackObjectKind` -- see `abilities.rs:2089-2177`, `stack.rs:407-427`, `resolution.rs:1643-1680`), Renown (CR 702.112, same trigger event, different resolution -- see `abilities.rs:2181-2266`, `resolution.rs:1842-1885`), Afflict (CR 702.130, different trigger but same parameterized `N` pattern -- see `types.rs:674-682`, `builder.rs:788-803`)

## CR Rule Text

702.70. Poisonous

702.70a Poisonous is a triggered ability. "Poisonous N" means "Whenever this creature deals combat damage to a player, that player gets N poison counters." (For information about poison counters, see rule 104.3d.)

702.70b If a creature has multiple instances of poisonous, each triggers separately.

Referenced rule:

104.3d If a player has ten or more poison counters, that player loses the game the next time a player would receive priority. (This is a state-based action. See rule 704.)

## Key Edge Cases

- **Poisonous gives poison counters IN ADDITION to normal combat damage** (CR 702.70a). Unlike Infect (which replaces damage with poison counters), Poisonous is a triggered ability that fires after combat damage is dealt. The creature still deals its normal combat damage (life loss to the player). This is the defining difference from Infect.
- **Fixed N, not damage-based** (Virulent Sliver ruling 2021-03-19): "Poisonous 1 causes the player to get just one poison counter when a Sliver deals combat damage to them, no matter how much damage that Sliver dealt." The number of poison counters is always N, regardless of how much damage was actually dealt.
- **Multiple instances trigger separately** (CR 702.70b, Virulent Sliver ruling 2021-03-19). A creature with Poisonous 1 and Poisonous 2 gives 1 + 2 = 3 poison counters total (two separate triggers on the stack, each resolving independently).
- **10+ poison counters SBA** (CR 104.3d, Virulent Sliver ruling 2021-03-19): "A player with ten or more poison counters loses the game as a state-based action, even if no permanents with poisonous are on the battlefield." The SBA already exists in `sba.rs:251-255`.
- **Trigger fires only on combat damage to a PLAYER** (CR 702.70a). Combat damage to a creature (blocking or being blocked) does NOT trigger Poisonous. The `CombatDamageTarget::Player(_)` guard already handles this in the `CombatDamageDealt` handler at `abilities.rs:2080`.
- **Damage must be > 0** (CR 603.2g). If combat damage is fully prevented (e.g., by protection or a prevention effect), the creature did not deal combat damage and Poisonous does not trigger. The existing `assignment.amount == 0` guard at `abilities.rs:2077-2078` handles this.
- **Triggered ability goes on the stack** -- unlike Toxic (702.164, static/inline), Poisonous triggers can be responded to with instants/abilities before they resolve. Each separate trigger resolves independently.
- **Multiplayer**: each Poisonous trigger targets the specific player who was dealt combat damage. In a 4-player game, if a creature with Poisonous 1 deals combat damage to P3, only P3 gets the poison counter.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: PendingTrigger fields
- [ ] Step 3: StackObjectKind + trigger dispatch + flush + resolution
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Design Decision: Custom StackObjectKind vs. Auto-generated TriggeredAbilityDef

Two implementation patterns were considered:

**Option A (Afflict/Training pattern)**: Add a new `Effect::GivePoisonCounters` variant, then auto-generate a `TriggeredAbilityDef` in `builder.rs`. This avoids a custom `StackObjectKind` but requires a new Effect variant with wider blast radius (effect execution engine, effects/mod.rs).

**Option B (Ingest/Renown pattern)**: Custom `StackObjectKind::PoisonousTrigger` with inline resolution in `resolution.rs`. More boilerplate (PendingTrigger fields + flush branch) but self-contained and mirrors the closest analog (Ingest -- same trigger event, same pattern of carrying `target_player`).

**Chosen: Option B.** Rationale:
1. Ingest is the closest analog (same trigger condition, same `target_player` transport pattern).
2. No new `Effect` variant needed -- resolution is 10 lines of direct `player.poison_counters += n`.
3. The `PoisonCountersGiven` event already exists from Infect infrastructure; reusing it is trivial.
4. Keeps the Poisonous resolution logic visually adjacent to Ingest in `resolution.rs`, making the contrast between them (exile vs. poison counters) clear.

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Poisonous(u32)` variant after `Training` (line ~700, before the closing `}`).
**Pattern**: Follow `KeywordAbility::Afflict(u32)` at line 682 (same `(u32)` parameter shape).
**Doc comment**:
```rust
/// CR 702.70: Poisonous N -- triggered ability.
/// "Whenever this creature deals combat damage to a player, that player gets N
/// poison counters."
/// CR 702.70b: Multiple instances trigger separately.
///
/// Unlike Infect (replacement effect converting damage to poison counters),
/// Poisonous is a triggered ability that adds poison counters IN ADDITION to
/// the normal combat damage. The N value is fixed, not based on damage dealt.
/// Reuses existing poison counter infrastructure (PlayerState.poison_counters,
/// PoisonCountersGiven event, 10-poison SBA CR 704.5c).
Poisonous(u32),
```

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
Add after `KeywordAbility::Training` hash (line 494), using discriminant **83**:
```rust
// Poisonous (discriminant 83) -- CR 702.70
KeywordAbility::Poisonous(n) => {
    83u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**View model keyword string**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
Add after `Training` arm (line 706):
```rust
KeywordAbility::Poisonous(n) => format!("Poisonous {n}"),
```

### Step 2: PendingTrigger Fields

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add three new fields to `PendingTrigger` struct before the closing `}` at line 300, after `renown_n`:
```rust
/// CR 702.70a: If true, this pending trigger is a Poisonous trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::PoisonousTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`.
/// The `poisonous_n` carries the N value (number of poison counters).
/// The `poisonous_target_player` carries the damaged player's ID.
#[serde(default)]
pub is_poisonous_trigger: bool,
/// CR 702.70a: The N value from "Poisonous N" -- how many poison counters
/// to give the damaged player when the trigger resolves.
///
/// Only meaningful when `is_poisonous_trigger` is true.
#[serde(default)]
pub poisonous_n: Option<u32>,
/// CR 702.70a: The player dealt combat damage (who receives poison counters).
///
/// Only meaningful when `is_poisonous_trigger` is true.
#[serde(default)]
pub poisonous_target_player: Option<PlayerId>,
```

**Match arms**: Every existing `PendingTrigger { .. }` struct literal in `abilities.rs` must include these three new fields set to their defaults. Grep for `is_renown_trigger: false` to find all ~15 construction sites. Add:
```rust
is_poisonous_trigger: false,
poisonous_n: None,
poisonous_target_player: None,
```

### Step 3: StackObjectKind + Trigger Dispatch + Flush + Resolution

#### 3a: StackObjectKind variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `PoisonousTrigger` variant after `RenownTrigger` (after line ~513):
```rust
/// CR 702.70a: Poisonous N triggered ability on the stack.
///
/// "Whenever this creature deals combat damage to a player, that player
/// gets N poison counters."
///
/// `source_object` is the creature with poisonous on the battlefield.
/// `target_player` is the player who was dealt combat damage (who receives
/// the poison counters).
/// `poisonous_n` is the number of poison counters to give.
///
/// When this trigger resolves:
/// 1. Give `target_player` exactly `poisonous_n` poison counters.
/// 2. Emit `PoisonCountersGiven` event (reusing the existing Infect event).
///
/// CR 702.70b: Multiple instances trigger separately (each creates its own
/// trigger with its own N value).
///
/// CR 603.10: The source creature does NOT need to be on the battlefield
/// at resolution time (the trigger is already on the stack). The poison
/// counters are given regardless of the source's current state.
PoisonousTrigger {
    source_object: ObjectId,
    target_player: crate::state::player::PlayerId,
    poisonous_n: u32,
},
```

#### 3b: StackObjectKind hash

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
Add after `RenownTrigger` hash (after line 1461), using discriminant **23**:
```rust
// PoisonousTrigger (discriminant 23) -- CR 702.70a
StackObjectKind::PoisonousTrigger {
    source_object,
    target_player,
    poisonous_n,
} => {
    23u8.hash_into(hasher);
    source_object.hash_into(hasher);
    target_player.hash_into(hasher);
    poisonous_n.hash_into(hasher);
}
```

#### 3c: Trigger dispatch in CombatDamageDealt handler

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add Poisonous trigger collection in the `CombatDamageDealt` handler, inside the `if matches!(assignment.target, CombatDamageTarget::Player(_))` block. Insert between the Ingest block (ends ~line 2178) and the Renown block (starts ~line 2181). Pattern mirrors the Ingest/Renown blocks.

```rust
// CR 702.70a: Poisonous N -- "Whenever this creature deals combat
// damage to a player, that player gets N poison counters."
// CR 702.70b: Multiple instances trigger separately.
if let Some(obj) = state.objects.get(&assignment.source) {
    if obj.zone == ZoneId::Battlefield {
        // Already guaranteed by the outer `if matches!(..., Player(_))`
        // guard -- use `let...else` for safety.
        let CombatDamageTarget::Player(damaged_player) = &assignment.target
        else {
            // Cannot reach here due to outer guard
            continue;
        };
        let damaged_player = *damaged_player;

        // Collect Poisonous N values from card definition.
        // CR 702.70b: Each keyword instance triggers separately.
        let poisonous_values: Vec<u32> = obj
            .card_id
            .as_ref()
            .and_then(|cid| state.card_registry.get(cid.clone()))
            .map(|def| {
                def.abilities
                    .iter()
                    .filter_map(|a| match a {
                        AbilityDefinition::Keyword(
                            KeywordAbility::Poisonous(n),
                        ) => Some(*n),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_else(|| {
                // Fallback: check keywords on the object itself
                obj.characteristics
                    .keywords
                    .iter()
                    .filter_map(|kw| match kw {
                        KeywordAbility::Poisonous(n) => Some(*n),
                        _ => None,
                    })
                    .collect()
            });

        let controller = obj.controller;
        let source_id = obj.id;
        for n in poisonous_values {
            triggers.push(PendingTrigger {
                source: source_id,
                ability_index: 0, // unused for poisonous triggers
                controller,
                triggering_event: Some(
                    TriggerEvent::SelfDealsCombatDamageToPlayer,
                ),
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
                is_flanking_trigger: false,
                flanking_blocker_id: None,
                is_rampage_trigger: false,
                rampage_n: None,
                is_provoke_trigger: false,
                provoke_target_creature: None,
                is_renown_trigger: false,
                renown_n: None,
                is_poisonous_trigger: true,
                poisonous_n: Some(n),
                poisonous_target_player: Some(damaged_player),
            });
        }
    }
}
```

**IMPORTANT**: The `let CombatDamageTarget::Player(damaged_player) = &assignment.target else { continue; }` pattern here uses `continue` which would skip subsequent keyword checks (Renown) in the same `for assignment in assignments` loop iteration. The Ingest block uses the same pattern. However, since the outer guard already ensures `Player(_)`, the `else` branch is unreachable. Alternatively, use the same `let...else` pattern as Ingest (line 2102-2104) which also uses `continue` safely inside the already-guarded block.

#### 3d: Flush branch

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add `is_poisonous_trigger` branch to `flush_pending_triggers` after the `is_renown_trigger` branch (~line 2717), before the final `else`:
```rust
} else if trigger.is_poisonous_trigger {
    // CR 702.70a: Poisonous N combat damage trigger -- "Whenever this creature
    // deals combat damage to a player, that player gets N poison counters."
    StackObjectKind::PoisonousTrigger {
        source_object: trigger.source,
        target_player: trigger.poisonous_target_player.unwrap_or(trigger.controller),
        poisonous_n: trigger.poisonous_n.unwrap_or(1),
    }
```

#### 3e: Resolution

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handler for `PoisonousTrigger` after the `RenownTrigger` handler (~line 1885), before the countering catch-all arm. Insert before the `StackObjectKind::ActivatedAbility { .. }` catch-all block at line ~1976.

```rust
// CR 702.70a: Poisonous trigger resolves -- give the damaged player
// N poison counters.
//
// CR 603.10: The source creature does NOT need to be on the battlefield
// at resolution time (the trigger is already on the stack).
// The poison counters are given regardless of the source's current state.
//
// Ruling (Virulent Sliver 2021-03-19): "Poisonous 1 causes the player to
// get just one poison counter when a Sliver deals combat damage to them,
// no matter how much damage that Sliver dealt." The N value is fixed.
StackObjectKind::PoisonousTrigger {
    source_object,
    target_player,
    poisonous_n,
} => {
    let controller = stack_obj.controller;

    // Give target_player exactly poisonous_n poison counters.
    if let Some(player) = state.players.get_mut(&target_player) {
        player.poison_counters += poisonous_n;
    }

    // Reuse the existing PoisonCountersGiven event from Infect infrastructure.
    // The event semantics are identical: a player received poison counters from
    // a source object. The only difference is the origin (Poisonous trigger vs.
    // Infect damage replacement), which is transparent to downstream consumers.
    events.push(GameEvent::PoisonCountersGiven {
        player: target_player,
        amount: poisonous_n,
        source: source_object,
    });

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

Also add `PoisonousTrigger` to the counter-spell catch-all match arm (~line 1997):
```rust
| StackObjectKind::PoisonousTrigger { .. }
```
Insert this after the `| StackObjectKind::RenownTrigger { .. }` line in the pattern.

#### 3f: View model + TUI

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
Add to the `stack_object_to_view` match after `RenownTrigger` (~line 487):
```rust
StackObjectKind::PoisonousTrigger { source_object, .. } => {
    ("poisonous_trigger", Some(*source_object))
}
```

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
Add arm to the exhaustive match after `RenownTrigger` (~line 94):
```rust
StackObjectKind::PoisonousTrigger { source_object, .. } => {
    ("Poisonous: ".to_string(), Some(*source_object))
}
```

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/poisonous.rs`
**Pattern**: Follow tests in `/home/airbaggie/scutemob/crates/engine/tests/ingest.rs` for structure (same trigger event, same combat setup). Reuse the `find_object`, `pass_all` helpers. Add a `poison_counters` helper:

```rust
fn poison_counters(state: &GameState, player: PlayerId) -> u32 {
    state
        .players
        .get(&player)
        .map(|p| p.poison_counters)
        .unwrap_or_else(|| panic!("player {:?} not found", player))
}
```

**Tests to write** (6 tests):

1. **`test_702_70a_poisonous_basic_gives_poison_counter`**
   - CR 702.70a -- basic trigger fires.
   - Setup: P1 has a 2/2 creature with `Poisonous(1)`. P2 has no blockers.
   - P1 attacks P2, no blockers, combat damage resolves.
   - Assert: P2 loses 2 life (from normal damage) AND has 1 poison counter.
   - Verifies Poisonous is additive (not a replacement like Infect).

2. **`test_702_70a_poisonous_amount_independent_of_damage`**
   - Virulent Sliver ruling 2021-03-19: N is fixed.
   - Setup: P1 has a 5/5 creature with `Poisonous(1)`.
   - P1 attacks P2, no blockers, combat damage resolves.
   - Assert: P2 loses 5 life AND gets exactly 1 poison counter (not 5).

3. **`test_702_70a_poisonous_blocked_no_trigger`**
   - CR 702.70a: only fires on combat damage to a **player**.
   - Setup: P1 has a 2/2 `Poisonous(1)` creature. P2 has a 3/3 blocker.
   - P1 attacks P2, P2 blocks. All combat damage goes to the blocker.
   - Assert: P2 has 0 poison counters. No Poisonous trigger fired.

4. **`test_702_70b_poisonous_multiple_instances_trigger_separately`**
   - CR 702.70b: Each instance triggers separately.
   - Setup: P1 has a creature with both `Poisonous(1)` and `Poisonous(2)` in its abilities.
   - P1 attacks P2, no blockers.
   - Assert: Two separate triggers fire. After both resolve, P2 has 3 poison counters (1 + 2).
   - Verify by checking that `state.stack_objects` briefly has 2 PoisonousTrigger items.

5. **`test_702_70a_poisonous_kills_via_sba`**
   - CR 104.3d + CR 704.5c: 10+ poison counters = lose.
   - Setup: P2 starts with 9 poison counters (`player_poison(p2, 9)`). P1 has `Poisonous(1)`.
   - P1 attacks P2, no blockers.
   - Assert: After trigger resolves and SBAs check, P2 has 10 poison counters and `has_lost == true`.

6. **`test_702_70a_poisonous_multiplayer_correct_player`**
   - CR 702.70a: trigger targets the specific damaged player.
   - Setup: 4 players. P1 has a `Poisonous(1)` creature. P1 attacks P3.
   - Assert: After trigger resolves, P3 has 1 poison counter. P2 and P4 have 0.

### Step 5: Card Definition (later phase)

**Suggested card**: Custom test creature (e.g., "Poisonous Viper") with `Poisonous(1)` natively.
- Virulent Sliver grants Poisonous via a Sliver lord effect, requiring Layer 6 static ability granting -- too complex for a simple card definition.
- Snake Cult Initiation is an Aura that grants Poisonous 3 to the enchanted creature, which requires Aura grant infrastructure.
- **For unit tests**: `ObjectSpec::creature(p1, "Viper", 2, 2).with_keyword(KeywordAbility::Poisonous(1))` is sufficient and avoids card definition complexity entirely.
- **Card definition deferred** until a simple creature with Poisonous is printed, or until Sliver lord infrastructure exists.

### Step 6: Game Script (later phase)

**Suggested scenario**: P1 controls a 2/2 creature with Poisonous 1. P1 declares attack against P2. P2 does not block. Combat damage resolves: P2 loses 2 life. Poisonous trigger fires and resolves: P2 gets 1 poison counter. Assert both the life total change and the poison counter.
**Subsystem directory**: `test-data/generated-scripts/combat/`
**Sequence number**: Next available after 120 (check existing files at script generation time).

### Step 7: Coverage Doc Update

**File**: `/home/airbaggie/scutemob/docs/mtg-engine-ability-coverage.md`
**Action**: Update the Poisonous row (line ~290) from `none` to `validated`:
```
| Poisonous | 702.70 | P4 | `validated` | state/types.rs, state/hash.rs, state/stubs.rs, state/stack.rs, rules/abilities.rs, rules/resolution.rs | (card name) | combat/NNN | CR 702.70a+b fully enforced; triggered ability on combat damage to player; N poison counters; multiple instances trigger separately; 6 unit tests | N poison counters on combat damage to player |
```

## Interactions to Watch

- **Poisonous + Infect on the same creature**: Infect replaces combat damage to a player with poison counters (no life loss). Poisonous triggers on "deals combat damage to a player." Both fire from the same `CombatDamageDealt` event. The creature deals combat damage (converted to poison counters by Infect), so Poisonous also triggers. Result: player gets damage-amount poison counters from Infect PLUS N poison counters from Poisonous. Both systems coexist without conflict.
- **Poisonous + Toxic on the same creature**: Toxic is static (gives poison counters inline during damage, CR 702.164c). Poisonous is triggered (separate stack object). Both would fire from the same combat damage. No conflict -- Toxic applies immediately, Poisonous goes on the stack and resolves later.
- **Poisonous + full damage prevention**: If all combat damage to a player is prevented (amount == 0 in the assignment), the `CombatDamageDealt` handler skips the assignment (`abilities.rs:2077-2078` guard) and Poisonous does NOT trigger. Correct per CR 603.2g.
- **Poisonous + Protection (DEBT)**: If the defending player has protection from the source's qualities, combat damage is prevented to 0. With 0 damage, Poisonous does not trigger. Correct behavior.
- **Poison counter SBA**: Already implemented at `sba.rs:251-255`. No changes needed. Works for all sources of poison counters (Infect, Poisonous, Proliferate, Toxic).
- **`PoisonCountersGiven` event**: Reuse the existing event from Infect infrastructure (`rules/events.rs:364-371`). The event has `player: PlayerId`, `amount: u32`, `source: ObjectId` -- all three fields match Poisonous resolution needs exactly. The doc comment says "infect damage" but the event semantics are generic.
- **Trample + Poisonous**: If a creature with both trample and Poisonous deals combat damage to both a blocker and the defending player, Poisonous triggers only for the player-damage assignment. The N value is still fixed regardless of how much trample damage went through to the player.
- **Lifelink + Poisonous**: Both are triggered abilities on combat damage. Both trigger from the same `CombatDamageDealt` event. The creature's controller gains life from lifelink; the damaged player also gets N poison counters from Poisonous. No conflict.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `Poisonous(u32)` variant (~5 lines with doc comment) |
| `crates/engine/src/state/hash.rs` | Add KeywordAbility discriminant 83 (~4 lines) + StackObjectKind discriminant 23 (~8 lines) |
| `crates/engine/src/state/stubs.rs` | Add 3 fields to `PendingTrigger` (~15 lines) |
| `crates/engine/src/state/stack.rs` | Add `PoisonousTrigger` variant (~20 lines) |
| `crates/engine/src/rules/abilities.rs` | Trigger dispatch in `CombatDamageDealt` (~50 lines) + flush branch (~7 lines) + `is_poisonous_trigger: false` in ~15 PendingTrigger literals (~45 lines) |
| `crates/engine/src/rules/resolution.rs` | `PoisonousTrigger` resolution (~20 lines) + counter-spell arm (~1 line) |
| `tools/replay-viewer/src/view_model.rs` | Keyword string (~1 line) + stack object view (~3 lines) |
| `tools/tui/src/play/panels/stack_view.rs` | Stack view arm (~3 lines) |
| `crates/engine/tests/poisonous.rs` | New test file with 6 tests (~350 lines) |

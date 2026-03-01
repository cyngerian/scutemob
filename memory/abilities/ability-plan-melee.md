# Ability Plan: Melee

**Generated**: 2026-03-01 (re-planned from scratch)
**CR**: 702.121 (NOT 702.122 -- the batch plan and coverage doc both have the wrong number; 702.122 is Crew)
**Priority**: P4
**Similar abilities studied**: Rampage (`KeywordAbility::Rampage`, `StackObjectKind::RampageTrigger` in `stack.rs:465`, custom resolution in `resolution.rs:1732`), Training (`KeywordAbility::Training`, `SelfAttacks` trigger via `builder.rs:497`, tagged in `abilities.rs:1508`), Dethrone (`SelfAttacksPlayerWithMostLife` pattern in `abilities.rs:1471`)

**NOTE**: The ability coverage doc (`docs/mtg-engine-ability-coverage.md`) and the batch
plan (`docs/ability-batch-plan.md`) both list Melee as CR 702.122, but that is **Crew**.
The correct CR number for Melee is **702.121**. This is another instance of gotcha #36
(batch-plan CR numbers can be wrong). The coverage doc must be corrected when marking
this ability as validated.

**DISCRIMINANT COORDINATION**: Toxic is being planned in parallel and will take
KeywordAbility discriminant **83**. Melee must use discriminant **84** to avoid collision.
StackObjectKind discriminant is **23** (MeleeTrigger), TriggerEvent is NOT needed
(Melee reuses existing `SelfAttacks`).

## CR Rule Text

```
702.121. Melee

702.121a Melee is a triggered ability. "Melee" means "Whenever this creature attacks,
it gets +1/+1 until end of turn for each opponent you attacked with a creature this combat."

702.121b If a creature has multiple instances of melee, each triggers separately.
```

## Key Edge Cases

- **Bonus is determined at resolution time** (ruling 2016-08-23 on Adriana, Captain of
  the Guard): "You determine the size of the bonus as the melee ability resolves. Count
  each opponent that you attacked with one or more creatures." This means we compute the
  count from `state.combat` at resolution time, not at trigger-collection time. This
  matches the Rampage pattern (custom StackObjectKind with resolution-time computation).

- **Only counts opponents (players), NOT planeswalkers** (ruling 2016-08-23): "Melee
  will trigger if the creature with melee attacks a planeswalker. However, the effect
  counts only opponents (and not planeswalkers) that you attacked with a creature when
  determining the bonus." Attacking a planeswalker does NOT count that planeswalker's
  controller as an "attacked opponent" for Melee purposes. Only `AttackTarget::Player(pid)`
  entries count.

- **Number of creatures attacking each player does not matter** (ruling 2016-08-23):
  "It doesn't matter how many creatures you attacked a player with, only that you attacked
  a player with at least one creature." The count is distinct opponents, not total
  attack-target pairs.

- **Creatures that entered the battlefield attacking don't count** (ruling 2016-08-23):
  "Creatures that enter the battlefield attacking were never declared as attackers, so
  they won't count toward melee's effect." This is naturally handled because
  `state.combat.attackers` is populated from `DeclareAttackers` commands only.

- **Creatures with melee entering the battlefield attacking don't trigger** (ruling
  2016-08-23): "Similarly, if a creature with melee enters the battlefield attacking,
  melee won't trigger." This is naturally handled because the `SelfAttacks` trigger
  event fires only from `AttackersDeclared` game events.

- **It does not matter if attackers are still attacking or on the battlefield** (ruling
  2016-08-23): "It doesn't matter if the attacking creatures are still attacking or even
  if they are still on the battlefield." The `state.combat.attackers` OrdMap retains all
  declared attackers even if they leave the battlefield, so this is naturally correct.

- **It does not matter if the attacked opponent is still in the game** (ruling 2016-08-23):
  The count includes opponents who have been eliminated after attackers were declared but
  before Melee resolves. The `state.combat.attackers` preserves the original targets.

- **Multiple instances trigger separately** (CR 702.121b): Each instance fires its own
  trigger, each computing the bonus independently. A creature with two Melee keywords
  gets double the bonus (two separate +N/+N effects stacking).

- **Multiplayer (Commander) is the primary use case**: In a 4-player Commander game, the
  attacking player can attack up to 3 opponents. If they attack all 3, the Melee bonus
  is +3/+3. This is where Melee shines and must be tested.

- **The bonus is +1/+1 per opponent** (CR 702.121a): Not a parameterized N like Rampage.
  Every Melee instance gives +1/+1 per opponent attacked. The total is always
  `opponents_attacked_count * 1`.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: StackObjectKind + builder.rs trigger auto-generation
- [ ] Step 3: Trigger wiring in abilities.rs AttackersDeclared handler
- [ ] Step 4: Resolution in resolution.rs
- [ ] Step 5: Unit tests
- [ ] Step 6: Card definition
- [ ] Step 7: Game script

## Implementation Steps

### Step 1: Enum Variant + Hash + View Model

#### 1a. KeywordAbility::Melee

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Melee` variant after `Training` (line 700, before the closing `}` of the enum on line 701).
**Doc comment**:
```rust
/// CR 702.121: Melee -- triggered ability.
/// "Whenever this creature attacks, it gets +1/+1 until end of turn for
/// each opponent you attacked with a creature this combat."
///
/// CR 702.121b: Multiple instances trigger separately.
///
/// Implemented via a custom `StackObjectKind::MeleeTrigger` because the
/// bonus is computed at resolution time from `state.combat` (ruling
/// 2016-08-23: "You determine the size of the bonus as the melee ability
/// resolves"). The trigger fires on `TriggerEvent::SelfAttacks`.
Melee,
```

#### 1b. Hash for KeywordAbility::Melee

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Location**: In `HashInto for KeywordAbility` impl, after `Training` (discriminant 82,
line 494). Insert before the closing `}` on line 495.
**Discriminant**: **84** (83 is reserved for Toxic, being implemented in parallel).
```rust
// Melee (discriminant 84) -- CR 702.121
KeywordAbility::Melee => 84u8.hash_into(hasher),
```

**IMPORTANT**: If Toxic has already been committed with discriminant 83 by the time this
runs, Melee uses 84 as planned. If Toxic has NOT been committed yet and Melee is
implemented first, Melee should still use 84 to avoid the collision when Toxic claims 83.

#### 1c. View Model format_keyword

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Location**: In `format_keyword` function, after `Training` arm (line 706):
```rust
KeywordAbility::Melee => "Melee".to_string(),
```

#### 1d. Match Arm Audit

Grep for exhaustive `match` on `KeywordAbility` across the codebase.
Add new arms to every match. Known locations:
- `state/hash.rs`: `HashInto for KeywordAbility` (1b above)
- `tools/replay-viewer/src/view_model.rs`: `format_keyword` (1c above)
- Any other exhaustive matches will cause a compile error -- fix them all.

### Step 2: StackObjectKind::MeleeTrigger + PendingTrigger Tag + builder.rs

#### 2a. StackObjectKind::MeleeTrigger

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `MeleeTrigger` variant after `RenownTrigger` (line 513, before the closing `}` on line 514).
**Doc comment**:
```rust
/// CR 702.121a: Melee triggered ability on the stack.
///
/// "Whenever this creature attacks, it gets +1/+1 until end of turn for
/// each opponent you attacked with a creature this combat."
///
/// When this trigger resolves:
/// 1. Count distinct opponents (players) targeted by any attacker in
///    `state.combat.attackers` (only `AttackTarget::Player` entries count,
///    NOT planeswalkers -- ruling 2016-08-23).
/// 2. If count > 0 and source is on the battlefield, apply +count/+count
///    as a ContinuousEffect (UntilEndOfTurn) in Layer 7c (PtModify).
///
/// CR 702.121b: Multiple instances trigger separately (each creates its
/// own MeleeTrigger; each computes the bonus independently).
///
/// The bonus is computed at resolution time (ruling 2016-08-23: "You
/// determine the size of the bonus as the melee ability resolves").
/// `state.combat.attackers` retains all declared attackers even if they
/// leave the battlefield, so the count is stable.
MeleeTrigger {
    source_object: ObjectId,
},
```

#### 2b. Hash for StackObjectKind::MeleeTrigger

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Location**: In `HashInto for StackObjectKind` impl, after `RenownTrigger`
(discriminant 22, line 1461). Insert before the closing `}` on line 1462.
Next available StackObjectKind discriminant is **23**.
```rust
// MeleeTrigger (discriminant 23) -- CR 702.121a
StackObjectKind::MeleeTrigger {
    source_object,
} => {
    23u8.hash_into(hasher);
    source_object.hash_into(hasher);
}
```

#### 2c. PendingTrigger fields

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add `is_melee_trigger` field after `renown_n` (line 299, before the closing `}` of `PendingTrigger` on line 300).
```rust
/// CR 702.121a: If true, this pending trigger is a Melee trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::MeleeTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`.
/// The bonus is computed at resolution time from `state.combat`.
#[serde(default)]
pub is_melee_trigger: bool,
```

**CRITICAL**: There are **19 PendingTrigger construction sites** across the codebase that
must each get `is_melee_trigger: false` added:
- `crates/engine/src/rules/abilities.rs`: 12 sites
- `crates/engine/src/effects/mod.rs`: 2 sites
- `crates/engine/src/rules/turn_actions.rs`: 3 sites
- `crates/engine/src/rules/resolution.rs`: 1 site
- `crates/engine/src/rules/miracle.rs`: 1 site

Search for `is_renown_trigger: false` to find all sites; add `is_melee_trigger: false`
after each `renown_n: None` line.

#### 2d. Hash for PendingTrigger::is_melee_trigger

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Location**: In `HashInto for PendingTrigger` impl, after `renown_n` (line 1132).
Insert before the closing `}` on line 1133.
```rust
// CR 702.121a: is_melee_trigger -- melee attack trigger marker
self.is_melee_trigger.hash_into(hasher);
```

#### 2e. builder.rs trigger auto-generation

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Add a Melee block after the Training block (after line 511, approximately).
**Pattern**: Follow the Rampage block at lines 748-767 (uses `effect: None` and custom
StackObjectKind), but trigger on `SelfAttacks` instead of `SelfBecomesBlocked`.

```rust
// CR 702.121a: Melee -- "Whenever this creature attacks, it gets
// +1/+1 until end of turn for each opponent you attacked with a
// creature this combat."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.121b).
// The effect is None because resolution is handled by the custom
// MeleeTrigger StackObjectKind -- the bonus is computed at resolution
// time from combat state (ruling 2016-08-23).
if matches!(kw, KeywordAbility::Melee) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacks,
        intervening_if: None,
        description: "Melee (CR 702.121a): Whenever this creature attacks, it gets \
                      +1/+1 until end of turn for each opponent you attacked with \
                      a creature this combat.".to_string(),
        effect: None, // Custom resolution via MeleeTrigger
    });
}
```

**Note**: Unlike Training (which uses a dedicated `SelfAttacksWithGreaterPowerAlly`
trigger event because it has a condition), Melee uses the standard `SelfAttacks` event.
The trigger fires unconditionally when the creature attacks -- the bonus computation
happens at resolution. The description starts with "Melee" so the AttackersDeclared
handler in abilities.rs can identify and tag it.

#### 2f. TUI stack_view.rs

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
**Action**: Add `MeleeTrigger` arm after `RenownTrigger` arm (line 94, before the closing `};`):
```rust
StackObjectKind::MeleeTrigger { source_object, .. } => {
    ("Melee: ".to_string(), Some(*source_object))
}
```

#### 2g. Replay viewer view_model.rs StackObjectKind display

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add `MeleeTrigger` arm in the StackObjectKind display match (after
`RenownTrigger` arm at line 487, before the closing `}`):
```rust
StackObjectKind::MeleeTrigger { source_object, .. } => {
    ("melee_trigger", Some(*source_object))
}
```

### Step 3: Trigger Wiring in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Location**: In the `GameEvent::AttackersDeclared` handler, inside the
`for (attacker_id, attack_target) in attackers` loop, specifically in the trigger
tagging section (after the Provoke block, around line 1469). Insert tag logic for
Melee triggers.

Also, in the flush-to-stack section (around line 2708, after `is_renown_trigger`
else-if block), add the `is_melee_trigger` branch.

#### 3a. Tag Melee triggers in AttackersDeclared handler

**Placement**: After the Myriad tagging block (around line 1419), alongside the
other SelfAttacks trigger taggers. Insert inside the per-attacker loop, operating
on `triggers[pre_len..]`.

**Logic**:
```rust
// CR 702.121a/b: Tag melee triggers for special stack handling.
// A SelfAttacks trigger is a melee trigger if its triggered ability
// description starts with "Melee" (set by builder.rs). Unlike
// Rampage which needs an N value, Melee always gives +1/+1 per
// opponent attacked -- no parameter to carry.
for t in &mut triggers[pre_len..] {
    if let Some(obj) = state.objects.get(&t.source) {
        if let Some(ta) =
            obj.characteristics.triggered_abilities.get(t.ability_index)
        {
            if ta.effect.is_none() && ta.description.starts_with("Melee") {
                t.is_melee_trigger = true;
            }
        }
    }
}
```

**Pattern**: This matches the Myriad tagging pattern at lines 1409-1419, which also
checks `ta.effect.is_none()` and `ta.description.starts_with("Myriad")`.

#### 3b. Flush MeleeTrigger to stack

**Placement**: In the flush-to-stack if-else chain in `flush_pending_triggers`
(around line 2708), add after the `is_renown_trigger` branch:

```rust
} else if trigger.is_melee_trigger {
    // CR 702.121a: Melee SelfAttacks trigger.
    // Bonus computed at resolution time from state.combat (ruling 2016-08-23).
    StackObjectKind::MeleeTrigger {
        source_object: trigger.source,
    }
```

#### 3c. Countered-trigger match arm

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Location**: In the "countering abilities" catch-all match (line 1997), add
`StackObjectKind::MeleeTrigger { .. }` to the pattern list before `=> {`.

```rust
| StackObjectKind::RenownTrigger { .. }
| StackObjectKind::MeleeTrigger { .. } => {
```

### Step 4: Resolution in resolution.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add a `StackObjectKind::MeleeTrigger` resolution arm after the
`RampageTrigger` block (around line 1789). Place near the other combat-modifier triggers.
**Pattern**: Follow `RampageTrigger` resolution at lines 1732-1789.
**CR**: 702.121a -- bonus computed at resolution time.

```rust
// CR 702.121a: Melee -- "Whenever this creature attacks, it gets +1/+1
// until end of turn for each opponent you attacked with a creature this
// combat."
// Ruling 2016-08-23: "You determine the size of the bonus as the melee
// ability resolves. Count each opponent that you attacked with one or
// more creatures."
// Ruling 2016-08-23: Only opponents (players) count, NOT planeswalkers.
// Only `AttackTarget::Player(pid)` entries in state.combat.attackers count.
StackObjectKind::MeleeTrigger {
    source_object,
} => {
    let controller = stack_obj.controller;

    // Count distinct opponents attacked with creatures (players only).
    // CR 702.121a: "for each opponent you attacked with a creature"
    // Ruling: "It doesn't matter how many creatures you attacked a player
    // with, only that you attacked a player with at least one creature."
    let opponents_attacked = state
        .combat
        .as_ref()
        .map(|c| {
            c.attackers
                .values()
                .filter_map(|target| {
                    if let AttackTarget::Player(pid) = target {
                        Some(*pid)
                    } else {
                        None
                    }
                })
                .collect::<im::OrdSet<PlayerId>>()
                .len()
        })
        .unwrap_or(0);

    let bonus = opponents_attacked as i32;

    if bonus > 0 {
        // Only apply if the source is still on the battlefield.
        let source_alive = state
            .objects
            .get(&source_object)
            .map(|obj| obj.zone == ZoneId::Battlefield)
            .unwrap_or(false);

        if source_alive {
            // Register the +bonus/+bonus continuous effect (Layer 7c, UntilEndOfTurn).
            let eff_id = state.next_object_id().0;
            let ts = state.timestamp_counter;
            state.timestamp_counter += 1;
            state.continuous_effects.push_back(
                crate::state::continuous_effect::ContinuousEffect {
                    id: crate::state::continuous_effect::EffectId(eff_id),
                    source: None,
                    timestamp: ts,
                    layer: crate::state::continuous_effect::EffectLayer::PtModify,
                    duration:
                        crate::state::continuous_effect::EffectDuration::UntilEndOfTurn,
                    filter: crate::state::continuous_effect::EffectFilter::SingleObject(
                        source_object,
                    ),
                    modification:
                        crate::state::continuous_effect::LayerModification::ModifyBoth(
                            bonus,
                        ),
                    is_cda: false,
                },
            );
        }
    }

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Key differences from Rampage resolution**:
- Rampage: `bonus = (blockers_for(source) - 1) * rampage_n` (reads blocker count)
- Melee: `bonus = count of distinct Player targets in combat.attackers` (reads attacker targets)
- Rampage carries `rampage_n` parameter; Melee has no parameter (always +1/+1 per opponent)

**Note on `im::OrdSet`**: Using `collect::<im::OrdSet<PlayerId>>()` for deduplication
is clean and deterministic. Alternatively, a `std::collections::HashSet` or manual
dedup would work, but `OrdSet` is already available from `im-rs` and idiomatic in this
codebase.

### Step 5: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/melee.rs`
**Pattern**: Follow `/home/airbaggie/scutemob/crates/engine/tests/rampage.rs`

**Module header**:
```rust
//! Melee keyword ability tests (CR 702.121).
//!
//! Melee is a triggered ability: "Whenever this creature attacks, it gets
//! +1/+1 until end of turn for each opponent you attacked with a creature
//! this combat." (CR 702.121a)
//!
//! Key rules verified:
//! - Trigger fires when the creature attacks (CR 702.121a).
//! - Bonus = number of distinct opponents attacked with creatures (CR 702.121a).
//! - Only opponents (players) count, not planeswalkers (ruling 2016-08-23).
//! - Bonus computed at resolution time (ruling 2016-08-23).
//! - Multiple instances trigger separately (CR 702.121b).
//! - Bonus is +N/+N (both power AND toughness) until end of turn.
//! - No bonus if only attacking planeswalkers (no Player targets).
//! - Multiplayer: attacking 3 opponents gives +3/+3.
```

**Imports**:
```rust
use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec,
    PlayerId, Step,
};
```

**Helpers** (copy from rampage.rs):
```rust
fn find_object(state: &GameState, name: &str) -> ObjectId { ... }
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) { ... }
```

**Tests to write** (7 tests):

1. **`test_702_121a_melee_basic_one_opponent_attacked`** -- CR 702.121a
   - 2 players: P1, P2.
   - P1 has 2/2 creature with Melee on battlefield.
   - P1 declares the creature as an attacker targeting P2.
   - Assert: `AbilityTriggered` event from melee source.
   - Assert: `state.stack_objects.len() == 1` (melee trigger on stack).
   - Pass all to resolve trigger.
   - Assert: `calculate_characteristics` shows power=3, toughness=3 (+1/+1 for 1 opponent).

2. **`test_702_121a_melee_multiplayer_two_opponents`** -- CR 702.121a + multiplayer
   - 4 players: P1, P2, P3, P4.
   - P1 has 2/2 creature with Melee (attacks P2), and a 3/3 vanilla creature (attacks P3).
   - P1 declares both as attackers (melee creature->P2, vanilla->P3).
   - Assert: melee trigger fires.
   - All 4 players pass priority to resolve.
   - Assert: melee creature gets +2/+2 (2 distinct opponents attacked), showing P/T = 4/4.

3. **`test_702_121a_melee_multiplayer_three_opponents`** -- CR 702.121a + full multiplayer
   - 4 players: P1, P2, P3, P4.
   - P1 has 2/2 creature with Melee, plus three vanilla creatures.
   - P1 declares: melee creature->P2, vanilla1->P3, vanilla2->P4, vanilla3->P2.
   - Assert: melee trigger fires.
   - All 4 players pass. Resolve trigger.
   - Assert: melee creature gets +3/+3 (3 distinct opponents: P2, P3, P4), showing
     P/T = 5/5.

4. **`test_702_121a_melee_does_not_count_planeswalker_attacks`** -- ruling 2016-08-23
   - 2 players: P1, P2.
   - P1 has 2/2 creature with Melee. P2 controls a planeswalker object on battlefield.
   - P1 declares the melee creature attacking the planeswalker (not the player).
   - Assert: trigger fires (melee triggers on any attack).
   - Resolve trigger.
   - Assert: melee creature gets +0/+0 (no Player target, only planeswalker).
     Effective P/T stays 2/2.
   - This verifies the "only opponents, not planeswalkers" ruling.

5. **`test_702_121b_melee_multiple_instances`** -- CR 702.121b
   - 2 players: P1, P2.
   - P1 has 2/2 creature with TWO Melee keywords (via `.with_keyword` twice).
   - P1 declares the creature attacking P2.
   - Assert: `state.stack_objects.len() == 2` (two separate triggers).
   - Resolve both (pass_all twice).
   - Assert: creature effective P/T = 4/4 (two +1/+1 bonuses from 2 Melee instances,
     each giving +1/+1 for 1 opponent attacked).

6. **`test_702_121a_melee_source_leaves_battlefield_no_bonus`** -- CR 603.10 + ruling
   - 2 players: P1, P2.
   - P1 has 2/2 creature with Melee. P2 has no blockers.
   - P1 declares the creature attacking P2. Melee trigger goes on stack.
   - Before trigger resolves, destroy the melee creature (move to graveyard).
   - Resolve trigger.
   - Assert: no continuous effect applied (source not on battlefield).
   - Assert: no crash.

7. **`test_702_121a_melee_attacking_alone_still_counts`** -- CR 702.121a (single attacker)
   - 2 players: P1, P2.
   - P1 has 2/2 creature with Melee. Only creature on P1's battlefield.
   - P1 declares the creature as sole attacker targeting P2.
   - Assert: trigger fires.
   - Resolve trigger.
   - Assert: creature gets +1/+1 (1 opponent attacked with 1 creature).
     P/T = 3/3.
   - This verifies that attacking alone still counts 1 opponent (unlike Exalted
     which only triggers when attacking alone, Melee triggers on any attack).

### Step 6: Card Definition (later phase)

**Suggested card**: Wings of the Guard
- **Name**: Wings of the Guard
- **Cost**: {1}{W}
- **Type**: Creature -- Bird
- **P/T**: 1/1
- **Oracle**: "Flying\nMelee (Whenever this creature attacks, it gets +1/+1 until end
  of turn for each opponent you attacked this combat.)"
- **Keywords**: `[KeywordAbility::Flying, KeywordAbility::Melee]`
- **Color identity**: ["W"]

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/wings_of_the_guard.rs`
**Action**: Use `card-definition-author` agent.

**Why this card**: Simple creature with Melee + Flying (both validated keywords, assuming
Flying is already validated). Low mana cost, no complex abilities beyond keywords. Good
for game script validation. Adriana, Captain of the Guard is the iconic Melee card but
has a second ability ("Other creatures you control have melee") that requires granting
keywords to other creatures -- a more complex interaction better suited for a later card.

### Step 7: Game Script (later phase)

**Suggested scenario**: "Melee trigger in 4-player Commander: attack 3 opponents"
**Subsystem directory**: `test-data/generated-scripts/combat/`
**Sequence number**: Use next available in the combat directory (likely 121+).

**Script outline**:
1. 4 players: P1 (40 life), P2 (40 life), P3 (40 life), P4 (40 life).
2. P1 controls Wings of the Guard (1/1, Flying, Melee) and three vanilla creatures
   on the battlefield.
3. Advance to Declare Attackers step.
4. P1 declares: Wings of the Guard -> P2, vanilla1 -> P3, vanilla2 -> P4, vanilla3 -> P2.
5. Assert: stack has 1 trigger item (Melee from Wings of the Guard).
6. All 4 players pass priority -- trigger resolves.
7. Assert: Wings of the Guard effective P/T is 4/4 (+3/+3 for attacking 3 distinct
   opponents: P2, P3, P4).

**File**: Use `game-script-generator` agent.

## Interactions to Watch

- **Melee + Exalted**: A creature with both Melee and Exalted benefits from both. If it
  attacks alone, Exalted fires (one creature attacking alone) AND Melee fires (creature
  attacks). With only 1 opponent attacked, Melee gives +1/+1 and Exalted gives +1/+1
  (from each permanent with Exalted). No conflict.

- **Melee + Myriad**: Myriad creates token copies tapped and attacking each other
  opponent. These tokens entered the battlefield attacking and were NOT declared as
  attackers. Per ruling 2016-08-23, they do NOT count for Melee's opponent tally.
  However, the Melee trigger fires on the original creature's declaration, and the
  opponent count at resolution includes only declared attack targets. This means Myriad
  does not boost Melee's count.

- **Melee + Goad**: Goaded creatures must attack if able, potentially spreading attacks
  across opponents. This naturally increases Melee's bonus in multiplayer if the goaded
  creatures attack different opponents.

- **Melee + Humility**: Same systemic concern as all triggered keyword abilities.
  Humility removes abilities in Layer 6, but `collect_triggers_for_event` reads raw
  `obj.characteristics.triggered_abilities` (pre-layer). This means Humility may not
  suppress Melee triggers. Not blocking -- systemic issue tracked separately.

- **Melee + Combat removal (fog, instant-speed removal)**: If a creature is removed from
  combat after attackers are declared but before Melee resolves, the original attack
  targets in `state.combat.attackers` are preserved. The bonus is still based on the
  declaration-time attackers. Per ruling: "It doesn't matter if the attacking creatures
  are still attacking or even if they are still on the battlefield."

- **Melee bonus is UntilEndOfTurn (temporary)**: The +N/+N is a continuous effect that
  expires at cleanup (CR 514.2). It is NOT a counter. On the next turn, the bonus is
  gone. This differs from Training (which places permanent +1/+1 counters).

- **Multiple Melee creatures**: Each Melee creature gets its own trigger, each computing
  the same opponent count independently. If P1 attacks P2 and P3 with two Melee
  creatures, both get +2/+2.

## Summary of Files Modified

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Melee` variant |
| `crates/engine/src/state/stack.rs` | Add `StackObjectKind::MeleeTrigger` variant |
| `crates/engine/src/state/stubs.rs` | Add `is_melee_trigger` field to `PendingTrigger` |
| `crates/engine/src/state/hash.rs` | Add hash arms for `KeywordAbility::Melee` (disc 84), `StackObjectKind::MeleeTrigger` (disc 23), `PendingTrigger::is_melee_trigger` |
| `crates/engine/src/state/builder.rs` | Auto-generate `TriggeredAbilityDef` for Melee keyword (SelfAttacks, effect: None) |
| `crates/engine/src/rules/abilities.rs` | Tag Melee triggers in AttackersDeclared handler + flush MeleeTrigger to stack; add `is_melee_trigger: false` to 12 PendingTrigger construction sites |
| `crates/engine/src/rules/resolution.rs` | MeleeTrigger resolution: count distinct opponents, apply +N/+N continuous effect; add to countered-trigger catch-all; add `is_melee_trigger: false` to 1 PendingTrigger site |
| `crates/engine/src/effects/mod.rs` | Add `is_melee_trigger: false` to 2 PendingTrigger construction sites |
| `crates/engine/src/rules/turn_actions.rs` | Add `is_melee_trigger: false` to 3 PendingTrigger construction sites |
| `crates/engine/src/rules/miracle.rs` | Add `is_melee_trigger: false` to 1 PendingTrigger construction site |
| `tools/replay-viewer/src/view_model.rs` | Add `format_keyword` arm for Melee + StackObjectKind display arm |
| `tools/tui/src/play/panels/stack_view.rs` | Add `MeleeTrigger` arm in exhaustive match |
| `crates/engine/tests/melee.rs` | New test file with 7 tests |
| `crates/engine/src/cards/defs/wings_of_the_guard.rs` | Wings of the Guard card definition (Step 6) |
| `test-data/generated-scripts/combat/` | Melee game script (Step 7) |
| `docs/mtg-engine-ability-coverage.md` | Correct CR from 702.122 to 702.121; update status to validated |

## PendingTrigger Construction Site Inventory

The 19 sites that need `is_melee_trigger: false` (find via `rg 'is_renown_trigger: false'`):

| File | Count | Context |
|------|-------|---------|
| `rules/abilities.rs` | 12 | collect_triggers_for_event calls, evoke trigger, ETB-triggered, cycling, etc. |
| `effects/mod.rs` | 2 | Effect-driven triggers |
| `rules/turn_actions.rs` | 3 | Turn-based triggers |
| `rules/resolution.rs` | 1 | Resolution-driven triggers |
| `rules/miracle.rs` | 1 | Miracle trigger |
| **Total** | **19** | |

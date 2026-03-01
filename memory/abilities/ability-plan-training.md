# Ability Plan: Training

**Generated**: 2026-03-01
**CR**: 702.149
**Priority**: P4
**Similar abilities studied**: Dethrone (`KeywordAbility::Dethrone`, `TriggerEvent::SelfAttacksPlayerWithMostLife` in `builder.rs:473`, `abilities.rs:1373`), Evolve (`KeywordAbility::Evolve`, power comparison pattern in `abilities.rs:1058`, resolution re-check in `resolution.rs:1066`)

**NOTE**: The ability coverage doc (`docs/mtg-engine-ability-coverage.md`) lists Training as CR 702.150, but the actual CR number is **702.149**. CR 702.150 is Compleated. This is another instance of gotcha #36 (batch-plan CR numbers can be wrong).

## CR Rule Text

```
702.149. Training

702.149a Training is a triggered ability. "Training" means "Whenever this creature and at
least one other creature with power greater than this creature's power attack, put a +1/+1
counter on this creature."

702.149b If a creature has multiple instances of training, each triggers separately.

702.149c Some creatures with training have abilities that trigger when they train. "When
this creature trains" means "When a resolving training ability puts one or more +1/+1
counters on this creature."
```

## Key Edge Cases

- **Trigger condition is checked at declaration time only** (ruling 2021-11-19): "A
  creature's training ability triggers only when both that creature and a creature with
  greater power are declared as attackers. Increasing a creature's power after attackers
  are declared won't cause a training ability to trigger." This means we check the
  condition in the `AttackersDeclared` handler, not at resolution time.

- **No intervening-if at resolution** (ruling 2021-11-19): "Once a creature's training
  ability has triggered, destroying the other attacking creature or reducing its power
  won't stop the creature with training from getting a +1/+1 counter." The counter is
  placed unconditionally at resolution. This is simpler than Evolve (which has an
  intervening-if re-check at resolution per CR 603.4).

- **Power comparison is strictly greater** (CR 702.149a): "power greater than this
  creature's power" -- not greater-or-equal. A 2/2 Training creature attacking alongside
  another 2/2 does NOT trigger. The companion must have strictly greater power.

- **The comparison is against the Training creature's OWN power at declaration time**:
  Use `calculate_characteristics` for the layer-aware power value (accounts for continuous
  effects, counters, etc.).

- **Multiple instances trigger separately** (CR 702.149b): A creature with two instances
  of Training gets two +1/+1 counters (two separate triggers on the stack).

- **"When this creature trains" (CR 702.149c)**: Some cards have a linked triggered
  ability that fires when Training resolves. This is a later-phase concern -- the base
  Training implementation just places counters. Cards like Savior of Ollenbock can use
  a custom `TriggeredAbilityDef` with an appropriate trigger event. For now, we only
  need the base Training keyword behavior.

- **The +1/+1 counter goes on the Training creature itself** (CR 702.149a), not the
  co-attacking creature.

- **The co-attacker must be controlled by the same player**: Since both creatures must
  "attack" together, they are both in the `attackers` list of the `DeclareAttackers`
  command issued by the same player. Multiple attackers in the same declaration are
  always controlled by the same player. The check is implicit.

- **Multiple Training creatures can each trigger independently**: If P1 attacks with a
  1/1 Training creature, a 2/2 Training creature, and a 3/3 vanilla, the 1/1 triggers
  (3/3 > 1 and 2/2 > 1) and the 2/2 triggers (3/3 > 2). Each gets its own trigger.

- **Multiplayer**: No special multiplayer considerations beyond the standard Commander
  attack rules. The trigger condition only cares about co-attackers' power, not about
  which player is being attacked. Different attackers can attack different players and
  Training still checks co-attackers' power.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Trigger auto-generation in builder.rs
- [ ] Step 3: Trigger wiring in abilities.rs AttackersDeclared handler
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant + TriggerEvent Variant

#### 1a. KeywordAbility::Training

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Training` variant after `Ingest` (line 638).
**Doc comment**:
```rust
/// CR 702.149: Training -- "Whenever this creature and at least one other creature
/// with power greater than this creature's power attack, put a +1/+1 counter on
/// this creature."
///
/// Implemented as a triggered ability. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// Multiple instances each trigger separately (CR 702.149b).
Training,
```

#### 1b. TriggerEvent::SelfAttacksWithGreaterPowerAlly

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add new variant to `TriggerEvent` enum after `ControllerProliferates`
(line 198).

**Why a dedicated variant**: Training's condition (co-attacker with strictly greater
power) cannot reuse `SelfAttacks` because `SelfAttacks` is already collected for ALL
attackers unconditionally (for Annihilator, Myriad, etc.). Adding Training's
`TriggeredAbilityDef` with `trigger_on: SelfAttacks` would fire the trigger on every
attack regardless of whether any co-attacker has greater power. A dedicated variant
lets us call `collect_triggers_for_event` with `SelfAttacksWithGreaterPowerAlly` ONLY
when the power condition is met, matching the Dethrone pattern
(`SelfAttacksPlayerWithMostLife`).

```rust
/// CR 702.149a: Triggers when this creature attacks alongside at least one
/// other attacking creature with strictly greater power.
/// The power comparison is done at trigger-collection time in
/// `rules/abilities.rs` AttackersDeclared handler.
/// Used by the Training keyword.
SelfAttacksWithGreaterPowerAlly,
```

#### 1c. Hash for KeywordAbility::Training

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Location**: In `HashInto for KeywordAbility` impl, after `Ingest` (discriminant 75,
line 468). Next available discriminant is **82** (per batch plan: Flanking=76,
Bushido=77, Rampage=78, Provoke=79, Afflict=80, Renown=81, Training=82).
```rust
// Training (discriminant 82) -- CR 702.149
KeywordAbility::Training => 82u8.hash_into(hasher),
```

#### 1d. Hash for TriggerEvent::SelfAttacksWithGreaterPowerAlly

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Location**: In `HashInto for TriggerEvent` impl, after `ControllerProliferates`
(discriminant 17, line 1146). Next available discriminant is **18**.
```rust
// CR 702.149a: Training "attacks with greater power ally" trigger -- discriminant 18
TriggerEvent::SelfAttacksWithGreaterPowerAlly => 18u8.hash_into(hasher),
```

#### 1e. View Model format_keyword

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Location**: In `format_keyword` function, add after the last keyword arm:
```rust
KeywordAbility::Training => "Training".to_string(),
```

#### 1f. Match Arm Audit

Grep for exhaustive `match` on `KeywordAbility` and `TriggerEvent` across the codebase.
Add new arms to every match. Known locations:
- `state/hash.rs`: `HashInto for KeywordAbility` (1c above)
- `state/hash.rs`: `HashInto for TriggerEvent` (1d above)
- `tools/replay-viewer/src/view_model.rs`: `format_keyword` (1e above)
- Any other exhaustive matches will cause a compile error -- fix them all.

### Step 2: Trigger Auto-Generation in builder.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Add a Training block after the Dethrone block (after line 487).
**Pattern**: Follow Dethrone at lines 465-487.

```rust
// CR 702.149a: Training -- "Whenever this creature and at least one
// other creature with power greater than this creature's power attack,
// put a +1/+1 counter on this creature."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.149b).
// The trigger uses SelfAttacksWithGreaterPowerAlly, a dedicated event
// that is only dispatched in abilities.rs when a co-attacker with
// strictly greater power exists. This avoids unconditional firing on
// all attacks.
if matches!(kw, KeywordAbility::Training) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacksWithGreaterPowerAlly,
        intervening_if: None,
        description: "Training (CR 702.149a): Whenever this creature and at \
                      least one other creature with greater power attack, put \
                      a +1/+1 counter on this creature.".to_string(),
        effect: Some(Effect::AddCounter {
            target: EffectTarget::Source,
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        }),
    });
}
```

**CR**: 702.149a -- "put a +1/+1 counter on this creature"

**Note**: No new `StackObjectKind` variant needed. The `TriggeredAbilityDef` has
`effect: Some(AddCounter)`, so it goes through the standard `TriggeredAbility` stack
object path in `flush_pending_triggers`. Resolution uses the standard
`StackObjectKind::TriggeredAbility` arm which executes the effect.

### Step 3: Trigger Wiring in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Location**: In the `GameEvent::AttackersDeclared` handler, inside the
`for (attacker_id, attack_target) in attackers` loop (line 1333).

**Placement**: After the Dethrone block (line 1408), BEFORE the closing `}` of the
per-attacker loop (line 1409). Insert inside the per-attacker loop so each attacker
with Training gets its own trigger check.

**Logic**:
```rust
// CR 702.149a: Training -- "Whenever this creature and at least one
// other creature with power greater than this creature's power attack,
// put a +1/+1 counter on this creature."
// The condition is: among ALL attackers declared in this batch, at
// least one other creature has strictly greater power than this
// creature.
// CR 508.2a: condition checked at declaration time only.
// Ruling 2021-11-19: "triggers only when both that creature and a
// creature with greater power are declared as attackers."
{
    // Get the power of the current attacker (layer-aware).
    let attacker_power = crate::rules::layers::calculate_characteristics(
        state, *attacker_id
    ).and_then(|c| c.power).unwrap_or(0);

    // Check if any OTHER attacker in this batch has strictly greater power.
    let has_greater_power_ally = attackers.iter().any(|(other_id, _)| {
        *other_id != *attacker_id && {
            let other_power = crate::rules::layers::calculate_characteristics(
                state, *other_id
            ).and_then(|c| c.power).unwrap_or(0);
            other_power > attacker_power
        }
    });

    if has_greater_power_ally {
        let pre_len_training = triggers.len();
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::SelfAttacksWithGreaterPowerAlly,
            Some(*attacker_id),
            None,
        );
        // Tag training triggers with defending player for consistency
        // with other attack triggers.
        for t in &mut triggers[pre_len_training..] {
            t.defending_player_id = defending_player;
        }
    }
}
```

**Key design points**:
- `attacker_power` uses `calculate_characteristics` for layer-aware power (accounts for
  continuous effects, counters, equipment, etc.).
- The comparison is `other_power > attacker_power` (strictly greater, per CR 702.149a).
- `defending_player` variable is already in scope from the `SelfAttacks` block above
  (line 1343-1348).
- The `{...}` block scoping prevents variable name conflicts with Dethrone variables.
- No new `PendingTrigger` fields needed -- Training uses the standard trigger path.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/training.rs`
**Pattern**: Follow `/home/airbaggie/scutemob/crates/engine/tests/dethrone.rs`

**Module header**:
```rust
//! Training keyword ability tests (CR 702.149).
//!
//! Training is a triggered ability: "Whenever this creature and at least one
//! other creature with power greater than this creature's power attack, put a
//! +1/+1 counter on this creature."
//!
//! Key rules verified:
//! - Trigger fires when attacking alongside a creature with greater power (CR 702.149a).
//! - The +1/+1 counter goes on the training creature itself (CR 702.149a).
//! - Does NOT trigger when no co-attacker has greater power (CR 702.149a).
//! - Does NOT trigger when attacking alone (CR 702.149a).
//! - Power comparison is strictly greater, not equal (CR 702.149a).
//! - Multiple instances each trigger separately (CR 702.149b).
//! - Multiple training creatures can each trigger from the same co-attacker.
//! - Multiplayer: co-attacker power check across all declared attackers.
```

**Imports**:
```rust
use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command,
    CounterType, GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
};
```

**Helpers** (copy from dethrone.rs):
```rust
fn find_object(state: &GameState, name: &str) -> ObjectId { ... }
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) { ... }
```

**Tests to write** (7 tests):

1. **`test_702_149a_training_basic_attacks_with_greater_power`** -- CR 702.149a
   - 2 players: P1, P2.
   - P1 has 1/2 creature with Training on battlefield.
   - P1 has 3/3 vanilla creature on battlefield.
   - P1 declares both as attackers targeting P2.
   - Assert: `AbilityTriggered` event from training source.
   - Assert: `state.stack_objects.len() == 1` (just the training trigger).
   - Pass all to resolve trigger.
   - Assert: training creature has 1 `PlusOnePlusOne` counter.
   - Assert: `calculate_characteristics` shows power=2, toughness=3.

2. **`test_702_149a_training_does_not_trigger_alone`** -- CR 702.149a (negative: attacking alone)
   - 2 players: P1, P2.
   - P1 has 1/2 creature with Training on battlefield. No other creatures.
   - P1 declares training creature as sole attacker.
   - Assert: NO `AbilityTriggered` from training source.
   - Assert: `state.stack_objects.is_empty()`.
   - Assert: creature has 0 counters.

3. **`test_702_149a_training_does_not_trigger_equal_power`** -- CR 702.149a (negative: equal power)
   - 2 players: P1, P2.
   - P1 has 2/2 creature with Training and a 2/2 vanilla creature.
   - P1 declares both as attackers.
   - Assert: NO `AbilityTriggered` from training source.
   - Assert: `state.stack_objects.is_empty()`.
   - Assert: training creature has 0 counters.
   - This confirms "greater than" is strict (not greater-or-equal).

4. **`test_702_149a_training_does_not_trigger_lower_power`** -- CR 702.149a (negative: co-attacker has lower power)
   - 2 players: P1, P2.
   - P1 has 3/3 creature with Training and a 1/1 vanilla creature.
   - P1 declares both as attackers.
   - Assert: NO trigger from training source (1/1 has lower power than 3/3).
   - Assert: training creature has 0 counters.

5. **`test_702_149b_training_multiple_instances`** -- CR 702.149b
   - 2 players: P1, P2.
   - P1 has 1/2 creature with TWO Training keywords (via `.with_keyword` twice).
   - P1 has 3/3 vanilla creature.
   - P1 declares both as attackers.
   - Assert: `state.stack_objects.len() == 2` (two separate triggers).
   - Resolve both (pass_all twice).
   - Assert: training creature has 2 `PlusOnePlusOne` counters.
   - Assert: P/T = 3/4 (1+2 / 2+2).

6. **`test_702_149a_training_two_training_creatures_both_trigger`** -- CR 702.149a
   - 2 players: P1, P2.
   - P1 has 1/1 Training creature ("Trainee A"), 2/2 Training creature ("Trainee B"),
     and a 4/4 vanilla creature.
   - P1 declares all three as attackers.
   - Assert: both training creatures trigger (4/4 > 1 and 4/4 > 2).
   - Assert: `state.stack_objects.len() == 2` (one trigger per training creature).
   - Resolve both triggers.
   - Assert: Trainee A has 1 counter, P/T = 2/2.
   - Assert: Trainee B has 1 counter, P/T = 3/3.

7. **`test_702_149a_training_multiplayer_four_player`** -- CR 702.149a + multiplayer
   - 4 players: P1, P2, P3, P4.
   - P1 has 1/2 Training creature and a 3/3 vanilla creature.
   - P1 declares training creature attacking P2, vanilla creature attacking P3.
   - Assert: training trigger fires (3/3 > 1, co-attacker exists in same batch).
   - All 4 players pass.
   - Assert: training creature has 1 counter.

### Step 5: Card Definition (later phase)

**Suggested card**: Gryff Rider
- **Name**: Gryff Rider
- **Cost**: {2}{W}
- **Type**: Creature -- Human Knight
- **P/T**: 2/1
- **Oracle**: "Flying\nTraining (Whenever this creature attacks with another creature
  with greater power, put a +1/+1 counter on this creature.)"
- **Keywords**: `[KeywordAbility::Flying, KeywordAbility::Training]`
- **Color identity**: ["W"]

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/gryff_rider.rs`
**Action**: Use `card-definition-author` agent.

**Why this card**: Simple creature with Training + Flying (both validated keywords).
No complex triggered/activated abilities beyond keyword auto-generation. Good for
validating Training end-to-end in a game script.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Training trigger in 4-player Commander"
**Subsystem directory**: `test-data/generated-scripts/combat/`
**Sequence number**: Use next available in the combat directory.

**Script outline**:
1. 4 players: P1 (40 life), P2 (40 life), P3 (40 life), P4 (40 life).
2. P1 controls Gryff Rider (2/1, Flying, Training) and a 4/4 creature on battlefield.
3. Advance to Declare Attackers step.
4. P1 declares both creatures attacking P2.
5. Assert: stack has 1 trigger item (Training).
6. All 4 players pass priority -- trigger resolves.
7. Assert: Gryff Rider has 1 +1/+1 counter.
8. Assert: Gryff Rider effective P/T is 3/2.

**File**: Use `game-script-generator` agent.

## Interactions to Watch

- **Training + Counters**: The +1/+1 counter is a real persistent counter (not a
  temporary continuous effect). It persists across turns via `Effect::AddCounter` which
  modifies `obj.counters` directly. On subsequent attacks, the Training creature's
  increased power from previous counters means the co-attacker needs even greater power
  to trigger Training again.

- **Training + Humility**: Pre-existing concern for all triggered keywords (Dethrone,
  Battle Cry, Exalted, Afterlife, etc.). `collect_triggers_for_event` reads
  `obj.characteristics.triggered_abilities` which are the raw printed characteristics,
  not the layer-calculated result. This means Humility may not suppress triggered keyword
  abilities. Not blocking -- this is a systemic issue to address separately.

- **Training + Power-modifying effects**: The power comparison uses
  `calculate_characteristics` which accounts for all layer effects (equipment, auras,
  counters, Humility, etc.). This means continuous effects that modify power are correctly
  reflected in the comparison at trigger-collection time.

- **Training + Evolve**: A creature with both Training and Evolve benefits from different
  trigger conditions. Training cares about co-attackers' power; Evolve cares about
  entering creatures' P/T. No conflict.

- **Training vs Dethrone pattern**: Both are attack triggers that place +1/+1 counters.
  Dethrone checks the defending player's life; Training checks co-attackers' power. Both
  use the same structural pattern (dedicated TriggerEvent, condition checked at
  trigger-collection time, standard TriggeredAbility stack path, AddCounter effect).

- **Training + "When this creature trains" (CR 702.149c)**: Some cards like Savior of
  Ollenbock have a linked ability "When this creature trains, [effect]." Implementing
  this is a separate concern that requires a new trigger event
  (e.g., `SelfTrains`) fired after Training resolution places counters. This is NOT
  required for the base Training implementation and can be added when authoring such cards.

## Summary of Files Modified

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Training` variant |
| `crates/engine/src/state/game_object.rs` | Add `TriggerEvent::SelfAttacksWithGreaterPowerAlly` variant |
| `crates/engine/src/state/hash.rs` | Add hash arms for both new variants (discriminants 82 and 18) |
| `crates/engine/src/state/builder.rs` | Auto-generate `TriggeredAbilityDef` for Training keyword |
| `crates/engine/src/rules/abilities.rs` | Training trigger collection in `AttackersDeclared` handler |
| `tools/replay-viewer/src/view_model.rs` | Add `format_keyword` arm for Training |
| `crates/engine/tests/training.rs` | New test file with 7 tests |
| `crates/engine/src/cards/defs/gryff_rider.rs` | Gryff Rider card definition (Step 5) |
| `test-data/generated-scripts/combat/` | Training game script (Step 6) |

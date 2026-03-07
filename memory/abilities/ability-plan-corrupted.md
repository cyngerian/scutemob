# Ability Plan: Corrupted

**Generated**: 2026-03-07
**CR**: Ability word (CR 207.2c) -- no dedicated CR section; no special rules meaning
**Priority**: P4
**Similar abilities studied**: Alliance (ability word, `replay_harness.rs:2161-2198`), Enrage (ability word, `card_definition.rs:1144`)

## CR Rule Text

CR 207.2c: "An ability word appears in italics at the beginning of some abilities.
Ability words are similar to keywords in that they tie together cards that have similar
functionality, but they have no special rules meaning and no individual entries in the
Comprehensive Rules."

**Note**: "Corrupted" is NOT listed in CR 207.2c's enumerated ability word list (likely
added after the CR snapshot). Regardless, its mechanical pattern is well-established:
the condition is "if an opponent has three or more poison counters." This appears on
Phyrexia: All Will Be One cards (2023).

## Key Edge Cases

- **Threshold is 3 poison counters** -- not "poisoned" (which would mean >= 1). The
  number 3 is hardcoded in the ability word's templating but should be parameterized
  in the engine as `Condition::OpponentHasPoisonCounters(3)` for generality.
- **"An opponent" means ANY one opponent** -- in multiplayer, if ANY single opponent
  has 3+ poison counters, the condition is met. This is important for Commander.
- **Corrupted appears on both triggered and static abilities**:
  - Triggered: "When this enters, if an opponent has 3+ poison, [effect]" (intervening-if)
  - Static: "As long as an opponent has 3+ poison, this has [keywords]" (conditional continuous effect)
- **Poison counter tracking already exists**: `PlayerState.poison_counters: u32` (at
  `state/player.rs:74`), with full infrastructure for Toxic, Infect, Poisonous, Proliferate,
  and the 10-poison SBA (CR 704.5c).
- **Multiplayer: opponent who had 3+ poison might be eliminated** -- eliminated players
  have `has_lost == true`; the condition should check only living opponents.
- **The condition is checked continuously for static abilities** -- if an opponent gains
  their 3rd poison counter mid-combat, the static effect turns on immediately (layer system
  recalculates characteristics on every query).

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- NOT needed (ability word, no KeywordAbility variant)
- [ ] Step 2: Rule enforcement -- new `Condition::OpponentHasPoisonCounters(u32)` variant
- [ ] Step 3: Trigger wiring -- N/A (uses existing `intervening_if` infrastructure)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant -- SKIP

Corrupted is an ability word (CR 207.2c), not a keyword ability. It has no special
rules meaning. No `KeywordAbility` variant is needed. The mechanical condition
("opponent has 3+ poison counters") is modeled as a `Condition` variant.

No changes to `crates/engine/src/state/types.rs`.
No changes to `tools/replay-viewer/src/view_model.rs`.
No changes to `tools/tui/src/play/panels/stack_view.rs`.

### Step 2: Condition Variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `OpponentHasPoisonCounters(u32)` to the `Condition` enum (after `WasCleaved`).

```
/// CR 207.2c (Corrupted ability word): "if an opponent has N or more poison counters."
/// Checked at the current game state. In multiplayer, true if ANY living opponent of
/// the controller has >= N poison counters.
OpponentHasPoisonCounters(u32),
```

**Pattern**: Follow `Condition::ControllerLifeAtLeast(u32)` -- same shape (single u32 parameter).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add discriminant 12 arm to `impl HashInto for Condition` (at line ~3235):
```
Condition::OpponentHasPoisonCounters(n) => {
    12u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add arm to `check_condition()` (after the `WasCleaved` arm at line ~3120):
```
// CR 207.2c (Corrupted): "if an opponent has N or more poison counters"
// In multiplayer, true if ANY living opponent has >= N poison counters.
Condition::OpponentHasPoisonCounters(n) => state
    .players
    .iter()
    .any(|(pid, ps)| {
        *pid != ctx.controller && !ps.has_lost && ps.poison_counters >= *n
    }),
```

### Step 3: Trigger Wiring -- N/A

Corrupted uses the existing `intervening_if: Option<Condition>` field on
`AbilityDefinition::Triggered`. No new trigger infrastructure is needed.

For triggered Corrupted abilities (e.g., Vivisection Evangelist), the card definition
uses:
```
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenEntersBattlefield,
    intervening_if: Some(Condition::OpponentHasPoisonCounters(3)),
    effect: ...,
}
```

For static Corrupted abilities (e.g., Bonepicker Skirge), a card definition needs to
model "as long as an opponent has 3+ poison, this has deathtouch and lifelink." This
requires a conditional static ability pattern. Two options:

**Option A (recommended for now)**: Use `AbilityDefinition::Triggered` with
`TriggerCondition::WhenEntersBattlefield` is wrong for statics. Instead, the static
effect should be hardcoded into the layer system check for the specific card, or...

**Option B (recommended)**: The `Condition` variant is sufficient for triggered
Corrupted abilities. Static Corrupted abilities (like Bonepicker Skirge) require
a `condition: Option<Condition>` field on `ContinuousEffectDef` or `ContinuousEffect`.
This is a larger infrastructure change. **Defer static Corrupted to when conditional
continuous effects are needed broadly.** For this batch, implement the `Condition`
variant and validate it with a triggered Corrupted card.

**Decision**: Implement `Condition::OpponentHasPoisonCounters(u32)` and validate with
a triggered-ability Corrupted card. Static Corrupted cards are deferred until
conditional continuous effects are added to the engine.

### Step 4: Unit Tests

**File**: `crates/engine/tests/corrupted.rs`
**Tests to write**:

- `test_corrupted_triggered_etb_fires_when_opponent_has_3_poison`
  CR 207.2c -- When a creature with a Corrupted ETB trigger enters and an opponent has
  3+ poison counters, the triggered effect fires.
  Setup: P2 has 3 poison counters. P1 plays a creature with Corrupted ETB trigger.
  Assert: the effect resolves.

- `test_corrupted_triggered_etb_does_not_fire_below_threshold`
  CR 207.2c -- When a creature with a Corrupted ETB trigger enters and no opponent has
  3+ poison counters, the triggered effect does NOT fire.
  Setup: P2 has 2 poison counters. P1 plays a creature with Corrupted ETB trigger.
  Assert: no trigger on the stack.

- `test_corrupted_condition_checks_any_opponent_multiplayer`
  CR 207.2c -- In a 4-player game, if ANY opponent (not just one specific one) has 3+
  poison, the condition is met.
  Setup: P2=0, P3=3, P4=1 poison. P1's Corrupted ETB fires.
  Assert: trigger fires.

- `test_corrupted_condition_ignores_controller_poison`
  The controller's own poison counters do not count. Even if P1 has 10 poison counters
  but no opponent has 3+, the condition is false.
  Setup: P1=5, P2=2, P3=1, P4=0 poison. P1's Corrupted ETB.
  Assert: trigger does NOT fire.

- `test_corrupted_condition_ignores_eliminated_opponents`
  An eliminated opponent with 3+ poison does not satisfy the condition.
  Setup: P2 has 10 poison (eliminated by SBA), P3=0, P4=0. P1's Corrupted ETB.
  Assert: trigger does NOT fire (P2 is eliminated).

- `test_corrupted_intervening_if_checked_at_resolution`
  CR 603.4 -- Intervening-if conditions are checked both when the trigger fires AND
  when the trigger resolves. If the opponent's poison drops below 3 between trigger
  and resolution (unlikely but testable via proliferate-negative or similar), the
  effect fizzles.
  Note: Since there's no standard way to remove poison counters, this test may use
  direct state manipulation to set poison_counters between trigger and resolution.

**Pattern**: Follow tests in `crates/engine/tests/alliance.rs` and
`crates/engine/tests/enrage.rs` for ability word test structure.

### Step 5: Card Definition

**Suggested card**: Vivisection Evangelist

Oracle text: "Vigilance. Corrupted -- When this creature enters, if an opponent has
three or more poison counters, destroy target creature or planeswalker an opponent
controls."

**Why this card**:
- Uses a triggered ability with `intervening_if`, which is the pattern we're validating
- Simple ETB trigger with a targeted destroy effect
- All components exist in the DSL: Vigilance keyword, WhenEntersBattlefield trigger,
  intervening_if Condition, DestroyTarget effect, TargetFilter for opponent's creatures/planeswalkers
- Does NOT require static conditional continuous effects (which we're deferring)

**Card definition structure**:
```
abilities: vec![
    AbilityDefinition::Keyword(KeywordAbility::Vigilance),
    AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WhenEntersBattlefield,
        intervening_if: Some(Condition::OpponentHasPoisonCounters(3)),
        effect: Effect::DestroyTarget { target_index: 0 },
    },
],
targets: vec![EffectTarget { ... creature or planeswalker an opponent controls ... }],
```

**Card lookup**: use `card-definition-author` agent.

### Step 6: Game Script

**Suggested scenario**: Vivisection Evangelist enters with and without Corrupted active.

**Subsystem directory**: `test-data/generated-scripts/etb-triggers/`

**Script outline**:
1. Initial state: P1 has Vivisection Evangelist in hand, P2 has a creature on battlefield,
   P2 has 3 poison counters.
2. P1 casts Vivisection Evangelist.
3. ETB trigger fires (intervening-if passes).
4. P1 targets P2's creature.
5. Trigger resolves, creature is destroyed.
6. Assert: P2's creature is in graveyard.

### Step 7: Coverage Doc Update

Update `docs/mtg-engine-ability-coverage.md`:
- Change Corrupted row status from `none` to `validated`
- Add file references, card name, script reference, and test summary

## Interactions to Watch

- **Poison counter sources**: Toxic (combat), Infect (combat), Poisonous (triggered),
  Proliferate (any counter), and direct "gets a poison counter" effects. All use the
  same `PlayerState.poison_counters` field, so no special interaction logic needed.
- **Corrupted + Proliferate**: Proliferate can push an opponent from 2 to 3 poison,
  enabling Corrupted. This is a natural game interaction, not an engine edge case.
- **Multiplayer implications**: The "any opponent" check is critical. A card with
  Corrupted should be enabled even if only one of three opponents has 3+ poison.
- **Static Corrupted abilities deferred**: Cards like Bonepicker Skirge and Skrelv's
  Hive need conditional continuous effects (`condition` field on `ContinuousEffect`
  or `ContinuousEffectDef`). This is a general infrastructure need beyond Corrupted.
  Track as a future enhancement.

## Summary of Changes

| File | Change |
|------|--------|
| `crates/engine/src/cards/card_definition.rs` | Add `Condition::OpponentHasPoisonCounters(u32)` variant |
| `crates/engine/src/state/hash.rs` | Add discriminant 12 arm for new Condition variant |
| `crates/engine/src/effects/mod.rs` | Add `check_condition` arm for the new variant |
| `crates/engine/tests/corrupted.rs` | 5-6 unit tests |
| `crates/engine/src/cards/defs/vivisection_evangelist.rs` | Card definition |
| `test-data/generated-scripts/etb-triggers/` | Game script |
| `docs/mtg-engine-ability-coverage.md` | Status update |

**Estimated effort**: Low. Three lines of new enum variant, one condition-check arm,
hash arm, and tests. No new infrastructure beyond a single `Condition` variant.

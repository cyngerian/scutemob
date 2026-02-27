# Ability Plan: Undying

**Generated**: 2026-02-26
**CR**: 702.93
**Priority**: P2
**Similar abilities studied**: Persist (CR 702.79) — fully validated, identical pattern with counter type swapped

## CR Rule Text

> **702.93. Undying**
>
> 702.93a Undying is a triggered ability. "Undying" means "When this permanent is put into
> a graveyard from the battlefield, if it had no +1/+1 counters on it, return it to the
> battlefield under its owner's control with a +1/+1 counter on it."

Compare with Persist (CR 702.79a):

> 702.79a Persist is a triggered ability. "Persist" means "When this permanent is put into
> a graveyard from the battlefield, if it had no -1/-1 counters on it, return it to the
> battlefield under its owner's control with a -1/-1 counter on it."

Undying is the exact mirror of Persist:
- Persist checks for **-1/-1** counters, returns with a **-1/-1** counter.
- Undying checks for **+1/+1** counters, returns with a **+1/+1** counter.

## Key Edge Cases

1. **Intervening-if at trigger time (CR 603.4)**: Undying only triggers if the creature
   had no +1/+1 counters on it when it died. Checked against `pre_death_counters` carried
   by the `CreatureDied` event (last known information, CR 603.10a).

2. **Intervening-if at resolution time (CR 603.4)**: The condition is checked again at
   resolution. Since the creature is in the graveyard with no counters, the condition passes
   unconditionally (same pattern as Persist). If the creature is no longer in the graveyard
   (e.g., exiled by another effect), `MoveZone` no-ops.

3. **Counter annihilation (CR 704.5q)**: If a creature with undying has a +1/+1 counter
   (from a previous undying return) and a -1/-1 counter is placed on it, SBA annihilates
   both. Now it has no +1/+1 counter and can undying again on next death.

4. **Token with undying (CR 704.5d)**: A token creature with undying triggers when it dies,
   but the token ceases to exist in the graveyard before the trigger resolves. MoveZone
   finds no source and the trigger has no effect.

5. **Multiplayer APNAP ordering (CR 603.3)**: Multiple undying creatures dying simultaneously
   have their triggers placed on the stack in APNAP order.

6. **Creature enters tapped (Geralf's Messenger)**: Some undying cards specify "enters
   tapped." This is handled by the card's ETB replacement, not by undying itself. Undying
   just returns to the battlefield; additional ETB conditions are separate.

7. **Mikaeus, the Unhallowed ruling**: The +1/+1 static bonus Mikaeus gives is NOT a
   counter. It does not prevent undying from triggering. (This is a Layer 7c effect, not
   a counter.)

8. **Persist + Undying on same creature**: If a creature has both persist and undying, and
   it dies with no +1/+1 AND no -1/-1 counters, both trigger. The controller chooses the
   order (APNAP). After the first resolves (e.g., undying returns it with +1/+1), the
   second trigger tries to MoveZone from graveyard but the creature is on the battlefield
   now, so the second trigger no-ops. (This is a natural consequence of the existing
   MoveZone no-op behavior.)

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant (`KeywordAbility::Undying`)
- [ ] Step 2: Rule enforcement (builder.rs triggered ability translation)
- [ ] Step 3: Trigger wiring (already done -- reuses existing SelfDies/InterveningIf infra)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Undying` variant immediately after `KeywordAbility::Persist` (line 277).
**Pattern**: Follow `KeywordAbility::Persist` at line 270-277.
**Doc comment**:
```rust
/// CR 702.93: Undying -- "When this permanent is put into a graveyard from
/// the battlefield, if it had no +1/+1 counters on it, return it to the
/// battlefield under its owner's control with a +1/+1 counter on it."
///
/// Translated to a TriggeredAbilityDef at object-construction time in
/// `state/builder.rs`. The trigger fires on SelfDies events; the
/// intervening-if checks pre-death counters via the CreatureDied event.
Undying,
```

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` `HashInto` impl for `KeywordAbility`.
Add after the `Persist` arm (line 357):
```rust
// Undying (discriminant 37) -- CR 702.93
KeywordAbility::Undying => 37u8.hash_into(hasher),
```

**Replay viewer**: Add to `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs` `keyword_display` function.
Add after the `Persist` arm (line 600):
```rust
KeywordAbility::Undying => "Undying".to_string(),
```

**Match arms**: Grep for exhaustive `match` on `KeywordAbility` -- the hash and view_model files are the only two that need new arms (all other keyword processing is generic via `keywords.contains()`).

### Step 2: Builder Translation (Rule Enforcement)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Add an `if matches!(kw, KeywordAbility::Undying)` block immediately after the Persist block (after line ~466).
**Pattern**: Identical to the Persist block at lines 438-466, with three differences:

1. `CounterType::MinusOneMinusOne` -> `CounterType::PlusOnePlusOne` in `InterveningIf::SourceHadNoCounterOfType`
2. `CounterType::MinusOneMinusOne` -> `CounterType::PlusOnePlusOne` in `Effect::AddCounter`
3. Description text updated to reference CR 702.93a and "+1/+1"

**Code**:
```rust
// CR 702.93a: Undying -- "When this permanent is put into a graveyard from
// the battlefield, if it had no +1/+1 counters on it, return it to the
// battlefield under its owner's control with a +1/+1 counter on it."
// Each keyword instance generates one TriggeredAbilityDef.
// The intervening-if is checked at trigger time against pre_death_counters
// carried by the CreatureDied event (last known information, CR 603.10a).
if matches!(kw, KeywordAbility::Undying) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDies,
        intervening_if: Some(InterveningIf::SourceHadNoCounterOfType(
            CounterType::PlusOnePlusOne,
        )),
        description: "Undying (CR 702.93a): When this permanent dies, \
                      if it had no +1/+1 counters on it, return it to the \
                      battlefield under its owner's control with a +1/+1 \
                      counter on it."
            .to_string(),
        effect: Some(Effect::Sequence(vec![
            Effect::MoveZone {
                target: EffectTarget::Source,
                to: crate::cards::card_definition::ZoneTarget::Battlefield {
                    tapped: false,
                },
            },
            Effect::AddCounter {
                target: EffectTarget::Source,
                counter: CounterType::PlusOnePlusOne,
                count: 1,
            },
        ])),
    });
}
```

**CR**: 702.93a -- the core undying mechanic.

### Step 3: Trigger Wiring

**No new wiring needed.** Undying reuses all existing infrastructure:

- `TriggerEvent::SelfDies` -- already dispatched from `CreatureDied` events in `rules/abilities.rs`
- `InterveningIf::SourceHadNoCounterOfType(CounterType::PlusOnePlusOne)` -- already implemented generically in `rules/abilities.rs:check_intervening_if` (line 1191)
- `pre_death_counters` -- already captured at all 8 `CreatureDied` emission sites (see `memory/gotchas-infra.md`)
- `Effect::MoveZone` + `Effect::AddCounter` -- already implemented in `effects/mod.rs` with the `ctx.source` update after MoveZone (line 762-767)
- `Effect::Sequence` -- already handles the MoveZone->AddCounter pattern

This is what makes Undying near-zero cost: the Persist infrastructure was designed to be counter-type-generic.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/undying.rs`
**Pattern**: Mirror the 6 tests in `/home/airbaggie/scutemob/crates/engine/tests/persist.rs`, swapping:
- `MinusOneMinusOne` -> `PlusOnePlusOne`
- `Persist` -> `Undying`
- CR 702.79 -> CR 702.93
- Counter annihilation test: creature has +1/+1 from undying, gains -1/-1 counter, SBA annihilates, creature can undying again

**Tests to write**:

1. `test_undying_basic_returns_with_plus_counter` -- CR 702.93a: Creature with Undying and no +1/+1 counters dies; undying trigger fires; creature returns with one +1/+1 counter.

2. `test_undying_does_not_trigger_with_plus_counter` -- CR 702.93a (intervening-if): Creature with Undying that already has a +1/+1 counter dies; undying does NOT trigger.

3. `test_undying_second_death_no_trigger` -- CR 702.93a: After undying creature returns with +1/+1 counter, if it dies again, undying does NOT trigger (has +1/+1 counter from first return).

4. `test_undying_token_trigger_but_no_return` -- CR 702.93a + CR 704.5d: Token creature with Undying triggers on death, but token ceases to exist in graveyard before resolution. MoveZone no-ops.

5. `test_undying_multiplayer_apnap_ordering` -- CR 603.3: Multiple undying creatures from different players die simultaneously; triggers ordered by APNAP. Both return to the battlefield.

6. `test_undying_minus_one_cancellation_enables_second_undying` -- CR 704.5q + CR 702.93a: Undying creature returns with +1/+1 counter. A -1/-1 counter is added. SBA annihilates both. Creature can undying again on next death.

**Imports**: Same as persist.rs:
```rust
use mtg_engine::{
    check_and_apply_sbas, process_command, CardRegistry, Command, CounterType, GameEvent,
    GameStateBuilder, KeywordAbility, ObjectSpec, PlayerId, StackObjectKind, Step, ZoneId,
};
```

**Helpers**: Same `find_by_name`, `find_by_name_in_zone`, `count_on_battlefield`, `pass_all` helpers (copy from persist.rs).

### Step 5: Card Definition (later phase)

**Suggested card**: Young Wolf ({G}, 1/1, Undying) -- simplest possible undying card.
**Alternative card**: Strangleroot Geist ({G}{G}, 2/1, Haste + Undying) -- tests keyword interaction.
**Card lookup**: Use `card-definition-author` agent.

### Step 6: Game Script (later phase)

**Suggested scenario**: Young Wolf dies in combat (blocked by a 2/2), undying triggers and returns it with a +1/+1 counter as a 2/2. Wolf attacks again, gets blocked and killed, undying does NOT trigger (has +1/+1 counter from first return).
**Subsystem directory**: `test-data/generated-scripts/combat/` (combat-death trigger)

## Interactions to Watch

1. **Counter annihilation (CR 704.5q)**: +1/+1 and -1/-1 counters annihilate as SBA. This enables infinite undying loops when combined with effects that add -1/-1 counters (e.g., Mikaeus + sacrifice outlet + creature that enters with -1/-1 counters). The loop_detection system (CR 726) should catch mandatory infinite loops.

2. **Persist + Undying on same creature**: Both check different counter types. If a creature dies with no counters of either type, both trigger. The first to resolve returns the creature; the second no-ops (MoveZone finds no source in graveyard). This is correct behavior with no special handling needed.

3. **Replacement effects on death (CR 614)**: If a replacement effect exiles the creature instead of putting it in the graveyard (e.g., Rest in Peace, Kalitas), undying does NOT trigger (the creature was not put into a graveyard). This is already handled because `TriggerEvent::SelfDies` only fires on `CreatureDied` events, which only emit when the creature actually goes to the graveyard.

4. **Commander zone-change (CR 903.9a)**: If a commander with undying dies and the owner chooses to move it to the command zone, the undying trigger still fires (it was put into the graveyard momentarily), but MoveZone will find the source in the command zone, not the graveyard, so it no-ops. This matches Persist behavior and is correct.

5. **Indestructible**: A creature with undying and indestructible can still die from -X/-X effects (toughness <= 0 after SBA) or sacrifice. Indestructible only prevents destroy effects and lethal damage. The undying trigger fires normally in these cases.

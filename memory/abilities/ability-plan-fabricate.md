# Ability Plan: Fabricate

**Generated**: 2026-03-07
**CR**: 702.123
**Priority**: P4
**Similar abilities studied**: Tribute (CR 702.104) in `resolution.rs:1033-1070`, `replacement.rs:906-970`, `tests/tribute.rs`

## CR Rule Text

702.123. Fabricate

702.123a Fabricate is a triggered ability. "Fabricate N" means "When this permanent enters, you may put N +1/+1 counters on it. If you don't, create N 1/1 colorless Servo artifact creature tokens."

702.123b If a permanent has multiple instances of fabricate, each triggers separately.

## Key Edge Cases

- **Fabricate is a triggered ability, NOT a replacement/static ability (CR 702.123a).** Unlike
  Tribute (which is "as this creature enters"), Fabricate says "when this permanent enters."
  The ability goes on the stack. Players can respond before it resolves. However, for
  deterministic bot play, we fire it inline in `fire_when_enters_triggered_effects` (same as
  Tribute's TributeNotPaid trigger).
- **Choice happens at resolution (ruling 2016-09-20).** "You choose whether to put +1/+1
  counters on the creature or create Servo tokens as the fabricate ability is resolving.
  No player may take actions between the time you choose and the time that counters are
  added or tokens are created."
- **Creature no longer on battlefield at resolution (ruling 2016-09-20).** "If you can't
  put +1/+1 counters on the creature for any reason as fabricate resolves (for instance,
  if it's no longer on the battlefield), you just create Servo tokens." For inline
  resolution (bot play), the creature is always on the battlefield at ETB trigger time,
  so this edge case is deferred until interactive play. However, we should still handle
  it defensively: if the permanent is gone, always create tokens.
- **Multiple instances (CR 702.123b).** Each triggers separately. Two Fabricate 1 instances
  means two separate choices (not one Fabricate 2). For bot play, each resolves
  independently.
- **Bot choice: counters (deterministic).** For bot play, always choose counters (simpler,
  consistent with "more power" heuristic). If the permanent is no longer on the battlefield,
  fall back to tokens per the ruling.
- **Servo token spec.** 1/1 colorless Servo artifact creature token. No abilities, no
  keywords, no mana abilities. Subtype "Servo" exists in `types.rs:1266`.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Fabricate(u32)` variant after `Tribute(u32)` (line ~1211).
**Pattern**: Follow `KeywordAbility::Tribute(u32)` at line 1211.
**Discriminant**: 132 (next after Tribute=131).
**Doc comment**:
```
/// CR 702.123: Fabricate N -- "When this permanent enters, you may put N
/// +1/+1 counters on it. If you don't, create N 1/1 colorless Servo
/// artifact creature tokens."
///
/// Triggered ability (CR 702.123a). Multiple instances trigger separately
/// (CR 702.123b).
///
/// Discriminant 132.
Fabricate(u32),
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Fabricate(n)` after the Tribute arm (line ~649).
**Pattern**: Follow `KeywordAbility::Tribute(n)` hash at line 646-649.
```rust
// Fabricate (discriminant 132) -- CR 702.123
KeywordAbility::Fabricate(n) => {
    132u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add display arm for `KeywordAbility::Fabricate(n)` in the keyword display match (line ~851).
**Pattern**: Follow `KeywordAbility::Tribute(n)` arm at line 851.
```rust
KeywordAbility::Fabricate(n) => format!("Fabricate {n}"),
```

**No new StackObjectKind needed.** Fabricate fires inline as a WhenEntersBattlefield
trigger, not as a separate stack object kind.

**No new AbilityDefinition variant needed.** Fabricate uses the existing
`AbilityDefinition::Keyword(KeywordAbility::Fabricate(n))` pattern. The trigger logic
is handled inline in `fire_when_enters_triggered_effects`, not through a dedicated
AbilDef discriminant.

### Step 2: Rule Enforcement (inline ETB trigger)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/replacement.rs`
**Action**: Add a `KeywordAbility::Fabricate(n)` handler inside `fire_when_enters_triggered_effects`
(lines 906-970). This is the same function that handles Tribute's TributeNotPaid trigger.
**Pattern**: The Fabricate handler should be added as a new match arm in the `for ability in &def.abilities`
loop. However, since Fabricate triggers based on the keyword presence (not a separate
TriggerCondition), add a **new block after the existing match** that scans the definition's
abilities for `KeywordAbility::Fabricate(n)` instances.
**CR**: 702.123a -- "When this permanent enters, you may put N +1/+1 counters on it.
If you don't, create N 1/1 colorless Servo artifact creature tokens."

Implementation approach (add after the existing `for ability` loop, before the return):

```rust
// CR 702.123a: Fabricate N -- "When this permanent enters, you may put N +1/+1
// counters on it. If you don't, create N 1/1 colorless Servo artifact creature tokens."
// CR 702.123b: Multiple instances trigger separately.
//
// Bot play: always choose counters if the permanent is still on the battlefield.
// Ruling 2016-09-20: if the permanent is no longer on the battlefield, create tokens.
{
    let fabricate_instances: Vec<u32> = def
        .abilities
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::Keyword(KeywordAbility::Fabricate(n)) => Some(*n),
            _ => None,
        })
        .collect();

    for n in fabricate_instances {
        let permanent_on_bf = state
            .objects
            .get(&new_id)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);

        if permanent_on_bf {
            // Bot choice: put N +1/+1 counters on it.
            if n > 0 {
                if let Some(obj) = state.objects.get_mut(&new_id) {
                    let current = obj
                        .counters
                        .get(&CounterType::PlusOnePlusOne)
                        .copied()
                        .unwrap_or(0);
                    obj.counters = obj.counters.update(CounterType::PlusOnePlusOne, current + n);
                }
                evts.push(GameEvent::CountersAdded {
                    object_id: new_id,
                    counter_type: CounterType::PlusOnePlusOne,
                    count: n,
                });
            }
        } else {
            // Ruling 2016-09-20: if permanent left the battlefield, create Servo tokens.
            if n > 0 {
                let servo_spec = TokenSpec {
                    name: "Servo".to_string(),
                    power: 1,
                    toughness: 1,
                    colors: OrdSet::new(),
                    card_types: ordset![CardType::Artifact, CardType::Creature],
                    subtypes: ordset![SubType::new("Servo")],
                    keywords: OrdSet::new(),
                    count: n,
                    tapped: false,
                    mana_color: None,
                    mana_abilities: vec![],
                    activated_abilities: vec![],
                };
                let mut ctx = EffectContext::new(controller, new_id, vec![]);
                evts.extend(execute_effect(
                    state,
                    &Effect::CreateToken { spec: servo_spec },
                    &mut ctx,
                ));
            }
        }
    }
}
```

**Required imports** (add to the existing `use` block at the top of the function):
- `CounterType` (likely already available)
- `TokenSpec` from `crate::cards::card_definition`
- `Effect` from `crate::cards::card_definition`
- `ZoneId` from `crate::state`
- `CardType`, `SubType` from `crate::state::types`
- `ordset` macro from `im`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/lands.rs`
**Action**: Add a no-op comment for Fabricate in the lands ETB section (after the Tribute
no-op comment at line ~360-363). Lands with Fabricate are not printed in Magic.
**CR**: 702.123a (ETB hook for consistency with gotchas-infra.md two-ETB-site rule).
```rust
// CR 702.123a: Fabricate N -- lands with Fabricate are not printed in Magic.
// ETB hook exists here for consistency with resolution.rs (gotchas-infra.md).
// No-op for lands.
```

### Step 3: Trigger Wiring

**Not applicable.** Fabricate fires inline in `fire_when_enters_triggered_effects`. No new
`TriggerCondition` variant, no new `StackObjectKind`, no new `PendingTriggerKind` needed.
The choice (counters vs. tokens) is resolved immediately at trigger time for bot play.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/fabricate.rs`
**Pattern**: Follow `tests/tribute.rs` structure exactly.
**Tests to write**:

1. `test_fabricate_basic_counters` -- CR 702.123a: Fabricate N creature enters, bot
   chooses counters. Verify N +1/+1 counters on the permanent. No Servo tokens created.

2. `test_fabricate_no_servo_tokens_when_counters_chosen` -- CR 702.123a: Verify no Servo
   tokens exist on the battlefield after bot chooses counters.

3. `test_fabricate_keyword_on_card` -- CR 702.123a: Verify `KeywordAbility::Fabricate(N)`
   is present on the card definition.

4. `test_fabricate_different_n_value` -- CR 702.123a: Fabricate 1 vs Fabricate 2 produce
   the correct number of counters.

5. `test_fabricate_multiple_instances` -- CR 702.123b: A creature with two instances of
   Fabricate (e.g., Fabricate 1 and Fabricate 2) triggers each separately.
   Bot chooses counters for each: total = 1 + 2 = 3 counters.

6. `test_fabricate_permanent_left_battlefield_creates_tokens` -- Ruling 2016-09-20: If
   the permanent is no longer on the battlefield when Fabricate resolves, create N Servo
   tokens instead. (This edge case may be hard to test with inline resolution; defer
   or simulate by manually removing the object before trigger dispatch.)

7. `test_fabricate_multiplayer` -- CR 702.123a: Fabricate fires correctly in a 4-player
   game. Controller gets N counters.

**Card definitions for tests** (inline in test file, not registered card defs):
- "Fabricate 2 Test Creature": 0/1, cost {2}{B}, Fabricate 2
- "Fabricate 1 Test Creature": 2/2, cost {2}, Fabricate 1
- "Double Fabricate Test": 1/1, cost {3}, Fabricate 1 + Fabricate 2

### Step 5: Card Definition (later phase)

**Suggested card**: Weaponcraft Enthusiast
- Name: Weaponcraft Enthusiast
- Cost: {2}{B}
- Type: Creature -- Aetherborn Artificer
- P/T: 0/1
- Oracle: "Fabricate 2 (When this creature enters, put two +1/+1 counters on it or create two 1/1 colorless Servo artifact creature tokens.)"
- Abilities: `[AbilityDefinition::Keyword(KeywordAbility::Fabricate(2))]`
- CardId: `weaponcraft-enthusiast`
- File: `crates/engine/src/cards/defs/weaponcraft_enthusiast.rs`
**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: Weaponcraft Enthusiast is cast from hand, resolves, Fabricate 2
triggers. Bot chooses counters: creature becomes 2/3 (0/1 + 2 +1/+1 counters). Verify
counter count and absence of Servo tokens.
**Subsystem directory**: `test-data/generated-scripts/abilities/`

## Interactions to Watch

- **Panharmonicon / Yarok interaction**: Fabricate is a triggered ability ("When this
  permanent enters"). If Panharmonicon or Yarok is on the battlefield, Fabricate should
  trigger twice. However, per `gotchas-infra.md`, `SelfEntersBattlefield` triggers are
  NOT doubled by `doubler_applies_to_trigger` (which only matches
  `AnyPermanentEntersBattlefield`). This is a known limitation -- fix holistically when
  addressing trigger doubling. No special handling needed in this implementation.
- **Humility / Dress Down**: If Humility removes all abilities before Fabricate resolves
  (as an inline trigger), the Fabricate trigger has already been dispatched. Since we fire
  inline during ETB (before continuous effects could remove the ability), this is a
  non-issue for inline resolution. In interactive play, this would matter.
- **Doubling Season / Anointed Procession**: Doubling Season doubles +1/+1 counters placed
  on permanents AND doubles tokens. If the player chooses counters, Doubling Season would
  double them; if tokens, it doubles the token count. Currently the engine's counter-add
  and token-create paths already handle these doublers if implemented. No Fabricate-specific
  work needed.
- **Multiplayer**: Fabricate is a self-ability (controller chooses). No opponent interaction
  needed. Works identically in 2-player and N-player games.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | Fabricate(u32) | 132 |
| AbilityDefinition | (none needed) | -- |
| StackObjectKind | (none needed) | -- |
| Effect | (none needed) | -- |
| GameEvent | (none needed) | -- |
| TriggerCondition | (none needed) | -- |

# Ability Plan: Adapt

**Generated**: 2026-02-28
**CR**: 701.46
**Priority**: P3
**Similar abilities studied**: Modular (`types.rs:489`, `resolution.rs:356-395`, `abilities.rs:1431-1443`), Evolve (`types.rs:490-496`, `abilities.rs:976-1081`), Outlast (not yet implemented -- same pattern of activated ability that places +1/+1 counters at sorcery speed)

## CR Rule Text

```
701.46. Adapt

701.46a "Adapt N" means "If this permanent has no +1/+1 counters on it, put N +1/+1
counters on it."
```

That is the complete rule -- there is only one sub-rule (701.46a). Adapt is classified as a
**keyword action** (section 701), not a keyword ability (section 702). This is significant:
Adapt describes what happens when the ability resolves, not a continuous or static ability on
the permanent.

On cards, Adapt always appears as an activated ability: `{cost}: Adapt N.` The cost varies
per card, and the N value varies per card. The activation is always at instant speed (no
sorcery-speed restriction on any printed Adapt card).

## Key Edge Cases

From card rulings (Sharktocrab, Growth-Chamber Guardian, Zegana, Incubation Druid):

1. **You can always activate the ability (ruling 2019-01-25)**: "You can always activate an
   ability that will cause a creature to adapt. As that ability resolves, if the creature has
   a +1/+1 counter on it for any reason, you simply won't put any +1/+1 counters on it."
   This means the activation cost is ALWAYS paid; the counter check happens on RESOLUTION,
   not on activation. The player pays the cost, the ability goes on the stack, and on
   resolution the engine checks for +1/+1 counters.

2. **Losing counters allows re-adapting (ruling 2019-01-25)**: "If a creature somehow loses
   all of its +1/+1 counters, it can adapt again and get more +1/+1 counters." The check is
   purely "has no +1/+1 counters at resolution time."

3. **"Whenever one or more +1/+1 counters are put on this creature" triggers**: Many Adapt
   creatures (Sharktocrab, Growth-Chamber Guardian) have a companion triggered ability that
   fires when counters are placed. The CounterAdded event emitted by Effect::AddCounter
   should trigger these. However, this is a card-specific triggered ability, not part of the
   Adapt keyword itself. The adapt implementation does not need special trigger infrastructure.

4. **Adapt only checks +1/+1 counters**: The condition is specifically "+1/+1 counters on
   it." Other counter types (charge, loyalty, -1/-1, etc.) are irrelevant.

5. **Multiplayer**: No special multiplayer interactions. Adapt is a self-contained activated
   ability with no targeting of opponents.

6. **Biomancer's Familiar interaction**: "The next time target creature adapts this turn, it
   adapts as though it had no +1/+1 counters on it." This is a card-specific override of the
   condition check -- out of scope for the base Adapt implementation but worth noting for
   future card definitions.

## Design Decision: Modeling Adapt

Adapt is a **keyword action** (CR 701.46), not a keyword ability (CR 702.xx). However, cards
with Adapt always present it as an activated ability with a specific cost.

**Recommended approach**: Model Adapt as **both**:

1. A `KeywordAbility::Adapt(u32)` variant on the enum -- for presence-checking, display, and
   pattern matching. The `u32` parameter is the N value (number of +1/+1 counters). This is
   consistent with how Modular(u32), Crew(u32), Afterlife(u32), Dredge(u32), etc. are modeled.

2. An `AbilityDefinition::Activated` on the card definition -- with `Conditional` effect that
   checks `SourceHasNoCountersOfType { counter: PlusOnePlusOne }` and, if true, adds N +1/+1
   counters. The activated ability goes on the stack and resolves through the standard
   activated ability pipeline.

This approach requires one new `Condition` variant (`SourceHasNoCountersOfType`) since the
existing `SourceHasCounters { counter, min }` checks for the presence of counters, not their
absence.

**Why not a dedicated StackObjectKind?** Unlike Modular or Evolve which need special
resolution logic (last-known-information, intervening-if re-checks, targeting artifact
creatures), Adapt's resolution is simple: check condition, add counters. The standard
`ActivatedAbility` resolution with a `Conditional` effect handles this cleanly. No new
`StackObjectKind` needed.

## Current State (from ability-wip.md)

The WIP file currently tracks Bolster, not Adapt. No steps are done for Adapt.

- [ ] Step 1: Enum variant (`KeywordAbility::Adapt(u32)`)
- [ ] Step 2: New Condition variant (`SourceHasNoCountersOfType`)
- [ ] Step 3: Card definition DSL wiring (no rule enforcement needed beyond existing activated ability pipeline)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Adapt(u32)` variant after `Hideaway(u32)` (after line ~555).
**Pattern**: Follow `KeywordAbility::Modular(u32)` at line 489.

```rust
/// CR 701.46: Adapt N -- keyword action used as activated ability.
/// "Adapt N" means "If this permanent has no +1/+1 counters on it, put N
/// +1/+1 counters on it."
///
/// Marker for quick presence-checking. The activation cost and conditional
/// effect are stored in `AbilityDefinition::Activated` on the card definition.
/// The u32 parameter is the N value (number of +1/+1 counters to add).
///
/// Always used as an activated ability on the card (instant speed).
/// The condition check (no +1/+1 counters) happens at resolution time,
/// not at activation time (ruling 2019-01-25).
Adapt(u32),
```

**Hash**: Add to `crates/engine/src/state/hash.rs` in the `KeywordAbility` `HashInto` impl.
Next available discriminant is **67** (after Hideaway at 66):

```rust
// Adapt (discriminant 67) -- CR 701.46
KeywordAbility::Adapt(n) => {
    67u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**View model**: Add to `tools/replay-viewer/src/view_model.rs` in the `keyword_display` function:

```rust
KeywordAbility::Adapt(n) => format!("Adapt {n}"),
```

**Match arms**: Grep for exhaustive `KeywordAbility` match expressions and add the new arm.
Key files to check:
- `crates/engine/src/state/hash.rs` (done above)
- `tools/replay-viewer/src/view_model.rs` (done above)
- `crates/engine/src/state/builder.rs` (if keyword->trigger translation is exhaustive)
- `crates/engine/src/rules/layers.rs` (if keywords are pattern-matched)
- Any other exhaustive matches found by the compiler

### Step 2: New Condition Variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `SourceHasNoCountersOfType { counter: CounterType }` to the `Condition` enum.
**Pattern**: Follow `SourceHasCounters` at line 769. This new variant is the logical negation:
"source has zero counters of the given type."

```rust
/// "if ~ has no [counter] counters on it" -- negation of SourceHasCounters.
/// Used by Adapt (CR 701.46a), and potentially by persist/undying condition
/// checks in the future.
SourceHasNoCountersOfType { counter: CounterType },
```

**Condition evaluation** in `crates/engine/src/effects/mod.rs` -- add arm to `check_condition`:

```rust
Condition::SourceHasNoCountersOfType { counter } => state
    .objects
    .get(&ctx.source)
    .map(|obj| obj.counters.get(counter).copied().unwrap_or(0) == 0)
    .unwrap_or(true), // If source doesn't exist, treat as "no counters" (safe default)
```

**Hash**: Add to `crates/engine/src/state/hash.rs` in the `Condition` `HashInto` impl.
Next available discriminant is **8** (after WasKicked at 7):

```rust
// SourceHasNoCountersOfType (discriminant 8) -- used by Adapt (CR 701.46)
Condition::SourceHasNoCountersOfType { counter } => {
    8u8.hash_into(hasher);
    counter.hash_into(hasher);
}
```

### Step 3: No Special Rule Enforcement Needed

Adapt uses the **existing activated ability pipeline**:

1. Player sends `Command::ActivateAbility { player, source, ability_index, targets: [] }`.
2. `handle_activate_ability` validates priority, pays cost (mana), pushes
   `StackObjectKind::ActivatedAbility` onto the stack with the embedded effect.
3. On resolution, `resolve_stack_object` executes the embedded effect.
4. The `Conditional` effect checks `SourceHasNoCountersOfType { counter: PlusOnePlusOne }`.
5. If true (no +1/+1 counters), `AddCounter` places N +1/+1 counters on the source.
6. If false (already has counters), `Nothing` executes (no-op).

**No new code in `rules/` is needed.** The effect DSL already supports this:

```rust
Effect::Conditional {
    condition: Condition::SourceHasNoCountersOfType {
        counter: CounterType::PlusOnePlusOne,
    },
    if_true: Box::new(Effect::AddCounter {
        target: EffectTarget::Source,
        counter: CounterType::PlusOnePlusOne,
        count: N, // the Adapt parameter
    }),
    if_false: Box::new(Effect::Nothing),
}
```

This is the **effect** stored in the `AbilityDefinition::Activated` for each card with Adapt.

**Builder.rs note**: No automatic trigger generation from `KeywordAbility::Adapt(n)` is
needed in `builder.rs`. Unlike BattleCry/Exalted/Afterlife which need auto-generated triggers,
Adapt is a standard activated ability. The card definition explicitly defines the activated
ability with its cost and conditional effect. The `KeywordAbility::Adapt(n)` variant is purely
a marker for display and presence-checking.

If the builder.rs keyword loop is a match-all, add a no-op arm:

```rust
KeywordAbility::Adapt(_) => {
    // Adapt is an activated ability, not a triggered ability.
    // The cost and conditional effect are defined in AbilityDefinition::Activated.
    // No trigger auto-generation needed.
}
```

### Step 4: Unit Tests

**File**: `crates/engine/tests/adapt.rs` (new file)
**Tests to write**:

1. **`test_adapt_basic_adds_counters`** -- CR 701.46a: A creature with Adapt 2 and no +1/+1
   counters activates adapt. After resolution, the creature has 2 +1/+1 counters.
   Setup: 4-player game, P1 has a creature with an Adapt 2 activated ability on the
   battlefield, sufficient mana. P1 activates the ability, all players pass priority,
   ability resolves. Assert: creature has 2 +1/+1 counters.

2. **`test_adapt_does_nothing_with_existing_counters`** -- CR 701.46a (ruling 2019-01-25):
   A creature that already has a +1/+1 counter activates adapt. After resolution, no
   additional counters are placed (the Conditional evaluates to false).
   Setup: creature starts with 1 +1/+1 counter. Activate adapt, resolve. Assert: still
   only 1 +1/+1 counter (not 1+N).

3. **`test_adapt_activation_always_legal`** -- CR 701.46a (ruling 2019-01-25): "You can
   always activate an ability that will cause a creature to adapt." The activation succeeds
   and goes on the stack even if the creature already has counters. The cost is paid.
   Setup: creature has 1 +1/+1 counter, P1 has enough mana. Activate adapt. Assert:
   ability is on the stack, mana was spent. (The resolution will do nothing, but activation
   was legal.)

4. **`test_adapt_after_losing_counters`** -- Ruling 2019-01-25: "If a creature somehow loses
   all of its +1/+1 counters, it can adapt again." Setup: creature adapts (gets N counters),
   then counters are removed (e.g., via Effect::RemoveCounter), then creature adapts again.
   Assert: creature has N +1/+1 counters after the second adapt.

5. **`test_adapt_pays_mana_cost`** -- CR 602.2: The mana cost component of the activated
   ability is paid at activation time. Setup: Adapt 1 with cost {2}{G}{U}. P1 has exactly
   enough mana. Activate. Assert: mana pool is drained. If insufficient mana, activation
   fails.

6. **`test_adapt_counter_added_event_emitted`** -- Verify that `GameEvent::CounterAdded` is
   emitted when adapt successfully places counters. This event is what companion triggers
   like Sharktocrab's "whenever one or more +1/+1 counters are put on this creature" would
   listen to (tested at the card definition level, not here).

**Pattern**: Follow tests in `crates/engine/tests/abilities.rs` for activated ability patterns
(setup with `ObjectSpec::creature().with_keyword()`, `with_activated_ability()`, mana pool
setup, `ActivateAbility` command, `pass_all_four` for resolution).

**Test helper**: Create a helper function that builds a creature with Adapt N:

```rust
fn creature_with_adapt(owner: PlayerId, name: &str, power: i32, toughness: i32, adapt_n: u32, cost: ManaCost) -> ObjectSpec {
    ObjectSpec::creature(owner, name, power, toughness)
        .with_keyword(KeywordAbility::Adapt(adapt_n))
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: false,
                mana_cost: Some(cost),
                sacrifice_self: false,
            },
            description: format!("Adapt {adapt_n} (CR 701.46a)"),
            effect: Some(Effect::Conditional {
                condition: Condition::SourceHasNoCountersOfType {
                    counter: CounterType::PlusOnePlusOne,
                },
                if_true: Box::new(Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: adapt_n,
                }),
                if_false: Box::new(Effect::Nothing),
            }),
            sorcery_speed: false,
        })
}
```

### Step 5: Card Definition (later phase)

**Suggested card**: Aeromunculus (simplest Adapt card with a second keyword)
- Mana cost: {1}{G}{U}
- Type: Creature -- Homunculus Mutant
- Oracle: Flying / {2}{G}{U}: Adapt 1.
- P/T: 2/3
- Color identity: G, U
- Abilities:
  - `AbilityDefinition::Keyword(KeywordAbility::Flying)`
  - `AbilityDefinition::Keyword(KeywordAbility::Adapt(1))`
  - `AbilityDefinition::Activated { cost: Cost::Mana(ManaCost { generic: 2, green: 1, blue: 1, .. }), effect: <conditional>, timing_restriction: None }`

**Alternative**: Sharktocrab (Adapt 1 + triggered ability on counter placement)
- More complex but tests the interaction between Adapt and "whenever counters are placed" triggers.

**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: Aeromunculus on battlefield, P1 activates Adapt 1, all pass, counters
are placed. Then P1 tries to activate Adapt 1 again, all pass, no counters are placed (because
the creature already has a +1/+1 counter).

**Setup**:
- Player 1: Aeromunculus on battlefield, 8 mana in pool ({4}{G}{G}{U}{U})
- Turn 1 actions:
  1. P1 activates Adapt (ability_index 0, cost {2}{G}{U}), all pass
  2. Assert: Aeromunculus has 1 +1/+1 counter, power/toughness increased to 3/4
  3. P1 activates Adapt again (cost {2}{G}{U}), all pass
  4. Assert: Aeromunculus STILL has 1 +1/+1 counter (not 2)

**Subsystem directory**: `test-data/generated-scripts/stack/` (activated ability resolution)

## Interactions to Watch

- **Wither / Infect interaction**: If a source with Wither deals damage to a creature that has
  adapted, the -1/-1 counters are placed. SBA 704.5q annihilates +1/+1 and -1/-1 counter
  pairs. If all +1/+1 counters are annihilated, the creature can adapt again. The engine's
  existing counter annihilation SBA handles this correctly.

- **Humility interaction**: Humility removes all abilities (Layer 6). If a creature with Adapt
  has its abilities removed by Humility, it can no longer activate Adapt (the activated
  ability is gone). However, any +1/+1 counters already on it remain (counters are not
  abilities). Humility sets P/T to 1/1 in Layer 7b, but +1/+1 counters modify in Layer 7d
  (counter adjustments), so the creature would be 1/1 + counter bonuses.

- **Stifle / counter the ability**: If the Adapt activated ability is countered while on the
  stack (e.g., by Stifle), no counters are placed. The mana cost was already paid. The
  creature still has no +1/+1 counters, so Adapt can be activated again. Handled by existing
  counter-stack-object infrastructure.

- **Doubling Season / Hardened Scales**: These are replacement effects that modify "put N
  +1/+1 counters" events. They would apply to the AddCounter effect from Adapt, doubling
  or adding to the counter count. The Adapt condition check happens first (no counters? yes),
  then the counter placement happens (and replacement effects apply). This is correct per CR.

- **Removal in response**: If the creature is destroyed in response to the Adapt activation,
  the ability still resolves (it's on the stack). The `EffectTarget::Source` resolution will
  fail to find the object (it's in the graveyard with a new ObjectId), and the AddCounter
  effect will silently do nothing. This is correct -- the ability "fizzles" in the colloquial
  sense (though activated abilities with no targets technically don't fizzle per CR 608.2b).

## Files Modified (Summary)

1. `crates/engine/src/state/types.rs` -- Add `KeywordAbility::Adapt(u32)` variant
2. `crates/engine/src/cards/card_definition.rs` -- Add `Condition::SourceHasNoCountersOfType { counter: CounterType }`
3. `crates/engine/src/effects/mod.rs` -- Add `check_condition` arm for `SourceHasNoCountersOfType`
4. `crates/engine/src/state/hash.rs` -- Add hash discriminants: KeywordAbility::Adapt (67), Condition::SourceHasNoCountersOfType (8)
5. `tools/replay-viewer/src/view_model.rs` -- Add display arm for `Adapt(n)`
6. `crates/engine/src/state/builder.rs` -- Add no-op arm for `Adapt(_)` in keyword loop (if exhaustive match)
7. `crates/engine/tests/adapt.rs` -- New test file (6 tests)

## Complexity Assessment

**Low complexity.** Adapt is one of the simplest abilities to implement because:

1. It uses the existing activated ability pipeline end-to-end.
2. The condition check is a simple counter presence test.
3. No new `StackObjectKind` is needed.
4. No trigger wiring is needed.
5. No special resolution logic is needed.
6. The only new infrastructure is one `Condition` variant.

The main risk is ensuring the `SourceHasNoCountersOfType` condition correctly handles edge
cases (object not found, counter type not present in map). Both should default to "no counters"
(allowing adapt to proceed).

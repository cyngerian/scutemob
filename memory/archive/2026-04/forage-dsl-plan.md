# Forage DSL Gap Fix Plan

**Generated**: 2026-03-08
**CR**: 701.61
**Status**: Engine-side forage payment fully working; DSL `Cost` enum missing `Forage` variant

## The Gap

The engine already handles forage cost payment correctly in `rules/abilities.rs:387-434`.
The `ActivationCost` struct in `state/game_object.rs:96-113` has a `forage: bool` field.
Seven tests in `crates/engine/tests/forage.rs` validate all payment paths (Food sacrifice,
graveyard exile, insufficient resources, mana+forage, non-token Food, non-Food rejection,
deterministic fallback).

The gap is that `Cost` enum in `cards/card_definition.rs:825-838` has no `Forage` variant.
The bridge function `flatten_cost_into()` in `testing/replay_harness.rs:3118-3127` maps
`Cost` variants to `ActivationCost` fields but has no `Cost::Forage` arm. This means
`AbilityDefinition::Activated { cost: ..., effect: ... }` cannot express forage-cost
abilities, so card definitions (like Camellia, the Seedmiser) cannot declare them.

## CR Rule Text

> **701.61.** Forage
>
> **701.61a** To forage means "Exile three cards from your graveyard or sacrifice a Food."

## Changes Required

### Step 1: Add `Cost::Forage` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Location**: line ~835, inside `pub enum Cost`
**Action**: Add a new variant:

```rust
/// Forage: exile 3 cards from your graveyard or sacrifice a Food (CR 701.61a).
Forage,
```

Place it after `DiscardCard` (line 835), before the closing brace. No fields needed --
the forage action is always the same (CR 701.61a defines it completely).

### Step 2: Wire `flatten_cost_into()` to set `forage: true`

**File**: `crates/engine/src/testing/replay_harness.rs`
**Location**: line 3118-3127, function `flatten_cost_into()`
**Action**: Add a match arm:

```rust
Cost::Forage => ac.forage = true,
```

This goes after the `Cost::DiscardCard` arm (line 3124). The existing engine-side forage
payment logic in `abilities.rs:387-434` already handles everything when
`ActivationCost.forage == true`.

### Step 3: Update Camellia, the Seedmiser card definition

**File**: `crates/engine/src/cards/defs/camellia_the_seedmiser.rs`
**Action**: Add the forage activated ability to the `abilities` vec. Replace the TODO
comment block (lines 29-36) with:

```rust
// CR 701.61a: "{2}, Forage: Put a +1/+1 counter on each other Squirrel you control."
AbilityDefinition::Activated {
    cost: Cost::Sequence(vec![
        Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
        Cost::Forage,
    ]),
    effect: Effect::AddCounterToAll {
        filter: EffectFilter::CreatureSubtype(SubType("Squirrel".to_string())),
        counter: CounterType::PlusOnePlusOne,
        amount: EffectAmount::Fixed(1),
        exclude_source: true,
    },
    timing_restriction: None,
},
```

**Important**: Verify that `Effect::AddCounterToAll` (or equivalent) exists in the DSL.
If not, the effect side may need a different encoding. Check:
- `Effect::ForEach` with a filter + `Effect::AddCounter` for each target
- Or a simpler `Effect::AddCounter` with `CardEffectTarget::AllMatching(filter)`

The exact effect encoding depends on what primitives exist. Grep for `AddCounterToAll`
or `AllMatching` or review how similar "each creature" effects are expressed.

**Note**: The other two TODO abilities on Camellia (continuous Menace grant to other
Squirrels, sacrifice-Food trigger) are separate DSL gaps unrelated to Forage. They
should remain as TODOs.

### Step 4: Export `Cost::Forage` from helpers if needed

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: `Cost` is already re-exported from helpers.rs (it's used in card defs).
Verify `Cost` is in the prelude -- if so, no change needed since `Forage` is just a
new variant on an existing enum.

### Step 5: Update ability coverage doc

**File**: `docs/mtg-engine-ability-coverage.md`
**Location**: Forage row (line ~324)
**Action**: Change status from `partial` to `validated` once tests pass. Update the
code locations to include the new `Cost::Forage` variant and the replay harness bridge.

## Tests

### Existing Coverage (7 tests in `forage.rs`)

All 7 tests construct `ActivationCost { forage: true }` directly on `ObjectSpec` objects
-- they bypass the DSL entirely. They are valid and should continue to pass unchanged.

### New Tests Needed

**One integration test** to verify the DSL-to-engine bridge works end-to-end:

- `test_forage_cost_from_card_definition` -- Create a `CardDefinition` with
  `AbilityDefinition::Activated { cost: Cost::Sequence(vec![Cost::Mana(...), Cost::Forage]), ... }`,
  register it, build a game state using the replay harness (which calls
  `cost_to_activation_cost`), and verify that activating the ability correctly pays the
  forage cost. This tests the `flatten_cost_into` bridge.

Optionally, a second test with `Cost::Forage` alone (no mana cost) to verify standalone
forage works through the DSL.

**No new engine-level tests needed** -- the 7 existing tests already validate all engine
payment paths.

## Risk Assessment

**Risk**: Very low. This is a one-variant enum addition + one match arm in a bridge
function. No engine logic changes. The engine-side forage payment is already complete
and tested.

**Compile impact**: `Cost::Forage` adds a variant to an enum. Check for exhaustive
matches on `Cost` anywhere:

```
Grep pattern="Cost::" path="crates/engine/src" output_mode="files_with_matches"
```

Any `match cost { ... }` that is exhaustive will need a `Cost::Forage` arm. The main
site is `flatten_cost_into` (Step 2 above). There may be others in `casting.rs` or
`abilities.rs` if `Cost` is matched there.

## Cards Unblocked

Adding `Cost::Forage` unblocks the forage activated ability on card definitions. Cards
from Bloomburrow that use Forage as an activated cost:

- **Camellia, the Seedmiser** -- already has a card def stub; needs the activated ability
- **Curious Forager** -- ETB trigger "you may forage" (different pattern: forage as an
  optional action on trigger resolution, not as an activation cost)
- **Bushy Bodyguard** -- triggered ability with optional forage
- **Treetop Sentries** -- triggered ability with optional forage
- **Osteomancer Adept** -- triggered ability with optional forage
- **Thornvault Forager** -- triggered ability with optional forage
- **Feed the Cycle** -- instant/sorcery with "forage, then..." effect

**Note**: Most Bloomburrow forage cards use forage as a keyword action within a triggered
ability's effect ("you may forage"), not as an activation cost. `Cost::Forage` only helps
cards where forage is part of the activation cost syntax (`{N}, Forage: Effect`).
Camellia is the primary beneficiary. The triggered-forage pattern would need a separate
`Effect::Forage` or similar -- that is a different DSL gap.

## Dependency Check

No dependencies on other incomplete work. The engine payment path, `ActivationCost.forage`,
hash impl, and all 7 test cases already exist and pass.

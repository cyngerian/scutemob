# Ability Plan: Replicate

**Generated**: 2026-03-05
**CR**: 702.56
**Priority**: P4
**Similar abilities studied**: Storm (CR 702.40, `casting.rs:2595-2611`, `resolution.rs:773-792`, `copy.rs:245-259`), Casualty (CR 702.153, `casting.rs:2697-2740`, `resolution.rs:794-822`, `casualty.rs` tests)

## CR Rule Text

702.56a Replicate is a keyword that represents two abilities. The first is a static
ability that functions while the spell with replicate is on the stack. The second is a
triggered ability that functions while the spell with replicate is on the stack.
"Replicate [cost]" means "As an additional cost to cast this spell, you may pay [cost]
any number of times" and "When you cast this spell, if a replicate cost was paid for it,
copy it for each time its replicate cost was paid. If the spell has any targets, you may
choose new targets for any of the copies." Paying a spell's replicate cost follows the
rules for paying additional costs in rules 601.2b and 601.2f-h.

702.56b If a spell has multiple instances of replicate, each is paid separately and
triggers based on the payments made for it, not any other instance of replicate.

## Key Edge Cases

- **Copies created even if original countered** (Shattering Spree ruling 2024-01-12):
  "you'll copy Shattering Spree for each time you paid its replicate cost, even if the
  original spell is no longer on the stack at that time." This differs from Casualty
  where the copy fails silently if the original is gone. The ReplicateTrigger must store
  enough data to create copies even when the original is no longer on the stack.
  **Implementation note**: The current `copy_spell_on_stack` returns `Err` when the
  original stack object is gone. For Replicate, the trigger resolution must use the
  existing `create_storm_copies` pattern (which calls `copy_spell_on_stack` in a loop
  and breaks on error). The ruling edge case (copies even if countered) is a known
  limitation of the current copy infrastructure -- document it as a LOW gap.
  The `create_storm_copies` function already works correctly for the common case.
- **Copies are NOT cast** (Shattering Spree ruling 2024-01-12): "The copies that
  replicate creates are created on the stack, so they're not 'cast.' Abilities that
  trigger when a player casts a spell won't trigger." Same as Storm/Casualty.
- **Pay any number of times**: Unlike kicker (0 or 1 for standard, 0..N for multikicker),
  replicate has no upper bound. The `replicate_count` on CastSpell is a `u32` where
  0 = not paid, N = paid N times.
- **Additional cost, not alternative**: The replicate cost is added to the normal mana
  cost N times. It follows the same cost pipeline as kicker (CR 601.2f-h).
- **Multiple instances (702.56b)**: Each instance triggers independently based on its
  own payments. Not a priority for initial implementation (no cards have multiple
  replicate instances), but the data model should not preclude it.
- **Multiplayer**: No special multiplayer considerations beyond standard copy behavior.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Replicate` variant (no parameter -- the replicate cost
is stored in `AbilityDefinition::Replicate { cost: ManaCost }`, not on the keyword enum,
since the cost is card-specific and not an N-value like Casualty).

**Pattern**: Follow `KeywordAbility::Storm` (no parameter, discriminant 106).

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Replicate { cost: ManaCost }` variant.
**Pattern**: Follow `AbilityDefinition::Surge { cost: ManaCost }` (discriminant 36 for
AbilityDefinition hash).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for both:
- `KeywordAbility::Replicate` => discriminant 106
- `AbilityDefinition::Replicate { cost }` => discriminant 36

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::ReplicateTrigger` variant:
```rust
ReplicateTrigger {
    source_object: ObjectId,
    original_stack_id: ObjectId,
    replicate_count: u32,
}
```
**Pattern**: Follow `StackObjectKind::StormTrigger` (discriminant 35 for StackObjectKind hash).

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `StackObjectKind::ReplicateTrigger` => discriminant 35.
Hash all three fields: `source_object`, `original_stack_id`, `replicate_count`.

**File**: `crates/engine/src/state/stack.rs` (StackObject fields)
**Action**: No new `was_replicated` field needed on StackObject. The replicate count is
tracked via the CastSpell command's `replicate_count` field and captured in the
ReplicateTrigger. Unlike Casualty's `was_casualty_paid`, there's no per-copy flag needed.

**Match arms to update** (grep for exhaustive matches on `StackObjectKind`):
- `tools/tui/src/play/panels/stack_view.rs` -- add `ReplicateTrigger` arm with label "Replicate:"
- `tools/replay-viewer/src/view_model.rs` -- add `ReplicateTrigger` arm with kind "replicate_trigger"
- `crates/engine/src/rules/resolution.rs` -- add resolution arm (Step 2) AND counter arm (~line 3361)
- `crates/engine/src/effects/mod.rs` -- ward counter arm (~line 851, the `_ =>` catch-all)

### Step 2: CastSpell Command Extension

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `replicate_count: u32` field to `Command::CastSpell`. Default 0.
```rust
/// CR 702.56a: Number of times the replicate cost was paid.
/// 0 = not paid. N = paid N times. Each payment adds the replicate cost
/// to the total mana cost. Validated against the spell having the Replicate keyword.
#[serde(default)]
replicate_count: u32,
```
**Pattern**: Follow `kicker_times: u32` at line ~100.

### Step 3: Rule Enforcement (casting.rs)

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Two additions:

**3a. Cost validation and payment** (near kicker handling, ~line 1722):
- If `replicate_count > 0`, validate the spell has `KeywordAbility::Replicate`.
- Look up the replicate cost from `AbilityDefinition::Replicate { cost }` in the card's
  ability list (similar to `get_kicker_cost`).
- Add `cost * replicate_count` to the total mana cost.
- CR 702.56a / CR 601.2f-h: The replicate cost is an additional cost paid as part of
  casting.

**3b. Trigger creation** (after storm trigger, ~line 2695):
- If `replicate_count > 0` (from the command, NOT from keywords -- the count is player-declared):
  - Create a `StackObjectKind::ReplicateTrigger` with `replicate_count`, `source_object`,
    and `original_stack_id`.
  - Push onto stack above the spell.
  - Emit `GameEvent::TriggerPlaced`.
- CR 702.56a: "When you cast this spell, if a replicate cost was paid for it, copy it
  for each time its replicate cost was paid."

**Pattern**: Follow Storm trigger creation at `casting.rs:2595-2640` and Casualty trigger
creation at `casting.rs:2697-2740`.

**Helper function**: Add `get_replicate_cost(card_id, registry) -> Option<ManaCost>` near
`get_kicker_cost`. It searches the card's abilities for `AbilityDefinition::Replicate { cost }`
and returns the cost if found.

### Step 4: Trigger Resolution (resolution.rs)

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution arm for `StackObjectKind::ReplicateTrigger`:
```rust
StackObjectKind::ReplicateTrigger {
    source_object: _,
    original_stack_id,
    replicate_count,
} => {
    let controller = stack_obj.controller;
    let copy_events = crate::rules::copy::create_storm_copies(
        state,
        original_stack_id,
        controller,
        replicate_count,
    );
    events.extend(copy_events);
    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```
**Pattern**: Identical to `StormTrigger` resolution at `resolution.rs:773-792`.
Reuses `create_storm_copies` which calls `copy_spell_on_stack` N times.

**CR**: 702.56a -- "copy it for each time its replicate cost was paid."

**Counter arm**: Add `StackObjectKind::ReplicateTrigger { .. }` to the exhaustive
counter-spell match at ~line 3328-3373.
Note: If countered (e.g., by Stifle), no copies are made but the original spell
remains on the stack.

### Step 5: Copy propagation (copy.rs)

**File**: `crates/engine/src/rules/copy.rs`
**Action**: No changes needed. The `copy_spell_on_stack` function already handles all
the copy semantics. Replicate copies don't need any special flag (they are `is_copy: true`,
not cast, same targets as original).

If a `was_replicated` field is ever added to StackObject in the future (for "if this
spell's replicate cost was paid" conditions), it should NOT propagate to copies
(copies are not cast, so they cannot have replicate paid). Currently no cards check this.

### Step 6: Unit Tests

**File**: `crates/engine/tests/replicate.rs` (new file)
**Tests to write**:

- `test_replicate_basic_two_copies` -- CR 702.56a: Pay replicate cost twice, verify
  ReplicateTrigger on stack, resolve it to get 2 copies + original = 3 spells resolving.
  Assert life gain (or similar effect) x3.
  **Pattern**: Follow `test_casualty_basic_copy` in `casualty.rs`.

- `test_replicate_zero_copies` -- CR 702.56a: Cast with `replicate_count: 0` (do not
  pay replicate). No trigger on stack, spell resolves normally once.
  **Pattern**: Follow `test_casualty_optional_no_sacrifice`.

- `test_replicate_one_copy` -- CR 702.56a: Pay once, verify exactly one copy created.

- `test_replicate_no_keyword_rejected` -- Engine validation: Providing `replicate_count > 0`
  for a spell without Replicate keyword must be rejected.
  **Pattern**: Follow `test_casualty_spell_without_keyword`.

- `test_replicate_copies_not_cast` -- Ruling 2024-01-12: Copies are NOT cast.
  `spells_cast_this_turn` increments only once (for the original).
  **Pattern**: Follow `test_casualty_copy_is_not_cast`.

- `test_replicate_mana_cost_added` -- CR 702.56a / CR 601.2f: Replicate cost is added
  to total mana cost N times. If replicate cost is {1}{R} and replicate_count is 2,
  total additional cost is {2}{R}{R}. Verify the spell requires the correct total mana
  to cast (insufficient mana -> rejected).

- `test_replicate_with_targets` -- CR 702.56a: If the spell has targets, copies keep the
  same targets (choose-new-targets not yet interactive). Verify targeted copies resolve
  against the same target.

### Step 7: Card Definition (later phase)

**Suggested card**: Shattering Spree ({R} sorcery, Replicate {R}, destroy target artifact)
**Alternative**: Train of Thought ({1}{U} sorcery, Replicate {1}{U}, draw a card) --
simpler since no targets.
**Card lookup**: use `card-definition-author` agent.

### Step 8: Game Script (later phase)

**Suggested scenario**: Cast Train of Thought with replicate_count=2, verify 3 cards drawn
total (2 copies + 1 original).
**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

- **Storm + Replicate on same spell**: Both are triggered abilities that fire "when you
  cast this spell." Both would trigger. Storm creates copies based on spells_cast_this_turn;
  Replicate creates copies based on replicate_count. Both sets of copies are independent.
  No cards currently have both, but the trigger ordering follows APNAP (same controller
  chooses order, CR 603.3b).

- **Copy infrastructure**: Replicate reuses `create_storm_copies` which calls
  `copy_spell_on_stack` in a loop. This is the identical pattern to Storm. No new copy
  infrastructure needed.

- **Kicker interaction**: If a Replicate spell also has kicker, the copies inherit
  `kicker_times_paid` from the original (already handled by `copy_spell_on_stack` at
  `copy.rs:176`). CR 707.2: copies copy choices made during casting.

- **Countering the trigger (Stifle)**: If the ReplicateTrigger is countered, no copies
  are made. The original spell stays on the stack. This is the same behavior as countering
  a StormTrigger.

- **Countering the original spell**: If the original spell is countered between casting
  and the ReplicateTrigger resolving, `copy_spell_on_stack` returns `Err` and
  `create_storm_copies` breaks. The ruling says copies should still be created, but this
  is a known limitation of the current copy infrastructure (same as Storm). Document as
  LOW gap.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| `KeywordAbility` | `Replicate` | 106 |
| `AbilityDefinition` | `Replicate { cost }` | 36 |
| `StackObjectKind` | `ReplicateTrigger` | 35 |

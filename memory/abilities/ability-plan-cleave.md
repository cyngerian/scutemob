# Ability Plan: Cleave

**Generated**: 2026-03-05
**CR**: 702.148 (NOT 702.150 as originally stated -- MCP-verified)
**Priority**: P4
**Similar abilities studied**: Overload (CR 702.96) -- alt cost that changes spell behavior;
Vandalblast card def at `crates/engine/src/cards/defs/vandalblast.rs`; Bargain condition pattern

## CR Rule Text

702.148. Cleave

702.148a Cleave is a keyword that represents two static abilities that function while a
spell with cleave is on the stack. "Cleave [cost]" means "You may cast this spell by
paying [cost] rather than paying its mana cost" and "If this spell's cleave cost was paid,
change its text by removing all text found within square brackets in the spell's rules
text." Casting a spell for its cleave cost follows the rules for paying alternative costs
in rules 601.2b and 601.2f-h.

702.148b Cleave's second ability is a text-changing effect. See rule 612,
"Text-Changing Effects."

## Key Edge Cases

- **Alt cost exclusivity (CR 601.2b)**: Cannot combine cleave with flashback or any other
  alt cost. Ruling: "You can't cast a spell for both its cleave cost and another
  alternative cost."
- **Mana value unchanged**: "Casting a spell for its cleave cost doesn't change the spell's
  mana value." (all cleave card rulings confirm this)
- **"Cast without paying mana cost" blocks cleave**: "If an effect allows you to cast a
  spell without paying its mana cost, you can't cast that spell for its cleave cost."
  (This is standard alt-cost behavior -- already handled by the engine's alt-cost pipeline.)
- **Text removal is on the stack**: "If you cast a spell for its cleave cost, that spell
  doesn't have any of the text in square brackets while it's on the stack." This means the
  cleaved text is gone at resolution time, not just at casting time.
- **Bracket content varies widely across cards**:
  - Dig Up: removes `[basic land]` and `[reveal it,]` -- changes search target AND removes
    reveal step
  - Path of Peril: removes `[with mana value 2 or less]` -- broadens destruction scope
  - Fierce Retribution: removes `[attacking]` -- broadens target filter
  - Alchemist's Retrieval: removes `[you control]` -- broadens target controller
  - Wash Away: removes `[that wasn't cast from its owner's hand]` -- broadens counterable spells
- **Multiplayer**: No special multiplayer interactions. Cleave is a simple alt cost +
  text-changing effect. All standard alt cost rules apply.

## DSL Modeling Strategy

In the rules engine, we do NOT model oracle text directly -- we model effects. The
"bracket removal" in oracle text corresponds to using a **different effect** or a
**different target filter** when cleaved. This is exactly the same pattern as Overload,
which uses `Condition::WasOverloaded` to branch between single-target and all-target
effects.

For Cleave, we use `Condition::WasCleaved` to branch between:
- `if_false`: the "normal" effect (with bracket text included -- more restrictive)
- `if_true`: the "cleaved" effect (bracket text removed -- broader/different)

Each card definition manually models both branches. This is faithful to the DSL
approach -- the bracket text is not parsed at runtime; instead, card authors encode
both versions of the effect.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant + AltCostKind + AbilityDefinition + Condition

#### 1a: KeywordAbility::Cleave

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Cleave` variant to `KeywordAbility` enum after `Gravestorm`.
**Pattern**: Follow `KeywordAbility::Overload` (no parameters -- just a marker).

#### 1b: AltCostKind::Cleave

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `Cleave` variant to `AltCostKind` enum after `Surge`.
**CR**: 702.148a -- cleave cost is an alternative cost.

#### 1c: AbilityDefinition::Cleave

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `Cleave { cost: ManaCost }` variant to `AbilityDefinition` enum.
**Pattern**: Follow `AbilityDefinition::Overload { cost: ManaCost }` at line ~263.
**Doc comment**: `/// CR 702.148: Cleave [cost]. Alternative cost; when paid, square-bracketed text
/// is removed from the spell -- modeled as conditional effect dispatch via
/// Condition::WasCleaved. Cards should also include
/// AbilityDefinition::Keyword(KeywordAbility::Cleave).`

#### 1d: Condition::WasCleaved

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `WasCleaved` variant to `Condition` enum after `WasBargained`.
**Pattern**: Follow `Condition::WasOverloaded` / `Condition::WasBargained`.
**Doc comment**: `/// CR 702.148a: "if this spell's cleave cost was paid" -- true when
/// was_cleaved is set on the EffectContext. Checked at resolution time.`

#### 1e: Hash discriminants

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for:
- `KeywordAbility::Cleave` -- discriminant **108**
- `AbilityDefinition::Cleave { cost }` -- discriminant **37**
- `Condition::WasCleaved` -- discriminant **11**

**Pattern**: Follow the existing hash arms for Overload/Bargain/Replicate.

### Step 2: StackObject + EffectContext + Casting

#### 2a: StackObject.was_cleaved

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `pub was_cleaved: bool` field to `StackObject`, with `#[serde(default)]`.
**Pattern**: Follow `was_overloaded: bool` at line ~134.
**Hash**: Add to `hash.rs` StackObject HashInto impl.

#### 2b: EffectContext.was_cleaved

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add `pub was_cleaved: bool` field to `EffectContext`.
**Pattern**: Follow `was_overloaded: bool` at line ~69.
**Update**: `EffectContext::new()` -- set `was_cleaved: false`.
**Update**: `EffectContext::new_with_kicker()` -- set `was_cleaved: false`.
**Update**: All `EffectContext` construction sites in ForEach inner context creation
(lines ~1356 and ~1375) -- propagate `was_cleaved: ctx.was_cleaved`.

#### 2c: Condition evaluation

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add arm to `evaluate_condition()` match:
```rust
// CR 702.148a: "if this spell's cleave cost was paid" -- true when cleaved.
Condition::WasCleaved => ctx.was_cleaved,
```
**Pattern**: Follow `Condition::WasOverloaded => ctx.was_overloaded` at line ~2781.

#### 2d: Casting pipeline

**File**: `crates/engine/src/rules/casting.rs`
**Action**:
1. Add `let cast_with_cleave = alt_cost == Some(AltCostKind::Cleave);` near line 83.
2. Add validation block (similar to overload block at line ~632):
   - If `cast_with_cleave`, verify card has `AbilityDefinition::Cleave { .. }`.
   - Look up the cleave cost from the card's abilities.
   - Mutually exclusive with other alt costs (already enforced by `alt_cost: Option<AltCostKind>`).
3. If `cast_with_cleave`, use the cleave cost as the base cost (instead of mana_cost).
4. Add `get_cleave_cost()` helper function.
   **Pattern**: Follow `get_overload_cost()` at line ~3207.
5. Set `was_cleaved: casting_with_cleave` on the `StackObject` construction (line ~2518 area).

**CR**: 702.148a -- casting for cleave cost follows rules for alternative costs (601.2b, 601.2f-h).

**Important difference from Overload**: Overload removes all targets (CR 702.96b -- "no targets").
Cleave does NOT inherently remove targets. Cleave changes the spell's text, which may or may
not affect targeting. Some cleave cards still have targets when cleaved (e.g., Fierce Retribution:
"Destroy target creature" -- just without the "[attacking]" restriction). Other cleave cards
may have different target requirements when cleaved vs. not. The card definition handles this
via `Condition::WasCleaved` branching in the Spell effect -- the target requirements in the
`AbilityDefinition::Spell` should be the UNION of both modes (the broadest targeting), and
the Conditional effect branches handle the actual behavior.

**Note on targeting**: Unlike Overload (which makes the spell targetless), Cleave cards
keep their targets. The target filter may be broader when cleaved (e.g., Fierce Retribution
targets any creature instead of only attacking creatures). Model this by having the
`AbilityDefinition::Spell.targets` use the broadest filter, and the `if_false` branch
of the Conditional validates the restriction at resolution time (if the target doesn't
meet the un-cleaved restriction, the spell fizzles normally through target legality).

Actually, the simpler approach: use `Condition::WasCleaved` to provide TWO different
`AbilityDefinition::Spell` variants -- but the engine only supports one Spell per card.
The correct approach (matching Overload exactly): single `AbilityDefinition::Spell` with
the broadest target requirements, and a `Conditional` effect that branches on `WasCleaved`.
For cards where cleave changes the target filter (like Fierce Retribution), the broad
target filter "target creature" is used, and the `if_false` branch checks for attacking.

#### 2e: Resolution pipeline

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Pass `was_cleaved` from `StackObject` to `EffectContext` at resolution time.
**Pattern**: Follow `ctx.was_overloaded = stack_obj.was_overloaded;` at line ~207.
Add: `ctx.was_cleaved = stack_obj.was_cleaved;`

#### 2f: Default false in all StackObject construction sites

**Files**: `crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/resolution.rs`
(copy construction), `crates/engine/src/rules/suspend.rs`, anywhere StackObject is
constructed.
**Action**: Add `was_cleaved: false` (or `was_cleaved: casting_with_cleave` in the
casting path).
**Pattern**: Grep for `was_overloaded:` to find all sites and add `was_cleaved:` next to each.

### Step 3: Trigger Wiring

**Not applicable.** Cleave is purely a static ability (alt cost + text change). It does not
produce triggers. No trigger wiring needed.

### Step 4: Unit Tests

**File**: `crates/engine/tests/cleave.rs` (new file)

**Tests to write**:

1. `test_cleave_basic_cast_with_cleave_cost` -- CR 702.148a
   - Card: mock Path of Peril (`{1}{B}{B}`, cleave `{4}{W}{B}`)
   - Normal: "Destroy all creatures [with mana value 2 or less]"
   - Cleaved: "Destroy all creatures"
   - Cast with `alt_cost: Some(AltCostKind::Cleave)`, pay `{4}{W}{B}`
   - Assert: all creatures destroyed (including those with MV > 2)
   - Pattern: follow overload test structure

2. `test_cleave_normal_cast_restricted` -- CR 702.148a (negative)
   - Same mock card, cast normally for `{1}{B}{B}`
   - Assert: only creatures with MV <= 2 destroyed; high-MV creatures survive

3. `test_cleave_cost_is_alternative_cost` -- CR 601.2b
   - Verify cleave and flashback cannot combine
   - Cast with `alt_cost: Some(AltCostKind::Cleave)` on a card that also has flashback
   - Assert: error (mutually exclusive alt costs -- already enforced by single `alt_cost` field)

4. `test_cleave_mana_value_unchanged` -- Ruling 2021-11-19
   - Cast a cleave spell for its cleave cost
   - Assert: on the stack, the spell's mana value equals the original mana cost (not the cleave cost)
   - Pattern: similar to overload mana value test

5. `test_cleave_cannot_cast_without_paying_mana_cost` -- Ruling 2021-11-19
   - If an effect allows casting without paying mana cost, cleave cannot be used
   - This is standard alt-cost exclusivity -- verify via the `alt_cost` field

6. `test_cleave_broadened_target_filter` -- Fierce Retribution pattern
   - Mock card: "Destroy target [attacking] creature" / cleave broadens to any creature
   - Normal cast: must target an attacking creature
   - Cleaved cast: can target any creature
   - This tests the `Condition::WasCleaved` branching on target restrictions

7. `test_cleave_search_broadened` -- Dig Up pattern
   - Mock card: "Search for a [basic land] card" vs. "Search for a card"
   - Normal: finds basic land only
   - Cleaved: finds any card
   - Tests the Conditional branching for SearchLibrary effect parameters

**Pattern**: Follow `crates/engine/tests/overload.rs` for test structure (mock card defs,
CastSpell command construction, state assertions).

### Step 5: Card Definition (later phase)

**Suggested card**: Path of Peril (simplest cleave card -- no targets, just a board wipe
with a MV filter that gets removed)

**Oracle text**: "Cleave {4}{W}{B}. Destroy all creatures [with mana value 2 or less]."

**Card definition structure**:
```rust
abilities: vec![
    AbilityDefinition::Keyword(KeywordAbility::Cleave),
    AbilityDefinition::Cleave {
        cost: ManaCost { generic: 4, white: 1, black: 1, ..Default::default() },
    },
    AbilityDefinition::Spell {
        effect: Effect::Conditional {
            condition: Condition::WasCleaved,
            if_true: Box::new(Effect::ForEach {
                over: ForEachTarget::EachPermanentMatching(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
                effect: Box::new(Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                }),
            }),
            if_false: Box::new(Effect::ForEach {
                over: ForEachTarget::EachPermanentMatching(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    max_mana_value: Some(2),  // needs DSL support
                    ..Default::default()
                }),
                effect: Box::new(Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                }),
            }),
        },
        targets: vec![],  // no targets -- board wipe
        modes: None,
        cant_be_countered: false,
    },
],
```

**Note**: `max_mana_value` filter on `TargetFilter` may not exist yet. If not, it will
need to be added or an alternative filtering approach used (e.g., `Condition` check
inside the ForEach loop). The runner should check whether `TargetFilter` has a
`max_mana_value` field.

**Alternative suggested card**: Fierce Retribution -- simpler (single target destruction,
bracket removes `[attacking]` restriction). May be easier to model if the MV filter is
not yet in TargetFilter.

### Step 6: Game Script (later phase)

**Suggested scenario**: Path of Peril cleave vs. normal cast
**Subsystem directory**: `test-data/generated-scripts/stack/`

Scenario: P1 has Path of Peril. Battlefield has a mix of low-MV and high-MV creatures.
- Turn 1: P1 casts Path of Peril normally for {1}{B}{B} -- only low-MV creatures die
- Turn 2: P1 casts Path of Peril with cleave for {4}{W}{B} -- all creatures die

## Interactions to Watch

- **Alt cost pipeline**: Cleave is an alternative cost. The existing `AltCostKind` + `alt_cost`
  field on CastSpell already enforces mutual exclusivity with other alt costs. No special
  handling needed beyond adding the variant and wiring the cost lookup.
- **Commander tax**: Commander tax applies on top of cleave cost (same as overload --
  CR 118.9d). Already handled by the casting pipeline's cost-modifier order.
- **Overload vs. Cleave**: Very similar mechanically (alt cost + changed spell behavior).
  Key difference: Overload removes all targets; Cleave does NOT inherently remove targets.
  Cleave cards may still have targets after cleaving. The engine handles this correctly
  because target requirements are defined in `AbilityDefinition::Spell.targets` and the
  `Condition::WasCleaved` only affects the effect execution, not the targeting.
- **Copy effects**: Copies of cleaved spells should preserve the cleaved status (the
  bracketed text is already removed on the stack). The existing copy infrastructure
  copies all StackObject fields including boolean flags, so `was_cleaved` will be
  preserved automatically. Verify in tests.
- **Text-changing effects (CR 612)**: The CR says cleave is a text-changing effect, but
  in our DSL we model effects directly, not oracle text. The `Condition::WasCleaved`
  branching is our representation of the text change. No interaction with other
  text-changing effects needs to be modeled (there are essentially none in Commander).

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | Cleave | 108 |
| AbilityDefinition | Cleave { cost } | 37 |
| Condition | WasCleaved | 11 |
| AltCostKind | Cleave | (no discriminant -- uses derived Hash) |
| StackObject | was_cleaved | (bool field, hashed inline) |

## Files to Modify

1. `crates/engine/src/state/types.rs` -- KeywordAbility::Cleave, AltCostKind::Cleave
2. `crates/engine/src/cards/card_definition.rs` -- AbilityDefinition::Cleave, Condition::WasCleaved
3. `crates/engine/src/state/hash.rs` -- hash discriminants for all new variants
4. `crates/engine/src/state/stack.rs` -- StackObject.was_cleaved field
5. `crates/engine/src/effects/mod.rs` -- EffectContext.was_cleaved, evaluate_condition arm
6. `crates/engine/src/rules/casting.rs` -- cleave cost lookup, validation, StackObject construction
7. `crates/engine/src/rules/resolution.rs` -- propagate was_cleaved to EffectContext
8. `crates/engine/src/rules/suspend.rs` -- was_cleaved: false on suspend cast StackObject
9. `crates/engine/src/testing/replay_harness.rs` -- (optional) if cleave-specific harness action needed
10. `crates/engine/tests/cleave.rs` -- new test file
11. `tools/tui/src/play/panels/stack_view.rs` -- if StackObjectKind gets a new variant (unlikely -- cleave uses existing spell kinds)

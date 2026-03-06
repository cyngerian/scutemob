# Ability Plan: Entwine

**Generated**: 2026-03-06
**CR**: 702.42
**Priority**: P4
**Similar abilities studied**: Kicker (additional cost pattern in `casting.rs`), Buyback (additional cost in `casting.rs`), Replicate (additional cost + `replicate_count` on CastSpell), Cleave (`was_cleaved` flag on StackObject + EffectContext + `Condition::WasCleaved`)

## CR Rule Text

702.42. Entwine

702.42a Entwine is a static ability of modal spells (see rule 700.2) that functions while
the spell is on the stack. "Entwine [cost]" means "You may choose all modes of this spell
instead of just the number specified. If you do, you pay an additional [cost]." Using the
entwine ability follows the rules for choosing modes and paying additional costs in rules
601.2b and 601.2f-h.

702.42b If the entwine cost was paid, follow the text of each of the modes in the order
written on the card when the spell resolves.

Related: CR 700.2 (modal spells), CR 601.2b (mode/cost announcement), CR 601.2f-h (paying
additional costs).

## Key Edge Cases

- **Modes execute in printed order** (CR 702.42b). When entwined, mode[0] resolves first,
  then mode[1], etc. This is not player-chosen order.
- **Entwine cost is an additional cost** (CR 702.42a), not an alternative cost. It stacks
  with commander tax, kicker, and other additional costs. It does NOT replace the mana cost.
- **Entwine does not change the spell's color** (ruling on Twisted Reflection 2019-06-14).
- **No priority between modes during resolution** (ruling on Grab the Reins 2004-12-01).
  Both modes resolve as a single resolution event.
- **Effects from earlier modes are visible to later modes** (ruling on Promise of Power
  2004-12-01; ruling on Unbounded Potential 2021-06-18: proliferate sees counters from mode 1).
- **Auto-all-modes approach (batch plan)**: Full interactive mode selection is deferred to
  Batch 11. For now: when `entwine_paid == true`, automatically select ALL modes; when
  `entwine_paid == false`, automatically select mode[0] (first mode only). This matches
  the batch plan's "stub with auto-all-modes" guidance.

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
**Action**: Add `KeywordAbility::Entwine` variant after `Splice` (around line 976).
Doc comment: `/// CR 702.42a: Entwine [cost] -- optional additional cost. When paid, the caster chooses all modes of this modal spell instead of just one. The entwine cost itself is stored in AbilityDefinition::Entwine { cost }.`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Entwine { cost: ManaCost }` variant after `Splice`.
Doc comment: `/// CR 702.42: Entwine [cost]. Optional additional cost that selects all modes. Cards with this ability should also include AbilityDefinition::Keyword(KeywordAbility::Entwine) for quick presence-checking. AbilityDefinition::Spell.modes must be Some(...) for entwine to be meaningful.`

**File**: `crates/engine/src/state/hash.rs`
**Action**:
1. Add `KeywordAbility::Entwine` arm to the `KeywordAbility` hash impl. Use discriminant 110.
2. Add `AbilityDefinition::Entwine { cost }` arm. Use discriminant 39.
3. Add `was_entwined` to the `StackObject` hasher (see Step 2).

**Match arms**: Grep for `KeywordAbility::` match expressions and add `Entwine` arm where needed:
- `hash.rs` KeywordAbility match (discriminant 110)
- `hash.rs` AbilityDefinition match (discriminant 39)
- Any other exhaustive matches on KeywordAbility

### Step 2: Rule Enforcement (Casting + Resolution)

#### 2a: CastSpell command — add `entwine_paid` field

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `entwine_paid: bool` field to `CastSpell` variant (after `splice_cards`, around line 231).
Doc comment: `/// CR 702.42a: If true, the entwine additional cost was paid. When true, all modes of the modal spell are chosen instead of just one. Validated against the spell having KeywordAbility::Entwine.`
Default: `#[serde(default)]`

#### 2b: StackObject — add `was_entwined` field

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `was_entwined: bool` field to `StackObject` (after `spliced_card_ids`, around line 243).
Doc comment: `/// CR 702.42a: If true, this spell was cast with its entwine cost paid. At resolution, all modes of the modal spell execute in printed order (CR 702.42b).`
Default: `#[serde(default)]`

Also: Add `was_entwined: false` to ALL existing `StackObject` construction sites in:
- `casting.rs` (main StackObject creation, ~line 2654+; all alt-cost StackObject sites ~lines 2816, 2873, 2931, 2983, 3037)
- `resolution.rs` (copy creation sites for storm, cascade, casualty, replicate, gravestorm)
- `copy.rs` (spell copy creation)

#### 2c: Casting validation — entwine cost addition

**File**: `crates/engine/src/rules/casting.rs`
**Action**: After the replicate cost section (~line 1840) and before convoke/improvise/delve reductions, add an entwine cost validation + addition block. Pattern follows kicker/buyback/replicate exactly:

1. If `entwine_paid == true`:
   - Validate the spell has `KeywordAbility::Entwine` in its characteristics keywords
   - Look up the entwine cost via a new `get_entwine_cost()` helper function
   - Add the entwine cost to the total mana cost (CR 601.2f)
   - If spell does not have entwine, return `Err(InvalidCommand("spell does not have entwine"))`
2. If `entwine_paid == false`: no-op.
3. Store `was_entwined: entwine_paid` on the StackObject.

**New function** `get_entwine_cost()`:
Pattern: Follow `get_buyback_cost()` (line 3157) and `get_kicker_cost()` (line 3115).
```rust
fn get_entwine_cost(card_id: &CardId, registry: &CardRegistry) -> Option<ManaCost> {
    registry.get(card_id.clone())?.abilities.iter().find_map(|a| {
        if let AbilityDefinition::Entwine { cost } = a {
            Some(cost.clone())
        } else {
            None
        }
    })
}
```

**Cost pipeline position** (from `casting.rs` line 2369 comment):
`base_mana_cost -> alt_cost -> commander_tax -> kicker -> affinity -> undaunted -> buyback -> replicate -> ENTWINE -> convoke -> improvise -> delve -> pay`

#### 2d: Resolution — mode dispatch based on `was_entwined`

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Modify the spell resolution block (lines 162-236) to check `was_entwined` and dispatch modes.

Currently, resolution extracts `spell_effect` from `AbilityDefinition::Spell { effect, .. }` and executes it as a single `Effect`. For entwine, the approach is:

1. After extracting the `AbilityDefinition::Spell { effect, modes, .. }`, check `stack_obj.was_entwined`.
2. **If `was_entwined == true` AND `modes.is_some()`**: Execute each mode in `modes.modes` sequentially (CR 702.42b: "follow the text of each of the modes in the order written on the card"). Each mode is an `Effect` that executes via `execute_effect()` with the same `EffectContext`.
3. **If `was_entwined == false` AND `modes.is_some()`**: Execute only `modes.modes[0]` (auto-select first mode, per batch plan "auto-all-modes" stub).
4. **If `modes.is_none()`**: Execute `effect` as before (non-modal spell, no change).

This is the key behavioral change. The `effect` field on `AbilityDefinition::Spell` is the fallback for non-modal spells; for modal spells, modes take precedence.

**Important**: Each mode's `Effect` shares the same `EffectContext` (same controller, source, targets, flags). Effects from mode[0] that modify state are visible to mode[1] (per rulings).

### Step 3: Trigger Wiring

**Not applicable.** Entwine is a static ability that modifies how modes are chosen during casting (CR 702.42a). It does not generate triggers. No `StackObjectKind` variant or trigger dispatch is needed.

### Step 4: Unit Tests

**File**: `crates/engine/tests/entwine.rs`
**Tests to write**:

1. `test_entwine_basic_both_modes_execute` -- CR 702.42b: Cast a modal spell with entwine paid. Both modes execute in printed order. Assert both effects are visible (e.g., mode[0] = GainLife, mode[1] = DrawCards; verify life total increased AND hand size increased).

2. `test_entwine_not_paid_only_first_mode` -- CR 702.42a/700.2: Cast the same modal spell without paying entwine. Only mode[0] executes. Assert mode[0] effect occurred and mode[1] did not.

3. `test_entwine_additional_cost_added_to_total` -- CR 601.2f: Cast with entwine paid. Verify the total mana paid includes base cost + entwine cost. Test with insufficient mana to confirm rejection.

4. `test_entwine_no_entwine_keyword_rejected` -- Engine validation: Attempt to cast a non-entwine spell with `entwine_paid: true`. Verify `InvalidCommand` error.

5. `test_entwine_modes_in_printed_order` -- CR 702.42b: Cast an entwined spell where mode[0] modifies state (e.g., adds counters) and mode[1] reads that state (e.g., draws cards equal to counters). Verify mode[1] sees the result of mode[0].

6. `test_entwine_stacks_with_commander_tax` -- CR 601.2f + CR 903.8: Cast from command zone with entwine. Verify total cost = base + commander tax + entwine.

**Pattern**: Follow `crates/engine/tests/kicker.rs` structure (helpers, registry, pass_all, find_object).

**Test card approach**: Create a synthetic test card definition for entwine tests:
- Name: "Entwine Test Spell"
- Type: Instant
- Cost: {1}{U}
- Modes: [GainLife(3), DrawCards(1)]
- Entwine: {2}
This avoids needing a real card definition and keeps tests self-contained (same pattern as kicker tests using Burst Lightning).

### Step 5: Card Definition (later phase)

**Suggested card**: Tooth and Nail
- Sorcery, {5}{G}{G}
- Choose one: Search library for up to two creature cards / Put up to two creature cards from hand onto battlefield
- Entwine {2}

However, Tooth and Nail uses SearchLibrary (complex, may have harness gaps). A simpler alternative for testing:

**Alternative**: "Reap and Sow" (Sorcery, {3}{G}, Choose one: Destroy target land / Search for a basic land. Entwine {1}{G}) -- still uses SearchLibrary.

**Simplest practical card**: "Promise of Power" (Sorcery, {2}{B}{B}{B}, Choose one: Draw 5 cards, lose 5 life / Create an X/X Demon token. Entwine {4}) -- both modes use existing DSL effects (DrawCards, GainLife negative, CreateToken).

**Card lookup**: use `card-definition-author` agent for the chosen card.

### Step 6: Game Script (later phase)

**Suggested scenario**: Two-turn test.
- Turn 1: Cast the entwine spell with `entwine_paid: true`. Assert both mode effects resolved.
- Turn 2: Cast the entwine spell with `entwine_paid: false`. Assert only mode[0] resolved.

**Subsystem directory**: `test-data/generated-scripts/stack/`

### Step 7: Harness Support

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"cast_spell_entwine"` action type that sets `entwine_paid: true` on the CastSpell command. Pattern follows `"cast_spell_replicate"` (line 1270).

**File**: `crates/engine/src/testing/script_schema.rs`
**Action**: Add `entwine_paid` field to `PlayerAction` (serde default false). Or use the action-type approach (`"cast_spell_entwine"` implies `entwine_paid: true`).

## Interactions to Watch

- **Kicker + Entwine on the same spell**: Both are additional costs; both stack on top of the base mana cost. The cost pipeline handles them independently. No special interaction needed.
- **Storm/Cascade copies of entwined spells**: CR 707.10 says copies copy mode choices. The `was_entwined` flag should be propagated to copies (same as `was_bargained`, `was_overloaded`). Check `copy.rs` copy-spell sites.
- **Overload + Entwine**: Cannot coexist on the same card (Overload replaces targeting, Entwine works on modes). No conflict to worry about.
- **Splice + Entwine**: Spliced effects execute after all modes (CR 702.47b). The current splice resolution code in `resolution.rs` (lines 218-236) already runs after the main spell effect. If we change the main spell effect to iterate modes, splice effects should still follow naturally since they are separate from the mode loop.
- **Multiplayer**: No special multiplayer implications. Entwine is a casting choice by the caster. Priority rotation is unaffected.
- **Modal choice (Batch 11)**: When full interactive mode selection is implemented in Batch 11, the "auto-select mode[0]" fallback must be replaced with actual player choice. Entwine's "choose all" override should be easy to integrate since it bypasses the choice entirely.

## File Summary

| File | Action |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Entwine` |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Entwine { cost }` |
| `crates/engine/src/state/hash.rs` | Add hash arms (KW disc 110, AbilDef disc 39, StackObject `was_entwined`) |
| `crates/engine/src/rules/command.rs` | Add `entwine_paid: bool` to CastSpell |
| `crates/engine/src/state/stack.rs` | Add `was_entwined: bool` to StackObject |
| `crates/engine/src/rules/casting.rs` | Validate entwine keyword, add entwine cost, store flag; `get_entwine_cost()` helper |
| `crates/engine/src/rules/resolution.rs` | Mode dispatch: was_entwined -> all modes; else -> mode[0] |
| `crates/engine/src/testing/replay_harness.rs` | `"cast_spell_entwine"` action type |
| `crates/engine/src/testing/script_schema.rs` | Optional: `entwine_paid` field |
| `crates/engine/tests/entwine.rs` | 6 unit tests |
| `tools/tui/src/play/panels/stack_view.rs` | No change needed (no new StackObjectKind) |

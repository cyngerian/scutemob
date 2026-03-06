# Ability Plan: Escalate

**Generated**: 2026-03-06
**CR**: 702.120 (NOT 702.121 -- that is Melee; batch plan had the wrong number)
**Priority**: P4
**Similar abilities studied**: Entwine (CR 702.42) -- `types.rs:L988-994`, `casting.rs:L1850-1878`, `resolution.rs:L210-229`, `stack.rs:L225-231`, `hash.rs:L558-559,L1728-1729,L3345-3349`, `tests/entwine.rs`

## CR Rule Text

**702.120. Escalate**

> 702.120a Escalate is a static ability of modal spells (see rule 700.2) that functions
> while the spell with escalate is on the stack. "Escalate [cost]" means "For each mode
> you choose beyond the first as you cast this spell, you pay an additional [cost]."
> Paying a spell's escalate cost follows the rules for paying additional costs in rules
> 601.2f-h.

There is only one sub-rule (702.120a). The CR does not have a 702.120b for multiple
escalate abilities -- that detail from the user prompt appears to be an assumption. The
single rule is sufficient: the cost is paid once per additional mode chosen.

## Key Edge Cases

From card rulings (Collective Defiance, Blessed Alliance, etc.):

1. **Escalate is an additional cost, not an alternative cost.** It stacks with the base
   mana cost (and commander tax, kicker, etc.). CR 601.2f-h governs payment.
2. **"Choose one or more" -- modes are all chosen at once.** You cannot wait to perform
   one mode's actions then choose more modes. (Ruling 2016-07-13)
3. **Cannot choose the same mode more than once** (unless the card says otherwise). (Ruling 2016-07-13)
4. **Cost reducers apply to the total cost including escalate.** (Ruling 2016-07-13)
5. **Escalate cost doesn't change mana value.** Additional costs never do. (Ruling 2016-07-13)
6. **"Without paying its mana cost" still requires escalate.** If an effect lets you cast
   the spell without paying its mana cost, you still pay escalate costs for additional
   modes. (Ruling 2016-07-13)
7. **Partial target illegality.** If one target becomes illegal, remaining targets are
   still affected. Only if ALL targets are illegal does the spell fizzle. (Ruling 2016-07-13)
8. **Multiplayer**: No special multiplayer considerations -- escalate works the same
   regardless of number of opponents. Targets can be different opponents for different modes.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- does not exist yet
- [ ] Step 2: Rule enforcement -- nothing implemented
- [ ] Step 3: Trigger wiring -- N/A (escalate is a static ability, not a trigger)
- [ ] Step 4: Unit tests -- no tests exist
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Design Decisions

### Escalate vs Entwine

Entwine is "pay once, choose ALL modes." Escalate is "pay per additional mode beyond the
first." The key structural differences:

| Aspect | Entwine | Escalate |
|--------|---------|----------|
| Command field | `entwine_paid: bool` | `escalate_modes: u32` |
| Stack field | `was_entwined: bool` | `escalate_modes_paid: u32` |
| Cost calculation | Add entwine cost once | Add escalate cost x N times |
| Resolution | All modes if paid | modes 0..=N execute |
| ModeSelection | `max_modes: 1` + entwine overrides | `min_modes: 1, max_modes: N` (N = mode count) |

### Auto-stub approach (per batch plan)

Full interactive mode selection (choosing WHICH specific modes) is deferred to Batch 11.
The stub approach is ordered/sequential:

- `escalate_modes = 0`: only mode[0] executes (single mode, no extra cost)
- `escalate_modes = 1`: modes[0] and modes[1] execute, escalate cost paid 1x
- `escalate_modes = 2`: modes[0..=2] execute, escalate cost paid 2x
- `escalate_modes = N` must satisfy: `N < modes.len()` and `N >= 0`

This auto-sequential approach matches the Entwine stub (auto-select mode[0] when not
entwined). When Batch 11 adds full modal choice, escalate will use the same mode-selection
infrastructure.

### AbilityDefinition::Escalate

Like `AbilityDefinition::Entwine { cost: ManaCost }`, we need:
```
AbilityDefinition::Escalate { cost: ManaCost }
```
This stores the per-mode-beyond-first cost on the card definition.

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Escalate` variant after `KeywordAbility::Entwine` (around line 994).
**Pattern**: Follow `KeywordAbility::Entwine` at line 988-994.
**Doc comment**: `/// CR 702.120a: Escalate [cost] -- optional additional cost on modal spells. For each mode chosen beyond the first, the escalate cost is paid once.`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Escalate { cost: ManaCost }` variant after `AbilityDefinition::Entwine` (around line 453).
**Pattern**: Follow `AbilityDefinition::Entwine { cost: ManaCost }` at line 446-453.
**Doc comment**: `/// CR 702.120: Escalate [cost]. Additional cost paid for each mode chosen beyond the first. Cards with this ability should also include AbilityDefinition::Keyword(KeywordAbility::Escalate) for quick presence-checking. AbilityDefinition::Spell.modes must be Some(...) with min_modes: 1, max_modes: <mode_count>.`

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for:
1. `KeywordAbility::Escalate => 111u8.hash_into(hasher)` -- after line 559 (discriminant 111)
2. `AbilityDefinition::Escalate { cost } => { 40u8.hash_into(hasher); cost.hash_into(hasher); }` -- after line 3349 (discriminant 40)

**Match arms**: Grep for all `KeywordAbility` exhaustive matches and add `Escalate` arm.
Also grep for all `AbilityDefinition` matches and add `Escalate { cost }` arm.

### Step 2: Command + CastSpell Integration

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `escalate_modes: u32` field to `Command::CastSpell` after `entwine_paid: bool` (around line 238).
**Doc comment**: `/// CR 702.120a: Number of additional modes beyond the first for which the escalate cost is paid. 0 = single mode (no extra cost). N = pay escalate cost N times and execute modes 0..=N.`
**Default**: `#[serde(default)]` with value 0.

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `pub escalate_modes_paid: u32` field to `StackObject` after `was_entwined: bool` (around line 231).
**Doc comment**: `/// CR 702.120a: Number of additional modes paid for via escalate. 0 = only mode[0]. N = modes 0..=N execute. Propagated to copies per CR 707.10.`
**Default**: `#[serde(default)]`
**Hash**: Add to `hash.rs` in the `StackObject` hash impl: `self.escalate_modes_paid.hash_into(hasher)` after the `was_entwined` hash line (~L1729).

### Step 3: Casting Cost Calculation

**File**: `crates/engine/src/rules/casting.rs`

**Action 3a**: Add `escalate_modes: u32` parameter to `handle_cast_spell()` function signature (after `entwine_paid: bool` at line 76).

**Action 3b**: Add escalate cost validation and addition block after the entwine cost block (after line 1878). Pattern follows entwine exactly:

```
// CR 702.120a / 601.2f-h: Escalate -- if escalate_modes > 0, validate the spell has
// KeywordAbility::Escalate and add the escalate cost * escalate_modes to the total.
let mana_cost = if escalate_modes > 0 {
    if !chars.keywords.contains(&KeywordAbility::Escalate) {
        return Err(GameStateError::InvalidCommand(
            "spell does not have escalate (CR 702.120a)".into(),
        ));
    }
    // Validate escalate_modes doesn't exceed available modes
    // (modes.len() - 1 is the max number of extra modes).
    match get_escalate_cost(&card_id, &state.card_registry) {
        Some(escalate_cost) => {
            // CR 601.2f: Add escalate cost * N to total mana cost.
            let mut total = mana_cost.unwrap_or_default();
            total.white += escalate_cost.white * escalate_modes;
            total.blue += escalate_cost.blue * escalate_modes;
            total.black += escalate_cost.black * escalate_modes;
            total.red += escalate_cost.red * escalate_modes;
            total.green += escalate_cost.green * escalate_modes;
            total.generic += escalate_cost.generic * escalate_modes;
            total.colorless += escalate_cost.colorless * escalate_modes;
            Some(total)
        }
        None => {
            return Err(GameStateError::InvalidCommand(
                "spell has escalate keyword but no escalate cost defined".into(),
            ));
        }
    }
} else {
    mana_cost
};
```

**Action 3c**: Add `get_escalate_cost()` helper function after `get_entwine_cost()` (after line 3218). Pattern is identical to `get_entwine_cost()` but matches `AbilityDefinition::Escalate { cost }`.

**Action 3d**: Set `escalate_modes_paid` on the StackObject when pushing to the stack (in the StackObject construction block around line 2730):
```
escalate_modes_paid: escalate_modes,
```

**Action 3e**: Set `escalate_modes_paid: 0` on ALL trigger/copy StackObject construction sites (same sites that set `was_entwined: false`). There are approximately 5+ such sites in `casting.rs`. Search for `was_entwined: false` to find them all.

### Step 4: Resolution Mode Dispatch

**File**: `crates/engine/src/rules/resolution.rs`

**Action**: Modify the mode dispatch block at lines 210-229 to also handle escalate:

The current logic is:
```
if stack_obj.was_entwined { all modes } else { mode[0] only }
```

Change to:
```
if stack_obj.was_entwined {
    // CR 702.42b: all modes
    modes.modes.clone()
} else if stack_obj.escalate_modes_paid > 0 {
    // CR 702.120a: execute modes 0..=escalate_modes_paid
    let count = (stack_obj.escalate_modes_paid as usize + 1).min(modes.modes.len());
    modes.modes[..count].to_vec()
} else {
    // Auto-select first mode
    modes.modes.into_iter().take(1).collect()
}
```

### Step 5: Replay Harness

**File**: `crates/engine/src/testing/replay_harness.rs`

**Action 5a**: Add `escalate_modes` field to all existing `Command::CastSpell` constructions in `translate_player_action()`. Set to `0` for all existing action types.

**Action 5b**: Add `"cast_spell_escalate"` action handler after `"cast_spell_entwine"` (around line 1415). Pattern follows `cast_spell_entwine`:
- Read `escalate_modes` from `action["escalate_modes"].as_u64().unwrap_or(0) as u32`
- Set `escalate_modes` in the `Command::CastSpell` construction
- Set `entwine_paid: false`

### Step 6: TUI Stack View

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: No new `StackObjectKind` variant needed -- Escalate uses the existing spell kind.
No changes required here.

### Step 7: Unit Tests

**File**: `crates/engine/tests/escalate.rs` (new file)
**Tests to write**:

1. **`test_escalate_single_mode_no_extra_cost`** -- CR 702.120a: escalate_modes=0, only mode[0] executes, only base mana cost paid. Like the entwine non-paid test.

2. **`test_escalate_two_modes_one_extra_cost`** -- CR 702.120a: escalate_modes=1 on a 3-mode spell, modes[0] and modes[1] execute, escalate cost paid 1x. Verify life gain (mode 0) AND draw (mode 1) both happen.

3. **`test_escalate_all_three_modes`** -- CR 702.120a: escalate_modes=2 on a 3-mode spell, all 3 modes execute, escalate cost paid 2x. Total cost = base + 2x escalate.

4. **`test_escalate_insufficient_mana_rejected`** -- CR 601.2f: Attempting to pay escalate_modes=2 but only providing mana for base+1x escalate fails.

5. **`test_escalate_no_keyword_rejected`** -- Engine validation: escalate_modes>0 on a spell without KeywordAbility::Escalate is rejected.

6. **`test_escalate_modes_paid_on_stack`** -- Verify `escalate_modes_paid` field is correctly set on the StackObject.

7. **`test_escalate_modes_exceed_available_rejected`** -- escalate_modes=5 on a 3-mode spell (max extra = 2) should be rejected or clamped.

8. **`test_escalate_modes_execute_in_printed_order`** -- CR 702.120a: modes execute sequentially, state changes from mode 0 visible to mode 1.

**Pattern**: Follow `tests/entwine.rs` structure closely.

**Test card**: Create a synthetic "Escalate Test Spell" with 3 modes:
- Sorcery {1}{R}
- Escalate {1}
- Choose one or more --
  - Mode 0: Controller gains 3 life
  - Mode 1: Controller draws 2 cards
  - Mode 2: Target opponent loses 2 life

Key difference from entwine test card: `ModeSelection { min_modes: 1, max_modes: 3, modes: [...] }` instead of `min_modes: 1, max_modes: 1`. The `max_modes` reflects that escalate naturally allows choosing up to all modes.

### Step 8: Card Definition (later phase)

**Suggested card**: Blessed Alliance ({1}{W}, Instant, Escalate {2})
- Simple modes: gain 4 life / untap 2 creatures / opponent sacrifices attacker
- Mode 0 (gain life) is testable without combat state
- Alternative simpler card: create a synthetic test-only card with pure effect modes

**Card lookup**: use `card-definition-author` agent

### Step 9: Game Script (later phase)

**Suggested scenario**: Cast Blessed Alliance (or synthetic test card):
1. Cast with escalate_modes=0 -- only mode[0] (gain 4 life), pay {1}{W}
2. Cast with escalate_modes=1 -- modes[0]+[1] (gain 4 life + untap), pay {1}{W} + {2} = {3}{W}

**Subsystem directory**: `test-data/generated-scripts/stack/`

## Interactions to Watch

1. **Entwine + Escalate on same spell**: Should be mutually exclusive in practice (no card has both), but if `was_entwined` is true, it should take precedence and execute ALL modes regardless of `escalate_modes_paid`. The resolution dispatch handles this correctly because entwine check comes first.

2. **Commander tax**: Escalate cost stacks with commander tax. The cost pipeline order in `casting.rs` already handles this: base cost -> alt cost -> commander tax -> kicker -> entwine -> **escalate** -> convoke/improvise/delve -> payment.

3. **Cost reduction effects**: Per ruling, cost reducers apply to the total including escalate. This is automatically handled because the engine reduces the final total, not the base cost.

4. **Copies (CR 707.10)**: Copies of an escalated spell should copy the escalate_modes_paid value. The copy system in `copy.rs` copies StackObject fields -- ensure `escalate_modes_paid` is included in the copy.

5. **"Without paying mana cost" + escalate**: Per ruling (2016-07-13), escalate costs must still be paid even when the base cost is waived. This is automatically handled because escalate is an additional cost added after alt-cost determination.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| `KeywordAbility` | `Escalate` | 111 |
| `AbilityDefinition` | `Escalate { cost }` | 40 |
| `StackObjectKind` | N/A (no new variant) | -- |

# Ability Plan: Modal Choice

**Generated**: 2026-03-07
**CR**: 700.2
**Priority**: P2
**Similar abilities studied**: Entwine (702.42) in `casting.rs`, `resolution.rs`, `stack.rs`, `tests/entwine.rs`; Escalate (702.120) in `casting.rs`, `resolution.rs`, `tests/escalate.rs`

## CR Rule Text

700.2. A spell or ability is modal if it has two or more options in a bulleted list preceded by instructions for a player to choose a number of those options, such as "Choose one --." Each of those options is a mode. Modal cards printed prior to the Khans of Tarkir set didn't use bulleted lists for the modes; these cards have received errata in the Oracle card reference so the modes do appear in a bulleted list.

700.2a The controller of a modal spell or activated ability chooses the mode(s) as part of casting that spell or activating that ability. If one of the modes would be illegal (due to an inability to choose legal targets, for example), that mode can't be chosen. (See rule 601.2b.)

700.2b The controller of a modal triggered ability chooses the mode(s) as part of putting that ability on the stack. If one of the modes would be illegal (due to an inability to choose legal targets, for example), that mode can't be chosen. If no mode is chosen, the ability is removed from the stack. (See rule 603.3c.)

700.2c If a spell or ability targets one or more targets only if a particular mode is chosen for it, its controller will need to choose those targets only if they chose that mode. Otherwise, the spell or ability is treated as though it did not have those targets. (See rule 601.2c.)

700.2d If a player is allowed to choose more than one mode for a modal spell or ability, that player normally can't choose the same mode more than once. However, some modal spells include the instruction "You may choose the same mode more than once." If a particular mode is chosen multiple times, the spell is treated as if that mode appeared that many times in sequence. If that mode requires a target, the same player or object may be chosen as the target for each of those modes, or different targets may be chosen.

700.2e Some spells and abilities specify that a player other than their controller chooses a mode for it. In that case, the other player does so when the spell or ability's controller normally would do so. If there is more than one other player who could make such a choice, the spell or ability's controller decides which of those players will make the choice.

700.2f Modal spells and abilities may have different targeting requirements for each mode. Changing a spell or ability's target can't change its mode.

700.2g A copy of a modal spell or ability copies the mode(s) chosen for it. The controller of the copy can't choose a different mode. (See rule 707.10.)

700.2h Some modal spells have one or more modes with a cost listed before the effect of that mode. This indicates that the mode has an additional cost that must be paid as the spell is cast if that mode is chosen. Paying these costs follows the rules for paying additional costs in rules 601.2b and 601.2f-h.

700.2i Some modal spells have one or more pawprint symbols ({P}) rather than bullet points, as well as an instruction to choose up to a specified number of {P} "worth of modes." While casting such a spell, its controller can choose any number of modes such that the total number of pawprint symbols listed for the chosen modes is not greater than the specified number.

601.2b If the spell is modal, the player announces the mode choice (see rule 700.2). [...]

603.3c If a triggered ability is modal, its controller announces the mode choice when putting the ability on the stack. If one of the modes would be illegal (due to an inability to choose legal targets, for example), that mode can't be chosen. If no mode is chosen, the ability is removed from the stack. (See rule 700.2.)

## Key Edge Cases

- **Per-mode targeting (CR 700.2c)**: Targets are only required for chosen modes. Unchosen modes' targets are ignored. This is the hardest part -- the existing target system declares all targets up front (see Blessed Alliance's TODO comments at `defs/blessed_alliance.rs:30-31`). For Phase 1, we can validate that the chosen mode indices are valid and only execute the chosen modes' effects, but per-mode target validation is deferred.
- **No duplicate modes (CR 700.2d)**: By default, the same mode cannot be chosen twice. Exception: "You may choose the same mode more than once" overrides this.
- **Copies inherit modes (CR 700.2g, 707.10)**: A copy of a modal spell uses the same chosen modes. The copy system must propagate `modes_chosen` from original to copy.
- **Illegal modes cannot be chosen (CR 700.2a)**: If a mode has targets that cannot be met, that mode is illegal. Phase 1 defers this validation -- any mode index within range is accepted.
- **Entwine interaction (CR 702.42)**: When entwine is paid, ALL modes are chosen regardless of `modes_chosen`. Entwine overrides the mode selection. This is already implemented.
- **Escalate interaction (CR 702.120)**: Escalate allows choosing additional modes beyond the first by paying cost. The existing `escalate_modes_paid` field already works with this system but uses implicit 0..=N indexing -- it should be updated to use explicit `modes_chosen` indices.
- **Multiplayer**: No special multiplayer concerns beyond normal targeting rules.
- **Bot strategy**: Bots always choose mode 0 (first mode). This matches the current fallback behavior.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant / type changes
- [ ] Step 2: Rule enforcement (casting validation, mode resolution)
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Existing Infrastructure

The modal system already has significant partial infrastructure:

1. **`ModeSelection` struct** (`card_definition.rs:1169-1176`): Defines `min_modes`, `max_modes`, and `modes: Vec<Effect>`. Already used by Promise of Power (Entwine) and Blessed Alliance (Escalate).

2. **`AbilityDefinition::Spell.modes`** (`card_definition.rs:100`): `Option<ModeSelection>` field on Spell variant. Card definitions already populate this.

3. **Resolution dispatch** (`resolution.rs:215-237`): Currently handles three cases:
   - `was_entwined` -> all modes
   - `escalate_modes_paid > 0` -> modes 0..=N
   - Default fallback -> mode[0] only (hardcoded)
   The comment at line 229 says: "Auto-select first mode (Batch 11 will add full interactive mode selection)."

4. **`ModeSelection` in helpers.rs** (`helpers.rs:18`): Already exported for card definitions.

5. **Copy system** (`copy.rs:225-228`): Already propagates `was_entwined` and `escalate_modes_paid`.

**What is missing**: A `modes_chosen: Vec<usize>` field on `CastSpell` command and `StackObject` to carry the player's explicit mode choices from cast time to resolution time. The current system either hardcodes mode[0] or uses Entwine/Escalate-specific fields to select modes.

## Implementation Steps

### Step 1: Type Changes

**File**: `crates/engine/src/rules/command.rs`
**Action**: Add `modes_chosen: Vec<usize>` field to `Command::CastSpell` variant.
**Location**: After the `escalate_modes` field (line ~246), add:
```rust
/// CR 700.2a / 601.2b: Mode indices chosen for a modal spell.
/// Empty vec = non-modal spell or auto-select mode[0].
/// For "choose one" spells: exactly 1 index (e.g., [0], [1], [2]).
/// For "choose two" spells: exactly 2 indices (e.g., [0, 2]).
/// For "choose up to N" spells: 1..=N indices.
///
/// Validated in `handle_cast_spell`: each index must be < modes.len(),
/// no duplicates (unless allow_duplicate_modes is set on ModeSelection),
/// and count must be between min_modes and max_modes.
///
/// When `entwine_paid` is true, this field is ignored (all modes are chosen).
/// When `escalate_modes` > 0, this field specifies WHICH modes are chosen
/// (not just how many). If empty with escalate, falls back to 0..=escalate_modes.
#[serde(default)]
modes_chosen: Vec<usize>,
```
**Note**: Use `#[serde(default)]` so all existing commands with `modes_chosen` absent deserialize as empty vec (backward compatible).

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add `modes_chosen: Vec<usize>` field to `StackObject`.
**Location**: After `escalate_modes_paid` field (line ~238), add:
```rust
/// CR 700.2a / 601.2b: Mode indices chosen at cast time for a modal spell.
/// Empty for non-modal spells. Propagated to copies per CR 700.2g / 707.10.
#[serde(default)]
pub modes_chosen: Vec<usize>,
```

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `allow_duplicate_modes: bool` field to `ModeSelection` struct.
**Location**: In `ModeSelection` struct (line ~1169), add:
```rust
/// CR 700.2d: If true, the same mode may be chosen more than once.
/// Default is false (standard modal behavior).
#[serde(default)]
pub allow_duplicate_modes: bool,
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `modes_chosen` to `HashInto` impl for `StackObject`, and `allow_duplicate_modes` to `HashInto` for `ModeSelection`.
**Pattern**: Follow existing fields -- hash the vec length then each element. For the bool, hash it directly.

**No new discriminants are needed.** Modal Choice is not a `KeywordAbility` (it is a fundamental rule of the game, not a keyword ability). No new `StackObjectKind`, `AbilityDefinition`, `Effect`, or `GameEvent` variants are required.

### Step 2: Casting Validation

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add mode validation logic to `handle_cast_spell`.

After the function signature change (add `modes_chosen: Vec<usize>` parameter), add validation between the existing Entwine/Escalate cost blocks:

1. **Look up the card's `ModeSelection`** from `AbilityDefinition::Spell.modes` in the registry.
2. **If `modes_chosen` is non-empty and the spell has `modes: Some(ms)`**:
   - Validate each index is `< ms.modes.len()`.
   - If `!ms.allow_duplicate_modes`, validate no duplicate indices.
   - Validate `modes_chosen.len() >= ms.min_modes` and `modes_chosen.len() <= ms.max_modes`.
   - If `entwine_paid` is true, ignore `modes_chosen` (entwine overrides).
3. **If `modes_chosen` is empty and the spell has modes**:
   - Auto-select mode[0] (backward compatible with existing behavior).
4. **Store `modes_chosen` on the `StackObject`** at construction time (alongside existing `was_entwined`, `escalate_modes_paid` fields).

**Location in casting.rs**: After the Escalate cost validation block (~line 1938), before the final StackObject construction. The StackObject construction sites (there are ~6 of them for different stack object types like triggers, copies, etc.) all need `modes_chosen: vec![]` for non-spell objects.

**CR reference**: CR 700.2a, CR 601.2b.

**Interaction with Escalate**: When `escalate_modes > 0` and `modes_chosen` is non-empty, use `modes_chosen` to determine which modes to execute (not just 0..=N). When `escalate_modes > 0` and `modes_chosen` is empty, fall back to 0..=escalate_modes for backward compatibility.

### Step 3: Resolution Dispatch

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Replace the hardcoded mode dispatch logic at lines 215-237.

Current logic:
```rust
if stack_obj.was_entwined {
    modes.modes.clone()
} else if stack_obj.escalate_modes_paid > 0 {
    modes.modes[..count].to_vec()
} else {
    modes.modes.into_iter().take(1).collect()
}
```

New logic:
```rust
if stack_obj.was_entwined {
    // CR 702.42b: all modes in printed order
    modes.modes.clone()
} else if !stack_obj.modes_chosen.is_empty() {
    // CR 700.2: execute chosen modes in index order
    stack_obj.modes_chosen.iter()
        .filter_map(|&idx| modes.modes.get(idx).cloned())
        .collect()
} else if stack_obj.escalate_modes_paid > 0 {
    // Backward compat: escalate without explicit modes_chosen
    let count = (stack_obj.escalate_modes_paid as usize + 1)
        .min(modes.modes.len());
    modes.modes[..count].to_vec()
} else {
    // Auto-select first mode (bot/test default)
    modes.modes.into_iter().take(1).collect()
}
```

**CR reference**: CR 700.2, CR 702.42b.

### Step 4: Copy System Update

**File**: `crates/engine/src/rules/copy.rs`
**Action**: Propagate `modes_chosen` from original to copy.
**Location**: In the `copy_spell_on_stack` function, alongside existing `was_entwined` and `escalate_modes_paid` propagation (~line 225), add:
```rust
// CR 700.2g: copies copy the mode(s) chosen for the original.
modes_chosen: original.modes_chosen.clone(),
```

Also add `modes_chosen: vec![]` to the cascade free-cast and any other StackObject construction sites in `copy.rs`.

### Step 5: Replay Harness Update

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `cast_spell_modal` action type and pass `modes_chosen` through to `translate_player_action`.

1. Add `modes_chosen` parameter to `translate_player_action` signature.
2. Parse a `"modes"` field from the action JSON as `Vec<usize>`.
3. For `cast_spell` and `cast_spell_modal` action types, pass `modes_chosen` to the `Command::CastSpell` construction.
4. For all other action types, pass `modes_chosen: vec![]`.

Also update all existing `Command::CastSpell` construction sites in the harness to include `modes_chosen: vec![]`.

### Step 6: View Model / TUI Update

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: No changes needed -- modal choice does not add any new `StackObjectKind` or `KeywordAbility` variants.

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: No changes needed -- no new `StackObjectKind` variants.

### Step 7: Trigger Wiring

No trigger wiring is needed. Modal choice is part of the casting/resolution process, not a triggered or static ability. The mode selection happens at cast time (CR 601.2b / 700.2a) and is consumed at resolution time.

Modal triggered abilities (CR 700.2b / 603.3c) are a future extension. The current engine does not have any cards with modal triggered abilities in the card pool.

### Step 8: Unit Tests

**File**: `crates/engine/tests/modal.rs` (new file)
**Tests to write**:

1. **`test_modal_choose_one_mode_zero`** -- Cast a "choose one" modal spell selecting mode[0]. Verify only mode[0]'s effect executes.
   - CR 700.2a: controller chooses mode at cast time.
   - Use a synthetic 3-mode spell (GainLife / DrawCards / DealDamage).

2. **`test_modal_choose_one_mode_one`** -- Same spell, select mode[1]. Verify only mode[1]'s effect executes.
   - Confirms that non-zero mode indices work correctly.

3. **`test_modal_choose_one_mode_two`** -- Select mode[2]. Verify mode[2]'s effect executes.

4. **`test_modal_choose_two_modes`** -- Cast a "choose two" (min=2, max=2) spell selecting modes [0, 2]. Verify both effects execute in index order.
   - CR 700.2: modes execute in the order they appear on the card (index order), not selection order.

5. **`test_modal_default_auto_selects_mode_zero`** -- Cast a modal spell with `modes_chosen: vec![]`. Verify mode[0] is auto-selected (backward compat).
   - Ensures existing tests and bot behavior are unchanged.

6. **`test_modal_invalid_index_rejected`** -- Attempt to choose mode index >= modes.len(). Verify `GameStateError::InvalidCommand`.
   - CR 700.2a: illegal modes can't be chosen.

7. **`test_modal_duplicate_index_rejected`** -- Attempt to choose [0, 0] on a standard modal spell. Verify rejection.
   - CR 700.2d: same mode can't be chosen twice (default).

8. **`test_modal_too_few_modes_rejected`** -- Choose 0 modes on a "choose one" (min=1) spell. Verify rejection.

9. **`test_modal_too_many_modes_rejected`** -- Choose 3 modes on a "choose two" (max=2) spell. Verify rejection.

10. **`test_modal_entwine_overrides_modes_chosen`** -- Cast an entwined modal spell with `modes_chosen: [0]`. Verify all modes execute (entwine takes precedence).
    - CR 702.42b: entwine means all modes.

11. **`test_modal_copy_inherits_modes`** -- Cast a modal spell with storm, choosing mode[1]. Verify the storm copy also executes mode[1].
    - CR 700.2g: copies copy the mode(s) chosen.

12. **`test_modal_escalate_with_explicit_modes`** -- Cast an escalate spell with `modes_chosen: [0, 2]` and `escalate_modes: 1`. Verify modes 0 and 2 (not 0 and 1) execute.
    - Tests that explicit mode selection works with escalate.

**Pattern**: Follow `tests/entwine.rs` structure -- synthetic card definitions, GameStateBuilder setup, CastSpell with mode fields, pass_all to resolve, assert effects.

### Step 9: Card Definition (later phase)

**Suggested card**: Abzan Charm ({W}{B}{G} Instant, "Choose one")
- Mode 0: Exile target creature with power 3 or greater.
- Mode 1: You draw two cards and you lose 2 life.
- Mode 2: Distribute two +1/+1 counters among one or two target creatures.

Abzan Charm is a good choice because:
- Simple "choose one" with 3 modes
- Mode 1 (draw + lose life) is fully expressible in the DSL
- Mode 0 (exile creature with power filter) needs a `TargetFilter` power check
- Mode 2 (distribute counters) is harder -- defer to a simpler approximation

**Alternative simpler card**: Dimir Charm ({U}{B} Instant, "Choose one")
- Mode 0: Counter target sorcery spell (needs spell-type target filter)
- Mode 1: Destroy target creature with power 2 or less
- Mode 2: Look at top 3 cards, put 1 back, rest to graveyard

For the simplest initial test, the **synthetic test spell** in the unit tests (GainLife / DrawCards / DealDamage) is sufficient. A real card definition can follow.

### Step 10: Game Script (later phase)

**Suggested scenario**: Player casts a modal spell choosing mode[1] instead of mode[0].
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Description**: P1 casts a "choose one" modal instant, selecting the second mode (draw cards). Verify that only the draw effect resolves.

## Discriminant Tracking

No new discriminants are needed for Modal Choice. Carrying forward from Batch 10:

| Type | Next Available |
|------|---------------|
| KeywordAbility | 131 |
| AbilityDefinition | 51 |
| StackObjectKind | 50 |
| Effect | 43 (after Amass=42, AttachFortification=42 -- verify actual) |
| GameEvent | 101 (after FortificationAttached, Amassed -- verify actual) |

## StackObject Construction Sites

Every place that constructs a `StackObject` needs `modes_chosen: vec![]` added. Grep for these:

**File**: `crates/engine/src/rules/casting.rs`
- Main spell construction (~line 2850)
- Trigger construction (multiple sites ~lines 2997, 3059, 3122, 3179, 3238)
- Suspend free-cast (~line 3518)

**File**: `crates/engine/src/rules/copy.rs`
- Spell copy construction (~line 222) -- use `original.modes_chosen.clone()`
- Cascade free-cast (~line 428) -- use `vec![]`

**File**: `crates/engine/src/rules/resolution.rs`
- Suspend cast trigger (~line 3518 in casting.rs, referenced from resolution)

All sites must be updated or compilation will fail (StackObject has no Default impl).

## Interactions to Watch

- **Entwine override**: `was_entwined` must take precedence over `modes_chosen` in resolution dispatch. Already covered by the if-chain ordering.
- **Escalate compatibility**: Escalate's `escalate_modes_paid` was previously used to determine mode count (0..=N). With explicit `modes_chosen`, escalate should prefer `modes_chosen` when present but fall back to the old behavior when empty (backward compat with existing scripts/tests).
- **Copy system (CR 700.2g)**: `modes_chosen` must propagate to copies. Storm, Cascade copies, Replicate copies, and Casualty copies all need this.
- **Per-mode targeting (CR 700.2c)**: Deferred to a future enhancement. Currently all targets are declared up front regardless of mode. This is a known limitation documented in `blessed_alliance.rs:30-31`.
- **Modal triggered abilities (CR 700.2b)**: Not implemented in this phase. No cards in the current pool use modal triggers.

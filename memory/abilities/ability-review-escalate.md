# Ability Review: Escalate

**Date**: 2026-03-06
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.120
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 995-1003)
- `crates/engine/src/cards/card_definition.rs` (lines 454-462)
- `crates/engine/src/state/hash.rs` (lines 560-561, 1732-1733, 3354-3358)
- `crates/engine/src/rules/command.rs` (lines 239-246)
- `crates/engine/src/state/stack.rs` (lines 232-238)
- `crates/engine/src/rules/casting.rs` (lines 77, 1883-1913, 2766, 3268-3287)
- `crates/engine/src/rules/resolution.rs` (lines 210-231)
- `crates/engine/src/rules/engine.rs` (lines 102-131)
- `crates/engine/src/rules/copy.rs` (lines 226-228, 431-432)
- `crates/engine/src/testing/replay_harness.rs` (lines 275-277, 1447-1477)
- `crates/engine/src/testing/script_schema.rs` (lines 321-326)
- `crates/engine/tests/escalate.rs` (all 863 lines, 8 tests)

## Verdict: needs-fix

One MEDIUM finding: no cast-time validation that the spell is actually modal when
`escalate_modes > 0`. A spell with `KeywordAbility::Escalate` but no `modes` in its
`AbilityDefinition::Spell` would accept the escalate cost, charge the player extra mana,
then silently execute only the spell's main effect at resolution (falling through to the
non-modal path). While no real card would be defined this way, the engine should reject
invalid commands defensively. All other aspects of the implementation are correct and
well-tested.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `casting.rs:1886` | **No validation that spell has modes when escalate requested.** The escalate block checks for keyword and cost but not for `modes`. **Fix:** After the keyword check, verify the spell's `AbilityDefinition::Spell` has `modes: Some(...)` and that `escalate_modes < modes.len()`. |
| 2 | LOW | `casting.rs:1886` | **No cast-time rejection for escalate_modes exceeding available modes.** Player can request `escalate_modes=5` on a 3-mode spell, overpay mana, and resolution silently clamps to 3 modes. **Fix:** Add validation `escalate_modes < modes.len()` at cast time (subsumes finding 1). |
| 3 | LOW | `tests/escalate.rs` | **No test for escalate on non-modal escalate spell.** A hypothetical misconfigured card def with Escalate keyword but no modes would silently accept extra mana. **Fix:** Add a test with a card that has `KeywordAbility::Escalate` but `modes: None`, verify `escalate_modes > 0` is rejected. |

### Finding Details

#### Finding 1: No validation that spell has modes when escalate requested

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:1886`
**CR Rule**: 702.120a -- "Escalate is a static ability of modal spells"
**Issue**: The escalate cost block at line 1886 validates two things: (1) the spell has
`KeywordAbility::Escalate`, and (2) the card definition has `AbilityDefinition::Escalate { cost }`.
However, it does not validate that the spell actually has modes defined in
`AbilityDefinition::Spell { modes: Some(...) }`. CR 702.120a explicitly states escalate is
"a static ability of modal spells." If a card were misconfigured with the Escalate keyword
but no modes, the engine would charge the player extra mana for escalate but at resolution
(line 220) the spell would fall through to the non-modal `else if let Some(effect) = spell_effect`
branch, executing only the base effect. The player would lose mana with no benefit and no error.
This violates the engine's defensive validation principle -- invalid commands should be rejected,
not silently degraded.
**Fix**: In the escalate cost block (after the keyword check at line 1887), look up the card
definition's `AbilityDefinition::Spell` to verify it has `modes: Some(ref m)` and that
`escalate_modes < m.modes.len()`. Return `GameStateError::InvalidCommand` if the spell is not
modal or if escalate_modes exceeds `modes.len() - 1`. This also subsumes Finding 2.

#### Finding 2: No cast-time rejection for escalate_modes exceeding available modes

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:1886`
**CR Rule**: 702.120a -- "For each mode you choose beyond the first"
**Issue**: A player can request `escalate_modes=5` on a 3-mode spell (where max extra is 2).
The engine charges for 5 extra modes but resolution clamps to 3 modes via `.min()` at line 224.
The player overpays by 3x the escalate cost with no error. While Test 7 documents this as
intentional clamping, it is arguably a command validation gap -- the engine should reject
impossible commands rather than silently accepting overpayment. However, since the plan explicitly
chose clamping, this is LOW.
**Fix**: Add `if escalate_modes as usize >= modes.modes.len()` check during casting validation,
returning an error. Adjust Test 7 to expect rejection instead of clamping.

#### Finding 3: No test for escalate on non-modal spell

**Severity**: LOW
**File**: `crates/engine/tests/escalate.rs`
**CR Rule**: 702.120a -- "Escalate is a static ability of modal spells"
**Issue**: Test 5 covers a spell without the Escalate keyword. There is no test for a spell
that has the Escalate keyword but is not modal (no modes defined). This edge case would
exercise the validation gap in Finding 1.
**Fix**: Add a test with a card definition that includes `KeywordAbility::Escalate` and
`AbilityDefinition::Escalate { cost }` but has `modes: None` in its `AbilityDefinition::Spell`.
Verify that `escalate_modes > 0` is rejected with an appropriate error.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.120a (core) | Yes | Yes | test_escalate_single_mode_no_extra_cost, test_escalate_two_modes_one_extra_cost, test_escalate_all_three_modes |
| 702.120a (additional cost) | Yes | Yes | Cost added to base; test_escalate_insufficient_mana_rejected |
| 702.120a (per-mode-beyond-first) | Yes | Yes | N*cost calculation verified in tests 2, 3 |
| 702.120a (modal spells only) | Partial | No | Keyword validated but modal-ness not validated (Finding 1) |
| 601.2f-h (payment rules) | Yes | Yes | Mana deduction tested; insufficient mana rejected |
| CR 707.2/707.10 (copy) | Yes | No | copy.rs line 228 propagates `escalate_modes_paid`; no explicit copy+escalate test |
| Printed order execution | Yes | Yes | test_escalate_modes_execute_in_printed_order |
| Stack field propagation | Yes | Yes | test_escalate_modes_paid_on_stack |
| Keyword validation | Yes | Yes | test_escalate_no_keyword_rejected |
| Clamping overflow | Yes | Yes | test_escalate_modes_exceed_available_clamped |
| Hash coverage | Yes | N/A | Discriminants 111 (keyword), 40 (ability def); StackObject hash at line 1733 |
| Replay harness | Yes | N/A | `cast_spell_escalate` action type at line 1451 |
| Script schema | Yes | N/A | `escalate_modes` field with serde default |

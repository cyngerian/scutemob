# Ability Review: Blitz

**Date**: 2026-03-02
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.152
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 95-112, 835-860)
- `crates/engine/src/cards/card_definition.rs` (lines 325-351)
- `crates/engine/src/state/stack.rs` (lines 145-167, 690-728)
- `crates/engine/src/state/stubs.rs` (lines 55-73)
- `crates/engine/src/state/hash.rs` (Blitz discriminants 96/29/32 + was_blitzed field)
- `crates/engine/src/rules/casting.rs` (lines 70-80, 716-854, 925-941, 1490-1502, 1920-1940)
- `crates/engine/src/rules/resolution.rs` (lines 280-325, 1077-1182, 1646-1650, 3191-3200)
- `crates/engine/src/rules/turn_actions.rs` (lines 255-299)
- `crates/engine/src/rules/abilities.rs` (BlitzSacrifice flush arm at line 3648; was_blitzed: false at 10 sites)
- `crates/engine/src/rules/copy.rs` (was_blitzed: false at 2 sites)
- `crates/engine/src/testing/replay_harness.rs` (cast_spell_blitz action at line 826)
- `tools/tui/src/play/panels/stack_view.rs` (BlitzSacrificeTrigger arm at line 125)
- `tools/replay-viewer/src/view_model.rs` (BlitzSacrificeTrigger arm at line 518)
- `crates/engine/tests/blitz.rs` (9 tests, 1040 lines)

## Verdict: needs-fix

The implementation is structurally correct and closely follows the Dash pattern. CR
702.152a is faithfully implemented: blitz casting, haste grant, SelfDies draw trigger
injection at ETB, delayed sacrifice trigger at end step, replacement effect handling on
sacrifice, and counter handling are all present. Hash discriminants are correctly assigned
and all StackObject construction sites include `was_blitzed: false`. The replay harness,
TUI, and replay viewer are updated.

One MEDIUM finding: Test 4 (`test_blitz_draw_card_on_death`) claims to verify the draw-on-death
trigger fires for any cause of death, but it bypasses the engine event path entirely via
manual `move_object_to_zone`, and the test itself acknowledges that the draw trigger never
fires. This means the "draw on any death, not just end-step sacrifice" case from the Mezzio
Mugger ruling (2022-04-29) is not actually tested. Two LOW findings exist for minor gaps.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `crates/engine/tests/blitz.rs:426` | **Test 4 does not verify draw-on-death for non-sacrifice paths.** The test manually moves the creature via `move_object_to_zone` which bypasses `check_triggers`, so the SelfDies draw trigger never fires. The test's own comments acknowledge this. **Fix:** rewrite the test to kill the creature through the engine path -- e.g., use a DealDamage effect or set toughness to 0 and let SBAs handle it, then verify hand size increases by 1. |
| 2 | LOW | `crates/engine/src/rules/resolution.rs:1131` | **BlitzSacrificeTrigger always emits CreatureDied regardless of permanent type.** CR 702.152a says "permanent," not "creature." If a non-creature permanent had blitz, the sacrifice would emit CreatureDied incorrectly. Pre-existing pattern (evoke does the same at line 795). **Fix:** capture `is_creature` from the source object (as the plan proposed) and conditionally emit CreatureDied vs PermanentDestroyed. Low priority since all blitz cards are creatures. |
| 3 | LOW | `crates/engine/tests/blitz.rs:426` | **Missing CR citation for Mezzio Mugger ruling in test doc comment.** Test 4 should cite the specific ruling: "The triggered ability that lets its controller draw a card triggers when it dies for any reason, not just when you sacrifice it during the end step" (Mezzio Mugger, 2022-04-29). **Fix:** add ruling citation to the test's doc comment. |

### Finding Details

#### Finding 1: Test 4 does not verify draw-on-death for non-sacrifice paths

**Severity**: MEDIUM
**File**: `crates/engine/tests/blitz.rs:426`
**CR Rule**: 702.152a -- "As long as this permanent's blitz cost was paid, it has haste and 'When this permanent is put into a graveyard from the battlefield, draw a card.'"
**Ruling**: Mezzio Mugger (2022-04-29): "The triggered ability that lets its controller draw a card triggers when it dies for any reason, not just when you sacrifice it during the end step."
**Issue**: The test is named `test_blitz_draw_card_on_death` and claims to verify that the blitz draw trigger fires when the creature dies for any reason (not just end-step sacrifice). However, the test manually calls `state.move_object_to_zone(bf_id, ZoneId::Graveyard(p1))` which bypasses the engine's event system entirely -- no `CreatureDied` event is generated, no `check_triggers` is called, and the SelfDies draw trigger never fires. The test's own comments (lines 516-545) acknowledge this and defer to Test 5. The test asserts only that the creature is in the graveyard, which is trivially true after a manual zone move. The Mezzio Mugger ruling's "dies for any reason" edge case is not tested.
**Fix**: Rewrite the test to kill the creature through a legitimate engine path. Options:
1. Use `process_command` to cast a damage spell (e.g., a "Shock" card definition that deals 2 damage to a creature) targeting the blitzed creature, then drain the stack and verify hand size increased by 1.
2. Set the creature's toughness to 0 (via a counter or effect) and call `process_command(PassPriority)` to trigger SBAs, which will move the creature to the graveyard via the engine path.
3. Cast a "Destroy target creature" effect to go through the destruction path.
Any of these will fire `CreatureDied` through the engine, which calls `check_triggers`, which finds the SelfDies draw trigger, pushes it to the stack, and upon resolution draws a card.

#### Finding 2: BlitzSacrificeTrigger always emits CreatureDied

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:1131`
**CR Rule**: 702.152a -- "sacrifice the permanent this spell becomes"
**Issue**: The BlitzSacrificeTrigger resolution arm (lines 1087-1182) always emits `GameEvent::CreatureDied` when the permanent is sacrificed, even in the `Redirect` fallback `_ =>` arm (line 1131) and the `Proceed` branch (line 1146). It does not check whether the permanent is actually a creature. If a non-creature permanent somehow had blitz (e.g., via a future card or unusual interaction), the event would be incorrect. This is the same pattern used by evoke sacrifice (lines 751-845), so it is pre-existing and consistent. The plan (step 3c) proposed capturing `is_creature` and conditionally emitting `CreatureDied` vs `PermanentDestroyed`, but the implementation simplified this.
**Fix**: Capture `is_creature` from the source object's card_types in the `source_info` tuple. In the Proceed branch and Redirect default arm, conditionally emit `GameEvent::CreatureDied` (if creature) or `GameEvent::PermanentDestroyed` (if not). Low priority -- all current blitz cards are creatures.

#### Finding 3: Missing Mezzio Mugger ruling citation

**Severity**: LOW
**File**: `crates/engine/tests/blitz.rs:426`
**CR Rule**: 702.152a
**Issue**: Test 4's doc comment says "When a blitzed creature is put into a graveyard from the battlefield (for any reason), its controller draws a card" but does not cite the specific Mezzio Mugger ruling (2022-04-29) that establishes this edge case. Per `memory/conventions.md`, tests should cite their rule sources.
**Fix**: Add to the doc comment: `/// Ruling: Mezzio Mugger (2022-04-29): "The triggered ability that lets its controller draw a card triggers when it dies for any reason, not just when you sacrifice it during the end step."`

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.152a: Alternative cost | Yes | Yes | test_blitz_basic_cast_with_blitz_cost |
| 702.152a: Haste grant | Yes | Yes | test_blitz_basic_cast_with_blitz_cost (checks Haste keyword) |
| 702.152a: Draw-on-death trigger | Yes | Partial | test_blitz_draw_on_sacrifice_at_end_step covers sacrifice path; non-sacrifice death path not tested (Finding 1) |
| 702.152a: Sacrifice at end step | Yes | Yes | test_blitz_sacrifice_at_end_step |
| 702.152a: Combined sacrifice+draw | Yes | Yes | test_blitz_draw_on_sacrifice_at_end_step |
| 702.152a: Follows 601.2b/601.2f-h | Yes | Yes | Casting validation in casting.rs; cost applied as alternative cost |
| 702.152b: Multiple instances | Partial | No | `find_map` naturally uses first instance; no test for multiple blitz costs |
| CR 118.9a: Mutual exclusion | Yes | Yes | test_blitz_alternative_cost_exclusivity; full exclusion checks in casting.rs |
| CR 118.9d: Commander tax | Yes | Yes | test_blitz_commander_tax_applies |
| CR 400.7: Zone change identity | Yes | Yes | test_blitz_creature_left_battlefield_before_end_step |
| Ruling: Dies for any reason | Yes (code) | No (test) | SelfDies trigger is always present; no test exercises non-sacrifice death (Finding 1) |
| Ruling: Sacrifice only if on BF | Yes | Yes | test_blitz_creature_left_battlefield_before_end_step |
| Ruling: Copies no benefits | Yes | Yes | All copy sites set was_blitzed: false |
| Ruling: No forced attack | Yes | N/A | No forced-attack mechanism for blitz; haste only removes summoning sickness |
| Normal cast negative test | Yes | Yes | test_blitz_normal_cast_no_sacrifice_no_draw |
| Non-blitz card rejected | Yes | Yes | test_blitz_card_without_blitz_rejected |
| Replacement effects on sacrifice | Yes | No | Code handles via check_zone_change_replacement; no test with Rest in Peace or similar |
| Stifle/counter interaction | Yes | No | Counter arm correctly handles BlitzSacrificeTrigger; no dedicated test |

## Implementation Quality Notes

**Correct patterns observed:**
- Hash discriminants: KW 96, AbDef 29, SOK 32 -- all correctly assigned and unique.
- All 12 StackObject construction sites include `was_blitzed: false` (10 in abilities.rs, 2 in copy.rs).
- Main CastSpell site sets `was_blitzed: casting_with_blitz` (casting.rs:1501).
- Suspend free-cast site sets `was_blitzed: false` (resolution.rs:1649).
- `cast_alt_cost` chain correctly includes Blitz after Dash (resolution.rs:290-291).
- Counter handling arm includes BlitzSacrificeTrigger (resolution.rs:3193).
- TUI and replay viewer match arms are present and follow the existing pattern.
- Replay harness `cast_spell_blitz` action is complete and correct.
- CR citations in doc comments are thorough and accurate.
- Replacement effect handling on sacrifice follows the established evoke pattern with full Redirect/Proceed/ChoiceRequired handling.

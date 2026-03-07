# Ability Review: Collect Evidence

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.59
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::CollectEvidence, Condition::EvidenceWasCollected)
- `crates/engine/src/state/stack.rs` (evidence_collected field)
- `crates/engine/src/state/game_object.rs` (evidence_collected field)
- `crates/engine/src/state/hash.rs` (hash coverage for all new fields)
- `crates/engine/src/state/mod.rs` (move_object_to_zone reset)
- `crates/engine/src/state/builder.rs` (builder reset)
- `crates/engine/src/rules/command.rs` (collect_evidence_cards on CastSpell)
- `crates/engine/src/rules/casting.rs` (validation + payment)
- `crates/engine/src/rules/resolution.rs` (propagation to EffectContext and GameObject)
- `crates/engine/src/rules/engine.rs` (call site pass-through)
- `crates/engine/src/rules/copy.rs` (copy propagation)
- `crates/engine/src/rules/abilities.rs` (evidence_collected: false in all StackObject literals)
- `crates/engine/src/effects/mod.rs` (EffectContext field, evaluate_condition)
- `crates/engine/src/testing/replay_harness.rs` (cast_spell_collect_evidence action)
- `crates/engine/src/testing/script_schema.rs` (collect_evidence_cards schema field)
- `crates/engine/tests/collect_evidence.rs` (11 tests)
- `crates/engine/tests/script_replay.rs` (pass-through to translate_player_action)

## Verdict: clean

The implementation is correct, thorough, and well-tested. All three CR 701.59 subrules are
faithfully implemented. Validation covers uniqueness, zone ownership, MV threshold, mandatory
vs optional, and spell-without-ability rejection. Hash coverage is complete for all new fields.
Propagation through StackObject -> EffectContext -> GameObject is correct. The copy system
correctly propagates evidence_collected per CR 707.2. No HIGH or MEDIUM findings. Two LOW
issues identified: a misleading doc comment on StackObject, and a missing overlap check
between collect_evidence_cards and delve/escape exile cards.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `state/stack.rs:208` | **Misleading doc comment.** Says copies must have evidence_collected=false, but copy.rs:243 correctly copies it from the original per CR 707.2. **Fix:** Change "Must always be false for copies" to "Propagated to copies per CR 707.2." |
| 2 | LOW | `rules/casting.rs:2402-2487` | **No overlap validation with delve/escape exile cards.** If a card somehow had both collect evidence and delve/escape, the same graveyard card could be exiled for both costs. Extremely unlikely in real cards but architecturally unsound. **Fix:** Add overlap check: `if collect_evidence_cards.iter().any(|id| delve_cards.contains(id) || escape_exile_cards.contains(id)) { return Err }`. Defer to future batch -- no real cards trigger this. |
| 3 | LOW | `tests/collect_evidence.rs` | **Missing test for Condition::EvidenceWasCollected on permanent.** Tests verify the condition at spell resolution (instant), but no test verifies that `evidence_collected` propagates to `GameObject` for permanents with ETB triggers that check the condition. **Fix:** Add a test with a creature that has CollectEvidence + ETB trigger checking EvidenceWasCollected. |

### Finding Details

#### Finding 1: Misleading doc comment on StackObject.evidence_collected

**Severity**: LOW
**File**: `crates/engine/src/state/stack.rs:208`
**CR Rule**: 707.2 -- "When copying an object, the copy acquires... choices made when casting or activating it (mode, targets, the value of X, whether it was kicked, how it will affect multiple targets, and so on)."
**Issue**: The doc comment says "Must always be false for copies (`is_copy: true`) -- copies are not cast." However, `copy.rs:243` correctly sets `evidence_collected: original.evidence_collected` per CR 707.2 (copies copy casting choices). The code is correct; the comment is wrong.
**Fix**: Change the doc comment on line 208 from "Must always be false for copies (`is_copy: true`) -- copies are not cast." to "Propagated to copies per CR 707.2 (copies copy choices made during casting)."

#### Finding 2: No overlap validation between collect_evidence_cards and delve/escape

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:2402-2487`
**CR Rule**: 701.59a -- "exile any number of cards from your graveyard"; 702.66a (Delve) -- "exile any number of cards from your graveyard"; 702.138a (Escape) -- "exile a specified number of other cards from your graveyard"
**Issue**: If a hypothetical card had both Collect Evidence and Delve (or Escape), the same card could be listed in both `collect_evidence_cards` and `delve_cards` (or `escape_exile_cards`), leading to a double-exile attempt (second move would fail with an error since the card is already gone). No real cards have this combination, so this is defensive.
**Fix**: Add an overlap check after the evidence validation block. Defer to a future batch.

#### Finding 3: Missing permanent propagation test

**Severity**: LOW
**File**: `crates/engine/tests/collect_evidence.rs`
**CR Rule**: 701.59c -- linked abilities
**Issue**: The test suite verifies `evidence_collected` propagation for instants (spell resolution -> EffectContext). It does not verify propagation to `GameObject` for permanent spells. The code at `resolution.rs:436` correctly does `obj.evidence_collected = stack_obj.evidence_collected`, but there is no test exercising this path. The existing test `test_collect_evidence_basic_exile_from_graveyard` uses an instant, which is sufficient for the Condition check via EffectContext, but a creature test would verify the GameObject propagation path.
**Fix**: Add a test with a creature card that has `CollectEvidence { threshold: 4, mandatory: false }` and an ETB trigger checking `Condition::EvidenceWasCollected`. Cast it with evidence, verify the permanent's `evidence_collected == true`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.59a (exile from graveyard, total MV >= N) | Yes | Yes | test_collect_evidence_basic_exile_from_graveyard, test_collect_evidence_over_threshold_allowed |
| 701.59a (under-threshold rejected) | Yes | Yes | test_collect_evidence_under_threshold_rejected, test_collect_evidence_insufficient_single_card_rejected |
| 701.59a (own graveyard only) | Yes | Yes | test_collect_evidence_card_not_in_graveyard_rejected, test_collect_evidence_opponents_graveyard_rejected |
| 701.59a (no mana reduction) | Yes | Yes | test_collect_evidence_mana_not_reduced |
| 701.59b (cannot choose if insufficient) | Yes | Yes | Enforced by MV threshold check; tested by under-threshold tests |
| 701.59c (linked ability condition) | Yes | Yes | test_collect_evidence_not_collected_optional (false branch), test_collect_evidence_basic_exile_from_graveyard (true branch) |
| Mandatory evidence | Yes | Yes | test_collect_evidence_mandatory_without_cards_rejected |
| Duplicate ObjectId | Yes | Yes | test_collect_evidence_duplicate_card_rejected |
| Spell without ability | Yes | Yes | test_collect_evidence_spell_without_ability_rejected |
| Copy propagation (CR 707.2) | Yes | No | copy.rs:243 propagates correctly; no dedicated test |
| Permanent propagation | Yes | No | resolution.rs:436 propagates correctly; no dedicated test (Finding 3) |
| Hash coverage | Yes | N/A | hash.rs: StackObject, GameObject, Condition, AbilityDefinition all covered |

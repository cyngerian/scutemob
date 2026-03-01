# Ability Review: Renown

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.112
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 683-692)
- `crates/engine/src/state/hash.rs` (lines 488-492, 658-659, 1128-1130, 1449-1457)
- `crates/engine/src/state/game_object.rs` (lines 437-444)
- `crates/engine/src/state/stubs.rs` (lines 287-299)
- `crates/engine/src/state/stack.rs` (lines 498-513)
- `crates/engine/src/state/builder.rs` (line 846)
- `crates/engine/src/state/mod.rs` (lines 299, 390)
- `crates/engine/src/rules/abilities.rs` (lines 2137-2225, 2664-2673)
- `crates/engine/src/rules/resolution.rs` (lines 1842-1885, 1997)
- `crates/engine/src/effects/mod.rs` (line 2474)
- `crates/engine/tests/renown.rs` (full file, 779 lines)
- `tools/tui/src/play/panels/stack_view.rs` (lines 92-94)
- `tools/replay-viewer/src/view_model.rs` (lines 485-487)
- `crates/engine/src/rules/copy.rs` (confirmed no `is_renowned` reference)

## Verdict: clean

The Renown implementation is correct and comprehensive. All three CR subrules (702.112a, 702.112b, 702.112c) are faithfully implemented. The intervening-if clause (CR 603.4) is correctly checked at both trigger time and resolution time. The `is_renowned` designation is properly placed on `GameObject` (not `Characteristics`), ensuring it is not part of copiable values per CR 702.112b. Zone-change reset is handled by `move_object_to_zone` (CR 400.7). All initialization sites set `is_renowned: false`. Hash coverage is complete for `KeywordAbility::Renown`, `GameObject.is_renowned`, `PendingTrigger.is_renown_trigger`/`renown_n`, and `StackObjectKind::RenownTrigger`. Tests cover all seven planned scenarios with correct CR citations. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `abilities.rs:2149-2174` | **Granted-keyword fallback gap (pre-existing pattern).** If a CardDefinition exists in registry but has no Renown keywords, continuous-effect-granted Renown in `characteristics.keywords` is ignored. **Fix:** Systemic issue shared with Ingest; defer to when continuous keyword-granting effects are implemented. |
| 2 | LOW | `renown.rs:466-468` | **Test double-Renown via card def only adds 1 keyword to ObjectSpec.** The test correctly gives the CardDefinition two Renown(1) entries but only calls `.with_keyword(Renown(1))` once on the ObjectSpec. This works because the trigger dispatch reads from card_registry first. If the registry-lookup path were broken, the fallback would only find 1 instance, masking a bug in the primary path. Not a correctness issue, but a fragile test design. **Fix:** Consider adding a comment explaining why only one keyword on the ObjectSpec is intentional, or add both keywords to the spec as well. |

### Finding Details

#### Finding 1: Granted-keyword fallback gap (pre-existing pattern)

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:2149-2174`
**CR Rule**: 702.112a -- "When this creature deals combat damage to a player, if it isn't renowned, put N +1/+1 counters on it and it becomes renowned."
**Issue**: The trigger dispatch first checks the `CardDefinition` from the card registry. If a `card_id` is present and the definition is found, `.map()` returns `Some(vec![])` even when no Renown keywords exist in the definition. The `.unwrap_or_else()` fallback (which checks `characteristics.keywords`) only runs when the Option is `None`. This means a creature that has Renown granted by a continuous effect (in `characteristics.keywords`) but whose original card definition lacks Renown will not trigger. This is a pre-existing systemic pattern shared with Ingest and is not Renown-specific. Continuous keyword-granting effects are not yet implemented in the engine.
**Fix**: Defer to when continuous keyword-granting effects are implemented. At that point, the pattern should be updated to check both the CardDefinition AND `characteristics.keywords`, or only check the post-layer-system characteristics.

#### Finding 2: Test double-Renown ObjectSpec has only one keyword

**Severity**: LOW
**File**: `crates/engine/tests/renown.rs:466-468` (CardDefinition) vs line 482 (ObjectSpec)
**CR Rule**: 702.112c -- "If a creature has multiple instances of renown, each triggers separately."
**Issue**: The `test_702_112c_renown_multiple_instances_first_resolves` test registers a CardDefinition with two `Renown(1)` entries but only calls `.with_keyword(KeywordAbility::Renown(1))` once on the ObjectSpec. The test passes because the trigger dispatch correctly reads from the card_registry (primary path). However, if the primary path were broken and fell through to the fallback (checking `characteristics.keywords`), the test would find only 1 instance and report only 1 trigger, potentially masking a regression in the registry-lookup logic.
**Fix**: Add a comment on the `.with_keyword()` line explaining that only one is added to the ObjectSpec because the trigger dispatch reads from the CardDefinition's abilities list. Alternatively, call `.with_keyword()` twice so the fallback would also produce 2 triggers.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.112a (basic trigger + counters) | Yes | Yes | `test_702_112a_renown_basic_counters_and_renowned` |
| 702.112a (Renown N > 1) | Yes | Yes | `test_702_112a_renown_n2_places_two_counters` |
| 702.112a (intervening-if at trigger time) | Yes | Yes | `test_702_112a_renown_no_trigger_when_already_renowned` |
| 702.112a (combat damage to player only) | Yes | Implicit | Inherent in `CombatDamageTarget::Player` guard at line 2036 |
| 702.112a (multiplayer) | Yes | Yes | `test_702_112a_renown_multiplayer_specific_player` |
| 702.112b (not copiable) | Yes | Structural | `is_renowned` on `GameObject`, not `Characteristics`; copy.rs does not reference it |
| 702.112b (resets on zone change) | Yes | Yes | `test_702_112b_renown_resets_on_zone_change` |
| 702.112b (designation persists through ability removal) | Yes | Structural | `is_renowned` is a flag on `GameObject`, independent of keywords |
| 702.112c (multiple instances trigger separately) | Yes | Yes | `test_702_112c_renown_multiple_instances_first_resolves` |
| 702.112c + 603.4 (intervening-if at resolution) | Yes | Yes | Resolution checks `!obj.is_renowned` at line 1860; tested by multiple-instances test |
| Ruling: creature leaves before resolution | Yes | Yes | `test_702_112_renown_creature_leaves_before_resolution` |
| Ruling: redirected damage to controller still triggers | Yes | Structural | Trigger fires on `CombatDamageTarget::Player(_)` for ANY player, not just opponents |
| Ruling: prevented damage (amount 0) does not trigger | Yes | Structural | `assignment.amount == 0 => continue` at line 2033 |

## Previous Findings (re-review only)

N/A -- first review.

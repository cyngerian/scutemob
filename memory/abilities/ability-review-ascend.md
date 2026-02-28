# Ability Review: Ascend

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.131
**Files reviewed**:
- `crates/engine/src/rules/sba.rs` (lines 77-139, 145-193)
- `crates/engine/src/rules/resolution.rs` (lines 192-228)
- `crates/engine/src/rules/events.rs` (lines 756-762)
- `crates/engine/src/state/hash.rs` (line 433 disc 62 for KeywordAbility, line 615 for PlayerState, lines 1974-1977 disc 80 for GameEvent)
- `crates/engine/src/state/player.rs` (lines 119-123)
- `crates/engine/src/state/builder.rs` (line 260)
- `crates/engine/src/state/types.rs` (lines 504-510)
- `crates/engine/tests/ascend.rs` (all 560 lines, 7 tests)
- `tools/replay-viewer/src/view_model.rs` (lines 62-74, 265-283)

## Verdict: needs-fix

The core Ascend enforcement logic is correct and well-structured. The SBA check
runs first in the SBA pass (ensuring the 10th-permanent edge case works), uses
layer-computed keywords (so Humility suppresses Ascend correctly), and the
instant/sorcery check runs at resolution time after spell effects execute. All
7 tests are well-designed with CR citations. However, the view_model.rs is
missing the `has_citys_blessing` field (the plan called for it in Step 5), and
there is one MEDIUM correctness concern about CR 702.131d (continuous effects
reapplication after gaining the blessing within the SBA pass).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `tools/replay-viewer/src/view_model.rs:62-74` | **PlayerView missing has_citys_blessing.** The field is absent from both the struct and the construction site. **Fix:** Add `pub has_citys_blessing: bool` to `PlayerView` and populate it from `player.has_citys_blessing` at line 282. |
| 2 | MEDIUM | `crates/engine/src/rules/sba.rs:130-134` | **CR 702.131d: continuous effects not reapplied after blessing.** After setting `has_citys_blessing = true`, the SBA pass continues with the same stale `chars_map`. CR 702.131d says "continuous effects are reapplied before the game checks to see if the game state ... have matched any trigger conditions." The current implementation does not rebuild `chars_map` after granting the blessing. **Fix:** See Finding 2 details. |
| 3 | LOW | `crates/engine/src/rules/resolution.rs:192-228` | **Ascend check runs for permanent spells too.** The code does not gate the check on `!is_permanent`. Per CR 702.131a, Ascend as a spell ability applies only to instants/sorceries; on permanents it is a static ability (702.131b) checked via the SBA path. The current code is functionally harmless (the permanent is still on the stack so it doesn't count toward 10), but it is semantically imprecise. **Fix:** Wrap the block in `if !is_permanent { ... }` for clarity. |
| 4 | LOW | `crates/engine/tests/ascend.rs` | **Missing test for 10th-permanent + legend-rule edge case.** The plan identifies a key edge case: "If your tenth permanent enters the battlefield and then a permanent leaves the battlefield immediately afterwards (most likely due to the Legend Rule)..." No test validates this timing guarantee. **Fix:** Add a test where the 10th permanent is a legendary that triggers the legend rule, verifying the blessing is still granted. |
| 5 | LOW | `crates/engine/tests/ascend.rs` | **Missing test for Ascend + Humility interaction.** The plan identifies Humility suppressing Ascend via layer-computed keywords as a key interaction. No test covers this. **Fix:** Add a test where a Humility-like continuous effect removes all abilities, verifying Ascend no longer grants the blessing. |

### Finding Details

#### Finding 1: PlayerView missing has_citys_blessing

**Severity**: MEDIUM
**File**: `tools/replay-viewer/src/view_model.rs:62-74` (struct) and `tools/replay-viewer/src/view_model.rs:265-283` (construction)
**CR Rule**: 702.131c -- "The city's blessing is a designation that has no rules meaning other than to act as a marker that other rules and effects can identify."
**Issue**: The `PlayerView` struct in the replay viewer does not include the `has_citys_blessing` field. The plan (Step 5) explicitly called for adding this field. Without it, the replay viewer cannot display whether a player has the city's blessing, which is important for debugging and visualization of ascend-related game states. The `PlayerState` has the field (player.rs:123), it is hashed (hash.rs:615), and the builder defaults it (builder.rs:260), but the view model omits it.
**Fix**: Add `pub has_citys_blessing: bool` to the `PlayerView` struct (after `has_conceded`) and add `has_citys_blessing: player.has_citys_blessing,` to the `PlayerView` construction at line 282.

#### Finding 2: CR 702.131d continuous effects not reapplied after blessing

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/sba.rs:130-134`
**CR Rule**: 702.131d -- "After a player gets the city's blessing, continuous effects are reapplied before the game checks to see if the game state or preceding events have matched any trigger conditions."
**Issue**: The `check_ascend` function sets `has_citys_blessing = true` and emits the event, but the `apply_sbas_once` function continues with the same `chars_map` snapshot. CR 702.131d requires that continuous effects are reapplied after a player gains the blessing. If any continuous effect is conditioned on having the city's blessing (e.g., "As long as you have the city's blessing, creatures you control get +1/+1"), that effect should be active for the remaining SBA checks in the same pass.

The plan notes: "This is automatically handled by the SBA loop: after each SBA pass, triggers are checked, and on the next pass the chars_map is recomputed." This is partially true -- continuous effects ARE reapplied on the next SBA pass. However, CR 702.131d says reapplication happens BEFORE trigger checking within the same pass, not on the next pass. In the current architecture, triggers are checked after `apply_sbas_once` returns (in `check_and_apply_sbas` at sba.rs:68), which IS after the ascend check. So triggers see the updated blessing state. The remaining concern is: other SBAs in the same pass (creature toughness, etc.) use the pre-blessing `chars_map` that doesn't reflect blessing-dependent continuous effects.

**Practical impact**: This is only relevant when a card has BOTH ascend AND a blessing-dependent continuous effect that changes characteristics checked by other SBAs (e.g., "+1/+1 to your creatures as long as you have the city's blessing" saving a creature from 704.5f zero-toughness death in the same SBA pass). This is a narrow edge case but technically CR-incorrect.

**Fix**: After `check_ascend` returns events, if any `CitysBlessingGained` events were produced, rebuild `chars_map` before continuing with the remaining SBA checks. Alternatively, document this as a known limitation with a `// TODO: CR 702.131d` comment noting the narrow edge case.

#### Finding 3: Ascend check runs for permanent spells

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:192-228`
**CR Rule**: 702.131a -- "Ascend on an instant or sorcery spell represents a spell ability."
**Issue**: The resolution-time ascend check runs for all spell types (including creature, artifact, enchantment, planeswalker, and battle permanent spells). CR 702.131a specifies that ascend as a spell ability applies only to instants and sorceries. For permanents, CR 702.131b says ascend is a static ability checked "any time" (via the SBA path). The current code is functionally harmless because the permanent is still on the stack (not on the battlefield) when this check runs, so it doesn't inflate the permanent count. However, it unnecessarily performs the check and could theoretically emit a redundant `CitysBlessingGained` event if the player already had 10 other permanents.
**Fix**: Add `if !is_permanent` guard around the ascend check block for semantic correctness: `if !is_permanent { /* ascend check */ }`.

#### Finding 4: Missing test for 10th-permanent + legend-rule edge case

**Severity**: LOW
**File**: `crates/engine/tests/ascend.rs`
**CR Rule**: 702.131b -- the ruling: "If your tenth permanent enters the battlefield and then a permanent leaves the battlefield immediately afterwards (most likely due to the Legend Rule or due to being a creature with 0 toughness), you get the city's blessing before it leaves the battlefield."
**Issue**: The plan identifies this as a key edge case and the implementation specifically addresses it by running `check_ascend` first in `apply_sbas_once`. However, no test validates this behavior. The implementation could be broken by someone reordering the SBA checks without noticing.
**Fix**: Add a test where P1 controls 9 permanents (including an Ascend creature), then a 10th permanent enters that is a duplicate Legendary (triggering the legend rule). Assert that the blessing is still granted because ascend is checked first.

#### Finding 5: Missing test for Ascend + Humility interaction

**Severity**: LOW
**File**: `crates/engine/tests/ascend.rs`
**CR Rule**: 702.131b -- uses layer-computed keywords. Humility (or similar) would remove Ascend via Layer 6.
**Issue**: The plan identifies "Ascend + Humility (Layer 6)" as a key interaction to watch. The implementation correctly uses `chars_map` (layer-computed) for the SBA check. However, no test validates that a continuous effect removing all abilities prevents Ascend from granting the blessing. Without such a test, a future refactor could switch to raw keywords without being caught.
**Fix**: Add a test where a Humility-like continuous effect is active. A creature with Ascend has Ascend removed by the effect. Despite 10+ permanents, the blessing is NOT granted because no permanent has Ascend in its layer-computed keywords.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.131a (instant/sorcery spell ability) | Yes | Yes | test_ascend_instant_sorcery_on_resolution |
| 702.131b (permanent static ability) | Yes | Yes | test_ascend_basic_permanent_grants_blessing |
| 702.131b (below threshold) | Yes | Yes | test_ascend_below_threshold_no_blessing |
| 702.131b (requires ascend source) | Yes | Yes | test_ascend_no_ascend_source_no_blessing |
| 702.131c (permanent once gained) | Yes | Yes | test_ascend_blessing_permanent_once_gained |
| 702.131c (multiple players) | Yes | Yes | test_ascend_multiple_players_independent |
| 702.131d (reapply continuous effects) | Partial | No | chars_map not rebuilt after blessing in same SBA pass |
| Ruling (tokens count) | Yes | Yes | test_ascend_tokens_count_as_permanents |
| Ruling (10th + legend rule) | Yes (ordering) | No | SBA ordering is correct but no test covers it |
| Ruling (Humility suppresses) | Yes (chars_map) | No | Layer-computed keywords used but no test |
| Hash coverage (KeywordAbility) | Yes | -- | disc 62, hash.rs:433 |
| Hash coverage (PlayerState) | Yes | -- | hash.rs:615 |
| Hash coverage (GameEvent) | Yes | -- | disc 80, hash.rs:1974-1977 |
| Builder default | Yes | -- | builder.rs:260, defaults to false |
| View model | No | -- | Missing from PlayerView (Finding 1) |

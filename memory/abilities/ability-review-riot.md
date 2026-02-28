# Ability Review: Riot

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.136
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 456-465)
- `crates/engine/src/state/hash.rs` (lines 417-418)
- `tools/replay-viewer/src/view_model.rs` (line 638)
- `crates/engine/src/rules/resolution.rs` (lines 256-307)
- `crates/engine/tests/riot.rs` (full file, 623 lines)

## Verdict: clean

The implementation correctly handles the core CR 702.136a/b rules. Riot is implemented
as an inline replacement effect at the primary ETB site in `resolution.rs`, following
the established Escape counter-placement pattern. The OrdSet deduplication issue for
multiple Riot instances is handled correctly by counting from the card definition rather
than the keywords set. All five tests cover positive cases (counter placement, P/T
calculation, keyword presence, multiple instances) and one negative case. No HIGH or
MEDIUM findings. Three LOW findings documented below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `resolution.rs:301-305` | **CounterAdded event pushed outside `if let Some` guard.** Event is emitted unconditionally even if the object lookup fails. **Fix:** Move `events.push(GameEvent::CounterAdded { ... })` inside the `if let Some(obj)` block, consistent with `effects/mod.rs:820`. |
| 2 | LOW | `resolution.rs:256-307` | **Missing Riot processing at Unearth ETB site.** The Unearth ETB path (line 779) does not include Riot counter placement. A creature with both Riot and Unearth being unearthed would not get its Riot choice. **Fix:** No immediate fix needed -- consistent with Escape-with-counters also being absent from the Unearth path. Track as a known gap. When the engine adds a centralized ETB-replacement pipeline, both Escape-with-counters and Riot should be handled there. |
| 3 | LOW | `tests/riot.rs` | **No test for the haste-choice path.** All tests validate the default counter choice. There is no test verifying that the haste path works (even if manually triggered). The TODO at line 264 documents this is deferred until `Command::ChooseRiot` is added. **Fix:** Add a comment in the test file's module-level doc noting the haste path is untested pending interactive choice infrastructure. |

### Finding Details

#### Finding 1: CounterAdded event pushed outside `if let Some` guard

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:301-305`
**CR Rule**: 702.136a -- "You may have this permanent enter with an additional +1/+1 counter on it."
**Architecture Invariant**: #4 -- "All state changes are Events." Events should correspond to actual state changes.
**Issue**: The `events.push(GameEvent::CounterAdded { ... })` call at line 301 is outside the `if let Some(obj) = state.objects.get_mut(&new_id)` block at line 291. If the object were to not exist at `new_id` (which cannot happen in practice at this point in the resolution flow), a `CounterAdded` event would be emitted without the counter actually being placed. Other counter-adding code (`effects/mod.rs:820`, `replacement.rs:1019`) places the event emission inside the guard.
**Fix**: Move the `events.push(GameEvent::CounterAdded { ... })` call inside the `if let Some(obj)` block, after the counter is updated. This makes the event emission conditional on the state change succeeding, consistent with the pattern in `effects/mod.rs`.

#### Finding 2: Missing Riot processing at Unearth ETB site

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:779` (Unearth ETB path, lines 776-838)
**CR Rule**: 702.136a -- Riot applies whenever a permanent with Riot enters the battlefield, regardless of how.
**Issue**: The Unearth ETB path (`StackObjectKind::UnearthAbility`) at line 766 moves the creature to the battlefield but does not include Riot processing. If a creature card has both Riot and Unearth (extremely unlikely in current card pool, would require external effects), it would enter without the Riot choice. This is consistent with how Escape-with-counters (line 238) is also only handled at the primary spell-resolution ETB site, not the Unearth path.
**Fix**: No immediate action required. This is a known limitation consistent with the existing ETB infrastructure. When the engine adds a centralized ETB-replacement pipeline (tracking replacement effects that modify how a permanent enters), both Escape-with-counters and Riot should be handled uniformly at all ETB sites.

#### Finding 3: No test coverage for haste-choice path

**Severity**: LOW
**File**: `crates/engine/tests/riot.rs`
**CR Rule**: 702.136a -- "If you don't, it gains haste."
**Issue**: The test suite validates only the +1/+1 counter default choice. The haste path is entirely untested because the engine always defaults to counter and has no mechanism for the player to choose haste. The plan explicitly defers this to a future `Command::ChooseRiot`. However, the test file's module doc at line 10 says "Riot creature enters with +1/+1 counter (default deterministic choice, CR 702.136a)" without explicitly noting the haste path is untested.
**Fix**: Add a note to the module-level doc comment in `riot.rs` stating: "// The haste choice path (CR 702.136a alternative) is untested pending Command::ChooseRiot infrastructure." This makes the gap explicit to future developers.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.136a (counter choice) | Yes | Yes | `test_riot_enters_with_counter`, `test_riot_creature_has_correct_stats` |
| 702.136a (haste choice) | No (deferred) | No | TODO at resolution.rs:264; needs `Command::ChooseRiot` |
| 702.136b (multiple instances) | Yes | Yes | `test_riot_multiple_instances_each_add_counter` |
| 614.1c (replacement effect) | Yes (inline, no stack) | Yes (implicit) | Riot is processed inline, not as a trigger |
| 614.12a (choice before ETB) | Partial | Partial | Counter is placed before `PermanentEnteredBattlefield` event (line 415), which is correct timing |

## Additional Observations

1. **Enum variant** (`types.rs:456-465`): Well-documented with CR citations. Doc comment mentions CR 702.136, CR 614.1c, and CR 702.136b. Placement after Unearth is reasonable.

2. **Hash discriminant** (`hash.rs:417-418`): Discriminant 56 follows Dethrone=55 sequentially. Match arm is exhaustive. No issues.

3. **Replay viewer** (`view_model.rs:638`): `Riot => "Riot".to_string()` -- correct and consistent with other keyword formatting.

4. **OrdSet deduplication handling**: The plan correctly identified that `OrdSet<KeywordAbility>` deduplicates unit variants, and the implementation correctly counts Riot instances from the card definition's `abilities` list instead of the runtime `keywords` set. This is the right approach for MVP.

5. **Counter placement timing**: The counter is placed at lines 291-299, before `PermanentEnteredBattlefield` is emitted at line 415. This means the creature enters with the counter already on it, which is correct per CR 702.136a ("enter with an additional +1/+1 counter") and CR 614.12a ("choice is made before the permanent enters the battlefield").

6. **Test quality**: All 5 tests have CR citations, use `GameStateBuilder`, and follow the naming convention. The negative test (`test_riot_no_counters_on_non_riot_creature`) verifies both counter absence and correct base P/T.

7. **No `.unwrap()` in engine library code**: The Riot block uses `unwrap_or(0)` and `if let Some`, no bare unwraps. Tests appropriately use `.unwrap()`.

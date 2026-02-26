# Ability Review: DeclareAttackers / DeclareBlockers Harness Actions

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 508.1 (DeclareAttackers), 509.1 (DeclareBlockers)
**Files reviewed**:
- `crates/engine/src/testing/script_schema.rs` (lines 208-369)
- `crates/engine/src/testing/replay_harness.rs` (lines 1-784, full file)
- `crates/engine/tests/script_replay.rs` (lines 135-174, PlayerAction destructure)
- `tools/replay-viewer/src/replay.rs` (lines 196-268, PlayerAction match arm)
- `crates/engine/tests/combat_harness.rs` (full file, 513 lines)
- `crates/engine/src/rules/command.rs` (full file, Command enum verification)
- `crates/engine/src/rules/combat.rs` (lines 1-400, engine enforcement)

## Verdict: needs-fix

The implementation is largely correct and well-structured. The schema additions use
`#[serde(default)]` correctly for backward compatibility, both call sites are properly
updated, and the 6 tests provide good baseline coverage. However, there are two MEDIUM
issues: (1) the default attack target in multiplayer uses `HashMap::values()` which has
non-deterministic iteration order, producing an arbitrary opponent rather than a
predictable one, and (2) `find_on_battlefield_by_name` returns the first match when
multiple creatures share a name, with no warning or disambiguation, which will silently
produce wrong results for duplicate-name scenarios. There are no HIGH findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `replay_harness.rs:291` | **Non-deterministic default attack target in multiplayer.** `HashMap::values()` iteration order is unspecified. **Fix:** sort or use a deterministic selection. |
| 2 | MEDIUM | `replay_harness.rs:702-709` | **`find_on_battlefield_by_name` silently picks first match for duplicate names.** No disambiguation for two creatures with the same card name. **Fix:** document limitation or add index-based disambiguation. |
| 3 | LOW | `replay_harness.rs:193` | **`#[allow(clippy::too_many_arguments)]` is a code smell.** 9 parameters suggests a struct parameter would be cleaner. **Fix:** defer; refactor to a params struct when adding the next action type. |
| 4 | LOW | `combat_harness.rs` | **No test for multiple creatures with the same name.** Edge case not covered. **Fix:** add a test with two Llanowar Elves. |
| 5 | LOW | `combat_harness.rs` | **No multiplayer (3+ player) test.** All tests are 2-player. **Fix:** add a 3-player test verifying the correct target is selected. |
| 6 | LOW | `combat_harness.rs` | **No test for `target_planeswalker` branch.** The planeswalker attack path is untested. **Fix:** add test when planeswalker card definitions exist. |

### Finding Details

#### Finding 1: Non-deterministic default attack target in multiplayer

**Severity**: MEDIUM
**File**: `crates/engine/src/testing/replay_harness.rs:291`
**CR Rule**: 508.1b -- "the active player announces which player, planeswalker, or battle each of the chosen creatures is attacking"
**Issue**: When no `target_player` or `target_planeswalker` is specified in an
`AttackerDeclaration`, the harness defaults to:
```rust
let target_pid = players.values().find(|&&pid| pid != player).copied()?;
```
The `players` parameter is a `std::collections::HashMap<String, PlayerId>`. `HashMap::values()`
has no guaranteed iteration order. In a 2-player game this is fine (only one opponent exists),
but in a 3+ player game, the "first non-active player" found depends on the HashMap's
internal hash seed, which can vary between runs, Rust versions, and platforms. This means
the same script could attack different players on different machines, violating the
replay determinism invariant (Architecture Invariant #3: deterministic testing).

The comment says "for 2-player scripts" which is accurate for current use, but the code
is reachable in multiplayer contexts without any compile-time or runtime guard.

**Fix**: Either (a) sort the `players` entries and pick the first non-active player
alphabetically (e.g., `let mut opponents: Vec<_> = players.iter().filter(...).collect();
opponents.sort_by_key(|(name, _)| name.clone()); opponents.first()`), or (b) return
`None` (reject the command) when there are 3+ players and no `target_player` is specified,
forcing scripts to be explicit. Option (b) is safer; option (a) is more convenient for
simple scripts.

#### Finding 2: `find_on_battlefield_by_name` silently picks first match for duplicate names

**Severity**: MEDIUM
**File**: `crates/engine/src/testing/replay_harness.rs:702-709`
**CR Rule**: 508.1 / 509.1 -- attackers and blockers are identified by ObjectId, not name
**Issue**: `find_on_battlefield_by_name` and `find_on_battlefield` both use
`state.objects.iter().find_map(...)` which returns the first match by `ObjectId` ordering
(since `state.objects` is an `OrdMap<ObjectId, GameObject>`). When two creatures share a
card name (e.g., two Llanowar Elves on the battlefield), the harness will always resolve
to the one with the lower `ObjectId`. There is no way for a script to specify "the second
Llanowar Elves."

This is a known limitation of all name-based resolution in the harness (the same issue
exists for `find_in_hand`, `find_on_battlefield`, etc.), and is tracked as a LOW in the
milestone reviews (`partial name matching`). However, for combat specifically, this is
more impactful because:
- Declaring two attackers with the same name will silently declare the same creature
  twice (the second `find_on_battlefield` call finds the same object again).
- Declaring a blocker for an attacker when there are two creatures with the attacker's
  name will always block the one with the lower ObjectId, even if the other is the one
  actually attacking.

**Fix**: For the immediate term, add a doc comment warning on `find_on_battlefield_by_name`
and `find_on_battlefield` stating the duplicate-name limitation. For the longer term,
consider adding an optional `index` field to `AttackerDeclaration` and `BlockerDeclaration`
(e.g., `"index": 1` to pick the second creature with that name), or use controller +
name to narrow results. The `find_on_battlefield` function already filters by controller,
which partially mitigates this for the attacker side. The `find_on_battlefield_by_name`
function (used for blocker-targets and planeswalker attacks) has no controller filter by
design, but could accept an optional controller hint.

#### Finding 3: `#[allow(clippy::too_many_arguments)]` is a code smell

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:193`
**Issue**: `translate_player_action` now has 9 parameters. The `#[allow]` suppression is
technically correct but signals that the function signature is accumulating fields that
belong to a struct. Each new harness action type that needs data will require adding yet
another parameter.
**Fix**: Deferred. When the next action type is added, refactor to accept a
`TranslateActionParams` struct or pass the full `ScriptAction` variant and destructure
inside. No action needed now.

#### Finding 4: No test for duplicate card names

**Severity**: LOW
**File**: `crates/engine/tests/combat_harness.rs`
**Issue**: No test exercises the scenario where two creatures with the same card name are
on the battlefield. This is an edge case called out in the review request.
**Fix**: Add a test `test_harness_declare_attackers_duplicate_names` that places two
Llanowar Elves under p1's control, declares both as attackers, and verifies that both
are included in the command (or documents the limitation that this currently produces
a broken command). This test would expose Finding 2.

#### Finding 5: No multiplayer test

**Severity**: LOW
**File**: `crates/engine/tests/combat_harness.rs`
**Issue**: All 6 tests use a 2-player setup. No test exercises 3+ players. This means
the default-target fallback (Finding 1) and the multiplayer blocker ordering (CR 802.4)
are untested.
**Fix**: Add a 3-player test that explicitly specifies `target_player` for each attacker.
This is a low-priority addition since the engine's combat handling is already tested for
multiplayer in `tests/combat.rs` and `tests/six_player.rs`. The harness layer is the
untested part.

#### Finding 6: No test for planeswalker attack target

**Severity**: LOW
**File**: `crates/engine/tests/combat_harness.rs`
**Issue**: The `target_planeswalker` branch in the `declare_attackers` arm (line 286-288)
calls `find_on_battlefield_by_name` to resolve a planeswalker target, but no test exercises
this path.
**Fix**: Defer until a Planeswalker card definition is available. Add a test at that time
that declares an attacker targeting a planeswalker.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 508.1 (declare attackers) | Yes (engine: combat.rs) | Yes | test_harness_declare_attackers_basic |
| 508.1a (untapped, haste) | Yes (engine: combat.rs:106-117) | No (harness test) | Tested in tests/combat.rs |
| 508.1b (announce targets) | Yes (harness: target resolution) | Yes | test_harness_declare_attackers_default_target |
| 508.1f (tap attackers) | Yes (engine: combat.rs) | Yes | test_harness_declare_attackers_basic checks tapped |
| 508.1k (becomes attacking) | Yes (engine: combat.rs) | Yes | Implicit in full combat test |
| 508.1m (triggers) | Yes (engine: combat.rs) | No (harness test) | Tested in tests/combat.rs |
| 508.8 (skip if no attackers) | Yes (engine) | Yes | test_harness_declare_attackers_empty |
| 509.1 (declare blockers) | Yes (engine: combat.rs) | Yes | test_harness_declare_blockers_basic |
| 509.1a (untapped, choose attacker) | Yes (engine: combat.rs:354-377) | No (harness test) | Tested in tests/combat.rs |
| 509.1g (becomes blocking) | Yes (engine) | Yes | Implicit in full combat test |
| 509.1h (blocked/unblocked) | Yes (engine) | Yes | test_harness_declare_blockers_empty (unblocked) |
| 509.1i (triggers) | Yes (engine) | No (harness test) | Tested in tests/combat.rs |
| Schema backward compat | Yes (`#[serde(default)]`) | Yes | Existing scripts parse unchanged |
| Harness-to-engine translation | Yes | Yes | All 6 tests exercise translate_player_action |

## Schema/Code Quality Check

| Check | Status | Notes |
|-------|--------|-------|
| `#[serde(default)]` on new Vec fields | Pass | Lines 211, 216 -- existing scripts parse without changes |
| `AttackerDeclaration` / `BlockerDeclaration` structs | Pass | Clean, well-documented, derive block correct |
| CR citations in doc comments | Pass | Lines 277-278, 302-304 cite CR 508.1, 509.1 |
| Import updates | Pass | Lines 19, 21-23 correctly import new types |
| `script_replay.rs` destructure updated | Pass | Lines 141-142 add `attackers`, `blockers` |
| `replay.rs` destructure updated | Pass | Lines 202-204 add `attackers`, `blockers` |
| Both call sites pass new params | Pass | Lines 152-153 (script_replay.rs), lines 213-214 (replay.rs) |
| `find_on_battlefield_by_name` helper | Pass | Lines 698-710, correct zone filter, no controller filter (by design) |
| No `.unwrap()` in engine library code | Pass | All `?` and `Option` returns |
| Hash coverage for new fields | N/A | No new fields on `GameState`, `PlayerState`, or `GameObject` |
| Match arms exhaustive | Pass | New arms in existing match, `_` arm still present |

# Ability Review: Proliferate

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.34
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (lines 508-525)
- `crates/engine/src/effects/mod.rs` (lines 1502-1572)
- `crates/engine/src/rules/events.rs` (lines 826-841)
- `crates/engine/src/state/game_object.rs` (lines 195-198)
- `crates/engine/src/state/hash.rs` (lines 1088-1089, 2029-2038, 2597-2600)
- `crates/engine/src/rules/abilities.rs` (lines 1542-1561)
- `crates/engine/tests/proliferate.rs` (all 895 lines, 12 tests)

## Verdict: needs-fix

The core proliferate logic is correct: battlefield-only permanents with counters each
receive one additional counter of each kind, players with poison receive one more poison
counter, and the `Proliferated` event is always emitted. Hash coverage, trigger wiring,
and the multiplayer test are all solid. Two MEDIUM issues: (1) the auto-select-all
simplification can cause game-losing states that a real player would never choose
(e.g., self-poisoning from 9 to 10), and (2) the plan's loyalty-counter-on-planeswalker
test was dropped without replacement, leaving a common proliferate interaction untested.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `effects/mod.rs:1509` | **Auto-select-all adds harmful counters to self.** Proliferating player cannot opt out of adding -1/-1 or poison to their own permanents/self. **Fix:** Add a doc comment acknowledging this limitation explicitly, and add a test that demonstrates the self-harming behavior so it is visible when interactive choice is implemented. |
| 2 | MEDIUM | `tests/proliferate.rs` | **Missing planeswalker loyalty counter test.** Plan test #3 (loyalty counter on planeswalker) was replaced with charge counter on artifact, leaving loyalty counter proliferation untested. **Fix:** Add `test_proliferate_loyalty_counter_on_planeswalker` per plan. |
| 3 | LOW | `effects/mod.rs:1545` | **Only poison counters tracked for players.** `CounterType::Experience` exists in the type system but `PlayerState` has no field for it; proliferate cannot increment experience counters. **Fix:** Add a doc comment noting this limitation; no code change needed until experience counter tracking is added to `PlayerState`. |
| 4 | LOW | `tests/proliferate.rs:46` | **`run_proliferate` helper uses `ObjectId(0)` as source.** While Proliferate does not reference the source object, `ObjectId(0)` may not exist in the game state. If future code adds source-based validation, this will silently break. **Fix:** Use a real ObjectId from the test state, or document why a dummy is acceptable. |

### Finding Details

#### Finding 1: Auto-select-all adds harmful counters to self

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:1509`
**CR Rule**: 701.34a -- "To proliferate means to choose any number of permanents and/or players that have a counter"
**Issue**: CR 701.34a says "choose any number" -- the player may choose zero or any subset.
The auto-select-all simplification means a player with 9 poison counters who casts a
proliferate spell will auto-increment their own poison to 10 and lose via SBA (CR 704.5c).
In real MTG, no player would choose to add poison to themselves or -1/-1 counters to their
own creatures. This simplification can cause game-losing states that are impossible under
correct CR play. The plan documents this as "deferred to M10+" but the current behavior
produces illegal game outcomes from the CR perspective.
**Fix**: (1) Add a prominent doc comment on the `Effect::Proliferate` handler stating that
auto-select-all can produce suboptimal/harmful results and that interactive selection in M10+
must allow subset choice. (2) Add a test `test_proliferate_auto_select_adds_own_poison` that
explicitly demonstrates this behavior and is marked with a `// TODO(M10): interactive choice
will fix this` comment, so the limitation is visible and will be caught during M10 implementation.

#### Finding 2: Missing planeswalker loyalty counter test

**Severity**: MEDIUM
**File**: `crates/engine/tests/proliferate.rs`
**CR Rule**: 701.34a -- proliferate adds one counter of each kind; 122.1e -- loyalty counters on planeswalkers
**Issue**: The plan explicitly listed test #3 as "proliferate loyalty counter on planeswalker"
(CR 701.34a + CR 122.1e). This was replaced with a charge counter test (which is useful but
already partially covered by the multiple-counter-types test). Loyalty counters are the most
common real-world proliferate interaction (e.g., Atraxa, Praetors' Voice + superfriends), and
planeswalker objects may have different setup requirements (CardType::Planeswalker, loyalty
counter initialization). The absence of this test means the loyalty counter path is untested.
**Fix**: Add `test_proliferate_loyalty_counter_on_planeswalker` that creates a planeswalker
object with `CardType::Planeswalker` and `CounterType::Loyalty` set to 3, proliferates, and
asserts loyalty is now 4. This verifies that the `CounterType::Loyalty` path through the
counter iteration works correctly.

#### Finding 3: Only poison counters tracked for players

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:1545`
**CR Rule**: 701.34a -- "players that have a counter"; 122.1 -- counters on players include poison, experience, rad, and others
**Issue**: The engine only tracks `poison_counters` on `PlayerState`. CR 122.1 recognizes
other player counter types (experience counters from Commander 2015, rad counters from CR 727,
energy counters). `CounterType::Experience` and `CounterType::Energy` exist in the type system
but have no corresponding `PlayerState` fields. If/when these are added, the proliferate
implementation must be updated to iterate them as well.
**Fix**: Add a doc comment on the player-counter section of the proliferate handler:
`// NOTE: When experience/energy/rad counters are added to PlayerState, update this loop.`

#### Finding 4: Dummy ObjectId(0) as source in test helper

**Severity**: LOW
**File**: `crates/engine/tests/proliferate.rs:52`
**CR Rule**: N/A (code quality)
**Issue**: The `run_proliferate` test helper passes `ObjectId(0)` as the source object.
Proliferate does not reference `ctx.source`, so this works today. However, if the source
is later used (e.g., for replacement effects that reference "the source of the effect"),
tests using this helper would silently produce wrong behavior without failing.
**Fix**: Either use an actual ObjectId from the game state (e.g., the proliferating player's
creature on the battlefield), or add a comment: `// ObjectId(0) is a dummy -- Proliferate
does not reference ctx.source.`

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.34a (permanents with counters) | Yes | Yes | Tests 1, 2, 3, 5, 6, 12 |
| 701.34a (players with counters) | Yes | Yes | Tests 4, 6, 10, 12 |
| 701.34a ("choose any number") | Partial | No | Auto-selects all; no subset choice (Finding 1) |
| 701.34a (add one of each kind) | Yes | Yes | Test 5 (multiple counter types) |
| 701.34a (battlefield only) | Yes | Yes | Test 9 (graveyard card ignored) |
| 701.34a (no counters = not chosen) | Yes | Yes | Tests 7, 10 |
| 701.34b (Two-Headed Giant) | N/A | N/A | Not applicable to Commander; correctly deferred |
| Ruling: event always emitted | Yes | Yes | Tests 7, 8 |
| Ruling: "whenever you proliferate" | Yes | Yes | Test 11 (trigger integration) |
| Interaction: loyalty counters | Yes (generic) | No | Finding 2 -- no planeswalker test |
| Interaction: multiplayer | Yes | Yes | Test 12 (4 players) |
| Interaction: +1/+1 and -1/-1 SBA | Yes (existing SBA) | No | Not tested in proliferate suite; relies on existing SBA coverage |

## Previous Findings (re-review only)

N/A -- this is the initial review.

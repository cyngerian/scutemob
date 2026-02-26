# Ability Review: Cycling

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.29
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Cycling)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Cycling { cost })
- `crates/engine/src/state/hash.rs` (KeywordAbility discriminant 28, AbilityDefinition discriminant 9, GameEvent discriminant 71)
- `crates/engine/src/rules/events.rs` (GameEvent::CardCycled, reveals_hidden_info)
- `crates/engine/src/rules/command.rs` (Command::CycleCard)
- `crates/engine/src/rules/engine.rs` (dispatch arm + trigger flush)
- `crates/engine/src/rules/abilities.rs` (handle_cycle_card, get_cycling_cost)
- `crates/engine/src/rules/resolution.rs` (embedded_effect resolution path -- unchanged, verified)
- `crates/engine/src/testing/replay_harness.rs` (cycle_card action)
- `crates/engine/tests/cycling.rs` (12 tests)
- `tools/replay-viewer/src/view_model.rs` (keyword display mapping)

## Verdict: clean

The implementation correctly models CR 702.29a-d for the P1 scope. The discard is properly treated as an activation cost (happens immediately before the draw ability goes on the stack), the draw is placed on the stack as a respondable `ActivatedAbility` with an embedded `DrawCards` effect, zone validation is correct (hand-only activation per CR 702.29a), the keyword is visible in all zones per CR 702.29b, no sorcery-speed restriction is imposed (correctly instant-speed), priority reset follows CR 602.2e, both `CardDiscarded` and `CardCycled` events are emitted (supporting future CR 702.29c-d trigger matching), and all hash discriminants are present. The trigger dispatch for `CardCycled`/`CardDiscarded` is explicitly deferred per plan, which is appropriate for P1 scope. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `abilities.rs:406` | **Unnecessary Arc clone.** `state.card_registry.clone()` clones the Arc to satisfy borrow checker. **Fix:** Use `Arc::clone(&state.card_registry)` for clarity per Rust conventions, or restructure to avoid the clone. |
| 2 | LOW | `abilities.rs:486` | **Street Wraith comment inaccuracy.** Doc comment says "free cycling, e.g., Street Wraith" but Street Wraith costs "Pay 2 life", not {0}. The zero-cost path is valid for cards like Edge of Autumn (with alternate cost), but the Street Wraith citation is misleading. **Fix:** Change doc comment to reference a generic zero-cost cycling pattern rather than Street Wraith specifically. |
| 3 | LOW | `tests/cycling.rs:662` | **Zero-cost test naming references Street Wraith.** Test comment says "like Street Wraith's {0}: Cycling" but Street Wraith pays 2 life, not {0}. **Fix:** Remove the Street Wraith reference or clarify this tests {0} mana cost cycling, not life-payment cycling. |
| 4 | LOW | `tests/cycling.rs` | **Missing multiplayer test.** No 4-player test verifying cycling works in a Commander-format game (e.g., cycling during an opponent's turn in a 4-player game, passing priority around all 4 players to resolve). The 2-player tests cover the core mechanics, but a 4-player test would be more consistent with the multiplayer-first invariant. **Fix:** Add a test with 4 players where a non-active player cycles, then all 4 pass priority to resolve. |
| 5 | LOW | `tests/cycling.rs` | **Missing "cycling ability countered" test.** The plan identifies "Cycling + Stifle" as a key interaction. No test verifies that when the cycling draw ability is countered on the stack, the card remains in the graveyard and no draw happens. **Fix:** Add a test that puts a counter effect on the stack above the cycling ability and verifies the card stays in graveyard with no draw. |
| 6 | LOW | `abilities.rs:486` | **`ManaCost` only; life-payment cycling unsupported.** `get_cycling_cost` returns `Option<ManaCost>`. Cards like Street Wraith ({B/P}) or Edge of Autumn (sacrifice a land) have non-mana cycling costs. This is a known limitation documented in the plan; logging here for completeness. **Fix:** No immediate fix needed. When life-payment or alternate cycling costs are needed, extend `AbilityDefinition::Cycling` to accept a `Cost` enum variant instead of just `ManaCost`. |

### Finding Details

#### Finding 1: Unnecessary Arc clone

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:406`
**Issue**: `state.card_registry.clone()` is used to avoid a borrow conflict (the function holds `&mut state` but needs to read the registry). The clone is cheap (Arc clone), but `Arc::clone(&state.card_registry)` is the idiomatic way to signal intent. This is a style-only concern.
**Fix**: Use `let registry = Arc::clone(&state.card_registry);` for clarity, or extract the card_id lookup before the mutable borrow.

#### Finding 2: Street Wraith comment inaccuracy

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:485`
**CR Rule**: 702.29a
**Issue**: The doc comment for `get_cycling_cost` says "free cycling, e.g., Street Wraith" but Street Wraith's cycling cost is "Pay 2 life" (a Phyrexian mana cost), not {0}. The `None` return path handles cards where no `AbilityDefinition::Cycling` exists in the registry, which is a different scenario.
**Fix**: Change the comment to say "free cycling (no mana cost)" without citing Street Wraith.

#### Finding 3: Zero-cost test naming

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/cycling.rs:662`
**Issue**: Comment says "like Street Wraith's {0}: Cycling" but Street Wraith pays life. Minor documentation issue.
**Fix**: Update comment to remove Street Wraith reference.

#### Finding 4: Missing multiplayer test

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/cycling.rs`
**CR Rule**: Architecture invariant #5 (multiplayer-first)
**Issue**: All 12 tests use 2-player setups. While cycling has no special multiplayer interactions, a 4-player test would validate the priority-pass-around cycle (all 4 players must pass for the cycling ability to resolve) and be consistent with the project's multiplayer-first principle.
**Fix**: Add one 4-player cycling test.

#### Finding 5: Missing counter-on-stack test

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/cycling.rs`
**CR Rule**: 702.29a -- "Cycling [cost]" means "[cost], Discard this card: Draw a card." The draw goes on the stack and can be countered.
**Issue**: The plan explicitly identifies the Stifle interaction as a key edge case. No test verifies that countering the cycling ability on the stack prevents the draw while the discard (cost) has already occurred. The resolution path (via embedded_effect on ActivatedAbility) should handle this correctly through the existing CounterSpell mechanism, but there's no test proving it.
**Fix**: Add a test that puts a counter effect above the cycling ability and verifies no draw occurs but the card remains in graveyard. This may require a card definition with a counter-ability effect.

#### Finding 6: ManaCost-only cycling cost

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:486-501`
**CR Rule**: 702.29a
**Issue**: The `AbilityDefinition::Cycling { cost: ManaCost }` variant only supports mana costs. Cards like Street Wraith (pay 2 life) and Edge of Autumn (sacrifice a land) have non-mana cycling costs. This is a known P2 limitation.
**Fix**: Deferred. When needed, change `cost` to accept the existing `Cost` enum or a new `CyclingCost` enum that supports mana, life, and sacrifice.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.29a (cycling is activated from hand) | Yes | Yes | test_cycling_card_not_in_hand_rejected, test_cycling_basic_discards_and_draws |
| 702.29a (discard is cost, draw is effect) | Yes | Yes | test_cycling_card_goes_to_graveyard_before_draw, test_cycling_draw_is_on_stack |
| 702.29a (mana cost payment) | Yes | Yes | test_cycling_insufficient_mana_rejected, test_cycling_colored_mana_cost, test_cycling_colored_mana_wrong_color_rejected |
| 702.29a (instant speed -- no timing restriction) | Yes | Yes | test_cycling_instant_speed_valid |
| 702.29a (priority required) | Yes | Yes | test_cycling_requires_priority |
| 702.29a (keyword check required) | Yes | Yes | test_cycling_card_without_cycling_rejected |
| 702.29b (keyword exists in all zones) | Yes | Yes | test_cycling_keyword_on_battlefield |
| 702.29c ("when you cycle" triggers) | Partial (event emitted) | No | CardCycled event emitted; trigger dispatch deferred per plan |
| 702.29d ("cycles or discards" single fire) | Partial (both events emitted) | No | Both CardDiscarded and CardCycled emitted; no duplicate-fire test yet |
| 702.29e (typecycling) | No -- deferred | No | Explicitly deferred per plan |
| 702.29f (typecycling is cycling) | No -- deferred | No | Follows from 702.29e deferral |
| 602.2 (priority required for activation) | Yes | Yes | test_cycling_requires_priority |
| 602.2a (reveal from hidden zone) | Implicit | No | Card moves to graveyard (public zone) as cost; reveals_hidden_info returns true for both events |
| 602.2e (priority reset after activation) | Yes | Yes | Verified in test_cycling_basic_discards_and_draws (priority holder is active player after cycling) |
| Zero-cost cycling | Yes | Yes | test_cycling_zero_cost_cycling |

## Previous Findings (re-review only)

N/A -- this is the initial review.

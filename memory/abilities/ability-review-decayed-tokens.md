# Ability Review: Decayed Tokens

**Date**: 2026-03-08
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.147 (Decayed)
**Files reviewed**:
- `crates/engine/src/cards/card_definition.rs:1625-1645` (zombie_decayed_token_spec)
- `crates/engine/src/cards/helpers.rs:16` (export)
- `crates/engine/src/cards/mod.rs:17-18` (export)
- `crates/engine/src/lib.rs:9-10` (export)
- `crates/engine/src/effects/mod.rs:2890-2980` (make_token keyword propagation)
- `crates/engine/src/rules/combat.rs:433-441,545-551` (flag set + can't-block)
- `crates/engine/src/rules/turn_actions.rs:1541-1568` (EOC sacrifice)
- `crates/engine/tests/decayed.rs` (all 12 tests)

## Verdict: clean

No new enforcement code was written. The implementation adds a `zombie_decayed_token_spec()` helper
function and 4 token-specific tests (tests 9-12) that verify existing Decayed enforcement applies
to tokens. All CR rule text is matched correctly. Exports are properly chained through
`card_definition.rs` -> `cards/mod.rs` -> `lib.rs` -> `helpers.rs`. The `make_token()` function
at `effects/mod.rs:2901-2904` correctly propagates `TokenSpec.keywords` into the token's
`Characteristics.keywords`, so a token created with `KeywordAbility::Decayed` in its spec
automatically inherits all existing enforcement (can't block, EOC sacrifice flag, EOC sacrifice
execution). No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `memory/ability-wip.md:4` | **WIP CR number wrong.** WIP says `cr: 702.36` but 702.36 is Fear. Decayed is 702.147. Tests and code correctly cite 702.147. **Fix:** Update WIP line 4 to `cr: 702.147 (Decayed keyword)`. |
| 2 | LOW | `tests/decayed.rs:563` | **Test 9 uses internal API.** `execute_effect` is `pub` but is engine-internal; the conventions doc says tests should use "the public API only." This is acceptable here since it directly tests the `make_token()` keyword propagation path which is the core feature being validated. No fix required. |

### Finding Details

#### Finding 1: WIP CR number wrong

**Severity**: LOW
**File**: `memory/ability-wip.md:4`
**CR Rule**: 702.147 -- "Decayed represents a static ability and a triggered ability."
**Issue**: The ability-wip.md metadata line says `cr: 702.36 (Decayed keyword)` but CR 702.36 is Fear. Decayed is CR 702.147. All implementation code and tests correctly reference 702.147a, so this is purely a metadata error with no functional impact.
**Fix**: Change line 4 of `memory/ability-wip.md` to `cr: 702.147 (Decayed keyword)`.

#### Finding 2: Test 9 uses internal API

**Severity**: LOW
**File**: `crates/engine/tests/decayed.rs:563`
**CR Rule**: N/A -- coding convention (conventions.md: "Black-box testing against the public API only")
**Issue**: Test 9 (`test_702_147_decayed_token_created_with_keyword`) imports and calls `execute_effect` and constructs an `EffectContext` directly. These are technically public (`pub fn`) but are engine internals, not the Command/Event public API surface. However, this is the only reasonable way to test the `TokenSpec` -> `make_token()` -> keyword propagation path without authoring a full card definition. The existing test suite has similar patterns for effect-level testing.
**Fix**: No fix required. This is acceptable pragmatic testing. Note for future: if a card definition is authored (Step 5), the card's integration test would implicitly cover this path through the Command API.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.147a (can't block) | Yes (combat.rs:546) | Yes | Tests 1, 10 (non-token + token) |
| 702.147a (sacrifice at EOC) | Yes (turn_actions.rs:1541-1568) | Yes | Tests 4, 11 (non-token + token) |
| 702.147a (flag on attack) | Yes (combat.rs:433-441) | Yes | Test 3 |
| Ruling: sacrifice persists after keyword loss | Yes (flag-based, not keyword-checked at EOC) | Yes | Test 5 |
| Ruling: no haste (summoning sickness) | Yes (generic, no haste grant) | Yes | Tests 8, 12 (non-token + token) |
| Ruling: no attack requirement | Yes (generic, voluntary attacks) | Yes | Test 6 |
| Token keyword propagation (make_token) | Yes (effects/mod.rs:2901-2904) | Yes | Test 9 |
| Token spec helper (zombie_decayed_token_spec) | Yes (card_definition.rs:1630) | Yes | Test 9 uses it |
| Baseline: non-decayed can block | Yes (generic) | Yes | Test 7 |

## Notes

- The `zombie_decayed_token_spec()` function correctly creates a 2/2 black Zombie creature token with Decayed keyword. The `colors` field is `[Color::Black]`, `card_types` is `[Creature]`, `subtypes` is `[Zombie]`, `keywords` is `[Decayed]`, and P/T is 2/2. This matches the standard MID/VOW token oracle text.
- `make_token()` sets `has_summoning_sickness: true` (line 2958), correctly implementing CR 302.6 for tokens. Test 12 validates this for the Decayed token case.
- The `decayed_sacrifice_at_eoc` flag is initialized to `false` in `make_token()` (line 2967) and only set to `true` when the token attacks (combat.rs:437-441). The EOC sacrifice code (turn_actions.rs:1555-1562) filters on `obj.zone == Battlefield && obj.decayed_sacrifice_at_eoc`, which correctly applies to both tokens and non-tokens.
- Export chain is complete: `card_definition.rs` (definition) -> `cards/mod.rs` (re-export) -> `lib.rs` (re-export) -> `helpers.rs` (DSL prelude). Tests import via `use mtg_engine::zombie_decayed_token_spec`.

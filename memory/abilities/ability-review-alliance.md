# Ability Review: Alliance

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 207.2c (ability word -- no dedicated CR section)
**Files reviewed**:
- `crates/engine/src/state/game_object.rs` (lines 237-278)
- `crates/engine/src/state/hash.rs` (lines 1472-1488)
- `crates/engine/src/rules/abilities.rs` (lines 4602-4680, 2109-2133)
- `crates/engine/src/testing/replay_harness.rs` (lines 2162-2199)
- `crates/engine/tests/alliance.rs` (full file, 643 lines)
- `crates/engine/src/state/builder.rs` (grep for etb_filter -- 21 sites)
- `crates/engine/src/rules/resolution.rs` (line 461)

## Verdict: clean

The implementation correctly models the Alliance ability word pattern. The ETBTriggerFilter
struct cleanly separates the three filter dimensions (creature_only, controller_you,
exclude_self) with AND logic. The hash coverage is complete. All existing TriggeredAbilityDef
construction sites (21 in builder.rs, 11 in replay_harness.rs, 1 in resolution.rs) include
`etb_filter: None`. The filter enforcement in `collect_triggers_for_event` correctly checks
all three conditions and skips conservatively when the entering object is missing. The wiring
in `enrich_spec_from_def` correctly maps `WheneverCreatureEntersBattlefield` to the runtime
trigger with appropriate filter flags. Tests cover all four edge cases (self-ETB, opponent
creature, non-creature, token creature) plus the positive case with life gain verification.

No HIGH or MEDIUM findings. Two LOW observations noted below.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `replay_harness.rs:2186` | **exclude_self always true.** Hardcoded for all `WheneverCreatureEntersBattlefield` uses. Correct for all current cards (Alliance = "another"; Impact Tremors = enchantment). Would be wrong for a hypothetical creature saying "Whenever a creature you control enters" (no "another"). Documented in plan as accepted. |
| 2 | LOW | `tests/alliance.rs:564` | **Token test uses cast-from-hand, not actual token creation.** Test 5 validates the filter logic correctly (creature_only + controller_you + exclude_self), but does not exercise the `CreateToken` effect path through resolution.rs. A real token ETB test would need a card with `Effect::CreateToken` and verify the `PermanentEnteredBattlefield` event fires for the token. |

### Finding Details

#### Finding 1: exclude_self hardcoded to true

**Severity**: LOW
**File**: `crates/engine/src/testing/replay_harness.rs:2186`
**CR Rule**: 207.2c -- ability words have no special rules meaning
**Issue**: The `exclude_self: true` default in the `WheneverCreatureEntersBattlefield` wiring
is correct for all current cards but would be incorrect for a creature card that says
"Whenever a creature you control enters" (without "another"). Examples include Soul Warden
(a creature with "Whenever another creature enters" -- correct) vs a hypothetical creature
with "Whenever a creature enters" (would incorrectly exclude self).
**Fix**: No action needed now. When a card with this pattern is authored, add an
`exclude_self` flag to the `TriggerCondition::WheneverCreatureEntersBattlefield` variant or
read the filter to determine the correct value.

#### Finding 2: Token test does not exercise CreateToken path

**Severity**: LOW
**File**: `crates/engine/tests/alliance.rs:564`
**CR Rule**: 207.2c / CR 111.1 -- tokens are objects
**Issue**: Test 5 (`test_alliance_fires_on_token_creature_etb`) simulates a token entering
by casting a creature spell from hand. While this correctly validates the ETB filter logic
(the filter does not distinguish tokens from non-tokens), it does not test the actual
`CreateToken` effect resolution path that emits `PermanentEnteredBattlefield` for tokens.
The test comment acknowledges this limitation.
**Fix**: Consider adding a test or game script that uses a card with `Effect::CreateToken`
(e.g., Prosperous Innkeeper's ETB) once the card definition is authored. This would
validate the full end-to-end token ETB -> Alliance trigger chain.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 207.2c (ability word, no rules meaning) | Yes -- no KeywordAbility variant | Yes | Correct: no enum needed |
| 603.2 (trigger condition matching) | Yes -- ETBTriggerFilter in collect_triggers_for_event | Yes | test_alliance_fires_when_another_creature_enters |
| "another" qualifier (exclude_self) | Yes -- etb_filter.exclude_self | Yes | test_alliance_does_not_fire_on_self_etb |
| "you control" qualifier (controller_you) | Yes -- etb_filter.controller_you | Yes | test_alliance_does_not_fire_on_opponents_creature_etb |
| "creature" qualifier (creature_only) | Yes -- etb_filter.creature_only | Yes | test_alliance_does_not_fire_on_noncreature_permanent_etb |
| Tokens count as creatures | Yes -- filter checks card_types only | Yes (partial) | test_alliance_fires_on_token_creature_etb (cast path, not CreateToken path) |
| Simultaneous ETBs | Supported by collect_triggers_for_event scanning all objects | No | Not tested; each ETB fires separately per existing architecture |
| Panharmonicon doubling | Supported -- AnyPermanentEntersBattlefield events already doubled | No | Not tested; existing doubler infrastructure covers this |
| Hash coverage | Yes -- ETBTriggerFilter.hash_into hashes all 3 bools; TriggeredAbilityDef.hash_into includes etb_filter | N/A | Verified via grep |
| All existing TriggeredAbilityDef sites updated | Yes -- 33 construction sites all have etb_filter: None | N/A | Verified via grep (21 builder.rs + 11 replay_harness.rs + 1 resolution.rs) |

## Impact Tremors Fix Verification

The `WheneverCreatureEntersBattlefield` wiring in `enrich_spec_from_def` (replay_harness.rs:2162-2199) correctly fixes the Impact Tremors trigger gap identified in the plan. The mapping from `TriggerCondition::WheneverCreatureEntersBattlefield` to `TriggerEvent::AnyPermanentEntersBattlefield` with an `ETBTriggerFilter` means Impact Tremors (and any card using this trigger condition) will now fire at runtime. The `exclude_self: true` is harmless for Impact Tremors since it is an enchantment and can never be the entering creature.

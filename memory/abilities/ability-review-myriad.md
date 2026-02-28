# Ability Review: Myriad

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.116
**Files reviewed**:
- `crates/engine/src/state/types.rs:521-530` (KeywordAbility::Myriad)
- `crates/engine/src/state/hash.rs:436-437` (KeywordAbility discriminant 64)
- `crates/engine/src/state/hash.rs:595-596` (GameObject myriad_exile_at_eoc hash)
- `crates/engine/src/state/hash.rs:1040-1041` (PendingTrigger is_myriad_trigger hash)
- `crates/engine/src/state/hash.rs:1273-1281` (StackObjectKind::MyriadTrigger discriminant 13)
- `crates/engine/src/state/game_object.rs:392-398` (myriad_exile_at_eoc field)
- `crates/engine/src/state/stack.rs:290-308` (MyriadTrigger variant)
- `crates/engine/src/state/stubs.rs:172-180` (is_myriad_trigger on PendingTrigger)
- `crates/engine/src/state/builder.rs:665-685` (Myriad TriggeredAbilityDef wiring)
- `crates/engine/src/state/builder.rs:745` (myriad_exile_at_eoc init in build())
- `crates/engine/src/state/mod.rs:290-291,373-374` (myriad_exile_at_eoc reset on zone change)
- `crates/engine/src/effects/mod.rs:2273` (myriad_exile_at_eoc init in make_token())
- `crates/engine/src/rules/abilities.rs:1186-1207` (myriad trigger tagging in AttackersDeclared)
- `crates/engine/src/rules/abilities.rs:1882-1894` (MyriadTrigger flush branch)
- `crates/engine/src/rules/resolution.rs:1137-1254` (MyriadTrigger resolution)
- `crates/engine/src/rules/resolution.rs:1357` (MyriadTrigger counter fallthrough)
- `crates/engine/src/rules/turn_actions.rs:476-518` (end-combat exile)
- `crates/engine/src/rules/abilities.rs:1644-1668` (collect_triggers_for_event defaults)
- `tools/replay-viewer/src/view_model.rs:658` (display arm)
- `crates/engine/tests/myriad.rs` (7 tests)

## Verdict: needs-fix

The Myriad implementation is largely correct and well-structured. Token creation,
combat registration, opponent filtering, and end-of-combat exile all follow the
CR rule text faithfully. Hash coverage is complete across all new fields and enum
variants. The code correctly handles the 2-player edge case (no tokens), multiple
myriad instances (CR 702.116b), and preventing re-triggering on tokens that enter
attacking.

Two MEDIUM issues exist: (1) the end-of-combat exile is implemented as a turn-based
action rather than a delayed triggered ability, deviating from CR 702.116a / CR 603.7
timing (cannot be Stifled); (2) the source creature leaving the battlefield before
trigger resolution produces tokens with empty/default characteristics rather than using
last-known information, because both the direct clone and the CopyOf reference fail.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `rules/turn_actions.rs:481` | **End-of-combat exile is TBA, not delayed triggered ability.** CR says exile is a delayed trigger that can be interacted with. **Fix:** document as known V1 limitation; no code change required until DelayedTrigger infra exists. |
| 2 | MEDIUM | `rules/resolution.rs:1176-1180` | **Source gone at resolution produces empty-characteristics tokens.** If source creature is removed, tokens get `Characteristics::default()` and CopyOf does nothing. **Fix:** guard the loop -- if source is not on battlefield, skip token creation entirely (myriad says "copy of this creature"; if it does not exist, use LKI or create nothing). |
| 3 | LOW | `rules/resolution.rs:1176-1180` | **Redundant characteristics initialization.** Token's `characteristics` field is cloned from source AND a CopyOf continuous effect overwrites it during layer calculation. The clone is harmless but redundant. **Fix:** use `Characteristics::default()` for the base since CopyOf handles it. |
| 4 | LOW | `rules/abilities.rs:1202` | **Myriad trigger identification uses string matching.** `ta.description.starts_with("Myriad")` is fragile. A description change breaks tagging. **Fix:** acceptable for V1; consistent with other special triggers in codebase. No change needed. |
| 5 | LOW | `tests/myriad.rs` | **Missing test: source removed before resolution.** No test verifying behavior when the source creature dies before the myriad trigger resolves. **Fix:** add test `test_myriad_source_removed_before_resolution` -- kill source in response, verify no tokens created (after Finding 2 fix) or that tokens have reasonable characteristics. |
| 6 | LOW | `tests/myriad.rs` | **Missing test: token copies have correct characteristics.** No test uses `calculate_characteristics()` to verify CopyOf actually produces matching name/P/T/types. Test 4 checks flags but not copied values. **Fix:** add assertions using `calculate_characteristics()` to verify name, power, toughness match the source. |
| 7 | LOW | `rules/resolution.rs:1173-1210` | **Token constructed inline, not via make_token().** All other token creation uses the `make_token()` helper in effects/mod.rs. Myriad constructs the GameObject inline, risking field drift. **Fix:** consider using `make_token()` and then overriding `myriad_exile_at_eoc`, `status.tapped`, etc. Low priority since all fields are present. |

### Finding Details

#### Finding 1: End-of-combat exile is TBA, not delayed triggered ability

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/turn_actions.rs:481`
**CR Rule**: 702.116a -- "If one or more tokens are created this way, exile the tokens at end of combat." / CR 603.7 -- delayed triggered abilities / CR 511.2 -- "Abilities that trigger 'at end of combat' trigger as the end of combat step begins."
**Issue**: The myriad exile is implemented as a turn-based action in `end_combat()` that directly exiles tokens. Per CR 702.116a, this should be a delayed triggered ability created when the myriad trigger resolves. As a delayed trigger, it would go on the stack at the beginning of the end of combat step (CR 511.2), giving players priority to respond (e.g., Stifle to keep the tokens). The current implementation exiles tokens with no opportunity for interaction.
**Fix**: This was an explicit V1 simplification documented in the plan (approach B: tag-based TBA). No code change required until the engine's DelayedTrigger infrastructure is expanded. Document this as a known limitation in the code comment and in the ability coverage doc. When DelayedTrigger is implemented, refactor to create a delayed trigger at myriad resolution time that triggers at end of combat.

#### Finding 2: Source gone at resolution produces empty-characteristics tokens

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:1176-1180`
**CR Rule**: 702.116a -- "create a token that's a copy of this creature" / CR 608.2b -- "If a triggered ability has a source that's no longer in the zone it was in when it triggered, the ability uses last-known information."
**Issue**: If the source creature leaves the battlefield before the myriad trigger resolves, two things fail: (1) the direct characteristics clone uses `unwrap_or_default()` producing empty characteristics; (2) the CopyOf continuous effect references a dead ObjectId and `get_copiable_values()` returns `None`, so the copy does nothing. The resulting tokens have no name, no P/T, no types -- they are blank objects on the battlefield that will be exiled at end of combat. Per CR, the ability should either use last-known information or simply not create tokens if the source no longer exists (since "this creature" refers to the specific object that attacked).
**Fix**: Add a guard at the top of the opponent loop: `if state.objects.get(&source_object).map_or(true, |o| o.zone != ZoneId::Battlefield) { break; }`. This skips token creation if the source is no longer on the battlefield. Alternatively, if LKI infrastructure exists, capture the source characteristics at trigger time (on the PendingTrigger or StackObject). The simpler "skip if gone" approach is acceptable for V1.

#### Finding 3: Redundant characteristics initialization

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:1176-1180`
**CR Rule**: 707.2
**Issue**: The token's `characteristics` is initialized by cloning the source's base characteristics. Then a CopyOf continuous effect is applied, which overwrites characteristics during `calculate_characteristics()`. The direct clone is redundant since CopyOf handles everything. In the edge case where the source is gone (Finding 2), the direct clone provides `Characteristics::default()` but the CopyOf also fails, so neither approach helps.
**Fix**: Use `Characteristics::default()` for the initial value instead of cloning from the source. The CopyOf effect is the authoritative source of copied characteristics. Low priority; current behavior is correct.

#### Finding 4: Myriad trigger identification uses string matching

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:1202`
**CR Rule**: N/A (code quality)
**Issue**: Myriad triggers are identified by checking `ta.description.starts_with("Myriad")` and `ta.effect.is_none()`. If the description format is ever changed (e.g., prefixed with "CR 702.116a:"), the identification silently breaks and no myriad triggers fire.
**Fix**: This pattern is consistent with how other special triggers are handled in this codebase. No change needed for V1. Future improvement: use a dedicated `TriggerKind` enum variant instead of description-string matching.

#### Finding 5: Missing test for source removed before resolution

**Severity**: LOW
**File**: `crates/engine/tests/myriad.rs`
**CR Rule**: 702.116a, 400.7, 608.2b
**Issue**: No test covers the scenario where the myriad creature is destroyed in response to its own trigger. This is a documented edge case in the plan (interaction #6: "After the source creature leaves the battlefield, the CopyOf continuous effect may reference a dead ObjectId"). The test suite should verify the engine handles this gracefully (no panic, reasonable behavior).
**Fix**: Add `test_myriad_source_removed_before_resolution` that kills the source creature after the trigger goes on the stack but before it resolves. After Finding 2 is fixed, verify no tokens are created (or that tokens have reasonable characteristics if using LKI).

#### Finding 6: Missing test for token copy characteristics

**Severity**: LOW
**File**: `crates/engine/tests/myriad.rs`
**CR Rule**: 707.2 -- copy uses copiable values
**Issue**: Test 4 (`test_myriad_token_has_correct_flags`) verifies `is_token`, `myriad_exile_at_eoc`, `tapped`, and `has_summoning_sickness`, but does not verify that the token's calculated characteristics (name, power, toughness, types) match the source creature. The plan's Test 4 ("test_myriad_tokens_are_copies_of_source") explicitly calls for verifying characteristics via `calculate_characteristics`.
**Fix**: In test 4 or a new test, call `calculate_characteristics()` on the token and assert that the name, power, toughness, and creature types match the source.

#### Finding 7: Token constructed inline, not via make_token()

**Severity**: LOW
**File**: `crates/engine/src/rules/resolution.rs:1173-1210`
**CR Rule**: N/A (code quality / maintainability)
**Issue**: The myriad token is constructed as an inline `GameObject { ... }` with all fields specified manually. All other token creation in the engine uses the `make_token()` helper in `effects/mod.rs`. If a new field is added to `GameObject` in the future, `make_token()` will be updated but the myriad inline construction may be missed, causing a compilation error at best or a missing initialization at worst.
**Fix**: Refactor to use `make_token()` and then override `myriad_exile_at_eoc = true`, `status.tapped = true`. Low priority since all required fields are currently present and the Rust compiler will error on missing struct fields.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.116a (trigger fires when attacks) | Yes | Yes | test 1, test 6, test 7 |
| 702.116a (for each opponent other than defending) | Yes | Yes | test 1, test 6 |
| 702.116a ("you may" -- optional) | Yes (auto-accept) | N/A | V1 simplification: always creates tokens |
| 702.116a (token is copy of creature) | Yes (CopyOf Layer 1) | Partial | Flags tested; calculated characteristics not verified (Finding 6) |
| 702.116a (tapped and attacking) | Yes | Yes | test 1, test 4 |
| 702.116a (attacking that player or planeswalker) | Partial (player only) | Yes | V1: always attacks player, not planeswalker. Documented simplification |
| 702.116a (exile at end of combat) | Yes (TBA, not delayed trigger) | Yes | test 3. Timing deviation (Finding 1) |
| 702.116a (2-player: no tokens) | Yes | Yes | test 2 |
| 702.116b (multiple instances separate) | Yes | Yes | test 5 |
| 603.7 (delayed trigger for exile) | No (TBA instead) | N/A | Finding 1 |
| 511.2 (at end of combat timing) | Partial (TBA) | Yes | test 3 |
| 707.2 (copy uses copiable values) | Yes | Partial | CopyOf applied; characteristics not asserted in tests |
| 400.7 (source leaves battlefield) | Partial | No | Creates blank tokens; Finding 2, Finding 5 |
| 508.5 (defending player resolution) | Yes | Yes | test 1, test 6 |

## Previous Findings (re-review only)

N/A -- first review.

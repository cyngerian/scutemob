# Ability Review: Retrace

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.81
**Files reviewed**:
- `crates/engine/src/state/types.rs:762-770`
- `crates/engine/src/state/hash.rs:513-514`
- `crates/engine/src/rules/command.rs:167-176`
- `crates/engine/src/rules/engine.rs:81-119`
- `crates/engine/src/rules/casting.rs:69,186-222,710-751,760-775,989-1001,1033`
- `crates/engine/src/testing/replay_harness.rs:224-244,739-766`
- `crates/engine/src/testing/script_schema.rs:269-273`
- `crates/engine/tests/retrace.rs:1-1252`
- `tools/replay-viewer/src/view_model.rs:725`

## Verdict: needs-fix

One HIGH finding (.expect() in engine library code) and one MEDIUM finding
(retrace + escape auto-detection conflict). The core CR 702.81a implementation is
correct: the card is cast from the graveyard by paying the normal mana cost plus
discarding a land card, and it returns to the graveyard on resolution/counter/fizzle
(not exiled). All 11 tests are well-structured and cover the documented CR rules
and edge cases. The replay harness integration is complete and all CastSpell
construction sites include the new `retrace_discard_land` field.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `casting.rs:721` | **`.expect()` in engine library code.** Violates conventions. **Fix:** replace with `ok_or_else` returning `GameStateError`. |
| 2 | MEDIUM | `casting.rs:430-434` | **Retrace + Escape auto-detection conflict.** When a card has both Retrace and Escape keywords, escape auto-detection wins and wrong cost is paid. **Fix:** suppress escape auto-detection when `casting_with_retrace` is true. |
| 3 | LOW | `casting.rs:993-1001` | **Retrace land discard bypasses madness replacement.** If a land with madness is discarded, madness replacement won't fire. **Fix (deferred):** align with cycling discard pattern in abilities.rs:460-480. |
| 4 | LOW | `retrace.rs:429` | **Redundant Flash keyword on Counterspell.** Counterspell is an instant; `.with_keyword(KeywordAbility::Flash)` is unnecessary. **Fix:** remove redundant keyword. |
| 5 | LOW | `casting.rs:186-197` | **Silent discard when Flashback auto-detects over Retrace.** If a card has both Flashback and Retrace and the player provides `retrace_discard_land`, Flashback auto-detection wins but the land discard is silently ignored (not errored). **Fix (deferred):** consider returning an error when `retrace_discard_land.is_some()` and `casting_with_flashback` is true. |

### Finding Details

#### Finding 1: `.expect()` in engine library code

**Severity**: HIGH
**File**: `crates/engine/src/rules/casting.rs:721`
**CR Rule**: N/A -- coding convention
**Issue**: The code uses `.expect("casting_with_retrace requires retrace_discard_land.is_some()")` on line 721. Per `memory/conventions.md`: "Engine crate uses typed errors -- never `unwrap()` or `expect()` in engine logic. Tests may use `unwrap()`." While the `.expect()` is logically unreachable (the `casting_with_retrace` condition on lines 191-197 guarantees `retrace_discard_land.is_some()`), the project convention forbids it unconditionally. There is only one other `.expect()` in the entire `rules/` directory (combat.rs), making this an inconsistency.
**Fix**: Replace line 720-721 with:
```rust
let land_id = retrace_discard_land.ok_or_else(|| {
    GameStateError::InvalidCommand(
        "retrace: internal error -- retrace_discard_land must be Some".into(),
    )
})?;
```

#### Finding 2: Retrace + Escape auto-detection conflict

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:430-434`
**CR Rule**: 702.81a / 702.138a / 118.9 -- "If a card has multiple abilities giving you permission to cast it... you choose which one to apply."
**Issue**: When a card has both the `Retrace` and `Escape` keywords and is in the graveyard, and the player provides `retrace_discard_land: Some(id)` with `cast_with_escape: false`, the escape auto-detection at line 430-434 (`casting_from_graveyard && card_has_escape_keyword && !casting_with_flashback && !casting_with_madness`) will set `casting_with_escape` to `true`. This causes the escape cost to be selected (line 589) instead of the normal mana cost, even though the player signaled retrace by providing a land to discard. The retrace land discard will still execute, so the player pays the wrong mana cost AND discards a land. No printed cards currently have both Retrace and Escape, but the CR allows granting keywords via external effects.
**Fix**: Add `&& !casting_with_retrace` to the escape auto-detection condition at line 430-434:
```rust
casting_from_graveyard
    && card_has_escape_keyword
    && !casting_with_flashback
    && !casting_with_madness
    && !casting_with_retrace  // Player explicitly chose retrace
```
This requires `casting_with_retrace` to be computed before the auto-escape check. It is already available from the tuple destructuring at line 221.

#### Finding 3: Retrace land discard bypasses madness replacement

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:993-1001`
**CR Rule**: 702.35a -- "If a player would discard a card with madness, the card is exiled instead."
**Issue**: The retrace land discard calls `state.move_object_to_zone(land_id, ZoneId::Graveyard(land_owner))` directly, bypassing the madness replacement effect. In the cycling discard handler (`abilities.rs:460-480`), the code checks for madness and routes to exile if applicable. However, no printed land cards have the madness keyword, so this is purely theoretical and does not affect any real game scenario.
**Fix (deferred)**: If a future card or effect grants madness to a land, align the retrace discard with the cycling discard pattern by checking for madness before deciding the destination zone.

#### Finding 4: Redundant Flash keyword on Counterspell

**Severity**: LOW
**File**: `crates/engine/tests/retrace.rs:429`
**CR Rule**: N/A -- test quality
**Issue**: The test ObjectSpec for Counterspell includes `.with_keyword(KeywordAbility::Flash)`, but Counterspell is already an instant (`CardType::Instant`), which inherently has instant-speed timing. The Flash keyword is only meaningful for non-instant cards.
**Fix**: Remove `.with_keyword(KeywordAbility::Flash)` from the Counterspell ObjectSpec in `test_retrace_card_returns_to_graveyard_when_countered`.

#### Finding 5: Silent discard when Flashback auto-detects over Retrace

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:186-197`
**CR Rule**: 702.34a / 702.81a -- player choice between multiple casting permissions
**Issue**: If a card has both Flashback and Retrace keywords, and the player provides `retrace_discard_land: Some(id)`, the Flashback auto-detection at line 114-119 sets `casting_with_flashback = true`, which then forces `casting_with_retrace = false` (line 196: `!casting_with_flashback`). The card is cast via Flashback, and the `retrace_discard_land` is silently unused. Per the CR ruling "If a card has multiple abilities giving you permission to cast it... you choose which one to apply," providing `retrace_discard_land` should arguably signal a retrace intention. No printed cards have both keywords.
**Fix (deferred)**: Consider either (a) returning an error when `retrace_discard_land.is_some()` and `casting_with_flashback` auto-detects, or (b) suppressing flashback auto-detection when `retrace_discard_land.is_some()` to let the player choose retrace.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.81a -- cast from graveyard | Yes | Yes | test_retrace_basic_cast_from_graveyard |
| 702.81a -- discard a land card as additional cost | Yes | Yes | test_retrace_basic_cast_from_graveyard (CardDiscarded event) |
| 702.81a -- follows CR 601.2b,f (additional cost rules) | Yes | Yes | Cost payment at casting.rs:989-1001 |
| 702.81a -- normal mana cost (not alternative) | Yes | Yes | test_retrace_pays_normal_mana_cost |
| 702.81a -- card returns to graveyard (not exile) on resolution | Yes (implicit) | Yes | test_retrace_card_returns_to_graveyard_on_resolution |
| 702.81a -- card returns to graveyard when countered | Yes (implicit) | Yes | test_retrace_card_returns_to_graveyard_when_countered |
| 702.81a -- re-castable after resolution | Yes | Yes | test_retrace_recast_after_resolution |
| Ruling -- sorcery-speed timing from graveyard | Yes | Yes | test_retrace_normal_timing_sorcery_cannot_cast_on_opponents_turn |
| 702.81a -- discarded card must be land type | Yes | Yes | test_retrace_discard_must_be_land |
| 702.81a -- discarded card must be in hand | Yes | Yes | test_retrace_discard_must_be_in_hand |
| Negative -- no keyword = no graveyard cast | Yes | Yes | test_retrace_no_retrace_keyword_cannot_cast_from_graveyard |
| Negative -- no land = no retrace | Yes | Yes | test_retrace_without_land_provided_cannot_cast_from_graveyard |
| Normal -- hand cast needs no land discard | Yes | Yes | test_retrace_normal_hand_cast_no_land_discard_needed |

## Additional Notes

### Architecture Compliance
- **Enum variant**: `KeywordAbility::Retrace` added correctly at types.rs:770 with comprehensive doc comment citing CR 702.81.
- **Hash coverage**: Discriminant 89 added at hash.rs:513-514. No gap in discriminant sequence (follows CommanderNinjutsu=88).
- **Command field**: `retrace_discard_land: Option<ObjectId>` with `#[serde(default)]` ensures backward compatibility with existing JSON scripts.
- **All CastSpell construction sites**: Verified 320 occurrences of `Command::CastSpell {` across 47 files all include `retrace_discard_land`. Count matches between construction sites and field occurrences.
- **No StackObject changes**: Correct -- Retrace is a static ability that does not change resolution behavior.
- **No resolution.rs changes**: Correct -- the card follows the normal instant/sorcery path to graveyard. The `cast_with_flashback` flag on StackObject is only set for Flashback casts, so retrace casts correctly fall through to graveyard destination in all three resolution paths (resolution, fizzle, counter).
- **Replay harness**: `cast_spell_retrace` action type added with proper JSON field extraction via `discard_land` in script schema.
- **Replay viewer**: `view_model.rs:725` correctly maps `KeywordAbility::Retrace` to display string.
- **TUI**: No changes needed (no new StackObjectKind variant).

### Test Quality Assessment
All 11 tests are well-structured, properly cite CR rules, use the standard `GameStateBuilder` pattern, and cover both positive and negative cases. The re-cast test (test 11) correctly handles zone-change identity (CR 400.7) by finding the new ObjectId after resolution. The counter test correctly exercises the full Counterspell flow including priority passing.

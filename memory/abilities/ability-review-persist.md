# Ability Review: Persist

**Date**: 2026-02-26
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.79
**Files reviewed**:
- `crates/engine/src/state/types.rs:270-277` (KeywordAbility::Persist variant)
- `crates/engine/src/state/hash.rs:356-357` (Persist discriminant 36)
- `crates/engine/src/state/hash.rs:1003-1015` (InterveningIf hash)
- `crates/engine/src/state/hash.rs:1306-1321` (CreatureDied hash with pre_death_counters)
- `crates/engine/src/state/game_object.rs:149-164` (InterveningIf::SourceHadNoCounterOfType)
- `crates/engine/src/state/builder.rs:438-469` (Persist keyword-to-trigger translation)
- `crates/engine/src/rules/events.rs:235-250` (CreatureDied.pre_death_counters field)
- `crates/engine/src/rules/abilities.rs:758-805` (CreatureDied trigger dispatch with pre_death_counters)
- `crates/engine/src/rules/abilities.rs:1170-1195` (check_intervening_if with SourceHadNoCounterOfType)
- `crates/engine/src/rules/resolution.rs:355-404` (resolution-time intervening-if passes None)
- `crates/engine/src/rules/sba.rs:301-357` (SBA death path captures pre_death_counters)
- `crates/engine/src/rules/replacement.rs:755-769` (zone_change_events caller captures counters)
- `crates/engine/src/rules/replacement.rs:1014-1059` (zone_change_events function signature + body)
- `crates/engine/src/effects/mod.rs:325-436` (DestroyPermanent captures pre_death_counters)
- `crates/engine/src/effects/mod.rs:758-767` (MoveZone ctx.source update for persist)
- `crates/engine/src/effects/mod.rs:1070-1170` (SacrificePermanents captures pre_death_counters)
- `crates/engine/src/rules/abilities.rs:233-252` (sacrifice-as-cost captures pre_death_counters)
- `tools/replay-viewer/src/view_model.rs:600` (format_keyword Persist arm)
- `crates/engine/tests/persist.rs` (6 unit tests)

## Verdict: clean

The Persist implementation is correct, well-structured, and thoroughly tested. It faithfully implements CR 702.79a with proper last-known-information semantics for the intervening-if counter check, correct ctx.source tracking after MoveZone (critical for the Sequence(MoveZone, AddCounter) pattern), and comprehensive CreatureDied event extension across all 8 emission sites. The infrastructure changes (pre_death_counters on CreatureDied, InterveningIf::SourceHadNoCounterOfType, MoveZone source update) are cleanly reusable for Undying (CR 702.93). All 6 tests are well-structured with clear CR citations. No HIGH or MEDIUM findings. Two LOW findings noted for completeness.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `abilities.rs:826` | **AuraFellOff passes None for pre_death_counters.** If an aura with persist and a -1/-1 counter falls off, the SourceHadNoCounterOfType check defaults to true (unwrap_or(true)), causing persist to trigger incorrectly. Extremely unlikely in practice. **Fix:** If addressing, capture aura counters before SBA move in the AuraFellOff emission site and pass them through the event. |
| 2 | LOW | `abilities.rs:773, builder.rs:446` | **SelfDies only fires from CreatureDied/AuraFellOff, not PermanentDestroyed.** CR 702.79a says "When this permanent is put into a graveyard from the battlefield" (any permanent), but persist only triggers via CreatureDied/AuraFellOff handlers. A non-creature, non-aura permanent with persist that is destroyed would emit PermanentDestroyed and not fire persist. No real cards create this scenario. **Fix:** If addressing, add SelfDies trigger check to PermanentDestroyed handler (with pre_death_counters). |

### Finding Details

#### Finding 1: AuraFellOff passes None for pre_death_counters

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:826`
**CR Rule**: 702.79a -- "if it had no -1/-1 counters on it"
**Issue**: The `AuraFellOff` event handler at line 826 passes `None` for `pre_death_counters` to `check_intervening_if`. For the `SourceHadNoCounterOfType` variant, `None` returns `true` via `unwrap_or(true)`. This means if a hypothetical aura with persist and a -1/-1 counter falls off, persist would incorrectly trigger. In practice, no aura cards have persist, and auras rarely receive -1/-1 counters, making this purely theoretical.
**Fix**: If ever needed, extend `AuraFellOff` event to carry `pre_death_counters: OrdMap<CounterType, u32>` (same pattern as CreatureDied), capture counters before the SBA move, and pass `Some(pre_death_counters)` at line 826. Defer until a card or test scenario requires it.

#### Finding 2: SelfDies limited to CreatureDied and AuraFellOff events

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:773` and `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs:446`
**CR Rule**: 702.79a -- "When this permanent is put into a graveyard from the battlefield"
**Ruling**: "If a creature with persist stops being a creature, persist will still work." (Scryfall 2013-06-07, multiple cards)
**Issue**: The CR text and rulings indicate persist should work on any permanent going to the graveyard from the battlefield, not just creatures. The implementation uses `TriggerEvent::SelfDies`, which is only checked in `CreatureDied` and `AuraFellOff` event handlers. A non-creature, non-aura permanent with persist would emit `PermanentDestroyed`, which does not check `SelfDies` triggers. However, the ruling scenario ("creature stops being a creature") would still emit `CreatureDied` if the permanent was a creature at the time it died (the SBA checks creature status to determine which event to emit). The only uncovered scenario is a permanent that was *never* a creature but somehow gained persist. No existing cards create this scenario.
**Fix**: If addressing, add `SelfDies` trigger checking to the `PermanentDestroyed` event handler in `check_triggers`, following the same pattern as the `CreatureDied` handler. Include `pre_death_counters` on `PermanentDestroyed` event. Defer until needed.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.79a (basic persist trigger) | Yes | Yes | test_persist_basic_returns_with_counter |
| 702.79a (intervening-if: no -1/-1 counters) | Yes | Yes | test_persist_does_not_trigger_with_minus_counter |
| 702.79a (returns under owner's control) | Yes | Yes | move_object_to_zone resets controller to owner; verified in test 1 |
| 702.79a (returns with -1/-1 counter) | Yes | Yes | test_persist_basic_returns_with_counter asserts counter=1 |
| 702.79a (new object, CR 400.7) | Yes | Yes | move_object_to_zone creates new ObjectId; verified by find_by_name finding different ID |
| 702.79a + CR 603.4 (intervening-if at resolution) | Yes | Partial | Resolution passes None; defaults to true; MoveZone no-ops if source left. Token test (test 4) covers the "source gone" path. No explicit test for "card exiled before trigger resolves." |
| 702.79a (second death with counter) | Yes | Yes | test_persist_second_death_no_trigger |
| 702.79a + CR 704.5d (token with persist) | Yes | Yes | test_persist_token_trigger_but_no_return |
| 702.79a + CR 603.3 (APNAP ordering) | Yes | Yes | test_persist_multiplayer_apnap_ordering (4-player) |
| 702.79a + CR 704.5q (counter annihilation re-enables persist) | Yes | Yes | test_persist_plus_one_cancellation_enables_second_persist |
| 702.79a (last known information for counters) | Yes | Yes | pre_death_counters captured before move_object_to_zone at all 8 emission sites |
| 702.79a + CR 903.9a (commander redirect skips persist) | Yes | No | sba.rs:346-348 already skips CreatureDied for command zone redirect; no explicit test |
| 702.79a (non-creature permanent) | No | No | Finding 2; SelfDies only fires from CreatureDied/AuraFellOff |

## Previous Findings (re-review only)

N/A -- first review.

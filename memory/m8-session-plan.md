# M8 Fix Phase — Session Plan

> Generated from M8 code review findings (MR-M8-01 through MR-M8-10).
> 3 HIGH + 7 MEDIUM = 10 issues across 2 fix sessions.

---

## Session 1: Critical Fixes (6 issues) — COMPLETE

Focus: The three HIGH findings plus tightly-related MEDIUM fixes in the same files.

| # | Issue | Severity | File(s) | Summary |
|---|-------|----------|---------|---------|
| 1 | MR-M8-01 | HIGH | `replacement.rs:569`, `state/replacement_effect.rs` | Add `original_from: ZoneType` to `PendingZoneChange`; update all creation sites (sba.rs, effects/mod.rs); use in `resolve_pending_zone_change` re-check instead of hardcoded `Battlefield` |
| 2 | MR-M8-02 | HIGH | `effects/mod.rs:323-332,401-410` | Add `ChoiceRequired` match arms in `DestroyPermanent` and `ExileObject` — create `PendingZoneChange` + emit `ReplacementChoiceRequired`, matching SBA pattern |
| 3 | MR-M8-03 | HIGH | `turn_actions.rs:216`, `rules/layers.rs` | Expire `UntilEndOfTurn` replacement effects during cleanup; also remove corresponding `prevention_counters` entries |
| 4 | MR-M8-04 | MEDIUM | `replacement.rs:779-796` | Add `owner: PlayerId` param to `zone_change_events`; use real owner instead of hardcoded `PlayerId(0)` in `ObjectExiled` |
| 5 | MR-M8-05 | MEDIUM | `replacement.rs:790-793` | Replace `ReplacementId(u64::MAX)` sentinel with a proper `GameEvent::CommanderZoneRedirect` variant (or use real effect ID) |
| 6 | MR-M8-06 | MEDIUM | `replacement.rs:781-784` | Check object card types before choosing event variant — don't always emit `CreatureDied` for graveyard moves |

All 6 fixes applied. 6 new tests added. 400 tests passing, 0 failures.

### Tests added (Session 1):
- [x] `test_until_end_of_turn_replacement_expires_at_cleanup` — covers MR-M8-03 + MR-M8-13
- [x] `test_indefinite_replacement_survives_cleanup` — covers MR-M8-03 negative case
- [x] `test_destroy_permanent_emits_choice_required_for_multiple_replacements` — covers MR-M8-02
- [x] `test_exile_object_emits_choice_required_for_multiple_replacements` — covers MR-M8-02
- [x] `test_zone_change_events_enchantment_emits_permanent_destroyed` — covers MR-M8-06
- [x] `GameEvent::CommanderZoneRedirect` hash test integrated into hash.rs impl — covers MR-M8-05

### Notes:
- MR-M8-05: Used `GameEvent::CommanderZoneRedirect` new variant (not real effect ID) — cleaner API
- MR-M8-04: `zone_change_events` now takes `state`, `old_id`, `new_id`, `dest`, `owner`
- All DestroyPermanent/ExileObject match arms rewritten as full exhaustive matches (no `_ =>` wildcard)

---

## Session 2: Remaining MEDIUMs (4 issues) — COMPLETE

Focus: Draw replacement DRY, WouldDraw NeedsChoice, Leyline filter, hash gap.

| # | Issue | Severity | File(s) | Summary |
|---|-------|----------|---------|---------|
| 1 | MR-M8-07 | MEDIUM | `turn_actions.rs:101-130`, `effects/mod.rs:1236-1263` | Extract shared WouldDraw replacement check into `replacement.rs` (e.g., `check_would_draw_replacement`); call from both draw paths |
| 2 | MR-M8-08 | MEDIUM | `turn_actions.rs:128`, `effects/mod.rs:1262` | Handle `NeedsChoice` for WouldDraw — emit `ReplacementChoiceRequired` or document as M8 limitation if no current card triggers it |
| 3 | MR-M8-09 | MEDIUM | `definitions.rs:1383-1394` | Add `ObjectFilter::OwnedByOpponentsOf(PlayerId)` variant; use for Leyline of the Void; implement in `object_matches_filter` |
| 4 | MR-M8-10 | MEDIUM | `state/hash.rs:401-416` | Add `self.cleanup_sba_rounds.hash_into(hasher)` to `TurnState::hash_into` |

All 4 fixes applied. 4 new tests added. 404 tests passing, 0 failures.

### Tests added (Session 2):
- [x] `test_draw_cards_effect_respects_skip_draw_replacement` — covers MR-M8-07 (draw_one_card path respects SkipDraw)
- [x] `test_draw_needs_choice_emits_replacement_choice_required` — covers MR-M8-08 (NeedsChoice defers draw)
- [x] `test_leyline_of_the_void_opponent_only_filter` — covers MR-M8-09 (filter bound to controller, matches opponents only)
- [x] `test_hash_cleanup_sba_rounds_affects_hash` — covers MR-M8-10 (cleanup_sba_rounds in hash)

### Notes:
- MR-M8-07/08: `DrawAction` enum in `replacement.rs` with `Proceed/Skip/NeedsChoice` variants; both draw paths use `check_would_draw_replacement`
- MR-M8-08: `NeedsChoice` now emits `ReplacementChoiceRequired` and defers the draw (not just a comment)
- MR-M8-09: `ObjectFilter::OwnedByOpponentsOf(PlayerId(0))` in card definition — `PlayerId(0)` is a placeholder; `register_permanent_replacement_abilities` binds the actual controller at registration time
- MR-M8-10: Single line added to `TurnState::hash_into`; hash sensitivity test confirms states differing only in `cleanup_sba_rounds` produce different hashes

---

## LOW findings (deferred)

MR-M8-11 through MR-M8-16 — address opportunistically during M9 or later:
- MR-M8-11: Damage prevention registration order vs player choice (rare edge case)
- MR-M8-12: Self-ETB bypasses replacement framework (correct for current cards)
- MR-M8-13: Test gap for UntilEndOfTurn expiration (covered by Session 1 fix of MR-M8-03)
- MR-M8-14: Darksteel Colossus shuffle simplification
- MR-M8-15: No multi-ETB interaction test
- MR-M8-16: Stale replacement effects grow unbounded

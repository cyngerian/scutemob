# M9 Fix Session Plan

8 issues total: 2 HIGH + 6 MEDIUM. Grouped into 2 sessions by subsystem.

## Session 1 -- Commander core logic (commander.rs)

- [x] MR-M9-01 (HIGH) -- Commander zone return SBA auto-applied without player choice (CR 903.9a)
- [x] MR-M9-02 (HIGH) -- `compute_color_identity` only reads mana cost, not rules text mana symbols (CR 903.4)
- [x] MR-M9-03 (MEDIUM) -- Commander tax overflow: `base_cost.generic + tax * 2` can overflow u32
- [x] MR-M9-05 (MEDIUM) -- Mulligan draws from library via `draw_card` can trigger library-empty loss during pregame
- [x] MR-M9-07 (MEDIUM) -- `validate_deck` silently skips cards not in registry (Architecture Invariant 9)

## Session 2 -- Casting, companion, and type checks

- [x] MR-M9-04 (MEDIUM) -- `cards_to_bottom.len() as u32` truncates on 64-bit platforms
- [x] MR-M9-06 (MEDIUM) -- Companion mana deduction duplicates `pay_cost` logic with fixed priority order
- [x] MR-M9-08 (MEDIUM) -- Command zone casting uses raw characteristics for type checks

## Notes

- Session 1 focuses on `commander.rs` core logic: the two HIGHs (SBA choice + color identity) plus three MEDIUMs in the same file.
- Session 2 focuses on cross-file concerns: casting.rs type checks, companion mana deduction, and the `as u32` cast.
- MR-M9-01 is the most complex fix -- requires a choice mechanism (ChoiceRequired path or new command variant). Consider whether the "known simplification" documentation is acceptable for now, or whether a full choice path is required.
- MR-M9-02 may be best addressed by adding a `color_identity` field to `CardDefinition` populated from Scryfall data, rather than parsing oracle text.
- All fixes should run `~/.cargo/bin/cargo test --all` and `~/.cargo/bin/cargo clippy -- -D warnings` after each session.

## Session 1 Completion Notes (2026-02-23)

All 5 Session 1 fixes applied; all tests pass; clippy clean; fmt clean.

**MR-M9-01** — Full choice path implemented (not just documentation). Added `CommanderZoneReturnChoiceRequired` event (discriminant 62), `LeaveCommanderInZone` command, `pending_commander_zone_choices: Vector<(PlayerId, ObjectId)>` field on `GameState`. SBA emits choice event and adds to pending list; player responds with `ReturnCommanderToCommandZone` (moves to command zone, clears pending) or `LeaveCommanderInZone` (stays, clears pending). Updated 4 test files (commander.rs, commander_damage.rs, replacement_effects.rs), script replay harness, and 2 game scripts. 9 files total changed.

**MR-M9-02** — Oracle text parsing via `add_colors_from_oracle_text` helper (byte-scan for `{...}` symbols; no regex dependency). Did not add a `color_identity` field to `CardDefinition` since oracle text parsing is sufficient and avoids a schema change.

**MR-M9-03** — Used `saturating_mul(2)` + `saturating_add` (not `checked_*`) for simpler code that always succeeds; saturating at `u32::MAX` is acceptable for overflow protection in an unreachable edge case.

**MR-M9-05** — Direct zone move loop with `break` on empty library; `CardDrawn` events still emitted for each draw. Removed now-unused `use crate::rules::turn_actions;` import.

**MR-M9-07** — Added `DeckViolation::UnknownCard { card_id: String }` variant; exported from `lib.rs` via existing `DeckViolation` re-export. Added test `test_unknown_card_produces_violation` in deck_validation.rs.

## Session 2 Completion Notes (2026-02-23)

All 3 Session 2 fixes applied; 448 tests pass; clippy clean; fmt clean.

**MR-M9-04** — Changed `cards_to_bottom.len() as u32 != required_bottom` to `cards_to_bottom.len() != required_bottom as usize`. Minimal one-line change; no behavior change in practice since library sizes never reach u32::MAX.

**MR-M9-06** — Added `use crate::rules::casting;` import to `commander.rs` and replaced the 18-line hand-rolled mana deduction loop with a call to `casting::pay_cost(&mut ps.mana_pool, &companion_cost)`. Intra-crate circular `use` between `commander` and `casting` is fine in Rust (single-crate modules compile together). The pre-validation `total_mana >= 3` check is preserved; `pay_cost` now handles the deduction.

**MR-M9-08** — Restructured the early section of `handle_cast_spell` in `casting.rs`. Extracted `casting_from_command_zone`, `card_id`, and `base_mana_cost` from `card_obj` inside a scoped block so the borrow ends before calling `calculate_characteristics`. The land-type check and instant-speed check now use the computed characteristics (with fallback to raw on `None`). No behavioral change for current tests (command zone objects don't participate in layer calculations), but pattern is now consistent with `validate_object_satisfies_requirement`.

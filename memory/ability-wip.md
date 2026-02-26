# Ability WIP: Dredge

ability: Dredge
cr: 702.52
priority: P1
started: 2026-02-26
phase: closed

## Review
findings: 5 (0 HIGH, 2 MEDIUM, 3 LOW)
review_file: memory/ability-review-dredge.md
verdict: needs-fix
plan_file: memory/ability-plan-dredge.md

## Step Checklist
- [x] 1. Enum variant — types.rs:205, hash.rs:317-321, events.rs:672-700, command.rs:186-199, view_model.rs:593
- [x] 2. Rule enforcement — replacement.rs: DrawAction::DredgeAvailable, check_would_draw_replacement dredge scan, handle_choose_dredge, draw_card_skipping_dredge; engine.rs: ChooseDredge arm; turn_actions.rs + effects/mod.rs: DredgeAvailable arm
- [x] 3. Trigger wiring — N/A (Dredge is a replacement effect, not a trigger)
- [x] 4. Unit tests — crates/engine/tests/dredge.rs (13 tests; +2 from fix phase: effect-based draw, milled-card-available-for-next-draw)
- [x] 5. Card definition — Golgari Grave-Troll ({4G}, 0/4, Dredge 6) @ definitions.rs:1942
- [x] 6. Game script — `test-data/generated-scripts/replacement/014_golgari_grave_troll_dredge.json` (approved, 9/9 pass; harness gap fixed: choose_dredge action added to replay_harness.rs)
- [x] 7. Coverage doc update — Dredge: none→validated; P2 validated 2→3; Total validated 38→39

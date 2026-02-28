# Ability WIP: Hideaway

ability: Hideaway
cr: 702.75
priority: P3
started: 2026-02-28
phase: closed
plan_file: memory/abilities/ability-plan-hideaway.md

## Step Checklist
- [x] 1. Enum variant (types.rs:544, hash.rs:439, game_object.rs:420, stack.rs:370, stubs.rs:205, hash.rs:1310, card_definition.rs:544, hash.rs:2660, events.rs:843, hash.rs:2099)
- [x] 2. Rule enforcement (resolution.rs:1468 HideawayTrigger arm; effects/mod.rs:1592 PlayExiledCard arm)
- [x] 3. Trigger wiring (abilities.rs:921 check_triggers ETB generation; abilities.rs:2011 flush_pending_triggers HideawayTrigger branch)
- [x] 4. Unit tests (crates/engine/tests/hideaway.rs: 7 tests — ETB trigger fires, resolution exiles face-down, exiled_by_hideaway tracking, empty library edge case, face_down enforcement, PlayExiledCard activation, negative no-keyword)
## Review
findings: 1 HIGH, 1 MEDIUM, 6 LOW — fixes applied (HIGH-1, MEDIUM-2); 6 LOWs deferred
review_file: memory/abilities/ability-review-hideaway.md

- [x] 5. Card definition — Windbrisk Heights (Land—Plains, Hideaway 4, enters tapped, {T}:{W}, {W},{T}: play exiled) — definitions.rs #112
- [x] 6. Game script — test-data/generated-scripts/baseline/103_windbrisk_heights_hideaway_etb.json
- [x] 7. Coverage doc update

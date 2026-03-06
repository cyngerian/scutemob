# Ability WIP: Phasing

ability: Phasing
cr: 702.26
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 7 (2 HIGH, 4 MEDIUM, 1 LOW)
review_file: memory/abilities/ability-review-phasing.md
plan_file: memory/abilities/ability-plan-phasing.md

## Fix Phase (completed 2026-03-06)
- [x] HIGH-1: turn_actions.rs — simultaneous phasing (snapshot both sets before mutating)
- [x] HIGH-2: tests/phasing.rs:173 — inverted assertion in test_phasing_basic_phase_in_on_next_untap
- [x] MEDIUM-3: turn_actions.rs — CR 702.26h direct+indirect inverted (force_indirect set)
- [x] MEDIUM-4: abilities.rs — 15 battlefield scan sites + is_phased_in() filter
- [x] MEDIUM-5: effects/mod.rs — 16 battlefield scan sites + is_phased_in() filter
- [x] MEDIUM-6: tests/phasing.rs — added test_phasing_excluded_from_continuous_effects (CR 702.26e)
- [ ] LOW-7: casting.rs, engine.rs, replacement.rs — phased-out target/command filters (deferred)

## Step Checklist
- [x] 1. Enum variant — types.rs:1082, hash.rs:604, view_model.rs:818
- [x] 2. Rule enforcement — game_object.rs:534-562, events.rs:961-992, turn_actions.rs:681-780, sba.rs (phased-out filters), layers.rs:187+219-248, combat.rs:74-83+516-524
- [x] 3. Trigger wiring — n/a (phasing does not use the stack, no triggers)
- [x] 4. Unit tests — tests/phasing.rs (16 tests covering CR 502.1/702.26a/b/d/e/g/h/p)
- [x] 5. Card definition — crates/engine/src/cards/defs/teferis_isle.rs
- [x] 6. Game script — test-data/generated-scripts/stack/155_phasing_teferis_isle.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Phasing: validated)

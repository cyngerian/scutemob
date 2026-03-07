# Ability WIP: Graft

ability: Graft
cr: 702.58
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 4 (1 HIGH, 3 LOW)
review_file: memory/abilities/ability-review-graft.md
plan_file: memory/abilities/ability-plan-graft.md

## Step Checklist
- [x] 1. Enum variant — `state/types.rs:1095` (Graft(u32), KW disc 119); `state/hash.rs` (119u8 arm); `state/stubs.rs:88` (PendingTriggerKind::Graft); `stubs.rs:303` (graft_entering_creature field); `state/stack.rs:957` (GraftTrigger disc 44); `hash.rs` (GraftTrigger 44u8 arm + PendingTrigger field); TUI `stack_view.rs`
- [x] 2. Rule enforcement — `rules/resolution.rs` ETB counter block after Modular (~line 639); `rules/lands.rs` ETB counter block after Fading (~line 178)
- [x] 3. Trigger wiring — `rules/abilities.rs` Graft collection in PermanentEnteredBattlefield arm (~line 2414); `flush_pending_triggers` Graft arm; `resolution.rs` GraftTrigger handler + catch-all arm
- [x] 4. Unit tests — `tests/graft.rs` (9 tests: ETB counters, trigger moves counter, no fire without counters, no self-trigger, fires for opponents, non-creature negative, multiple instances 2 triggers, multiple instances ETB sum, resolution recheck)
- [x] 5. Card definition — crates/engine/src/cards/defs/simic_initiate.rs
- [x] 6. Game script — test-data/generated-scripts/stack/156_graft_simic_initiate.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Graft: validated)

# Ability WIP: Flanking

ability: Flanking
cr: 702.25
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-flanking.md

## Step Checklist
- [x] 1. Enum variant — state/types.rs:638, state/hash.rs:470, tools/replay-viewer/src/view_model.rs:688
- [x] 2. Rule enforcement — state/stubs.rs:242, state/hash.rs:1095, rules/abilities.rs (all PendingTrigger literals updated)
- [x] 3. Trigger wiring — state/stack.rs:445, state/hash.rs:1392, rules/abilities.rs (BlockersDeclared handler + flush), rules/resolution.rs (FlankingTrigger resolution arm), tools/tui/stack_view.rs, tools/replay-viewer/view_model.rs
- [x] 4. Unit tests — crates/engine/tests/flanking.rs (7 tests: basic -1/-1, no trigger on flanking blocker, kills 1/1, multiple instances, multiple blockers, EOT expiry, multiplayer)
- [x] 5. Card definition — crates/engine/src/cards/defs/suq_ata_lancer.rs (Suq'Ata Lancer, {2}{R}, 2/2, Haste + Flanking)
- [x] 6. Game script — test-data/generated-scripts/combat/114_suq_ata_lancer_flanking_trigger.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 7/88, 99 total validated)

# Ability WIP: Backup

ability: Backup
cr: 702.165
priority: P4
started: 2026-03-06
phase: close

## Review
findings: 3 (1 HIGH, 1 MEDIUM, 1 LOW)
review_file: memory/abilities/ability-review-backup.md
plan_file: memory/abilities/ability-plan-backup.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1143, hash.rs:625-629, stack.rs:991-1013, stubs.rs:93-95,302-309, view_model.rs:561-563,836, stack_view.rs:174-176
- [x] 2. Rule enforcement — stubs.rs:PendingTriggerKind::Backup+backup_abilities+backup_n fields; hash.rs:BackupTrigger arm
- [x] 3. Trigger wiring — abilities.rs:check_triggers Backup block; abilities.rs:flush_pending_triggers Backup→BackupTrigger arm; resolution.rs:BackupTrigger resolution+countering arms
- [x] 4. Unit tests — crates/engine/tests/backup.rs (10 tests, all passing)
- [x] 5. Card definition — crates/engine/src/cards/defs/backup_agent.rs
- [x] 6. Game script — test-data/generated-scripts/stack/163_backup_agent_etb_trigger.json (self-target path; another-creature path covered by unit tests)
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Backup: validated)

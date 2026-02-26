# Ability WIP: DeclareAttackers

ability: DeclareAttackers
cr: 508.1
priority: P1
started: 2026-02-26
phase: closed

## Review
findings: 6 (0 HIGH, 2 MEDIUM, 4 LOW)
review_file: memory/ability-review-declareattackers.md
verdict: needs-fix
plan_file: memory/ability-plan-declareattackers.md

## Step Checklist
- [x] 1. Enum variant — N/A (Command::DeclareAttackers/DeclareBlockers already exist in command.rs:86-101)
- [x] 2. Rule enforcement — N/A (handle_declare_attackers/handle_declare_blockers already exist in combat.rs)
- [x] 3. Trigger wiring — N/A (triggers already fire for attacker/blocker declarations)
- [x] 4. Unit tests — crates/engine/tests/combat_harness.rs (6 tests: basic/empty/default_target for attackers; basic/empty/full_combat for blockers)
- [x] 5. Card definition — N/A (no new cards needed; existing cards used in scripts)
- [x] 6. Game script — combat/015_declare_attackers_unblocked.json (12/12), combat/016_declare_blockers_creature_dies.json (21/21)
- [x] 7. Coverage doc update — DeclareAttackers+DeclareBlockers: partial→validated (both); P1 validated 40→42; P1 gaps: 0 remaining

# Ability WIP: Spree

ability: Spree
cr: 702.172
priority: P4
started: 2026-03-07
phase: closed
plan_file: memory/abilities/ability-plan-spree.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1230, hash.rs:656, view_model.rs:853
- [x] 2. Rule enforcement — card_definition.rs:1213 (mode_costs field), hash.rs:3257 (ModeSelection hash), casting.rs:2013 (Spree cost block)
- [x] 3. Trigger wiring — n/a (Spree is static, not triggered)
- [x] 4. Unit tests — crates/engine/tests/spree.rs (9 tests: all pass)
## Review
findings: 1 MEDIUM (fixed), 2 LOW (deferred)
review_file: memory/abilities/ability-review-spree.md
fix: MEDIUM #1 applied — casting.rs `mut modes_chosen` + `modes_chosen.sort_unstable()` at line 2988; test replaced with stack-object assertion

- [x] 5. Card definition — final_showdown.rs (Spree: modes 0+1 no-op (DSL gap), mode 2 destroys all creatures)
- [x] 6. Game script — 173_spree_final_showdown.json (mode 2: destroy all creatures)
- [x] 7. Coverage doc update — P4 66/88, 160 validated total

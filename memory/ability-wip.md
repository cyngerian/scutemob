# Ability WIP: Renown

ability: Renown
cr: 702.112
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-renown.md

## Step Checklist
- [x] 1. Enum variant — state/types.rs:682 (Renown(u32)), state/hash.rs:488 (discriminant 81), state/game_object.rs:437 (is_renowned: bool), state/hash.rs:653 (is_renowned hash), state/hash.rs:1120 (PendingTrigger renown hash), state/stubs.rs:287 (is_renown_trigger, renown_n), state/stack.rs:510 (RenownTrigger variant), state/hash.rs:1439 (discriminant 22)
- [x] 2. Rule enforcement — rules/abilities.rs:2139 (Renown dispatch in CombatDamageDealt), rules/abilities.rs:2562 (flush to stack), rules/resolution.rs:1842 (RenownTrigger resolution arm)
- [x] 3. Trigger wiring — n/a (covered by Step 2; custom StackObjectKind pattern like Ingest)
- [x] 4. Unit tests — crates/engine/tests/renown.rs (7 tests: basic, renown-2, no-trigger-when-renowned, zone-change-reset, multiple-instances-cr603.4, leaves-before-resolution, multiplayer)
- [x] 5. Card definition — crates/engine/src/cards/defs/topan_freeblade.rs (Topan Freeblade, {1}{W}, 2/2, Vigilance + Renown 1)
- [x] 6. Game script — test-data/generated-scripts/combat/119_topan_freeblade_renown_combat_damage.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 12/88, 104 total validated)

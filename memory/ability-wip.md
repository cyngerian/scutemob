# Ability WIP: Suspect

ability: Suspect
cr: 701.60
priority: P4
started: 2026-03-07
phase: closed

## Review
findings: 0 HIGH/MEDIUM + 2 LOW (deferred)
review_file: memory/abilities/ability-review-suspect.md
plan_file: memory/abilities/ability-plan-suspect.md

## Step Checklist
- [x] 1. Enum variant — game_object.rs:476, card_definition.rs:884, events.rs:797, hash.rs:840, builder.rs:973, mod.rs:347+502, effects/mod.rs:2843, resolution.rs (4 sites)
- [x] 2. Rule enforcement — layers.rs (Menace pre-loop), combat.rs:546+837 (can't-block), effects/mod.rs:1715 (Effect::Suspect/Unsuspect handlers)
- [x] 3. Trigger wiring — n/a (keyword action, not a triggered ability)
- [x] 4. Unit tests — crates/engine/tests/suspect.rs (9 tests: gains_menace, cant_block, can_attack, menace_evasion, idempotent, zone_change_clears, unsuspect_removes, not_copiable, baseline)
- [x] 5. Card definition — crates/engine/src/cards/defs/frantic_scapegoat.rs (Goat, ETB self-suspect)
- [x] 6. Game script — test-data/generated-scripts/etb-triggers/180_suspect_frantic_scapegoat_etb.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md: Suspect → validated

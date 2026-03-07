# Ability WIP: Soulbond

ability: Soulbond
cr: 702.95
priority: P4
started: 2026-03-06
phase: closed
plan_file: memory/abilities/ability-plan-soulbond.md

## Step Checklist
- [x] 1. Enum variant (types.rs:1185, game_object.rs:570, hash.rs, builder.rs:964, mod.rs:368+519, effects/mod.rs:2672, resolution.rs, continuous_effect.rs:56, layers.rs:213, replacement.rs:238, stubs.rs:103+334, stack.rs:1044, card_definition.rs:540, replay_harness.rs:1784, view_model.rs:845, stack_view.rs:183)
- [x] 2. Rule enforcement (sba.rs:1112 check_soulbond_unpairing; mod.rs zone-change unpairing; resolution.rs SoulbondTrigger resolution with fizzle check)
- [x] 3. Trigger wiring (abilities.rs: SoulbondSelfETB + SoulbondOtherETB blocks after Champion ~line 2452)
- [x] 4. Unit tests (crates/engine/tests/soulbond.rs: 10 tests)
- [x] 5. Card definition (silverblade_paladin.rs)
- [x] 6. Game script (test-data/generated-scripts/stack/167_soulbond_silverblade_paladin.json)
- [x] 7. Coverage doc update

## Review
findings: 3 (1 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-soulbond.md

## Fix Phase Complete
- MEDIUM #1 fixed: resolution.rs SoulbondTrigger fizzle check now uses `calculate_characteristics` for layer-resolved creature-type check (lines ~3025, 3036)
- LOW #2 fixed: `SoulbondGrant` added to helpers.rs re-export list
- LOW #3 skipped: trigger-time partner search base-types issue (abilities.rs:2507,2578) — systemic pattern, deferred per instructions
- All 1641 tests pass after fixes

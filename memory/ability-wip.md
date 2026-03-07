# Ability WIP: Living Metal

ability: Living Metal
cr: 702.161
priority: P4
started: 2026-03-06
phase: close

## Review
findings: 2 (0 HIGH, 0 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-living-metal.md
plan_file: memory/abilities/ability-plan-living-metal.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1174, hash.rs:637, view_model.rs:844
- [x] 2. Rule enforcement — layers.rs:112-130 (Layer 4 inline, CR 702.161a)
- [x] 3. Trigger wiring — N/A (static ability, no triggers)
- [x] 4. Unit tests — crates/engine/tests/living_metal.rs (7 tests, all pass)
- [x] 5. Card definition — crates/engine/src/cards/defs/steel_guardian.rs (synthetic test Vehicle, 3/3 {2}, LivingMetal; all real Living Metal cards are DFCs blocked by Transform subsystem)
- [x] 6. Game script — test-data/generated-scripts/stack/166_living_metal_steel_guardian.json (declares as attacker to prove Creature type; "not creature on opponent's turn" covered by unit tests)
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Living Metal: validated)

## Notes
- CR corrected from 702.176 (Impending) to 702.161 (Living Metal) during plan phase.
- KW discriminant 128.
- No SOK or AbilDef discriminants needed (static continuous effect, no triggers).
- Layer 4 inline check, follows Impending pattern at layers.rs:86-110.
- No Layer 7b needed -- Vehicle already has printed P/T.

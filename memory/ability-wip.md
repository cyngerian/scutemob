# Ability WIP: Totem Armor

ability: Umbra Armor (formerly Totem Armor)
cr: 702.89
priority: P4
started: 2026-03-06
phase: close

## Review
findings: 5 (0 HIGH, 2 MEDIUM, 3 LOW)
review_file: memory/abilities/ability-review-umbra-armor.md
plan_file: memory/abilities/ability-plan-totem-armor.md

## Step Checklist
- [x] 1. Enum variant (state/types.rs:1174, state/hash.rs:636, tools/replay-viewer/src/view_model.rs:843)
- [x] 2. Rule enforcement (rules/replacement.rs:1696-1771, rules/sba.rs:411-424, effects/mod.rs:575-591, rules/events.rs:870-887, state/hash.rs:2839-2845)
- [x] 3. Trigger wiring (n/a — static replacement effect, no triggers)
- [x] 4. Unit tests (crates/engine/tests/umbra_armor.rs — 10 tests, all pass)
- [x] 5. Card definition — crates/engine/src/cards/defs/hyena_umbra.rs ({W} Aura, Enchant creature, +1/+1 + First Strike + UmbraArmor)
- [x] 6. Game script — test-data/generated-scripts/stack/165_umbra_armor_hyena_umbra.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Umbra Armor: validated, row renamed from Totem Armor)

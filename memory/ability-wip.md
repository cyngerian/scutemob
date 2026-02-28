# Ability WIP: Food tokens

ability: Food tokens
cr: 111.10b
priority: P3
started: 2026-02-28
phase: close
plan_file: memory/ability-plan-food.md

## Review
findings: 3 (1 MEDIUM, 2 LOW) — all fixed
review_file: memory/ability-review-food.md

## Step Checklist
- [x] 1. Enum variant / token type — card_definition.rs:763-770, definitions.rs:707/743/955/2174, builder.rs:581/638, hash.rs:2061, mod.rs:16, lib.rs:9
- [x] 2. Rule enforcement — effects/mod.rs:2013-2016 (make_token propagates activated_abilities); card_definition.rs:803-847 (food_token_spec)
- [x] 3. Trigger wiring — n/a (Food uses activated abilities, not triggers)
- [x] 4. Unit tests — crates/engine/tests/food_tokens.rs (11 tests, all passing)
- [x] 5. Card definition — Bake into a Pie (definitions.rs)
- [x] 6. Game script — test-data/generated-scripts/stack/097_bake_into_pie_food_token_gain_life.json
- [ ] 7. Coverage doc update

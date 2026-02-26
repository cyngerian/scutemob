# Ability WIP: Kicker

ability: Kicker
cr: 702.33
priority: P2
started: 2026-02-26
phase: closed
plan_file: memory/ability-plan-kicker.md

## Review
findings: 7 (0 HIGH, 1 MEDIUM, 6 LOW)
review_file: memory/ability-review-kicker.md
verdict: needs-fix

## Step Checklist
- [x] 1. Enum variant (state/types.rs:248, cards/card_definition.rs:152+581, state/stack.rs:51, state/game_object.rs:260, state/hash.rs, rules/command.rs:80)
- [x] 2. Rule enforcement (rules/casting.rs: get_kicker_cost, kicker validation, cost addition, StackObject population)
- [x] 3. Trigger wiring (rules/resolution.rs: kicker_times_paid to permanent + EffectContext; rules/replacement.rs: ETB trigger context; effects/mod.rs: Condition::WasKicked, EffectContext::new_with_kicker)
- [x] 4. Unit tests (crates/engine/tests/kicker.rs: 10 tests)
- [x] 5. Card definition — Burst Lightning (definitions.rs:2067), Torch Slinger (definitions.rs:2106)
- [x] 6. Game script — stack/065_burst_lightning_kicked_vs_unkicked.json
- [x] 7. Coverage doc update — Kicker: none→validated; P2 validated 5→6

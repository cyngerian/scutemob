# Ability WIP: Attack Trigger

ability: Attack Trigger
cr: 603
priority: P1
started: 2026-02-25
phase: closed
plan_file: memory/ability-plan-attack-trigger.md

## Step Checklist
- [x] 1. Enum variant — already done: TriggerCondition::WhenAttacks @ cards/card_definition.rs:470, TriggerEvent::SelfAttacks @ state/game_object.rs:111, hash support @ state/hash.rs:890,1830
- [x] 2. Rule enforcement — already done: GameEvent::AttackersDeclared @ rules/combat.rs:285-288, check_triggers SelfAttacks arm @ rules/abilities.rs:377-388, flush @ combat.rs:290-298
- [x] 3. Trigger wiring — WhenAttacks->SelfAttacks enrichment block added @ testing/replay_harness.rs:426-445
- [x] 4. Unit tests — 5 tests added @ crates/engine/tests/abilities.rs:1790-2253 (attack_trigger_fires_on_declare_attackers, via_card_definition_enrich_path, resolves_draws_card, does_not_fire_for_non_attacker, multiple_attackers)
- [x] 5. Card definition — Audacious Thief (`cards/definitions.rs:~1478`); WhenAttacks → draw+lose 1, 2/2, {2B}
- [x] 6. Game script — `test-data/generated-scripts/combat/011_audacious_thief_attack_trigger.json` (10/10 assertions pass)
- [x] 7. Coverage doc update — Attack trigger row: partial → validated; P1 validated 32→33; P1 partial 7→6

## Review
findings: 3 (0 HIGH, 1 MEDIUM, 2 LOW)
review_file: memory/ability-review-attack-trigger.md
verdict: fixed

## Fix Phase
- [x] Finding 1 (MEDIUM): Wrong CR citation in abilities.rs:378 — changed to `(CR 508.1m, CR 508.3a)` @ rules/abilities.rs:378
- [x] Finding 2 (LOW): Pre-existing test in combat.rs:744,748 — changed CR 603.5 to CR 508.3a @ tests/combat.rs:744,748
- [x] Finding 3 (LOW): WhenBlocks enrichment block added @ testing/replay_harness.rs after line 448

# Ability WIP: Collect Evidence

ability: Collect Evidence
cr: 701.59
priority: P4
started: 2026-03-07
phase: closed

## Review
findings: 0 HIGH/MEDIUM + 3 LOW (deferred)
review_file: memory/abilities/ability-review-collect-evidence.md
plan_file: memory/abilities/ability-plan-collect-evidence.md

## Step Checklist
- [x] 1. Enum variant — AbilityDefinition::CollectEvidence (disc 53), Condition::EvidenceWasCollected (disc 13); collect_evidence_cards on Command::CastSpell; evidence_collected on StackObject/GameObject/EffectContext; hash.rs updated — crates/engine/src/cards/card_definition.rs, crates/engine/src/state/stack.rs, crates/engine/src/state/game_object.rs, crates/engine/src/effects/mod.rs, crates/engine/src/state/hash.rs, crates/engine/src/rules/command.rs
- [x] 2. Rule enforcement — Validate in casting.rs: uniqueness, zone check (caster's GY), total MV >= threshold, mandatory check; payment: exile cards to ZoneId::Exile; propagate to StackObject.evidence_collected, resolution.rs propagates to EffectContext and GameObject; cast_spell_collect_evidence harness action added — crates/engine/src/rules/casting.rs, crates/engine/src/rules/resolution.rs, crates/engine/src/testing/replay_harness.rs, crates/engine/src/testing/script_schema.rs
- [x] 3. Trigger wiring — N/A (Collect Evidence is a keyword action / additional cost, not a trigger)
- [x] 4. Unit tests — crates/engine/tests/collect_evidence.rs (11 tests: basic exile, over-threshold, under-threshold, not-collected optional, insufficient single card, mandatory without cards, duplicate card, card not in GY, opponent's GY, spell without ability, mana not reduced); also fixed missing ctx.evidence_collected propagation in main resolution.rs path (line 327)
- [x] 5. Card definition — crates/engine/src/cards/defs/crimestopper_sprite.rs (Crimestopper Sprite, Collect Evidence 6)
- [x] 6. Game script — test-data/generated-scripts/etb-triggers/181_collect_evidence_crimestopper_sprite.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md: Collect Evidence → validated

# Ability WIP: The Ring Tempts You

ability: The Ring Tempts You
cr: 701.54
priority: P4 (W1-B16)
started: 2026-03-09
phase: implement
plan_file: memory/abilities/ability-plan-ring-tempts-you.md

## Step Checklist
- [x] 1. Data model — `ring_level: u8` + `ring_bearer_id: Option<ObjectId>` on PlayerState (state/player.rs:152+); `RING_BEARER = 1 << 8` on Designations (state/game_object.rs:37+); hash in state/hash.rs; serde defaults; builder.rs init
- [x] 2. Effect + events — `Effect::TheRingTemptsYou` (card_definition.rs); `Condition::RingHasTemptedYou(u8)` (card_definition.rs); `TriggerCondition::WheneverRingTemptsYou` (card_definition.rs); `GameEvent::RingTempted` + `GameEvent::RingBearerChosen` (events.rs); `handle_ring_tempts_you` in rules/engine.rs:2290+; execute_effect arm in effects/mod.rs; evaluate_condition arm; `StackObjectKind::RingAbility` (stack.rs:808); resolution arm (resolution.rs); TUI/replay-viewer SOK arms
- [x] 3. Layer + combat — ring-bearer Legendary supertype in layers.rs pre-loop section; blocking restriction in combat.rs validate_blocker + provoke section; SBA check in sba.rs (check_ring_bearer_sba); ring level 2/3/4 PendingTrigger dispatch in abilities.rs (check_triggers); `flush_pending_triggers` arms for RingLoot/RingBlockSacrifice/RingCombatDamage (abilities.rs:6609+)
- [x] 4. Unit tests — crates/engine/tests/ring_tempts_you.rs (13 tests passing)

## Discriminant chain used
- Effect discriminant 51: TheRingTemptsYou
- GameEvent discriminants 117 (RingTempted), 118 (RingBearerChosen)
- Condition discriminant 18: RingHasTemptedYou(u8)
- TriggerCondition discriminant 26: WheneverRingTemptsYou
- SOK discriminant 66: RingAbility
- PendingTriggerKind: RingLoot=47, RingBlockSacrifice=48, RingCombatDamage=49

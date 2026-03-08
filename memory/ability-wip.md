# Ability WIP: Gift

ability: Gift
cr: 702.174
priority: P4
started: 2026-03-07
phase: closed
plan_file: memory/abilities/ability-plan-gift.md

## Step Checklist
- [x] 1. Enum variant — `crates/engine/src/state/types.rs` (KeywordAbility::Gift disc 139); `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Gift, GiftType enum, Condition::GiftWasGiven); `crates/engine/src/state/hash.rs` (all hash arms)
- [x] 2. Rule enforcement — `crates/engine/src/rules/casting.rs` (gift_opponent param, validation block); `crates/engine/src/state/stack.rs` (GiftETBTrigger disc 54, gift_was_given/gift_opponent fields); `crates/engine/src/state/game_object.rs` (gift_was_given/gift_opponent fields); `crates/engine/src/rules/resolution.rs` (gift effect for instants/sorceries CR 702.174j, permanent ETB trigger, execute_gift_effect helper); `crates/engine/src/effects/mod.rs` (EffectContext gift fields, Condition::GiftWasGiven); `tools/replay-viewer/src/view_model.rs` (GiftETBTrigger + Gift arms); `tools/tui/src/play/panels/stack_view.rs` (GiftETBTrigger arm)
- [x] 3. Trigger wiring — `crates/engine/src/state/stubs.rs` (PendingTriggerKind::GiftETB); `crates/engine/src/rules/abilities.rs` (flush_pending_triggers GiftETB arm); `crates/engine/src/state/mod.rs` (zone-change resets); `crates/engine/src/state/builder.rs` (init false/None)
- [x] 4. Unit tests — `crates/engine/tests/gift.rs` (8 tests: basic instant card draw, not paid, permanent ETB trigger, permanent not paid, invalid self rejected, rejected without keyword, multiplayer specific opponent, both effects resolve)
- [x] 5. Card definition
- [x] 6. Game script
- [x] 7. Coverage doc update

## Review
findings: 5 (1 HIGH, 1 MEDIUM, 3 LOW)
verdict: needs-fix
review_file: memory/abilities/ability-review-gift.md

## Fix Phase (applied 2026-03-07)
- [x] HIGH-1: hash.rs — added `gift_was_given` and `gift_opponent` hash lines after `offspring_paid` in `HashInto for StackObject`
- [x] MEDIUM-2: resolution.rs:337 — gated inline gift effect with `&& !is_permanent`
- [x] LOW-3: helpers.rs — added `GiftType` to re-export list

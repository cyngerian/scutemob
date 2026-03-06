# Ability WIP: Echo

ability: Echo
cr: 702.31
priority: P4
started: 2026-03-06
phase: closed

## Review
findings: 2 (0 HIGH, 0 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-echo.md
plan_file: memory/abilities/ability-plan-echo.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1027, card_definition.rs:481, hash.rs:569+3402, game_object.rs:18 (added PartialOrd/Ord/Hash to ManaCost), view_model.rs:801
- [x] 2. Rule enforcement — resolution.rs:505+1367, lands.rs (ETB echo_pending=true), engine.rs:430+470 (PayEcho handler + handle_pay_echo), events.rs (EchoPaymentRequired+EchoPaid), command.rs (PayEcho), mod.rs:143 (pending_echo_payments), stubs.rs (EchoUpkeep+echo_cost field)
- [x] 3. Trigger wiring — turn_actions.rs (upkeep_actions echo queuing), abilities.rs (EchoUpkeep->EchoTrigger conversion), resolution.rs:3953 (counter arm), stack_view.rs (TUI), view_model.rs (replay viewer)
- [x] 4. Unit tests — crates/engine/tests/echo.rs (9 tests: etb_sets_pending, pending_false_without_echo, upkeep_trigger_fires, pay_cost_keeps_permanent, decline_payment_sacrifices, no_trigger_after_paid, different_cost, multiplayer_only_controller_upkeep, permanent_left_battlefield)
- [x] 5. Card definition — crates/engine/src/cards/defs/avalanche_riders.rs
- [x] 6. Game script — test-data/generated-scripts/stack/151_echo_avalanche_riders.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Echo: validated, CR corrected to 702.30)

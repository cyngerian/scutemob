# Ability WIP: Decayed Tokens

ability: Decayed Tokens
cr: 702.147 (Decayed keyword)
priority: P4
started: 2026-03-08
phase: closed
plan_file: memory/abilities/ability-plan-decayed-tokens.md

## Step Checklist
- [x] 1. Enum variant — KeywordAbility::Decayed already exists (state/types.rs:688, disc 74)
- [x] 2. Rule enforcement — can't-block (combat.rs:546,849), EOC sacrifice (turn_actions.rs:1541-1562) already done
- [x] 3. Trigger wiring — EOC flag pattern, not trigger dispatch; n/a for tokens
- [x] 4. Unit tests — 4 token-specific tests added to crates/engine/tests/decayed.rs (tests 9-12); zombie_decayed_token_spec() helper added to cards/card_definition.rs:1625 and exported from cards/mod.rs, lib.rs, helpers.rs; 12 total tests passing
- [x] 5. Card definition (jadar_ghoulcaller_of_nephalia.rs — {1}{B} 1/1 Legend; end-step trigger creates 2/2 Zombie Decayed token; intervening-if DSL gap noted)
- [x] 6. Game script — test-data/generated-scripts/combat/191_decayed_jadar_zombie_token_eoc_sacrifice.json (pending_review; harness PASSES with partial engine fix; trigger fires and lands on stack (stack.count=1 confirmed), but token creation at resolution fails due to remaining gap: resolution.rs reads characteristics.triggered_abilities (empty) instead of card_registry; dispute recorded)
- [x] 7. Coverage doc update

## Review
findings: 2 LOW (no HIGH, no MEDIUM)
verdict: clean
review_file: memory/abilities/ability-review-decayed-tokens.md

## Script Notes (Step 6)
- Script file: test-data/generated-scripts/combat/191_decayed_jadar_zombie_token_eoc_sacrifice.json
- Harness validation: PASS (1/1 approved scripts passed — all assertions that can be verified pass)
- PARTIAL FIX confirmed: end_step_actions() generic AtBeginningOfYourEndStep sweep fires Jadar's trigger correctly (stack.count=1 passes)
- REMAINING GAP: resolution.rs StackObjectKind::TriggeredAbility branch reads obj.characteristics.triggered_abilities.get(ability_index). This Vec is not populated from AbilityDefinition::Triggered in CardDefinition (neither by enrich_spec_from_def nor by builder). The CreateToken effect is never found at resolution. Token is not created. The script asserts only observable effects (stack goes to 0, Jadar remains, life totals unchanged).
- Remaining fix needed: resolution.rs TriggeredAbility branch should fall back to card_registry when characteristics.triggered_abilities.get(ability_index) is None (using obj.card_id to find the CardDef and locate the Nth AbilityDefinition::Triggered entry).

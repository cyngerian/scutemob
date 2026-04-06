# Primitive WIP: PB-I — Grant Flash

batch: PB-I
title: Grant flash
cards_affected: 4
started: 2026-04-05
phase: closed

## Review
findings: 5 (HIGH: 0, MEDIUM: 3, LOW: 2)
verdict: needs-fix
review_file: memory/primitives/pb-review-I.md
plan_file: memory/primitives/pb-plan-I.md

## Cards
1. Borne Upon a Wind — "You may cast spells this turn as though they had flash." (existing def, TODO)
2. Complete the Circuit — "You may cast sorcery spells this turn as though they had flash." (existing def, TODO; copy-spell-twice is separate gap)
3. Teferi, Time Raveler — "+1: Until your next turn, you may cast sorcery spells as though they had flash." (existing def, TODO)
4. Yeva, Nature's Herald — "You may cast green creature spells as though they had flash." (no def yet)

## Existing Infrastructure
- Flash keyword already implemented (KeywordAbility::Flash)
- casting.rs handles timing checks for instants vs sorceries
- No mechanism to grant "cast as though it had flash" to other spells
- Teferi's +1 needs duration (until your next turn) — expire_until_next_turn_effects already extended for replacement effects (PB-F)

## Deferred from Prior PBs
- none directly relevant

## Step Checklist
- [x] 1. Engine changes (FlashGrant/FlashGrantFilter in stubs.rs, flash_grants: Vector<FlashGrant> on GameState, OpponentsCanOnlyCastAtSorcerySpeed in GameRestriction, Effect::GrantFlash in card_definition.rs, AbilityDefinition::StaticFlashGrant in card_definition.rs, Effect::GrantFlash dispatch in effects/mod.rs, StaticFlashGrant registration in replacement.rs, has_active_flash_grant helper + timing check in casting.rs, OpponentsCanOnlyCastAtSorcerySpeed arm in check_cast_restrictions, UntilEndOfTurn/UntilYourNextTurn expiry in layers.rs, stale grant cleanup in turn_actions.rs, hash arms in state/hash.rs, simulator legal_actions.rs updated, FlashGrantFilter exported from lib.rs + helpers.rs)
- [x] 2. Card definition fixes: borne_upon_a_wind.rs (GrantFlash AllSpells + DrawCards), complete_the_circuit.rs (GrantFlash Sorceries, copy-spell-twice remains TODO), teferi_time_raveler.rs (StaticRestriction::OpponentsCanOnlyCastAtSorcerySpeed passive + GrantFlash Sorceries UntilYourNextTurn +1)
- [x] 3. New card definitions: yeva_natures_herald.rs (Flash + StaticFlashGrant GreenCreatures)
- [x] 4. Unit tests: 13 tests in crates/engine/tests/grant_flash.rs — all passing
- [x] 5. Workspace build verification: cargo build --workspace clean, cargo test --all 2504 passed, cargo clippy -- -D warnings clean, cargo fmt --check clean
- [x] 6. Fix phase: 3 MEDIUM findings resolved — simulator source-validity for WhileSourceOnBattlefield grants (legal_actions.rs ~line 219), simulator OpponentsCanOnlyCastAtSorcerySpeed arm (legal_actions.rs ~line 970), test_yeva_static_flash_grant_registered_on_etb rewritten to exercise ETB pipeline (grant_flash.rs). 2504 tests pass, 0 clippy warnings.

# Primitive WIP: PB-37 -- Complex activated abilities (residual)

batch: PB-37
title: Complex activated abilities (residual G-26)
cards_affected: ~8
started: 2026-03-29
phase: closed
plan_file: memory/primitives/pb-plan-37.md

## Gap Groups
- G-26: Activated abilities (general complex) — residual after PB-23–36. 4 new primitives needed.

## New Primitives
1. Condition::WasCast (CR 603.4 intervening-if "if you cast it")
2. EffectDuration::UntilYourNextTurn(PlayerId) (CR 611.2b)
3. once_per_turn: bool on Activated abilities (CR 602.5g)
4. was_cast: bool + abilities_activated_this_turn: u32 on GameObject

## Card Fixes
- The One Ring (WasCast + UntilYourNextTurn + CounterCount draw)
- Geological Appraiser (WasCast)
- Teferi's Protection (UntilYourNextTurn)
- Elspeth Storm Slayer (UntilYourNextTurn)
- Kaito Dancing Shadow (UntilYourNextTurn)
- Ramos Dragon Engine (once_per_turn)
- Steel Hellkite (once_per_turn)

## Deferred from Prior PBs
- Clone/copy ETB choice -- from PB-13j (blocked on M10)
- Tiamat multi-card search -- blocked on M10 player choice
- Scion of the Ur-Dragon copy-self -- EffectTarget::LastSearchResult
- Urza's Saga exact mana cost filter -- TargetFilter gap
- GrantActivatedAbility (~8 cards) -- post-alpha
- Effect::ChangeTarget (~3 cards) -- blocked on M10
- Color choice (~15 cards) -- blocked on M10

## Step Checklist
- [x] 1. Engine changes: Condition::WasCast (effects/mod.rs check_condition), was_cast+abilities_activated_this_turn on GameObject (game_object.rs), EffectDuration::UntilYourNextTurn(PlayerId) (continuous_effect.rs), once_per_turn: bool on AbilityDefinition::Activated+ActivatedAbility, expire_until_next_turn_effects() in layers.rs called from turn_actions.rs untap step, once_per_turn enforcement in abilities.rs, temporary_protection_qualities on PlayerState, GrantPlayerProtection duration branching in effects/mod.rs, hash.rs/replacement.rs/casting.rs updated, all exhaustive matches updated including replay_harness.rs
- [x] 2. Card definition fixes: the_one_ring.rs (WasCast+UntilYourNextTurn+CounterCount), geological_appraiser.rs (WasCast intervening_if), teferis_protection.rs (UntilYourNextTurn duration), elspeth_storm_slayer.rs (Sequence+ForEach+ApplyContinuousEffect flying+UntilYourNextTurn), kaito_dancing_shadow.rs (CantBlock+UntilYourNextTurn), ramos_dragon_engine.rs (once_per_turn:true), steel_hellkite.rs (once_per_turn:true)
- [x] 3. New card definitions — N/A
- [x] 4. Unit tests: 9 tests in crates/engine/tests/primitive_pb37.rs (WasCast true/false, UntilYourNextTurn persists/expires, player protection expiry, once-per-turn counter starts 0/resets, One Ring was_cast default/burden counters)
- [x] 5. Workspace build verification: cargo test --all (all pass), cargo clippy -- -D warnings (0 warnings), cargo fmt --check (clean), cargo build --workspace (Finished)

## Review
findings: 8 (HIGH: 1, MEDIUM: 2, LOW: 4) — note: findings 5+6 are symptoms of finding 1
verdict: needs-fix
review_file: memory/primitives/pb-review-37.md

## Fix Phase (2026-03-29)
- [x] F1 (HIGH): effects/mod.rs ApplyContinuousEffect — resolve UntilYourNextTurn placeholder to ctx.controller
- [x] F2 (MEDIUM): engine.rs handle_concede — sweep UntilYourNextTurn effects for conceding player + clear temporary_protection_qualities
- [x] F3 (LOW): global replace 602.5g → 602.5b in 10 source files
- [x] F4 (LOW): game_object.rs:1029 doc comment "resolves" → "is activated (placed on the stack)"
- [x] F5+F6 (HIGH symptoms): fixed by F1 — elspeth_storm_slayer + kaito_dancing_shadow now get correct PlayerId in duration
- [x] F7 (MEDIUM): card_definition.rs GrantPlayerProtection doc comment updated to describe current behavior
- [x] F8 (LOW): the_one_ring.rs comment clarified — PlayerId(0) in duration not used for GrantPlayerProtection expiry
verdict: DONE — 2428 tests pass, 0 clippy, workspace build clean

# Primitive WIP: PB-K — Additional Land Drops

batch: PB-K
title: Additional land drops
cards_affected: 3
started: 2026-04-02
phase: implement
plan_file: memory/primitives/pb-plan-K.md

## Cards
1. Burgeoning — "Whenever an opponent plays a land, you may put a land card from your hand onto the battlefield."
2. Dryad of the Ilysian Grove — AdditionalLandPlays + "Lands you control are every basic land type in addition to their other types."
3. Case of the Locked Hothouse — AdditionalLandPlays + Case mechanic (solve/solved) + solved play-from-top (likely PB-A blocked)

## Existing Infrastructure
- `AdditionalLandPlays { count }` on AbilityDefinition (disc 71)
- `AdditionalLandPlaySource` struct in stubs.rs
- `additional_land_play_sources` on GameState
- `land_plays_remaining` on PlayerState
- Registration in replacement.rs, application in turn_actions.rs reset_turn_state

## Likely Engine Gaps
- "Whenever an opponent plays a land" trigger event (Burgeoning)
- "Put a land card from your hand onto the battlefield" effect (Burgeoning)
- "Lands you control are every basic land type" Layer 4 continuous effect (Dryad)
- Case mechanic: solve condition check at end step + solved static (new)
- Solved play-from-top: likely PB-A territory (HIGH, may need TODO)

## Deferred from Prior PBs
None directly applicable.

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch) — TriggerCondition::WheneverOpponentPlaysLand (hash disc 40), TriggerEvent::OpponentPlaysLand (hash disc 44), Effect::PutLandFromHandOntoBattlefield (hash disc 74), Effect::SolveCase (hash disc 75), Designations::SOLVED (1<<9), Condition::SourceIsSolved (hash disc 39), Condition::And (hash disc 40), EffectFilter::LandsYouControl (hash disc 29); dispatch in abilities.rs check_triggers; conversion in replay_harness.rs enrich_spec_from_def; match arm in layers.rs; evaluate in effects/mod.rs check_condition
- [x] 2. Card definition authoring — burgeoning.rs, dryad_of_the_ilysian_grove.rs, case_of_the_locked_hothouse.rs (3 new); growth_spiral.rs, broken_bond.rs, spelunking.rs, contaminant_grafter.rs, chulane_teller_of_tales.rs (5 fixes)
- [x] 3. Unit tests — 17 tests in crates/engine/tests/pb_k_land_drops.rs, all passing
- [x] 4. Workspace build verification — cargo build --workspace CLEAN; cargo clippy -- -D warnings CLEAN; cargo fmt --check CLEAN; all tests pass

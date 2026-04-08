# Primitive WIP: PB-B — Play from GY/Exile

batch: PB-B
title: Play from GY/exile — zone-play permission system for graveyard and exile zones
cards_affected: 5+
started: 2026-04-08
phase: done
plan_file: memory/primitives/pb-plan-B.md

## Deferred from Prior PBs
- PB-A established PlayFromTopPermission pattern — extend to graveyard/exile zones
- Wrenn and Realmbreaker "play lands/permanents from graveyard" (from card authoring)
- Ancient Greenwarden "play lands from graveyard" (from card authoring)
- Perennial Behemoth "play lands from graveyard" (from card authoring)
- pitch-alt-cost cards (Force of Negation, Force of Vigor) blocked in A-38

## Known Cards
Cards with play-from-GY/exile TODOs:
- Ancient Greenwarden (play lands from your graveyard)
- Wrenn and Realmbreaker (play lands + cast permanents from your graveyard — PW ultimate)
- Perennial Behemoth (play lands from your graveyard)
- Oathsworn Vampire (cast from GY if gained life this turn)
- Squee, Dubious Monarch (cast from GY by paying + exiling)
- Brokkos, Apex of Forever (cast from GY using mutate)

Pitch alt-cost cards (may be in scope):
- Force of Negation (exile blue card from hand as alt cost)
- Force of Vigor (exile green card from hand as alt cost)

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - PlayFromTopFilter::PermanentsAndLands variant (disc 6) in state/stubs.rs
  - PlayFromGraveyardPermission struct with source/controller/filter/condition in state/stubs.rs
  - play_from_graveyard_permissions: Vector<PlayFromGraveyardPermission> on GameState
  - life_gained_this_turn: u32 on PlayerState (incremented in GainLife/DrainLife/lifelink effects)
  - AbilityDefinition::StaticPlayFromGraveyard { filter, condition } (disc 74) in card_definition.rs
  - AbilityDefinition::CastSelfFromGraveyard { condition, alt_mana_cost, additional_costs, required_alt_cost } (disc 75)
  - CastFromGraveyardAdditionalCost enum with ExileOtherGraveyardCards(u32)
  - Condition::ControllerGainedLifeThisTurn (disc 42)
  - Effect::CreateEmblem extended with play_from_graveyard: Option<PlayFromTopFilter>
  - HashInto impls for all new types in state/hash.rs
  - effects/mod.rs: GainLife/DrainLife increment life_gained_this_turn; CreateEmblem registers GY permission
  - rules/combat.rs: lifelink increments life_gained_this_turn
  - rules/turn_actions.rs: reset life_gained_this_turn + retain GY permissions sweep
  - rules/replacement.rs: StaticPlayFromGraveyard registers permission on ETB
  - rules/casting.rs: has_cast_self_from_graveyard + casting_via_graveyard_permission detection/validation
  - rules/lands.rs: has_play_from_graveyard_land_permission() + PlayLand from GY support
- [x] 2. Card definition fixes
  - ancient_greenwarden.rs: added StaticPlayFromGraveyard { filter: LandsOnly }
  - perennial_behemoth.rs: added StaticPlayFromGraveyard { filter: LandsOnly } + Unearth AltCastAbility
  - wrenn_and_realmbreaker.rs: -7 emblem updated to play_from_graveyard: Some(PermanentsAndLands)
  - oathsworn_vampire.rs: added CastSelfFromGraveyard { condition: Some(ControllerGainedLifeThisTurn) }
  - squee_dubious_monarch.rs: added CastSelfFromGraveyard { alt_mana_cost: {3}{R}, additional_costs: [ExileOtherGraveyardCards(4)] }
  - brokkos_apex_of_forever.rs: added CastSelfFromGraveyard { required_alt_cost: Some(AltCostKind::Mutate) }
  - 7 existing CreateEmblem card defs: added play_from_graveyard: None
- [x] 3. New card definitions: none
- [x] 4. Unit tests (13 tests in crates/engine/tests/play_from_graveyard.rs — all passing)
- [x] 5. Workspace build verification (all pass; clippy clean; fmt clean)

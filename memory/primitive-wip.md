# Primitive WIP: PB-M — Panharmonicon Trigger Doubling

batch: PB-M
title: Panharmonicon trigger doubling — wire existing infrastructure + card def
cards_affected: 1 (Panharmonicon) + related cards with trigger doubling TODOs
started: 2026-04-09
phase: closed
plan_file: memory/primitives/pb-plan-M.md

## Deferred from Prior PBs
- SelfEntersBattlefield triggers (PartnerWith, Hideaway, Exploit) NOT doubled — doubler_applies_to_trigger only matches AnyPermanentEntersBattlefield (documented in MEMORY.md)
- Ancient Greenwarden: land-ETB trigger doubling TODO (needs land-filtered TriggerDoublerFilter)
- Delney, Streetwise Lookout: power-filtered trigger doubling TODO

## Existing Infrastructure
- TriggerDoubler struct (source, controller, filter, additional_triggers) in state/stubs.rs
- TriggerDoublerFilter enum (ArtifactOrCreatureETB, CreatureDeath) in state/stubs.rs
- trigger_doublers: Vector<TriggerDoubler> on GameState
- AbilityDefinition::TriggerDoubling { filter } in card_definition.rs
- compute_trigger_doubling() and doubler_applies_to_trigger() in abilities.rs
- Registration in replacement.rs (ReplacementModification::RegisterTriggerDoubler)

## Step Checklist
- [x] 1. Engine changes
  - Fixed `doubler_applies_to_trigger` (abilities.rs): `ArtifactOrCreatureETB` arm now also matches `SelfEntersBattlefield` (Bug 1)
  - Fixed `queue_carddef_etb_triggers` (replacement.rs): both ETB trigger push sites now set `entering_object_id: Some(new_id)` (Bug 2)
  - Added `AnyPermanentETB` and `LandETB` variants to `TriggerDoublerFilter` (stubs.rs)
  - Added match arms for new variants in `doubler_applies_to_trigger` (abilities.rs)
  - Added hash discriminants 2 and 3 for new variants (hash.rs)
- [x] 2. Card definition: created panharmonicon.rs (crates/engine/src/cards/defs/panharmonicon.rs)
- [x] 3. Related card def fixes
  - drivnod_carnage_dominus.rs: added `TriggerDoubling { filter: CreatureDeath, additional_triggers: 1 }`
  - elesh_norn_mother_of_machines.rs: added `TriggerDoubling { filter: AnyPermanentETB, additional_triggers: 1 }` (opponent suppression still TODO — separate DSL gap)
  - ancient_greenwarden.rs: added `TriggerDoubling { filter: LandETB, additional_triggers: 1 }`, removed deferred TODO
- [x] 4. Unit tests (6 new tests in trigger_doubling.rs — 8 total, all pass)
  - test_panharmonicon_doubles_self_etb_trigger (Bug 1 fix: SelfEntersBattlefield)
  - test_panharmonicon_does_not_double_enchantment_etb (negative: type filter)
  - test_any_permanent_etb_doubler_doubles_enchantment (AnyPermanentETB variant)
  - test_land_etb_doubler_doubles_landfall_not_creature (LandETB variant, negative for creature)
- [x] 5. Workspace build verification
  - `cargo test --all`: all tests pass (0 failures)
  - `cargo clippy -- -D warnings`: 0 warnings
  - `cargo build --workspace`: clean (replay-viewer + TUI compile, no new exhaustive match gaps)
  - `cargo fmt --check`: clean

## Fix Phase (pb-review-M.md)
- [x] HIGH-1: Removed `SuperType::Legendary` from ancient_greenwarden.rs line 20 — oracle text confirms "Creature — Elemental" only
- [x] MEDIUM-2: Added `test_panharmonicon_doubles_carddef_etb_trigger` to trigger_doubling.rs — exercises `queue_carddef_etb_triggers` path specifically (CardDef-based ETB via `AbilityDefinition::Triggered`, not ObjectSpec runtime trigger); test passes

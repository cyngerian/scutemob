# Primitive WIP: PB-19 -- Mass Destroy / Board Wipes

batch: PB-19
title: Mass Destroy / Board Wipes
cards_affected: 12
started: 2026-03-19
phase: implement
plan_file: memory/primitives/pb-plan-19.md

## Deferred from Prior PBs
none

## Step Checklist
- [x] 1. Engine changes: Effect::DestroyAll, Effect::ExileAll, EffectAmount::LastEffectCount, AddCounterAmount, last_effect_count on EffectContext, AllPermanentsMatching controller filter fix. Files: card_definition.rs, effects/mod.rs, state/hash.rs
- [x] 2. Card definition fixes: wrath_of_god.rs, damnation.rs, supreme_verdict.rs, path_of_peril.rs (max_cmc filter), sublime_exhalation.rs, final_showdown.rs mode 2, scavenger_grounds.rs (implemented activated ability). Updated script 145_cleave_path_of_peril.json for corrected path_of_peril behavior.
- [x] 3. New card definitions: vanquish_the_horde.rs, fumigate.rs, bane_of_progress.rs, ruinous_ultimatum.rs, cyclonic_rift.rs
- [x] 4. Unit tests: 13 tests in crates/engine/tests/mass_destroy.rs — all passing
- [x] 5. Workspace build verification: cargo build --workspace clean; cargo clippy -- -D warnings clean; cargo test --all: 2175 tests, 0 failures

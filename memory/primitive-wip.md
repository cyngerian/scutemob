# Primitive WIP: PB-19 -- Mass Destroy / Board Wipes

batch: PB-19
title: Mass Destroy / Board Wipes
cards_affected: 12
started: 2026-03-19
phase: fix
plan_file: memory/primitives/pb-plan-19.md

## Deferred from Prior PBs
none

## Step Checklist
- [x] 1. Engine changes: Effect::DestroyAll, Effect::ExileAll, EffectAmount::LastEffectCount, AddCounterAmount, last_effect_count on EffectContext, AllPermanentsMatching controller filter fix. Files: card_definition.rs, effects/mod.rs, state/hash.rs
- [x] 2. Card definition fixes: wrath_of_god.rs, damnation.rs, supreme_verdict.rs, path_of_peril.rs (max_cmc filter), sublime_exhalation.rs, final_showdown.rs mode 2, scavenger_grounds.rs (implemented activated ability). Updated script 145_cleave_path_of_peril.json for corrected path_of_peril behavior.
- [x] 3. New card definitions: vanquish_the_horde.rs, fumigate.rs, bane_of_progress.rs, ruinous_ultimatum.rs, cyclonic_rift.rs
- [x] 4. Unit tests: 13 tests in crates/engine/tests/mass_destroy.rs — all passing
- [x] 5. Workspace build verification: cargo build --workspace clean; cargo clippy -- -D warnings clean; cargo test --all: 2175 tests, 0 failures

## Review
findings: 4 (HIGH: 0, MEDIUM: 2, LOW: 2)
verdict: needs-fix
review_file: memory/primitives/pb-review-19.md

## Fix Phase (2026-03-19)
- [x] Finding 1 (MEDIUM): DestroyAll ZoneId::Command arm now increments destroyed_count per CR 903.9a. effects/mod.rs:897-900
- [x] Finding 2 (LOW): ExileAll ObjectExiled events now use `owner` instead of `ctx.controller` in both Redirect and Proceed arms. effects/mod.rs:1017,1049
- [x] Finding 3 (MEDIUM): Added PlayerTarget::OwnerOf(Box<EffectTarget>) variant to card_definition.rs; wired in resolve_player_target_list (effects/mod.rs); added hash discriminant 5 in state/hash.rs; updated Manifest/Cloak match arms; updated cyclonic_rift.rs to use OwnerOf for both normal and overloaded modes.
- [x] Finding 4 (LOW): No action — final_showdown.rs modes 0/1 TODOs are pre-existing and deferred.
- [x] Post-fix verification: cargo build --workspace clean; cargo clippy -- -D warnings clean; cargo test --all: 2175 tests, 0 failures

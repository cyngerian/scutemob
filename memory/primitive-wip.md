# Primitive WIP: PB-H — Mass Reanimate

batch: PB-H
title: Mass reanimate
cards_affected: 5
started: 2026-04-06
phase: closed
plan_file: memory/primitives/pb-plan-H.md

## Review
findings: 4 (HIGH: 0, MEDIUM: 1, LOW: 3)
verdict: needs-fix
review_file: memory/primitives/pb-review-H.md

## Deferred from Prior PBs
- Nexus of Fate "from anywhere" graveyard replacement only covers resolution case (needs full non-permanent replacement infrastructure)

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch) — added Effect::ReturnAllFromGraveyardToBattlefield and Effect::LivingDeath to card_definition.rs; dispatch in effects/mod.rs; hash arms in state/hash.rs (discriminants 79/80)
- [x] 2. Card definition fixes — splendid_reclamation.rs, open_the_vaults.rs, eerie_ultimatum.rs, world_shaper.rs all updated
- [x] 3. New card definitions — living_death.rs created (auto-discovered by build.rs)
- [x] 4. Unit tests — 14 tests in mass_reanimate.rs, all passing
- [x] 5. Workspace build verification — cargo build --workspace, cargo test --all, cargo clippy -- -D warnings, cargo fmt all pass

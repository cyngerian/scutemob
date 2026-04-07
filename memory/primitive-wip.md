# Primitive WIP: PB-L — Reveal/X Effects

batch: PB-L
title: Reveal/X effects
cards_affected: 7
started: 2026-04-07
phase: closed
plan_file: memory/primitives/pb-plan-L.md

## Review
findings: 6 (HIGH: 2, MEDIUM: 4, LOW: 0)
verdict: needs-fix
review_file: memory/primitives/pb-review-L.md

## Deferred from Prior PBs
- none directly relevant

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch) — DomainCount + CommanderFreeCast + harness action done; cargo check clean
- [x] 2. Card definition fixes — coiling_oracle (RevealAndRoute), bounty_of_skemfar (TODO→NOTE), fierce_guardianship/deadly_rollick/flawless_maneuver (CommanderFreeCast AltCastAbility)
- [x] 3. New card definitions — allied_strategies.rs (domain draw), territorial_maro.rs (domain CDA */*)
- [x] 4. Unit tests — 11 tests in domain_and_freecast.rs: DomainCount (5), Territorial Maro CDA (2), CommanderFreeCast (3), Coiling Oracle ETB (2)
- [x] 5. Workspace build verification — cargo test --all: all pass; cargo clippy: clean; cargo build --workspace: clean; cargo fmt: clean

## Fix Phase Checklist (review findings)
- [x] H1/H2: DomainCount { player: PlayerTarget } — variant changed in card_definition.rs, effects/mod.rs, layers.rs (CDA path), hash.rs; allied_strategies uses DeclaredTarget { index: 0 }; territorial_maro uses Controller; 8 DomainCount usages in tests updated
- [x] M1/M4: Doc comment on DomainCount variant updated to clarify effect path vs CDA path distinction
- [x] M2: bounty_of_skemfar oracle_text fixed to include "up to one" in both clauses
- [x] M3: test_domain_count_dual_land added — Layer 4 continuous effect grants all 5 basic land subtypes to one land; verifies domain = 5 via resolve_amount
- [x] Final: cargo test --all: all 12 domain tests pass, full suite clean; clippy clean; build --workspace clean; fmt clean

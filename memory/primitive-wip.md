# Primitive WIP: PB-22 S5 -- Copy/Clone Primitives

batch: PB-22
session: S5
title: Copy/Clone Primitives (BecomeCopyOf, CreateTokenCopy)
cards_affected: 4 (Scion of the Ur-Dragon, Thespian's Stage, Shifting Woodland, Thousand-Faced Shadow)
started: 2026-03-21
phase: closed
plan_file: memory/primitives/pb-22-session-plan.md (Session 5 section)

## Deferred from Prior PBs
- Clone/copy ETB choice (PB-13j) — 2 cards (partial, S5 covers become-copy but not ETB choose-what-to-copy)
- Scion of the Ur-Dragon copy-self (PB-17) — partially covered (search works, copy needs EffectTarget::LastSearchResult)

## Step Checklist
- [x] 1. Engine changes: Effect::BecomeCopyOf (Layer 1 continuous effect, configurable EffectDuration)
  - Added Effect::BecomeCopyOf { copier, target, duration } in card_definition.rs
  - Dispatch in effects/mod.rs: resolves targets, verifies copier on BF, creates CopyOf CE with specified duration
  - GameEvent::BecameCopyOf { copier, source } (discriminant 123)
  - Hash discriminant 64 (Effect), 123 (GameEvent)
- [x] 2. Engine changes: Effect::CreateTokenCopy (token copy + optional tapped-and-attacking)
  - Added Effect::CreateTokenCopy { source, enters_tapped_and_attacking } in card_definition.rs
  - Dispatch in effects/mod.rs: creates blank token, applies CopyOf CE, registers combat if attacking
  - Applies token replacement effects, tracks last_created_permanent
  - Hash discriminant 65
- [x] 3. Bonus: Condition::CardTypesInGraveyardAtLeast(u32) for Delirium activation conditions
  - Added to Condition enum, check_condition dispatch, hash discriminant 31
  - check_condition made pub (was pub(crate)) for external test access
- [x] 4. Card definition fixes
  - Thespian's Stage: full {2},{T} copy ability with BecomeCopyOf (Indefinite duration, target land)
  - Shifting Woodland: full Delirium {2}{G}{G} copy ability (UntilEndOfTurn, activation condition)
  - Thousand-Faced Shadow: ETB trigger with CreateTokenCopy (tapped and attacking)
  - Scion of the Ur-Dragon: TODO improved (search works, copy needs LastSearchResult target)
- [x] 5. Unit tests (5 new tests in copy_effects.rs)
  - test_effect_become_copy_of: BecomeCopyOf via execute_effect, verifies BecameCopyOf event + layer resolution
  - test_effect_become_copy_reverts_at_eot: UntilEndOfTurn copy reverts after cleanup
  - test_effect_create_token_copy: CreateTokenCopy creates token with copied characteristics
  - test_effect_create_token_copy_tapped_attacking: token enters tapped+attacking in combat
  - test_delirium_condition_evaluation: CardTypesInGraveyardAtLeast true/false checks
- [x] 6. Workspace build verification
  - cargo test --all: 2265 tests pass (was 2260, +5 new)
  - cargo clippy -- -D warnings: clean
  - cargo build --workspace: clean
  - cargo fmt --check: clean

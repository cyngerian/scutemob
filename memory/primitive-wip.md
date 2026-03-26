# Primitive WIP: PB-31 -- Cost primitives (RemoveCounter, AdditionalSacrificeCost)

batch: PB-31
title: Cost primitives (RemoveCounter, AdditionalSacrificeCost)
cards_affected: ~23
started: 2026-03-26
phase: fix
plan_file: memory/primitives/pb-plan-31.md

## Gap Reference
G-16 from `docs/dsl-gap-closure-plan.md`:
- G-16: Cost::RemoveCounter (~16 cards) — New `Cost::RemoveCounter { counter: CounterType, count: u32 }` variant

G-17 from `docs/dsl-gap-closure-plan.md`:
- G-17: AdditionalCost::SacrificeCreature for spells (~7 cards) — Extend `AdditionalCost` vec on `CastSpell` — creature sacrifice at cast time

## Deferred from Prior PBs
- PB-30 deferred: Heartstone, Training Grounds, Silver-Fur Master, Puresteel Paladin, Morophon (to PB-37)
- PB-4 had sacrifice-as-activation-cost; PB-31 extends to sacrifice-as-casting-cost

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - Added `Cost::RemoveCounter { counter: CounterType, count: u32 }` to `card_definition.rs` (~line 918)
  - Added `SpellAdditionalCost` enum to `card_definition.rs` (SacrificeCreature, SacrificeLand, SacrificeArtifactOrCreature, SacrificeSubtype(SubType), SacrificeColorPermanent(Color))
  - Added `spell_additional_costs: Vec<SpellAdditionalCost>` to `CardDefinition` struct
  - Added `remove_counter_cost: Option<(CounterType, u32)>` to `ActivationCost` in `game_object.rs`
  - Updated `hash.rs`: `Cost::RemoveCounter` arm (discriminant 9), `ActivationCost::remove_counter_cost` field, `(A, B)` tuple `HashInto` impl
  - Wired `Cost::RemoveCounter` in `flatten_cost_into()` in `replay_harness.rs`
  - Wired `sacrifice_card_name` in `cast_spell` handler in `replay_harness.rs`
  - Added counter-removal cost validation + payment in `handle_activate_ability()` in `abilities.rs`
  - Added spell additional sacrifice cost validation in `casting.rs` (`spell_sac_id` block)
  - Added spell sacrifice cost execution in `casting.rs` (before bargain execution block)
  - Exported `SpellAdditionalCost` from `cards/mod.rs` and `lib.rs`
  - Updated all explicit `ActivationCost {}` struct constructors (3 in card_definition.rs, 1 in replay_harness.rs, ~20+ in tests) to include `remove_counter_cost: None`
  - Updated all explicit `CardDefinition {}` struct constructors in defs/ (~139 files) and tests/ (~16 files) to include `spell_additional_costs: vec![]`

- [x] 2. Card definition fixes (G-16: 8 cards; G-17: 9 cards)
  - G-16 (Cost::RemoveCounter):
    - dragons_hoard.rs — added {T}, Remove gold counter: Draw a card
    - spawning_pit.rs — added Sacrifice creature: charge counter; {1}, Remove 2 charge: create Spawn token
    - ominous_seas.rs — added Remove 8 foreshadow counters: Create 8/8 Kraken token
    - gemstone_array.rs — added Remove charge counter: Add mana of any color
    - golgari_grave_troll.rs — added {1}, Remove +1/+1 counter: Regenerate
    - ghave_guru_of_spores.rs — added ETB 5 +1/+1 counters; {1}, Remove +1/+1: Create Saproling; {1}, Sacrifice creature: Add +1/+1 counter
    - spike_weaver.rs — added {2}, Remove +1/+1 counter: Put +1/+1 counter on target (ability 2 deferred to PB-32)
    - ramos_dragon_engine.rs — added Remove 5 +1/+1 counters: Add WWUUBBRRGG (once-per-turn deferred to PB-37)
    - druids_repository.rs — added Remove charge counter: Add mana of any color
    - umezawas_jitte.rs — added combat trigger (2 charge counters) + Remove charge counter: +2/+2 (modes 2/3 deferred to PB-37)
  - G-17 (SpellAdditionalCost):
    - village_rites.rs — SacrificeCreature
    - deadly_dispute.rs — SacrificeArtifactOrCreature
    - goblin_grenade.rs — SacrificeSubtype(Goblin)
    - altar_of_bone.rs — SacrificeCreature (+ proper search effect)
    - crop_rotation.rs — SacrificeLand
    - corrupted_conviction.rs — SacrificeCreature
    - lifes_legacy.rs — SacrificeCreature (power-based draw deferred to PB-37)
    - abjure.rs — SacrificeColorPermanent(Blue) (+ counterspell effect)

- [x] 3. New card definitions (if any) — None (all existing defs fixed)

- [x] 4. Unit tests
  - Created `crates/engine/tests/cost_primitives.rs` with 12 tests:
    - test_remove_counter_cost_basic (CR 602.2)
    - test_remove_counter_cost_insufficient (CR 118.3)
    - test_remove_counter_cost_exact_zero (CR 118.3)
    - test_remove_counter_cost_in_sequence (CR 601.2h)
    - test_village_rites_has_sacrifice_creature_cost (CR 118.8)
    - test_crop_rotation_has_sacrifice_land_cost (CR 118.8)
    - test_deadly_dispute_has_sacrifice_artifact_or_creature_cost (CR 118.8)
    - test_goblin_grenade_has_sacrifice_goblin_cost (CR 118.8)
    - test_abjure_has_sacrifice_blue_permanent_cost (CR 118.8)
    - test_spell_sacrifice_cost_creature (CR 118.8)
    - test_spell_sacrifice_cost_missing (CR 118.8)
    - test_spell_sacrifice_cost_wrong_type (CR 118.8)
  - All 12 tests pass

- [x] 5. Workspace build verification
  - `cargo check` — clean
  - `cargo build --workspace` — clean
  - `cargo test --all` — all tests pass (0 failures)
  - `cargo clippy -- -D warnings` — 0 warnings
  - `cargo fmt --check` — clean

## Deferred to PB-37
- Ghave "remove counter from another creature you control" — needs target-based counter cost
- Crucible of the Spirit Dragon "Remove X counters" — needs X-value integration in Cost
- Tekuthal "remove counters from among others" — complex multi-source counter removal
- Plumb the Forbidden "may sacrifice one or more" — variable-count sacrifice + copy-per
- Flare of Fortitude sacrifice as alternative cost — needs AltCostKind extension
- Ramos once-per-turn activation restriction — needs ActivatedAbility once_per_turn field
- Mana ability classification for counter-removal abilities that produce mana
- Life's Legacy EffectAmount::SacrificedCreaturePower
- Spike Weaver ability 2 effect (PreventAllCombatDamage — G-19, PB-32)
- Umezawa's Jitte modes 2 (-1/-1) and 3 (gain 2 life)

## Review
findings: 7 (HIGH: 0, MEDIUM: 2, LOW: 5)
verdict: needs-fix
review_file: memory/primitives/pb-review-31.md

## Fix Phase Results
- M2 (umezawas_jitte.rs): Added TODO(PB-37) comment citing oracle text discrepancy —
  trigger remains WhenEquippedCreatureDealsCombatDamageToPlayer (no unqualified variant exists);
  updated comment from "to a player" to "deals combat damage" to match oracle header.
- M3 (lifes_legacy.rs): Replaced empty abilities vec with placeholder AbilityDefinition::Spell
  drawing 1 card (Fixed(1)); added TODO(PB-37) for EffectAmount::SacrificedCreaturePower.
- L1-L5: Documented deferrals — no action taken.
- cargo build --workspace: clean
- cargo test --all: all pass (0 failures)
- cargo clippy -- -D warnings: 0 warnings
- phase: DONE

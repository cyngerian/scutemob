# Primitive WIP: PB-29 -- Cost reduction statics

batch: PB-29
title: Cost reduction statics
cards_affected: ~30
started: 2026-03-25
phase: fix
plan_file: memory/primitives/pb-plan-29.md

## Gap Reference
G-7 from `docs/dsl-gap-closure-plan.md`:
- G-7: Cost reduction statics (~30 cards) — `ContinuousEffectDef` with `EffectLayer::CostModification` — "spells cost {N} less" as a static continuous effect

## Deferred from Prior PBs
- PB-8 already implemented cost reduction statics for 10 cards (basic pattern exists)
- PB-28 added CDA layer 7a support (may inform layer architecture)

## Step Checklist
- [x] 1. Engine changes (new types/variants/dispatch)
  - Added `legendary: bool` to `TargetFilter` + `matches_filter()` enforcement in effects/mod.rs
  - Added `SpellCostFilter::ColorAndCreature(Color)`, `HasChosenCreatureSubtype`, `AllSpells` variants
  - Added `SelfCostReduction::ConditionalKeyword{keyword,reduction}` and `MaxOpponentPermanents{filter,per}` variants
  - Added `SelfActivatedCostReduction` enum with `PerPermanent{per,filter,controller}` variant
  - Added `activated_ability_cost_reductions: Vec<(usize, SelfActivatedCostReduction)>` to `CardDefinition` (alternative design — avoids touching 400+ AbilityDefinition::Activated match sites)
  - Updated `spell_matches_cost_filter()` for new variants; special-cased `HasChosenCreatureSubtype` inline in `apply_spell_cost_modifiers()`
  - Updated `evaluate_self_cost_reduction()` for `ConditionalKeyword` and `MaxOpponentPermanents`
  - Added `get_self_activated_reduction()` + `evaluate_self_activated_reduction()` helpers in abilities.rs
  - Applied reduction in `handle_activate_ability()` after X-value resolution
  - Exported `SelfActivatedCostReduction` from helpers.rs, cards/mod.rs, lib.rs
  - Updated 139 card def files and 16 test files with `activated_ability_cost_reductions: vec![]` default
- [x] 2. Card definition fixes (13 cards)
  - archmage_of_runes: SpellCostFilter::InstantOrSorcery (already existed)
  - bontus_monument: ColorAndCreature(Black)
  - hazorets_monument: ColorAndCreature(Red)
  - oketras_monument: ColorAndCreature(White)
  - urzas_incubator: HasChosenCreatureSubtype, scope=AllPlayers
  - winged_words: SelfCostReduction::ConditionalKeyword{Flying, 1}
  - cavern_hoard_dragon: SelfCostReduction::MaxOpponentPermanents{Artifact, per=1}
  - boseiju_who_endures: SelfActivatedCostReduction::PerPermanent{per=1, legendary creature, index=0}
  - otawara_soaring_city: same as boseiju
  - eiganjo_seat_of_the_empire: same as boseiju
  - takenuma_abandoned_mire: same as boseiju
  - sokenzan_crucible_of_defiance: same as boseiju
  - voldaren_estate: SelfActivatedCostReduction::PerPermanent{per=1, Vampire, index=1}
- [x] 3. New card definitions — N/A (authoring paused)
- [x] 4. Unit tests — 11 new tests added to spell_cost_modification.rs; all 22 tests pass; total: 2363 tests pass
- [x] 5. Workspace build verification — `cargo build --workspace` clean, `cargo clippy -- -D warnings` clean, `cargo fmt` applied

## Review
findings: 3 (HIGH: 1, MEDIUM: 1, LOW: 1)
verdict: needs-fix
review_file: memory/primitives/pb-review-29.md

## Fix Phase
- [x] HIGH-1: Added `self.legendary.hash_into(hasher);` to `TargetFilter::hash_into()` in `state/hash.rs` at line 3714 (after `has_card_types`)
- [x] MEDIUM-3: Hazoret's Monument — TODO comment already present at line 23 of `hazorets_monument.rs`; no code change needed (pre-existing DSL gap)
- [x] LOW-2: ConditionalKeyword base characteristics approximation — no fix, known documented approximation
- Fix results: 2363 tests pass, 0 clippy warnings

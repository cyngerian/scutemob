# Primitive WIP: PB-Q4 — EnchantTarget::Filtered (bundled land-variant enchant targets)

batch: PB-Q4
title: EnchantTarget::Filtered — bundled land-variant enchant target filter
cards_unblocked_planned: 11 considered; 5 committed for clean ship; 6 deferred
started: 2026-04-12 (plan session)
phase: implement-complete
plan_file: memory/primitives/pb-plan-Q4.md

## Planner checklist

- [x] Read primitive-wip.md scope + gates
- [x] Read relevant feedback entries (verify_full_chain, pb_yield_calibration)
- [x] Gate 1 (Genju animate-land): PASS literal; Genju cycle still DEFERRED on rebuy-trigger grounds
- [x] Gate 2 (Chained controller filter): FAIL — EnchantTarget is flat; resolved by new Filtered variant
- [x] Gate 3 (Corrupted Roots disjunction): PASS via existing `TargetFilter.has_subtypes: Vec<SubType>`
- [x] MCP oracle lookups done for all considered cards
- [x] Plan file written: memory/primitives/pb-plan-Q4.md
- [x] Mandatory/optional test labels applied (12 mandatory, 2 optional)

## Verification gate summary

1. **Genju animate-land**: PASS (LayerModification::AddCardTypes/AddSubtypes/SetPowerToughness/AddKeywords all exist; EffectFilter::AttachedLand exists; live precedent in tatyova_steward_of_tides.rs). Genju cycle (5 cards) nonetheless DEFERRED because "when enchanted Forest is put into graveyard, return THIS from graveyard to hand" requires a `WhenAttachedPermanentLeavesBattlefield` + self-graveyard-rebuy trigger that does not exist.
2. **Chained to the Rocks controller filter**: FAIL in current enum (8 flat variants, no filter). Plan adds `EnchantTarget::Filtered(TargetFilter)` and extends `matches_enchant_target` to receive `aura_controller` and `target_controller`.
3. **Corrupted Roots disjunction**: PASS via existing `TargetFilter.has_subtypes: Vec<SubType>`. Corrupted Roots still DEFERRED because "whenever enchanted land becomes tapped" trigger does not exist.

## Yield reality

- Planner considered: 11 cards (Genju×5, Utopia Sprawl, Awaken the Ancient, Chained to the Rocks, Spreading Algae, Corrupted Roots, Uncontrolled Infestation, Ossification, Dimensional Exile, Caribou Range, Crackling Emergence, Harmonious Emergence, Hot Springs, Mystic Might, Tourach's Gate, Earthlore — actually ~17 if counted fully)
- **Committed for clean ship (HIGH confidence, 4)**: Awaken the Ancient, Chained to the Rocks, Ossification, Dimensional Exile
- **Committed pending verification (MEDIUM, 1 of 2)**: Hot Springs OR Earthlore if granted-activated-ability + prevention/untapped-state primitives exist
- **Deferred (blocking primitive named)**:
  - Genju cycle ×5 → needs `WhenAttachedPermanentLeavesBattlefield` + graveyard self-rebuy trigger
  - Utopia Sprawl → needs `AddMana(ChosenColor)` + as-enters chosen_color
  - Corrupted Roots, Spreading Algae, Uncontrolled Infestation → need "whenever enchanted land becomes tapped" trigger (no current analog of `WhenSelfBecomesTapped` for attached)
  - Caribou Range, Crackling Emergence, Harmonious Emergence, Mystic Might, Tourach's Gate → each needs complex grant (activated-ability with custom cost), replacement ("if would be destroyed instead…"), or Cumulative Upkeep + time-counter sac loops

**Realistic shipping expectation: 5 cards (45% of 11 considered)**, matching the pb_yield_calibration retro average.

## Non-negotiable constraints carried forward

- `apply_mana_production_replacements` (PB-Q) stays — do NOT revert
- Commit `464d9e79` is the baseline
- One PB: bundle all 4 isomorphic variants (LandSubtype, BasicLand, NonbasicLand, LandYouControl) under the single `Filtered(TargetFilter)` variant
- 12 MANDATORY tests (numbered in the plan) — skipping requires explicit justification in review doc
- `cargo build --workspace` is MANDATORY before completion (catches replay-viewer + TUI exhaustive-match drift)

## Implementation completed: 2026-04-12

### Engine changes
- `state/types.rs`: Added `EnchantControllerConstraint` enum, `EnchantFilter` struct, `EnchantTarget::Filtered(EnchantFilter)` variant.
- `state/hash.rs`: Added `HashInto for EnchantTarget::Filtered`, `HashInto for EnchantFilter`, `HashInto for EnchantControllerConstraint`. Added `nonbasic` field to `HashInto for TargetFilter`.
- `cards/card_definition.rs`: Added `pub nonbasic: bool` field to `TargetFilter`.
- `rules/sba.rs`: Extended `matches_enchant_target` signature to `(enchant, target_chars, aura_controller, target_controller)`. Added `EnchantTarget::Filtered` arm. Added `enchant_filter_matches` helper covering all 6 filter checks.
- `rules/casting.rs`: Updated call site to pass `player` and `target_ctrl` to `matches_enchant_target`.
- `effects/mod.rs`: Added `nonbasic` check in TargetFilter evaluation.
- `state/mod.rs`: Re-exported `EnchantControllerConstraint`, `EnchantFilter`.
- `cards/helpers.rs`: Re-exported `EnchantControllerConstraint`, `EnchantFilter`.
- `lib.rs`: Exported `EnchantControllerConstraint`, `EnchantFilter`.

### Design deviation from plan
Plan called for `EnchantTarget::Filtered(TargetFilter)` (or boxed). Changed to `Filtered(EnchantFilter)` using a new minimal struct in `state/types.rs` to avoid a circular dependency (`state/types.rs` → `cards/card_definition.rs` → `state/types.rs`). `EnchantFilter` captures all 6 fields needed for enchant enforcement. Semantically identical to the plan's intent; no information loss.

### Card defs authored (4 HIGH confidence, 0 MEDIUM)
- `awaken_the_ancient.rs`: Enchant Mountain, 7/7 Creature+Giant+Red+Haste animation via 5 layer effects.
- `chained_to_the_rocks.rs`: Enchant Mountain you control, ETB exile creature until leaves.
- `ossification.rs`: Enchant basic land you control, ETB exile creature or planeswalker until leaves.
- `dimensional_exile.rs`: Enchant basic land you control, ETB exile creature until leaves.

### MEDIUM cards deferred
- `Hot Springs`: Prevention-shield activated ability — no `Effect::PreventDamage` in DSL.
- `Earthlore`: Grants activated ability targeting "blocking creature" — no `is_blocking` field in `TargetFilter`.

### Mandatory tests (12/12 PASS)
1. test_enchant_filtered_land_subtype_cast_time_legal — PASS
2. test_enchant_filtered_land_subtype_cast_time_illegal — PASS
3. test_enchant_filtered_controller_cast_time_legal — PASS
4. test_enchant_filtered_controller_cast_time_illegal — PASS
5. test_enchant_filtered_basic_land_legal — PASS
6. test_enchant_filtered_basic_land_illegal_nonbasic — PASS
7. test_enchant_filtered_sba_control_change — PASS
8. test_enchant_filtered_sba_land_becomes_nonland — PASS
9. test_enchant_filtered_disjunction_forest_or_plains — PASS
10. test_enchant_filtered_nonbasic_land — PASS
11. test_animate_land_pt_and_types_via_chained_or_awaken — PASS
12. test_animate_land_summoning_sickness_propagation — PASS

### Final verification
- `cargo build --workspace`: CLEAN
- `cargo test --all`: 2637 passed (was 2625, +12), 0 failed
- `cargo clippy --workspace -- -D warnings`: CLEAN
- `cargo fmt --check`: CLEAN
- `apply_mana_production_replacements`: UNTOUCHED (verified via git diff 464d9e79..HEAD)

## Next action for reviewer
Review `memory/primitives/pb-plan-Q4.md` against implementation. Key deviation: `EnchantFilter` instead of `Box<TargetFilter>` in `EnchantTarget::Filtered` — verify this is acceptable. Check that `enchant_filter_matches` covers all 6 filter fields correctly.

# Primitive WIP: PB-Q4 — EnchantTarget::LandSubtype (+ bundled variants)

batch: PB-Q4
title: EnchantTarget::LandSubtype + bundled enchant target variants
cards_unblocked: 10-20 (pending Genju animate-land verification gate)
started: not yet — next session begins plan phase
phase: pending
plan_file: memory/primitives/pb-plan-Q4.md (not yet written)

## Yield audit (2026-04-12)

**Direct LandSubtype yield: 10 commander-legal cards, all unauthored**:
- Utopia Sprawl (Forest)
- Genju cycle (5): Cedars/Falls/Spires/Fields/Fens — each enchants its basic land + animate-land activated ability
- Awaken the Ancient (Mountain → 5/5 elemental)
- Chained to the Rocks (Mountain you control)
- Spreading Algae (Swamp)
- Corrupted Roots (Forest or Plains — disjunction)

**Bundled scope yield: ~20 cards** (3 isomorphic adjacent variants):
- `EnchantTarget::BasicLand` (basic supertype only): Dimensional Exile, Ossification (2)
- `EnchantTarget::NonbasicLand`: Uncontrolled Infestation (1)
- `EnchantTarget::LandYouControl`: Caribou Range, Crackling Emergence, Earthlore, Harmonious Emergence, Hot Springs, Mystic Might, Tourach's Gate (7)

## Three Verification Gates (planner must run BEFORE writing pb-plan-Q4.md)

1. **Genju animate-land effect** — does the engine have "enchanted land becomes an N/N creature until end of turn"?
   - Grep `crates/engine/src/effects/` for: `BecomesCreature`, `AnimateLand`, `SetCreatureType`, `Effect::Animate*`, `Effect::LandBecomes*`
   - Also check `LayerModification` variants in `state/continuous_effect.rs`
   - **If MISSING → exclude Genju cycle (5 cards), yield drops from 10→5 narrow / 20→15 bundled.** This is the make-or-break gate.

2. **Chained to the Rocks controller filter** — "Enchant Mountain you control" needs subtype + controller constraint.
   - Check `EnchantTarget` enum in `crates/engine/src/cards/card_definition.rs`
   - **If no controller predicate → defer Chained to the Rocks (1 card), yield drops by 1.**

3. **Corrupted Roots disjunction** — "Enchant Forest or Plains" needs OR / `Vec<SubType>`.
   - Decide: support `LandSubtype(SubType)` only, or `LandSubtypes(Vec<SubType>)` with len-1 default?
   - **If single-subtype only → defer Corrupted Roots (1 card), yield drops by 1.**

## Scoping Directive

**Bundle all 4 isomorphic enchant target variants into ONE PB**. Do NOT split:
- `EnchantTarget::LandSubtype(SubType)` — 10 cards (pending gates)
- `EnchantTarget::BasicLand` — 2 cards
- `EnchantTarget::NonbasicLand` — 1 card
- `EnchantTarget::LandYouControl` — 7 cards

These are the same dispatch pattern repeated in `casting.rs::validate_enchant_target` (or wherever Enchant validation lives). One plan, one implement, one review, one close.

## Apply Yield Calibration

Per `feedback_pb_yield_calibration.md` (auto-memory): planners overcount in-scope cards by 2-3x. Whatever count the planner claims, **expect 40-50% of those cards to actually ship clean** and the rest to spawn micro-PBs (PB-Q4a, PB-Q4b, etc.).

Realistic shipping expectations:
- **Best case** (all 3 gates pass): plan 20, ship 12-14
- **Genju gate fails**: plan 15, ship 9-11
- **All gates fail**: plan 13, ship 7-9

## CR Reference

- CR 303.4 — Enchant keyword and target restrictions
- CR 702.5 — Enchant ability
- CR 205.3i — Land subtypes (basic land types: Plains, Island, Swamp, Mountain, Forest)

## Hazards

- Genju gate is load-bearing. Run it FIRST.
- Planner will be tempted to claim "20 cards" — discount before scoping
- `apply_mana_production_replacements` refactor (PB-Q) stays — do NOT touch
- PB-Q close (commit `464d9e79`) is the baseline state for PB-Q4 work

## Next Action

Next session: `/start-work W6-PB-Q4`, then dispatch primitive-impl-planner with the 3 verification gates as mandatory pre-plan steps.

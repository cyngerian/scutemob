# Primitive WIP: PB-6 -- Static Grant with Controller Filter (REVIEW-ONLY)

batch: PB-6
title: Static grant with controller filter
cards_affected: 30
mode: review-only
started: 2026-03-16
phase: fix
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-6 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

Cards: 30 cards with static grant effects filtered by controller.
Engine change: EffectFilter::CreaturesControlledBySource and
EffectFilter::CreaturesYouControlWithSubtype(SubType). Source controller
resolved at layer-application time in layers.rs.

Key areas:
- EffectFilter variants for controller-scoped grants
- Layer application logic in layers.rs for source controller resolution
- continuous_effect.rs for new filter types
- Oracle text accuracy for all 30 cards
- Correct grant scoping (your creatures only vs all creatures)

## Review
findings: 12 (HIGH: 1, MEDIUM: 5, LOW: 6)
verdict: needs-fix
review_file: memory/primitives/pb-review-6.md

Actionable findings (PB-6 scope):
- [x] Finding 2 (HIGH): Goblin Warchief — added Keyword(Haste) intrinsic + kept OtherCreaturesYouControlWithSubtype for other Goblins (functionally equivalent to oracle)
- [x] Finding 3 (MEDIUM): Archetype of Endurance — added Static with CreaturesYouControl + AddKeyword(Hexproof); updated TODO to removal-half-only
- [x] Finding 4 (MEDIUM): Archetype of Imagination — added Static with CreaturesYouControl + AddKeyword(Flying); updated TODO to removal-half-only
- [x] Finding 5 (MEDIUM): Iroas, God of Victory — added Static with CreaturesYouControl + AddKeyword(Menace); kept TODOs for devotion and damage prevention
- [x] Finding 6 (MEDIUM): Vito, Thorn of the Dusk Rose — added Activated with Cost::Mana({3}{B}{B}) + ApplyContinuousEffect(Lifelink, CreaturesYouControl, UntilEndOfTurn); kept TODO for trigger
- [x] Finding 7 (MEDIUM): Vault of the Archangel — added Activated with Cost::Sequence([Mana({2}{W}{B}), Tap]) + Effect::Sequence([Deathtouch, Lifelink] grants)
- [x] Finding 1 (LOW): layers.rs comment typos already correct (// CR not / CR) — no change needed
- [x] Findings 8-12 (LOW): TODO comments updated in all 5 card defs to reflect grant-half implemented; only blocked halves noted

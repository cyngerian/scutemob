# Primitive WIP: PB-13 -- Specialized Mechanics (REVIEW-ONLY)

batch: PB-13
title: Specialized Mechanics
cards_affected: 19
mode: review-only
started: 2026-03-18
phase: fix
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-13 implementation.
10+ sub-batches: land animation, channel, ascend/city's blessing, equipment auto-attach,
dredge, buyback, player hexproof, coin flip/d20, timing restriction, clone/copy ETB,
monarch, flicker, adventure, living weapon.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

## Review
findings: 16 (HIGH: 2, MEDIUM: 13, LOW: 1)
verdict: needs-fix
review_file: memory/primitives/pb-review-13.md

Actionable findings:
- [x] Finding 2 (HIGH): Golgari Grave-Troll P/T — fixed toughness: Some(4) → Some(0) in golgari_grave_troll.rs
- [x] Finding 3 (HIGH): Batterskull missing {3} bounce-to-hand ability — added AbilityDefinition::Activated with Cost::Mana(3) + Effect::MoveZone to Hand in batterskull.rs
- [x] Finding 4 (MEDIUM): Wayward Swordtooth — DEFERRED: no AdditionalLandPlay static or DoesNotHaveCitysBlessing attack/block restriction in DSL. Updated TODO with full gap analysis.
- [x] Finding 5 (MEDIUM): Otawara target filter — improved from TargetPermanent to TargetPermanentWithFilter(non_land:true). Cost reduction TODO retained.
- [x] Finding 6 (MEDIUM): Boseiju target filter — improved to TargetPermanentWithFilter(controller:Opponent). Full type-restriction (artifact|enchantment|nonbasic-land) deferred (multi-type OR with nonbasic DSL gap). Updated TODO.
- [x] Finding 7 (MEDIUM): Sokenzan tokens — added KeywordAbility::Haste to token spec keywords. Note: permanent haste vs UntilEndOfTurn is a minor deviation (documented in TODO). Cost reduction TODO retained.
- [x] Finding 8 (MEDIUM): Takenuma graveyard return — added Effect::Sequence with MoveZone from graveyard using TargetCardInYourGraveyard(has_card_types:[Creature,Planeswalker]) target. Cost reduction TODO retained.
- [x] Finding 9 (MEDIUM): Eiganjo target filter — DEFERRED: no TargetFilter.is_attacking/is_blocking field. Updated TODO with precise DSL gap description.
- [x] Finding 10 (MEDIUM): Eomer ETB trigger — added AbilityDefinition::Triggered with BecomeMonarch(DeclaredTarget{0}) + DealDamage(PowerOf(Source), DeclaredTarget{1}). Counter placement (X per Human) deferred (AddCounters needs dynamic EffectAmount). Updated TODO.
- [x] Finding 11 (MEDIUM): Twilight Prophet — DEFERRED: TriggerCondition and Condition exist but EffectAmount::ManaValueOfLastDrawnCard doesn't (no EffectTarget for "the card just drawn"). Updated TODO with precise gap.
- [x] Finding 12 (MEDIUM): Crystal Barricade — DEFERRED: noncombat damage prevention requires new ReplacementTrigger variant. Updated TODO with precise DSL gap.
- [x] Finding 13 (MEDIUM): Serra Ascendant — DEFERRED: ControllerLifeAtLeast(30) exists as Condition but EffectDuration has no WhileCondition variant. Updated TODO.
- [x] Finding 14 (MEDIUM): Hammer of Nazahn — DEFERRED: needs EffectTarget::TriggeringObject (doesn't exist) and optional trigger choice. Updated both file header TODO and inline TODO.
- [x] Finding 15 (MEDIUM): Monster Manual — DEFERRED: activated ability needs TargetRequirement::TargetCardInHand (doesn't exist). Adventure deferred per 13m. Updated TODO with full analysis.
- [x] Finding 1 (LOW): Arch of Orazca activation restriction — DEFERRED: PB-18 stax framework uses PermanentFilter not Condition; would require legal_actions gate change. Comment already notes this accurately.
- [x] Finding 16 (LOW): Golgari Grave-Troll ETB/regen/CDA — DEFERRED: toughness fixed (Finding 2); ETB counter placement, regen, and CDA power deferred per existing DSL gap comments. P/T is now correctly Some(0)/Some(0).

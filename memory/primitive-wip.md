# Primitive WIP: PB-10 -- Return From Zone Effects (REVIEW-ONLY)

batch: PB-10
title: Return from zone effects (graveyard targeting)
cards_affected: 8
mode: review-only
started: 2026-03-18
phase: fix
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-10 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

## Review
findings: 10 (HIGH: 2, MEDIUM: 5, LOW: 3)
verdict: needs-fix
review_file: memory/primitives/pb-review-10.md

Actionable findings:
- [x] Finding 1 (HIGH): Hexproof/shroud blocks graveyard targeting — gated validate_target_protection on Battlefield|Stack zone in casting.rs:5161
- [x] Finding 2 (HIGH): "Under your control" not supported — added controller_override: Option<PlayerTarget> to Effect::MoveZone; updated hash.rs, effects/mod.rs, all 14 MoveZone construction sites; Reanimate+Teneb use Some(Controller)
- [x] Finding 3 (MEDIUM): Bladewing missing "permanent card" filter — added has_card_types: vec![Creature, Artifact, Enchantment, Land, Planeswalker] to TargetFilter
- [x] Finding 4 (MEDIUM): Emeria intervening-if fires unconditionally — documented as DSL gap in card def (Condition::YouControlNOrMorePermanentsWithSubtype missing)
- [x] Finding 5 (MEDIUM): Teneb optional mana payment not implemented — documented as DSL gap in card def (Cost on triggered abilities missing)
- [x] Finding 6 (MEDIUM): Reanimate life loss not implemented — documented as DSL gap in card def (EffectAmount::ManaValueOfTarget missing)
- [x] Finding 7 (MEDIUM): Connive // Concoct Concoct half — implemented with Sequence([Surveil{3}, MoveZone to Battlefield]) + TargetCardInYourGraveyard(Creature)
- Finding 8 (LOW — DEFER): Bladewing activated ability TODO — creature-type-filtered pump DSL gap
- Finding 9 (LOW — DEFER): Den Protector static evasion TODO — unrelated DSL gap
- Finding 10 (LOW — DEFER): Haven "Ugin planeswalker" union target — name-or-type union DSL gap

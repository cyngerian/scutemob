# Primitive WIP: PB-1 -- Mana With Damage (REVIEW-ONLY)

batch: PB-1
title: Mana with damage (pain lands)
cards_affected: 8
mode: review-only
started: 2026-03-16
phase: closed
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-1 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

Cards: battlefield_forge, caves_of_koilos, city_of_brass, llanowar_wastes,
shivan_reef, sulfurous_springs, underground_river, yavimaya_coast

Key areas:
- Effect::Sequence pattern for pain lands (AddMana + DealDamage)
- City of Brass: TriggerCondition::WhenBecomesTapped (engine addition)
- Oracle text accuracy for all 8 cards

## Review
findings: 1 (HIGH: 0, MEDIUM: 1, LOW: 0)
verdict: fixed
review_file: memory/primitives/pb-review-1.md

MEDIUM [FIXED]: All 7 pain lands split into two separate activated abilities (one per color).
Each ability adds exactly 1 mana of one color and deals 1 damage. ability_index 1 = first color,
ability_index 2 = second color. Tests updated: removed "adds both" comment, added
battlefield_forge_second_colored_tap_deals_damage, all_pain_lands_deal_damage_on_second_colored_tap,
shivan_reef_produces_exactly_one_blue_or_red. 7 tests pass.

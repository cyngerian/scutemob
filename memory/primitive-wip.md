# Primitive WIP: PB-11 -- Mana Spending Restrictions + ETB Player Choice (REVIEW-ONLY)

batch: PB-11
title: Mana Spending Restrictions + ETB Player Choice
cards_affected: 13
mode: review-only
started: 2026-03-18
phase: fix
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-11 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

## Review
findings: 16 (HIGH: 1, MEDIUM: 6, LOW: 9)
verdict: needs-fix
review_file: memory/primitives/pb-review-11.md

Actionable findings:
- [x] Finding 1 (HIGH): Restricted mana never used in casting flow — SpellContext never constructed in handle_cast_spell. Fixed: import SpellContext in casting.rs, construct from chars (card_types + subtypes), call can_pay_cost_with_context / pay_cost_with_context at lines 3304-3315.
- [x] Finding 3 (MEDIUM): ChosenTypeCreaturesOnly resolves to SubtypeOnly, losing creature requirement — need CreatureWithSubtype variant. Fixed: added CreatureWithSubtype(SubType) to ManaRestriction enum (card_definition.rs), restriction_matches arm in player.rs, hash arm in state/hash.rs (discriminants shifted: 3=CreatureWithSubtype, 4=ChosenTypeCreaturesOnly, 5=ChosenTypeSpellsOnly). resolve_mana_restriction in effects/mod.rs maps ChosenTypeCreaturesOnly→CreatureWithSubtype.
- [x] Finding 5 (MEDIUM): Haven SubtypeOnly(Dragon) should be CreatureWithSubtype(Dragon). Fixed.
- [x] Finding 6 (MEDIUM): Seedcore SubtypeOnly(Phyrexian) should be CreatureWithSubtype(Phyrexian). Fixed.
- [x] Finding 7 (MEDIUM): Gnarlroot SubtypeOnly(Elf) should be CreatureWithSubtype(Elf). Fixed.
- [x] Finding 8 (MEDIUM): Gnarlroot missing Pay 1 life cost — document as DSL gap. Fixed: expanded TODO comment with Cost::PayLife note.
- [x] Finding 9 (MEDIUM): Voldaren Estate missing Pay 1 life cost — document as DSL gap. Fixed: expanded TODO comment with Cost::PayLife note.
- Finding 2 (MEDIUM — DEFER): AddManaAnyColorRestricted adds colorless — inherited from AddManaAnyColor limitation
- Finding 4 (LOW — DEFER): HashMap non-deterministic tie-breaking in ChooseCreatureType
- Findings 10-16 (LOW — DEFER): Documented DSL gap TODOs in card defs

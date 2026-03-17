# Primitive WIP: PB-7 -- Count-Based Scaling (REVIEW-ONLY)

batch: PB-7
title: Count-based scaling
cards_affected: 29
mode: review-only
started: 2026-03-16
phase: fix
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-7 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

Cards: 29 cards with count-based scaling effects ("for each creature you control,"
"number of lands you control," devotion, counter counts, etc.)

Engine changes: EffectAmount extended with:
- PermanentCount { filter: TargetFilter, controller: PlayerTarget }
- DevotionTo(Color)
- CounterCount { target: EffectTarget, counter: CounterType }

Key areas:
- EffectAmount variants for count-based scaling
- Effect evaluation logic in effects/mod.rs
- Layer application for devotion-based effects
- Oracle text accuracy for all 29 cards
- Correct count semantics (your permanents vs all permanents)

## Review
findings: 8 (HIGH: 3, MEDIUM: 2, LOW: 3)
verdict: needs-fix
review_file: memory/primitives/pb-review-7.md

Actionable findings (PB-7 scope):
- [x] Finding 1 (MEDIUM): DevotionTo ignores hybrid/phyrexian mana symbols (CR 700.5) — fixed in effects/mod.rs:3398-3465; added test_devotion_counts_hybrid_and_phyrexian_mana_symbols
- [x] Finding 4 (HIGH): Nykthos — wrong Shrine subtype (oracle: Legendary Land, no subtypes) — changed full_types to supertypes in nykthos_shrine_to_nyx.rs
- [x] Finding 5 (HIGH): Faeburrow Elder — P/T None/None should be 0/0 (not a CDA) — changed to Some(0)/Some(0), removed CDA comment
- [x] Finding 6 (HIGH): Multani — P/T None/None should be 0/0 (not a CDA) — changed to Some(0)/Some(0), removed CDA comment
- [x] Finding 7 (MEDIUM): Frodo — oracle text completely wrong — replaced with actual oracle text ({W/B}{W/B} Citizen→Scout/lifelink; {B}{B}{B} Scout→Rogue/ring-loss condition)
- [x] Finding 8 (LOW): Toothy — explicit struct construction — migrated to ..Default::default()
- Findings 2-3 (LOW): raw characteristics reads — systemic, deferred

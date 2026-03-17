# Primitive WIP: PB-9 -- Hybrid Mana & X Costs (REVIEW-ONLY)

batch: PB-9
title: Hybrid mana & X costs
cards_affected: 7
mode: review-only
started: 2026-03-17
phase: fix
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-9 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

Cards: 7 cards with hybrid mana costs and/or X costs (brokkos_apex_of_forever,
connive, nethroi_apex_of_death, cut_ribbons, mockingbird, + X-cost cards)

Engine changes: `hybrid: Vec<HybridMana>` and `x_count: u32` on `ManaCost`.
Hybrid payment in casting.rs and mana_solver.rs.

Key areas:
- HybridMana enum and ManaCost hybrid/x_count fields
- casting.rs hybrid mana payment logic
- mana_solver.rs hybrid handling
- Oracle text accuracy for all 7 cards
- Correct hybrid cost semantics (CR 107.4)

## Review
findings: 15 (HIGH: 1, MEDIUM: 7, LOW: 7)
verdict: needs-fix
review_file: memory/primitives/pb-review-9.md

Actionable findings (PB-9 scope):
- [x] Finding 3 (HIGH): Brokkos main mana cost fixed — {2}{B}{G}{U} separate pips (brokkos_apex_of_forever.rs)
- [x] Finding 1 (MEDIUM): mana_solver PipTracker::from_cost() now flattens hybrid/phyrexian/x_count (mana_solver.rs)
- [x] Finding 4 (MEDIUM): Connive Concoct half added as AbilityDefinition::Fuse with {3}{U}{B} cost + Surveil 3 effect + TODO for return-creature (connive.rs)
- [x] Finding 5 (MEDIUM): Connive spell ability added with TargetCreatureWithFilter(max_power:2) + Effect::Nothing placeholder + TODO for GainControl DSL gap (connive.rs)
- [x] Finding 6 (MEDIUM): Revitalizing Repast front face effect implemented — Sequence(AddCounter +1/+1, ApplyContinuousEffect Indestructible UntilEOT) (revitalizing_repast.rs)
- [x] Finding 7 (MEDIUM): Old-Growth Grove back face added — land with EntersTapped replacement + {T}: Add {G} activated ability (revitalizing_repast.rs)
- [x] Finding 8 (MEDIUM — DEFER): Blade Historian already has proper DSL gap TODO; no change needed
- [x] Finding 9 (MEDIUM — DEFER): Treasure Vault already has proper TODO; no change needed
- [x] Finding 10 (MEDIUM): Cut // Ribbons stale TODO block removed (cut_ribbons.rs)
- Findings 2, 11-17 (LOW): filter land approximations, PW stubs, DSL gaps — deferred

# Primitive WIP: PB-14 -- Planeswalker Support + Emblems (REVIEW-ONLY)

batch: PB-14
title: Planeswalker Support + Emblems
cards_affected: 31
mode: review-only
started: 2026-03-18
phase: closed
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-14 implementation.
Full planeswalker framework: loyalty counters, loyalty abilities (+N/-N/-X), one-per-turn
restriction, 0-loyalty SBA (CR 704.5i), emblem creation, damage redirection to planeswalkers
(CR 306.7). Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

## Review
findings: 4 (HIGH: 1, MEDIUM: 1, LOW: 2)
verdict: needs-fix
review_file: memory/primitives/pb-review-14.md

Actionable findings:
- [x] Finding 1 (HIGH): Combat damage to planeswalkers uses damage_marked instead of removing loyalty counters (combat.rs:1690-1695). Fixed: replaced damage_marked assignment with CounterType::Loyalty removal (saturating_sub) in combat.rs:1690-1702. Added test_combat_damage_to_planeswalker_removes_loyalty_cr306_8 in planeswalker.rs.
- [x] Finding 2 (MEDIUM): Emblem creation (CR 114) not implemented. DEFERRED: added to Deferred Items table in docs/project-status.md. Updated ajani_sleeper_agent.rs TODO comment to reference the deferral.
- [x] Finding 3 (LOW): loyalty_ability_activated_this_turn is bool not Designations bitflag. DEFERRED — works correctly, low priority.
- [x] Finding 4 (LOW): x_value not wired for -X abilities in replay harness. DEFERRED — no card currently uses MinusX.

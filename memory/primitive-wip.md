# Primitive WIP: PB-0 -- Quick Wins (REVIEW-ONLY)

batch: PB-0
title: Quick wins (no engine changes)
cards_affected: 23
mode: review-only
started: 2026-03-16
phase: closed
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-0 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

Categories:
- 12 ETB-tapped lands (simple replacement)
- 5 Cycling cards (Cycling cost + keyword)
- 1 Flying fix (Thousand-Faced Shadow)
- 1 Color indicator fix (Dryad Arbor)
- 1 Wither keyword (Boggart Ram-Gang)
- 3 Forced attack cards

## Review
findings: 2 (HIGH: 0, MEDIUM: 1, LOW: 1)
verdict: fixed
review_file: memory/primitives/pb-review-0.md

## Fixes Applied
- [x] MEDIUM: thousand_faced_shadow.rs — added Ninjutsu keyword marker + cost ability (generic: 2, blue: 2)
- [x] LOW: keywords.rs — added test_508_1d_must_attack_each_combat_enforced (positive + negative cases)

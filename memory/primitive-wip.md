# Primitive WIP: PB-12 -- Complex Replacement Effects (REVIEW-ONLY)

batch: PB-12
title: Complex Replacement Effects
cards_affected: 11
mode: review-only
started: 2026-03-18
phase: fix
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-12 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

## Review
findings: 10 (HIGH: 2, MEDIUM: 8)
verdict: needs-fix
review_file: memory/primitives/pb-review-12.md

Actionable findings:
- [x] Finding 1 (HIGH): Life-loss doubling is dead code — fixed: apply_life_loss_doubling now called in Effect::LoseLife and Effect::DrainLife (effects/mod.rs)
- [x] Finding 2 (HIGH): Combat damage bypasses damage doubling — fixed: apply_damage_doubling now called per-assignment before apply_damage_prevention in apply_combat_damage (rules/combat.rs)
- [x] Finding 3 (MEDIUM): Counter replacement ignores player receivers — DEFERRED: added apply_counter_replacement_player stub with documented TODO in replacement.rs; requires ObjectFilter::Player variant or new WouldPlaceCountersOnPlayer trigger
- [x] Finding 4 (MEDIUM): Teysa CreatureDeath doubling only matches SelfDies — DEFERRED: updated TODO comment in abilities.rs:7277 with full implementation plan; WheneverCreatureDies triggers not wired in enrich_spec_from_def so doubling them is moot until that's fixed
- [x] Finding 5 (HIGH — card): Bloodletter replacement never fires — fixed (depends on F1, now wired)
- [x] Finding 6 (HIGH — card): Twinflame Tyrant damage doubling incomplete for combat — fixed (depends on F2, combat path now wired)
- [x] Finding 7 (MEDIUM — card): Tekuthal activated ability TODO — DSL gap, existing TODO comment is accurate (no change needed)
- [x] Finding 8 (MEDIUM — card): Teysa static token grant TODO — DSL gap, existing TODO comment is accurate (no change needed)
- [x] Finding 9 (MEDIUM — card): Bloodletter "during your turn" condition — fixed: apply_life_loss_doubling now checks active_player == effect.controller; bloodletter_of_aclazotz.rs comment updated
- [x] Finding 10 (MEDIUM — card): Twinflame Tyrant opponent-target filter — fixed: added DamageTargetFilter::ToOpponentOrTheirPermanent variant; card def updated to use it; apply_damage_doubling checks both source controller and target side; registration binding added

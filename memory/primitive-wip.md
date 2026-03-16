# Primitive WIP: PB-5 -- Targeted Activated/Triggered Abilities (REVIEW-ONLY)

batch: PB-5
title: Targeted activated/triggered abilities
cards_affected: 32
mode: review-only
started: 2026-03-16
phase: closed
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-5 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

Cards: 32 cards with targeted activated or triggered abilities.
Engine change: targets: Vec<TargetRequirement> on AbilityDefinition::Activated and ::Triggered.
Target validation wired in command.rs (activated) and engine.rs/resolution.rs (triggered).

Key areas:
- targets field on Activated and Triggered AbilityDefinition variants
- Target validation in command.rs for activated abilities
- Target validation in engine.rs/resolution.rs for triggered abilities
- Oracle text accuracy for all 32 cards
- Correct target requirements (creature, player, permanent, etc.)

## Review
findings: 15 (HIGH: 1, MEDIUM: 8, LOW: 6)
verdict: needs-fix
review_file: memory/primitives/pb-review-5.md

Actionable findings (PB-5 scope):
- [x] Finding 1 (HIGH): Triggered ability targets not populated from CardDef — FIXED in abilities.rs:6184 (auto-select logic in flush_pending_triggers else branch for Normal/CardDefETB)
- [x] Finding 2 (MEDIUM): No fizzle check for CardDef triggered abilities — FIXED in resolution.rs:2009 (CR 608.2b check before condition_holds)
- [x] Finding 3 (MEDIUM): No target validation for triggered abilities at trigger time — FIXED as part of Finding 1 (validate_target_protection called per object candidate in auto-select loop)
- [x] Finding 5 (MEDIUM): Blinkmoth Nexus target too permissive — FIXED in blinkmoth_nexus.rs:85 (TargetCreatureWithFilter with has_subtype Blinkmoth)
- [x] Finding 10 (LOW): Ghost Quarter verbose target — FIXED in ghost_quarter.rs:52 (replaced TargetPermanentWithFilter(Land) with TargetLand)

Deferred findings (other PB scope or DSL gaps):
- Finding 4 (MEDIUM): Forerunner colorless filter — DSL gap (is_colorless on TargetFilter)
- Findings 6-9 (MEDIUM): Boseiju/Otawara/Eiganjo over-permissive — DSL gaps
- Findings 11-15 (LOW): Various card def gaps — other DSL gaps

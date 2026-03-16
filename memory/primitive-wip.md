# Primitive WIP: PB-2 -- Conditional ETB Tapped (REVIEW-ONLY)

batch: PB-2
title: Conditional ETB tapped
cards_affected: 56
mode: review-only
started: 2026-03-16
phase: fix
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-2 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

Cards: 56 ETB-tapped lands with conditional untapped entry. Includes check-lands (12),
fast-lands (6), bond-lands (10), reveal-lands (6), battle-lands (4), slow-lands (6+),
and misc castle/conditional lands (12).

Key areas:
- Condition variants: YouControlLandWithSubtype, YouControlAtMostNOtherLands,
  YouHaveTwoOrMoreOpponents, RevealCardOfType, YouControlTwoOrMoreBasicLands,
  YouControlOtherLandCount
- unless_condition on AbilityDefinition::Replacement
- Oracle text accuracy for all 56 cards

## Review
findings: 5 (HIGH: 1, MEDIUM: 2, LOW: 2)
verdict: needs-fix
review_file: memory/primitives/pb-review-2.md

HIGH [FIXED]: 10 cards missed by PB-2 — added Replacement + mana tap abilities to all 10 (castle_ardenvale, castle_embereth, castle_locthwain, castle_vantress, mystic_sanctuary, witchs_cottage, arena_of_glory, mistrise_village, shifting_woodland, spymasters_vault)
MEDIUM [FIXED]: Isolated Chapel stale TODO comments removed + oracle text updated to current template
MEDIUM [DEFERRED]: Minas Tirith + Temple of the Dragon Queen remaining TODOs (out of PB-2 scope — future primitives)
LOW [DEFERRED]: ControlLandWithSubtypes doesn't exclude ctx.source (safe for all current cards)
LOW [DEFERRED]: ControlBasicLandsAtLeast doesn't exclude ctx.source (safe for all current cards)

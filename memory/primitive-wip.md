# Primitive WIP: PB-18 -- Stax / Action Restrictions (REVIEW-ONLY)

batch: PB-18
title: Stax / Action Restrictions
cards_affected: 10
mode: review-only
started: 2026-03-19
phase: done
plan_file: n/a (retroactive review -- no plan needed)

## Review
findings: 7 (HIGH: 2, MEDIUM: 4, LOW: 1)
verdict: needs-fix
review_file: memory/primitives/pb-review-18.md

## Review Scope
Engine changes and card definition fixes from the original PB-18 implementation.
ContinuousRestriction system: CantCastSpells, CantAttackYouUnlessPay, CantActivateAbilities, CantPlayLands.
Checked in casting.rs, combat.rs, legal_actions.rs. 10 card defs.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

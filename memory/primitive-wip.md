# Primitive WIP: PB-16 -- Meld (REVIEW-ONLY)

batch: PB-16
title: Meld
cards_affected: 1
mode: review-only
started: 2026-03-19
phase: done
plan_file: n/a (retroactive review -- no plan needed)

## Review Scope
Engine changes and card definition fixes from the original PB-16 implementation.
Review against CR rules and oracle text. No plan file -- reviewer works from
primitive-card-plan.md batch specification and the actual code.

- Meld pairs tracked on CardDefinition (front + paired card)
- Command::Meld checks both cards present on battlefield
- Melded permanent has combined characteristics
- Zone-change splits back into individual cards (similar to Mutate)
- Cards: hanweir_battlements (melds with Hanweir Garrison)
- Files: card_definition.rs, game_object.rs, command.rs

## Review
findings: 5 (HIGH: 1, MEDIUM: 2 fixable + 2 DSL-gap deferred, LOW: 1)
verdict: needs-fix
review_file: memory/primitives/pb-review-16.md

## Fix Phase (complete 2026-03-19)

- [x] HIGH: Phantom exiled cards after meld (effects/mod.rs:2368-2370)
      Captured new ObjectIds from move_object_to_zone calls; removed both phantom
      objects from exile zone and objects map after melded permanent created.
- [x] MEDIUM: Wrong mana value for melded permanents (rules/layers.rs:162)
      Compute source_mv + partner_mv from both front face mana costs; store as
      synthetic ManaCost { generic: combined_mv } per CR 712.8g.
- [ ] MEDIUM: Hanweir Garrison attack trigger TODO (hanweir_garrison.rs:19-20)
      DEFERRED -- DSL gap: "tapped and attacking" token creation not yet available.
- [ ] MEDIUM: Hanweir, the Writhing Township attack trigger TODO (hanweir_the_writhing_township.rs:35-36)
      DEFERRED -- DSL gap: "tapped and attacking" token creation not yet available.
- [x] LOW: Oracle text mismatch (hanweir_the_writhing_township.rs:31)
      Changed "Whenever Hanweir, the Writhing Township attacks" to "Whenever Hanweir attacks".

All tests pass (2155+), clippy clean, workspace build clean.

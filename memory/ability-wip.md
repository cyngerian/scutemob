# Ability WIP: Embalm

ability: Embalm
cr: 702.128
priority: P4
started: 2026-03-01
phase: fix
plan_file: memory/abilities/ability-plan-embalm.md

## Step Checklist
- [x] 1. Enum variant — types.rs:794 (KeywordAbility::Embalm), card_definition.rs:280 (AbilityDefinition::Embalm), stack.rs:630 (StackObjectKind::EmbalmAbility), command.rs:432 (Command::EmbalmCard), hash.rs (disc 92/27/25)
- [x] 2. Rule enforcement — abilities.rs: handle_embalm_card() + get_embalm_cost(); resolution.rs: EmbalmAbility arm; engine.rs: Command::EmbalmCard dispatch; replay_harness.rs: "embalm_card" action; view_model.rs + stack_view.rs: EmbalmAbility match arms
- [x] 3. Trigger wiring — n/a (no triggers needed for embalm)
- [x] 4. Unit tests — tests/embalm.rs: 12 tests all passing
- [x] 5. Card definition — Sacred Cat (crates/engine/src/cards/defs/sacred_cat.rs)
- [x] 6. Game script — 129_embalm_sacred_cat.json (stack/)
- [x] 7. Coverage doc update — Embalm row → validated (P4: 23/88)

## Review
findings: 6 (2 MEDIUM, 4 LOW)
review_file: memory/abilities/ability-review-embalm.md

## Fix Phase (complete)
- [x] MEDIUM #1 — resolution.rs:2286: `supertypes: def.types.supertypes.clone()` (was `im::OrdSet::new()`)
- [x] MEDIUM #2 — resolution.rs:2290-2294: Added TODO comment documenting systemic gap (no code change)
- LOW #3, #4, #5 deferred per review guidance; LOW #6 no action needed
phase: closed

# Ability WIP: Prototype

ability: Prototype
cr: 702.160
priority: P4
started: 2026-03-02
phase: closed
plan_file: memory/abilities/ability-plan-prototype.md

## Step Checklist
- [x] 1. Enum variant — KeywordAbility::Prototype (types.rs:870), AbilityDefinition::Prototype (card_definition.rs:358), GameObject.is_prototyped (game_object.rs:502), StackObject.was_prototyped (stack.rs:172), hash entries (hash.rs:530,705,1643,3197), commander identity (commander.rs:210)
- [x] 2. Rule enforcement — prototype validation + cost selection (casting.rs:971-998), prototype chars on stack (casting.rs:1680), prototype chars at resolution (resolution.rs:335); helper fns: get_prototype_data, colors_from_mana_cost (casting.rs:2957-3007)
- [x] 3. Trigger wiring — N/A (Prototype is static, no triggers per CR 702.160a)
- [x] 4. Unit tests — crates/engine/tests/prototype.rs (10 tests: basic_cast, normal_cast, color_change, mana_value, leaves_battlefield_resumes_normal, in_graveyard_normal_chars, retains_keyword_ability, negative_not_prototype_keyword, sba_toughness_check, stack_characteristics)
- [x] 5. Card definition — Blitz Automaton (crates/engine/src/cards/defs/blitz_automaton.rs)
- [x] 6. Game script — 135_prototype_blitz_automaton.json (stack/, validated)
- [x] 7. Coverage doc update — Prototype → validated (P4: 29/88, total validated: 121)

## Review
findings: 5 (2 HIGH, 1 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-prototype.md

## Fix Phase (2026-03-02)
- [x] HIGH 1 — Characteristics not reverted on zone change (CR 718.4): state/mod.rs move_object_to_zone:318-335, move_object_to_bottom_of_zone:440-457 — both `new_object` bindings made `mut`, revert block added after struct construction
- [x] HIGH 2 — Copy of prototyped spell not prototyped (CR 718.3c): rules/copy.rs:204 — `was_prototyped: original.was_prototyped` with CR citation
- [x] MEDIUM 3 — Test 5 doesn't verify characteristics revert: tests/prototype.rs — added power/toughness/mana_value/colors assertions after zone change
- LOW 4, LOW 5 — deferred per review instructions

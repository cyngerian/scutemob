# Ability WIP: Prowess

ability: Prowess
cr: 702.108
priority: P1
started: 2026-02-25
phase: closed
plan_file: memory/ability-plan-prowess.md

## Step Checklist
- [x] 1. Enum variant — `crates/engine/src/state/types.rs:123` + hash at `hash.rs:282`
- [x] 2. Rule enforcement — `EffectFilter::Source` at `continuous_effect.rs:98`, `hash.rs:548`, `effects/mod.rs:892`, `layers.rs:217`; `TriggerEvent::ControllerCastsNoncreatureSpell` at `game_object.rs:118`, `hash.rs:877`; SpellCast branch at `abilities.rs:314`
- [x] 3. Trigger wiring — Prowess `TriggeredAbilityDef` at `builder.rs:372`
- [x] 4. Unit tests — `crates/engine/tests/prowess.rs` (8 tests, all pass)
- [x] 5. Card definition — Monastery Swiftspear (`cards/definitions.rs:1432`); Haste + Prowess, 1/2, {R}
- [x] 6. Game script — `test-data/generated-scripts/stack/056_monastery_swiftspear_prowess_combat.json` (13/13 assertions pass)
- [x] 7. Coverage doc update — Prowess row: partial → validated; P1 validated 28→29; P1 total corrected to 42

## Review
findings: 3 (0 HIGH, 0 MEDIUM, 3 LOW)
review_file: memory/ability-review-prowess.md
verdict: clean

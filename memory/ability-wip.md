# Ability WIP: Fortify

ability: Fortify
cr: 702.67
priority: P4
started: 2026-03-07
phase: closed
plan_file: memory/abilities/ability-plan-fortify.md

## Step Checklist
- [x] 1. Enum variant — `state/types.rs:1194` (Fortify(u32) disc 130), `state/hash.rs:643`, `tools/replay-viewer/src/view_model.rs:850`
- [x] 2. Rule enforcement — `cards/card_definition.rs:850` (AttachFortification effect), `state/continuous_effect.rs:92` (AttachedLand filter), `effects/mod.rs:2063` (effect handler), `rules/abilities.rs:184` (activation validation), `rules/layers.rs:352` (AttachedLand filter match), `rules/events.rs:285` (FortificationAttached event), `state/hash.rs` (Effect disc 42, EffectFilter disc 13, GameEvent disc 100)
- [x] 3. Trigger wiring — n/a (Fortify is purely an activated ability, no triggers)
- [x] 4. Unit tests — `crates/engine/tests/fortify.rs` (7 tests: basic attach, sorcery speed, target must be land, controller ownership, reattach moves, SBA unattach, static ability via AttachedLand)
- [x] 5. Card definition (darksteel_garrison.rs)
- [x] 6. Game script (test-data/generated-scripts/stack/168_fortify_darksteel_garrison.json)
- [x] 7. Coverage doc update

## Review
findings: 3 (1 MEDIUM, 2 LOW) — all fixed 2026-03-07
review_file: memory/abilities/ability-review-fortify.md

## Fix Log
- MEDIUM #1 (CR 301.6 creature-Fortification check): added layer-aware source_is_creature guard in `effects/mod.rs` (after equip_id==target_id self-attach check) and `rules/abilities.rs` (before target validation block). effects/mod.rs:~2096, abilities.rs:~188
- LOW #2 (Fortify(u32) → unit variant): changed `KeywordAbility::Fortify(u32)` → `KeywordAbility::Fortify` in `state/types.rs:1201`, updated hash arm in `state/hash.rs:644`, updated view_model.rs:850
- LOW #3 (field rename equipment → fortification): renamed field in `cards/card_definition.rs:861`, updated pattern bindings in `effects/mod.rs:2073`, `state/hash.rs:3483`, `tests/fortify.rs:71`

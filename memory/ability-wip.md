# Ability WIP: Persist

ability: Persist
cr: 702.79
priority: P2
started: 2026-02-26
phase: closed
plan_file: memory/ability-plan-persist.md

## Step Checklist
- [x] 1. Enum variant — `crates/engine/src/state/types.rs:270`, hash.rs:355, view_model.rs:599
- [x] 2. Rule enforcement — `InterveningIf::SourceHadNoCounterOfType` in game_object.rs:157, hash.rs:1008, `CreatureDied.pre_death_counters` in events.rs:244, updated 8 emission sites across sba.rs/effects/mod.rs/abilities.rs/replacement.rs, extended `check_intervening_if` in abilities.rs:1163, resolution.rs:368
- [x] 3. Trigger wiring — `KeywordAbility::Persist` block in builder.rs:437, `ctx.source` update in MoveZone handler in effects/mod.rs:748
- [x] 4. Unit tests — `crates/engine/tests/persist.rs` (6 tests: basic return, no trigger with counter, second death, token, APNAP, counter annihilation)
## Review
findings: 2 (0 HIGH, 0 MEDIUM, 2 LOW)
review_file: memory/ability-review-persist.md
verdict: clean

- [x] 5. Card definition — Kitchen Finks (definitions.rs)
- [x] 6. Game script — combat/069_persist_kitchen_finks_returns_then_stays_dead.json
- [x] 7. Coverage doc update — Persist: none→validated; P2 validated 9→10

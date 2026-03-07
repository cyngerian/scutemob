# Ability WIP: Champion

ability: Champion
cr: 702.72
priority: P4
started: 2026-03-06
phase: close

## Review
findings: 4 (0 HIGH, 2 MEDIUM, 2 LOW)
review_file: memory/abilities/ability-review-champion.md
plan_file: memory/abilities/ability-plan-champion.md

## Fix Phase (complete)
- [x] MEDIUM 1 — Removed `KeywordAbility::Champion` guard from `ObjectExiled` and `ObjectReturnedToHand` arms in `abilities.rs`; now relies solely on `champion_exiled_card.is_some()` matching `CreatureDied`/`PermanentDestroyed` arms. CR 607.2a.
- [x] MEDIUM 2 — Added `test_champion_ltb_full_return_path` (Test 9) in `champion.rs`: places champion+fodder directly on battlefield/exile, sets `champion_exiled_card`, marks lethal damage, passes priority → SBA kills champion → LTB trigger fires → resolves → fodder returns; asserts all three stages.

## Step Checklist
- [x] 1. Enum variant — `KeywordAbility::Champion` (disc 126) in `state/types.rs`; `ChampionFilter` enum; hash in `state/hash.rs`; exported from `lib.rs`; arms in `view_model.rs` KeywordAbility match
- [x] 2. Rule enforcement — ETB trigger resolution (`resolution.rs:2751-2936`): exile qualifying permanent or sacrifice self; LTB trigger resolution (`resolution.rs:2938-3014`): return exiled card under owner's control; subtype filter via layer-resolved characteristics
- [x] 3. Trigger wiring — ETB trigger: `PermanentEnteredBattlefield` arm in `abilities.rs`; LTB trigger: `CreatureDied`/`PermanentDestroyed`/`ObjectExiled`/`ObjectReturnedToHand` arms; `flush_pending_triggers` arms for both; `StackObjectKind::ChampionETBTrigger` (disc 47) + `ChampionLTBTrigger` (disc 48); arms in `stack_view.rs` + `view_model.rs`; `champion_exiled_card` field on `GameObject` preserved across `move_object_to_zone`
- [x] 4. Unit tests — `crates/engine/tests/champion.rs`: 9 tests (8 original + Test 9 LTB full return path added in fix phase)
- [x] 5. Card definition — crates/engine/src/cards/defs/changeling_hero.rs (4/4 {3}{W}, Changeling + Lifelink + Champion a creature)
- [x] 6. Game script — test-data/generated-scripts/stack/164_champion_changeling_hero.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (Champion: validated)

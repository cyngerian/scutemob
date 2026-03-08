# Ability WIP: Haunt

ability: Haunt
cr: 702.55
priority: P4
started: 2026-03-08
phase: closed
plan_file: memory/abilities/ability-plan-haunt.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1332, hash.rs:677, game_object.rs:681, mod.rs:392/564, view_model.rs:884, stack.rs (HauntExileTrigger disc 57 / HauntedCreatureDiesTrigger disc 58), events.rs (HauntExiled disc 107), stubs.rs (HauntExile/HauntedCreatureDies PendingTriggerKind + haunt fields)
- [x] 2. Rule enforcement — resolution.rs: HauntExileTrigger resolution (move graveyard→exile, set haunting_target, emit HauntExiled); HauntedCreatureDiesTrigger resolution (look up card def, execute effect); counter-spell catch-all updated
- [x] 3. Trigger wiring — abilities.rs: CreatureDied handler fires HauntExile trigger (dies w/ Haunt kw) + HauntedCreatureDies trigger (scan exile for haunting_target == pre-death ObjectId); hash.rs: TriggerCondition::HauntedCreatureDies discriminant 23
- [x] 4. Unit tests — tests/haunt.rs: 8 tests (trigger fires, trigger resolves, exile with haunting_target, full lifecycle, no creatures fizzles, card removed from exile, exiled directly no trigger, multiplayer controller)
- [x] 5. Card definition (blind_hunter.rs — {2}{W}{B} 2/2 Flying Bat; ETB + HauntedCreatureDies → DrainLife 2)
- [x] 6. Game script — script_baseline_188 (test-data/generated-scripts/baseline/script_188_haunt.json): full lifecycle verified (ETB inline drain, HauntExileTrigger stack, HauntedCreatureDiesTrigger stack); PASS
- [x] 7. Coverage doc update

## Review
findings: 1 MEDIUM, 2 LOW
verdict: needs-fix
review_file: memory/abilities/ability-review-haunt.md

### Fix Required (MEDIUM)
1. **resolution.rs:4409-4414**: Clear `haunting_target` to `None` on the exiled haunt card after HauntedCreatureDiesTrigger resolves successfully. Without this, the stale haunting_target could cause spurious re-triggers if ObjectIds are ever recycled.

### Optional (LOW, deferred)
2. Auto-target selection is MVP — document as known simplification.
3. Test card effect is GainLife(0) — change to GainLife(2) and assert life total in full lifecycle test.

# Ability WIP: Reconfigure

ability: Reconfigure
cr: 702.151
priority: P4
started: 2026-03-08
phase: closed
plan_file: memory/abilities/ability-plan-reconfigure.md

## Step Checklist
- [x] 1. Enum variant — types.rs:1342 (KW 143), card_definition.rs:666 (AbilDef 58), card_definition.rs:1017 (Effect::DetachEquipment), game_object.rs:698 (is_reconfigured), hash.rs, view_model.rs
- [x] 2. Rule enforcement — effects/mod.rs:AttachEquipment sets is_reconfigured, DetachEquipment handler, layers.rs Layer4 type removal, sba.rs clears flag on SBA unattach + March-of-Machines check, abilities.rs unattach pre-check, replay_harness.rs enrich_spec_from_def expands Reconfigure into 2 abilities
- [x] 3. Trigger wiring — N/A (reconfigure is purely activated abilities + static type change)
- [x] 4. Unit tests — tests/reconfigure.rs: 8 tests (attach removes creature type, unattach restores, sorcery-speed, self-equip, equipped creature leaves, unattach when not attached, opponent's creature, artifact type retained)
- [x] 5. Card definition (lizard_blades.rs — {1}{R} 1/1 Double Strike + Reconfigure {2}; gives equipped +1/+1 + double strike)
- [x] 6. Game script — script_baseline_189 (test-data/generated-scripts/baseline/script_189_reconfigure.json); PASS 1/1
- [x] 7. Coverage doc update

## Review
findings: 3 LOW (no HIGH, no MEDIUM)
verdict: clean
review_file: memory/abilities/ability-review-reconfigure.md

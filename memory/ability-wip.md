# Ability WIP: Rampage

ability: Rampage
cr: 702.23
priority: P4
started: 2026-03-01
phase: closed
plan_file: memory/abilities/ability-plan-rampage.md

## Step Checklist
- [x] 1. Enum variant — types.rs:644 (Rampage(u32)), hash.rs:478 (discriminant 78)
- [x] 2. Rule enforcement — stack.rs:465 (RampageTrigger), resolution.rs:1722 (resolution arm), resolution.rs:1833 (counter passthrough), hash.rs:1415 (discriminant 20)
- [x] 3. Trigger wiring — stubs.rs:256 (PendingTrigger fields), abilities.rs:1575 (BlockersDeclared dispatch + Rampage tagging), abilities.rs:2385 (flush arm), builder.rs:722 (TriggeredAbilityDef)
- [x] 4. Unit tests — tests/rampage.rs (8 tests: blocked-by-2, blocked-by-1, scaled-3, multiple-instances, not-blocked, expiry, resolution-time, blocked-by-4)
- [x] 5. Card definition — crates/engine/src/cards/defs/wolverine_pack.rs (Wolverine Pack, {2}{G}{G}, 2/4, Rampage 2)
- [x] 6. Game script — test-data/generated-scripts/combat/116_wolverine_pack_rampage_three_blockers.json
- [x] 7. Coverage doc update — docs/mtg-engine-ability-coverage.md (P4 9/88, 101 total validated)

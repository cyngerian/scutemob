# Ability WIP: Plot

ability: Plot
cr: 702.170
priority: P4
started: 2026-03-02
phase: closed
plan_file: memory/abilities/ability-plan-plot.md

## Step Checklist
- [x] 1. Enum variant (types.rs:860,111; card_definition.rs:351; game_object.rs:488; stack.rs:167; hash.rs:528,3163,699,1631; events.rs:778; command.rs:354; mod.rs:plot; engine.rs:PlotCard; state/mod.rs:304,398; builder.rs:921; effects/mod.rs:2466; replay_harness.rs:plot_card,cast_spell_plot)
- [x] 2. Rule enforcement (rules/plot.rs NEW; casting.rs Plot validation+zone-bypass+mutual-exclusion+cost)
- [x] 3. Trigger wiring (N/A -- Plot has no triggers)
- [x] 4. Unit tests (tests/plot.rs: 20 tests, all passing)
- [x] 5. Card definition — Slickshot Show-Off (crates/engine/src/cards/defs/slickshot_show_off.rs)
- [x] 6. Game script — 134_plot_slickshot_show_off.json (stack/)
- [x] 7. Coverage doc update — Plot → validated (P4: 28/88, total validated: 120)

## Review
findings: 3 (0 HIGH, 0 MEDIUM, 3 LOW)
review_file: memory/abilities/ability-review-plot.md
fix_applied: none — verdict clean, 3 LOW deferred

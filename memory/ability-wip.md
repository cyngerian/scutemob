# Ability WIP: B15 Partner Variants

ability: B15 Partner Variants (FriendsForever, ChooseABackground, DoctorsCompanion)
cr: 702.124 (subrules i, k, m)
priority: P4
started: 2026-03-08
phase: review
plan_file: memory/abilities/ability-plan-b15-partner-variants.md

## Step Checklist
- [x] 1. Enum variants — FriendsForever/ChooseABackground/DoctorsCompanion added to state/types.rs:277-291
- [x] 2. Hash support — 3 arms added to state/hash.rs:682-687 (disc 144, 145, 146)
- [x] 3. View model arms — 3 display arms added to tools/replay-viewer/src/view_model.rs:892-898
- [x] 4. Validation logic — validate_partner_commanders() extended (cases 3-10) + is_legendary_background() + is_time_lord_doctor() helpers + Background creature-type exemption in per-commander loop (rules/commander.rs:498-699)
- [x] 5. Unit tests — 16 tests in crates/engine/tests/partner_variants.rs; all pass

## Review
findings: pending
verdict: pending
review_file: (none yet)

## Script Notes
(n/a — B15 has no game scripts)

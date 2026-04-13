# W3-LOW Session 2: Test Gaps — Review

**Review Status**: REVIEWED (2026-03-20)
**Reviewer**: Claude Opus 4.6 (milestone-reviewer agent)
**Branch**: `w3-low-s2-test-gaps`
**Tests**: 2229 passing (all green), clippy clean

## Files Changed

| File | Delta | Purpose |
|------|-------|---------|
| `crates/engine/tests/protection.rs` | +332 | 4 new tests: multicolor source protection (SR-PRO-03) and subtype-based protection (SR-PRO-04) |
| `crates/engine/tests/combat.rs` | +183 | 1 new test: first-strike vs first-strike (SR-FS-03), plus SR-FS-02 deferred documentation comment |
| `crates/engine/tests/card_def_fixes.rs` | +95 | 1 new test: Thought Vessel counter-assertion (MR-M9.4-15) — other players still discard |
| `test-data/generated-scripts/combat/205_attacker_survives_blocker_dies.json` | +192 | New combat script: 3/3 attacker kills 2/2 blocker, attacker survives (MR-M6-08/MR-M7-16) |

## CR Sections Tested

| CR Section | Test |
|------------|------|
| CR 702.16a | `test_protection_from_red_blocks_multicolor_red_source` — multicolor source shares one protected color |
| CR 702.16a | `test_protection_from_red_allows_green_only_multicolor_source` — positive control, no color overlap |
| CR 702.16a/b | `test_protection_from_subtype_goblin_blocks_goblin_source` — subtype-quality protection blocks targeting |
| CR 702.16b | `test_protection_from_subtype_goblin_allows_wizard_source` — non-matching subtype allowed |
| CR 702.7b | `test_sr_fs03_first_strike_vs_first_strike_damage_only_in_fs_step` — both FS creatures deal in FS step |
| CR 402.2 / 514.1 | `test_thought_vessel_only_affects_its_controller_other_players_discard` — NoMaxHandSize per-controller |
| CR 508.1/509.1/510.1c/510.2/704.5g | Script 205: full combat sequence, attacker survives with damage marked |

## Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| W3S2-01 | **LOW** | `205_attacker_survives_blocker_dies.json` | **Script references missing card definitions.** "Grizzly Bears" and "Bear Cub" have no `CardDefinition` in the registry. The script is correctly marked `pending_review` so it won't run in CI, but it cannot be promoted to `approved` until those card defs exist. **Fix:** Author card definitions for Grizzly Bears and Bear Cub, or swap to cards that already have definitions in the registry. | OPEN |
| W3S2-02 | **LOW** | `combat.rs:~2466` | **First-strike test uses raw `characteristics.name` for object lookup.** The test does `o.characteristics.name == "FS Attacker"` for object lookup during test setup. This is fine for tests (not engine code), and the objects are builder-created with no continuous effects, but it diverges from the pattern used in `protection.rs` which has a dedicated `find_object()` helper. Minor inconsistency. **Fix:** Extract a `find_object()` helper in `combat.rs` or use the existing inline pattern consistently. No functional impact. | OPEN |
| W3S2-03 | **INFO** | `combat.rs:~2596` | **SR-FS-02 deferred documentation is thorough.** The block comment explains why the test is deferred, what existing tests cover the underlying mechanism, and what infrastructure is needed. Well-documented deferral. | N/A |

## Test Quality Assessment

| Test | Quality | Notes |
|------|---------|-------|
| `test_protection_from_red_blocks_multicolor_red_source` | Good | Negative test: multicolor (red+green) spell correctly blocked by protection from red. Non-tautological — actually casts the spell and checks for error. |
| `test_protection_from_red_allows_green_only_multicolor_source` | Good | Positive control: pure-green spell is NOT blocked. Paired with the negative test above — important to verify both directions. |
| `test_protection_from_subtype_goblin_blocks_goblin_source` | Good | Negative test: Goblin-subtype instant blocked by protection from Goblins. Tests 702.16a's subtype clause. |
| `test_protection_from_subtype_goblin_allows_wizard_source` | Good | Positive control: Wizard-subtype instant passes through protection from Goblins. |
| `test_sr_fs03_first_strike_vs_first_strike_damage_only_in_fs_step` | Good | Comprehensive: verifies both creatures die in FS step, total damage is correct, zero regular-step damage, both objects removed from battlefield. Cites CR 702.7b correctly. |
| `test_thought_vessel_only_affects_its_controller_other_players_discard` | Good | Tests the per-controller nature of NoMaxHandSize. P1 has Thought Vessel, but P2 (active player) still discards. Counter-assertion to existing test. |
| Script 205 | Blocked | Well-structured script with proper CR citations, but fails due to missing card defs (W3S2-01). |

## Summary

- **0 HIGH, 0 MEDIUM, 2 LOW, 1 INFO** findings
- All 6 new tests pass and are non-tautological (each one exercises a specific engine path and checks for the expected outcome)
- CR citations are accurate (verified via MCP rules lookup: 702.16a, 702.16b, 702.7b, 402.2, 514.1, 704.5g)
- No accidental behavioral changes to existing tests (diff is additive only)
- Protection tests follow the good pattern of pairing negative tests (blocked) with positive controls (allowed)
- The combat script (205) is well-written but blocked on missing card definitions
- No fix phase needed (LOW-only findings); address opportunistically

# W3 LOW Sprint S5 Review

**Review Status**: REVIEWED (2026-03-20)
**Branch**: `w3-low-s5-quick-fixes` (1 commit: `11f6691`)
**Baseline**: `main` at `1425216`

## Files Changed

| File | +/- | Purpose |
|------|-----|---------|
| `crates/engine/src/state/hash.rs` | +41/-1 | TC-23: explicit HashInto for AltCostKind with stable discriminant constants 0-26; refactored GameObject hash to use new impl |
| `crates/engine/src/state/player.rs` | +13/-7 | MR-M1-13: saturating_add on ManaPool::total() and ManaPool::add() |
| `crates/engine/src/rules/loop_detection.rs` | +9 | MR-M9.4-12: doc comment explaining why check_for_mandatory_loop takes &mut GameState |
| `crates/engine/src/rules/resolution.rs` | +5 | MR-B12-09: doc comment explaining why RavenousDrawTrigger Err is intentionally dropped |
| `crates/engine/tests/partner_variants.rs` | +37 | MR-B15-02: test for Doctor's Companion with missing Doctor subtype |
| `crates/engine/tests/alliance.rs` | +136/-4 | MR-B12-06: test verifying Alliance fires on token creation via Effect::CreateToken |
| `crates/engine/tests/mutate.rs` | +244/-1 | MR-Mutate-01 + MR-Mutate-02: tests for mutate stack data invariant and mutate onto face-down creature |

**Total**: +485/-13 lines across 7 files. All tests pass (0 failures).

## Issues Addressed

| Original Issue | Status | Notes |
|---------------|--------|-------|
| MR-M1-13 (overflow in ManaPool) | CLOSED | saturating_add on total() and add() |
| MR-B15-02 (Doctor's Companion test gap) | CLOSED | New test covers missing Doctor subtype case |
| MR-B12-09 (undocumented Err drop) | CLOSED | Comment explains SBA-based loss per CR 704.5b |
| MR-M9.4-12 (&mut GameState rationale) | CLOSED | Doc comment explains loop_detection_hashes side effect |
| TC-23 (AltCostKind hash fragility) | CLOSED | Explicit HashInto with 27 stable discriminant constants |
| MR-Mutate-01 (stack data invariant) | CLOSED | Test verifies AdditionalCost::Mutate on stack object |
| MR-Mutate-02 (face-down mutate target) | CLOSED | Test verifies mutate onto Morph creature accepted |
| MR-B12-06 (Alliance token creation) | CLOSED | Test verifies Alliance fires on Effect::CreateToken |

## Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| W3S5-01 | **LOW** | `player.rs:71` | **total_with_restricted() still uses non-saturating add.** `total_with_restricted()` computes `self.total() + self.restricted.iter().map(\|r\| r.amount).sum::<u32>()` using `+` which can overflow even though `total()` now uses `saturating_add`. Inconsistent overflow protection. **Fix:** Change to `self.total().saturating_add(self.restricted.iter().map(\|r\| r.amount).sum::<u32>())`. | OPEN |
| W3S5-02 | **LOW** | `resolution.rs:4365` | **CR 121.3 citation is tangential.** Comment cites CR 121.3 (about offering a *choice* to draw) but Ravenous triggers a mandatory draw, not a choice. CR 704.5b (also cited) is the correct and sufficient reference. CR 121.3 could be removed from the comment without losing accuracy. **Fix:** Remove "CR 121.3" from the comment, keep "CR 704.5b". | OPEN |
| W3S5-03 | **INFO** | `hash.rs:956-962` | **Redundant manual Option hash pattern.** The `if let Some(k) / else` block for `cast_alt_cost` is functionally identical to `self.cast_alt_cost.hash_into(hasher)` now that `AltCostKind` has a `HashInto` impl and the generic `Option<T: HashInto>` impl exists (line 120). The manual code hashes `true/false` which maps to `1u8/0u8`, matching the generic impl's `1u8/0u8`. Not a bug -- the explicit form may be intentional for readability -- but the one-liner is available. | OPEN |

## Detailed Analysis

### TC-23: AltCostKind HashInto (hash.rs)

**Correctness: GOOD.** All 27 variants of `AltCostKind` are mapped to unique, sequential constants 0-26. The match is exhaustive (no `_ =>` catch-all), so any future variant addition will cause a compile error, forcing the author to assign a new discriminant. This is the correct pattern for hash stability.

**Hash compatibility: PRESERVED.** The old code used `k as u8` which produces the Rust-assigned discriminant (declaration order). The new explicit constants match that order exactly: Flashback=0, Buyback=1, ..., Prototype=26. Combined with the identical `Option` wrapping pattern (0u8/1u8 prefix), the hash output is byte-identical for all existing states. No hash migration needed.

**Cross-check:** No other site in `hash.rs` uses `AltCostKind as u8` (verified via grep). The `StackObject` struct does not have an `alt_cost` field, so no secondary hash site exists.

### MR-M1-13: ManaPool saturating_add (player.rs)

**Correctness: GOOD.** Both `total()` and `add()` now use `saturating_add`, preventing panic on u32 overflow. The overflow scenario is extremely unlikely in normal play but could be triggered by infinite loops or edge-case mana generation.

**Gap:** `total_with_restricted()` still uses `+` (finding W3S5-01). Low risk since restricted mana pools are typically small, but the inconsistency should be noted.

### New Tests

**test_doctors_companion_doctor_missing_doctor_subtype_rejected (partner_variants.rs):**
Correctly tests the inverse case of the existing "missing Time Lord" test. A creature with only "Time Lord" subtype but not "Doctor" should fail `is_time_lord_doctor()`. CR 702.124m citation is correct. Test is well-structured: builds a CardDefinition with explicit subtype, calls `validate_partner_commanders()`, asserts error.

**test_alliance_fires_on_create_token_effect (alliance.rs):**
Tests that a creature with an ETB trigger creating a 1/1 Saproling token via `Effect::CreateToken` fires an Alliance trigger on a separate creature. The test correctly asserts `initial_life + 2` (one fire for Token Creator entering, one for the Saproling token). CR 207.2c / CR 603.2 citations are correct. Test structure is sound: CardRegistry with a real CardDefinition, Alliance creature on battlefield, cast + resolve + verify.

**test_mutate_onto_face_down_creature_accepted (mutate.rs):**
Tests that a face-down Morph creature (no visible subtypes per CR 708.2) passes the non-Human check for Mutate targeting (CR 702.140a). Correctly sets `face_down_as = Some(FaceDownKind::Morph)` on the target after building state. Asserts the CastSpell succeeds and the stack has one object. CR citations are correct.

**test_mutate_stack_object_has_mutate_additional_cost (mutate.rs):**
Verifies the data model invariant that a MutatingCreatureSpell on the stack has an `AdditionalCost::Mutate` entry with the correct target. This validates the structural contract needed for CR 729.8 (copy of mutating spell inherits target). Test is straightforward: cast with mutate, inspect stack object's `additional_costs`.

### Doc Comments

**loop_detection.rs:** The added doc comment clearly explains why `check_for_mandatory_loop` takes `&mut GameState` despite being conceptually read-only. The mutation of `loop_detection_hashes` is correctly noted as excluded from the public hash. Good documentation.

**resolution.rs:** The added comment correctly explains that drawing from an empty library doesn't cause inline game loss -- it's handled by SBA (CR 704.5b). The CR 121.3 citation is tangential (see W3S5-02) but the behavioral claim is correct.

## Summary

- **0 HIGH, 0 MEDIUM, 2 LOW, 1 INFO** findings
- All 8 targeted issues properly addressed
- No behavioral regressions (all tests pass)
- Hash compatibility preserved (byte-identical output for existing states)
- New tests are well-structured with correct CR citations
- No fix phase needed -- LOWs can be addressed opportunistically

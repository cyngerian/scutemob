# Ability Review: Intimidate

**Date**: 2026-02-25
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.13
**Files reviewed**: `crates/engine/src/rules/combat.rs:453-473`, `crates/engine/src/state/types.rs:114`, `crates/engine/src/state/hash.rs:274`, `crates/engine/tests/keywords.rs:632-969`

## Verdict: clean

The Intimidate implementation is correct and complete. The blocking restriction at
`combat.rs:453-473` matches CR 702.13b exactly: it checks (1) whether the blocker is an
artifact creature and (2) whether the blocker shares at least one color with the attacker,
rejecting the block only if neither exception applies. Colorless attackers are handled
correctly because iterating an empty color set yields `false` for the color-sharing check.
The layer system is respected via `calculate_characteristics`, and all seven tests cover
the core scenarios including the colorless-attacker edge case and flying+intimidate
stacking. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/keywords.rs:1-13` | **Module doc missing Intimidate entry.** The module-level doc comment lists all tested keywords but omits Intimidate. **Fix:** Add `//! - Intimidate: blocking restriction — artifact creatures and/or color-sharing (CR 702.13)` to the list. |
| 2 | LOW | `tests/keywords.rs:632-969` | **No Menace+Intimidate interaction test.** Plan documents Menace+Intimidate interaction (plan item 4 under Interactions) but no test verifies that both restrictions apply simultaneously (2+ blockers required, each must be artifact or share color). **Fix:** Add `test_702_13_intimidate_plus_menace_both_restrictions_apply` with a multicolor intimidate+menace attacker, two valid color-sharing blockers, assert Ok; then one valid + one invalid blocker, assert Err. |
| 3 | LOW | `state/types.rs:114` | **No doc comment on Intimidate variant.** `Intimidate` has no CR citation doc comment. Other simple keyword variants (Deathtouch, Defender, Flying, etc.) also lack doc comments, so this is consistent with the existing style. Only noting for completeness. **Fix:** Optionally add `/// CR 702.13a: Intimidate is an evasion ability.` above the variant. |

### Finding Details

#### Finding 1: Module doc missing Intimidate entry

**Severity**: LOW
**File**: `crates/engine/tests/keywords.rs:1-13`
**CR Rule**: 702.13 -- "Intimidate"
**Issue**: The module-level doc comment at the top of `keywords.rs` enumerates all tested keyword abilities (Defender, Haste, Flying/Reach, Hexproof/Shroud, Indestructible, Menace, Lifelink) but does not include Intimidate. The seven Intimidate tests are present starting at line 632, but the doc header is stale.
**Fix**: Add `//! - Intimidate: blocking restriction -- artifact creatures and/or color-sharing (CR 702.13)` to the module doc between the Indestructible and Menace entries (after line 8).

#### Finding 2: No Menace+Intimidate interaction test

**Severity**: LOW
**File**: `crates/engine/tests/keywords.rs:632-969`
**CR Rule**: 702.13b + 702.110a -- both restrictions apply independently
**Issue**: The plan identifies Menace+Intimidate as a key interaction (plan section "Interactions to Watch", item 4). Menace requires 2+ blockers; Intimidate restricts which creatures can block. Each individual blocker must satisfy Intimidate, and the total count must satisfy Menace. The implementation handles this correctly (Intimidate is checked per-blocker in the per-pair loop; Menace is checked per-attacker in the aggregate count loop after), but there is no test verifying this interaction.
**Fix**: Add a test `test_702_13_intimidate_plus_menace_both_restrictions_apply` that creates an attacker with both Intimidate and Menace, two blockers that share a color with the attacker, and asserts the combined block succeeds. Then test with only one valid blocker and assert failure (menace requires 2+). This is a LOW gap because the logic is structurally sound -- Intimidate and Menace checks are in separate code blocks with no interaction.

#### Finding 3: No doc comment on Intimidate variant

**Severity**: LOW
**File**: `crates/engine/src/state/types.rs:114`
**CR Rule**: 702.13a -- "Intimidate is an evasion ability."
**Issue**: The `Intimidate` enum variant has no doc comment citing CR 702.13. However, this is consistent with the existing pattern: simple keyword variants (Deathtouch, Defender, DoubleStrike, Enchant, Equip, FirstStrike, Flash, Flying, Haste, Hexproof, Indestructible, Landwalk, Lifelink, Menace, Prowess, Reach, Shroud, Trample, Vigilance) all lack doc comments. Only complex variants with data fields (ProtectionFrom, Ward, Partner, NoMaxHandSize) have doc comments.
**Fix**: Optionally add `/// CR 702.13a: Intimidate is an evasion ability.` to maintain discoverability. Not required given the existing style.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.13a (evasion ability classification) | Yes | Implicit | Static restriction, no trigger/stack interaction -- correct categorization |
| 702.13b (blocking restriction: artifact creature exception) | Yes | Yes | `test_702_13_intimidate_allows_artifact_creature_blocker`, `test_702_13_intimidate_colorless_attacker_artifact_creature_blocks` |
| 702.13b (blocking restriction: color-sharing exception) | Yes | Yes | `test_702_13_intimidate_allows_same_color_blocker` |
| 702.13b (blocking restriction: basic enforcement) | Yes | Yes | `test_702_13_intimidate_blocks_non_matching_creature` |
| 702.13b (colorless attacker: no shared colors) | Yes | Yes | `test_702_13_intimidate_colorless_attacker_only_artifact_can_block` |
| 702.13b (multicolor: partial match suffices) | Yes | Yes | `test_702_13_intimidate_multicolor_attacker_allows_partial_color_match` |
| 702.13b (uses current colors via layer system) | Yes | Implicit | `calculate_characteristics` applies all layers including Layer 5 ColorChange; no explicit color-change test but structurally correct |
| 702.13c (multiple instances redundant) | Yes | Implicit | `OrdSet<KeywordAbility>` deduplicates automatically; no explicit test needed |
| Interaction: Flying + Intimidate | Yes | Yes | `test_702_13_intimidate_plus_flying_both_must_be_satisfied` |
| Interaction: CantBeBlocked + Intimidate | Yes | Implicit | CantBeBlocked checked first (line 443), rejects all blockers before Intimidate |
| Interaction: Protection + Intimidate | Yes | Implicit | Protection checked independently (line 477); both restrictions apply |
| Interaction: Menace + Intimidate | Yes | No | Structurally correct but untested (Finding 2) |

## Verification Notes

- **Authoritative CR text verified via MCP**: CR 702.13a-c matches the plan exactly.
- **Rulings verified via MCP**: Hideous Visage (multicolor partial match), Surrakar Marauder / Guul Draz Vampire / Halo Hunter / Bladetusk Boar (current colors, not printed colors; once blocked, color changes don't matter). All rulings are consistent with the implementation.
- **No `.unwrap()` in engine code**: Confirmed. The implementation uses `?` propagation and `Err` returns.
- **Hash coverage**: Confirmed at `hash.rs:274` with discriminant `11u8`.
- **All match arms**: `Intimidate` has no data, so no pattern matching required beyond `contains()` checks.
- **Immutable state**: The check is read-only against `attacker_chars` and `blocker_chars`. No state mutation.

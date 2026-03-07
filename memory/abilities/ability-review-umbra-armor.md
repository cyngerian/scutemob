# Ability Review: Umbra Armor

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.89
**Files reviewed**:
- `crates/engine/src/state/types.rs:1174` (KeywordAbility::UmbraArmor)
- `crates/engine/src/state/hash.rs:636-637` (KW disc 127), `hash.rs:2841-2849` (GameEvent disc 99)
- `crates/engine/src/rules/events.rs:870-883` (GameEvent::UmbraArmorApplied)
- `crates/engine/src/rules/replacement.rs:1696-1781` (check_umbra_armor + apply_umbra_armor)
- `crates/engine/src/rules/sba.rs:411-428` (umbra armor wiring in dying loop)
- `crates/engine/src/effects/mod.rs:575-592` (umbra armor wiring in DestroyPermanent)
- `tools/replay-viewer/src/view_model.rs:844` (UmbraArmor keyword display)
- `crates/engine/tests/umbra_armor.rs` (10 tests)

## Verdict: needs-fix

Implementation is largely correct: the replacement effect fires at both destruction sites
(SBA and DestroyPermanent), correctly guards behind `is_destruction`, removes damage and
deathtouch flag, uses layer-resolved characteristics (respects Humility/Dress Down), and
does not tap or remove from combat. Hash coverage is complete. However, two issues were
found: one MEDIUM (phased-out Aura bypass) and one MEDIUM (Aura destruction bypasses
zone-change replacement effects).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `replacement.rs:1718` | **Phased-out Aura bypass.** Missing `is_phased_in()` check. **Fix:** add phasing filter. |
| 2 | MEDIUM | `replacement.rs:1770-1772` | **Zone-change replacements skipped on Aura destruction.** Direct `move_object_to_zone` bypasses commander redirect. **Fix:** route through `check_zone_change_replacement`. |
| 3 | LOW | `replacement.rs:1709-1710` | **Regen-before-umbra ordering is hardcoded.** CR 616.1 requires controller choice. TODO is present but no tracking issue. |
| 4 | LOW | `tests/umbra_armor.rs` | **No test for multiple umbra armor Auras.** Plan edge case #3 (controller chooses which Aura) has no test coverage. |
| 5 | LOW | `replacement.rs:1720-1722` | **Auto-select first Aura in multi-Aura case.** Plan says controller should choose; current logic picks `auras[0]` which is HashMap iteration order (non-deterministic). |

### Finding Details

#### Finding 1: Phased-out Aura bypass

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/replacement.rs:1718`
**CR Rule**: 702.26b -- "Phased-out permanents are treated as though they do not exist."
**Issue**: `check_umbra_armor` filters for `aura_obj.zone == ZoneId::Battlefield` but does
not check `aura_obj.is_phased_in()`. A phased-out Aura with umbra armor would incorrectly
be returned as a valid protector. Per CR 702.26b, phased-out permanents are treated as though
they do not exist, so their static abilities (including umbra armor) should not function.
All other SBA battlefield scans in `sba.rs` include the `is_phased_in()` filter (lines 129,
163, 660, 775, 1020).
**Fix**: Add `&& aura_obj.is_phased_in()` to the filter chain at line 1718, after the zone
check. Specifically:
```rust
if !matches!(aura_obj.zone, ZoneId::Battlefield) || !aura_obj.is_phased_in() {
    return None;
}
```

#### Finding 2: Zone-change replacements skipped on Aura destruction

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/replacement.rs:1770-1772`
**CR Rule**: 614.1 -- "Some continuous effects are replacement effects." + 903.9a (commander
zone-change replacement)
**Issue**: `apply_umbra_armor` calls `state.move_object_to_zone(aura_id, ZoneId::Graveyard(aura_owner))`
directly, which is a raw zone move. When the normal SBA destruction path processes a dying
permanent, it first calls `check_zone_change_replacement` (sba.rs:436) to handle effects like
commander redirect to the command zone. By bypassing this, if the Aura is a commander (e.g., a
commander enchantment creature that somehow has umbra armor, or if a future card grants umbra
armor to arbitrary permanents), it would go to the graveyard instead of being redirected to
the command zone. The plan's own step 2b notes: "zone-change replacement effects on IT should
still apply (e.g., a commander Aura could redirect to command zone)" but the implementation
does not follow through on this.
**Fix**: Before calling `move_object_to_zone`, call `check_zone_change_replacement(state,
aura_id, ZoneType::Battlefield, ZoneType::Graveyard, aura_owner, &HashSet::new())` and
handle the `Redirect`/`NeedsChoice` cases, or at minimum add a `pending_zone_changes` entry
if a replacement applies. Alternatively, mark this as a known gap with a TODO citing the
specific scenario (commander Aura with umbra armor) if the full replacement path is too
complex for this phase.

#### Finding 3: Regen-before-umbra ordering is hardcoded

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:1709-1710` and `sba.rs:416-417`
**CR Rule**: 616.1 -- controller chooses which replacement effect to apply when multiple apply.
**Issue**: Both the SBA path and the DestroyPermanent path check regeneration shields before
umbra armor, with no controller choice. This is noted with a TODO but has no tracking issue.
The practical impact is minimal (having both regen and umbra armor on the same permanent is
rare), but it is technically a CR violation.
**Fix**: No immediate fix required. This is correctly documented as a TODO. Consider adding
to the LOW issues remediation doc for tracking.

#### Finding 4: No test for multiple umbra armor Auras

**Severity**: LOW
**File**: `crates/engine/tests/umbra_armor.rs`
**CR Rule**: 616.1 -- controller chooses which Aura to destroy when multiple umbra armor Auras
protect the same permanent.
**Issue**: The plan identifies this as edge case #3 but no test was written. The current code
auto-selects `auras[0]` which is non-deterministic (HashMap iteration order). A test would
document the expected behavior even if the auto-select simplification is acceptable for now.
**Fix**: Add a test `test_umbra_armor_multiple_auras_one_consumed` that attaches two umbra
armor Auras to one creature, destroys it, and verifies: (a) creature survives, (b) exactly
one Aura is in the graveyard, (c) exactly one Aura remains on the battlefield.

#### Finding 5: Non-deterministic Aura selection in multi-Aura case

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:1720-1722` (via `check_umbra_armor` return
used at `sba.rs:423` and `effects/mod.rs:586`)
**CR Rule**: 616.1 -- controller chooses.
**Issue**: `check_umbra_armor` returns a `Vec<ObjectId>` collected from `state.objects.iter()`.
Since `state.objects` is an `im::HashMap`, iteration order is not insertion-ordered. The
callers take `auras[0]`, which means the "chosen" Aura varies by hash seed. For deterministic
replay, the selection should be stable (e.g., sort by ObjectId).
**Fix**: Add `.sorted()` (or collect-and-sort) to the returned Vec before returning from
`check_umbra_armor`, or sort in the callers before selecting `auras[0]`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.89a (replacement effect) | Yes | Yes | Tests 1, 2, 3, 8 |
| 702.89a (damage removal) | Yes | Yes | Tests 1, 2, 3, 8 |
| 702.89a (destroy the Aura) | Yes | Yes | Tests 1, 2, 3, 10 |
| 702.89b (old name errata) | Yes (name is UmbraArmor) | N/A | Cosmetic |
| 704.5g (lethal damage SBA) | Yes | Yes | Test 2 |
| 704.5h (deathtouch SBA) | Yes | Yes | Test 3 |
| 704.5f (zero toughness - NOT destruction) | Yes | Yes | Test 4 |
| 701.8b (sacrifice not destruction) | Yes | Yes | Test 7 |
| Not-regeneration (no tap, no remove from combat) | Yes | Yes | Tests 1, 5 |
| Indestructible priority | Yes | Yes | Test 6 |
| Non-creature permanent | Yes | Yes | Test 10 |
| GameEvent emitted | Yes | Yes | Test 9 |
| Multiple umbra armor (CR 616.1) | Partial (auto-select) | No | Finding 4 |
| Phased-out Aura (CR 702.26b) | No | No | Finding 1 |
| Zone-change replacement on Aura (CR 614) | No | No | Finding 2 |

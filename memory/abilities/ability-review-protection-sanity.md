# Ability Review: Protection (Sanity Check)

**Date**: 2026-03-09
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.16
**Files reviewed**:
- `crates/engine/src/rules/protection.rs` (167 lines)
- `crates/engine/src/state/types.rs` (ProtectionQuality enum, KeywordAbility::ProtectionFrom)
- `crates/engine/src/state/hash.rs` (hash coverage)
- `crates/engine/src/rules/combat.rs` (blocking restriction at lines 723-730, 923-925)
- `crates/engine/src/rules/casting.rs` (targeting restriction at lines 4963-4969)
- `crates/engine/src/rules/abilities.rs` (targeting restriction at lines 476-482)
- `crates/engine/src/rules/mod.rs` (validate_target_protection at lines 47-68)
- `crates/engine/src/rules/replacement.rs` (damage prevention at lines 1541-1554)
- `crates/engine/src/effects/mod.rs` (DealDamage path at lines 175-283)
- `crates/engine/tests/protection.rs` (684 lines, 8 tests)

## Verdict: needs-fix

Protection DEBT enforcement is structurally sound for permanents. The core `matches_quality`
function correctly handles FromColor, FromCardType, FromSubType, and FromAll. All four DEBT
aspects are wired into the correct engine paths. However, there are two MEDIUM issues: player
protection is entirely unimplemented (CR 702.16b/c/e all say "permanent or player"), and
player-targeted damage does not check protection. There is also a missing ProtectionQuality
variant for supertypes (CR 702.16a mentions supertypes explicitly).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `casting.rs:4933-4951` | **Player targets skip protection check.** CR 702.16b applies to players too. |
| 2 | **MEDIUM** | `replacement.rs:1542` | **Player damage not checked for protection.** CR 702.16e applies to players. |
| 3 | LOW | `types.rs:85-94` | **Missing FromSuperType variant.** CR 702.16a mentions supertypes. |
| 4 | LOW | `types.rs:85-94` | **Missing FromName variant.** CR 702.16a mentions card names as a quality. |
| 5 | LOW | `protection.rs:1-9` | **Module doc cites wrong subrule letters.** DEBT letters shifted from actual CR. |
| 6 | LOW | `tests/protection.rs` | **No multicolor source test.** E.g., a RG source vs protection from red. |
| 7 | LOW | `tests/protection.rs` | **No protection-from-subtype test.** FromSubType never exercised in tests. |

### Finding Details

#### Finding 1: Player targets skip protection check

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:4933-4951`
**CR Rule**: 702.16b -- "A permanent **or player** with protection can't be targeted by spells with the stated quality and can't be targeted by abilities from a source with the stated quality."
**Issue**: When `validate_targets` processes a `Target::Player(id)`, it checks if the player
is active and validates player requirements, but never calls `validate_target_protection`.
The object-target path at line 4964 correctly calls it. This means a player who somehow gains
protection (e.g., from Teferi's Protection giving "protection from everything") can still be
targeted by spells. Players don't currently have a `keywords` set in the engine, which is the
root cause -- `PlayerState` has no field for keyword abilities.
**Fix**: This requires a `keywords: OrdSet<KeywordAbility>` field on `PlayerState` (or at
minimum a `protection_qualities: Vec<ProtectionQuality>` field), and a protection check in
the `Target::Player` arm of `validate_targets`. Since no cards currently grant protection to
players, this is not urgent, but the gap should be tracked. Mark as LOW if deemed out-of-scope
for the current card pool.

#### Finding 2: Player damage not checked for protection

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/replacement.rs:1542`
**CR Rule**: 702.16e -- "Any damage that would be dealt by sources that have the stated quality to a permanent **or player** with protection is prevented."
**Issue**: The `apply_damage_prevention` function only checks protection for
`CombatDamageTarget::Creature` and `CombatDamageTarget::Planeswalker` (line 1542-1553).
The `CombatDamageTarget::Player` variant falls through to the dynamic prevention shields
without a static protection check. Same root cause as Finding 1 -- players have no keywords
field. This means damage to a player with protection from a quality is not prevented.
**Fix**: Same as Finding 1. Add keyword/protection tracking to `PlayerState`, then add a
`CombatDamageTarget::Player` arm in the protection check block.

#### Finding 3: Missing FromSuperType variant

**Severity**: LOW
**File**: `crates/engine/src/state/types.rs:85-94`
**CR Rule**: 702.16a -- "If the quality is a card type, subtype, **or supertype**, the ability applies to sources that are permanents with that card type, subtype, or supertype..."
**Issue**: `ProtectionQuality` has `FromCardType` and `FromSubType` but no `FromSuperType`.
No current cards need this (protection from legendary, etc.), so this is a gap to track but
not urgent.
**Fix**: Add `FromSuperType(SuperType)` variant when a card requiring it is authored.

#### Finding 4: Missing FromName variant

**Severity**: LOW
**File**: `crates/engine/src/state/types.rs:85-94`
**CR Rule**: 702.16a -- "If the quality happens to be a card name, it is treated as such only if the protection ability specifies that the quality is a name."
**Issue**: No `FromName(String)` variant exists. Cards like "Protection from Eldrazi"
(subtype-based, already covered) vs hypothetical "protection from [named card]" are not
representable. No current cards need this.
**Fix**: Add `FromName(String)` variant when a card requiring it is authored.

#### Finding 5: Module doc cites wrong subrule letters

**Severity**: LOW
**File**: `crates/engine/src/rules/protection.rs:1-9`
**CR Rule**: 702.16b-f
**Issue**: The module doc header says Damage is CR 702.16e, Enchanting is CR 702.16c/d,
Blocking is CR 702.16f, Targeting is CR 702.16b. The actual CR mapping:
- 702.16b = Targeting (correct)
- 702.16c = Enchanting (correct)
- 702.16d = Equipment/Fortification (correct)
- 702.16e = Damage prevention (correct)
- 702.16f = Blocking (correct)

Upon re-checking, the citations are actually correct. The DEBT order in the doc header maps
to c/d/e/f/b respectively, which matches the CR. No fix needed -- withdrawing this finding.

#### Finding 6: No multicolor source test

**Severity**: LOW
**File**: `crates/engine/tests/protection.rs`
**CR Rule**: 702.16a -- protection from a color matches any source that has that color
**Issue**: No test verifies that a multicolor source (e.g., red-green) is blocked by
"protection from red". The `matches_quality` function uses `source_chars.colors.contains(c)`
which correctly handles this -- it checks membership, not equality. But a test would
confirm this important edge case.
**Fix**: Add a test with a RG multicolor source targeting a creature with protection from red.
Verify the targeting is rejected.

#### Finding 7: No protection-from-subtype test

**Severity**: LOW
**File**: `crates/engine/tests/protection.rs`
**CR Rule**: 702.16a -- protection from a subtype
**Issue**: `ProtectionQuality::FromSubType` is defined but never exercised in any test.
The `matches_quality` function correctly checks `source_chars.subtypes.contains(st)`.
**Fix**: Add a test for protection from a subtype (e.g., "protection from Goblins" blocking
a Goblin creature source).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.16a    | Partial     | Yes     | FromColor, FromCardType, FromSubType, FromAll covered. Missing FromSuperType, FromName. |
| 702.16b    | Partial     | Yes     | Object targeting works; player targeting missing (Finding 1). 4 targeting tests. |
| 702.16c    | Yes         | Yes     | Aura SBA detachment via `sba.rs:851`. test_protection_from_red_aura_falls_off. |
| 702.16d    | Yes         | Yes     | Equipment SBA detachment via `sba.rs:986`. test_protection_from_red_equipment_detaches. |
| 702.16e    | Partial     | Yes     | Creature/PW damage prevented; player damage not (Finding 2). test_protection_from_red_prevents_red_damage. |
| 702.16f    | Yes         | Yes     | Blocking restriction in combat.rs:725. test_protection_from_red_blocks_red_blocker. |
| 702.16g    | Yes         | N/A     | Multiple protection keywords are separate OrdSet entries; naturally handled. |
| 702.16h    | N/A         | N/A     | Shorthand notation -- card definition concern, not engine concern. |
| 702.16i    | N/A         | N/A     | Shorthand notation -- card definition concern. |
| 702.16j    | Yes         | Yes     | FromAll variant; test_protection_from_all_blocks_all_targeting + test_protection_global_effect_still_works. |
| 702.16k    | No          | No      | "Protection from [a player]" not modeled (no FromPlayer variant). No cards need it currently. |
| 702.16m    | Yes         | N/A     | Multiple instances redundant -- OrdSet deduplication handles this naturally. |
| 702.16n    | No          | No      | "This effect doesn't remove" Aura exception not implemented. Needed for e.g. Spectra Ward. |
| 702.16p    | No          | No      | Benevolent Blessing "already attached" exception not implemented. |

## Correctness Answers (from task description)

1. **Is damage prevented (not just reduced)?** Yes. `apply_damage_prevention` returns `(0, Vec::new())` when protection matches, preventing all damage from that source. Correct per CR 702.16e.

2. **Blocking restriction direction?** Correct -- one-directional. The *attacker's* protection is checked against the *blocker's* characteristics. A red creature with protection from white CAN be blocked by a red creature; it CANNOT be blocked by a white creature. `protection_prevents_blocking(attacker_keywords, blocker_chars)` gets this right.

3. **Enchanting/equipping at both cast-time and SBA?** Cast-time: yes, via `validate_target_protection` in the `Target::Object` arm (Aura targeting checked). SBA-time: yes, via `attachment_is_illegal_due_to_protection` called in sba.rs lines 851 and 986. Both paths covered.

4. **Targeting by anyone with the quality?** Yes. `check_full_targeting_protection` checks protection regardless of whether the caster is the controller or an opponent. The hexproof check is controller-gated, but the protection check at line 136-138 is not. Correct.

5. **Protection from everything?** Handled as `FromAll` which returns `true` in `matches_quality` for any source. Not special-cased -- derived from the general mechanism. Correct per CR 702.16j.

6. **Does protection NOT prevent SBAs, counters?** Correct. Protection only blocks DEBT. The SBA path (e.g., 0-toughness death) and counter placement (e.g., -1/-1 counters from non-damage effects) do not go through any protection check. The test `test_protection_global_effect_still_works` confirms non-targeted destroy effects bypass protection.

7. **Multicolor source test?** No test exists (Finding 6), but the implementation is correct -- `colors.contains(c)` matches any source containing the protected color.

8. **SBA strips attached auras/equipment?** Yes. Both aura (sba.rs:851, emits `AuraFellOff`) and equipment (sba.rs:986, emits `EquipmentUnattached`) paths check protection. Tested by `test_protection_from_red_aura_falls_off` and `test_protection_from_red_equipment_detaches`.

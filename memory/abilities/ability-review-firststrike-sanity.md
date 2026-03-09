# Ability Review: First Strike (CR 702.7) + Double Strike (CR 702.4)

**Date**: 2026-03-09
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.7 (First Strike), 702.4 (Double Strike), 510.4 (two-step combat damage)
**Files reviewed**:
- `crates/engine/src/rules/combat.rs` (lines 1086-1660)
- `crates/engine/src/rules/turn_structure.rs` (lines 1-70)
- `crates/engine/src/rules/turn_actions.rs` (lines 1721-1737)
- `crates/engine/src/state/turn.rs` (Step::FirstStrikeDamage definition + next())
- `crates/engine/src/state/combat.rs` (first_strike_damage_resolved field)
- `crates/engine/tests/combat.rs` (all first/double strike tests)
- `test-data/generated-scripts/combat/004_first_strike_kills_before_damage.json`
- `test-data/generated-scripts/combat/006_double_strike_damage_steps.json`
- `test-data/generated-scripts/combat/104_first_strike_deathtouch_kills_before_normal_damage.json`

## Verdict: needs-fix

One MEDIUM finding: the `deals_damage_in_step` function checks current keywords
instead of snapshotting keywords at the start of the first combat damage step, as
CR 702.7b requires. This produces incorrect results when a creature gains or loses
first strike between the two combat damage steps. The practical impact is low today
(no current cards/tests modify keywords mid-combat), but the logic is architecturally
wrong and will produce incorrect behavior once instant-speed keyword-granting effects
exist (e.g., combat tricks granting first strike).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `combat.rs:1626-1635` | **No first-strike snapshot.** `deals_damage_in_step` checks current keywords, not keywords at start of first damage step. **Fix:** snapshot first/double-strike status at step entry. |
| 2 | LOW | `combat.rs:1626-1635` + `turn_actions.rs:1727` | **Dead field.** `first_strike_damage_resolved` is set but never read. **Fix:** either use it in the snapshot mechanism or remove it. |
| 3 | LOW | `tests/combat.rs` | **Missing test: gain/lose first strike between steps.** CR 702.7c and CR 702.4c/d describe explicit edge cases not tested. **Fix:** add tests when snapshot mechanism exists. |
| 4 | LOW | `tests/combat.rs` | **Missing test: first strike vs first strike.** No test where both attacker and blocker have first strike (both deal damage simultaneously in the first step, nothing happens in the regular step). CC#20 covers FS-blocker vs DS-attacker but not FS vs FS. **Fix:** add a simple FS-vs-FS test. |

### Finding Details

#### Finding 1: No first-strike keyword snapshot (MEDIUM)

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/combat.rs:1626-1635`
**CR Rule**: 702.7b -- "the remaining attackers and blockers that had neither first strike nor double strike **as the first combat damage step began**"
**CR Rule**: 702.7c -- "Giving first strike to a creature without it after combat damage has already been dealt in the first combat damage step **won't preclude** that creature from assigning combat damage in the second combat damage step. Removing first strike from a creature after it has already dealt combat damage in the first combat damage step **won't allow** it to also assign combat damage in the second combat damage step (unless the creature has double strike)."
**CR Rule**: 702.4c -- "Removing double strike from a creature during the first combat damage step will stop it from assigning combat damage in the second combat damage step."
**CR Rule**: 702.4d -- "Giving double strike to a creature with first strike after it has already dealt combat damage in the first combat damage step will allow the creature to assign combat damage in the second combat damage step."

**Issue**: The function `deals_damage_in_step` evaluates the creature's *current* keywords
at the time each damage step is processed. Per CR 702.7b, the determination of which
creatures deal damage in the second step is based on which creatures **had** first strike
or double strike at the **beginning** of the first combat damage step. The CR explicitly
calls out four edge cases (702.7c, 702.4c, 702.4d) that produce different outcomes depending
on whether keywords changed between steps.

Current behavior when keywords change mid-combat:
- Creature gains first strike after first step: incorrectly **excluded** from regular step
  (702.7c says it should still deal damage)
- Creature loses first strike after first step: incorrectly **included** in regular step
  (702.7c says it should not deal damage again)
- Creature loses double strike after first step: currently **excluded** from regular step
  (702.4c says this is correct -- but for the wrong reason; the code happens to get the
  right answer because `!has_first` is true when both are false)
- Creature with first strike gains double strike after first step: currently **included**
  in regular step (702.4d says this is correct -- again, right answer, wrong mechanism)

**Practical impact**: Low today. No current tests or cards modify keywords between damage
steps. But the logic is structurally wrong and will produce bugs once combat tricks
(instants granting/removing first strike during combat) are implemented.

**Fix**: At the start of `first_strike_damage_step()`, snapshot which creatures have
first strike and/or double strike. Store this on `CombatState` (e.g.,
`had_first_strike: OrdSet<ObjectId>`, `had_double_strike: OrdSet<ObjectId>`).
In `deals_damage_in_step`, for the regular step (`first_strike_step == false`), consult
the snapshot to determine eligibility. The `first_strike_damage_resolved` field already
exists and could serve as the "snapshot was taken" flag.

#### Finding 2: Dead field `first_strike_damage_resolved` (LOW)

**Severity**: LOW
**File**: `crates/engine/src/state/combat.rs:50`, `crates/engine/src/rules/turn_actions.rs:1727`
**Issue**: The field `first_strike_damage_resolved` is initialized to `false`, set to `true`
after the first-strike damage step, and included in the hash -- but is never *read* by any
logic. It appears to have been intended for use in the snapshot mechanism described in
Finding 1 but was never wired up.
**Fix**: Wire it into the snapshot mechanism from Finding 1, or remove it if not needed.

#### Finding 3: Missing test for gain/lose first strike between steps (LOW)

**Severity**: LOW
**File**: `crates/engine/tests/combat.rs`
**CR Rule**: 702.7c, 702.4c, 702.4d
**Issue**: The CR defines four explicit edge cases for keyword changes between damage steps.
None are tested. These are difficult to test without instant-speed keyword manipulation
infrastructure, but should be documented as known gaps.
**Fix**: Add tests when the snapshot mechanism (Finding 1) and combat-trick infrastructure
exist. For now, add a `// TODO(CR 702.7c): ...` comment in the test file.

#### Finding 4: Missing test for first strike vs first strike (LOW)

**Severity**: LOW
**File**: `crates/engine/tests/combat.rs`
**CR Rule**: 702.7b, 510.4
**Issue**: No test verifies that when both attacker and blocker have first strike, they
deal damage simultaneously in the first-strike step and nothing happens in the regular
step. Test `test_cc20_first_strike_blocks_double_strike` covers FS-blocker vs DS-attacker
(close but not identical). A FS-vs-FS test would confirm no damage leaks into the regular
step.
**Fix**: Add `test_702_7_first_strike_vs_first_strike_simultaneous()` with a 3/3 FS attacker
and a 2/4 FS blocker. Verify: both deal damage in the first step, the 3/3 survives (took 2,
has 3 toughness), the blocker survives (took 3, has 4 toughness), and no damage is dealt in
the regular step.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.7a     | Yes         | Yes     | First strike is static, modifies combat damage step |
| 702.7b     | Partial     | Yes     | Two-step logic works; snapshot missing (Finding 1) |
| 702.7c     | No          | No      | Gain/lose FS between steps not handled (Finding 1, 3) |
| 702.7d     | Yes         | N/A     | Redundant instances -- OrdSet deduplication handles this |
| 702.4a     | Yes         | Yes     | Double strike is static, modifies combat damage step |
| 702.4b     | Yes         | Yes     | DS deals damage in both steps: test_702_4_double_strike_deals_in_both_steps |
| 702.4c     | No          | No      | Remove DS during first step -- no snapshot (Finding 1, 3) |
| 702.4d     | No          | No      | Gain DS after first step -- no snapshot (Finding 1, 3) |
| 702.4e     | Yes         | N/A     | Redundant instances -- OrdSet deduplication handles this |
| 510.4      | Yes         | Yes     | Conditional first-strike step insertion via should_have_first_strike_step |

## Correctness Questions Answered

1. **Skip first-strike step when no FS/DS?** Yes. `should_have_first_strike_step()` returns false,
   `advance_step()` goes directly to `CombatDamage`. Verified by `test_510_unblocked_attacker_deals_damage_to_player`.

2. **FS creature dies in first step, no re-damage in regular step?** Yes. Dead creatures are
   removed from battlefield; `apply_combat_damage` checks `zone == ZoneId::Battlefield`.
   Verified by `test_702_7_first_strike_kills_blocker_before_regular_damage`.

3. **DS creature deals damage in both steps?** Yes. `deals_damage_in_step` returns true for
   `has_double` in both branches. Verified by `test_702_4_double_strike_deals_in_both_steps`.

4. **Creature gains FS after blockers but before damage?** Partially correct. If gained before
   the step begins, `should_have_first_strike_step()` sees it and creates the step.
   `deals_damage_in_step` correctly includes it. However, the snapshot issue (Finding 1) means
   the regular step would incorrectly exclude it. **Not tested.**

5. **FS creature kills blocker; blocker damage in FS step?** Correctly handled. Within a step,
   damage is simultaneous (CR 510.2). A blocker without FS doesn't deal damage in the FS step
   (`deals_damage_in_step` returns false). Verified by `test_702_7_first_strike_kills_blocker_before_regular_damage`.

6. **DS + trample test?** Yes. `test_702_19d_trample_blockers_removed_before_damage` tests a
   4/4 DS+Trample vs 1/1 blocker. First step: 1 lethal + 3 trample. Regular step: blocker gone,
   4 trample to player. Total: 7 damage (40 -> 33). Correct.

7. **Two FS creatures on opposite sides?** Not directly tested. CC#20 tests FS-blocker vs
   DS-attacker (close but not identical). See Finding 4.

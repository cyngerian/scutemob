# Ability Review: Ninjutsu

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.49
**Files reviewed**:
- `crates/engine/src/state/types.rs:746-761` (KeywordAbility variants)
- `crates/engine/src/cards/card_definition.rs:264-278` (AbilityDefinition variants)
- `crates/engine/src/state/hash.rs:509-512,1514-1526,3067-3076` (hash discriminants)
- `crates/engine/src/state/stack.rs:583-602` (StackObjectKind::NinjutsuAbility)
- `crates/engine/src/rules/command.rs:399-419` (Command::ActivateNinjutsu)
- `crates/engine/src/rules/engine.rs:341-358` (command dispatch)
- `crates/engine/src/rules/abilities.rs:804-1040` (handle_ninjutsu + get_ninjutsu_cost)
- `crates/engine/src/rules/resolution.rs:2070-2202` (NinjutsuAbility resolution)
- `crates/engine/src/rules/resolution.rs:2318` (counter_stack_object arm)
- `crates/engine/src/testing/replay_harness.rs:239-241,517-530,1147-1160` (harness action + helper)
- `crates/engine/src/testing/script_schema.rs:264-268` (attacker_name field)
- `crates/engine/tests/script_replay.rs:150,169` (attacker_name threading)
- `tools/replay-viewer/src/view_model.rs:497-499,723-724` (view model arms)
- `tools/tui/src/play/panels/stack_view.rs:104-105` (TUI stack arm)
- `crates/engine/tests/ninjutsu.rs` (12 tests)

## Verdict: needs-fix

The implementation is architecturally sound and covers CR 702.49a-d accurately. Timing
validation, owner-vs-controller handling, attack target inheritance, zone checks, ETB site
pattern, combat state cleanup, and commander ninjutsu are all correct. There are two
HIGH findings for `.unwrap()` and `.expect()` calls in engine library code, violating the
project convention. No CR correctness issues were found.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `abilities.rs:910` | **`.unwrap()` in engine library code.** Guarded by prior `is_none()` check but violates convention. **Fix:** replace with safe alternative. |
| 2 | **HIGH** | `abilities.rs:929,933` | **`.unwrap()` + `.expect()` in engine library code.** Both logically safe but violate "never unwrap/expect in engine logic." **Fix:** replace with safe alternatives. |
| 3 | LOW | `abilities.rs:804-1020` | **CR 702.49b reveal tracking not implemented.** Card should remain revealed while ability is on stack. StackObjectKind stores the card reference, so UI can identify it, but no explicit reveal flag. Deferred -- hidden-info gap consistent with project-wide LOWs. |
| 4 | LOW | `tests/ninjutsu.rs` | **No negative test for attacker controlled by different player.** Validation exists at line 904 but no test exercises it. |

### Finding Details

#### Finding 1: `.unwrap()` on combat state in engine library code

**Severity**: HIGH
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:910`
**Convention**: `memory/conventions.md` -- "Engine crate uses typed errors -- never `unwrap()` or `expect()` in engine logic."
**Issue**: Line 910 uses `state.combat.as_ref().unwrap()` to access combat state. While this is logically safe because line 851 returns early if `state.combat.is_none()`, the project convention prohibits `.unwrap()` in engine library code unconditionally. The borrow checker cannot prove the guard, so a future refactor could silently remove the guard path and introduce a panic.
**Fix**: Replace with:
```rust
let combat = state.combat.as_ref().ok_or_else(|| {
    GameStateError::InvalidCommand("ActivateNinjutsu: no active combat state".into())
})?;
```

#### Finding 2: `.unwrap()` + `.expect()` on combat state and attacker lookup

**Severity**: HIGH
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:929,933`
**Convention**: `memory/conventions.md` -- "Engine crate uses typed errors -- never `unwrap()` or `expect()` in engine logic."
**Issue**: Lines 926-933 use `.unwrap()` on `state.combat.as_ref()` (same pattern as Finding 1) and `.expect()` on the attacker lookup in `combat.attackers.get()`. Both are logically guarded by prior checks: combat existence (line 851) and attacker membership (line 911). However, the convention is absolute: no `.unwrap()` or `.expect()` in engine logic.
**Fix**: Replace the entire block with:
```rust
let attack_target = state
    .combat
    .as_ref()
    .and_then(|c| c.attackers.get(&attacker_to_return).cloned())
    .ok_or_else(|| {
        GameStateError::InvalidCommand(
            "ActivateNinjutsu: could not retrieve attack target from combat state".into(),
        )
    })?;
```

#### Finding 3: CR 702.49b reveal tracking not implemented

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:804-1020`
**CR Rule**: 702.49b -- "The card with ninjutsu remains revealed from the time the ability is announced until the ability leaves the stack."
**Issue**: The implementation does not explicitly track the revealed state of the ninja card while the ability is on the stack. The `StackObjectKind::NinjutsuAbility` stores `source_object` and `ninja_card` ObjectIds, allowing a UI to identify the card, but there is no `revealed: bool` flag on the card object or similar mechanism. This is consistent with the project's existing hidden-information gaps (~40 LOW deferred) and does not affect game-state correctness.
**Fix**: Deferred. When addressing hidden-information LOWs holistically, add a revealed flag or mechanism. The current approach (card identity stored on stack object) is sufficient for correctness.

#### Finding 4: Missing negative test for attacker controlled by different player

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/ninjutsu.rs`
**CR Rule**: 702.49a -- "Return an unblocked attacking creature **you control**"
**Issue**: The validation at `abilities.rs:904` checks `obj.controller != player` and rejects the command, but no test verifies this negative case (e.g., P2's creature attacking P3, P1 tries to ninjutsu using P2's attacker). Test 8 (`test_ninjutsu_returns_to_owner_not_controller`) tests the positive case where P1 controls P2's creature -- the inverse is not tested.
**Fix**: Add a test where the attacker is controlled by a different player than the ninjutsu activator. Expect `InvalidCommand` error.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.49a (basic ninjutsu) | Yes | Yes | test_ninjutsu_basic_swap |
| 702.49a (owner's hand) | Yes | Yes | test_ninjutsu_returns_to_owner_not_controller |
| 702.49a (ninja stays in hand until resolution) | Yes | Yes | test_ninjutsu_ninja_leaves_hand_before_resolution |
| 702.49b (card remains revealed) | No | No | LOW -- hidden-info gap, Finding 3 |
| 702.49c (unblocked requirement) | Yes | Yes | test_ninjutsu_blocked_attacker_rejected |
| 702.49c (attacks same target) | Yes | Yes | test_ninjutsu_ninja_attacks_same_target |
| 702.49c (timing: after blockers declared) | Yes | Yes | test_ninjutsu_wrong_step_rejected (3 sub-tests) |
| 702.49d (commander ninjutsu from command zone) | Yes | Yes | test_commander_ninjutsu_from_command_zone |
| 702.49d (bypasses commander tax) | Yes | Yes | test_commander_ninjutsu_from_command_zone |
| 508.3a (not "declared as attacker") | Yes | Yes | test_ninjutsu_not_declared_as_attacker |
| 508.4 (put onto battlefield attacking) | Yes | Yes | test_ninjutsu_basic_swap (combat.attackers check) |
| 508.4a (target invalid at resolution) | Yes | No | Resolution code handles it; no dedicated test |
| 508.4c (no attack restrictions) | N/A | N/A | Attack restrictions not yet enforced in engine |
| 702.61a (split second blocks) | Yes | Yes | test_ninjutsu_split_second_blocks |
| 400.7 (zone change identity) | Yes | Yes | test_ninjutsu_ninja_leaves_hand_before_resolution |
| Combat damage delivery | Yes | Yes | test_ninjutsu_combat_damage |
| Multiplayer correctness | Yes | Yes | test_ninjutsu_multiplayer_four_player |
| ETB triggers fire | Yes | Implicit | Resolution calls fire_when_enters_triggered_effects |
| Mana cost payment | Yes | Yes | test_ninjutsu_basic_swap (mana deducted) |
| Combat state cleanup (stale attacker removal) | Yes | Yes | test_ninjutsu_basic_swap (old attacker gone from combat.attackers) |

## Notes

### Correctness Highlights

1. **Timing validation** (CR 702.49c): Correctly restricts to DeclareBlockers, FirstStrikeDamage, CombatDamage, and EndOfCombat steps. DeclareAttackers and BeginningOfCombat are correctly excluded.

2. **Owner vs controller** (CR 702.49a): Line 965 correctly uses `state.object(attacker_to_return)?.owner` and moves to `ZoneId::Hand(attacker_owner)`. Test 8 verifies this.

3. **Attack target inheritance** (CR 702.49c): Attack target is captured at line 926 BEFORE the attacker is returned to hand (line 966). The resolution at line 2143 inserts the ninja into `combat.attackers` with the inherited target. Not the generic "controller chooses" rule from 508.4.

4. **No "attacks" trigger** (CR 508.3a/508.4): The resolution does NOT emit `AttackersDeclared`. Test 3 explicitly verifies this.

5. **Commander ninjutsu tax bypass** (CR 702.49d): Ninjutsu is an activated ability, not a spell cast. The handler never touches `commander_tax`. Test 12 verifies tax remains 0. This matches ruling 2020-11-10 (Yuriko).

6. **Ninja leaves hand** (CR 400.7): Resolution checks `still_in_zone` at line 2097-2101 before proceeding. If the ninja left the expected zone, the ability resolves and does nothing. Test 7 verifies this.

7. **Combat state cleanup**: Line 970-972 removes the stale attacker ObjectId from `combat.attackers` after `move_object_to_zone` creates a new ObjectId (CR 400.7).

8. **ETB site pattern**: Resolution follows the full 6-step ETB pattern: self-ETB replacements, global ETB replacements, register replacement abilities, register static continuous effects, emit PermanentEnteredBattlefield, fire WhenEntersBattlefield triggers.

9. **Hash coverage**: All three hash impls updated -- KeywordAbility (87, 88), AbilityDefinition (22, 23), StackObjectKind (26). All fields hashed. No collisions with existing discriminants.

10. **Replay harness**: `find_in_command_zone` helper added. `activate_ninjutsu` action type tries hand first, then command zone. `attacker_name` field added to script schema with `#[serde(default)]`.

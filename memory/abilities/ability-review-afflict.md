# Ability Review: Afflict

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.130
**Files reviewed**:
- `crates/engine/src/state/types.rs:674-682` (KeywordAbility::Afflict variant)
- `crates/engine/src/state/hash.rs:483-487` (KeywordAbility::Afflict hash, discriminant 80)
- `crates/engine/src/state/builder.rs:764-782` (Afflict -> TriggeredAbilityDef generation)
- `crates/engine/src/state/game_object.rs:199-205` (TriggerEvent::SelfBecomesBlocked -- pre-existing, shared)
- `crates/engine/src/state/hash.rs:1175-1176` (TriggerEvent::SelfBecomesBlocked hash -- pre-existing)
- `crates/engine/src/rules/abilities.rs:1642-1704` (SelfBecomesBlocked dispatch + defending_player_id tagging)
- `crates/engine/src/effects/mod.rs:319-335` (LoseLife effect -- pre-existing, no changes)
- `tools/replay-viewer/src/view_model.rs:701` (Afflict display arm)
- `crates/engine/tests/afflict.rs` (6 tests, 565 lines)

## Verdict: clean

The Afflict implementation is correct, minimal, and well-documented. It correctly leverages
the existing `SelfBecomesBlocked` trigger infrastructure (shared with Bushido and Rampage),
the `defending_player_id` -> `Target::Player` pipeline (shared with Annihilator), and the
pre-existing `Effect::LoseLife` (not damage, as required by CR 702.130a and rulings). No
new StackObjectKind, no new Effect, no changes to flush_pending_triggers -- the implementation
is purely additive and narrow. All CR 702.130 subrules are implemented and tested. Two LOW
findings exist (incomplete doc comment, minor assertion gap) but neither affects correctness
or warrants blocking.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `game_object.rs:199-201` | **SelfBecomesBlocked doc comment does not mention Afflict.** Doc lists Bushido and Rampage but not Afflict. **Fix:** Update doc comment to include "the Afflict keyword (CR 702.130a)". |
| 2 | LOW | `afflict.rs:549-553` | **LifeLost event assertion uses wildcard match.** `matches!(e, GameEvent::LifeLost { .. })` does not verify the correct player or amount. A more specific match would improve test rigor. **Fix:** Change to `matches!(e, GameEvent::LifeLost { player, amount } if *player == p2 && *amount == 3)`. |

### Finding Details

#### Finding 1: SelfBecomesBlocked doc comment does not mention Afflict

**Severity**: LOW
**File**: `crates/engine/src/state/game_object.rs:199-201`
**CR Rule**: 702.130a -- "Afflict is a triggered ability."
**Issue**: The doc comment on `TriggerEvent::SelfBecomesBlocked` lists Bushido (CR 702.45a)
and Rampage (CR 702.23a) as users of this trigger event, but does not mention Afflict
(CR 702.130a), which is now the third keyword using this trigger. This was also noted in the
Rampage review (Finding 3) for the omission of Rampage itself; that fix was applied but
Afflict was not added when Afflict was implemented.
**Fix**: Update the doc comment to:
```rust
/// CR 509.1h / CR 702.45a / CR 702.23a / CR 702.130a: Triggers when this attacking creature
/// becomes blocked. Used by Bushido (CR 702.45a), Rampage (CR 702.23a), and Afflict (CR 702.130a).
```

#### Finding 2: LifeLost event assertion uses wildcard match

**Severity**: LOW
**File**: `crates/engine/tests/afflict.rs:549-553`
**CR Rule**: N/A (test quality)
**Issue**: The test `test_afflict_life_loss_not_damage` asserts that a `LifeLost` event exists
in the resolution events using `matches!(e, GameEvent::LifeLost { .. })`, but does not verify
that the event's `player` field is `p2` or that the `amount` is `3`. While the life total
assertion on line 542-546 provides the primary correctness check, a more specific event
assertion would catch regressions where the event is emitted but with wrong metadata (e.g.,
wrong player or amount).
**Fix**: Change the assertion to:
```rust
assert!(
    resolution_events.iter().any(|e| matches!(
        e,
        GameEvent::LifeLost { player, amount } if *player == p2 && *amount == 3
    )),
    "Afflict: LifeLost event should target P2 with amount 3"
);
```

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.130a (trigger on "becomes blocked") | Yes | Yes | test_702_130a_afflict_basic_life_loss |
| 702.130a (defending player loses N life) | Yes | Yes | test_702_130a_afflict_basic_life_loss (asserts life delta) |
| 702.130a (life loss, not damage) | Yes | Yes | test_afflict_life_loss_not_damage (asserts no DamageDealt event) |
| 702.130a (no trigger when unblocked) | Yes | Yes | test_702_130a_afflict_not_blocked_no_trigger |
| 702.130b (multiple instances trigger separately) | Yes | Yes | test_702_130b_afflict_multiple_instances_trigger_separately |
| 509.3c (single trigger for multiple blockers) | Yes | Yes | test_509_3c_afflict_multiple_blockers_single_trigger |
| 508.5 (correct defending player in multiplayer) | Yes | Yes | test_afflict_multiplayer_correct_defending_player (4-player) |

## Additional Notes

### Correct aspects of the implementation

1. **Life loss via LoseLife effect, not DealDamage**: The plan correctly identifies that
   Afflict causes life loss, not damage (CR 702.130a: "defending player loses N life"). The
   implementation uses `Effect::LoseLife`, which emits `GameEvent::LifeLost` (not
   `GameEvent::DamageDealt`). This means afflict correctly bypasses damage prevention,
   does not interact with lifelink, and does not count as combat damage for commander
   damage tracking. Test 6 explicitly verifies this.

2. **defending_player_id pipeline reuse**: The implementation correctly reuses the
   `PendingTrigger.defending_player_id` -> `flush_pending_triggers` -> `Target::Player(dp)`
   at index 0 -> `PlayerTarget::DeclaredTarget { index: 0 }` pipeline. This is the same
   pipeline used by Annihilator (CR 702.86a). No modifications to flush_pending_triggers
   were needed.

3. **SelfBecomesBlocked infrastructure reuse**: The `TriggerEvent::SelfBecomesBlocked`
   variant was already created by Bushido (Batch 2) and is shared with Rampage. The dedup
   logic (`blocked_attackers.sort(); blocked_attackers.dedup()`) correctly ensures one
   trigger per attacker per combat (CR 509.3c). Afflict adds no new dispatch code -- it
   simply adds a new `TriggeredAbilityDef` in builder.rs that fires on the same trigger.

4. **Multiple instances via builder.rs loop**: Each `KeywordAbility::Afflict(n)` in the
   keywords set generates its own `TriggeredAbilityDef`, so a creature with Afflict 2 and
   Afflict 1 gets two separate triggered ability definitions. When the creature becomes
   blocked, `collect_triggers_for_event` finds both and creates two PendingTriggers. This
   correctly implements CR 702.130b.

5. **No new StackObjectKind**: Afflict triggers use the standard `TriggeredAbility` stack
   kind, which is correct. The LoseLife effect resolves through the normal effect execution
   pipeline. No custom resolution logic is needed.

6. **Hash coverage complete**: `KeywordAbility::Afflict(n)` has discriminant 80 with the
   `n` value hashed (hash.rs:483-487). No new TriggerEvent or StackObjectKind variants were
   added, so no additional hash arms were needed.

7. **Multiplayer test quality**: Test 5 uses a 4-player setup with two attackers attacking
   different defenders, verifying that only the correct defending player loses life. This is
   a strong test for the `defending_player_id` tagging correctness.

8. **Tagging safety for Bushido/Rampage**: The `defending_player_id` tagging on all
   `SelfBecomesBlocked` triggers (line 1701-1703) is safe for Bushido and Rampage because
   their effects use `CEFilter::Source` (Bushido) and a custom `RampageTrigger`
   StackObjectKind (Rampage), neither of which references `PlayerTarget::DeclaredTarget`.
   The comment at lines 1697-1700 explicitly documents this safety property.

### Interaction with existing abilities verified

- **Bushido**: Shares `SelfBecomesBlocked`. Bushido's `ApplyContinuousEffect` with
  `CEFilter::Source` ignores the `defending_player_id` target entirely. No conflict.
- **Rampage**: Shares `SelfBecomesBlocked`. Rampage uses a custom `RampageTrigger`
  StackObjectKind which reads blocker count from combat state. No conflict with the
  `defending_player_id` tagging.
- **Annihilator**: Uses `SelfAttacks` (not `SelfBecomesBlocked`), so no overlap in trigger
  dispatch. Shares the `defending_player_id` -> `Target::Player` pipeline pattern.

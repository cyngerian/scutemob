# Ability Review: Provoke

**Date**: 2026-03-01
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.39
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Provoke variant)
- `crates/engine/src/state/combat.rs` (forced_blocks field on CombatState)
- `crates/engine/src/state/stack.rs` (StackObjectKind::ProvokeTrigger)
- `crates/engine/src/state/stubs.rs` (PendingTrigger provoke fields)
- `crates/engine/src/state/builder.rs` (TriggeredAbilityDef auto-generation)
- `crates/engine/src/state/hash.rs` (all hash coverage)
- `crates/engine/src/rules/abilities.rs` (trigger tagging + flush)
- `crates/engine/src/rules/resolution.rs` (ProvokeTrigger resolution)
- `crates/engine/src/rules/combat.rs` (forced-block enforcement in handle_declare_blockers)
- `crates/engine/tests/provoke.rs` (7 tests)
- `tools/replay-viewer/src/view_model.rs` (format_keyword + convert_stack_object_kind)
- `tools/tui/src/play/panels/stack_view.rs` (ProvokeTrigger match arm)

## Verdict: needs-fix

The implementation is architecturally sound and follows the established patterns
(StackObjectKind, PendingTrigger tagging, builder auto-generation). CR 702.39a/b
core behavior is correctly implemented: the trigger fires on SelfAttacks, resolves
to untap the provoked creature and insert a forced-block requirement, and the
forced-block enforcement in handle_declare_blockers covers all major evasion
restrictions. However, there is one MEDIUM finding: when a creature has multiple
Provoke instances (CR 702.39b), both triggers deterministically target the SAME
creature instead of different creatures. There is also one LOW finding regarding
missing controller validation at resolution time.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `abilities.rs:1416-1447` | **Multiple Provoke instances target same creature.** Both triggers select `.next()` from the same iterator, targeting the same creature. CR 702.39b says each triggers separately, implying different targets. **Fix:** track already-targeted creatures and skip them. |
| 2 | LOW | `resolution.rs:1802-1807` | **Missing controller check at resolution.** Target validity only checks zone, not controller. **Fix:** also verify provoked creature is controlled by the defending player. |
| 3 | LOW | `provoke.rs:390-448` | **Test 5 does not verify different targets.** Only checks trigger count=2 and stack size=2, does not assert the two triggers target different creatures. **Fix:** add assertion checking provoked_creature differs between the two stack objects. |
| 4 | LOW | `combat.rs:598-800` | **Menace+Provoke interaction not holistic.** Sequential validation (menace check before provoke check) can create impossible declarations. Noted as general limitation of sequential approach per CR 509.1c. **Fix:** no immediate fix needed; document as known limitation. |

### Finding Details

#### Finding 1: Multiple Provoke instances target same creature

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:1416-1447`
**CR Rule**: 702.39b -- "If a creature has multiple instances of provoke, each triggers separately."
**Issue**: When a creature has two (or more) Provoke keyword instances, the builder correctly
generates two TriggeredAbilityDef entries, and collect_triggers_for_event correctly produces two
PendingTrigger entries. However, the tagging loop at lines 1416-1447 iterates over all newly
collected triggers and for each provoke trigger, selects the target via:
```rust
let target = state.objects.values()
    .filter(|o| { /* defending player's creature on battlefield */ })
    .map(|o| o.id)
    .next(); // Always picks the first by ObjectId
```
Both triggers independently call `.next()` on freshly constructed iterators, so both get the
same first creature. In a real game, the player would choose a different target for each trigger
(or could choose the same, but the engine should not force it). The deterministic fallback should
at least attempt to select different targets when multiple provoke triggers fire from the same
source creature.

**Fix**: Before the tagging loop, collect the set of already-assigned provoke targets for the
current attacker. When selecting a target for subsequent provoke triggers from the same source,
filter out already-targeted creatures. Example approach:
```rust
let mut provoke_targets_used: Vec<ObjectId> = Vec::new();
for t in &mut triggers[pre_len..] {
    // ... existing provoke detection ...
    if ta.description.starts_with("Provoke") {
        t.is_provoke_trigger = true;
        if let Some(dp) = defending_player {
            let target = state.objects.values()
                .filter(|o| {
                    o.zone == ZoneId::Battlefield
                        && o.controller == dp
                        && !provoke_targets_used.contains(&o.id)
                        && /* creature check */
                })
                .map(|o| o.id)
                .next();
            t.provoke_target_creature = target;
            if let Some(tid) = target {
                provoke_targets_used.push(tid);
            }
        }
    }
}
```

#### Finding 2: Missing controller check at resolution

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs:1802-1807`
**CR Rule**: 702.39a -- "target creature **defending player controls**"
**Issue**: The target legality check at resolution time only verifies that the provoked creature
is still on the battlefield (`obj.zone == ZoneId::Battlefield`). It does not check that the
creature is still controlled by the defending player. If the creature changed controllers between
trigger and resolution (e.g., via Threaten/Act of Treason effects), it would still be untapped
and given a forced-block requirement even though it no longer satisfies the targeting restriction.

The practical impact is minimal because:
1. Controller-change during this timing window is extremely rare.
2. The forced-block enforcement in `handle_declare_blockers` checks `o.controller == player`,
   so the forced block would naturally not apply to the wrong player.
3. The only incorrect behavior would be the untap itself.

**Fix**: Add a controller check to the target validity test:
```rust
let target_valid = state.objects.get(&provoked_creature)
    .map(|obj| {
        obj.zone == ZoneId::Battlefield
        // Optionally: && obj.controller == expected_defending_player
    })
    .unwrap_or(false);
```
This requires carrying the defending_player through the ProvokeTrigger variant or the stack
object. Since ProvokeTrigger already has `source_object`, the defending player can be derived
from `CombatState::attackers` at resolution time. Low priority.

#### Finding 3: Test 5 does not verify different targets

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/provoke.rs:390-448`
**CR Rule**: 702.39b -- "each triggers separately"
**Issue**: Test `test_702_39b_provoke_multiple_instances_trigger_separately` only asserts that
two triggers exist on the stack (`trigger_count == 2` and `stack_objects.len() == 2`). It does
not verify that the two ProvokeTrigger stack objects target different creatures. This means the
test would pass even with Finding 1's bug (both targeting the same creature).

**Fix**: After the existing assertions, add:
```rust
// Verify the two triggers target different creatures (CR 702.39b).
let targets: Vec<ObjectId> = state.stack_objects.iter()
    .filter_map(|so| match so.kind {
        StackObjectKind::ProvokeTrigger { provoked_creature, .. } => Some(provoked_creature),
        _ => None,
    })
    .collect();
assert_eq!(targets.len(), 2);
assert_ne!(targets[0], targets[1],
    "CR 702.39b: Two provoke triggers should target different creatures");
```

#### Finding 4: Menace+Provoke interaction

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs:598-800`
**CR Rule**: 509.1c -- "If the number of requirements that are being obeyed is fewer than the
maximum possible number of requirements that could be obeyed without disobeying any restrictions,
the declaration of blockers is illegal."
**Issue**: The menace check (lines 598-628) and provoke forced-block check (lines 630-800) run
sequentially. If an attacker has both Provoke and Menace:
- Provoke requires a creature to block it.
- Menace requires at least two blockers.
- If only one creature can block, declaring it as a sole blocker satisfies provoke but violates
  menace. Declaring no blockers satisfies menace (no single-creature block) but violates provoke.

The correct CR 509.1c behavior: restrictions always take precedence over requirements. If
satisfying the provoke requirement would violate the menace restriction, the requirement is
dropped. The current sequential approach does not handle this holistically.

This is NOT provoke-specific -- it's a general limitation of the blocker validation architecture
that would affect any combination of blocking requirements and restrictions.

**Fix**: No immediate fix needed. Document as a known limitation. A holistic CR 509.1c
implementation (requirement/restriction solver) would be a separate infrastructure task.
The provoke forced-block check already handles evasion restrictions (flying, shadow, etc.)
by skipping impossible requirements, but menace is checked separately and earlier.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.39a (trigger fires on attack) | Yes | Yes | test_702_39a_provoke_basic_untap_and_forced_block |
| 702.39a (untap provoked creature) | Yes | Yes | test_702_39a_provoke_tapped_creature_untapped |
| 702.39a (block if able requirement) | Yes | Yes | test_702_39a_provoke_forces_block_requirement |
| 702.39a (target: defending player's creature) | Yes | Yes | test_702_39a_provoke_multiplayer_correct_defender |
| 702.39b (multiple instances trigger separately) | Yes | Partial | test_702_39b -- verifies count but not different targets (Finding 3) |
| 509.1c (requirement vs restriction) | Yes | Yes | test_702_39a_provoke_creature_cant_block_flying (evasion); menace gap (Finding 4) |
| 603.3d (no valid target = no trigger) | Yes | Yes | test_702_39a_provoke_no_valid_target |
| 508.5/508.5a (defending player in multiplayer) | Yes | Yes | test_702_39a_provoke_multiplayer_correct_defender |
| 608.2b (target fizzle) | Yes | No | Resolution checks zone; no explicit fizzle test |

## Notes

- **Hash coverage**: Complete. KeywordAbility::Provoke (discriminant 79), StackObjectKind::ProvokeTrigger (discriminant 21), CombatState::forced_blocks, PendingTrigger::is_provoke_trigger + provoke_target_creature -- all covered. No discriminant collisions within respective impl blocks.
- **Match arm coverage**: Complete. view_model.rs format_keyword, view_model.rs convert_stack_object_kind, stack_view.rs ProvokeTrigger arm, resolution.rs countering match arm -- all present.
- **No `.unwrap()` in engine library code**: Confirmed. All provoke logic uses `if let`, `map`, `unwrap_or`, `match`. Only test code uses `.unwrap()`.
- **Builder pattern**: Correct. Each Provoke keyword instance generates a separate TriggeredAbilityDef. OrdSet deduplication of keywords is irrelevant since triggered abilities are stored in a Vec.
- **Deterministic targeting**: Acceptable for non-interactive engine. First creature by ObjectId order from OrdMap. Consistent with other deterministic fallbacks (Scry, SacrificePermanents).
- **CombatState lifecycle**: forced_blocks cleared naturally when CombatState is dropped at end of combat. No leak.

# Ability Review: Foretell

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.143
**Files reviewed**:
- `crates/engine/src/state/types.rs` (KeywordAbility::Foretell, discriminant 51)
- `crates/engine/src/cards/card_definition.rs` (AbilityDefinition::Foretell { cost })
- `crates/engine/src/rules/command.rs` (Command::ForetellCard, cast_with_foretell on CastSpell)
- `crates/engine/src/state/game_object.rs` (is_foretold, foretold_turn fields)
- `crates/engine/src/state/stack.rs` (cast_with_foretell on StackObject)
- `crates/engine/src/rules/events.rs` (GameEvent::CardForetold, reveals_hidden_info)
- `crates/engine/src/rules/foretell.rs` (handle_foretell_card)
- `crates/engine/src/rules/casting.rs` (foretell zone detection, cost, mutual exclusion)
- `crates/engine/src/rules/engine.rs` (ForetellCard dispatch, CastSpell destructuring)
- `crates/engine/src/state/hash.rs` (discriminants 51, 17, 78; GameObject + StackObject fields)
- `crates/engine/src/state/mod.rs` (zone-change reset: is_foretold, foretold_turn)
- `crates/engine/src/state/builder.rs` (builder defaults)
- `crates/engine/src/rules/copy.rs` (StackObject copy construction)
- `crates/engine/src/rules/abilities.rs` (StackObject construction sites)
- `crates/engine/src/testing/replay_harness.rs` (foretell_card, cast_spell_foretell, find_foretold_in_exile)
- `crates/engine/src/rules/mod.rs` (pub mod foretell)
- `tools/replay-viewer/src/view_model.rs` (Foretell keyword string)
- `crates/engine/tests/foretell.rs` (18 tests)

## Verdict: needs-fix

The implementation is largely correct and well-structured. The foretell special action, exile
mechanics, alternative cost integration, mutual exclusion checks, zone-change reset, hash
coverage, and replay harness support are all properly implemented. However, there is one MEDIUM
finding: the foretell handler does not validate that the player actually has priority before
allowing the special action, unlike comparable handlers (casting.rs checks priority at line 69).
This violates CR 116.2h which specifies "any time they have priority during their turn" -- the
"has priority" condition is not enforced. Additionally, a test is misnamed (flashback vs escape).

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `foretell.rs:43` | **Missing priority check.** Handler does not validate `priority_holder == player`. **Fix:** Add priority validation. |
| 2 | LOW | `foretell.rs:111` | **Silent failure on get_mut.** `if let Some(...)` silently skips setting foretold flags if object not found. **Fix:** Use `ok_or` with an error. |
| 3 | LOW | `tests/foretell.rs:809` | **Misnamed test.** `test_foretell_mutual_exclusion_with_flashback` actually tests foretell+escape. **Fix:** Rename to `test_foretell_mutual_exclusion_with_escape`. |
| 4 | LOW | `tests/foretell.rs` | **Missing kicker interaction test.** Plan specified test #12 (foretell + kicker) but it was not implemented. **Fix:** Add test. |
| 5 | LOW | `engine.rs:292` | **No trigger checking after foretell.** Cards like Ranar the Ever-Watchful trigger on foretell. **Fix:** Add trigger check when such cards are implemented. |

### Finding Details

#### Finding 1: Missing priority check in handle_foretell_card

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/foretell.rs:43`
**CR Rule**: 116.2h -- "A player may take this action any time they have priority during their turn."
**Issue**: The `handle_foretell_card` function validates that it is the player's own turn
(`state.turn.active_player != player`) but does NOT validate that the player currently has
priority (`state.turn.priority_holder == Some(player)`). CR 116.2h requires BOTH conditions:
(1) it is the player's turn AND (2) the player has priority. The casting handler
(`casting.rs:69`) performs this check explicitly. Without this check, a player could foretell
a card during their own turn even when another player has priority (e.g., during response
windows where the opponent holds priority). The engine dispatch calls `validate_player_active`
(engine.rs:293) but that function only checks if the player has lost/conceded, not whether
they hold priority.
**Fix**: Add a priority check at the top of `handle_foretell_card`, before the active-player
check:
```rust
// CR 116.2h: Foretell requires the player to have priority.
if state.turn.priority_holder != Some(player) {
    return Err(GameStateError::NotPriorityHolder {
        expected: state.turn.priority_holder,
        actual: player,
    });
}
```

#### Finding 2: Silent failure on post-zone-move mutation

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/foretell.rs:111`
**CR Rule**: 702.143a -- card must be exiled face-down with foretold status
**Issue**: The code uses `if let Some(exile_obj) = state.objects.get_mut(&new_exile_id)` to
set `face_down`, `is_foretold`, and `foretold_turn` on the newly created exile object. If
`get_mut` returns `None`, the function silently continues and emits a `CardForetold` event
even though the card was not actually marked as foretold. While this should never happen
in practice (the object was just created by `move_object_to_zone`), the silent failure
pattern contradicts the project's no-`.unwrap()` + explicit-error convention.
**Fix**: Replace the `if let Some` with an explicit error:
```rust
let exile_obj = state.objects.get_mut(&new_exile_id).ok_or_else(|| {
    GameStateError::ObjectNotFound(new_exile_id)
})?;
exile_obj.status.face_down = true;
exile_obj.is_foretold = true;
exile_obj.foretold_turn = current_turn;
```

#### Finding 3: Misnamed test -- flashback vs escape

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/foretell.rs:809`
**CR Rule**: 118.9a -- only one alternative cost
**Issue**: The test `test_foretell_mutual_exclusion_with_flashback` (line 809) does not
actually test flashback mutual exclusion. Instead, it tests `cast_with_escape: true` combined
with `cast_with_foretell: true` (line 891-893). The test name is misleading and makes the
test inventory harder to audit. There is no actual test for foretell+flashback mutual exclusion.
**Fix**: Rename the test to `test_foretell_mutual_exclusion_with_escape`. Optionally, add a
separate `test_foretell_mutual_exclusion_with_flashback` test that actually tests the
flashback combination (though this is hard to construct since flashback auto-detects from
graveyard, and foretell requires exile -- the zones are mutually exclusive by nature).

#### Finding 4: Missing kicker interaction test

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/foretell.rs` (missing)
**CR Rule**: 118.9d -- "If an alternative cost is being paid to cast a spell, any additional
costs, cost increases, and cost reductions that affect that spell are applied to that
alternative cost."
**Issue**: The plan specified test #12 (`test_foretell_with_kicker`) to verify that kicker
(an additional cost) can be paid alongside foretell (an alternative cost) per CR 118.9d.
This test was not implemented. The engine likely handles this correctly since kicker is an
additional cost applied on top of the alternative cost, but the test gap means this
interaction is unverified.
**Fix**: Add a test that foretells a card with kicker, then casts it from exile with
`cast_with_foretell: true` and `kicker_times: 1`, verifying the spell resolves as kicked.

#### Finding 5: No trigger checking after ForetellCard dispatch

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs:292`
**CR Rule**: 603.2 -- "Whenever a game event or game state matches a triggered ability's
trigger condition..."
**Issue**: The `ForetellCard` dispatch (engine.rs:292-298) does not call
`abilities::check_triggers()` or `abilities::flush_pending_triggers()` after processing.
Other commands (CastSpell, CycleCard, CrewVehicle, etc.) all include trigger checking.
While no currently implemented cards trigger on the foretell action, cards like Ranar the
Ever-Watchful ("Whenever you foretell a card or cast a foretold spell...") require this.
The `CardForetold` event provides the hook, but the trigger system is not wired up to
check for it after the foretell action.
**Fix**: When trigger cards that reference foretelling are added, update the ForetellCard
dispatch to mirror the pattern used by CycleCard/CrewVehicle:
```rust
let new_triggers = abilities::check_triggers(&state, &events);
for t in new_triggers {
    state.pending_triggers.push_back(t);
}
let trigger_events = abilities::flush_pending_triggers(&mut state);
events.extend(trigger_events);
```
No immediate fix needed -- this is future work. The `CardForetold` event is already emitted
to support this.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.143a (foretell from hand, pay {2}, exile face-down) | Yes | Yes | test_foretell_basic_exile_face_down |
| 702.143a (cast after current turn ends) | Yes | Yes | test_foretell_cannot_cast_same_turn, test_foretell_cast_from_exile_on_later_turn |
| 702.143a (foretell cost as alternative cost) | Yes | Yes | test_foretell_cast_from_exile_on_later_turn |
| 702.143b (special action, no stack) | Yes | Yes | test_foretell_does_not_use_stack |
| 702.143c (definition of "foretold") | Yes | Partial | is_foretold flag tested; "cast for other cost" clause not tested (no test casts a foretold card for its normal cost) |
| 702.143d (effects making cards foretold) | No (out of scope) | No | Plan explicitly deferred |
| 702.143e (differentiation of foretold cards) | N/A | N/A | Physical game concern; digital engine tracks by ObjectId |
| 702.143f (reveal on player leave) | No | No | Deferred; would require player-elimination + reveal logic |
| 116.2h (any time with priority during own turn) | Partial (Finding 1) | Yes | test_foretell_during_combat, test_foretell_requires_player_turn; priority check missing |
| 118.9a (mutual exclusion) | Yes | Yes | test_foretell_mutual_exclusion_with_escape (misnamed), test_foretell_mutual_exclusion_with_evoke |
| 118.9c (mana value unchanged) | Yes | Yes | test_foretell_mana_value_unchanged |
| 118.9d (additional costs apply) | Yes (by engine design) | No | Missing kicker test (Finding 4) |
| 400.7 (new ObjectId on zone change) | Yes | Yes | test_foretell_card_identity_tracking |
| Ruling: sorcery timing preserved | Yes | Yes | test_foretell_sorcery_timing_restriction |
| Ruling: instant timing preserved | Yes | Yes | test_foretell_instant_timing |
| Hidden info (face-down exile) | Yes | Yes | test_foretell_reveals_hidden_info |

## Additional Notes

### Correctly Handled

- **Zone-change reset**: Both `move_object_to_zone` call sites in `state/mod.rs` (lines 286-287, 365-366) properly reset `is_foretold: false` and `foretold_turn: 0`. Builder defaults are also correct.
- **Hash coverage**: All new fields and variants have hash support: `KeywordAbility::Foretell` (discriminant 51), `AbilityDefinition::Foretell` (discriminant 17), `GameEvent::CardForetold` (discriminant 78), `GameObject.is_foretold` + `foretold_turn`, `StackObject.cast_with_foretell`.
- **All StackObject construction sites**: Verified 8 construction sites across casting.rs (3: main, storm, cascade), copy.rs (2), and abilities.rs (4). All set `cast_with_foretell: false` except the main cast path which uses `casting_with_foretell`.
- **Mutual exclusion completeness**: The foretell block (Step 1f in casting.rs, lines 442-481) checks against all 6 other alternative costs (flashback, evoke, bestow, madness, miracle, escape). Since foretell's block is evaluated last, it catches all combinations regardless of whether earlier blocks check foretell.
- **Split second compatibility**: Foretell is a special action (CR 702.143b). CR 702.61b explicitly allows special actions during split second. The implementation correctly has no split second check in the foretell handler.
- **Replay harness**: Both `foretell_card` and `cast_spell_foretell` action types are implemented. The `find_foretold_in_exile` helper correctly filters by name + zone + `is_foretold` + owner.
- **View model**: `KeywordAbility::Foretell` maps to `"Foretell"` string in the replay viewer.
- **Foretell does NOT change resolution behavior**: Unlike flashback (which exiles on resolution), foretell resolves normally -- permanents to battlefield, instants/sorceries to graveyard. No changes needed in resolution.rs.

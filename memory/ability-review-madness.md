# Ability Review: Madness

**Date**: 2026-02-27
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.35
**Files reviewed**:
- `crates/engine/src/state/types.rs` (line 361-372)
- `crates/engine/src/cards/card_definition.rs` (line 184-192)
- `crates/engine/src/state/stack.rs` (line 76-84, 162-176)
- `crates/engine/src/state/hash.rs` (lines 386-387, 1123-1135, 1179-1180, 2447-2451, 936-958)
- `crates/engine/src/state/stubs.rs` (line 91-109)
- `crates/engine/src/effects/mod.rs` (line 1907-1987)
- `crates/engine/src/rules/abilities.rs` (line 381-549, 1168-1198)
- `crates/engine/src/rules/turn_actions.rs` (line 169-298)
- `crates/engine/src/rules/casting.rs` (line 80-128, 234-252, 309-325, 509-526, 775-795)
- `crates/engine/src/rules/resolution.rs` (line 603-646, 742)
- `crates/engine/src/rules/copy.rs` (line 174-182, 348-352)
- `crates/engine/src/rules/command.rs` (line 58-110)
- `tools/replay-viewer/src/view_model.rs` (line 434)
- `crates/engine/tests/madness.rs` (all 1033 lines)

## Verdict: needs-fix

One MEDIUM finding: the alternative cost mutual exclusion check is missing for madness
combined with evoke/bestow, violating CR 601.2b. A malicious or buggy client could submit
a CastSpell command with `cast_with_evoke: true` while the card is in exile with madness,
causing the evoke cost to be used instead of the madness cost. All other aspects of the
implementation are correct and well-tested.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `casting.rs:186-232` | **Missing alternative cost exclusion for madness.** CR 601.2b violated: madness + evoke/bestow not rejected. **Fix:** Add validation checks. |
| 2 | LOW | `casting.rs:246-249` | **Madness cost fallback to `None` allows free cast.** Defensive fallback when card has keyword but no AbilityDefinition. **Fix:** Return error instead of allowing free cast. |
| 3 | LOW | `resolution.rs:603-646` | **Auto-decline only; no player choice during trigger resolution.** CR 702.35a says "its owner may cast it." MVP limitation is documented. **Fix:** None required for MVP; future work. |

### Finding Details

#### Finding 1: Missing alternative cost exclusion for madness

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs:186-232`
**CR Rule**: 601.2b -- "A player can't apply two alternative methods of casting or two alternative costs to a single spell."
**Issue**: The code validates mutual exclusion between:
- evoke + flashback (line 188-191)
- bestow + flashback (line 205-209)
- bestow + evoke (line 211-214)

But it does NOT validate exclusion between:
- madness + evoke
- madness + bestow
- madness + flashback (flashback is from graveyard, madness from exile, so in practice they can't overlap for the same cast -- but the explicit check is still good practice for defense-in-depth)

Since `casting_with_madness` is auto-detected (not a command parameter), a player could submit `CastSpell { cast_with_evoke: true }` for a madness card in exile. The evoke cost would take priority in the cost selection chain (line 235-252), overriding the correct madness cost. This violates CR 601.2b and could allow the player to pay a different cost than intended.

**Fix**: After the bestow validation block (after line 232), add a check:

```rust
// Step 1c: Validate madness exclusion (CR 601.2b / CR 118.9a).
if casting_with_madness {
    if casting_with_evoke {
        return Err(GameStateError::InvalidCommand(
            "cannot combine madness with evoke (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    if casting_with_bestow {
        return Err(GameStateError::InvalidCommand(
            "cannot combine madness with bestow (CR 118.9a: only one alternative cost)".into(),
        ));
    }
    // Note: madness (exile) and flashback (graveyard) are mutually exclusive by zone,
    // but we validate explicitly for defense-in-depth.
    if casting_with_flashback {
        return Err(GameStateError::InvalidCommand(
            "cannot combine madness with flashback (CR 118.9a: only one alternative cost)".into(),
        ));
    }
}
```

#### Finding 2: Madness cost fallback to None allows free cast

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs:246-249`
**CR Rule**: 702.35b -- "Casting a spell using its madness ability follows the rules for paying alternative costs in rules 601.2b and 601.2f-h."
**Issue**: When `casting_with_madness` is true, `get_madness_cost()` returns `None` if the card registry has no `AbilityDefinition::Madness { cost }` for the card. A `None` cost causes the `if let Some(ref cost) = mana_cost` block (line 460) to be skipped entirely, resulting in a free cast. The doc comment at line 778-779 says "free madness -- rare but correct per CR 118.9" but this is not correct -- it indicates a malformed card definition (keyword present but no AbilityDefinition), not a valid free-cast scenario.
**Fix**: When `casting_with_madness && get_madness_cost() == None`, return an error:

```rust
} else if casting_with_madness {
    let cost = get_madness_cost(&card_id, &state.card_registry);
    if cost.is_none() {
        return Err(GameStateError::InvalidCommand(
            "card has Madness keyword but no madness cost defined".into(),
        ));
    }
    cost
}
```

#### Finding 3: Auto-decline only; no player choice during trigger resolution

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs:603-646`
**CR Rule**: 702.35a -- "its owner may cast it by paying [cost] rather than paying its mana cost. If that player doesn't, they put this card into their graveyard."
**Issue**: The madness trigger resolution always auto-declines (moves card to graveyard). The player CAN cast the spell by issuing a CastSpell command before the trigger resolves (while it's on the stack), which is tested and works. However, the CR says the choice happens "when the madness triggered ability resolves," not before. The current approach is functionally equivalent for testing purposes (the player either casts before resolution or the trigger resolves with auto-decline), but it reverses the decision timing: the real rule gives the player a choice AT resolution, not a window to pre-empt the trigger.

This is documented as an MVP limitation in the plan. The workaround (cast from exile before trigger resolves) covers the main gameplay case. A `ChooseMadness` command or similar would be needed for full CR compliance.

**Fix**: None required for P3 priority. Document in the ability coverage audit that madness player choice is deferred to a future interactive-choice system.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.35a (static: exile on discard) | Yes | Yes | test 1 (cleanup), test 9 (cycling), test 11 (effect), test 2 (negative) |
| 702.35a (triggered: may cast or graveyard) | Yes (auto-decline) | Yes | test 6 (decline to graveyard), test 3 (trigger on stack) |
| 702.35a (cast from exile for madness cost) | Yes | Yes | test 4 (cast from exile), test 10 (mana value), test 7 (flag) |
| 702.35b (alternative cost rules) | Yes | Partial | test 4 (cost paid), test 10 (mana value unchanged). Missing: test for mutual exclusion with evoke/bestow (Finding 1) |
| 702.35c (public zone tracking) | No | No | 400.7k linkage not implemented; this is a niche edge case for effects referencing "the discarded card" after the madness trigger resolves and the card goes to a public zone |
| Timing override (sorcery at instant speed) | Yes | Yes | test 5 (sorcery cast during opponent's turn) |
| Discard still counts as discarded | Yes | Yes | test 1, test 9, test 11 (CardDiscarded / DiscardedToHandSize events) |
| All 3 discard sites covered | Yes | Yes | effects/mod.rs (test 11), abilities.rs cycling (test 9), turn_actions.rs cleanup (test 1) |
| cast_with_madness flag on StackObject | Yes | Yes | test 7 |
| Negative: non-madness in exile can't cast | Yes | Yes | test 8 |
| Hash coverage: KeywordAbility::Madness | Yes | N/A | hash.rs line 387, discriminant 48 |
| Hash coverage: AbilityDefinition::Madness | Yes | N/A | hash.rs line 2448, discriminant 13 |
| Hash coverage: StackObjectKind::MadnessTrigger | Yes | N/A | hash.rs line 1123, discriminant 6 |
| Hash coverage: StackObject.cast_with_madness | Yes | N/A | hash.rs line 1180 |
| Hash coverage: PendingTrigger madness fields | Yes | N/A | hash.rs lines 956-958 |
| All StackObject construction sites | Yes | N/A | 10/10 sites have cast_with_madness field |
| Copy logic (cast_with_madness: false) | Yes | N/A | copy.rs line 182 -- copies are never cast |
| View model serialization | Yes | N/A | view_model.rs line 434 |
| Cleanup step re-grant priority (CR 514.3a) | Yes | Yes | engine.rs lines 370-404 handle pending triggers during cleanup |

## Test Quality Assessment

The test file is well-structured with 11 tests covering all major paths:

| Test | CR Cited | Positive/Negative | Edge Case |
|------|----------|-------------------|-----------|
| test 1: discard goes to exile | 702.35a | Positive | Cleanup discard path |
| test 2: non-madness to graveyard | 702.35a | Negative | - |
| test 3: trigger on stack | 702.35a | Positive | - |
| test 4: cast from exile | 702.35a/b | Positive | Alternative cost |
| test 5: sorcery timing override | 702.35 ruling | Positive | Opponent's turn |
| test 6: decline to graveyard | 702.35a | Positive | Auto-decline flow |
| test 7: cast_with_madness flag | 702.35 | Positive | Stack flag |
| test 8: non-madness exile can't cast | 702.35a | Negative | - |
| test 9: cycling + madness | 702.35a + cycling | Positive | Multi-ability interaction |
| test 10: mana value unchanged | 118.9c | Positive | CR 118.9c verification |
| test 11: effect-based discard | 702.35a | Positive | Effect discard path |

**Missing test**: No test for the alternative cost mutual exclusion (madness + evoke should be rejected). This should be added when Finding 1 is fixed.

## Code Quality Notes

- CR citations present in all doc comments: Yes
- No `.unwrap()` in engine library code: Correct (all `.unwrap()` are in test code only)
- Hash coverage complete: Yes (all 5 new hash sites verified)
- All match arms for new StackObjectKind::MadnessTrigger covered: Yes (resolution.rs, hash.rs, view_model.rs counter arm)
- `#[serde(default)]` on all new fields: Yes (cast_with_madness, is_madness_trigger, madness_exiled_card, madness_cost)
- Consistent with conventions.md: Yes (test naming, CR citations, error handling)
- No over-engineering: Yes (auto-decline is appropriate for MVP)

# Ability Review: Buyback

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.27
**Files reviewed**:
- `crates/engine/src/rules/casting.rs` (lines 52-68, 560-627, 870-895, 975-1046, 1113-1132)
- `crates/engine/src/rules/resolution.rs` (lines 70-97, 460-501)
- `crates/engine/src/rules/command.rs` (lines 130-157)
- `crates/engine/src/rules/engine.rs` (lines 72-117)
- `crates/engine/src/rules/copy.rs` (lines 186-190, 344-364)
- `crates/engine/src/rules/abilities.rs` (all `was_buyback_paid: false` sites)
- `crates/engine/src/effects/mod.rs` (CounterSpell path, lines 688-734)
- `crates/engine/src/state/stack.rs` (lines 95-119)
- `crates/engine/src/state/types.rs` (lines 497-503)
- `crates/engine/src/state/hash.rs` (lines 430-431, 1301-1305, 2634-2638)
- `crates/engine/src/cards/card_definition.rs` (lines 238-245)
- `crates/engine/src/testing/script_schema.rs` (lines 243-247)
- `crates/engine/src/testing/replay_harness.rs` (lines 194-256, 263-615)
- `crates/engine/tests/buyback.rs` (all 980 lines, 8 tests)
- `crates/engine/tests/script_replay.rs` (lines 143-168)
- `tools/replay-viewer/src/view_model.rs` (line 652)

## Verdict: needs-fix

The implementation is largely correct and well-structured. The CR rule text is faithfully
implemented: buyback is an additional cost added in the casting pipeline (CR 601.2f),
and the resolution path correctly routes to `ZoneId::Hand(owner)` when `was_buyback_paid`
is true (CR 702.27a). The flashback-overrides-buyback priority is correct
(CR 702.34a checked first). Countered spells correctly bypass buyback
(CounterSpell effect does not check `was_buyback_paid`). Hash coverage, view model display,
and all StackObject construction sites are properly updated.

However, there is one HIGH finding (`.unwrap()` in engine library code at `casting.rs:615`)
and one MEDIUM finding (missing fizzle test from the plan). Both require fixes before
the ability can be closed.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `casting.rs:615` | **`.unwrap()` in engine library code.** Violates architecture invariant. **Fix:** bind cost from validation match arm. |
| 2 | MEDIUM | `tests/buyback.rs` | **Missing fizzle test.** Plan Test 4 (CR 608.2b + 702.27a) not implemented. **Fix:** add test. |
| 3 | LOW | `casting.rs:600-615` | **Double `get_buyback_cost` lookup.** Minor redundancy. **Fix:** bind result from first call. |

### Finding Details

#### Finding 1: `.unwrap()` in engine library code

**Severity**: HIGH
**File**: `crates/engine/src/rules/casting.rs:615`
**CR Rule**: n/a (architecture invariant from `memory/conventions.md`)
**Issue**: The buyback cost addition block calls `get_buyback_cost(&card_id, &state.card_registry).unwrap()` on line 615. While this is logically safe (the validation block at lines 600-611 already confirmed `Some` and returned `Err` on `None`), the project conventions explicitly prohibit `.unwrap()` in engine library code: "Never `unwrap()` or `expect()` in engine logic. Tests may use `unwrap()`." (from `memory/conventions.md`). This is non-negotiable per architecture invariants.

**Fix**: Bind the buyback cost from the validation match arm at line 602. Change:

```rust
let was_buyback_paid = if cast_with_buyback {
    match get_buyback_cost(&card_id, &state.card_registry) {
        Some(_) => true,
        None => { return Err(...); }
    }
} else { false };

let mana_cost = if was_buyback_paid {
    let buyback_cost = get_buyback_cost(&card_id, &state.card_registry).unwrap();
    ...
```

To:

```rust
let buyback_cost_opt = if cast_with_buyback {
    match get_buyback_cost(&card_id, &state.card_registry) {
        Some(cost) => Some(cost),
        None => {
            return Err(GameStateError::InvalidCommand(
                "spell does not have buyback".into(),
            ));
        }
    }
} else {
    None
};
let was_buyback_paid = buyback_cost_opt.is_some();

let mana_cost = if let Some(buyback_cost) = buyback_cost_opt {
    let mut total = mana_cost.unwrap_or_default();
    total.white += buyback_cost.white;
    // ... (rest unchanged)
    Some(total)
} else {
    mana_cost
};
```

This eliminates both the `.unwrap()` and the double lookup (Finding 3).

#### Finding 2: Missing fizzle test

**Severity**: MEDIUM
**File**: `crates/engine/tests/buyback.rs`
**CR Rule**: 608.2b + 702.27a -- "If the buyback cost was paid, put this spell into its owner's hand instead of into that player's graveyard **as it resolves**." A fizzled spell does not resolve. CR 608.2b: when all targets are illegal, the spell is removed from the stack without resolving.
**Issue**: The implementation plan explicitly listed Test 4 (`test_buyback_paid_spell_fizzles_goes_to_graveyard`) as a key edge case. The fizzle path at `resolution.rs:89-93` is correctly implemented (does NOT check `was_buyback_paid` -- fizzled spells go to graveyard/exile regardless of buyback), but there is no test to prevent regression. The plan's Test 4 was replaced with `test_buyback_spell_cast_event_emitted` (a less critical positive test).

**Fix**: Add a test `test_buyback_paid_spell_fizzles_goes_to_graveyard` to `crates/engine/tests/buyback.rs`. Setup: P1 casts Searing Touch with `cast_with_buyback: true` targeting a creature. Before resolution, remove the creature (e.g., destroy it with another spell, or use a creature that dies to SBAs). All targets become illegal. Assert: Searing Touch is in P1's graveyard (not hand). This locks in the correct behavior at `resolution.rs:89-93`.

#### Finding 3: Double `get_buyback_cost` lookup

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:600-615`
**CR Rule**: n/a (code quality)
**Issue**: `get_buyback_cost()` is called twice -- once at line 601 (validation, result discarded) and again at line 615 (to get the actual cost). This is a minor performance issue (two registry lookups) and a style issue. The plan itself noted this: "The `get_buyback_cost` is called twice -- the runner may optimize to a single call."

**Fix**: Resolved by Finding 1's fix (bind the cost from the first call).

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.27a (additional cost) | Yes | Yes | test_buyback_cost_added_to_total, test_buyback_insufficient_mana_rejected |
| 702.27a (return to hand on resolve) | Yes | Yes | test_buyback_basic_return_to_hand |
| 702.27a (not paid = graveyard) | Yes | Yes | test_buyback_not_paid_goes_to_graveyard |
| 702.27a + 701.6a (countered = graveyard) | Yes | Yes | test_buyback_paid_spell_countered_goes_to_graveyard |
| 702.27a + 608.2b (fizzle = graveyard) | Yes | **No** | Fizzle path correct at resolution.rs:89-93 but no test (Finding 2) |
| 702.27a + 702.34a (flashback overrides) | Yes | Yes | test_buyback_with_flashback_exile_wins |
| 601.2b (announce intention) | Yes | Yes | cast_with_buyback field on Command::CastSpell |
| 601.2f (total cost calculation) | Yes | Yes | Buyback cost added after kicker, before reductions |
| 118.8 (additional cost rules) | Yes | Yes | Cost pipeline order correct |
| 118.8d (doesn't change mana cost) | Yes | n/a | Buyback adds to total paid, not to characteristics.mana_cost |
| No buyback ability = error | Yes | Yes | test_buyback_no_buyback_ability_rejected |
| SpellCast event emitted | Yes | Yes | test_buyback_spell_cast_event_emitted |
| Copies have was_buyback_paid: false | Yes | n/a | copy.rs:189, copy.rs:363 |
| Hash coverage | Yes | n/a | hash.rs:1304 (StackObject), hash.rs:431 (KeywordAbility), hash.rs:2636 (AbilityDefinition) |

## Detailed Verification Notes

### Casting Pipeline Order
The buyback cost is added at lines 614-627, which is AFTER kicker (lines 580-595) and BEFORE affinity/undaunted/convoke/improvise/delve reductions. This matches CR 601.2f: "total cost = mana cost + additional costs + increases - reductions." Correct.

### CounterSpell Path (effects/mod.rs:720-724)
Correctly does NOT check `was_buyback_paid`. Countered spells go to graveyard (or exile if flashback). This matches CR 701.6a. Verified -- no change needed.

### Fizzle Path (resolution.rs:89-93)
Correctly does NOT check `was_buyback_paid`. Fizzled spells go to graveyard (or exile if flashback). This matches CR 702.27a ("as it resolves" -- fizzled spells do not resolve). Verified -- no change needed, but test coverage is missing (Finding 2).

### Resolution Path (resolution.rs:487-493)
Flashback is checked first (line 487), buyback second (line 489), graveyard last (line 492). Priority order is correct per CR 702.34a ("exile instead of putting it anywhere else").

### StackObject Construction Sites
All construction sites across `abilities.rs` (7 sites), `copy.rs` (2 sites), `casting.rs` (2 sites for storm/cascade triggers) correctly set `was_buyback_paid: false`. Verified.

### Command Wiring
`cast_with_buyback` is correctly destructured in `engine.rs:86` and passed to `casting::handle_cast_spell` at `engine.rs:106`. The `Command::CastSpell` variant has the field with proper `#[serde(default)]` annotation and CR citation at `command.rs:150-156`.

### Replay Harness
The `buyback` field is correctly wired through `script_schema.rs:243-247`, `script_replay.rs:148+165`, and `replay_harness.rs:207+254`. All other `cast_spell_*` arms (flashback, evoke, bestow, madness, miracle, escape, foretell) correctly set `cast_with_buyback: false`.

### Test Quality
All 8 tests cite CR rules in comments. Test naming follows conventions. Assertions check both positive and negative conditions (e.g., card IS in hand AND is NOT in graveyard). The `pass_all` helper is reused correctly. Card definitions are well-crafted with proper AbilityDefinition structures.

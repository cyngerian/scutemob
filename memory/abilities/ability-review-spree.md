# Ability Review: Spree

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.172
**Files reviewed**:
- `crates/engine/src/state/types.rs:1232-1240` (KeywordAbility::Spree)
- `crates/engine/src/state/hash.rs:657-658` (KW hash), `:3259-3276` (ModeSelection hash)
- `crates/engine/src/cards/card_definition.rs:1200-1219` (ModeSelection.mode_costs)
- `crates/engine/src/rules/casting.rs:2013-2069` (Spree cost block)
- `crates/engine/src/rules/casting.rs:2929-2995` (modes_chosen validation)
- `crates/engine/src/rules/resolution.rs:271-309` (mode dispatch)
- `crates/engine/src/rules/copy.rs:233-235` (modes_chosen propagation)
- `tools/replay-viewer/src/view_model.rs:854` (display arm)
- `crates/engine/tests/spree.rs` (9 tests, 748 lines)
- 7 existing ModeSelection sites (abzan_charm, blessed_alliance, promise_of_power, escalate, entwine, modal x4)

## Verdict: needs-fix

One MEDIUM finding: test 7 (`test_spree_mode_order_ascending`) claims to verify CR 700.2a
mode execution order but uses commutative effects, making the assertion vacuous. The
underlying resolution code does NOT sort `modes_chosen` into ascending order, so modes
execute in whatever order the caller supplies. This is a pre-existing modal spell issue
(not Spree-specific), but Spree's plan explicitly requires it and the test gives false
confidence. All other aspects are correct and well-implemented.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `tests/spree.rs:596-652` | **Mode order test is vacuous.** Uses commutative effects; does not verify ascending execution order. |
| 2 | LOW | `tests/spree.rs` (missing) | **Missing CR 118.8d mana value test.** Plan called for it; architecture makes it inherently correct. |
| 3 | LOW | `rules/resolution.rs:288-292` | **Pre-existing: modes_chosen not sorted.** Resolution iterates in stored order, not ascending index order. |

### Finding Details

#### Finding 1: Mode order test is vacuous

**Severity**: MEDIUM
**File**: `crates/engine/tests/spree.rs:596-652`
**CR Rule**: 700.2a -- "always follow the instructions in the order they are written"
**Issue**: `test_spree_mode_order_ascending` chooses modes `[2, 0]` and asserts the net life
total is `initial_life + 4 - 3`. Mode 0 is GainLife(4) and mode 2 is LoseLife(3). These
effects are commutative -- the final result is identical regardless of execution order.
The test comment at line 646 even acknowledges this: "If executed in wrong order, same math."
This means the test does not actually verify that modes execute in ascending printed order
per CR 700.2a. It would pass even if modes executed in reverse order `[2, 0]`.
**Fix**: Replace mode 2's effect with something order-dependent (e.g., mode 0 sets life to a
value, mode 2 reads life total) OR use event inspection to verify the order of GameEvents
emitted during resolution. Alternatively, accept this as a known gap and add a comment
noting the test only verifies both modes execute, not their order. Since the underlying
resolution code (Finding 3) also doesn't sort, fixing the test alone would just reveal
the deeper issue -- consider sorting `modes_chosen` in `validated_modes_chosen` (casting.rs
line 2988) and then the test would pass meaningfully.

#### Finding 2: Missing CR 118.8d mana value test

**Severity**: LOW
**File**: `crates/engine/tests/spree.rs` (absent test)
**CR Rule**: 118.8d -- "Additional costs don't change a spell's mana value"
**Issue**: The plan specified test 9 as `test_spree_mana_value_unchanged` to verify that
per-mode costs don't inflate the spell's mana value. Instead, test 9 was implemented as
`test_spree_non_spree_spell_unchanged`. The mana value test was dropped. This is LOW
because the architecture inherently prevents the bug -- `ManaCost::mana_value()` operates
on the card's static mana cost, not the dynamically computed total payment.
**Fix**: Optionally add a test that casts a Spree spell with 2 modes, then inspects the
StackObject's source card's mana value and asserts it equals the base cost's mana value
(2, for `{1}{W}`). Low priority.

#### Finding 3: Pre-existing -- modes_chosen not sorted at resolution

**Severity**: LOW (pre-existing, not Spree-specific)
**File**: `crates/engine/src/rules/resolution.rs:288-292`
**CR Rule**: 700.2a -- modes execute in printed order
**Issue**: `resolution.rs:288-292` iterates `stack_obj.modes_chosen.iter()` in stored order.
The validation block at `casting.rs:2988` returns `modes_chosen` unsorted. If a caller
passes `modes_chosen: vec![2, 0]`, mode 2 executes before mode 0, violating CR 700.2a.
This affects all modal spells, not just Spree. Currently harmless because most callers
(tests, bots, harness) provide modes in ascending order, but a network client could
trigger incorrect behavior.
**Fix**: Add `.sort()` or `.sorted()` to `validated_modes_chosen` at casting.rs line 2988
before returning, OR sort in resolution.rs before iterating. This is a pre-existing issue
and should be tracked separately from the Spree implementation.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.172a (choose one or more, pay mode costs) | Yes | Yes | tests 1-5 cover positive + negative |
| 702.172b (visual reminder, no rules meaning) | N/A | N/A | Cosmetic only |
| 700.2a (modes in printed order) | Partial | Vacuous | See Finding 1+3 |
| 700.2d (no duplicate modes) | Yes | Yes | test 6 |
| 700.2g (copies copy modes) | Yes (pre-existing) | No | copy.rs:235 propagates; no Spree-specific copy test |
| 700.2h (per-mode additional costs) | Yes | Yes | tests 1-3 |
| 601.2f (additional cost rules) | Yes | Yes | test 5 |
| 118.8d (mana value unchanged) | Yes (by architecture) | No | See Finding 2 |

## Additional Observations (no action needed)

1. **Hash coverage is complete.** `KeywordAbility::Spree` hashed at hash.rs:658.
   `ModeSelection.mode_costs` hashed at hash.rs:3267-3274 with proper Some/None discrimination.

2. **All 7 existing ModeSelection sites updated.** Every struct literal has `mode_costs: None`:
   abzan_charm.rs:42, blessed_alliance.rs:43, promise_of_power.rs:25, escalate.rs:93,
   entwine.rs:92, modal.rs:84/137/880/1301.

3. **Serde compatibility preserved.** `#[serde(default)]` on `mode_costs` means existing
   JSON without this field deserializes correctly as `None`.

4. **Alt-cost interaction is correct.** The Spree cost block (casting.rs:2040) starts from
   `mana_cost.unwrap_or_default()` which is the post-alt-cost value. Alt costs zero the
   base, then Spree adds per-mode costs on top. This matches the ruling that "without
   paying its mana cost" still requires mode costs.

5. **Entwine + Spree edge case handled.** casting.rs:2043-2044 correctly charges all mode
   costs when `entwine_paid` is true, and line 2021 skips the empty-modes check when
   `entwine_paid`.

6. **Copy propagation correct.** copy.rs:235 clones `modes_chosen`, satisfying CR 700.2g.

7. **No `.unwrap()` in engine library code.** The Spree block in casting.rs uses
   `and_then`, `match`, and `if let` throughout.

8. **Non-Spree spell path is clean.** The `else` branch at casting.rs:2067-2068 passes
   `mana_cost` through unchanged when `KeywordAbility::Spree` is absent.

9. **view_model.rs updated.** Spree arm at line 854.

10. **Discriminant 134 is correct and sequential** after Fuse (133).

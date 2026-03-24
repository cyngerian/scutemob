# Primitive Batch Review: PB-24 -- Conditional Statics ("as long as X")

**Date**: 2026-03-23
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 604.1, 604.2, 604.3a(5), 613.1, 613.1d, 700.5, 700.5a
**Engine files reviewed**: `state/continuous_effect.rs`, `rules/layers.rs`, `effects/mod.rs`, `rules/replacement.rs`, `state/hash.rs`, `cards/card_definition.rs`, `cards/helpers.rs`, `state/types.rs`
**Card defs reviewed**: 13 (serra_ascendant, dragonlord_ojutai, bloodghast, purphoros_god_of_the_forge, athreos_god_of_passage, iroas_god_of_victory, nadaar_selfless_paladin, beastmaster_ascension, quest_for_the_goblin_lord, arixmethes_slumbering_isle, razorkin_needlehead, mox_opal, indomitable_archangel)
**Test file reviewed**: `crates/engine/tests/conditional_statics.rs` (11 tests)

## Verdict: needs-fix

One HIGH finding (wrong mana cost on Nadaar), two MEDIUM findings (devotion not using layer-resolved
mana costs per CR 700.5a, recursive calculate_characteristics in YouControlNOrMoreWithFilter).
Several LOW findings for pre-existing card def issues and edge cases.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `effects/mod.rs:5416` | **Devotion uses base mana costs instead of Layer 3-resolved.** CR 700.5a says devotion is calculated "after considering any copy, control, or text-changing effects." Current impl reads `obj.characteristics.mana_cost` (base). **Fix:** Document as known limitation; full fix requires partial layer resolution (Layer 1-3 only) per object, which is complex. Add a comment citing CR 700.5a and noting the deviation. |
| 2 | **MEDIUM** | `effects/mod.rs:5356` | **Recursive calculate_characteristics in YouControlNOrMoreWithFilter.** `check_static_condition` calls `calculate_characteristics` for each battlefield object to check type filters. Since `is_effect_active` is called within `calculate_characteristics`, this creates recursive re-entrant calls. While safe for immutable state, it causes O(n^2) work and could theoretically infinite-loop if a conditional static modifies types that another conditional static checks. **Fix:** Add a brief comment documenting the recursion is safe because `im-rs` state is immutable and conditions do not modify state. Consider using base characteristics for the filter check (like devotion does) to avoid recursion, or accept the current approach with documentation. |
| 3 | LOW | `effects/mod.rs:5320` | **check_static_condition missing CR citation in doc comment.** The function has good inline CR citations but the doc comment doesn't cite CR 604.2 formally. Minor style issue. **Fix:** Add `/// CR 604.2` to the doc comment. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 4 | **HIGH** | `nadaar_selfless_paladin.rs:16` | **Wrong mana cost.** Oracle: `{2}{W}` (CMC 3). Def has `generic: 3, white: 1` (CMC 4). **Fix:** Change to `generic: 2, white: 1`. Also fix the header comment from `{3}{W}` to `{2}{W}`. |
| 5 | LOW | `purphoros_god_of_the_forge.rs:50` | **Pre-existing: "another creature" trigger lacks self-exclusion.** Oracle says "Whenever another creature you control enters" but the trigger `WheneverCreatureEntersBattlefield` with `TargetController::You` filter does not exclude self. Purphoros triggers on its own ETB. Not introduced by PB-24. **Fix:** Deferred -- requires `exclude_self` on `WheneverCreatureEntersBattlefield` (may be PB-25+ scope). |
| 6 | LOW | `beastmaster_ascension.rs:22` | **Pre-existing: "you may" not modeled.** Oracle says "you may put a quest counter" (optional). The trigger always adds the counter (mandatory). Not introduced by PB-24. **Fix:** Deferred -- needs optional effect wrapper. |
| 7 | LOW | `arixmethes_slumbering_isle.rs` | **Pre-existing: missing ETB counters.** Oracle says "enters tapped with five slumber counters." Only the enters-tapped part is implemented; the 5 slumber counters are a documented TODO. Without counters, the conditional statics have no effect at game start. Not introduced by PB-24. **Fix:** Already documented as DSL gap (ETB-with-counters replacement). |
| 8 | LOW | `arixmethes_slumbering_isle.rs:39-67` | **"It's a land" ruling says earlier type additions are overwritten.** Ruling (2020-08-07): "Arixmethes's effect causing it to be a land overwrites any earlier effects that gave it additional types." AddCardTypes(Land) + RemoveCardTypes(Creature) does not overwrite other types added by earlier effects (e.g., Phyrexian Metamorph adding Artifact). **Fix:** Deferred -- would need SetTypeLine or a more nuanced type override. Extremely niche interaction. |

### Finding Details

#### Finding 1: Devotion uses base mana costs instead of Layer 3-resolved

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:5416`
**CR Rule**: 700.5a -- "A player's devotion to each color and combination of colors, taking into account any effects that modify devotion, is calculated after considering any copy, control, or text-changing effects but before any other effects that modify the characteristics of permanents."
**Issue**: `calculate_devotion_to_colors` reads `obj.characteristics.mana_cost` which is the base (printed) mana cost. If a Layer 1 copy effect or Layer 3 text-changing effect modifies the mana cost, devotion would be calculated incorrectly. For example, if a Clone copies a creature with a different mana cost, devotion should use the copied mana cost, not the Clone's original.
**Fix**: Add a comment at line 5403 citing CR 700.5a and documenting this as a known deviation. The full fix would require computing Layer 1-3 partial characteristics for each controlled permanent before summing devotion. This is complex and the affected interactions are rare. Acceptable for pre-alpha.

#### Finding 2: Recursive calculate_characteristics in YouControlNOrMoreWithFilter

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:5356`
**CR Rule**: 613.1 (layer system ordering)
**Issue**: `YouControlNOrMoreWithFilter` calls `calculate_characteristics(state, obj.id)` for each battlefield permanent to resolve types for the filter check. This function is called from `is_effect_active`, which is itself called from within `calculate_characteristics`. The recursion is safe because `im-rs` state is immutable (no mutation during traversal), but it has two concerns: (1) O(n^2) performance for n battlefield objects with conditional statics, (2) theoretical possibility of infinite recursion if a conditional static modifies types that the same or another conditional static's YouControlNOrMoreWithFilter checks. In practice, Metalcraft (artifact count) does not interact with type-removing effects this way.
**Fix**: Add a comment above the `calculate_characteristics` call documenting that the recursion terminates because `is_effect_active` filters are checking types on other objects, not the object being calculated. If performance becomes an issue, consider using base characteristics for the filter check.

#### Finding 4: Nadaar, Selfless Paladin wrong mana cost

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/nadaar_selfless_paladin.rs:16`
**Oracle**: Mana Cost: {2}{W} (CMC 3)
**Issue**: Card def has `ManaCost { generic: 3, white: 1, ..Default::default() }` which represents `{3}{W}` (CMC 4). The header comment also says `{3}{W}` incorrectly. This makes the card cost 1 more to cast than it should, and devotion calculations on it would return wrong values (1 white instead of 1 white, same actually, but CMC would be wrong for effects that care about mana value).
**Fix**: Change line 16 to `ManaCost { generic: 2, white: 1, ..Default::default() }`. Change the header comment from `{3}{W}` to `{2}{W}`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 604.1 | Yes | Indirectly | Static abilities are always-on statements |
| 604.2 | Yes | Yes | test_conditional_static_life_threshold, _untapped, etc. (all 11 tests) |
| 604.3a(5) | Yes | N/A | Conditional statics are NOT CDAs; is_cda=false on all conditional effects |
| 613.1 | Yes | Yes | Layer order applied correctly; TypeChange, Ability, PtModify all tested |
| 613.1d | Yes | Yes | test_conditional_static_remove_type, _devotion_single, _devotion_multicolor |
| 700.5 | Yes | Yes | test_conditional_static_devotion_single, _devotion_multicolor |
| 700.5a | Partial | No | Devotion reads base mana costs, not Layer 1-3 resolved (see Finding 1) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| serra_ascendant | Yes | 0 | Yes | Lifelink + conditional +5/+5 + flying |
| dragonlord_ojutai | Yes | 1 | Partial | TODO: combat damage trigger (DSL gap) |
| bloodghast | Yes | 2 | Partial | TODOs: CantBlock keyword, Landfall from graveyard |
| purphoros_god_of_the_forge | Yes | 1 | Partial | TODO: activated +1/+0 ability; pre-existing: self-ETB trigger (Finding 5) |
| athreos_god_of_passage | Yes | 1 | Partial | TODO: death trigger with opponent choice |
| iroas_god_of_victory | Yes | 1 | Partial | TODO: blanket damage prevention for attackers |
| nadaar_selfless_paladin | **No** | 0 | **No** | **Finding 4: mana cost {3}{W} should be {2}{W}** |
| beastmaster_ascension | Yes | 0 | Partial | Pre-existing: "you may" not modeled (Finding 6) |
| quest_for_the_goblin_lord | Yes | 0 | Yes | Trigger + conditional static both correct |
| arixmethes_slumbering_isle | Yes | 2 | Partial | TODOs: ETB counters, spell-cast trigger; type change logic correct |
| razorkin_needlehead | Yes | 1 | Partial | TODO: opponent-draws-card trigger |
| mox_opal | Yes | 0 | Yes | Metalcraft activation condition correct |
| indomitable_archangel | Yes | 1 (blocked) | Partial | BLOCKED on PB-25 EffectFilter (correctly documented) |

## Test Coverage Assessment

The 11 tests in `conditional_statics.rs` provide good coverage:
- Each of the 5 new Condition variants has at least one test
- RemoveCardTypes has an isolation test
- Toggle behavior (condition change mid-game) is tested
- Source filter resolution is tested
- Both positive (condition met) and negative (condition not met) paths are tested for each variant
- CR citations present in all test doc comments

Missing test coverage (LOW priority):
- No test for `YouControlNOrMoreWithFilter` with Mox Opal (activation condition path)
- No test for multi-color devotion with hybrid mana symbols specifically
- No test for `DevotionToColorsLessThan` threshold exactly at boundary (devotion == threshold should be false since it's "less than")

## Summary

The engine changes are well-implemented and correct for the common cases. The `condition` field
is properly threaded through ContinuousEffectDef -> ContinuousEffect -> is_effect_active ->
check_static_condition. Hash support is complete. The RemoveCardTypes layer modification works
correctly in Layer 4. The Source filter resolution in register_static_continuous_effects is correct.

The one HIGH finding is a simple mana cost typo on Nadaar that needs immediate correction.
The two MEDIUM findings (CR 700.5a devotion layer timing, recursive calculate_characteristics)
are acceptable for pre-alpha with documentation added.

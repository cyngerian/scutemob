# Primitive Batch Review: PB-CC-B — `TargetFilter.has_counter_type` field addition

**Date**: 2026-04-29
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 122.1, CR 122.2, CR 122.6 (counters), CR 121 (drawing — *cited but not the right rule*), CR 613.1d (layer-resolved characteristics), Ruling 2020-11-10 (Armorcraft Judge: counts creatures, not counters)
**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (TargetFilter struct, ~2453-2461)
- `crates/engine/src/state/hash.rs` (HASH_SCHEMA_VERSION 9→10, line 51; TargetFilter HashInto, line 4210)
- `crates/engine/src/effects/mod.rs` (helper at 6588-6606; 14 callsites: 876, 1063, 1194, 2132, 2533, 3970, 4866, 5502, 5989, 6015, 6762, 6774, 7149, 7344)
- `crates/engine/src/rules/layers.rs` (2 CDA callsites: 1443, 1464)
**Card defs reviewed**: `armorcraft_judge.rs` (1 card)
**Tests reviewed**: `crates/engine/tests/armorcraft_judge_etb.rs` (4 tests)

## Verdict: **NEEDS FIXES**

The engine implementation is mechanically correct and well-scoped: the new
`has_counter_type: Option<CounterType>` field is added to `TargetFilter` with
`#[serde(default)]`, hashed at the right position with a +1 schema bump, and
checked via a `pub(crate) check_has_counter_type` helper at every battlefield
callsite where `TargetFilter` is consulted (16 total, including both CDA paths).
The Armorcraft Judge card def correctly uses `EffectAmount::PermanentCount`
with `has_counter_type: Some(CounterType::PlusOnePlusOne)`, matching oracle
text exactly. However, the review surfaces **two HIGH findings**: (1) all four
shipped tests have inadequate library setup such that the assertions cannot
distinguish a working filter from a broken one — by the project convention
(`memory/conventions.md` "Test-validity MEDIUMs are fix-phase HIGHs"), this is
HIGH severity; and (2) the CR rule citations throughout doc-comments and tests
say "CR 121" when the correct rule for counters is **CR 122** (CR 121 is about
drawing cards). Both findings should block merge until corrected.

## Engine Change Findings

| #  | Severity | File:Line                            | Description |
|----|----------|--------------------------------------|-------------|
| E1 | **HIGH** | (multiple)                           | **Wrong CR citation throughout — CR 121 used in place of CR 122.** Doc-comments at `card_definition.rs:2455`, `state/hash.rs:47`, `effects/mod.rs:6588-6606`, and at all 16 callsites cite "CR 121" or "CR 121.6"; the correct rule for counters is **CR 122** (122.1 = "a counter is a marker placed on an object…"; 122.2 = "counters are not retained if that object moves from one zone to another"; 122.6 = "counters being put on an object … while it's on the battlefield"). CR 121 is about drawing cards. Misciting the CR is a correctness hazard for future readers and contradicts `memory/conventions.md` "Comprehensive Rules Citation Format". **Fix**: Replace every `CR 121` / `CR 121.6` reference in this PB's diff with the correct CR 122 subrule (122.1 for "counters live on objects", 122.2 for "counters cease on zone change", 122.6 for "counters on the battlefield"). 16 call-site comments + 1 helper doc-comment + 1 hash module comment + 1 field doc-comment + 1 card-def comment + 4 test files. |
| E2 | LOW      | `effects/mod.rs:6598-6606`           | **Helper's doc-comment cites CR 121 instead of CR 122.** Same root cause as E1, but flagged separately because this is the new helper's primary contract documentation. **Fix**: rewrite as `/// CR 122.1 / 122.6: counters live on `GameObject`, not `Characteristics`.` |
| E3 | LOW      | `effects/mod.rs:2128-2129`, `3963-3964`, `4864-4865`, `5978-5979` | **Library/graveyard "naturally fails" claim is correct but rule-citation is wrong.** The comment "library/graveyard cards have no counters; has_counter_type naturally fails for them" is correct (per CR 122.2: counters cease to exist on zone change). But the `CR 121` citation at each of these four sites is wrong. **Fix**: replace with `CR 122.2`. |

## Test Findings

| #  | Severity | Test                                                                  | Description |
|----|----------|-----------------------------------------------------------------------|-------------|
| T1 | **HIGH** | `armorcraft_judge_etb.rs` — all 4 tests                                | **Test setup cannot distinguish working filter from broken filter.** Every test uses 0 or 1 library cards; `Effect::DrawCards` with `count = N` against an empty/short library results in `hand_count = min(N, library_size)`. Under any plausible filter break (counts all creatures; counts counters instead of creatures; ignores controller), the observed `hand_count` is the same as the correct value because the library bottleneck masks the count. Per `memory/conventions.md` "Test-validity MEDIUMs are fix-phase HIGHs": this is a HIGH. **Fix below in Finding Details.** |

### Per-test analysis (T1)

| Test | Library size | Expected `n` (correct) | `n` if filter broken (counts all creatures) | `n` if filter broken (sums counters) | Distinguishable? |
|------|-------------|-----------------------|---------------------------------------------|--------------------------------------|------------------|
| `armorcraft_judge_no_counters_zero_draw`                | **0**         | 0 → hand=0 | 4 → hand=0 (lib empty) | 0 → hand=0 | **NO** (all cases yield hand=0) |
| `armorcraft_judge_one_creature_with_counter_draws_one`  | **1**         | 1 → hand=1 | 3 → hand=1 (lib bottleneck) | 1 → hand=1 | **NO** (broken filter yields 3, but only 1 card available) |
| `armorcraft_judge_multiple_counters_one_creature_still_one` | **1**     | 1 → hand=1 | 2 → hand=1 (lib bottleneck) | 3 → hand=1 (lib bottleneck) | **NO** (the very behavior the test name claims to validate is masked) |
| `armorcraft_judge_filters_other_players_creatures`      | **0**         | 0 → hand=0 | 1 (opp's creature) → hand=0 (no lib) | 1 → hand=0 | **NO** (controller filter break is invisible) |

**Particularly damaging**: test 3's name promises to validate Ruling 2020-11-10
("counts CREATURES, not counters") — i.e., that 1 creature with 3 counters yields
n=1, not n=3. With only 1 library card, `n=1` and `n=3` produce identical
`hand_count=1`. The test does not actually exercise the ruling it claims to test.

This is the exact failure mode `conventions.md` warns about: "if the test title
says 'pre-death LKI' and the setup can't discriminate pre- vs post-death
evaluation, that is a test-validity bug with the same urgency as a
wrong-game-state bug."

## Card Definition Findings

| #  | Severity | Card               | Description |
|----|----------|--------------------|-------------|
| C1 | (none)   | `armorcraft_judge.rs` | **Card def matches oracle text exactly.** All fields verified against `mcp__mtg-rules__lookup_card`: name, mana cost {3}{G}, type Creature — Elf Artificer, P/T 3/3, oracle text verbatim. ETB trigger uses `EffectAmount::PermanentCount` (correct primitive — counts creatures, not counters per Ruling 2020-11-10). `has_counter_type: Some(CounterType::PlusOnePlusOne)` is the correct value. No remaining TODOs. No findings. |

### Finding Details

#### Finding E1: Wrong CR citation — CR 121 used in place of CR 122

**Severity**: HIGH
**Files** (all in this PB's diff):
- `crates/engine/src/cards/card_definition.rs:2455` — field doc-comment
- `crates/engine/src/state/hash.rs:47` — schema-version history entry
- `crates/engine/src/effects/mod.rs:6588-6597` — helper doc-comment
- `crates/engine/src/effects/mod.rs:875, 1062, 1193, 2128, 2532, 3963, 4864, 5501, 5978, 6013, 6761, 6773, 7148, 7343` — 14 inline call-site comments
- `crates/engine/src/rules/layers.rs:1442, 1463` — 2 CDA call-site comments
- `crates/engine/src/cards/defs/armorcraft_judge.rs:5-7` — card def doc-comment
- `crates/engine/tests/armorcraft_judge_etb.rs:7-11, 62, 87, 101, 134, 192, 222` — 4 test files cite CR 121.1 / 121.6

**CR Rule (verified independently via MCP)**:
- CR 121 = "Drawing a Card" (CR 121.1: "A player draws a card by putting the top card of their library into their hand").
- CR 122 = "Counters" (CR 122.1: "A counter is a marker placed on an object or player that modifies its characteristics"; CR 122.2: "Counters on an object are not retained if that object moves from one zone to another"; CR 122.6: "spells and abilities refer to counters being put on an object … while it's on the battlefield").

**Issue**: Every doc-comment and test in the PB-CC-B diff that intends to cite the
counter CR rule cites CR 121 instead of CR 122. The plan brief itself uses
"CR 121" repeatedly (e.g., line 1455-1461 of `card_definition.rs`'s new
doc-comment claims "Per CR 121: counters are tracked in `GameObject.counters`").
This is incorrect. CR 121 is about drawing cards. The correct rule for "counters
live on objects, not characteristics" is **CR 122.1** ("A counter is a marker
placed on an object"). The correct rule for "library/graveyard cards have no
counters" is **CR 122.2** ("Counters on an object are not retained if that
object moves from one zone to another"). The correct rule for "counters on
the battlefield" semantics is **CR 122.6**.

**Fix**: Search-and-replace across the PB-CC-B diff (NOT across the entire
codebase — pre-existing references to CR 121 in damage/draw paths are
correct):
- `CR 121:` → `CR 122.1:` (where the comment is about counters living on objects)
- `CR 121.1:` → `CR 122.1:` (in tests)
- `CR 121.6:` → `CR 122.6:` (in tests; CR 122.6 is "counters being put on an object … while it's on the battlefield")
- `Per CR 121:` → `Per CR 122.1:` (in field doc-comments)
- `library/graveyard cards have no counters` justification → cite **CR 122.2** ("counters cease to exist on zone change")
- The Armorcraft Judge card-def doc-comment already cites `CR 121.1: Counters are artifacts of game state tracked on the object.` and `CR 121.6: Counters on permanents are tracked in GameObject.counters.` — both must be CR 122.1 / 122.6.

#### Finding T1: Test setup cannot distinguish working filter from broken filter

**Severity**: HIGH (per `memory/conventions.md` "Test-validity MEDIUMs are fix-phase HIGHs")
**File**: `crates/engine/tests/armorcraft_judge_etb.rs` — all 4 tests
**Issue**: `Effect::DrawCards { count: PermanentCount{...} }` resolves the count
once, then attempts `count` individual draws via `draw_one_card`. With an empty
or short library, the observed `hand_count` is `min(count, library_size)`. None
of the 4 tests has a library large enough to detect a wrong count, so under
plausible filter breaks (counts all creatures; sums counters; ignores
controller) the assertion still passes.

**Concrete demonstration** (test 3):
- Setup: 1 creature with 3 +1/+1 counters; Armorcraft Judge; 1 library card.
- Test name promises Ruling 2020-11-10: "creatures, not counters" — i.e. validate
  that the count is 1 (one creature) not 3 (three counters).
- Correct filter behavior: `n = 1` → 1 draw → `hand_count = 1`.
- Hypothetical broken filter that sums counters: `n = 3` → 3 attempted draws,
  but library only has 1 card → `hand_count = 1`.
- **Both yield identical observed value.** The test name promises to exercise a
  semantic distinction the setup cannot reach.

**Fix**: For each test, add enough library cards that the broken-filter and
working-filter counts produce distinguishable hand sizes. Specific guidance:

- **Test 1 (`no_counters_zero_draw`)**: Add at least 4 library cards. Then a
  broken filter that counted all 4 creatures would yield `hand_count = 4`,
  distinguishable from the correct `hand_count = 0`.
- **Test 2 (`one_creature_with_counter_draws_one`)**: Add at least 4 library
  cards. Then `n = 3` (broken: counts all P1 creatures incl. Judge) → `hand_count
  = 3`, distinguishable from correct `hand_count = 1`. Also add a P2 creature
  with a counter to also exercise the controller filter implicitly.
- **Test 3 (`multiple_counters_one_creature_still_one`)**: Add at least 4
  library cards. Then a broken "sum counters" filter yields `n = 3` →
  `hand_count = 3`, distinguishable from correct `hand_count = 1`. **This is
  the test the convention is most strict about — the test name promises to
  validate Ruling 2020-11-10 and currently does not.**
- **Test 4 (`filters_other_players_creatures`)**: Add at least 2 library cards
  for P1. Then a broken filter that ignored the controller restriction would
  yield `n = 1` (opponent's creature) → `hand_count = 1`, distinguishable from
  correct `hand_count = 0`.

After the fix, each test should explicitly assert against the broken-case
hand_count value to make the discriminator visible (e.g. `assert_eq!(hand_count,
0, "must be 0 not 4 (broken filter would count all 4 creatures)")`).

## CR Coverage Check

| CR Rule | Implemented? | Tested?                | Notes |
|---------|--------------|------------------------|-------|
| CR 122.1 (counters as markers on objects) | Yes — helper reads `GameObject.counters` | Partial (T1) | Tests cite "CR 121.1" — wrong number. Setup cannot distinguish behavior (T1 HIGH). |
| CR 122.2 (counters cease on zone change)  | Implicit (library/graveyard objects have empty counters maps) | No | Documentary citation is wrong (CR 121 → CR 122.2). |
| CR 122.6 (counters put on battlefield)    | Yes — battlefield-scoped check | Implicit | Cited as CR 121.6 throughout. |
| CR 613.1d (layer-resolved chars before filter check) | Yes — every callsite uses `calculate_characteristics` | Implicit (test setups don't exercise layer effects) | Standard pattern, follows precedent. |
| Ruling 2020-11-10 (Armorcraft Judge: creatures not counters) | Yes — uses `PermanentCount`, not summed counters | **NO** (T1 HIGH) | Test 3 promises this, doesn't reach it. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|--------------|-----------------|---------------------|-------|
| Armorcraft Judge | Yes | 0 | Yes | Mana cost {3}{G}, type Creature — Elf Artificer, P/T 3/3, oracle text verbatim. Uses `EffectAmount::PermanentCount` with `has_counter_type: Some(PlusOnePlusOne)` — exactly correct for Ruling 2020-11-10. Card-def doc-comment cites "CR 121.1 / 121.6" (wrong; should be CR 122). |

## Wiring Audit

The brief specified 11 battlefield callsites; the worker added 16 actual
call-site checks (14 in `effects/mod.rs` + 2 in `rules/layers.rs`). Verified
each one:

| Callsite | File:line | Effect / Use | Counter check present? | Layer-resolved chars used? |
|----------|-----------|--------------|------------------------|---------------------------|
| `Effect::DestroyAll` | `effects/mod.rs:876` | filter battlefield | Yes | Yes |
| `Effect::ExileAll` (sweep) | `effects/mod.rs:1063` | filter battlefield | Yes | Yes |
| `Effect::BounceAll` | `effects/mod.rs:1194` | filter battlefield | Yes | Yes |
| `Effect::SearchLibrary` | `effects/mod.rs:2132` | filter library/grave (uniform; harmless) | Yes | N/A (lib chars are base) |
| `Effect::SacrificePermanents` (PB-SFT) | `effects/mod.rs:2533` | filter battlefield | Yes | Yes |
| `Effect::RevealAndRoute` | `effects/mod.rs:3970` | top-N library partition | Yes (uniform) | N/A |
| Graveyard search (Living Death etc.) | `effects/mod.rs:4866` | filter graveyard | Yes (uniform) | N/A |
| `EffectTarget::AllPermanentsMatching` | `effects/mod.rs:5502` | resolve target list | Yes | Yes |
| `EffectAmount::CardCount` | `effects/mod.rs:5989` | zone-aware count | Yes (uniform) | Yes (Battlefield branch) |
| `EffectAmount::PermanentCount` (primary) | `effects/mod.rs:6015` | count permanents | Yes | Yes |
| `Condition::YouControlPermanent` | `effects/mod.rs:6762` | static-condition check | Yes | Yes |
| `Condition::OpponentControlsPermanent` | `effects/mod.rs:6774` | static-condition check | Yes | Yes |
| `Condition::YouControlNOrMoreWithFilter` (static) | `effects/mod.rs:7149` | static-condition check | Yes | Yes |
| `ForEachTarget::EachPermanentMatching` | `effects/mod.rs:7344` | iterate permanents | Yes | Yes |
| `EffectAmount::PermanentCount` (CDA path) | `rules/layers.rs:1443` | CDA permanent count | Yes | No (uses base — intentional, prevents recursion) |
| `EffectAmount::CardCount` (CDA path) | `rules/layers.rs:1464` | CDA card count | Yes | No (intentional) |

**No missed callsites** observed. `EffectFilter` (used in the layer-system
continuous-effect filter at `layers.rs:600+`) is a *separate enum* from
`TargetFilter` and is correctly out of scope. Replacement-effect filters
(`ObjectFilter`, `DamageTargetFilter`, `PlayerFilter`) are also separate types
and out of scope.

## Hash Correctness

- `HASH_SCHEMA_VERSION` bumped 9 → 10 with v10 history entry. ✓
- `impl HashInto for TargetFilter` adds `self.has_counter_type.hash_into(hasher)` as the **last** field after `exclude_chosen_subtype`. ✓
- `Option<T>` and `CounterType` already have `HashInto` impls (used by other counter-tracking code). ✓
- 5 sentinel test files updated 9u8 → 10u8 per worker brief; tests pass at 2675 (4 new + 2671 prior). ✓ (per worker brief; not directly verified by reviewer, no findings depend on this.)

## Backward Compatibility

- `#[serde(default)]` on `pub has_counter_type: Option<CounterType>` ensures pre-v10 serialized state deserializes as `None` (no restriction). ✓
- `Default for TargetFilter` (derived) yields `has_counter_type: None`. ✓
- LOW: no test exercises this explicitly, but the pattern matches every prior `#[serde(default)]` field on `TargetFilter` (`is_token`, `has_subtypes`, etc.) so the risk is minimal. No finding.

## Architectural-Invariant Check

- ✓ Counters live on `GameObject`, not `Characteristics`. Helper takes
  `&GameObject`, NOT folded into `matches_filter()`.
- ✓ Helper is `pub(crate)` — appropriate (called from `layers.rs` cross-module).
- ✓ No `.unwrap()` in library code; helper uses `match` on the `Option`.
- ✓ Hash-bump rule (`memory/conventions.md`): bumped on field addition. ✓
- ✓ "Default to defer" (PB-N standing rule): worker did not extend scope — only the field, the helper, and 16 call-site additions.

## Summary Recommendation

**FIX before merge**: address E1 (CR 121 → CR 122) and T1 (test setups). E1 is
mechanical (search-and-replace within the diff). T1 requires extending each
test's library and adding distinguishing assertions; the test count stays at 4
but the assertions become load-bearing. After both fixes, this PB ships clean.

The engine implementation itself (struct field, helper, hash, 16 callsites,
card def) is correct and well-scoped. No HIGH or MEDIUM findings on engine
behavior. The two HIGH findings are entirely in documentation and test design.

## Per-criterion mapping (ESM acceptance criteria 3691-3697)

| Criterion | Status | Notes |
|-----------|--------|-------|
| 3691 — `pub has_counter_type: Option<CounterType>` field added with `#[serde(default)]`         | **MET** | `card_definition.rs:2461`. ✓ |
| 3692 — `check_has_counter_type` helper added; checked at every battlefield callsite             | **MET** | 14 + 2 = 16 callsites verified. ✓ |
| 3693 — Armorcraft Judge ETB uses the new field with `Some(CounterType::PlusOnePlusOne)`        | **MET** | `armorcraft_judge.rs:34`. ✓ |
| 3694 — `HASH_SCHEMA_VERSION` bumped 9 → 10; hash impl extended for `TargetFilter`              | **MET** | `state/hash.rs:51, 4210`. ✓ |
| 3695 — 4 tests as named, all pass; tests cite CR rules                                          | **MET-WITH-FAIL**: tests exist with the correct names and pass; **but** CR rule citations are wrong (CR 121 → CR 122; see E1) and **all 4 test setups fail to distinguish working from broken filter** (see T1). T1 is HIGH per project convention. |
| 3696 — `cargo test -p mtg-engine` passes (2675); clippy clean; fmt clean; build clean         | **MET** (per worker brief) | Reviewer did not re-run; trusts brief on this. |
| 3697 — Doc-comments cite CR rules accurately                                                    | **NOT MET** (E1 HIGH): all CR 121 citations should be CR 122. Mechanical fix. |

**Final verdict**: NEEDS FIXES — 2 HIGH (E1, T1), 2 LOW (E2, E3 — both subsumed
by E1 fix). Engine behavior is correct; documentation and tests need surgery.

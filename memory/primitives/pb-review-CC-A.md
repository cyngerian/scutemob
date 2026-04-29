# Primitive Batch Review: PB-CC-A — `EffectAmount::PlayerCounterCount`

**Date**: 2026-04-29
**Reviewer**: primitive-impl-reviewer (Opus 4.7 1M)
**CR Rules**: CR 122.1 (counters; "the number of counters on it"), CR 122.1f (poison counters loss SBA), CR 613 (layer system; no recursion concern), CR 604.3 / CR 611.3a (CDAs; static-ability live re-evaluation), CR 608.2h (X-locked-at-resolution vs CR 611.3a), CR 800.4i (LKI for left players), Vishgraz ruling 2023-02-04 (sum semantic for "each poison counter your opponents have")
**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (variant added at lines 2217-2247 with sum-semantic and CDA-safety doc)
- `crates/engine/src/effects/mod.rs` (resolve_amount arm at lines 6156-6178)
- `crates/engine/src/rules/layers.rs` (resolve_cda_amount arm at lines 1528-1546; resolve_cda_player_target unchanged at 1595-1614)
- `crates/engine/src/state/hash.rs` (HASH_SCHEMA_VERSION 11→12 with history entry at 54-60; hash arm with discriminant 16 at 4456-4462)
**Card defs reviewed**: `crates/engine/src/cards/defs/vishgraz_the_doomhive.rs` (1 card, deferred per Option B with PB-CC-C-followup citation)
**Tests reviewed**: `crates/engine/tests/primitive_pb_cc_a.rs` (8 tests T1-T8)
**Test-file sentinel updates**: `pbp_power_of_sacrificed_creature.rs:782`, `pbn_subtype_filtered_triggers.rs:553-559`, `pbd_damaged_player_filter.rs:594-601`, `pbt_up_to_n_targets.rs:404-414` (asserts 12 with stale function name `test_pbt_hash_schema_version_is_11`), `primitive_pb_cc_a.rs:99-105` (own sentinel test).

## Verdict: **PASS-WITH-NITS**

The new `EffectAmount::PlayerCounterCount { player, counter }` variant ships
correctly. The engine surface is consistent across `resolve_amount` (effect
path) and `resolve_cda_amount` (layer path), the sum semantic per CR 122.1 +
Vishgraz 2023-02-04 is correctly implemented, the hash discriminant is unique
(16), the schema version bump is properly historied (11→12), and the 8-test
suite covers Controller/EachOpponent/EachPlayer/DeclaredTarget/non-Poison/CDA
synthetic-harness/CDA scaling-distribution paths.

The Vishgraz card-def Option-B deferral is correct and well-cited — the
identical CR 611.3a "Layer-7c dynamic-static" trap that PB-CC-C documented for
Exuberant Fuseling applies here. The TODO comment in
`vishgraz_the_doomhive.rs` correctly explains both the engine path (`is_cda:
false` registration via `register_static_continuous_effects`) and the
debug_assert silent no-op trap. AC 3710 is correctly classified **N/A —
deferred** with explicit PB-CC-C-followup attribution.

Open nits at LOW severity: (1) one test-name/assertion mismatch in
`pbt_up_to_n_targets.rs` (function still named `test_pbt_hash_schema_version_is_11`
but now asserts `12u8`); (2) subtle behavioral inconsistency between
`resolve_player_target_list` (filters `has_lost`) and `resolve_cda_player_target`
(does not filter); (3) test 6 uses `CounterType::PlusOnePlusOne` as the
"unsupported on player" case, which is a slightly weak choice when the
contract specifically targets future energy/experience/rad counter kinds; (4)
test 7 uses Layer 7a `SetPtDynamic` as a synthetic harness — correctly
documented in its docstring, but worth flagging that Layer 7c CDA-style
re-evaluation has no test in this batch (intentionally — that's the
deferred PB-CC-C-followup primitive).

No HIGH or MEDIUM findings open. PB-CC-A is mergeable.

## Engine Change Findings

| #  | Severity | File:Line | Description |
|----|----------|-----------|-------------|
| E1 | LOW      | `rules/layers.rs:1604-1610` | **CDA path does not filter `has_lost` players.** `resolve_cda_player_target` for `EachPlayer`/`EachOpponent` iterates `state.turn.turn_order` without filtering players who have lost. `resolve_player_target_list` (effects/mod.rs:5588-5612) DOES filter `has_lost`. Per CR 800.4i, last-known-info applies for left players, so "include" is arguably more CR-correct, but the inconsistency means the same logical query yields different sums in spell-effect vs CDA contexts. **Fix:** add an inline comment at `resolve_cda_player_target:1602-1610` documenting the deliberate divergence (cite CR 800.4i for last-known-info), OR add `state.players.get(&p).map(|ps| !ps.has_lost).unwrap_or(false)` filter for symmetry. Either is acceptable; documentation is sufficient at LOW. |
| E2 | LOW      | `cards/card_definition.rs:2230-2237` | **Counter-kind support documented but no `Energy`/`Experience`/`Rad` field on `PlayerState`.** The doc-comment correctly states "today only `CounterType::Poison` reads from `PlayerState::poison_counters`" and "non-Poison kinds resolve to 0 (defensive)." This contract is sound, but a future implementer may forget to extend BOTH `resolve_amount` AND `resolve_cda_amount` when adding (e.g.) `PlayerState::energy_counters`. **Fix:** add a CRITICAL-INVARIANT note in the variant docstring requiring that any new `CounterType` arm be added in BOTH `effects/mod.rs:6165-6178` AND `rules/layers.rs:1528-1546`. Mirror the PB-CC-C "Static-ability footgun" precedent. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | LOW | `vishgraz_the_doomhive.rs:49-79` | **TODO comment is correct and thorough.** The 30-line citation is excellent — names the engine trap (`is_cda: false` via `register_static_continuous_effects`), the layer-path trap (`debug_assert!` in `apply_modification`), the precedent (PB-CC-C / Exuberant Fuseling), and points to both the review file and the retriage. Mirrors PB-CC-C exactly. **No fix required**; recorded for completeness. |

## Test Findings

| # | Severity | Test/File | Description |
|---|----------|-----------|-------------|
| T1 | LOW | `pbt_up_to_n_targets.rs:404` | **Stale test function name vs assertion.** Function is `test_pbt_hash_schema_version_is_11` but asserts `HASH_SCHEMA_VERSION == 12u8`. The docstring at line 400 also still says "HASH_SCHEMA_VERSION is 11 (PB-CC-C bump)". The assertion is correctly updated; only the symbolic name and docstring are stale. **Fix:** rename to `test_pbt_hash_schema_version_is_12` and update the docstring to "HASH_SCHEMA_VERSION is 12 (PB-CC-A bump)". |
| T2 | LOW | `primitive_pb_cc_a.rs:301` | **Test 6 picks a weak "unsupported counter kind" example.** Using `CounterType::PlusOnePlusOne` as the non-Poison test case is contractually correct (the variant returns 0), but a more discriminating choice would be a counter type that genuinely could plausibly live on a player in future (e.g., `Custom("energy")` or `Custom("experience")`), exactly the kinds the doc-comment names. The test would be strictly more informative if it assert-ed against an "energy-shaped" Custom counter, since `PlusOnePlusOne` is semantically nonsensical on a player and might be silently masked by other future bugs. **Fix:** swap `CounterType::PlusOnePlusOne` for `CounterType::Custom("energy".to_string())` and update the comment to clarify "these counter kinds are listed in the variant doc-comment as future-proof targets." Optional. |
| T3 | LOW | `primitive_pb_cc_a.rs:323-381` | **Test 7 uses `SetPtDynamic` (Layer 7a) as a synthetic harness — correctly documented but worth flagging.** The docstring honestly states "this test exercises the CDA-evaluation path. It does NOT ship a Layer-7c CDA-style modify (which is the deferred PB-CC-C-followup primitive — Vishgraz's card def). It uses `CdaPowerToughness` as a synthetic harness because that is the only Layer-7a dynamic-CDA registration today." This is the right call — it provides real coverage of `resolve_cda_amount` flowing through the layer system. **No fix required**; recorded for transparency. The actual Vishgraz Layer-7c CDA path will need its own end-to-end test when PB-CC-C-followup ships. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|--------------|---------|-------|
| 122.1   | Yes (counter count read) | Yes (T2 controller, T3 EachOpponent, T4 EachPlayer, T5 DeclaredTarget) | "The number of counters on a player" semantic correctly reads `PlayerState::poison_counters`. |
| 122.1f  | N/A (poison counters loss SBA) | N/A | Pre-existing SBA in state machine; PB-CC-A only reads poison count, doesn't enforce loss. |
| 604.3   | Yes (CDA flows through `resolve_cda_amount`) | Yes (T7, T8 synthetic harness using Layer 7a SetPtDynamic) | CDA evaluation path covered for `EachOpponent` sum semantic. |
| 608.2h  | N/A in PB-CC-A scope | N/A | Spell-resolution X-lock semantic is PB-CC-C territory; PB-CC-A's `resolve_amount` is reached during spell resolution but the lock-in semantic is implemented by the surrounding substitution arm in `effects/mod.rs:2330-2347`, not by PlayerCounterCount itself. |
| 611.3a  | N/A in PB-CC-A scope (deferred) | N/A | Vishgraz's Layer-7c CDA-style continuous re-evaluation is deferred to PB-CC-C-followup; documented in the Vishgraz card def TODO. |
| 613.4a  | N/A (uses Layer 7a `SetPtDynamic`, not Layer 7c) | Yes (T7, T8 use SetPtDynamic) | Layer 7a CDA dispatch via `SetPtDynamic` exercises `resolve_cda_amount`. Variant works correctly here. |
| 800.4i  | Implicit (CDA path includes left players via turn_order) | No (no test of has_lost) | Inconsistency with `resolve_player_target_list` (filters has_lost). LOW E1. |
| Vishgraz 2023-02-04 ruling (sum semantic) | Yes (T3 explicit sum: 1+2+5=8 with controller's 100 excluded; T8 explicit "Sum-semantic: 5+2+1 = 8 (NOT max(5)=5, NOT count(>0)=3)") | Yes | Explicit discriminating assertions per the ruling. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|--------------|-----------------|---------------------|-------|
| Vishgraz, the Doomhive | Yes (oracle text in card def matches MCP verbatim, including the deferred third clause) | 1 (the static "+1/+1 for each poison counter your opponents have" clause) | N/A — deferred to PB-CC-C-followup | Menace + Toxic 1 + ETB Phyrexian Mite token half (creature-side, including "This token can't block.") shipped correctly. The third clause is BLOCKED on Layer-7c dynamic-static primitive. TODO citation is thorough. |

## Acceptance Criteria Verification

| AC ID | Description | Verdict |
|-------|-------------|---------|
| 3706 | New `EffectAmount::PlayerCounterCount { player: PlayerTarget, counter: CounterType }` variant added | **PASS** — variant at `card_definition.rs:2244-2247` with sum-semantic and CDA-safety doc-comment. |
| 3707 | `resolve_amount` arm in `effects/mod.rs` reads `PlayerState::poison_counters`, sums for `EachOpponent`/`EachPlayer` | **PASS** — arm at `effects/mod.rs:6165-6178`; `Poison` reads `ps.poison_counters`, others return 0. |
| 3708 | `resolve_cda_amount` arm in `rules/layers.rs` mirrors `resolve_amount` | **PASS** — arm at `rules/layers.rs:1536-1546` with same Poison/0 contract. |
| 3709 | `state/hash.rs` arm with discriminant 16; HASH_SCHEMA_VERSION bumped 11→12 | **PASS** — discriminant 16 at hash.rs:4458-4462 with `player.hash_into` + `counter.hash_into`; HASH_SCHEMA_VERSION = 12 at line 61; history entry 12 at lines 54-60. |
| 3710 | `vishgraz_the_doomhive.rs` re-authored with CDA Layer 7c `ModifyBothDynamic { amount: PlayerCounterCount{EachOpponent, Poison} }`, `is_cda=true`, `EffectFilter::Source` | **N/A — deferred per Option B (correct decision)** — card def reverted with PB-CC-C-followup citation. The construction asked for in 3710 does NOT work in the current engine: (a) `AbilityDefinition::Static { continuous_effect }` registers with `is_cda: false` per `replacement.rs:1795`; (b) `ModifyBothDynamic` reaching `apply_modification` triggers `debug_assert!(false, ...)` at `layers.rs:1164-1170` and silently no-ops in release. This is the identical trap PB-CC-C documented for Exuberant Fuseling. Reviewer agrees with deferral. |
| 3711 | Mandatory tests (≥3) in `crates/engine/tests/primitive_pb_cc_a.rs` covering each PlayerTarget variant + non-Poison + CDA path | **PASS** — 8 tests cover Controller (T2), EachOpponent sum (T3), EachPlayer sum (T4), DeclaredTarget (T5), non-Poison (T6), CDA Layer-7a path (T7), CDA scaling distribution (T8), plus hash sentinel (T1). Exceeds floor. |
| 3712 | All existing tests pass; clippy clean; fmt clean | Trust runner's claim ("Tests: 2716 passing. clippy --all-targets -- -D warnings clean. fmt clean."). Sentinel-assertion test-file updates verified manually in this review. |
| 3713 | HASH_SCHEMA_VERSION sentinel sweep (5 test files) | **PASS-WITH-NITS** — `pbp_power_of_sacrificed_creature.rs:782` ✓, `pbn_subtype_filtered_triggers.rs:553-559` ✓, `pbd_damaged_player_filter.rs:594-601` ✓, `pbt_up_to_n_targets.rs:404-414` ✓ (assertion correct, function name stale: T1 LOW), `primitive_pb_cc_a.rs:99-105` ✓ (own sentinel test). All 5 assert `12u8`. |
| 3714 | /review pass via primitive-impl-reviewer agent (this review) | **PASS-WITH-NITS** — see verdict above. 0 HIGH/MEDIUM, 5 LOW (E1, E2, C1, T1, T2, T3). |

## Option B Vishgraz Deferral — Reviewer Concurrence

**Reviewer agrees with the Option B deferral decision.** The reasoning is
identical to PB-CC-C / Exuberant Fuseling, with one extra subtlety worth
documenting:

1. **Engine trap is real**: `register_static_continuous_effects`
   (replacement.rs:1753-1797) hardcodes `is_cda: false` for
   `AbilityDefinition::Static`. There is no static-ability path that registers
   with `is_cda: true` today — only `CdaPowerToughness` does (replacement.rs:1854),
   and that registers `SetPtDynamic` at Layer 7a (not Layer 7c).

2. **Substitution path is wrong for Vishgraz**: routing through
   `Effect::ApplyContinuousEffect` (e.g., from a "WhenEntersBattlefield"
   trigger) would substitute `ModifyBothDynamic { amount: PlayerCounterCount{...} }`
   into `ModifyBoth(N)` at ETB time and lock that value forever (CR 608.2h
   semantic, NOT CR 611.3a re-evaluation). Vishgraz's static ability requires
   live re-evaluation as opponents accumulate poison — the substitution path
   would freeze the value at ETB.

3. **Layer 7a CdaPowerToughness is wrong layer**: a mechanical port using
   `CdaPowerToughness { power: Sum(Fixed(3), PlayerCounterCount{...}),
   toughness: Sum(Fixed(3), PlayerCounterCount{...}) }` would set base P/T
   at Layer 7a — this is wrong because Layer 7b "becomes a 0/2" overrides
   would erase the dynamic value, contradicting the "+1/+1 modifier"
   semantics. The card-def TODO correctly identifies this as forbidden by
   W6 policy ("wrong-game-state").

4. **The right path is PB-CC-C-followup**: Layer-7c dynamic-static modify
   primitive that re-evaluates its `EffectAmount` on each
   `calculate_characteristics` call (mirroring how `SetPtDynamic` works at
   Layer 7a, but for Layer 7c modify rather than Layer 7a set). PlayerCounterCount
   is a building block for that future primitive — shipping it now means
   PB-CC-C-followup just needs the layer-7c-dynamic-static plumbing.

The Vishgraz card-def TODO citation at lines 49-79 is thorough and accurate.
The runner correctly avoided shipping wrong game state. **Acceptance
criterion 3710 is properly classified N/A — deferred.**

## Summary

PB-CC-A ships the `EffectAmount::PlayerCounterCount { player, counter }`
variant cleanly across both spell-effect (`resolve_amount`) and CDA
(`resolve_cda_amount`) paths. Sum semantic per CR 122.1 + Vishgraz
2023-02-04 ruling is explicitly tested with discriminating assertions
(T3: 1+2+5=8 NOT 100; T8: 5+2+1=8 NOT max=5 NOT count-of-poisoned=3).
Hash discriminant 16 unique, HASH_SCHEMA_VERSION 11→12 with history entry,
sentinel sweep applied to 5 test files (1 stale function name = T1 LOW).
Test 7 (CDA path) uses `SetPtDynamic` as a synthetic harness — correct
choice given PB-CC-A's scope explicitly excludes the Layer-7c
dynamic-static primitive (deferred to PB-CC-C-followup).

The Option B Vishgraz deferral is the right call. The 30-line TODO citation
in `vishgraz_the_doomhive.rs` correctly identifies all three engine traps
(is_cda registration, ModifyBothDynamic substitution semantic, Layer 7a
vs 7c) and points to PB-CC-C-followup for the path forward.

LOW findings (E1 has_lost inconsistency, E2 future-proofing invariant doc,
C1 informational, T1-T3 test polish) are all opportunistic cleanup — none
block merge. The build trail (clippy clean, fmt clean, 2716 tests passing)
is consistent with prior PB merges.

**Verdict: PASS-WITH-NITS** — mergeable. Recommended pre-merge cleanup:
fix T1 (rename `test_pbt_hash_schema_version_is_11` → `..._is_12`,
update docstring at line 400). All other LOWs are post-merge opportunistic.

## Previous Findings (re-review)

N/A — first review of PB-CC-A. No prior findings to track.

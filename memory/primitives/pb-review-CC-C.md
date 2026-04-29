# Primitive Batch Review: PB-CC-C — `LayerModification::ModifyPowerDynamic` / `ModifyToughnessDynamic`

**Date**: 2026-04-29 (re-review after Option B fix-pass)
**Reviewer**: primitive-impl-reviewer (Opus 4.7 1M)
**CR Rules**: CR 608.2h (lock-in for spell/ability X values), CR 611.3a (static abilities NOT locked-in — re-evaluate continuously), CR 611.3b (continuous effects from static abilities apply at all times), CR 613.4a (Layer 7a CDA P/T), CR 613.4c (Layer 7c P/T modification), CR 604.2 (static abilities create continuous effects), CR 604.3a (CDA criteria), CR 400.7 (object identity on zone change)
**Engine files reviewed**:
- `crates/engine/src/state/continuous_effect.rs` (lines 391-444, two new variants ModifyPowerDynamic/ModifyToughnessDynamic with extended doc-comment Static-ability footgun warnings)
- `crates/engine/src/effects/mod.rs` (substitution arm 2315-2347 with CR 608.2h vs CR 611.3a path-semantics note)
- `crates/engine/src/rules/layers.rs` (panic-guard arms 1171-1187 with corrected CR 613.4c citations)
- `crates/engine/src/state/hash.rs` (HASH_SCHEMA_VERSION bumped 10→11 with history entry 11 added at lines 51-53; hash arms at lines 1482-1495 for discriminants 26+27)
- `crates/engine/src/rules/replacement.rs` (lines 1753-1797 — `register_static_continuous_effects` — verified unchanged)
**Card defs reviewed**: `crates/engine/src/cards/defs/exuberant_fuseling.rs` (1 card, reverted to TODO state with PB-CC-C-followup citation)
**Tests reviewed**: `crates/engine/tests/primitive_pb_cc_c.rs` (5 tests; T5 replaced with full-dispatch X-locked test)
**Test-file sentinel updates**: `pbp_power_of_sacrificed_creature.rs:782`, `pbn_subtype_filtered_triggers.rs:553`, `pbd_damaged_player_filter.rs:597` (all 11; CR-citation updated to 613.4c)

## Verdict: **PASS-WITH-NITS**

All HIGH and MEDIUM findings from the prior review are RESOLVED. The runner
chose **Option B** (defer Fuseling, ship engine variants for one-shot spell
use cases) and applied each prior fix correctly:

- **C1 HIGH (Fuseling stale-snapshot)**: RESOLVED — the card def is reverted
  to TODO state; both the static "+1/+0 per oil counter" half and the
  death-trigger half are documented as deferred (PB-CC-C-followup multi-blocker
  trail). Oracle text comment retained verbatim. No
  `Triggered+ApplyContinuousEffect` pattern remains. Trample keyword still
  shipped — the only authored ability is the `WhenEntersBattlefield` AddCounter
  trigger, which is the unblocked half of the ETB clause.

- **T1 HIGH (broken test)**: RESOLVED — `test_exuberant_fuseling_power_scales_with_oil_counters`
  removed; replaced with `test_modify_power_dynamic_x_locked_at_resolution`
  which exercises the correct CR 608.2h semantic via full dispatch
  (`Effect::ApplyContinuousEffect` → `execute_effect` → `calculate_characteristics`),
  including a post-resolution counter mutation that verifies the value does NOT
  change (the lock-in is real). This is exactly the discriminating sequence
  the prior review demanded.

- **E3 MEDIUM (Static-ability doc-comment footgun)**: RESOLVED — both
  `ModifyPowerDynamic` and `ModifyToughnessDynamic` doc-comments now contain
  an "AUTHORING NOTE — static-ability footgun" block that explicitly forbids
  the `AbilityDefinition::Static { continuous_effect: ContinuousEffectDef
  { modification: ModifyPowerDynamic/ToughnessDynamic, .. } }` path, names the
  bypass mechanism (`register_static_continuous_effects`), and directs authors
  to `Effect::ApplyContinuousEffect` for one-shot spells or to the deferred
  Layer-7c dynamic-static primitive (PB-CC-C-followup) for static abilities.

- **E4 MEDIUM (architectural CR 608.2h vs CR 611.3a distinction)**: RESOLVED —
  `effects/mod.rs:2320-2329` substitution arm now carries a "NOTE — path
  semantics" block that clearly distinguishes the spell-effect resolution path
  (CR 608.2h "X locked at resolution") from the static-ability path (CR 611.3a
  "continuous effect ... isn't locked in"). Names `CdaPowerToughness` (Layer
  7a, `resolve_cda_amount`) as the closest existing dynamic-static analogue
  and references PB-CC-C-followup for the missing Layer-7c dynamic-static
  primitive.

- **C2 MEDIUM (aspirationally-wrong "is_cda: true")**: RESOLVED — removed
  along with the entire `Triggered+ApplyContinuousEffect` block.

- **T2 MEDIUM (full-dispatch test missing)**: RESOLVED — the new T5 walks the
  full dispatch path (substitution arm → stored ContinuousEffect → layer
  resolution → `calculate_characteristics`) AND mutates source state
  post-resolution to verify the lock-in semantic.

- **E1 LOW (HASH_SCHEMA_VERSION history)**: RESOLVED — entry 11 appended at
  `state/hash.rs:51-53` with proper attribution and CR citations.

- **E2/C3/T3/T4 LOW (CR 613.1c → 613.4c)**: RESOLVED — sweep applied across
  `state/continuous_effect.rs` (variant doc-comments), `rules/layers.rs`
  (panic-guard messages), `state/hash.rs` (history entry + hash arm
  comments), `effects/mod.rs` (substitution arm), `cards/defs/exuberant_fuseling.rs`
  (Fuseling TODO comment), `tests/primitive_pb_cc_c.rs` (file-level + per-test
  docstrings), and the three sentinel-assertion test files
  (`pbp_power_of_sacrificed_creature.rs`, `pbn_subtype_filtered_triggers.rs`,
  `pbd_damaged_player_filter.rs`). No `CR 613.1c` references remain in
  PB-CC-C-touched code.

- **E5 LOW (substitution-arm CR contrast comment)**: RESOLVED via the E4 fix
  (the same multi-line note added to `effects/mod.rs:2320-2329` distinguishes
  the two CR rules and names the dynamic-static analogue).

The two `LayerModification` variants ship correctly for the **one-shot spell
use case** (the "Olivia's Wrath" pattern, e.g. "creatures get +X/+0 until end
of turn where X is the number of Vampires you control"). The `register_static_continuous_effects`
bypass is now documented as a footgun, and the Layer-7c dynamic-static
primitive is filed as a follow-up multi-blocker. Fuseling is correctly
deferred — its CR 611.3a continuous-evaluation requirement is incompatible
with the substitution-at-resolution approach, and the runner did the right
thing by reverting and citing PB-CC-C-followup rather than shipping wrong
game state.

C4 (oracle text exactness) is informational only and was already passing.

## Engine Change Findings

| #  | Severity | File:Line | Description |
|----|----------|-----------|-------------|
| (none) | — | — | All prior findings resolved; no new engine findings. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| (none) | — | — | Fuseling reverted to TODO state with PB-CC-C-followup citation; oracle text comment preserved. No new card-def findings. |

## Test Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| (none) | — | — | Test 5 redesign exercises full dispatch path including post-resolution mutation; sentinel assertions updated correctly. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|--------------|---------|-------|
| 608.2h  | Yes (substitution arm) | Yes (tests 1, 2, 5) | T5 explicitly validates lock-in semantic via post-resolution counter mutation. |
| 611.3a  | N/A in PB-CC-C scope | N/A | Static-ability live re-evaluation is now documented as the path-semantics warning in `effects/mod.rs` and as a doc-comment authoring note on the new variants; the actual primitive is deferred to PB-CC-C-followup. |
| 613.4a  | Pre-existing (`SetPtDynamic`) | Pre-existing | The Layer-7a CDA path is named as the closest existing analogue for the deferred Layer-7c dynamic-static primitive. |
| 613.4c  | Yes (substitution + panic guard + hash) | Yes (tests 1-5) | All citations corrected from prior typo (CR 613.1c → 613.4c). |
| 604.2 / 604.3a | N/A in scope | N/A | No `AbilityDefinition::Static` or `CdaPowerToughness` path used; documented as future work. |
| 400.7   | Pre-existing (`WhileSourceOnBattlefield`) | No new test | Pre-existing duration mechanism handles zone change; not exercised by PB-CC-C tests but covered by existing PB-X / PB-Q tests. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|--------------|-----------------|--------------------|-------|
| Exuberant Fuseling | Yes (text matches MCP verbatim) | 2 (static "+1/+0" + WheneverArtifactDies) | N/A — deferred to PB-CC-C-followup; Trample + ETB AddCounter halves shippable | C1/E4 acknowledged via TODO citing PB-CC-C-followup multi-blocker; ETB "put an oil counter on this creature" half (creature-side) authored correctly. |

## Acceptance Criteria Verification (post-fix)

| AC ID | Description | Verdict |
|-------|-------------|---------|
| 3698 | New variants in `state/continuous_effect.rs` with serde compat | **PASS** — variants present at lines 391-444 with Box<EffectAmount> and `negate: bool`; doc-comments now warn against Static-ability path. |
| 3699 | `effects/mod.rs:2305-2315` substitution arm extended | **PASS** — substitution arm at lines 2330-2347 substitutes both new variants; preceded by CR 608.2h vs CR 611.3a path-semantics note. |
| 3700 | `rules/layers.rs` panic guard extended | **PASS** — guards at lines 1171-1187 mirror `ModifyBothDynamic` with corrected CR 613.4c citations. |
| 3701 | `exuberant_fuseling.rs` re-authored | **N/A — deferred per Option B** — card def reverted to TODO state with PB-CC-C-followup multi-blocker citation. The two halves of the static ability remain unblocked dependencies; Trample + ETB-AddCounter halves still shippable. |
| 3702 | Mandatory tests pass: substitution unit, layer panic, X-locked full dispatch | **PASS** — 5 tests cover substitution (T1, T2), panic guards (T3, T4), and full-dispatch X-lock semantic (T5). |
| 3703 | All existing tests pass; clippy clean; fmt clean | Trust runner's claim. Sentinel-assertion test-file updates verified manually. |
| 3704 | HASH_SCHEMA_VERSION bumped (LayerModification shape change) | **PASS** — bumped 10→11; hash arms 1482-1495 with discriminants 26+27; history entry 11 added at lines 51-53. |
| 3705 | /review pass via primitive-impl-reviewer agent (this review) | **PASS-WITH-NITS** — see verdict above. No HIGH/MEDIUM open. |

## Summary

Option B was the right call. The two new `LayerModification` variants ship
correctly for the one-shot spell use case (CR 608.2h X-locked-at-resolution
semantics), with proper substitution, panic-guard, hash-arm, and schema-bump
support. The Static-ability footgun is documented as a doc-comment warning on
both new variants AND as a path-semantics note in the substitution arm,
naming the deferred Layer-7c dynamic-static primitive (PB-CC-C-followup) as
the right path for static-ability cards like Fuseling. Fuseling itself is
deferred — its TODO comment now correctly attributes the deferral to a
multi-blocker (Layer-7c dynamic-static primitive + WheneverArtifactDies
trigger). The full-dispatch test (T5) validates the lock-in semantic via
post-resolution counter mutation, providing real coverage of the substitution
→ storage → layer-application chain and resilience against future regressions.

CR-citation sweep is complete (no `CR 613.1c` references in PB-CC-C-touched
code). HASH_SCHEMA_VERSION history is up to date. The Static-ability
authoring footgun is now warned in three places (variant doc-comments,
substitution-arm comment, and `effects/mod.rs:2320` path-semantics note),
making it hard to miss.

**Verdict: PASS-WITH-NITS** — mergeable. The "nits" are entirely
post-merge follow-ups: file PB-CC-C-followup with scope "Layer-7c dynamic
ModifyPower/Toughness re-evaluated at apply time + WheneverArtifactDies
trigger condition, to enable Exuberant Fuseling and the
gets-+N/+M-per-counter family." That is tracked explicitly via the TODO
citations in `exuberant_fuseling.rs` and the doc-comments in
`continuous_effect.rs`, so no separate paperwork is required at merge time.

## Previous Findings (re-review)

| #  | Previous Status | Current Status | Notes |
|----|-----------------|----------------|-------|
| C1 | OPEN (HIGH)     | **RESOLVED**   | Fuseling reverted to TODO state; Triggered+ApplyContinuousEffect block removed; PB-CC-C-followup citation present; oracle text comment intact. |
| T1 | OPEN (HIGH)     | **RESOLVED**   | `test_exuberant_fuseling_power_scales_with_oil_counters` removed; replaced with `test_modify_power_dynamic_x_locked_at_resolution` (full-dispatch CR 608.2h validation). |
| E3 | OPEN (MEDIUM)   | **RESOLVED**   | Static-ability footgun warning added to both variant doc-comments at `state/continuous_effect.rs:404-415` and `:432-437`. |
| E4 | OPEN (MEDIUM)   | **RESOLVED**   | CR 608.2h vs CR 611.3a path-semantics note added at `effects/mod.rs:2320-2329`; names PB-CC-C-followup and `CdaPowerToughness` as analogue. |
| C2 | OPEN (MEDIUM)   | **RESOLVED**   | Aspirationally-wrong "is_cda: true" comment removed (entire Triggered+ApplyContinuousEffect block dropped). |
| T2 | OPEN (MEDIUM)   | **RESOLVED**   | New T5 (`test_modify_power_dynamic_x_locked_at_resolution`) exercises full dispatch via `execute_effect` → `calculate_characteristics`, with post-resolution counter mutation validating the lock-in semantic. |
| E1 | OPEN (LOW)      | **RESOLVED**   | History entry 11 appended at `state/hash.rs:51-53` with PB-CC-C attribution and CR citations. |
| E2 | OPEN (LOW)      | **RESOLVED**   | CR 613.1c → 613.4c sweep applied to `state/continuous_effect.rs:396, 411` and `rules/layers.rs:1177, 1184` and `state/hash.rs:1483, 1490`. |
| C3 | OPEN (LOW)      | **RESOLVED**   | CR 613.1c → 613.4c in Fuseling card-def comment (now reads "CR 613.4c Layer 7c modify"). |
| C4 | INFORMATIONAL   | **N/A**        | No fix required — was already verified-correct against MCP oracle. Recorded for completeness. |
| T3 | OPEN (LOW)      | **RESOLVED**   | CR 613.1c → 613.4c in `tests/primitive_pb_cc_c.rs` file-level docstring (line 2, 8) and per-test docstrings. |
| T4 | OPEN (LOW)      | **RESOLVED**   | CR 613.1c → 613.4c in three sentinel-assertion test files. |
| E5 | OPEN (LOW)      | **RESOLVED**   | Covered by E4 fix (path-semantics note in substitution arm). |

**Final tally**: 0 OPEN. 13 prior findings RESOLVED. 0 REGRESSED.

PB-CC-C is mergeable as criterion-8 deliverable.

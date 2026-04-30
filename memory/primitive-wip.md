# Primitive WIP: PB-CC-C-followup — Layer-7c dynamic CDA with continuous re-evaluation (CR 611.3a)

batch: PB-CC-C-followup
title: Layer-7c dynamic CDA with continuous re-evaluation (CR 611.3a continuous-eval, complement to PB-CC-C's CR 608.2h substitution path)
cards_unblocked_estimated: exactly 2 (Vishgraz the Doomhive, Exuberant Fuseling) — both explicit Option-B deferrals from PB-CC-C and PB-CC-A. Per `feedback_pb_yield_calibration.md`: yield expectation is exactly 2; below 2 = hidden compound blocker → STOP and report.
cards_unblocked_confirmed_post_plan: 2 (Vishgraz, Fuseling) — confirmed via dispatch-chain walk; verdict PASS, no hidden blockers
started: 2026-04-29
phase: fix-complete
plan_file: memory/primitives/pb-plan-CC-C-followup.md
review_file: memory/primitives/pb-review-CC-C-followup.md
shape_chosen: A+D hybrid — new `AbilityDefinition::CdaModifyPowerToughness { power: Option<EffectAmount>, toughness: Option<EffectAmount> }` variant + live-eval branch in `apply_layer_modification` that calls existing `resolve_cda_amount`. Mirrors Layer-7a `CdaPowerToughness`/`SetPtDynamic` precedent at Layer 7c.
hash_version_pre: 12 (PB-CC-A)
hash_version_post: 13 (PB-CC-C-followup — new AbilityDefinition variant + relaxed dynamic-LayerModification storage invariant for is_cda=true)

## Task reference
- ESM task: scutemob-15
- Branch: feat/pb-cc-c-followup-layer-7c-dynamic-cda-with-continuous-re-eva
- Acceptance criteria: 3715 (engine primitive), 3716 (register_static routing), 3717 (post-mutation re-read), 3718 (Vishgraz re-author), 3719 (Fuseling re-author), 3720 (HASH bump + sentinel sweep), 3721 (mandatory tests a-e), 3722 (gates), 3723 (delegation chain + visible task list)

## Context (from PB-CC-C and PB-CC-A reviews)

PB-CC-C shipped `LayerModification::ModifyPower/ToughnessDynamic` with substitution-at-resolution (CR 608.2h: spell X-locked-at-resolution). Substitution arm at `effects/mod.rs:2330-2347`. Panic guard at `layers.rs:1171-1187` traps `debug_assert!(false, ...)` if a dynamic modification reaches `apply_modification` un-substituted.

PB-CC-A shipped `EffectAmount::PlayerCounterCount { player, counter }` with both spell-effect (`resolve_amount`) and CDA (`resolve_cda_amount`) paths. Sum semantic per CR 122.1 + Vishgraz 2023-02-04 ruling.

The MISSING path (this PB): static abilities require **CR 611.3a continuous re-evaluation**, NOT substitution. `register_static_continuous_effects` (replacement.rs:1753-1797) hardcodes `is_cda: false` for `AbilityDefinition::Static`. Routing a `ModifyBothDynamic` / `ModifyPowerDynamic` through this path stores the un-substituted variant, which then panics in `apply_modification`.

Cards explicitly blocked:
- **Vishgraz the Doomhive** — "Vishgraz gets +1/+1 for each poison counter your opponents have." Needs ModifyBothDynamic + PlayerCounterCount{EachOpponent, Poison}, is_cda=true, EffectFilter::Source. PB-CC-A deferred per Option B with thorough TODO citation at vishgraz_the_doomhive.rs:49-79.
- **Exuberant Fuseling** — "Exuberant Fuseling gets +1/+0 for each oil counter on it." Needs ModifyPowerDynamic + CounterCount{Source, Oil}. PB-CC-C deferred per Option B with TODO at exuberant_fuseling.rs.

## OUT OF SCOPE

- Fuseling's `WheneverCreatureOrArtifactDies` death trigger (separate primitive — needs trigger-condition variant)
- Per-target dynamic EffectAmount (Phyresis Outbreak — separate seed)
- Layer 7a CDA path (already covered by `CdaPowerToughness` / `SetPtDynamic`)
- Adding new EffectAmount variants (PB-CC-A already shipped PlayerCounterCount)

## STOP-AND-FLAG triggers (from task description)

1. Layer-7c continuous re-eval requires layers.rs architectural refactor beyond one primitive — escalate
2. Shape that unblocks Vishgraz blocks Fuseling or vice versa — split into two micro-PBs
3. Existing `CdaPowerToughness`/`resolve_cda_amount` (Layer 7a) already covers static-ability case via wiring rather than new variant — re-scope to wire-up, report to oversight before authoring tests
4. Per `feedback_verify_full_chain.md`: walk every dispatch site (DSL → register_static → store → layer-apply → hash → replay → simulator); don't stop at variant existence
5. Per `feedback_pb_yield_calibration.md`: yield expectation is exactly 2 (Vishgraz + Fuseling); below 2 = hidden compound blocker — STOP and report

## Reference docs

- `memory/primitives/pb-review-CC-C.md` (path-semantics note effects/mod.rs:2320-2329, doc-comment footgun warnings)
- `memory/primitives/pb-review-CC-A.md` (Vishgraz deferral analysis lines 95, 129-156)
- `memory/primitives/pb-retriage-CC.md` lines 180-184 (original PB-CC-C surface)
- `crates/engine/src/cards/defs/{vishgraz_the_doomhive,exuberant_fuseling}.rs` (TODO citations)
- CR 611.3a, 611.3b, 613.4a, 613.4c, 604.2, 604.3a

## Planner checklist

- [x] Step 1: CR research — quoted 611.3a, 611.3b, 613.4a/b/c, 604.2, 604.3, 604.3a, 608.2h, 122.1 verbatim with notes on the "X is locked / X is continuously evaluated" distinction
- [x] Step 2: Engine architecture walk — every dispatch site for static-ability dynamic Layer-7c re-eval:
  1. DSL (existing variants ModifyPower/ToughnessDynamic, ModifyBothDynamic — verified full set, doc-comments need relaxation)
  2. `register_static_continuous_effects` (replacement.rs:1753-1797) — current is_cda=false hardcode; new branch needed for CdaModifyPowerToughness variant
  3. ContinuousEffect storage (state/continuous_effect.rs) — invariant relaxed; storage shape unchanged
  4. Layer apply path (`apply_modification` in layers.rs around 1162-1187) — replace panic guards with live-eval calls to resolve_cda_amount
  5. Substitution arm (effects/mod.rs:2315-2347) — UNCHANGED for spell effects (only doc-comment refresh)
  6. CdaPowerToughness / SetPtDynamic / resolve_cda_amount (layers.rs Layer-7a precedent for live re-eval) — direct model for the new branch
  7. Hash schema (state/hash.rs) — bump 12→13, new arm for AbilityDefinition::CdaModifyPowerToughness
  8. Replay harness (testing/replay_harness.rs) — pass-through verification at impl time
  9. Simulator/legal_actions — pass-through verification (CDAs not activatable)
- [x] Step 3: Shape decision (planner-chosen, documented with rationale): Shape A+D hybrid — new AbilityDefinition variant + live-eval branch in apply_modification using existing resolve_cda_amount. Rejected Shape B (flag) for footgun risk; rejected pure Shape C (no register-side change) for losing type discipline; rejected pure Shape A (new LayerModification variants) for variant proliferation.
- [x] Step 4: Dispatch unification verdict — PASS (yield = exactly 2; all 5 stop-and-flag triggers walked and addressed)
- [x] Step 5: Hash strategy — bump 12→13, new arm for AbilityDefinition::CdaModifyPowerToughness, history entry 13, sentinel sweep across pbp/pbn/pbd/pbt/primitive_pb_cc_a (5 files; renames pbt's stale `..._is_12` to `..._is_13`)
- [x] Step 6: Test plan — 5 mandatory (a-e from criterion 3721) numbered with CR citations:
  - (a) PB-CC-C T5 substitution-path regression — must continue passing UNMODIFIED (no test code change)
  - (b) Static-ability path re-evaluates after counter mutation — `test_cda_modify_power_toughness_re_evaluates_after_counter_mutation`
  - (c) Vishgraz P/T scales correctly across 0/3/8 opponent poison counters with multiple opponents — `test_vishgraz_scales_with_opponent_poison_counters`
  - (d) Fuseling P scales correctly across 0/1/3 oil counters — `test_exuberant_fuseling_power_scales_with_oil_counters`
  - (e) Hash determinism — same dynamic ContinuousEffect produces same hash; HASH_SCHEMA_VERSION assertion (= 13)
- [x] Plan file written: `memory/primitives/pb-plan-CC-C-followup.md`

## Implementation checklist (runner fills in)

- [x] Engine change 1: shape implemented per plan — `AbilityDefinition::CdaModifyPowerToughness { power, toughness }` (disc 76) added to card_definition.rs; doc-comments relaxed on three dynamic LayerModification variants in continuous_effect.rs (commit c4bba5d3)
- [x] Engine change 2: register_static_continuous_effects routes CdaModifyPowerToughness through new branch with is_cda=true at Layer 7c in replacement.rs:~1857 (mirrors CdaPowerToughness Layer-7a pattern) (commit c4bba5d3)
- [x] Engine change 3: substitution path UNCHANGED for spell effects (effects/mod.rs comment updated to reflect completion; PB-CC-C T5 passes unmodified) (commit 8711fffe)
- [x] Engine change 4: panic guards in apply_modification replaced with resolve_cda_amount live-eval calls in layers.rs — 3 arms: ModifyBothDynamic, ModifyPowerDynamic, ModifyToughnessDynamic (commit c4bba5d3)
- [x] Engine change 5: hash arm for AbilityDefinition::CdaModifyPowerToughness (disc 76u8) + HASH_SCHEMA_VERSION bump 12→13 + history entry 13 in state/hash.rs (commit c4bba5d3 + 8711fffe)
- [x] Engine change 6: sentinel-assertion test files updated — 6 files total (pbp/pbn/pbd/pbt×2/primitive_pb_cc_a + effect_sacrifice_permanents_filter found by grep sweep); pbt tests renamed `..._is_12` → `..._is_13` (commit 8711fffe)
- [x] Card def 1: vishgraz_the_doomhive.rs re-authored — CdaModifyPowerToughness with PlayerCounterCount{EachOpponent, Poison} on both axes (commit 15ca37ce)
- [x] Card def 2: exuberant_fuseling.rs re-authored — CdaModifyPowerToughness with CounterCount{Source, Oil} on power axis only; keeps Trample + ETB AddCounter; WheneverCreatureOrArtifactDies death trigger remains TODO (commit 15ca37ce)
- [x] Tests written: 4 tests in new `tests/primitive_pb_cc_c_followup.rs` + 2 updated tests in `tests/primitive_pb_cc_c.rs` (T3/T4 converted from #[should_panic] to positive assertions) (commit c5ff6bca)
- [x] PB-CC-C T5 still passes unmodified (confirmed via cargo test --all)
- [x] All existing tests pass: cargo test --all — 2720 tests, 0 failures (4 new tests added)
- [x] cargo build --workspace clean (verified replay-viewer + TUI exhaustive matches)
- [x] cargo fmt --check clean (ran cargo fmt to fix auto-formatting, then --check passed)
- [x] cargo clippy -- -D warnings — 0 lints after fixing doc_overindented_list_items / doc_lazy_continuation in primitive_pb_cc_c_followup.rs (commit to follow)

## Reviewer checklist

- [x] CR rules independently verified (611.3a, 611.3b, 613.4a, 613.4c, 604.2, 604.3a, 608.2h) — fetched via mtg-rules MCP and matched against engine implementation
- [x] Card oracle text verified via MCP for Vishgraz and Fuseling — both card defs match MCP-reported oracle text exactly
- [x] Every dispatch site walked and confirmed correct — DSL (card_definition.rs:962), register (replacement.rs:1873), storage (continuous_effect.rs unchanged), apply (layers.rs:1170-1212), hash (hash.rs:6072), substitution arm UNCHANGED for spell-only path (effects/mod.rs:2332)
- [x] Hash arm + version bump + history entry verified — disc 76 unique, HASH_SCHEMA_VERSION=13, history entry 13 at hash.rs:61-66
- [x] Test (a) — PB-CC-C T5 confirmed unmodified at primitive_pb_cc_c.rs:336-419
- [x] Test (b) — post-mutation re-read genuinely re-evaluates (4 discriminating mutation steps: 0→2→5→0 oil counters)
- [x] Test (c) — multi-opponent sum semantic confirmed (Vishgraz: 5+2+1=8 NOT max=5 NOT count=3, with controller's own poison ignored)
- [x] Test (d) — counter scaling confirmed (Fuseling: 0→1→3→1 oil counters; toughness stays at 1; up- and down-scaling both verified)
- [x] Test (e) — hash determinism + HASH_SCHEMA_VERSION assertion present (= 13); 5 sub-assertions
- [x] No scope creep (death trigger out of scope; per-target dynamic EffectAmount out of scope)
- [x] Review file written: `memory/primitives/pb-review-CC-C-followup.md`
- [x] Verdict: PASS-WITH-NITS (1 MEDIUM E1 — both-Some toughness silent discard, footgun-prevention; 7 LOW — engine doc-comments + test polish). No HIGH open.

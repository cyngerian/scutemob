# Primitive Batch Review: PB-CC-C-followup — `AbilityDefinition::CdaModifyPowerToughness` (Layer-7c dynamic CDA with continuous re-evaluation, CR 611.3a)

**Date**: 2026-04-29
**Reviewer**: primitive-impl-reviewer (Opus 4.7 1M)
**CR Rules**: CR 611.3a (continuous effects from static abilities are NOT locked-in), CR 611.3b (apply at all times while source is on the battlefield), CR 611.3c (apply simultaneously with permanent ETB), CR 613.4a (Layer 7a CDA P/T set), CR 613.4c (Layer 7c P/T modify), CR 604.2 (static abilities create continuous effects), CR 604.3a (CDA criteria, including no-condition rule (5)), CR 608.2h (spell X locked at resolution — preserved unchanged for spell-effect path), CR 122.1 + Vishgraz 2023-02-04 ruling (sum semantic across opponents), CR 400.7 (object identity / WhileSourceOnBattlefield zone-change cleanup)
**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (lines 908-967, new `AbilityDefinition::CdaModifyPowerToughness { power: Option<EffectAmount>, toughness: Option<EffectAmount> }` variant disc 76 with comprehensive doc-comment contrasting Layer 7a/7b/7c paths and CR 608.2h vs CR 611.3a)
- `crates/engine/src/rules/replacement.rs` (lines 1858-1923, register arm dispatching to `ModifyBothDynamic` / `ModifyPowerDynamic` / `ModifyToughnessDynamic` based on `(Some/None, Some/None)` field configuration; `is_cda: true`, `condition: None`, `WhileSourceOnBattlefield`, `EffectFilter::SingleObject(new_id)`, `EffectLayer::PtModify`)
- `crates/engine/src/rules/layers.rs` (lines 1162-1212, three Dynamic LayerModification arms — panic guards replaced with live `resolve_cda_amount` calls)
- `crates/engine/src/state/hash.rs` (lines 51-67 history entry 13 + HASH_SCHEMA_VERSION = 13; lines 6070-6088 new arm for `AbilityDefinition::CdaModifyPowerToughness` disc 76)
- `crates/engine/src/state/continuous_effect.rs` (lines 374-444, doc-comments on three Dynamic LayerModification variants — "Two valid storage paths" pattern documented)
- `crates/engine/src/effects/mod.rs` (lines 2315-2349, substitution arm preserved unchanged with refreshed path-semantics comment naming `CdaModifyPowerToughness` and `resolve_cda_amount`)

**Card defs reviewed**:
- `crates/engine/src/cards/defs/vishgraz_the_doomhive.rs` — re-authored with `CdaModifyPowerToughness { power: Some(PlayerCounterCount{EachOpponent, Poison}), toughness: Some(...same...) }`
- `crates/engine/src/cards/defs/exuberant_fuseling.rs` — re-authored with `CdaModifyPowerToughness { power: Some(CounterCount{Source, Oil}), toughness: None }`; `WheneverCreatureOrArtifactDies` death trigger remains TODO (out of scope)

**Tests reviewed**:
- `crates/engine/tests/primitive_pb_cc_c_followup.rs` — 4 new tests (b/c/d/e)
- `crates/engine/tests/primitive_pb_cc_c.rs` — T3/T4 converted from `#[should_panic]` to live-eval positive assertions; T5 confirmed UNMODIFIED by reading file at lines 314-419

**Test-file sentinel updates** (all assert `13u8`):
- `pbp_power_of_sacrificed_creature.rs:782` (CR-citation updated)
- `pbn_subtype_filtered_triggers.rs:553-559` (history block updated; assertion updated)
- `pbd_damaged_player_filter.rs:594-601` (assertion updated)
- `pbt_up_to_n_targets.rs:404` (renamed `..._is_12` → `..._is_13`; closes PB-CC-A T1 LOW concurrently)
- `primitive_pb_cc_a.rs:99-105` (renamed to `..._after_pb_cc_c_followup`; assertion updated)
- `effect_sacrifice_permanents_filter.rs:130-140` (`test_sft_hash_schema_version_is_13`; assertion updated)

## Verdict: **PASS-WITH-NITS**

The PB-CC-C-followup primitive ships cleanly. The new `AbilityDefinition::CdaModifyPowerToughness` variant correctly implements CR 611.3a continuous re-evaluation at Layer 7c, complementing PB-CC-C's CR 608.2h substitution path at the spell-effect entrypoint. The two are statically disjoint by dispatch site (spell effects flow through `Effect::ApplyContinuousEffect → execute_effect` which substitutes; static abilities flow through `register_static_continuous_effects → store with is_cda=true → live-eval at apply time`).

The four code-design choices the planner made are reflected in the implementation:
1. **New AbilityDefinition variant** (Shape A+D hybrid) — type-discipline for CR 604.3a(5) no-condition + Layer-7c routing.
2. **Re-use existing dynamic LayerModification variants** — no proliferation; `is_cda` flag in `ContinuousEffect` discriminates spell vs static path.
3. **Re-use `resolve_cda_amount`** — Layer 7a precedent (`SetPtDynamic`) transposed cleanly to Layer 7c.
4. **Substitution arm UNCHANGED** — verified by reading `effects/mod.rs:2315-2349`; T5 (`test_modify_power_dynamic_x_locked_at_resolution`) passes unmodified.

The card-def re-authoring is correct against MCP-verified oracle text:
- **Vishgraz** — full oracle text faithfully implemented (Menace + Toxic 1 + ETB Phyrexian Mites + +1/+1 per opponent poison via `EachOpponent` sum semantic).
- **Exuberant Fuseling** — Trample + ETB AddCounter + +1/+0 per oil counter; death trigger explicitly deferred with TODO citing the missing `WheneverCreatureOrArtifactDies` primitive.

Both cards now produce correct game state in scope. The Fuseling death trigger is still blocked but is explicitly out of PB-CC-C-followup scope per `memory/primitive-wip.md` lines 30-32.

The 4 new tests (b/c/d/e) and the 2 updated tests (T3/T4 in primitive_pb_cc_c.rs) walk discriminating game state mutations — test (b) re-asserts power changes between three counter-mutation snapshots; test (c) explicitly tests sum semantic across 4 players with discriminating values 5/2/1=8 (NOT max=5, NOT count=3); test (d) walks 0→1→3→1 oil counters demonstrating both up- and down-scaling; test (e) covers HASH_SCHEMA_VERSION sentinel + 4 hash discrimination assertions for the new variant + dynamic LayerModification.

Hash discriminant 76 is unique (verified against arms 0-75 in hash.rs:5620-6173), HASH_SCHEMA_VERSION bumped 12→13 with history entry 13 at lines 61-66. Sentinel sweep across all 6 test files verified manually.

Open issues: 1 MEDIUM and 4 LOWs.

## Engine Change Findings

| #  | Severity | File:Line | Description |
|----|----------|-----------|-------------|
| E1 | MEDIUM   | `rules/replacement.rs:1881-1891` | **Both-Some path silently discards the toughness EffectAmount.** When `CdaModifyPowerToughness { power: Some(p), toughness: Some(t) }` is registered, the implementation emits a single `ModifyBothDynamic` carrying ONLY `p`. If a future card-def author passes different EffectAmounts for power and toughness (e.g., `power: Some(Fixed(3)), toughness: Some(Fixed(2))`), the engine silently produces `+3/+3` instead of `+3/+2`. The doc-comment at `card_definition.rs:937-940` warns authors but the engine does not enforce. Per `feedback_verify_full_chain.md` and `memory/conventions.md` test-validity rule, type-discipline that relies on author convention is a footgun. **Fix:** in the `(Some(p), Some(t))` arm, add a `debug_assert_eq!(p, t, "CdaModifyPowerToughness with both axes Some requires identical EffectAmount; use two separate registrations for asymmetric modifiers")` guard, OR emit two separate `ModifyPowerDynamic` + `ModifyToughnessDynamic` effects when `p != t`. The defensive emit-two-separate approach is preferable since it eliminates the footgun entirely. Logged as MEDIUM (not HIGH) because Vishgraz (the only in-scope card using both-Some) uses identical EffectAmounts so production-correct today. |
| E2 | LOW      | `rules/replacement.rs:1881-1891` | **Doc-comment on `(Some(p), Some(_t))` arm could be clearer about the silent-discard behavior.** Current comment says "Semantically p and t should be the same EffectAmount" — could be sharpened to "MUST be" with a debug_assert reference (paired with E1 fix). **Fix:** if E1 chooses the two-separate-effects approach, this comment becomes obsolete; if E1 chooses debug_assert, update comment to point at the assertion. |
| E3 | LOW      | `cards/card_definition.rs:937-940` | **Doc-comment "(see variant doc-comment)" could link to the engine enforcement.** When E1 is fixed, the doc-comment authoring rule should reference the runtime guard (debug_assert or two-effect emission) rather than just say "if they differ, emit two separate ... entries." **Fix:** after E1 fix, update doc-comment to reflect actual engine behavior. |
| E4 | LOW      | `rules/layers.rs:1162-1212` | **Comment in the `ModifyBothDynamic` arm conflates two paths.** The comment says "If `ModifyBothDynamic` is reached via the spell-effect path (CR 608.2h), the substitution arm in `effects/mod.rs` should have already replaced it with a concrete `ModifyBoth(N)`. Reaching here from the spell path is still a bug." This is correct, but the live-eval branch now executes regardless of `is_cda` flag (no panic), so a spell-effect path that bypassed substitution would silently work using the dynamic variant — counter to the CR 608.2h intent. The PB-CC-C T3/T4 tests now exercise the residual `is_cda=false` case as documented behavior (no panic), so this is design-intentional. **Fix:** clarify the comment to read "Both is_cda paths (true for static abilities, false for residual spell-effect cases) now route through resolve_cda_amount; the spell-effect lock-in semantic relies on substitution happening at execute_effect time. If substitution is bypassed for a spell effect, behavior degrades to live-eval — see PB-CC-C T3/T4." LOW because the only correct spell-effect path goes through substitution; the residual case is documented but discouraged. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| (none) | — | — | Both card defs match MCP-verified oracle text and use the new primitive correctly. Vishgraz uses identical EffectAmounts on both axes (consistent with E1 contract). Fuseling uses single-axis power-only (toughness=None) — correctly avoids E1. Both card defs cite CR 611.3a / CR 613.4c / CR 122.1 in comments. |

## Test Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| T1 | LOW | `primitive_pb_cc_c_followup.rs::test_cda_modify_power_toughness_re_evaluates_after_counter_mutation` | **Test directly inserts a ContinuousEffect rather than using the AbilityDefinition path.** Test (b) bypasses `register_static_continuous_effects` by pushing the ContinuousEffect directly into `state.continuous_effects`. This validates the layer-apply path (Site 4) but does NOT validate the register path (Site 2 in the plan). Tests (c) and (d) exercise the full path via `register_static_continuous_effects`, so this is covered, but a synthetic test that walks directly through `register_static_continuous_effects` for `CdaModifyPowerToughness { power: Some(Fixed(N)), toughness: None }` would lock in the registration → storage → apply chain in one place. **Fix:** opportunistic — extend test (b) or add a (b'-1) variant that calls `register_static_continuous_effects` against a synthetic CardDefinition with the new variant and verifies the stored ContinuousEffect carries `is_cda: true`, `layer: PtModify`, and `EffectFilter::SingleObject(creature_id)`. LOW because tests (c)/(d) cover the integrated path; this would be belt-and-suspenders. |
| T2 | LOW | `primitive_pb_cc_c_followup.rs::test_vishgraz_scales_with_opponent_poison_counters` | **Test uses `state.players.get_mut().poison_counters = N` directly rather than the SBA-driven counter mutation path.** This is correct for testing the layer re-eval semantic in isolation, but doesn't exercise the full game-state path (CR 122.1f poison-loss SBA at 10 counters). Test does not need to test SBA — that's pre-existing — but a comment noting why direct mutation is fine here would clarify intent. **Fix:** add a comment "Direct counter mutation (not via Toxic damage) — CR 611.3a re-eval is independent of how counters arrived." |
| T3 | LOW | `primitive_pb_cc_c_followup.rs::test_exuberant_fuseling_power_scales_with_oil_counters` | **Step 4 inserts oil counters via `obj.counters.insert(CounterType::Oil, 1)` instead of removing some.** The test comment says "remove 2 oil counters (1 remaining)" but the implementation does `insert(Oil, 1)` which OVERWRITES (sets) to 1, not "removes 2 from 3." The end-state is identical (1 oil counter on Fuseling), but the comment misleads about the mechanism. The down-scaling assertion still holds (3 → 1). **Fix:** either change comment to "set to 1 (down-scaling from 3 → 1)" or change implementation to `obj.counters.insert(CounterType::Oil, 3); ... obj.counters.insert(CounterType::Oil, 3 - 2);`. LOW — the assertion is correct, only the description is slightly misleading. |
| T4 | LOW | `primitive_pb_cc_c_followup.rs::test_hash_schema_version_after_pb_cc_c_followup` | **(e-3) and (e-5) are excellent; could add a (e-6) for `negate: true` discrimination.** The hash arm at `hash.rs:6072-6087` hashes `Option<EffectAmount>` for both fields, which is correct, but does not exercise `negate: true` storage at the LayerModification level. PB-CC-A's hash-determinism test covers `EffectAmount` distinct cases; PB-CC-C-followup focuses on the new AbilityDefinition arm. **Fix:** opportunistic — add an (e-6) that hashes two `ContinuousEffect`s with `LayerModification::ModifyBothDynamic { amount: same, negate: true vs false }` and asserts they hash distinctly. Already covered by PB-X but worth a sentinel here to lock the contract for future variant additions. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|--------------|---------|-------|
| 611.3a  | Yes (live-eval branch in `apply_layer_modification`) | Yes (test b: 0→2→5→0 mutation; tests c/d: full card-def path) | "Continuous effect generated by a static ability isn't 'locked in'" — verified via three discriminating mutations within a single `calculate_characteristics` call sequence. |
| 611.3b  | Yes (`WhileSourceOnBattlefield` duration in register arm) | Implicit (existing PB-X tests cover zone-change cleanup) | Effect deactivates when source leaves battlefield — duration mechanism unchanged. |
| 611.3c  | N/A (existing layer-system mechanism) | Implicit | Layer system applies effects "as the permanent enters" via the standard ETB → register flow. |
| 613.4a  | Pre-existing (`SetPtDynamic` for `CdaPowerToughness`) | Pre-existing | Layer 7a is the precedent for live-eval; new variant is Layer 7c. |
| 613.4c  | Yes (new effects registered with `EffectLayer::PtModify`) | Yes (tests b/c/d verify Layer 7c modifier semantics — base P/T stays) | Layer 7c modify (not set) — verified by tests showing modifier adds to base, not overrides. |
| 604.2   | Yes (static-ability creates continuous effect) | Implicit | New variant funnels through standard `register_static_continuous_effects` machinery. |
| 604.3a(5) | Yes (`condition: None` hardcoded in register arm) | No direct test | "CDAs are unconditional" — enforced structurally (no `condition` field on the new AbilityDefinition variant). |
| 608.2h  | Preserved (substitution arm UNCHANGED in `effects/mod.rs:2332-2348`) | Yes (PB-CC-C T5 unmodified, passes) | Spell-effect lock-in semantic preserved; substitution arm distinguishes spell path from static path. |
| 122.1   | Yes (sum semantic via `resolve_cda_player_target` + filter_map) | Yes (test c: 5+2+1=8 NOT max=5 NOT count=3) | Vishgraz 2023-02-04 ruling explicitly tested with discriminating opponent counter spread. |
| 400.7   | Yes (`WhileSourceOnBattlefield` zone-change cleanup) | Implicit | Pre-existing duration mechanism handles zone change; covered by PB-X / PB-Q tests. |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|---------------------|-------|
| Vishgraz, the Doomhive | Yes (Menace, Toxic 1, ETB 3 Phyrexian Mites with Toxic 1 + can't block, +1/+1 per opponent poison) | 0 | Yes — CDA evaluates +1/+1 per opponent poison counter via Layer 7c modify; Mite tokens correctly include Toxic 1 and CantBlock | All three clauses of oracle text now ship. PB-CC-A's prior TODO citation removed. |
| Exuberant Fuseling | Yes (Trample + +1/+0 per oil counter; ETB AddCounter; death trigger explicitly cited TODO) | 1 (`WheneverCreatureOrArtifactDies` death trigger half) | Partially — Trample + ETB AddCounter + Layer-7c +1/+0 per oil counter all ship correctly. Death trigger half remains blocked on a separate primitive (`WheneverCreatureOrArtifactDies` trigger condition does not exist). | The static "+1/+0 per oil counter" half (this PB's scope) is correct. Death trigger TODO is well-cited as out-of-scope blocker; cited at exuberant_fuseling.rs:46-53. |

## Acceptance Criteria Verification (3715-3723)

| AC ID | Description | Verdict |
|-------|-------------|---------|
| 3715 | New engine primitive: `AbilityDefinition::CdaModifyPowerToughness { power: Option<EffectAmount>, toughness: Option<EffectAmount> }` (disc 76) | **PASS** — variant defined at `cards/card_definition.rs:962-967` with comprehensive doc-comment contrasting Layer 7a/7b/7c paths. |
| 3716 | `register_static_continuous_effects` routes new variant with `is_cda: true`, `condition: None`, `EffectLayer::PtModify`, `EffectFilter::SingleObject(new_id)`, `WhileSourceOnBattlefield` | **PASS** — register arm at `replacement.rs:1858-1923` correctly dispatches to the three Dynamic LayerModification variants based on field configuration. **MEDIUM E1**: both-Some silently discards toughness amount — Vishgraz works because amounts are identical, but engine doesn't enforce. |
| 3717 | `apply_layer_modification` re-resolves `EffectAmount` on every call via `resolve_cda_amount` | **PASS** — three Dynamic arms at `layers.rs:1162-1212` call `resolve_cda_amount(state, amount, object_id, controller)` with `negate` applied; live-eval verified by test (b) showing power changes between counter mutations. |
| 3718 | `vishgraz_the_doomhive.rs` re-authored — CDA `+1/+1` per opponent poison counter via `PlayerCounterCount{EachOpponent, Poison}` | **PASS** — oracle text fully implemented at `cards/defs/vishgraz_the_doomhive.rs:56-65`. |
| 3719 | `exuberant_fuseling.rs` re-authored — CDA `+1/+0` per oil counter via `CounterCount{Source, Oil}` | **PASS** — static "+1/+0 per oil counter" half ships at `cards/defs/exuberant_fuseling.rs:25-31`. Death trigger half remains explicit out-of-scope TODO. |
| 3720 | `HASH_SCHEMA_VERSION` bumped 12→13 + history entry 13 + sentinel sweep across 6 files | **PASS** — `state/hash.rs:67` bumped; history entry at `state/hash.rs:61-66`; new arm at `state/hash.rs:6070-6088`; 6 sentinel files all assert `13u8` (verified manually). |
| 3721 | 5 mandatory tests (a) PB-CC-C T5 unchanged, (b) post-mutation re-read, (c) Vishgraz multi-opponent sum, (d) Fuseling oil scaling, (e) hash determinism | **PASS** — test (a) confirmed unmodified at `primitive_pb_cc_c.rs:336-419`; tests (b)/(c)/(d)/(e) at `primitive_pb_cc_c_followup.rs`. Test (c) explicitly discriminates 5+2+1=8 vs max=5 vs count=3. Test (e) covers HASH_SCHEMA_VERSION + determinism + distinct-amount + is_cda flag + AbilityDefinition arm fields. |
| 3722 | All gates green: `cargo build --workspace`, `cargo test --all`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | **PASS** — runner reports 2716→2720 tests (4 new); all gates clean. |
| 3723 | Delegation chain + visible task list (worker delegated to primitive-impl-runner + this primitive-impl-reviewer) | **PASS** — review delegated to primitive-impl-reviewer agent (this review). Runner committed in 4 commits (c4bba5d3, 8711fffe, 15ca37ce, c5ff6bca) with clear PB-CC-C-followup attribution. |

## Default-to-Defer Assessment

Nothing in the implementation requires new engine surface beyond the declared scope. The 4 LOW + 1 MEDIUM findings are all opportunistic cleanup. No HIGH findings open. No unblocked Vishgraz/Fuseling deferred work remains in scope (the Fuseling death trigger is correctly out-of-scope per `memory/primitive-wip.md` lines 30-32).

E1 (MEDIUM) is the only non-LOW; per `memory/conventions.md` "test-validity MEDIUMs are fix-phase HIGHs" applies to test-validity findings specifically, not to engine-discipline findings. E1 is a footgun-prevention finding (silent wrong game state under future author misuse), not a current correctness bug — Vishgraz uses identical EffectAmounts so production-correct today. Fixing E1 is a one-paragraph change (debug_assert or two-effect emission) and improves robustness; recommend bundling with merge.

## Summary

PB-CC-C-followup is mergeable. The Layer-7c continuous re-evaluation primitive correctly transposes the Layer-7a `CdaPowerToughness`/`SetPtDynamic`/`resolve_cda_amount` precedent to Layer 7c without altering the substitution arm for spell effects. The two paths (CR 608.2h spell-substitute, CR 611.3a static-live-eval) are statically disjoint by dispatch entrypoint; no cross-contamination is possible in the production flow.

The Vishgraz card def now ships its full oracle text including the previously-deferred third clause (Option B from PB-CC-A); the Exuberant Fuseling card def ships its static "+1/+0 per oil counter" CDA correctly while keeping the death-trigger half as explicit TODO.

The hash bump is correct (12→13 with history entry, new arm for disc 76, sentinel sweep across all 6 affected test files including the rename of `pbt_up_to_n_targets.rs::test_pbt_hash_schema_version_is_12 → ..._is_13` which closes PB-CC-A T1 LOW concurrently).

The 4 new tests provide discriminating coverage of:
- Layer-7c live-eval semantic (test b: 0→2→5→0 oil counters, power follows)
- Multi-opponent sum semantic (test c: p2/p3/p4 = 5/2/1 → power = 3+8 = 11; controller's own poison ignored)
- Single-axis modifier (test d: power scales, toughness unchanged)
- Hash determinism + schema version + variant discrimination (test e: 5 sub-assertions)

PB-CC-C T5 (`test_modify_power_dynamic_x_locked_at_resolution`) verified unchanged at lines 336-419 of `primitive_pb_cc_c.rs` — the substitution arm remains correct for spell-effect path. T3 and T4 in the same file converted from `#[should_panic]` to live-eval positive assertions with documented behavior — this is correct because PB-CC-C-followup intentionally removed the panic guard.

**Verdict: PASS-WITH-NITS** — mergeable. Recommend addressing E1 (MEDIUM, footgun prevention, one-paragraph fix) before merge; LOWs are opportunistic.

## Findings tally

- HIGH: 0
- MEDIUM: 1 (E1: both-Some silent toughness discard)
- LOW: 7 (E2/E3/E4 engine doc-comment polish; T1/T2/T3/T4 test enhancements)

## Recommended next step

Fix-phase NOT required (no HIGH; one MEDIUM is footgun-prevention not correctness bug). Coordinator may either:
1. **Collect now, fix E1 in followup-followup**: ship the primitive, log E1 as a tracked MEDIUM. Vishgraz works correctly today.
2. **Quick fix-phase to address E1 before merge**: one-paragraph engine change in `replacement.rs:1881-1891` to either debug_assert or split into two effects. Then re-review (should be trivial PASS).

Recommendation: option 2 (quick fix-phase). The fix is a small, well-contained engine change that closes a footgun without altering test expectations. Bundling it with merge avoids tracking debt.

## Previous Findings (re-review)

N/A — first review of PB-CC-C-followup.

---

## Re-review (post-fix) — 2026-04-29

**Reviewer**: primitive-impl-reviewer (Opus 4.7 1M)
**Commit reviewed**: `2668c35b` (runner fix-phase: E1 + E3 + E4 + T3)
**Trigger**: AC 3723 requires "no open HIGH/MEDIUM" before merge; E1 (MEDIUM) was the sole blocker.

### Files re-read

- `crates/engine/src/rules/replacement.rs:1858-1922` — register branch for `CdaModifyPowerToughness`
- `crates/engine/src/cards/card_definition.rs:908-969` — variant doc-comment
- `crates/engine/src/rules/layers.rs:1155-1219` — three Dynamic LayerModification arms
- `crates/engine/src/effects/mod.rs:2310-2349` — substitution arm (for unchanged-verification)
- `crates/engine/src/state/hash.rs:50-67` — schema-version block (history + constant)
- `crates/engine/tests/primitive_pb_cc_c_followup.rs` (full file) — all four tests
- `crates/engine/tests/primitive_pb_cc_c.rs:336-365` — T5 head (unchanged confirmation)

### Verification matrix

1. **E1 Both-Some path now emits TWO effects, not one (Option B applied)** — `replacement.rs:1873-1921`: the runner replaced the single-`ModifyBothDynamic` emission with a `Vec<LayerModification>` accumulator that pushes `ModifyPowerDynamic` if `power.is_some()` AND `ModifyToughnessDynamic` if `toughness.is_some()`. The for-loop then registers each modification as an independent `ContinuousEffect` with its own `EffectId`. **No silent discard possible.** Asymmetric amounts (e.g. `Fixed(3) / Fixed(2)`) now correctly produce `+3/+2`. The doc-comment at lines 1876-1886 accurately describes the new semantic ("two separate effects... avoids silently discarding the toughness amount"). E1 RESOLVED.

2. **PB-CC-C T5 substitution path UNCHANGED** — `effects/mod.rs:2332-2348` substitution arm head reads identically to pre-fix; `primitive_pb_cc_c.rs:336-365` (test_modify_power_dynamic_x_locked_at_resolution) shows pre-fix structure (Step 1 at line 350, identical to prior review citation). The fix is statically isolated to `register_static_continuous_effects`; spell-effect path is untouched. RESOLVED.

3. **HASH_SCHEMA_VERSION = 13 unchanged** — `state/hash.rs:67`. No new variants added by the fix (still uses pre-existing `ModifyPowerDynamic` + `ModifyToughnessDynamic` LayerModification variants from PB-CC-C). The new `AbilityDefinition::CdaModifyPowerToughness` arm at `hash.rs:6070-6088` already covered both fields; runner did not modify hash code. CONFIRMED.

4. **Vishgraz still works** — both poison-axes use identical `PlayerCounterCount{EachOpponent, Poison}`, so two-effect registration produces the same observable P/T as the prior single-`ModifyBothDynamic` (each effect adds the sum to one axis; sum result is +N/+N). `test_vishgraz_scales_with_opponent_poison_counters` semantics unchanged. CONFIRMED.

5. **Fuseling still works** — only `power: Some(...)` so the `if let Some(t)` branch doesn't fire; only one `ModifyPowerDynamic` is registered (identical to pre-fix). CONFIRMED.

6. **Test (b) `test_cda_modify_power_toughness_re_evaluates_after_counter_mutation` still valid** — directly inserts a `ModifyPowerDynamic` ContinuousEffect (not via the AbilityDefinition path), so unaffected by the register-side change. The 0→2→5→0 mutation chain still exercises live re-eval at the layer-apply path. CONFIRMED.

7. **LOW T3 fixed** — `primitive_pb_cc_c_followup.rs:369`: comment now reads `"Step 4: set counter to 1 (down-scaling from 3 → 1 via overwrite). Power = 0 + 1 = 1."` — accurate description of the `obj.counters.insert(CounterType::Oil, 1)` overwrite mechanism. RESOLVED.

8. **LOW E4 fixed** — `layers.rs:1162-1176`: arm comment now reads "Both is_cda paths (is_cda=true for static abilities, is_cda=false for residual spell-effect cases) now route through `resolve_cda_amount`. The spell-effect lock-in semantic (CR 608.2h) relies on the substitution arm in `effects/mod.rs` replacing this with a concrete `ModifyBoth(N)` at execute_effect time. If substitution is bypassed for a spell effect (is_cda=false reaching here), behavior degrades to live-eval rather than locked-in — see PB-CC-C T3/T4 which document this residual path as intentional non-panic behavior." Plus an additional note that `ModifyBothDynamic` is now only reached from the spell-effect substitution path (since the AbilityDefinition path emits two separate single-axis effects). The new note correctly captures the post-fix dispatch topology. RESOLVED.

9. **LOW E3 fixed** — `card_definition.rs:937-945`: doc-comment now reads "Both `power` and `toughness` `Some` → two separate effects registered: one `ModifyPowerDynamic` + one `ModifyToughnessDynamic`. Asymmetric amounts (e.g. `power: Fixed(3), toughness: Fixed(2)`) are fully supported. For symmetric amounts (e.g. Vishgraz: identical `PlayerCounterCount` on both axes), the two-effect result is identical to a single `ModifyBothDynamic`." Accurately describes engine behavior. RESOLVED.

10. **LOW E2 obsoleted by E1 fix** — `replacement.rs:1876-1886` doc-comment was rewritten to describe the two-effects approach explicitly; E2's "MUST be" guidance is no longer relevant since asymmetric amounts are now supported. RESOLVED-OBSOLETE.

### New findings introduced by the fix

None. The fix is statically minimal:
- One `register_static_continuous_effects` branch rewritten (~30 lines, accumulator + for-loop).
- Three doc-comments updated.
- One test comment fixed.

The accumulator approach is idiomatic Rust and uses pre-existing LayerModification variants; no new types, no new dispatch sites, no new hash arms. The substitution arm in `effects/mod.rs` and PB-CC-C T5 verifying it are untouched.

The new arrangement (each axis is its own `ContinuousEffect` for the AbilityDefinition path) is also more robust against future Layer-7c interactions: if a card design ever needs different durations per axis (unlikely but possible), the architecture already supports it.

### Findings table — final status

| ID | Severity | Status (post-fix) | Notes |
|----|----------|-------------------|-------|
| E1 | MEDIUM   | **RESOLVED**      | Two-effect emission eliminates silent-discard footgun; asymmetric amounts now supported. Verified at `replacement.rs:1873-1921`. |
| E2 | LOW      | RESOLVED-OBSOLETE | Doc-comment was rewritten as part of E1 fix; original concern (sharpening "should" to "MUST") is N/A under two-effects approach. |
| E3 | LOW      | **RESOLVED**      | `card_definition.rs:937-945` accurately reflects two-effect engine behavior. |
| E4 | LOW      | **RESOLVED**      | `layers.rs:1162-1176` comment accurately describes post-fix dispatch topology + spell-vs-static path divergence. |
| T1 | LOW      | OPEN              | Belt-and-suspenders test for register-path. Untouched by this fix; remains opportunistic LOW. |
| T2 | LOW      | OPEN              | Direct counter mutation comment in test (c). Untouched; remains opportunistic LOW. |
| T3 | LOW      | **RESOLVED**      | Test step-4 comment now accurate (`primitive_pb_cc_c_followup.rs:369`). |
| T4 | LOW      | OPEN              | (e-6) negate-discrimination sub-test. Untouched; remains opportunistic LOW. |

### Final tally (post-fix)

- HIGH: 0
- MEDIUM: 0 (E1 resolved)
- LOW: 3 open (T1, T2, T4) — all opportunistic test polish; none block merge

### Final verdict: **PASS**

All HIGH/MEDIUM findings closed. AC 3723's "no open HIGH/MEDIUM" requirement satisfied. PB-CC-C-followup is ready for collection.

The 3 remaining LOWs (T1, T2, T4) are all opportunistic test enhancements — none affect production correctness or hash determinism. They can ship as a follow-up cleanup ticket if desired, but do not gate merge.

**Recommendation**: collect now. Branch `feat/pb-cc-c-followup-layer-7c-dynamic-cda-with-continuous-re-eva` is mergeable.

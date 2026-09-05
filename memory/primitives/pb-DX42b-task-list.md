# PB-DX42b — task list (`scutemob-233`)

Derived from the four acceptance criteria (7386-7389) and adjudication §5.2 steps 1-3.
Status legend: `[ ]` pending · `[~]` in progress · `[x]` done.

## Stage 0 — measure before anything moves
- [x] 0.1 Pre-edit full-workspace baseline to a file; reconcile against PB-DX54's 5,231 / 0 / 5 pin
- [x] 0.2 Wire prediction IN WRITING for BOTH fingerprints, committed before any production line
- [x] 0.3 Stage-0 `CLOSURE_MUST_NOT_CONTAIN` probe: verify the counterfactual by execution
- [x] 0.4 Census by `all_cards()` (SR-36), PRINTED by a test: layer-querying
      `ContinuousEffectDef.condition` population (re-measure the memo's 2)
- [x] 0.5 Re-derive the seven Archangel pairs from the Artifact-moving supply — floor AND ceiling
- [x] 0.6 Coverage flip prediction NAMED before regeneration (0 expected — state the reason)
- [ ] 0.7 Merge-base bench worktree prepared; same-code band measured FIRST across three base runs

## Stage 1 — engine (adjudication §5.2)
- [x] 1.1 Split `is_effect_active` into `is_effect_duration_active` + `is_effect_condition_satisfied`
- [x] 1.2 `CharacteristicEvalContext` keyed by `EffectId` replacing the ambient depth counter
- [x] 1.3 `calculate_characteristics_through(state, id, through_layer, ctx)` with the ACTIVITY
      SWEEP bounded by the same `through_layer` (§3.2(iii)'s load-bearing precondition)
- [x] 1.4 `TargetFilter::required_characteristic_layer` per filter INSTANCE, all 33
      declared fields exhaustively destructured (`has_name`, `min/max_cmc` on Layers 1/3
      included); sibling `Condition::required_characteristic_layer`, exhaustive, no wildcard
- [x] 1.5 `debug_assert!` when a condition's required layer >= its effect's layer
- [x] 1.6 Decided + stated: `LayerWalkGuard` / `LAYER_WALK_DEPTH` / `in_layer_walk` RETIRED
      entirely, together with the `process_command` balance assert (`OOS-DX19-4` closed by
      construction — a `&mut CharacteristicEvalContext` cannot outlive its call, so the
      hazard the assert guarded is unrepresentable, not merely unlikely)
- [x] 1.7 The surviving `eval.in_flight` cycle-breaker shipped LABELLED as an undocumented
      deviation in `is_effect_condition_satisfied`'s doc comment, citing CR 613.8a(a)/613.8b
      per adjudication §3.2(ii). **No wrong-way-round pin test was added** — that is item 2.5
      below, out of this task's scope (assigned to a second agent); the label exists in the
      engine doc regardless of whether a probe exists yet.

## Stage 2 — tests / probes
- [x] 2.1 INVERTED `deviation_animated_nexus_does_not_count_toward_metalcraft` → renamed
      `nexus_animated_by_a_continuous_effect_now_counts_toward_metalcraft` (never deleted)
- [ ] 2.2 UNDER-count direction on a real `LocalGame`/`HumanChoice` drive — **OUT OF SCOPE for
      this task** (assigned to a second agent per the dispatch brief)
- [ ] 2.3 OVER-count direction on the same channel — **OUT OF SCOPE**, second agent
- [ ] 2.4 `thaumatic_compass` DFC case as its own test — **OUT OF SCOPE**, second agent
- [ ] 2.5 "two distinct conditional effects nest without mutual suppression" probe —
      **OUT OF SCOPE**, second agent
- [ ] 2.6 `debug_assert` fires on a synthetic same-layer case — test — **OUT OF SCOPE**,
      second agent
- [x] 2.7 `no_condition_evaluator_resolves_characteristics_directly` RE-KEYED with its reason
      written into the test's own doc comment: now scans all FOUR bodies
      (`check_condition`/`check_condition_ctx`/`check_static_condition`/
      `check_static_condition_ctx`) with a non-vacuity floor on body SIZE per function, so the
      two three-line `pub fn` wrappers cannot make the gate pass vacuously. Also repaired in
      place (broken by this task's refactor, not listed as a stage-2 deliverable but required
      to keep the suite green): `pb_dx39_source_view_gates.rs`'s `r4_is_effect_active_reads_are_
      live_only` (re-keyed onto the two split functions, one vocabulary whitelist each) and
      `r1c_effect_source_is_named_only_where_the_roster_says` (roster entry
      `("is_effect_active", 2, ..)` split into two 1-occurrence entries). Also reworded
      `the_deviation_is_scoped_to_the_layer_walk_only` → renamed
      `characteristics_for_condition_gives_full_resolution_outside_any_walk` (the ambient flag
      it read no longer exists; states the new boundary instead).
- [ ] 2.8 Rider `OOS-ADJ-2` (`pb_dx42a_continuous_condition_roster`) — **OUT OF SCOPE**, second
      agent (explicitly named as out of scope in the dispatch brief's item 9)

## Stage 3 — gates and measurement against the FINAL tree
- [ ] 3.1 Post-run full workspace; delta by test NAME via byte-exact set difference (`OOS-DX20b-5`)
- [ ] 3.2 Count-vs-name reconciliation + duplicate-name scan (`OOS-DX35-8`); leavers/renames disclosed
- [ ] 3.3 `clippy --workspace --all-targets -D warnings`, `cargo fmt --check`,
      `tools/check-defs-fmt.sh`, `cargo build --workspace` — all AGAINST THE FINAL TREE
- [ ] 3.4 Both wire gates EXECUTED; unmoved verdict published with the counterfactual
- [ ] 3.5 Coverage regenerated, self-dating churn reverted
- [ ] 3.6 `npm run build` N/A stated with its reason
- [ ] 3.7 Benches: merge-base A/B on a quiet machine, published honestly; `size_of` for any
      struct that grows
- [ ] 3.8 Fuzz A/B on the PB-DX32 gate config; movement attributed by ablation if any
- [ ] 3.9 Revert matrix executed by the coordinator; UNDISCRIMINATED rows disclosed in the test

## Stage 4 — bookkeeping
- [ ] 4.1 `/review` run; every finding taken or declined with a reason
- [ ] 4.2 Registry rows closed (pipes escaped); adjudication §6 rows updated
- [ ] 4.3 `OOS-DX42b-N` seeds filed — grep the registry FIRST (dispatch hygiene 5)
- [ ] 4.4 v4 memo §4 row 18 struck; banner repointed to rank 19 with the
      "ranks 14-18 shipped, NO further dispatch authorised" note
- [ ] 4.5 CLAUDE.md Current State + narrative; `memory/workstream-state.md` handoff AND its W6 row
- [ ] 4.6 Headline surfaces re-checked against the registry AFTER the `/review` fix cycle (hygiene 8)
- [ ] 4.7 `/tmp` bench dirs deleted; execution notes written

# Primitive WIP — PB-OS4b (face-aware ability gathering)

<!-- last_updated: 2026-07-19 -->

**PB**: PB-OS4b — face-aware ability gathering: transformed permanents must register/queue/scan
their **back-face** non-keyword abilities (triggered/activated/static/mana/ETB), and must NOT use
the **front** face's, when `obj.is_transformed` is true.
**Seed**: OOS-OS4-2 (`memory/primitives/ef-batch-plan-2026-07-17.md` §13; evidence in
`memory/primitives/pb-review-OS4.md` E1/C1).
**Task**: `scutemob-134`, branch `feat/pb-os4b-face-aware-ability-gathering-transformed-permanents-`.
**Phase**: fix (review verdict needs-fix: E1 MEDIUM residual sweeps + E2 LOW deregister asymmetry)

## Scope

- **Mandatory (correctness, NO wire bump)**: make ability gathering face-aware at *every* site that
  gathers a battlefield permanent's effective abilities. Sweep for the full roster — the reviewer
  named three (`register_static_continuous_effects` @ replacement.rs:2057, `queue_carddef_etb_triggers`
  @ replacement.rs:1415, upkeep trigger scan @ turn_actions.rs:278) but there are more candidate
  `def.abilities` read sites (activated-ability dispatch in `abilities.rs`, triggered scans, mana
  abilities in `mana.rs:675`, SBA in `sba.rs`, etc — 164 `.abilities` reads engine-wide). Chain-verify
  each: does it read the abilities of a *permanent as it currently is on the battlefield*? If yes and
  it can be transformed, it must branch on `is_transformed` → `back_face.abilities`. Casting/alt-cost
  sites that read the printed front face (morph, suspend, miracle, plot, partner detection) do NOT.
- **Probe by execution**: transform `docent_of_perfection` + `bloodline_keeper` (PB-EF5 in-place
  `TransformSelf` Completes) in tests; assert back-face abilities fire + front statics stop. Keep
  Complete with pinning tests, or demote honestly. Update `fable_of_the_mirror_breaker` Reflection
  blocker status.
- **Optional extension (planner decides; SEPARATE commit if taken; IS a wire bump PROTOCOL 19→20 /
  HASH 56→57)**: re-add `Effect::ReturnSourceToBattlefieldTransformed` + re-author
  `edgar_charmed_groom` Complete with a full transform-lifecycle test. Else file as follow-up. The
  mandatory commit must remain wire-neutral.

## Architecture note (for planner)

`CardDefinition.abilities` (front) and `CardDefinition.back_face: Option<CardFace>` where
`CardFace.abilities` is the back list. Today the ONLY back-face ability reader is `layers.rs:116`,
and it copies **keywords only** into `chars.keywords`. Candidate clean fix: a helper
`effective_abilities(def, is_transformed) -> &[AbilityDefinition]` (returns `&back.abilities` when
`is_transformed && back_face.is_some()`, else `&def.abilities`) routed through each face-aware
gathering site. Planner must decide helper-vs-inline and the precise site roster.

## Planner findings (2026-07-19)

Two ability channels, BOTH front-only for transformed permanents:
- **Channel A (runtime `characteristics.{triggered,activated,mana}_abilities`)** — lowered from
  front `def.abilities` at construction (`enrich_spec_from_def`), NOT rebuilt on transform; layer
  back-face substitution (`layers.rs:97-139`) rewrites name/types/keywords/PT only. Read by
  activation (`resolution.rs:1856`, `expect_characteristics`) and Normal-trigger scans. → **Fix by
  base-rebuild at the transform boundary** (extract `build_face_ability_vectors` from `enrich`; call
  from `apply_face_change`). Base == layer-resolved → all readers fixed, zero reader-auditing.
- **Channel B (`def.abilities` direct-index)** — static registration, CardDefETB ETB queue, upkeep
  sweep (`turn_actions.rs:277`), mana sweep (`mana.rs:675`), Saga SBA (`sba.rs:839/878`), CardDefETB
  consumers (`abilities.rs:6002/6816/6889/7012/8169`). → **Fix with `CardDefinition::effective_abilities`
  (card-types) on producer+consumer, gated on live `is_transformed`.**
- **Statics**: deregister-old / register-new at the boundary (no face tag on `ContinuousEffect`).
- **8 boundary sites** route through one `apply_face_change`: engine.rs:1209/1433, effects/mod.rs:4292,
  turn_actions.rs:1648, resolution.rs:665/7173/7214/7317.
- **Affected roster**: docent_of_perfection + bloodline_keeper (Complete but currently WRONG — probe,
  expect stay Complete); growing_rites_of_itlimoc + thaumatic_compass (Partial, back abilities become
  functional, stay Partial); fable_of_the_mirror_breaker (Partial, correct C2 message). delver/
  braided_net/beloved_beggar/brutal_cathar back faces are keyword-only or empty — unaffected.
- **edgar half DEFERRED**: `edgar_charmed_groom.rs` does not exist; needs re-added
  `Effect::ReturnSourceToBattlefieldTransformed` (removed in OS4 narrowing → 1 wire bump). File as
  OOS-OS4-3.
- Wire baseline confirmed: PROTOCOL 19 (`protocol.rs:178`), HASH 56 (`hash.rs:504`). Mandatory = no
  change.

## Step checklist
- [x] Plan written (`memory/primitives/pb-plan-OS4b.md`, 2026-07-19).
- [x] Change 1: `CardDefinition::effective_abilities` (card-types), `cargo check -p mtg-card-types` clean.
- [x] Change 2a: `build_face_ability_vectors` extracted from `enrich_spec_from_def`
  (testing/replay_harness.rs); zero-diff refactor verified (full `cargo test --all` green
  before/after). Committed `96c5cbfd`.
- [x] Change 2b/3: new `rules/face.rs::apply_face_change` (deregister-old -> flip ->
  rebuild Channel A -> register-new); all 8 boundary sites routed (engine.rs:1209/1433,
  effects/mod.rs:4292, turn_actions.rs:1648, resolution.rs disturb-enter/TransformTrigger/
  CraftAbility/DayboundTransformTrigger). Committed `6a29ad52`.
- [x] Change 4: `register_static_continuous_effects` gained `is_transformed: bool`, iterates
  `def.effective_abilities(is_transformed)`; `deregister_face_statics` added (structural
  match, resolves `EffectFilter::Source` first). All 20 call sites updated (14 production +
  5 test + 1 inside face.rs). Committed `6a29ad52`.
- [x] Change 5: Channel-B def-index sites routed through `effective_abilities` — ETB queue
  (replacement.rs), upkeep sweep (turn_actions.rs), mana-tap sweep (mana.rs), Saga SBA
  (sba.rs ×2), 5 CardDefETB stack-build consumers (abilities.rs), PLUS 3 resolution-time
  CardDefETB/Normal-fallback/modal-mode consumers (resolution.rs, found by the probe/decoy
  test suite — not in the plan's line-number table but squarely in scope: the actual effect
  lookup at trigger resolution, not just stack placement). `get_self_activated_reduction`
  guarded on `is_transformed`. Committed `217f511b` + `a3161d81` (the resolution.rs +
  turn_actions.rs upkeep-sweep fixes were a second pass after the probe suite caught that
  the first turn_actions.rs edit had silently failed to apply).
- [x] Probe tests (docent, bloodline, growing_rites, thaumatic_compass, fable) — 19 tests in
  `crates/engine/tests/mechanics_m_z/pb_os4b_face_aware_abilities.rs`, all passing.
  docent_of_perfection + bloodline_keeper verified `Complete` by execution (pinning tests
  added). Committed `a3161d81`.
- [x] Decoy tests (front-static-removed, back-upkeep-only, there-and-back, saga-not-sacrificed,
  non-DFC-noop, off-battlefield-front) — same file/commit, all passing.
- [x] Card-def message fixes: `fable_of_the_mirror_breaker.rs` C2 message + ability-level +
  module-level comments corrected (back activated ability now reachable/activatable; stays
  Partial for ch. I/II only). `growing_rites_of_itlimoc.rs` message verified already accurate
  (its "fully implemented" claim is now true post-fix) — no edit needed. `thaumatic_compass.rs`
  left unchanged per plan. `beloved_beggar`/`brutal_cathar` spot-checked: keyword-only back
  faces, no regression risk, no marker change.
- [ ] Docs (OOS-OS4-2 resolved banners) — deferred to close-out per runner brief.
- [ ] Review (`pb-review-OS4b.md`): verify docent/bloodline stay Complete by execution; static
  deregister correctness; PROTOCOL 19 / HASH 56 unchanged; front-only sites untouched.
- [ ] Deferred follow-up: file OOS-OS4-3 (edgar_charmed_groom + re-add
  `Effect::ReturnSourceToBattlefieldTransformed`, one wire bump).

## Fix phase (pb-review-OS4b.md)
- [x] E1 (MEDIUM): first-main (`turn_actions.rs` precombat_main_actions), postcombat-main
  (`postcombat_main_actions`), and end-step (`end_step_actions`) CardDef trigger sweeps now
  route through `def.effective_abilities(obj.is_transformed)`, mirroring the upkeep sweep at
  `turn_actions.rs:284`. Added `test_front_end_step_no_trigger` +
  `test_back_end_step_trigger_fires_only_when_transformed` decoys to
  `pb_os4b_face_aware_abilities.rs` (mirroring the upkeep decoy pair). 21/21 tests in that file
  pass.
- [x] E2 (LOW): took the "leave as-is, correct + expand the doc constraint" path, not the
  symmetric extension. Re-reading `register_static_continuous_effects` for this fix found the
  non-`Static` family is actually **10 variants** registered from the effective face (not the
  4 the review named): `TriggerDoubling`, `SuppressCreatureETBTriggers`, `StaticRestriction`,
  `CdaPowerToughness`, `CdaModifyPowerToughness` (up to 2 entries/ability), `AdditionalLandPlays`,
  `StaticFlashGrant`, `StaticPlayFromGraveyard`, `StaticPlayFromTop` — spread across 7 different
  `state.*` collections with heterogeneous shapes (some `Option<ObjectId>` source fields, some
  1-or-2-entries-per-ability). That's materially larger/riskier than the `Static` case per the
  task's own fallback criterion, so `deregister_face_statics`'s doc comment in `face.rs` was
  rewritten to enumerate the full, verified list of untouched variants + their collections
  (correcting the review's undercount) rather than attempting the extension. No roster card
  reaches this today.
- [x] Full gates re-run post-fix: `cargo build --workspace` clean; `cargo test --all` 3512
  passed / 0 failed; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` +
  `tools/check-defs-fmt.sh` (1799 defs) clean; `PROTOCOL_VERSION == 19` / `HASH_SCHEMA_VERSION
  == 56` confirmed unchanged (behavior-only fix, no wire bump).

## Files (plan/review)
- Plan: `memory/primitives/pb-plan-OS4b.md`
- Review: `memory/primitives/pb-review-OS4b.md`

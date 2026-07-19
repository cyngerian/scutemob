# Primitive WIP — PB-OS7 (phase: implement)

<!-- PLAN COMPLETE. Key findings: EffectFilter::CreaturesControlledBy(PlayerId) ALREADY EXISTS
(layers.rs:646, hash.rs:2077). Add DSL placeholder EffectFilter::CreaturesControlledByDefendingPlayer,
substituted at Effect::ApplyContinuousEffect time into the locked CreaturesControlledBy(pid) using
ctx.defending_player (None => skip, never unwrap_or(controller) — PlayerId(0) binds to controller, footgun).
PROTOCOL: NOT bumped (EffectFilter off SR-8 wire closure per protocol.rs:109-112; PB-EF4 precedent) —
verify gate stays green at v21, STOP+FLAG if it moves. HASH: 58->59 forced (EffectFilter in GameState hash
closure). Karazikar BLOCKED -> filed OOS-OS7-1; ship = 1 (silumgar). Plan: memory/primitives/pb-plan-OS7.md -->


<!-- last_updated: 2026-07-19 -->

**PB**: PB-OS7 — defending-player-scoped continuous filter (OOS-EF3-1)
**Task**: scutemob-137
**Branch**: feat/pb-os7-defending-player-scoped-continuous-filter-oos-ef3-1-s
**Phase**: fix — DONE (all 3 review findings applied; ready for collect)

## Fix phase (review pb-review-OS7.md: no HIGH, card ships; 1 MEDIUM doc-only + 2 LOW)

- [x] **Finding 1 (MEDIUM, doc-only)**: corrected the plan's CR 611.2c mischaracterization
  (`memory/primitives/pb-plan-OS7.md`, the "Membership semantics" design note + the "Membership
  vs player capture" risk bullet) — the live-membership behavior is NOT CR-correct, it is a
  pre-existing engine-wide simplification (Golgari Charm, Eyeblight Massacre share it). Filed
  **OOS-OS7-2** (correctness) in `memory/primitives/oos-retriage-plan-2026-07-18.md` under the
  PB-OS7 section: resolution-time affected-set snapshot semantics for one-shot continuous P/T
  effects (CR 611.2c). Renumbered the plan's pre-existing, never-formally-filed
  "optional adjacent seed OOS-OS7-2" (CreaturesControlledByTargetPlayer note) to OOS-OS7-3 to
  avoid an id collision. No engine code changed (correctly out of scope per
  implement-phase-default-to-defer).
- [x] **Finding 2 (LOW)**: updated the stale `FROZEN_HISTORY_PREFIX_DIGEST` prose in both
  `crates/engine/tests/core/protocol_schema.rs` (now describes the 21→22 bump, twenty-row prefix
  `[2..21]`) and `crates/engine/tests/core/hash_schema.rs` (now describes the 58→59 bump). Digest
  *values* were already correct (re-pinned during implement phase) — comment-only edit. Both
  `frozen_prefix_is_pinned` gates re-run and confirmed green (not escalated to HIGH).
- [x] **Finding 3 (LOW)**: added 2 tests to `pb_os7_defending_player_continuous_filter.rs`:
  `test_os7_non_dragon_attacker_does_not_trigger` (CR 205.3m subtype-filter negative) and
  `test_os7_planeswalker_attack_scopes_to_controller` (CR 508.4, `Some(pw.controller)` path).
  Suite is now 11 tests (was 9), all pass first try. Renumbered subsequent `// ── Test N` headers.
  Module docstring updated with test count + a note on the Finding-1 limitation's non-impact on
  these tests (all use static boards).

## Post-fix verification (ALL GREEN)

- [x] `cargo build --workspace` clean
- [x] `cargo test --all` — **3549 passed, 0 failed** (was 3547; +2 from Finding 3)
- [x] `cargo clippy --workspace --all-targets -- -D warnings` — clean
- [x] `cargo fmt --check` — clean
- [x] `tools/check-defs-fmt.sh` — 1803 defs checked, clean
- [x] `frozen_prefix_is_pinned` (both protocol_schema + hash_schema) — green

## Pre-fix verification checklist (implement phase, superseded by post-fix above)

- [x] `cargo build --workspace` clean
- [x] `cargo test --all` — 3547 passed, 0 failed
- [x] `cargo clippy --workspace --all-targets -- -D warnings` — clean
- [x] `cargo fmt --check` — clean (fmt applied once to the new test file)
- [x] `tools/check-defs-fmt.sh` — 1803 defs checked, clean
- [x] HASH_SCHEMA_VERSION 58→59, all ~40 test sentinels swept
- [x] PROTOCOL_VERSION 21→22 (deviation from plan, documented below and in protocol.rs)

## Engine changes (DONE)

- [x] `EffectFilter::CreaturesControlledByDefendingPlayer` added (`crates/card-types/src/state/continuous_effect.rs`, after `TriggeringCreature`)
- [x] Substitution arm in `effects/mod.rs` `ApplyContinuousEffect` (`None => return`, never `unwrap_or(controller)`)
- [x] `layers.rs` `filter_matches` `=> false` guard arm
- [x] `hash.rs` HashInto discriminant 36 + `HASH_SCHEMA_VERSION` 58→59 + `- 59:` History line + v59 `HashSchemaEpoch` row (re-pinned from failing-gate output); ~40 test sentinels swept 58→59
- [x] `silumgar_the_drifting_death.rs` authored `Complete` (no gated stubs, no TODOs); `check-defs-fmt.sh` clean; Karazikar NOT authored, OOS-OS7-1 already filed in the plan
- [x] `crates/engine/tests/primitives/pb_os7_defending_player_continuous_filter.rs` — 9 tests, all pass (plan asked for 8; added `test_os7_card_registered` smoke test). Registered in `tests/primitives/main.rs`.
- [x] **PROTOCOL DEVIATION FROM PLAN**: the plan predicted NO PROTOCOL bump (`EffectFilter` "off the wire closure" per the PB-EF4/v9 note). Empirically WRONG — `protocol_schema_fingerprint_is_pinned` failed. Root cause: PB-EF9 (v14) put `EffectDuration` — a sibling field of `EffectFilter` on the same `ContinuousEffectDef` struct — into the wire closure via `Effect::ApplyContinuousEffect`, which transitively pulled `EffectFilter` in too. The PB-EF4-era "off the wire closure" claim went stale at v14 and nobody updated it. Bumped `PROTOCOL_VERSION` 21→22, re-pinned `PROTOCOL_SCHEMA_FINGERPRINT` + appended `PROTOCOL_HISTORY` row + `FROZEN_HISTORY_PREFIX_DIGEST` + `protocol_version_sentinel`, and swept 5 test-suite `PROTOCOL_VERSION, 21` sentinels to 22. Documented the correction inline in `rules/protocol.rs`'s `- 22:` history line. Flagged here per the plan's explicit "STOP AND FLAG" instruction — reported, not silently absorbed.

## Brief

Add a **locked** `EffectFilter::CreaturesControlledBy(PlayerId)` (or a `DefendingPlayer`-locked
filter variant) that a continuous-effect **builder stamps with the captured defending player at
effect creation**. The layer system cannot read the resolving `EffectContext`, so the player must
be baked into the stored `ContinuousEffectDef` instance (PB-EF9's `WhileYouControlSource`
captured-you precedent; PB-EF3's per-attacker `DefendingPlayer` capture supplies the value).

**Wire**: new `EffectFilter` variant → single PROTOCOL 21→22 (+HASH 58→59 if forced) bump, justified.

## Candidates (both currently UNAUTHORED — new Complete defs, not flips)

- `silumgar_the_drifting_death` — "Flying, hexproof. Whenever a Dragon you control attacks,
  creatures defending player controls get -1/-1 until end of turn." Per-Dragon trigger (ALL your
  attacking Dragons trigger separately), scope = the DEFENDING player of *that* attack. Ruling
  2014-11-24 confirms: two Dragons at one opponent → that opponent's creatures -2/-2; a Dragon at a
  different opponent → that opponent's creatures -1/-1; a third opponent untouched.
  **Complete only if per-Dragon trigger + per-defender scope + EOT expiry + SBA all proven.**
- `karazikar` (Eye Tyrant) — "Whenever you attack a player, tap target creature that player
  controls and goad it." Needs the defending-player-scoped *target filter* (target-selection
  sibling) + goad. Planner decides: ship expressible half or stay honestly blocked with remainder
  named.

## Tests required (AC 5073)

- 4-player bystander decoy — non-defending opponents' creatures NOT debuffed
- until-EOT expiry — assert -1/-1 expires at cleanup (PB-OS1 reversion machinery)
- multi-attack same-turn stacking — two Dragons same player → -2/-2; different players → each
  defender own scope
- toughness-death SBA — 1-toughness defender creature dies; 1-toughness bystander does not

## Pipeline
planner → runner → reviewer (invoked directly; /implement-primitive not installed worker-side).

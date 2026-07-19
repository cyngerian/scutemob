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
**Phase**: implement

## Engine changes (DONE)

- [x] `EffectFilter::CreaturesControlledByDefendingPlayer` added (`crates/card-types/src/state/continuous_effect.rs`, after `TriggeringCreature`)
- [x] Substitution arm in `effects/mod.rs` `ApplyContinuousEffect` (`None => return`, never `unwrap_or(controller)`)
- [x] `layers.rs` `filter_matches` `=> false` guard arm
- [x] `hash.rs` HashInto discriminant 36 + `HASH_SCHEMA_VERSION` 58→59 + `- 59:` History line + v59 `HashSchemaEpoch` row (re-pinned from failing-gate output); ~40 test sentinels swept 58→59
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

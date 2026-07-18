# Primitive Batch Review: PB-EF4 — TriggeringCreature as effect subject/source

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 119.3, 702.15a (lifelink), 702.2b (deathtouch), 702.90b/e (infect),
611.2a / 613.1f (continuous effects / Layer 6), 603.6a / 508.1m (triggered abilities),
400.7 / 113.7a / 608.2m (object identity / LKI)
**Engine files reviewed**: `crates/card-types/src/state/continuous_effect.rs`,
`crates/card-types/src/cards/card_definition.rs`, `crates/engine/src/effects/mod.rs`
(ApplyContinuousEffect + DealDamage executors), `crates/engine/src/rules/layers.rs`
(matches_filter), `crates/engine/src/rules/replacement.rs`, `crates/engine/src/rules/copy.rs`,
`crates/engine/src/state/hash.rs`, `crates/engine/src/rules/protocol.rs`,
`crates/engine/src/testing/replay_harness.rs`
**Card defs reviewed**: 7 shipped (dragon_tempest, scourge_of_valkas, ogre_battledriver,
atarka_world_render, fervent_charge, dreadhorde_invasion, warstorm_surge) + shared_animosity
(inert, OOS-EF4-1) + terror_of_the_peaks (contrast, untouched); spot-checked the ~111-site
`source: None` migration.
**Tests reviewed**: `crates/engine/tests/primitives/pb_ef4_triggering_creature_subject_source.rs`
(10 tests: 2 required decoys + 1 regression companion + 7 card integration).

## Verdict: ship

Zero HIGH, zero MEDIUM findings. One LOW (a documented, plan-accepted LKI edge case). The
two capability additions are correctly implemented and fully chain-verified: the
`EffectFilter::TriggeringCreature` placeholder is substituted to `SingleObject` **before**
the `ContinuousEffect` is stored (with a clean `None => return` that leaks no partial
effect), and the `damage_source_id` override replaces **all 12** damage-source reads in
both the Player and Object branches of the `DealDamage` executor while leaving target
resolution and `ctx.controller` untouched. All 7 card defs match oracle text exactly,
including the subtle `exclude_self` boundaries and the count-includes-the-enterer semantics.
Both required decoys are provably non-vacuous. Wire bumps (PROTOCOL 8→9, HASH 46→47) carry
re-pinned fingerprints and history rows. Full suite 3382 passing.

## Engine Change Findings

None at HIGH or MEDIUM.

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `effects/mod.rs:285-295` | **Source override falls back to `ctx.source` (not the triggering creature's LKI) when the creature has left before resolution.** See detail below. |

## Card Definition Findings

None at HIGH or MEDIUM.

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 2 | LOW | `dragon_tempest.rs`, `scourge_of_valkas.rs` | **`PermanentCount` filter carries `has_card_type: Some(Creature)`**, technically narrowing "number of Dragons you control" to *creature* Dragons. Practically inert (Dragon is a creature type); flagged for completeness. |

### Finding Details

#### Finding 1: Source override loses LKI when the triggering creature is gone

**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:285-295`
**CR Rule**: 113.7a / 608.2m — "the game uses the last-known information about [an object
that has left]"; SR-13 made `damage_source_characteristics` read damage keywords via LKI
"so a source that has left its zone still applies infect / lifelink."
**Issue**: `damage_source_id` resolves `source` through `resolve_effect_target_list`, whose
`EffectTarget::TriggeringCreature` arm (`effects/mod.rs:6358-6367`) returns `vec![]` when
`state.objects.contains_key(&id)` is false. So if the entering/attacking creature has left
the battlefield before the triggered ability resolves (e.g. sacrificed or killed in
response), the override silently falls back to `ctx.source` — the enchantment (Warstorm
Surge) or Dragon Tempest itself — rather than the creature. The lifelink/deathtouch/infect
reads (which SR-13 deliberately made LKI-capable) then read the *wrong* object's
characteristics, and `DamageDealt.source` is mis-attributed. This is a rare response-window
edge case and the plan explicitly documented it as "acceptable degradation," but it partially
defeats SR-13's intent for the override path.
**Fix** (optional, low priority): compute the source id from `ctx.triggering_creature_id`
directly (bypassing the `contains_key` gate) when `source == Some(TriggeringCreature)`, so the
downstream LKI reads in `damage_source_characteristics` / `damage_source_controller` still fire
for a departed source, matching SR-13's pattern. Keep the `ctx.source` fallback only for the
genuinely-unresolvable case (Player-only resolution / `None` triggering id).

#### Finding 2: "Dragons you control" count restricted to creatures

**Severity**: LOW
**Cards**: `dragon_tempest.rs:77-82`, `scourge_of_valkas.rs:48-53`
**Oracle**: "…where X is the number of Dragons you control."
**Issue**: The `EffectAmount::PermanentCount` filter uses both `has_card_type: Some(Creature)`
and `has_subtype: Some(Dragon)`. A permanent that is a Dragon but not a creature (a rare
corner: a creature type granted to a non-creature permanent) would be excluded from the count,
whereas oracle counts all Dragons you control regardless of type. In every realistic board
state a Dragon subtype implies a creature, so this is practically inert.
**Fix** (optional): drop `has_card_type: Some(Creature)` from the two count filters and count
purely by `has_subtype: Dragon`. Not blocking — both defs are internally consistent and pass
their integration tests.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 611.2a / 613.1f (continuous subject, lock-in at resolution) | Yes | Yes | decoy 1 + ogre/atarka/fervent/dreadhorde tests |
| 119.3 (damage source object) | Yes | Yes | dragon_tempest/scourge/warstorm assert `DamageDealt.source == enterer` |
| 702.15a (lifelink reads source + source's controller) | Yes | Yes | decoy 2 + warstorm lifelink variant (P1 gains, P2 does not) |
| 702.2b / 702.80 / 702.90 (deathtouch/wither/infect off source) | Yes (threaded through `damage_source_characteristics`) | Indirect | not card-covered this batch; source path shared with lifelink which is tested |
| 508.1m (attack triggers) | Yes | Yes | atarka/fervent/dreadhorde via real `DeclareAttackers` |
| 603.6a (ETB triggers) | Yes | Yes | dragon_tempest/scourge/ogre/warstorm |
| 113.7a / 608.2m (LKI source) | Partial (Finding 1) | No | departed-source edge not tested; plan-accepted |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| dragon_tempest | Yes | 0 | Yes | flying→haste (`TriggeringCreature`); Dragon→X dmg (`Some(TriggeringCreature)` source), X counts enterer (no exclude_self) |
| scourge_of_valkas | Yes | 0 | Yes | self+other Dragon merged into one `exclude_self: false` trigger; `{R}:+1/+0` via `EffectFilter::Source` retained; Flying retained |
| ogre_battledriver | Yes | 0 | Yes | `exclude_self: true` ("another"); +2/+0 & Haste both `TriggeringCreature` |
| atarka_world_render | Yes | 0 | Yes | Legendary; Flying+Trample; Dragon-attack filter; double strike via `TriggeringCreature` |
| fervent_charge | Yes | 0 | Yes | +2/+2 via `ModifyBoth(2)`, no filter (any attacker you control) |
| dreadhorde_invasion | Yes | 0 | Yes | upkeep LoseLife+Amass retained; attack half filters `has_subtype: Zombie` + `min_power: 6` + `is_token`; lifelink via `TriggeringCreature` |
| warstorm_surge | Yes | 0 | Yes | `PowerOf(TriggeringCreature)` amount + `Some(TriggeringCreature)` source |
| shared_animosity | N/A (inert) | 1 (documented) | N/A | correctly stays `inert`, empty abilities vec (`registers_no_behavior`), OOS-EF4-1 cited |
| terror_of_the_peaks | N/A (contrast) | — | — | correctly untouched (no DealDamage construction; source would be `ctx.source` = Terror) |

## Verification Notes (chain-walked)

- **`EffectFilter::TriggeringCreature` substituted before storage**: `effects/mod.rs:3043-3046`
  resolves the placeholder inside `resolved_filter` at the top of the `ApplyContinuousEffect`
  arm — the `ContinuousEffect` is built (`id_inner`, `ts`, `eff`) only *after* this match, so
  `None => return` exits before any state mutation or event push. No raw `TriggeringCreature`
  can reach the layer system, and no partial effect leaks. `layers.rs:657` (`=> false`),
  `replacement.rs:2061` (`other.clone()`), and `copy.rs:114` (`_ => false`) all handle a stray
  raw variant safely; none can appear on a static ability, so the clone path is never
  exercised in practice.
- **DealDamage override completeness**: all 12 damage-source reads use `damage_source_id` —
  Player branch: doubling (308), prevention (319), `damage_source_characteristics` (330),
  `DamageDealt.source` infect (349), `PoisonCountersGiven.source` (356), `DamageDealt.source`
  non-infect (368), `damage_source_controller` (381); Object branch: doubling (413),
  prevention (427), `damage_source_characteristics` (437), `damage_source_controller` (512),
  `DamageDealt.source` (525). The only remaining `ctx.source` in the arm is the
  `unwrap_or(ctx.source)` fallback (295). `ctx.controller` is untouched, so `EachOpponent`
  target selection remains ability-controller-relative (correct). `source: None` →
  `damage_source_id == ctx.source`, exact prior behaviour.
- **Migration**: exactly 3 card files carry `source: Some(EffectTarget::TriggeringCreature)`
  (the override cards); 108 `source: None` insertions across 88 files + `replay_harness.rs`
  handled as a match **pattern** (`..`), not a construction. `cargo build --workspace` +
  `cargo test --all --no-run` closed the set (incl. ~30 test-file sites).
- **Tests non-vacuous**: decoy 1 (same-type decoy present before ETB) asserts only the enterer
  is pumped/hasted; non-vacuity by swapping to `CreaturesYouControl`. decoy 2 constructs
  source-controller (P1) ≠ ability-controller (P2) and asserts lifelink credits P1, not P2;
  non-vacuity by reverting the `damage_source_id` threading. Attack-trigger card tests exercise
  the **real** `Command::DeclareAttackers` → stack-resolution path, so `triggering_creature_id`
  threading (resolution.rs) is validated end to end. ETB tests synthesize the ETB event but
  still resolve the stack through real `process_command`, exercising the same threading.
- **Wire**: `PROTOCOL_VERSION = 9` with fingerprint `9bf63ef2…`, history `- 9:` row + ledger
  epoch present; `HASH_SCHEMA_VERSION = 47` with `decl 35e95651…` / `stream 64546a7b…`,
  history `- 47:` row + ledger epoch present; `EffectFilter::TriggeringCreature` hashed at
  discriminant 35 (prior last-used 34); `Effect::DealDamage` hashes `source` after `amount`.
  Both bumps machine-forced by the schema/hash gates (suite green at 3382).

## Previous Findings (re-review only)

N/A — first review of PB-EF4.

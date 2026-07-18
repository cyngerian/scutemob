# Primitive WIP: PB-EF4 — TriggeringCreature as effect subject/source

batch: PB-EF4
title: Add EffectFilter::TriggeringCreature (continuous-effect subject = the just-triggered creature) and an optional source: Option<EffectTarget> on Effect::DealDamage (the triggering permanent as damage source, honoring its characteristics for lifelink/infect/prevention/doubling). Closes EF-W-PB2-6 (≡ EF-W-MISS-5) and EF-W-PB2-7.
task: scutemob-105
branch: feat/pb-ef4-triggeringcreature-as-effect-subjectsource-ef-w-pb2-6
started: 2026-07-18
phase: plan
plan_file: memory/primitives/pb-plan-EF4.md

## Source findings
- memory/primitives/ef-batch-plan-2026-07-17.md — PB-EF4 section (line ~290), Cluster B (§1a)
- memory/card-authoring/w-pb2-engine-findings-2026-07-17.md — EF-W-PB2-6 (line 101), EF-W-PB2-7 (line 115)
- memory/card-authoring/w-miss-roster-2026-07-17.md — EF-W-MISS-5 (exact dedup of PB2-6)

## Recon (done by coordinator before planning) — VERIFY before implementing

### Two gaps, one PB
1. **EF-W-PB2-6 ≡ EF-W-MISS-5** — `EffectFilter` (crates/card-types/src/state/continuous_effect.rs:67)
   has no `TriggeringCreature`. So "when a creature enters, IT gains <keyword> until end of turn"
   (a continuous effect granted to the entering creature) is inexpressible. `EffectTarget` already
   HAS `TriggeringCreature` (point effects), but `EffectFilter` (continuous) does not.
2. **EF-W-PB2-7** — `Effect::DealDamage { target, amount }` (crates/card-types/src/cards/card_definition.rs:1330)
   always sources from `ctx.source`. So "when another permanent enters, IT deals X damage" (entering
   permanent as damage source) is inexpressible. Dragon Tempest is never a Dragon, so it misattributes
   on 100% of firings (currently `inert`).

### Threading already exists (PB-EF3)
- `EffectContext.triggering_creature_id: Option<ObjectId>` exists (effects/mod.rs:120), threaded
  StackObject→EffectContext at resolution.rs:2109 / 2202. Set at abilities.rs:7930
  (`stack_obj.triggering_creature_id = trigger.entering_object_id`). BUILD ON THIS — do not duplicate.

### EffectFilter::TriggeringCreature resolution site
- `EffectFilter::Source` is resolved to `SingleObject(ctx.source)` at `ApplyContinuousEffect` execution
  time and also handled in layers.rs:653 (returns false in the static-registration matcher) and
  replacement.rs:2058. The new `TriggeringCreature` variant should resolve to
  `SingleObject(ctx.triggering_creature_id)` at the SAME ApplyContinuousEffect execution site (mirror
  Source / DeclaredTarget). If triggering_creature_id is None, the continuous effect applies to nothing
  (no panic). hash.rs (~1857) needs a new discriminant byte.

### DealDamage source override
- Executor: effects/mod.rs:271. Every damage-source read in this arm currently uses `ctx.source`:
  apply_damage_doubling, apply_damage_prevention, damage_source_characteristics (infect/lifelink),
  damage_source_controller (lifelink gain), and the `source:` field of DamageDealt/PoisonCountersGiven
  events. Compute ONE `let damage_source_id = source.as_ref().map(|t| resolve to single ObjectId via
  resolve_effect_target_list, first).unwrap_or(ctx.source)` at the top of the arm and thread it through
  every one of those reads. `EffectTarget::TriggeringCreature` is already resolvable, so
  `source: Some(EffectTarget::TriggeringCreature)` works out of the box.
- `source: None` = existing behaviour (ctx.source). Full suite must prove no regression.

### Blast radius
- `Effect::DealDamage` is constructed at ~110 sites across 90 card-defs files + 4 in engine/card-types.
  Adding a required `source` field means adding `source: None,` at every construction site AND updating
  the executor match-arm pattern to bind `source`. Mechanical but large; must keep check-defs-fmt.sh green.
- Wire: both changes reach the SR-8 fingerprint closure (EffectFilter variant + DealDamage shape) →
  **PROTOCOL 8→9** forced. HASH: DealDamage/EffectFilter appear in the GameState hash closure via
  Characteristics→Effect → likely **HASH 46→47** forced too. Let the machine gates force both; re-pin
  PROTOCOL_SCHEMA_FINGERPRINT + sentinel hashes; append history rows.

### Candidates (8, discounted ~4-5) — chain-verify each vs oracle via MCP (feedback_verify_full_chain)
- dragon_tempest (BOTH halves — flip `inert`): "Whenever Dragon Tempest or another Dragon enters, that
  creature deals X damage..." (DealDamage source=triggering) + "...and gains haste" (if flying → haste,
  TriggeringCreature filter). VERIFY exact oracle.
- scourge_of_valkas (flip): DealDamage source override (the "or another Dragon enters" half).
- ogre_battledriver (flip): TriggeringCreature filter ("that creature gains haste, +2/+0 UEOT").
- shared_animosity (verify — may be a ForEach count effect, not a TriggeringCreature filter — chain-verify!).
- Atarka World Render, Fervent Charge, Goblin Piledriver, Muxus Goblin Grandee — chain-verify; several
  may be OUT of scope (Goblin Piledriver is a self-pump P/T static; Muxus is a search/reveal — likely
  NOT this primitive). Demote honestly where a clause is still inexpressible. NO gated-stub effects.

## Phase log
- 2026-07-18 plan: dispatched primitive-impl-planner.
- 2026-07-18 plan DONE: `memory/primitives/pb-plan-EF4.md` written. Roster-recall TODO sweep found
  2 forced adds beyond the 8-card brief (dreadhorde_invasion, warstorm_surge) → **7 ship** (was ~4-5
  est): dragon_tempest (flip inert, BOTH primitives), scourge_of_valkas (flip partial, DealDamage
  source), ogre_battledriver (flip inert, TriggeringCreature ×2), atarka_world_render (NEW),
  fervent_charge (NEW), dreadhorde_invasion (flip partial, TriggeringCreature lifelink grant),
  warstorm_surge (flip partial, DealDamage source + PowerOf(TriggeringCreature)). BLOCKED:
  shared_animosity (count EffectAmount missing → file OOS-EF4-1), goblin_piledriver + muxus (OUT OF
  SCOPE — self-attack Source / ETB reveal, neither PB-EF4 primitive is the blocker). terror_of_the_peaks
  = deliberate contrast (source=ctx.source, keep source:None). Wire: PROTOCOL 8→9, HASH 46→47 (both
  machine-forced). ~115 DealDamage construction sites need `source: None,` (3 override cards get
  `Some(TriggeringCreature)`). serde(default) on source: YES (codebase convention; does NOT reduce
  blast radius). Exhaustive-match compile-forced sites: layers.rs matches_filter, hash.rs EffectFilter
  (disc 35) + Effect::DealDamage. Next: impl phase.

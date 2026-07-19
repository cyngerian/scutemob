# Primitive WIP — PB-OS6 (DFC flip-condition sub-batch, OOS-EF5-4 a/b/c/d/g)

<!-- last_updated: 2026-07-19 -->

**Phase**: implement
**Task**: scutemob-136
**Branch**: feat/pb-os6-dfc-flip-condition-sub-batch-oos-ef5-4-abcdg-delver-l

## Plan decisions (pb-plan-OS6.md)
**SHIP 3** → Complete: (a) delver_of_secrets, (b) legions_landing (NEW), (g) thaumatic_compass.
**DEFER 2**: (d) growing_rites → PB-OS8 (`LookAtTopThenPlace`); (c) westvale_abbey → new seed
OOS-OS6-1 (multi-count sacrifice cost needs Command wire reshape, ~90 edits, single-card yield).
**Wire**: single batched PROTOCOL 20→21 + HASH 57→58 (both forced). Engine changes:
`Condition::TopCardIsInstantOrSorcery`, `Condition::YouAttackedWithNOrMore(u32)` +
`PlayerState.attackers_declared_this_turn`, `Effect::RemoveFromCombat{target}` +
`GameEvent::RemovedFromCombat` + shared `remove_from_combat` helper.

## Brief
DFC flip-condition sub-batch. Five small, INDEPENDENT sub-primitives, each additive to an
existing enum already in the SR-8 wire closure. Canonical seed: OOS-EF5-4 in
`memory/primitives/ef-batch-plan-2026-07-17.md` §9. Queue entry: `oos-retriage-plan-2026-07-18.md`
§3 (PB-OS6). PB-OS4b landed face-aware ability gathering, so back-face abilities now genuinely
register — delver/growing_rites/thaumatic flips are real once their conditions become expressible.

### Sub-primitives
- **(a) delver_of_secrets** — `Condition` "top card of library is instant/sorcery" with reveal
  (upkeep flip). Only `TopCardIsCreatureOfChosenType` exists. Card is `partial` → target Complete.
- **(b) legions_landing** — count field on `TriggerCondition::WheneverYouAttack` ("attacked with
  3+ creatures"). Bare unit today. UNAUTHORED — author to Complete if (b) ships.
- **(c) westvale_abbey** — count field on `Cost::Sacrifice` ("sacrifice five creatures").
  No count field today. UNAUTHORED — author to Complete if (c) ships.
- **(d) growing_rites_of_itlimoc** — "look at top N, put a matching card into hand, bottom the
  rest" ETB. Overlaps PB-OS8 `LookAtTopThenPlace` family. **Planner decides**: implement minimal
  shape here OR defer to OS8 with recorded justification (AC 5069). Card is `partial`.
- **(g) thaumatic_compass** — `Effect::RemoveFromCombat { target }` (Spires of Orazca back face,
  CR 506.4/508). Card is `partial` → target Complete.

### Sequence
(a)/(g) first (smallest), then (b)/(c), then (d) decision.

### Wire (SR-8)
Several sub-primitives touch fingerprint-closure enums (`Condition`, `TriggerCondition`, `Cost`,
`Effect`). Batch into a SINGLE PROTOCOL 20→21 (+HASH 57→58 only if forced) bump for the whole PB;
justify once. Verify exact wire impact at plan time.

### Candidates (5) / discounted ship ~3
delver_of_secrets (partial→Complete), legions_landing (new), westvale_abbey (new),
growing_rites_of_itlimoc (partial→Complete or stays partial if (d) deferred), thaumatic_compass
(partial→Complete).

### Tests
Each shipped sub-primitive pinned by a decoy failing on exactly the field under test:
- delver reveal decoy: top card creature → no flip
- westvale count decoy: 4 creatures cannot pay
- legions count decoy: 2 attackers no trigger, 3 fires once
Probe by execution (SR-34/36): each flipped card needs an executing test path.

## Progress (implement phase)
- [x] Change 1 — `Condition::TopCardIsInstantOrSorcery` (card_definition.rs enum + check_condition eval arm + hash.rs discriminant 49)
- [x] Change 2 — `Condition::YouAttackedWithNOrMore(u32)` + `PlayerState.attackers_declared_this_turn` (player.rs field, builder.rs init, combat.rs setter, turn_actions.rs reset, check_condition eval arm, hash.rs discriminant 50 + PlayerState field hash)
- [x] Change 3 — `Effect::RemoveFromCombat{target}` + `GameEvent::RemovedFromCombat` + shared `remove_from_combat` helper (combat.rs helper factored out of apply_regeneration; execute_effect dispatch arm; hash.rs Effect discriminant 95 + GameEvent discriminant 128)
- [x] Change 4 — exhaustive-match sweep: `cargo build --workspace` clean after Changes 1-3 (no tool/replay-viewer arms needed — verified no exhaustive Condition/Effect/GameEvent match outside hash.rs + effects/mod.rs)
- [ ] Wire bump: PROTOCOL 20→21 + HASH 57→58 (batched)
- [ ] Card defs: delver_of_secrets, thaumatic_compass → Complete
- [ ] New card def: legions_landing.rs → Complete
- [ ] Doc-only: growing_rites_of_itlimoc.rs re-point to PB-OS8
- [ ] Tests: crates/engine/tests/primitives/pb_os6_dfc_flip_conditions.rs

## Plan output
`memory/primitives/pb-plan-OS6.md`

## Prior
Last completed: **PB-OS5** (`scutemob-135`) — relative-count EffectAmount, PROTOCOL 19→20 /
HASH 56→57. Active queue: `memory/primitives/oos-retriage-plan-2026-07-18.md`.

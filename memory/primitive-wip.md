# Primitive WIP — IDLE (no PB in progress)

<!-- last_updated: 2026-07-19 -->

No primitive batch is currently in flight. Last completed: **PB-OS5** (`scutemob-135` — dynamic
relative-count `EffectAmount`, OOS-EF4-1 closed). Added ONE new variant
`EffectAmount::OtherAttackersSharingCreatureType { relative_to: EffectTarget }` (discriminant 24) —
resolution-time count of OTHER attacking creatures (any controller) sharing ≥1 layer-resolved
creature type with the triggering creature (Changeling-safe). `shared_animosity` inert→**Complete**;
`goblin_piledriver` NEW→**Complete** (+2/+0 via `Sum(count,count)`); `goblin_rabblemaster` pump
implemented (stays partial — forced-attack blocker); `muxus_goblin_grandee` attack-half authored
(stays partial — ETB reveal/put → OOS-EF10/PB-OS8). Piledriver/rabblemaster/muxus reused existing
`AttackingCreatureCount`/`PermanentCount` (zero new surface). **Single PROTOCOL 19→20 / HASH 56→57.**
Reviewer clean bill. Plan `pb-plan-OS5.md`, review `pb-review-OS5.md`.

**Active queue**: `memory/primitives/oos-retriage-plan-2026-07-18.md` — next **PB-OS6** (DFC
flip-condition sub-batch, OOS-EF5-4), then OS7..OS11; OOS-OS4-3 (edgar + re-added
ReturnSourceToBattlefieldTransformed, 1 wire bump) rides with a future capability batch.

Start the next PB via /dispatch per the queue plan — /implement-primitive picks up this file.

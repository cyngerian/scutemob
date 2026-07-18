---
name: Card Authoring Campaign Plan (2026-05-16)
description: Costed, sequenced plan to take card authoring from 52.9% clean to 100% clean coverage.
type: plan
---

# Card Authoring Campaign Plan — 2026-05-16

**Companion doc**: `dsl-gap-audit-2026-05-16.md` (the gap analysis this plan costs).
**Status**: LAUNCHED (`scutemob-39..42` + PB-AC0) — **RECALIBRATED 2026-07-07** against measured batch data. See §0.
**Ground truth**: `docs/authoring-status.md` — regenerate with `python3 tools/authoring-report.py`. Clean coverage 928/1,748 = 53.1% as of 2026-07-07.

---

## 0. Recalibration — 2026-07-07 (measured data overrides §1 estimates)

Two derisking batches (W-NOW-1 batches 1–2, `scutemob-40`/`scutemob-42`, 24 cards
from the *strongest* "verified-stale" cohort) measured the actual disposition mix:

| Measured (24 cards) | Count | Rate |
| --- | ---: | ---: |
| Fully CLEAN by re-authoring alone | 4 | **17%** |
| PARTIAL (expressible parts authored; ≥1 `ENGINE-BLOCKED` clause remains) | 13 | 54% |
| Fully BLOCKED | 7 | 29% |

And batch 1's "clean" ETB-subtype cards only became clean because **PB-AC0**
(merge `df997fd2`) fixed the creature-ETB path silently dropping
`has_subtype`/nontoken filters — i.e. even the flagship stale cohort needed an
engine fix first.

**What this changes:**

1. **The ~435 "AUTHORABLE NOW" estimate is wrong.** Gap-audit "NOW-EXPRESSIBLE"
   claims must be verified per card. Extrapolating the measured 17% over the
   ~210 stale-TODO files yields only **~35–50 fully-clean cards** from pure
   re-authoring. The ~110 empties and ~115 missing files were classified by the
   same audit — apply the same discount until a derisking batch measures them.
2. **Engine primitives are the bottleneck, not authoring throughput.** Most
   cards carry ≥1 genuinely blocked clause. The PB track (PB-AC1..AC9) is the
   critical path; §4's "two equal parallel tracks" framing is superseded —
   run **PB-first**, with each PB's unblocked cohort authored immediately behind
   it (the PB-AC0 rhythm: one engine fix flips a whole cohort).
3. **ETB-subtype cluster: RESOLVED** by PB-AC0. `WheneverCreatureEntersBattlefield`
   now honors `has_subtype` + nontoken on the creature-ETB path (+13 tests, 2873 total).
4. **W-NOW batches 3+ are PAUSED.** Re-authoring partials produces fidelity but
   not clean-coverage movement (batch 2: 0/12 clean). Resume stale-TODO authoring
   as post-PB cohort backfill. Optionally run one 12-card derisking batch each of
   W-EMPTY and W-MISS to measure those cohorts before scheduling them.
5. **Measured blocked-clause themes confirm the PB definitions** (no redesign
   needed): beneficial-pay riders ("you may pay/sac/discard, if you do…") → PB-AC2;
   once-per-turn limiter → PB-AC1; static ability-grants to filtered permanents +
   spell-subtype filters (Aura/Equipment/Vehicle) → PB-AC7; batched/filtered
   attack triggers → PB-AC6; death-trigger "exile it"/LKI riders → PB-AC2/AC9 review.
6. **Dual markers**: incomplete clauses are marked `// TODO` **or**
   `// ENGINE-BLOCKED` — all tooling/greps must match both (`fa4d593f` fixed
   `authoring-report.py` for this).

**Next action** ~~dispatch **PB-AC1**~~ — **SUPERSEDED 2026-07-17.** The PB-AC chain
(AC0..AC9), the marker sweep (`scutemob-88`), and the W-PB2 / W-EMPTY / W-MISS authoring
waves (`scutemob-95/96/97`) are all COMPLETE (clean coverage 59.8%). The active
engine-primitive queue is now **`memory/primitives/ef-batch-plan-2026-07-17.md`**
(`scutemob-98`) — it consolidates the 19 EF findings those waves filed + EF-13 into an
ordered, deduped, correctness-first PB queue. **Recommended first dispatch: PB-EF1**
(`exclude_self` enforcement sweep), preceded by the coordinator one-liner demoting
`swan_song` (EF-W-MISS-1, the one live-wrong `Complete` def). ~~EF-13 (105 mis-typed
markers) is a coordinator decision presented in that plan's §3.~~ **EF-13 RESOLVED
(Option A, `scutemob-101`, 2026-07-18):** the no-behaviour `Partial` class — enumerated
from the compiled registry (`all_cards()` + `card_registry_gate::registers_no_behavior`)
as **101 defs** (down from 105 at the marker sweep; PB-EF1 and the W-* waves flipped a
few) — was reclassified `Partial→Inert`, and `card_registry_gate` now machine-forbids
`Partial`/`KnownWrong` on any def where `registers_no_behavior` is true so the class
cannot re-form. **Reporting shift** (deliberate, per Option A): `todo` 655→554,
`empty` 57→158 (both ±101). **Clean-coverage headline unchanged: 1,070 = 60.0%** — no
def's behaviour changed, no `Complete` def moved. No HASH/PROTOCOL bump (marker-only).
The PB-first + cohort-backfill rhythm below still holds; the EF batch plan is where the
PB definitions now live.

---

## 1. Headline split — ⚠️ SUPERSEDED by §0 (kept for the original reasoning)

Card authoring is at **52.9% clean** (924 / 1,748 def files). To reach 100% clean
we must resolve: 642 TODO-bearing files, 182 empty placeholders, and 194 missing
plan cards — 1,018 cards of work in three forms.

Classifying every card by what it needs:

| Disposition | Cards (est.) | What it takes |
| --- | ---: | --- |
| **AUTHORABLE NOW** — no engine work | **~435** | Re-author / write the def against today's DSL. ~210 stale-TODO files + ~110 empties + ~115 missing files. |
| **ENGINE-BLOCKED** — needs a new primitive | **~470** | One of 9 proposed PBs unblocks it, then author. |
| **DEFER** — honestly out of scope | **~110** | Hidden-info, cosmetic-only, post-alpha. Many already function; TODO is a fidelity note. |

~~The single most important finding: **~435 cards are free**~~ — **FALSIFIED by
measurement** (see §0): the fully-clean rate on the strongest stale cohort was 17%,
and even those needed PB-AC0 first. Engine primitives are the bottleneck.

---

## 2. Proposed primitive batches

Nine PBs cover the genuine engine gaps. They are deliberately **few and large** —
the v2-era plan had 15 micro-PBs (PB-23..37) and the per-PB overhead was high.
Each PB below bundles a coherent gap cluster. CR refs given where obvious.

Yields are **discounted ~2.5x** from raw TODO-theme counts per the project's
documented PB-overcount bias (`feedback_pb_yield_calibration.md`).

| PB | Name | Primitives added | CR refs | Card yield (disc.) | Effort |
| --- | --- | --- | --- | ---: | --- |
| **PB-AC0** ✅ DONE | Creature-ETB filter forwarding (unplanned, found by batch 1) | `ETBTriggerFilter` subtype/nontoken fields forwarded on the creature-ETB path (`replay_harness.rs`, `abilities.rs`) | 603.2 | ETB-subtype cohort | merged `df997fd2`, +13 tests |
| **PB-AC1** | Counter / untap / once-per-turn | `Effect::UntapAll { filter }`; `TriggerCondition::WheneverPermanentUntaps`; `WhenCounterPlaced`; generic `once_per_turn` limiter on triggered abilities; "doesn't untap" static | 701.20, 701.21, 603.2 | ~22 | M |
| **PB-AC2** | Optional-cost & counter-tax | optional-cost wrapper on triggered effects (general "you may pay/sacrifice/discard, if you do…"); `Effect::CounterUnlessPays { cost }` (caster-side, vs existing `MayPayOrElse`) | 118.8, 603.2, 701.5 | ~20 | M |
| **PB-AC3** | Dynamic P/T & count amounts (CDA residual) | `LayerModification::ModifyBoth` accepting `EffectAmount`; `EffectAmount::{AttackingCreatureCount, TappedCreatureCount, HandSize}` + power-based token count | 613, 107.3 | ~14 | L |
| **PB-AC4** | Modal & optional targeting | per-mode `TargetRequirement` on `ModeSelection`; `TargetRequirement` optional / `UpToN` variant | 601.2c, 700.2 | ~20 | M |
| **PB-AC5** | Alt-costs & timing keywords | `AltCostKind::Warp` + `KeywordAbility::Warp`; `KeywordAbility::Transmute`; `KeywordAbility::Exert`; `Cost::ExileFromHand` (pitch) | 702.x, 118.9 | ~14 | M |
| **PB-AC6** | Phase / opponent-action conditions | `TriggerCondition::{AtBeginningOfFirstMainPhase, AtBeginningOfPostcombatMain, WhenBecomesTarget}`; `Condition::{YouAttackedThisTurn, CreatedATokenThisTurn, OpponentCastNSpells, SpellMastery, OpponentControlsMoreLandsThanYou}` | 500, 603.2, 700.2 | ~18 | M |
| **PB-AC7** | Type-changing & ability-removal | `Effect::LoseAbilities`; `Effect::SetCreatureTypes`; one-shot layer-4 type override; spell-subtype filter (Aura/Equipment/Vehicle) | 613.1, 205.3 | ~14 | M |
| **PB-AC8** | Static restrictions, win-cons, no-max-hand | `GameRestriction::{NoMaximumHandSize, MustAttack, CantAttackOwner, CantBeSacrificed}`; `Effect::WinGame { condition }` | 104.2, 508, 119 | ~14 | S |
| **PB-AC9** | Misc & mana | `Effect::WheelHand`; multi-output filter mana (hybrid filter lands); `SearchLibrary` multi-name; token-doubling replacement; d20 + tiered outcome | 701.x, 605, 614 | ~16 | M |

**Total PB engine yield: ~150 cards** unblocked across 9 batches. The remaining
~320 engine-blocked count (470 − 150) is the *partially-expressible* set: those
cards become authorable once their PB ships AND the now-expressible parts are
written — they are counted once here, not double-counted with the authoring waves.

Each PB runs the `/implement-primitive` pipeline (plan → implement → review → fix
→ close). **Close includes backfill**: the PB is not done until every card it
unblocks has its TODO removed. This is the lesson from PB-23..37.

---

## 3. Authoring waves (no-engine-work re-authoring)

Layered on top of the PBs. These waves consume only authoring throughput. Each
wave = batches of 10-15 cards via the `bulk-card-author` / `card-fix-applicator`
agents, with a `card-batch-reviewer` pass per batch (per `/author-wave`).

| Wave | Content | Cards | Depends on |
| --- | --- | --- | --- |
| **W-NOW-1** | Stale-TODO re-author: subtype-filtered ETB/dies/attacks triggers, GainControl, conditional statics, triggering-object targets. The biggest stale cohort. | ~120 | nothing — launch immediately |
| **W-NOW-2** | Stale-TODO re-author: remaining now-expressible TODOs (GY-zone abilities, cost-reduction, LKI dies-triggers, additional-land). | ~90 | nothing |
| **W-EMPTY** | Author the ~110 authorable empty `vec![]` placeholders. | ~110 | nothing |
| **W-MISS** | Author the ~115 authorable missing-file cards (attack-trigger, activated-tap, activated-sacrifice, sacrifice-outlet, most of draw/token). | ~115 | nothing |
| **W-PB1** | Author cards unblocked by PB-AC1..AC3. | ~55 | PB-AC1/2/3 closed |
| **W-PB2** | Author cards unblocked by PB-AC4..AC6. | ~55 | PB-AC4/5/6 closed |
| **W-PB3** | Author cards unblocked by PB-AC7..AC9 + the partially-expressible residual. | ~95 | PB-AC7/8/9 closed |
| **W-AUDIT** | `/audit-cards` full sweep, fix stragglers, certify 100% clean. | — | everything |

---

## 4. Critical path & sequencing — ⚠️ SUPERSEDED by §0.2: run PB-first; Track A is paused, not parallel

```
   PARALLEL TRACK A (authoring, no engine dep) ── launch day 1 ──────────────►
     W-NOW-1 → W-NOW-2 → W-EMPTY → W-MISS                (~435 cards)

   PARALLEL TRACK B (engine, serialized — HASH-bumping work) ── launch day 1 ─►
     PB-AC1 → PB-AC2 → PB-AC3 → PB-AC4 → PB-AC5 → PB-AC6 → PB-AC7 → PB-AC8 → PB-AC9
     (each PB's close-step backfill IS part of W-PB1/2/3)

   CONVERGENCE
     W-AUDIT  (after both tracks drain)  →  100% clean, certify
```

- **Track A and Track B run concurrently.** Track A is pure authoring and shares
  no files with the engine PBs (card defs vs `crates/engine/src/`), so there is no
  collision — this is the W6 parallel model. Authoring waves can be `/dispatch`ed
  as worker batches while PBs proceed.
- **Track B is serialized internally** — PBs that touch `state/hash.rs`
  discriminants or shared enums must land one at a time to avoid HASH-merge
  conflicts (the established `scutemob-22..38` discipline).
- PB-AC1..AC3 first: they have the highest yield and lowest novelty (untap,
  optional-cost, CDA are well-trodden). PB-AC9 last (most heterogeneous).

---

## 5. Effort estimate

Calibration: a PB ≈ 1 plan + 1 implement + 1 review + 1 fix dispatch ≈ 3-4
worker sessions including its backfill. An authoring batch ≈ 12 cards ≈ 1
dispatch (author + review + fix folded). S/M/L effort tiers ≈ 3 / 4 / 6 sessions.

| Track | Item | Sessions |
| --- | --- | ---: |
| B | PB-AC1 (M), AC2 (M), AC4 (M), AC5 (M), AC6 (M), AC7 (M), AC9 (M) — 7×4 | 28 |
| B | PB-AC3 (L) — 6 | 6 |
| B | PB-AC8 (S) — 3 | 3 |
| A | W-NOW-1 (~120 cards / 12) | 10 |
| A | W-NOW-2 (~90 / 12) | 8 |
| A | W-EMPTY (~110 / 12) | 9 |
| A | W-MISS (~115 / 12) | 10 |
| A | W-PB1/2/3 authoring (~205 / 12) | 17 |
| — | W-AUDIT certify + straggler fixes | 4 |
| **Total** | | **~105 sessions** |

Because Tracks A and B run in parallel, **wall-clock is gated by the longer
track** (Track A authoring ≈ 64 sessions; Track B engine ≈ 37 sessions). With a
single coordinator dispatching, realistic completion is **~70-85 dispatched
worker sessions** end to end. Treat ~105 as the total *work*, ~75 as the
*critical-path* estimate.

> Risk note: PB yields are discounted but still optimistic — expect spawned
> micro-PBs and "this card needs one more thing" findings. Budget +15% (the
> `feedback_pb_yield_calibration.md` reality). Some DEFER cards may also need a
> tiny primitive once examined; fold into PB-AC9 or accept the defer.

---

## 6. DO-NOW section — ⚠️ EXECUTED (batches 1–2 = `scutemob-40`/`42`); batches 3+ PAUSED per §0.4

The user can launch this **today**, before reviewing any PB design:

> **W-NOW-1 — stale-TODO re-authoring, subtype-filtered triggers cohort.**
> ~120 cards whose TODOs cite primitives that already shipped. Start with the
> verified-stale clusters from the gap audit:
> - Subtype-filtered ETB triggers (`ganax_astral_hunter`, `lathliss_dragon_queen`, …)
>   → `WheneverCreatureEntersBattlefield { filter, exclude_self }`
> - Subtype-filtered death triggers (`crossway_troublemakers`, `pashalik_mons`,
>   `omnath_locus_of_rage`, …) → `WheneverCreatureDies { filter, … }`
> - "Creature you control attacks" triggers (`shared_animosity`, `hellrider`, …)
>   → `WheneverCreatureYouControlAttacks { filter }`
> - GainControl / ExchangeControl cards (9 files) → `Effect::GainControl`
> - Conditional-static cards → `ContinuousEffectDef.condition`
>
> Dispatch as `/author-wave`-style batches of 12. Each worker: confirm the DSL
> construct in `card_definition.rs`, look up oracle text via the MTG-rules MCP,
> rewrite the def, delete the TODO, run `cargo build --workspace`. Review every
> batch with `card-batch-reviewer` before commit (`W5-cards:` prefix).

This wave alone moves clean coverage from 52.9% to roughly **59-60%** and proves
the campaign cadence with zero engine risk.

---

## 7. Recommended first step — ⚠️ SUPERSEDED: the derisking run happened; next step is PB-AC1 per §0

**~~Launch W-NOW-1 (the DO-NOW wave) and PB-AC1 in parallel.~~**

1. `/dispatch` the first W-NOW-1 batch (12 subtype-filtered-trigger cards) — pure
   authoring, no engine risk, immediately demonstrates the stale-TODO thesis.
2. In parallel, `/dispatch` PB-AC1 (untap / counter / once-per-turn) via
   `/implement-primitive` — highest-yield, lowest-novelty engine batch.

If the user wants to derisk before committing the full campaign, run **just the
first W-NOW-1 batch** and confirm the staleness rate empirically (the gap audit
predicts most TODOs in that cohort delete cleanly). A high confirmed-stale rate
validates the ~435-card AUTHORABLE-NOW estimate that the whole plan rests on.

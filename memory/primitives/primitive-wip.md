<!-- STALE DUPLICATE (frozen at PB-AC9, 2026-07). The LIVE WIP file is memory/primitive-wip.md — /implement-primitive repointed there 2026-07-18 (scutemob-121). -->
---
pb: PB-AC9
title: Misc & mana (final AC-chain batch)
phase: closed
task: scutemob-52
branch: feat/pb-ac9-misc-mana
plan_file: memory/primitives/pb-plan-AC9.md
recon_file: memory/primitives/pb-recon-AC9.md
review_file: memory/primitives/pb-review-AC9.md
backfill_review_file: memory/card-authoring/review-pb-ac9-backfill.md
---

# PB-AC9 — Misc & mana (final AC-chain batch)

**PB-AC9 closes the AC chain (PB-AC1 … PB-AC9).**

## Scope decision — 3 of 5 briefed primitives already existed

The brief listed five primitives. Recon-first verification (worker recon, then an
independent planner recon that **overturned the worker's recon**, then a third
worker grep confirming the planner) established:

| Briefed primitive | Verdict | Evidence |
|---|---|---|
| `Effect::WheelHand` | **BUILT** | genuinely absent |
| d20 + tiered outcome | **ALREADY EXISTS** | `Effect::RollDice { sides, results }` (`effects/mod.rs:3547`), `EffectAmount::LastDiceRoll`, `GameEvent::DiceRolled`. Determinism already solved. |
| token-doubling replacement | **ALREADY EXISTS** (but half-wired) | `ReplacementModification::DoubleTokens` (`state/replacement_effect.rs:186`) |
| multi-output filter mana | **ALREADY EXISTS** | `Effect::AddManaFilterChoice` (`effects/mod.rs:2009`), used by `graven_cairns.rs`. Full 3-way choice is M10-interactive. |
| `SearchLibrary` multi-name | **DROPPED → OOS seed** | genuinely absent, but **zero card roster** — no card needs "a card named A or B". Per AC-chain precedent, do not build 0-yield primitives. |

Plus one **unbriefed co-blocker** discovered by the planner and built:
`Effect::SetNoMaximumHandSize`.

> **Methodological lesson (generalizes `feedback_verify_full_chain`):** a grep proving
> *absence* is only as good as the name you guess. The worker's first recon declared all
> five absent because it grepped `WheelHand|DiscardHand`, `d20|die roll`, `TokenDoubl` —
> the engine calls them `RollDice`, `DoubleTokens`, `AddManaFilterChoice`. **Absence must
> be established by reading the enum, not by grepping for the name you expect.**

## What shipped

- [x] **`Effect::WheelHand { player, disposal, draw }`** — `WheelDisposal` (Discard /
      ShuffleHandIntoLibrary / ShuffleHandAndGraveyardIntoLibrary) + `WheelDraw`
      (ThatMany / Fixed(u32)). Snapshots hand size BEFORE disposal (a naive
      Discard-then-Draw{HandSize} reads 0). Routes Discard through `discard_cards()`
      so Madness → exile is preserved. New `move_zone_all_then_shuffle()` helper reuses
      the `seed_from_u64(timestamp_counter)` pattern. CR 701.9 / 701.24 / 121.1.
- [x] **`Effect::SetNoMaximumHandSize { player }`** + persistent
      `PlayerState.no_max_hand_size_permanent`. The pre-existing `no_max_hand_size` flag
      is **recomputed from the battlefield every cleanup** (`rules/turn_actions.rs`), so
      "for the rest of the game" was inexpressible. Cleanup now ORs in the persistent
      flag. PB-AC8's `calculate_characteristics()` layer-correctness fix in that same
      scan was verified NOT regressed. CR 402.2.
- [x] **Token-doubling completeness pass** — `apply_token_creation_replacement()` was
      wired at only **2 of 13** `GameEvent::TokenCreated` sites. Now 13/13:
      CreateTokenAndAttachSource (Living Weapon), Squad, Offspring, Myriad, Embalm,
      Eternalize, Encore, Gift Food/Treasure (**keyed to the recipient**, not the
      gifting spell's controller — negative-control test proves it), plus
      `Effect::Investigate` and `Effect::Amass`, which the plan's own site table
      **missed entirely** and which were found only by a final grep sweep.
- [x] hash.rs: `Effect::WheelHand` disc 91, `Effect::SetNoMaximumHandSize` disc 92,
      `HashInto` for `WheelDisposal`/`WheelDraw`, `no_max_hand_size_permanent` hashed
      (`hash.rs:1450`), `HASH_SCHEMA_VERSION` 35 → 36, 27 test-file sentinels bulk-updated.
- [x] Card backfill — **11 fully clean** (all markers deleted, grep-verified):
      `incendiary_command` (mode 3), `shattered_perception`, `winds_of_change`,
      `echo_of_eons` (shuffles hand AND graveyard), `ancient_silver_dragon`,
      `parallel_lives`, `anointed_procession`, `doubling_season`,
      `ancient_copper_dragon`, `ancient_gold_dragon`, **`reforge_the_soul`**.
- [x] Review (primitive-impl-reviewer) → `pb-review-AC9.md`: **0 HIGH, 1 MEDIUM, 3 LOW**.
      Reviewer independently re-derived all 13 token sites and MCP-verified all card defs.
- [x] Review (card-batch-reviewer) → `review-pb-ac9-backfill.md`: **1 HIGH, 0 MEDIUM, 1 LOW**.
- [x] All HIGH/MEDIUM fixed (see below).
- [x] Gates: `build --workspace` clean · `test --all` **3062 → 3090 (+28), 0 failed** ·
      `clippy --all-targets -D warnings` clean · `fmt --check` clean.
      Hazard 3 verified: **0** `Effect::` matches in `tools/tui/src` or
      `tools/replay-viewer/src`, so no exhaustive-match arms were needed.
- [x] `authoring-report.py`: clean **973 → 983 (+10)**, 55.7% → **56.2%**.
- [x] `/review` — 4/4 criteria PASS.

## Findings fixed

**E1 (MEDIUM, engine, `effects/mod.rs`)** — `Effect::Amass` placed its +1/+1 counters
with a direct `obj.counters` write, **bypassing `apply_counter_replacement`**. Under
Corpsejack Menace / Doubling Season / Vorinclex, `amass 3` placed 3 counters instead of
6 (CR 701.47a + CR 614.1). Reachable in real play via **Dreadhorde Invasion + Doubling
Season**. Routed through the replacement chokepoint. The regression test was verified
**non-vacuous** (stashed the fix → test fails → restored → passes).

**HIGH (card, `reforge_the_soul.rs`)** — a **stale marker**: the TODO claimed
`KeywordAbility::Miracle` was unimplemented. Miracle has long been implemented
(`rules/miracle.rs`, `Command::ChooseMiracle`, `MiracleTrigger`) and `terminus.rs` /
`temporal_mastery.rs` both use it. The marker, not the engine, kept the card incomplete.
Added the standard dual-def + a regression test asserting no marker remains in the
source. **This upgraded the batch from the planned +9 clean / +1 partial to +10 clean.**

**E2 (LOW)** — added hash-discrimination tests for the `WheelDisposal`/`WheelDraw`
payloads (the mutation test only covered the `PlayerState` field), including a
non-vacuous identical-hash sanity assertion.

**E3 (LOW)** — `next_object_id()` mints `ObjectId(self.timestamp_counter)` after
incrementing, so the object-id counter **is** the timestamp counter. Rewinding
`timestamp_counter` in a test to pin an RNG seed silently aliases new objects onto
existing ones (no panic, just wrong state). Adjudicated as a *test-authoring hazard*,
not an engine bug (the production counter is monotonic). Documented in
`memory/gotchas-infra.md`.

**C1 (LOW)** — reviewer's "docs say Reforge is partial" note was already stale by the
time the review landed; the worker had fixed it in `b9397215`.

## Outcome

**Yield honesty.** Briefed at ~16 cards; **11 clean unblocks** delivered. But only
**2 of the 5 briefed primitives** were real gaps. The single most valuable artifact of
this batch is not a primitive at all — it is the **token-doubling completeness pass**:
Doubling Season / Parallel Lives / Anointed Procession were silently failing on 11 of 13
token-creation paths (Squad, Offspring, Myriad, Embalm, Eternalize, Encore, Living Weapon,
Gift, Investigate, Amass). That is a live correctness bug that no card marker recorded and
no roster would have surfaced.

**Second consecutive stale-marker HIGH.** PB-AC8's card review caught one (Curiosity
Crafter); PB-AC9's caught another (Reforge the Soul), and PB-AC9's *plan* independently
mis-scoped three primitives as absent. This **strongly corroborates PB-AC8's close-out
recommendation**: a campaign-wide stale-marker sweep would recover coverage faster than
another primitive batch. The AC chain is now complete, so that sweep is the natural
next W6 action.

## Open seeds (OOS)

- **OOS-AC9-SEARCHNAME** — `SearchLibrary` multi-name ("a card named A or B", up-to-N with
  different names). Genuinely absent from `TargetFilter`. **Zero card roster today.**
  Build only when a card demands it.
- **OOS-AC9-FILTERMANA** — full 3-way interactive filter-land choice (Mystic Gate:
  `{W}{W}` / `{W}{U}` / `{U}{U}`). `Effect::AddManaFilterChoice` currently produces the
  middle option deterministically. Needs M10 interactive choice; not an AC9 gap.
- **OOS-AC9-ELSPETH** — `elspeth_storm_slayer.rs` grants flying via a live
  `EffectFilter::CreaturesYouControl` filter, but the oracle refers to the fixed set that
  received counters at resolution. Pre-existing approximation, not introduced by AC9.
- **OOS-AC9-AMASSCHOICE** — amass picks the Army with the smallest `ObjectId` as a
  deterministic stand-in for "choose an Army creature you control" (CR 701.47a). Needs
  M10 interactive choice.

## Process note for the next worker

The `/implement-primitive` skill doc still points at **`memory/primitive-wip.md`**, but
PB-AC4 … PB-AC8 all wrote to **`memory/primitives/primitive-wip.md`** (this file). The
AC9 runner followed the stale skill doc and overwrote PB-AC3's close-out in the old path;
that file was restored from `main` and the close-out written here instead. **Fix the skill
doc's path** — it is a live trap.

## Commit prefix

`W6-prim:` (engine) / `W6-cards:` (card defs)

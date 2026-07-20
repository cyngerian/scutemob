# Primitive WIP — IDLE (no PB in progress)

<!-- last_updated: 2026-07-19 -->

**THE PB-OS QUEUE IS COMPLETE** (PB-OS1..OS11 + OS4b, `scutemob-116`..`141`, 2026-07-18/19).
Last collected: **PB-OS11** (`scutemob-141`, merge `bd220b00` — RemoveCounter mana-ability
lowering + batch-filtered-attack trigger; 6 flips incl. 2 backfills; PROTOCOL 25→26 /
HASH 62→63; both seed premises verified stale vs oracle and reframed before building).

**Rider-seed mini-triage DONE** (`scutemob-142`, 2026-07-19). Canonical inventory + ranked queue:
**`memory/primitives/rider-seed-triage-2026-07-19.md`** — read that, not the old 8-seed list.

Triage found 11 OS-series IDs (not 8; **OOS-OS10-1 is a phantom**, **OOS-OS7-3 was never filed**),
restored the dropped **OOS-OS4-1**, and filed **6 new seeds (OOS-RS-1..6), of which these 4 are
correctness-class and outrank every previously-filed seed** — two live on cards currently marked
`Complete` (Invariant #9):

- **OOS-RS-1** — library top/bottom **inverted** between the draw path and the reveal/scry family;
  Scry/`RevealAndRoute`/`LookAtTopThenPlace` read the opposite end from `draw_card`. ~52 files.
- **OOS-RS-2** — **every hybrid/Phyrexian pip in an activated cost is free** (`can_spend`/`spend`
  never read `cost.hybrid`/`cost.phyrexian`); all 7 filter lands are live "{T}: Add two mana" today.
- **OOS-OS9-1** — no card-def `AtBeginningOfCombat` sweep; `helm_of_the_host` is `Complete`,
  passes `validate_deck`, and its only real ability silently never fires.
- **OOS-RS-3** — OOS-OS4-2 is documented CLOSED but has 3 surviving CR 712.8d/e deviations.

**Next dispatch: PB-RS1 (OOS-RS-1), fully specified** in that doc §5 — correctness, no wire bump,
starts with a probe that decides the fix direction. Full ranked queue R1..R11 in §3; ~11-13
discounted flips plus integrity repairs on 10+ already-`Complete` cards.

Also open: older dormant/deferred backlog (`oos-retriage-plan` §1c/§1d), retired-scripts worklist, M10.

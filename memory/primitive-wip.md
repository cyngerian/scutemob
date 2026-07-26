# Primitive WIP — PB-DP8 (DP-6 / OOS-M11-4: triggered-ability targets are auto-selected) · PLAN

<!-- last_updated: 2026-07-26 -->

> Previous occupant: **PB-DP7 (DP-3: cleanup discard has no `Command`) — SHIPPED**
> `scutemob-155`, merge `8f890611`, PROTOCOL 27 → **28**, HASH 64 → **65**, tests **3,837**,
> 0 card-def edits, 0 completeness flips. Its record lives in
> `docs/audits/decision-point-audit.md` §5 DP-3 / §8 / §8.1 (OOS-DP7-1..12),
> `memory/primitives/pb-plan-DP7.md` + `pb-review-DP7.md`, and the CLAUDE.md changelog.
> **`pb-plan-DP7.md` §1 is the spec this PB inherits — read §1.5 first.**

- **PB**: PB-DP8 — **DP-6 / OOS-M11-4** (CR **603.3d**). Triggered-ability targets are
  auto-selected first-match at `crates/engine/src/rules/abilities.rs:7174-7500` across **84**
  effectively-`Complete` defs. **The fallback is CR 603.3d-COMPLIANT** — layer-resolved
  characteristics (`layers::expect_characteristics`), protection/hexproof/shroud honoured
  (`validate_target_protection`), never self-targets a `TargetOpponent` requirement, and removes
  the trigger from the stack when a required slot has no legal candidate. **This is an agency
  fix, not a rules fix.** Preserve the compliant fallback for bot seats and as the
  no-legal-target path.
- **Task**: `scutemob-156`
- **Branch**: `feat/pb-dp8-triggered-ability-target-choice-surface-the-84-def-ag`
- **Class**: AGENCY (Tier 1 top, class **B**). Rank 8 of the PB-DP suite; the suite's **second**
  wire change and the largest single-site agency loss after tutors.
- **Phase**: plan
- **Binding spec**: `docs/audits/decision-point-audit.md`
  - §5 **DP-6 row** (line ~457) — the finding proper, with the cited site
  - §7 **OOS-M11-4** (line ~570) — "CONFIRMED; reclassify B, not D"; this PB closes it
  - §8 **PB-DP8 row** (line ~603) — *"new `Command` + pending state ⇒ PROTOCOL + HASH bump …
    The big one: 84 `Complete` defs. Should follow PB-DP7 so the pending-decision shape is
    already proven"*
  - §8 sequencing note (lines ~606-624) — PB-DP7's UPDATED banner names the two consult sites
    and the block-vs-deadline test
  - §8.1 — where new seeds get filed (**OOS-DP8-N**); note **OOS-DP3-4** (modal *triggered*
    abilities auto-select mode 0) is explicitly flagged "bundle with PB-DP8"
  - §10 — re-audit triggers: a new `Command` fires the DP-24 accepted-and-discarded-field
    check; a new `GameEvent` fires the `reveals_hidden_info` sweep (OOS-DP7-3)
- **Inherited spec**: `memory/primitives/pb-plan-DP7.md` §1 (§1.5 states exactly what DP-8
  inherits and what it must design fresh: **cardinality** — a `Vector` answered as a sequence
  in APNAP order per CR 603.3b — and **partial-flush resumption**, which DP-7 got for free)
- **Plan file**: `memory/primitives/pb-plan-DP8.md`
- **Review file**: `memory/primitives/pb-review-DP8.md`
- **Baseline**: PROTOCOL **28**, HASH **65**, tests **3,837**, coverage 1,139/1,804 = 63.1%

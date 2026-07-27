# Primitive WIP — PB-DP9 (DP-7 / DP-8 / DP-9: search, scry, surveil player choice)

<!-- last_updated: 2026-07-27 -->

> Previous occupant: **PB-DP8 (DP-6 / OOS-M11-4: triggered-ability target choice) — SHIPPED**
> `scutemob-156`, merge `48353a36`, PROTOCOL 28 → **30**, HASH 65 → **67**, tests **3,878**.

- **PB**: PB-DP9 — **DP-7 / DP-8 / DP-9** (CR **701.23** / **701.22a** / **701.25a**).
- **Task**: `scutemob-157`
- **Branch**: `feat/pb-dp9-search-scry-surveil-player-choice-auto-pick-inverts-t`
- **Class**: AGENCY + CORRECTNESS (Tier 1, class **B** ×3). Rank 9 of the PB-DP suite; the
  suite's **third** wire change. Scry and surveil actively **invert** the printed mechanic.
- **Phase**: **plan**
- **Plan**: `memory/primitives/pb-plan-DP9.md`
- **Review file**: `memory/primitives/pb-review-DP9.md`
- **Baseline**: PROTOCOL **30**, HASH **67**, tests **3,878**

## The three findings (audit §5 Tier 1, §4.9)

| finding | CR | site (as of this branch) | defs |
|---|---|---|---|
| **DP-7** library search picks the lowest `ObjectId` — every tutor fetches for you | 701.23 | `effects/mod.rs:3026` (`candidates.iter().min_by_key(\|&&id\| id.0)`) | 74 claimed |
| **DP-8** scry sends **all** cards to the bottom; keep-on-top unreachable | 701.22a | `effects/mod.rs:3072-3098` | 16 claimed |
| **DP-9** surveil sends **all** cards to the graveyard; Surveil N ≡ Mill N | 701.25a | `effects/mod.rs:3101-3132` | 8 claimed |

Roster counts are **claimed**, not verified. Enumerate from `all_cards()` per SR-36 — PB-DP8's
audit-claimed 84 was really 77 and the planner's grep-derived 74 was also wrong.

## The hard problem (audit §8 / `pb-plan-DP7.md` §1.6)

DP-7 and DP-8 pause *between* actions. **PB-DP9 pauses INSIDE `execute_effect`** with an effect
list still to run. It inherits `BlockingDecision`, the `blocking_decision()` predicate, the
`process_command` admission gate, the `enter_step` progress guard, and the
`LegalAction`/`DecisionKind`/`LocalGame` DTO shape — but **not** the resume mechanism. It needs a
resumable effect-list cursor (a `GameState` shape change + a re-entrancy audit of every
`execute_effect` caller). This is the OOS-DP5-5 problem.

## Phase log

- 2026-07-27 — plan phase opened.

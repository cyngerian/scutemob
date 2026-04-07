# Project Status

> **Machine-parseable progress tracker.** Read by the TUI dashboard's Progress tab.
> Updated by `/implement-primitive` (close phase), `/end-session`, and manual edits.
>
> This is the single source of truth for what's done, what's in progress, and what's next.

---

## Primitive Batches

| Batch | Title | Status | Cards Fixed | Cards Remaining | Review | Sessions |
|-------|-------|--------|-------------|-----------------|--------|----------|
| PB-0 | Quick wins (no engine changes) | done | 23 | 0 | fixed | 1 |
| PB-1 | Mana with damage (pain lands) | done | 8 | 0 | fixed | 1 |
| PB-2 | Conditional ETB tapped | done | 56 | 0 | fixed | 3 |
| PB-3 | Shockland ETB (pay-or-tapped) | done | 10 | 0 | clean | 1 |
| PB-4 | Sacrifice as activation cost | done | 26 | 0 | fixed | 2 |
| PB-5 | Targeted activated/triggered abilities | done | 32 | 0 | fixed | 3 |
| PB-6 | Static grant with controller filter | done | 30 | 0 | fixed | 2 |
| PB-7 | Count-based scaling | done | 29 | 0 | fixed | 2 |
| PB-8 | Cost reduction statics | done | 10 | 0 | fixed | 2 |
| PB-9 | Hybrid mana & X costs | done | 7 | 0 | fixed | 2 |
| PB-9.5 | Architecture cleanup | done | 0 | 0 | fixed | 1 |
| PB-10 | Return from zone effects | done | 8 | 0 | fixed | 1 |
| PB-11 | Mana spending restrictions + ETB choice | done | 13 | 0 | fixed | 2 |
| PB-12 | Complex replacement effects | done | 11 | 0 | fixed | 3 |
| PB-13 | Specialized mechanics (10 sub-batches) | done | 19 | 0 | fixed | 4 |
| PB-14 | Planeswalker support + emblems | done | 31 | 0 | fixed | 5 |
| PB-15 | Saga & Class mechanics | done | 3 | 0 | fixed | 2 |
| PB-16 | Meld | done | 1 | 0 | fixed | 1 |
| PB-17 | Library search filters | done | 74 | 0 | fixed | 4 |
| PB-18 | Stax / action restrictions | done | 10 | 0 | fixed | 2 |
| PB-19 | Mass destroy / board wipes | done | 12 | 0 | fixed | 2 |
| PB-20 | Additional combat phases | done | 3 | 0 | fixed | 1 |
| PB-21 | Fight & Bite | done | 4 | 0 | fixed | 1 |
| PB-23 | Controller-filtered creature triggers | done | 34 | ~111 | fixed | 1 |
| PB-24 | Conditional statics ("as long as X") | done | 13 | ~188 | fixed | 1 |
| PB-25 | Continuous effect grants | done | 28 | ~70 | clean | 1 |
| PB-26 | Trigger variants (all remaining) | done | 55 | ~17 | fixed | 1 |
| PB-27 | X-cost spells | done | 15 | ~27 | fixed | 1 |
| PB-28 | CDA / count-based P/T | done | 9 | ~23 | fixed | 1 |
| PB-29 | Cost reduction statics | done | 13 | ~17 | fixed | 1 |
| PB-30 | Combat damage triggers | done | 27 | ~22 | fixed | 1 |
| PB-31 | Cost primitives (RemoveCounter, SacCost) | done | 18 | ~5 | fixed | 1 |
| PB-32 | Static/effect (lands, prevention, ctrl, anim) | done | 22 | ~17 | fixed | 1 |
| PB-33 | Copy/clone + exile/flicker timing | done | 15 | ~24 | fixed | 1 |
| PB-34 | Mana production (filter, devotion, cond.) | done | 7 | ~33 | clean | 1 |
| PB-35 | Modal triggers + graveyard + PW | done | 14 | ~46 | fixed | 1 |
| PB-36 | Evasion/protection extensions | done | 16 | ~5 | fixed | 1 |
| PB-37 | Complex activated (residual) | done | 7 | ~TBD | fixed | 1 |
| PB-C | Extra turns | done | 4 | ~0 | fixed | 1 |
| PB-F | Damage multiplier | done | 3 | ~0 | clean | 1 |
| PB-I | Grant flash | done | 4 | ~0 | fixed | 1 |
| PB-H | Mass reanimate | done | 5 | ~0 | fixed | 1 |
| PB-L | Reveal/X effects | done | 7 | ~0 | fixed | 1 |

**Status values**: `done`, `active`, `planned`
**Review values**: `clean` (reviewed, no issues), `fixed` (reviewed, issues fixed), `none` (not reviewed), `—` (not yet implemented)
**Note**: PB-23+ are gap closure batches. Full plan: `docs/dsl-gap-closure-plan.md`.

---

## Card Health

| Category | Count | Percentage |
|----------|-------|------------|
| Clean (no TODOs) | ~785 | 54% |
| With TODOs (fixable now) | ~88 | 6% |
| With TODOs (still blocked) | ~578 | 40% |
| Not Yet Authored | ~286 | — |
| **Total Universe** | **1743** | — |
| **Total Authored** | **1456** | **84%** |

**Post-BF-1**: 678 files have TODOs (1,070 lines). ~100 fixable now, ~578 blocked.
Re-triage report: `memory/card-authoring/bf1-retriage-report.md`.
As of 2026-03-30 (post PB-37, BF-1 complete).

---

## Workstreams

| # | Name | Status | Last Activity | Next Action |
|---|------|--------|---------------|-------------|
| W1 | Abilities | done | 2026-03-11 | — |
| W2 | TUI & Simulator | stalled | 2026-02-28 | Phase 2: blocker UI, ability targeting |
| W3 | LOW Remediation | active | 2026-03-19 | **W3-LC S2 DONE**. S3 next: fix MEDIUM sites |
| W4 | M10 Networking | not-started | — | Blocked: finish W6 first |
| W5 | Card Authoring | retired | — | Replaced by W6 |
| W6 | Primitive + Card Authoring | active | 2026-03-29 | **Phase 2.5: DSL gap closure COMPLETE (PB-23–37)**. Next: BF-1 re-triage, then resume authoring. |

**Status values**: `done`, `active`, `stalled`, `partial`, `not-started`, `retired`

**W6-review COMPLETE**: All 21/21 reviews done (PB-0 through PB-21).
**NEXT OBJECTIVE**: Phase 2.5 DSL gap closure (PB-23 through PB-37), then resume authoring, then Phase 3 audit.

---

## Path to Alpha

| Milestone | Status | Blocked By | Key Deliverable |
|-----------|--------|------------|-----------------|
| M0-M9 | done | — | Engine core complete |
| M9.5 | done | — | Replay viewer + type consolidation |
| **W6-review** | **done** | **—** | **20/20 retroactive reviews complete** |
| **W6 Phase 1** | **done** | **—** | **ALL 22 primitive batches complete (PB-0 through PB-21)** |
| W6 Phase 2 | blocked | W6 Phase 1 | Author ~1,025 remaining cards |
| W6 Phase 3 | blocked | W6 Phase 2 | Audit: zero TODOs, zero wrong game state |
| M10 | blocked | W6 | Networking: WebSocket server, player choices |
| M11 | blocked | M10 | Game UI: human-playable Commander |
| M12 | blocked | W6 Phase 3 | Card pipeline (downscoped: agent-based) |
| Alpha | blocked | M10 + M11 + W6 | End-to-end networked Commander games |

---

## Engine Test Summary

| Metric | Count |
|--------|-------|
| Total tests | 2531 |
| Test files | 213 |
| Game scripts | 270 |
| Approved scripts | 112 |
| Abilities validated | 194 / 204 |
| Corner cases covered | 32 / 36 |

---

## Deferred Items

Items explicitly deferred from completed PB batches. Must be addressed before Phase 2 authoring.

| Item | Deferred From | Blocked Until | Impact | Status |
|------|---------------|---------------|--------|--------|
| ~~Equipment auto-attach~~ | PB-13d | ~~PB-22 S4~~ | 2 cards | **DONE (S4)** |
| ~~Timing restriction (sorcery-speed activation)~~ | PB-13i | ~~PB-22 S1~~ | 2 cards | **DONE (S1)** |
| Clone/copy ETB choice | PB-13j | dedicated session | 2 cards | Partial (BecomeCopyOf exists, choose-target gap remains) |
| ~~Adventure (split-card from exile)~~ | PB-13m | ~~PB-22 S7~~ | 1 card | **DONE (S7)** |
| ~~Coin flip / d20~~ | PB-13h | ~~PB-22 S2~~ | 2 cards | **DONE (S2)** |
| ~~Flicker (exile + return)~~ | PB-13l | ~~PB-22 S3~~ | 1+ cards | **DONE (S3)** |
| Tiamat multi-card search | PB-17 | M10 (player choice) | 1 card | |
| ~~Goblin Ringleader reveal-route pattern~~ | PB-17 | ~~PB-22 S3~~ | 1 card | **DONE (S3)** |
| ~~Finale of Devastation graveyard search~~ + X pump | PB-17 | ~~PB-22 S7~~ (search done) / dedicated (X pump) | 1 card | **Partial (S7)** — also_search_graveyard done, X-based filter + pump remain |
| Scion of the Ur-Dragon copy-self | PB-17 | EffectTarget::LastSearchResult | 1 card | Partial (BecomeCopyOf exists, needs LastSearchResult) |
| ~~Inventors' Fair activation condition~~ | PB-17 | ~~PB-22 S1~~ | 1 card | **DONE (S1)** |
| Urza's Saga exact mana cost filter | PB-17 | TargetFilter exact_mana_cost field | 1 card | |
| ~~Hanweir attack triggers (tapped-and-attacking tokens)~~ | PB-14 | ~~PB-22 S4~~ | 2 cards | **DONE (S4)** |
| ~~Emblem creation (CR 114)~~ | PB-14 | ~~PB-22 S6~~ | 11 cards | **DONE (S6)** |

---

## Review Backlog (W6-review — PRIMARY OBJECTIVE)

**ALL 20 REVIEWS COMPLETE.** Forward progress unblocked — PB-19 through PB-21 can proceed.

| # | Batch | Title | Cards Fixed | Review Status | Findings |
|---|-------|-------|-------------|---------------|----------|
| 1 | PB-0 | Quick wins | 23 | fixed | 1M 1L fixed |
| 2 | PB-1 | Mana with damage | 8 | fixed | 1M fixed |
| 3 | PB-2 | Conditional ETB tapped | 56 | fixed | 1H 1M fixed; 1M 2L deferred |
| 4 | PB-3 | Shockland ETB | 10 | clean | clean |
| 5 | PB-4 | Sacrifice as activation cost | 26 | fixed | 1M fixed; 1M 13M 6L deferred (other PBs) |
| 6 | PB-5 | Targeted abilities | 32 | fixed | 1H 2M 1L fixed; 5M 5L deferred |
| 7 | PB-6 | Static grant with filter | 30 | fixed | 1H 5M 6L fixed |
| 8 | PB-7 | Count-based scaling | 29 | fixed | 3H 2M fixed; 2L deferred |
| 9 | PB-8 | Cost reduction statics | 10 | fixed | 3M fixed |
| 10 | PB-9 | Hybrid mana & X costs | 7 | fixed | 1H 5M fixed; 2M 7L deferred |
| 11 | PB-9.5 | Architecture cleanup | 0 | fixed | 1M 2L |
| 12 | PB-10 | Return from zone | 8 | fixed | 2H 5M fixed; 3L deferred |
| 13 | PB-11 | Mana restrictions + ETB choice | 13 | fixed | 1H 6M fixed; 1M 7L deferred |
| 14 | PB-12 | Complex replacements | 11 | fixed | 2H 4M fixed; 2M deferred; 2M documented |
| 15 | PB-13 | Specialized mechanics | 19 | fixed | 2H 5M fixed; 9M 1L deferred |
| 16 | PB-14 | Planeswalker support | 31 | fixed | 1H 1M fixed; 2L deferred |
| 17 | PB-15 | Saga & Class | 3 | fixed | 1H 1M fixed; 2M 1L deferred |
| 18 | PB-16 | Meld | 1 | fixed | 1H 1M 1L fixed; 2M deferred (DSL gap) |
| 19 | PB-17 | Library search filters | 74 | fixed | 4H 4M fixed; 1M deferred (DSL gap) |
| 20 | PB-18 | Stax / restrictions | 10 | fixed | 2H 4M fixed; 2M deferred (DSL gap) |

**Review Status values**: `pending`, `in-review`, `needs-fix`, `fixing`, `clean`, `fixed`
**Progress**: 20 / 20 reviewed — ALL REVIEWS COMPLETE

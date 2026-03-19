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
| PB-13 | Specialized mechanics (10 sub-batches) | done | 19 | 0 | none | 4 |
| PB-14 | Planeswalker support + emblems | done | 31 | 0 | none | 5 |
| PB-15 | Saga & Class mechanics | done | 3 | 0 | none | 2 |
| PB-16 | Meld | done | 1 | 0 | none | 1 |
| PB-17 | Library search filters | done | 74 | 0 | none | 4 |
| PB-18 | Stax / action restrictions | done | 10 | 0 | none | 2 |
| PB-19 | Mass destroy / board wipes | planned | 0 | 12 | — | 2 |
| PB-20 | Additional combat phases | planned | 0 | 10 | — | 2 |
| PB-21 | Fight & Bite | planned | 0 | 5 | — | 1 |

**Status values**: `done`, `active`, `planned`
**Review values**: `clean` (reviewed, no issues), `fixed` (reviewed, issues fixed), `none` (not reviewed), `—` (not yet implemented)

---

## Card Health

| Category | Count | Percentage |
|----------|-------|------------|
| Complete (no TODOs, correct game state) | 412 | 57% |
| Has TODOs (compiles but incomplete) | 309 | 43% |
| Wrong Game State (dangerous partials) | 122 | 17% |
| Not Yet Authored | 1025 | — |
| **Total Universe** | **1743** | — |
| **Total Authored** | **721** | **41%** |

---

## Workstreams

| # | Name | Status | Last Activity | Next Action |
|---|------|--------|---------------|-------------|
| W1 | Abilities | done | 2026-03-11 | — |
| W2 | TUI & Simulator | stalled | 2026-02-28 | Phase 2: blocker UI, ability targeting |
| W3 | LOW Remediation | partial | 2026-03-03 | T3: ManaPool encapsulation |
| W4 | M10 Networking | not-started | — | Blocked: finish W6 first |
| W5 | Card Authoring | retired | — | Replaced by W6 |
| W6 | Primitive + Card Authoring | active | 2026-03-16 | **W6-review**: retroactive review of PB-0 through PB-18 |

**Status values**: `done`, `active`, `stalled`, `partial`, `not-started`, `retired`

**PRIMARY OBJECTIVE**: W6-review — retroactive review of all 19 completed PB batches
(PB-0 through PB-18) before any new forward progress. No new PB batches or card authoring
until all reviews are complete and findings fixed.

---

## Path to Alpha

| Milestone | Status | Blocked By | Key Deliverable |
|-----------|--------|------------|-----------------|
| M0-M9 | done | — | Engine core complete |
| M9.5 | done | — | Replay viewer + type consolidation |
| **W6-review** | **active** | **—** | **Retroactive review of PB-0 through PB-18 (PRIMARY)** |
| W6 Phase 1 | blocked | W6-review | 3 remaining PB batches (PB-19 to PB-21) |
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
| Total tests | 2154 |
| Test files | 209 |
| Game scripts | 270 |
| Approved scripts | 112 |
| Abilities validated | 194 / 204 |
| Corner cases covered | 32 / 36 |

---

## Deferred Items

Items explicitly deferred from completed PB batches. Must be addressed before Phase 2 authoring.

| Item | Deferred From | Blocked Until | Impact |
|------|---------------|---------------|--------|
| Equipment auto-attach | PB-13d | PB-18 or dedicated | 2 cards |
| Timing restriction (sorcery-speed activation) | PB-13i | PB-18 or dedicated | 2 cards |
| Clone/copy ETB choice | PB-13j | dedicated session | 2 cards |
| Adventure (split-card from exile) | PB-13m | dedicated session | 1 card |
| Coin flip / d20 | PB-13h | dedicated session | 2 cards |
| Flicker (exile + return) | PB-13l | dedicated session | 1+ cards |
| Tiamat multi-card search | PB-17 | M10 (player choice) | 1 card |
| Goblin Ringleader reveal-route pattern | PB-17 | dedicated primitive | 1 card |
| Finale of Devastation graveyard search + X pump | PB-17 | dedicated session | 1 card |
| Scion of the Ur-Dragon copy-self | PB-17 | copy subsystem | 1 card |
| Inventors' Fair activation condition | PB-17 | Condition variant | 1 card |
| Hanweir attack triggers (tapped-and-attacking tokens) | PB-14 | combat token primitive | 2 cards |

---

## Review Backlog (W6-review — PRIMARY OBJECTIVE)

**All 19 completed PB batches must be reviewed before forward progress resumes.**
Use `/implement-primitive --review-only PB-<N>` for each batch.
Order: sequential (PB-0 first, PB-18 last). Earlier batches are foundational.

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
| 15 | PB-13 | Specialized mechanics | 19 | in-review | — |
| 16 | PB-14 | Planeswalker support | 31 | pending | — |
| 17 | PB-15 | Saga & Class | 3 | pending | — |
| 18 | PB-16 | Meld | 1 | pending | — |
| 19 | PB-17 | Library search filters | 74 | pending | — |
| 20 | PB-18 | Stax / restrictions | 10 | pending | — |

**Review Status values**: `pending`, `in-review`, `needs-fix`, `fixing`, `clean`, `fixed`
**Progress**: 14 / 20 reviewed

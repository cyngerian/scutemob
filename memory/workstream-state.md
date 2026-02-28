# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | Batch 0: P3 stragglers (Overload, Bolster, Adapt, Partner With) | ACTIVE | 2026-02-28 | Hideaway already closed; working remaining 4 |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | Phase 0 complete; T2 done; Phase 1 (abilities) next |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | available | — | Phase 9 ready; top Tier 2 by deck count |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred)

## Last Handoff

**Date**: 2026-02-28 (night)
**Workstream**: W2: TUI & Simulator
**Task**: 6 UX fixes for play mode (scrolling, zones, discard)
**Completed**:
- Fix 1: Hand list scrolling — auto-scroll with skip/take in hand_view.rs
- Fix 2: Discard events — DiscardedToHandSize + CardDiscarded in event log with tan color
- Fix 3: Zone counters — sidebar widened to 22, 2-line per player with H/L/G/E counts
- Fix 4: Zone browser overlay — new zone_browser.rs, g/x keys, scrollable card list
- Fix 5: CardDetail return-to — struct variant with return_to field; Esc=Normal, Space=return
- Fix 6: Action menu hints — [g]rave [x]ile added
- Bug fix: Esc from CardDetail now always exits to Normal (was returning to ZoneBrowser causing apparent freeze)
**Next**: Test the Esc fix in live play; consider adding `p` passthrough from overlays; visual test hand scrolling with 8+ cards
**Hazards**: Pre-existing clippy warnings in engine crate (blood_artist.rs, goblin_recruiter.rs needless_update) — not from this session
**Commit prefix used**: `W2:` (all changes included in f9f7c45)

## Handoff History

### 2026-02-28 (evening) — W5: Card Authoring (setup)
- Added W5 as workstream; worklist confirmed (1,061 ready); commit prefix `W5-cards:`

### 2026-02-28 (late) — W2: Card Pipeline Phases 5-9
- Split definitions.rs → 112 files in defs/; build.rs auto-discovery; skeleton generator; agent rewrite; commit `f9f7c45`

### 2026-02-28 — W2: Overview layout + Card DSL detail pane
- Overview bottom row: 3-column layout; Card DSL parser + detail pane; commit `8d78f7b`

### 2026-02-28 — W3: T2 dead code removal + Phase 0 complete
- MR-M1-14, MR-M9.5-08, MR-M9.4-11 closed; commit `7d535ec`; Phase 0 complete

### 2026-02-28 — W3: T1 tests (14 total)
- MR-M1-19/20, MR-M2-07/08/17, MR-M4-13, MR-M5-08, MR-M6-08, MR-M8-15, MR-M9-14/15, MR-M9.4-13/14/15; commit `320b77f`

# Workstream State

> Coordination file for sessions: the Active Claims table plus the LAST handoff only, capped at
> 60 lines (`docs/course-correction-2026-09.md` §3.1 item 3). Older handoffs are rotated verbatim
> to `memory/archive/workstream-state-<date>.md` (latest: `workstream-state-2026-09-14.md`).
> Per-batch narrative lives in `CHANGELOG.md` + `memory/primitives/pb-<id>-execution-notes.md`.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | All abilities done (B16 Dungeon + Ring last) |
| W2: TUI & Simulator | — | available | — | Phase 1 done; hardening pending |
| W3: LOW Remediation | — | available | — | LOW Sweep COMPLETE 2026-05-16; 6 LOWs remain, deferred |
| W4: M10 Networking | — | not-started | — | P3 of the course correction; after hot-seat |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6 |
| W6: Primitive + Card Authoring | — | available | — | v4 queue CLOSED at rank 21. CC batch + LL-3/LL-1 COMPLETE; data sync `scutemob-259` (`5ce2795a`) 2026-09-14. STOP: CC-9 (`-245`) awaits owner approval; LL-2 (`-256`) after (baseline: `data-freshness.py cites`). LL-4 (`-258`) blocked on CC-5 (owner). History: `CHANGELOG.md` |

## Last Handoff (primary session, 2026-10-12) — play surface: mockups → the alpha UI (v3 brief)

**Date**: 2026-10-12 (primary; self-assigned `/task` → `/done` `scutemob-265`; merges `f6dbc355`, `475100fe`)
**Workstream**: play client (`tools/play-server`), no engine semantics touched

**Completed**: owner judged v2 (B1/B2, `?v2`) "busted" — found to be at 150 % browser zoom; owner now
plays at 100 % (2560×1340). Five static layout mockups, then mockup 7 iterated ~30 rated rounds to
"the alpha UI" (`tools/play-server/mockups/07-table6.html`; snapshots + tags `mockup-7-readable` /
`-zones` / `-combat`). NEW `docs/play-surface-v3-brief.md` (owner-approved): settled rules, four
must-land additions (drag-arrow targeting, card right-click menus incl. counters, drag-rearrange
hand/battlefield, summoning-sick outline), phases C1–C3 = `scutemob-266/267/268` (backlog), server
gaps (per-player exile, commander tax, revealed cards, image proxy, pass-until-step), §7 dispatch
notes for C1. `frontend/src/lib/v3/layout.js` is a starting helper. CHANGELOG entry written. After the
ESM close the owner added brief rules 3 + 14 (mirrored opponent cells; greyed focused cell that lifts
while targetable; one-second hover peek, not over cards) — mockup + snapshot tag refreshed; C1 must include them.

**Not done / deferred**: the v3 build itself (C1–C3) — to be DISPATCHED to workers next session,
not built inline (owner, 2026-10-12). `cockatrice*.png` at the repo root are the owner's untracked
reference shots (6 MB; not committed). Mockup server `python3 tools/play-server/mockups/serve.py 3043`
may still be running; owner's game server on 3040 (seed 3) too.

**Next session candidates**: (1) `/dispatch` C1 (`scutemob-266`) from brief §4/§7 — single worker,
UI change class; (2) C2 (`-267`) after C1 collects; (3) C3 (`-268`) incl. the server gaps; then the
owner playtest. CC-9 hot-seat (`-245`) still awaits owner approval; design C1 with seats in mind.

**Operator-delta line**: nothing playable yet — the alpha table exists only as a static mockup
(`http://127.0.0.1:3043/07-table6.html`); v2 on `?v2` unchanged. Pod-facing item C1 is at the front.

**Hazards**: ask the owner's browser zoom before judging any layout verdict; design fluid, measure at
2560×1340 AND 1707×880; a cached `common.css` hid a fix for a whole round — serve mockups with
`serve.py` (no-store). Scryfall rate-limits the probe IP (429 → ORB-blocked images): fetch once into
`mockups/img/` (gitignored). Never `pgrep -f` a string that matches the invoking shell.
CLAUDE.md is 249/250.

**Commit prefix used**: `scutemob-265:` / `scutemob-266:` / `merge:` / `chore:`

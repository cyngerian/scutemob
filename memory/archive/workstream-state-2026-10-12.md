# Workstream-state handoffs archived 2026-10-12

## Last Handoff (primary session, 2026-09-14) — data sync + `/start` freshness check

**Date**: 2026-09-14 (primary, self-assigned `/task` → `/done`; merge `5ce2795a`)
**Workstream**: cross-cutting infra (`scutemob-259`), no engine semantics touched

**Completed**: owner found the local CR seven months stale. Synced everything to current: CR
effective 2026-08-07, Scryfall 2026-09-14, `cards.sqlite` rebuilt, SR-37 fixture regenerated.
NEW `tools/data-freshness.py check|refresh|cites` + `tools/start-check.sh`, run by a scutemob-only
SessionStart hook (`.claude/settings.json`; the start-skill route was withdrawn, see Hazards). `scryfall-import` moved to gzipped JSON-Lines (the API
had changed; +4 tests); `mtg-mcp-server --import-only`. CR 722–732 renumbered +1 (new 722
Preparation Cards): 378 cites in 60 files swept, pure shift verified. New Scryfall layout
`front_card` shadowed Savage Lands — excluded in fixture script, MCP lookup, both skeleton
generators. Monster Manual gained the printed Book subtype. Suite 5,346 → 5,350 / 0 / 6; wire 44/85
unchanged. Notes: `memory/data-sync-2026-09-14.md`.

**Not done / deferred**: 33 dangling CR numbers / 307 cites (exist in neither CR) → LL-2 (`-256`,
commented). `memory/` still cites old 722–732 numbers (93 / 34 files; historical). Rules pass over
reworded CR 603.10a (sacrifice look-back), 605.1a (library-moving ≠ mana ability), 714.3 (Saga
intrinsic replacement) not filed as seeds — read the notes file before filing.

**Next session candidates**: (1) CC-9 hot-seat (`scutemob-245`) — pod-facing, AWAITS OWNER
APPROVAL; (2) LL-2 (`-256`) after it, starting from `data-freshness.py cites`; (3) CC-5 decklists
(owner) → LL-4, CC-6/7/8.

**Operator-delta line**: nothing — infra session; Monster Manual's Book subtype is deck-builder
visible but the card is `inert`. Second consecutive empty entry is NOT reached (LL-1 was non-empty);
CC-9 is the pod-facing item and is already at the front.

**Hazards**: a scutemob-only SessionStart hook (`.claude/settings.json`) runs `tools/start-check.sh
--hook`; a STALE result must be reported and the refresh OFFERED, never run unprompted. Do NOT put
this in the start skill: `~/.claude/skills/start` symlinks the ESM template and personal skills
shadow project ones (owner: scutemob only). After a CR refresh run `cites`. CLAUDE.md is 249/250.

**Commit prefix used**: `scutemob-259:` / `merge:` / `chore:`

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

## Last Handoff (primary session, 2026-10-12) — play surface v3 C1 collected: the alpha table is live on `?v3`

**Date**: 2026-10-12 (primary; dispatched `scutemob-266`, collected; merge `5025a70a`, pushed)
**Workstream**: play client (`tools/play-server`), no engine / wire change (PROTOCOL 44 / HASH 85)

**Completed**: data refresh first (`e5382c5a`: CR 2026-09-25 + Scryfall 2026-10-12; SR-37 fixture
4 wording tweaks; `cites` 33 / 307 dangling — unchanged baseline). Then C1: mockup 7 built as the
third surface behind `?v3` (`frontend/src/lib/v3/`, 25 files) incl. the three late brief rules
(mirrored cells, greyed focused cell + lift, peek); `?card-window` pop-out; v2 `PromptPanel` hosted
unmodified in the decision panel; NEW `probes/v3sweep.py` + `v3interact.py` (0 scroll / 0 errors,
35/35); three source gates with executed defeats; tests 5,352 → 5,355. Opus review: 1 HIGH + 3
MEDIUM fixed in-cycle, 4 LOW logged. Full record: `memory/play-v3-c1-2026-10-12.md`; CHANGELOG entry.
Brief rule 7 amended (card window fed by a `storage` event, not `BroadcastChannel`).

**Not done / deferred**: C2 (`-267`) arrows, card context menus, blocker arrows, auto-pass panel,
summoning-sick outline, drag-rearrange; C3 (`-268`) server gaps (per-player exile, commander tax,
image proxy — Scryfall rate-limits the probes, pass-until-step), game-over run, owner playtest.
Portfolio session's suggestion recorded on `-268`: a playtest friction log (hotkey → note + state
snapshot). Owner to rule: ride with C3 or its own small task before the first playtest.
`cockatrice*.png` at the repo root are still the owner's untracked reference shots (6 MB).

**Next session candidates**: (1) owner looks at `?v3` on 3040 (the six-seat side-cell card size
~34px is a LOW worth an owner look before C2); (2) `/dispatch` C2 (`-267`) — single worker, change
class row 3 (+ row 4 per gate); (3) C3 (`-268`) incl. the friction log if ruled in. CC-9 hot-seat
(`-245`) still awaits owner approval. Every dispatch needs explicit owner approval.

**Operator-delta line**: the alpha table is playable through priority on `?v3` (pass, pass-until,
card-item actions, every existing picker); targeting is still by decision box until C2.

**Hazards**: one play-server per game — probes on 3041/3042, never the owner's 3040; Scryfall 429s
on back-to-back probes (chips are the fallback; C3 proxy fixes it); the frontend gate bans
`.postMessage(` — use the storage-event transport; v3 page root is 13px (pickers `zoom: 1.23`).
CLAUDE.md is 249/250.

**Commit prefix used**: `scutemob-266:` / `merge:` / `chore:`

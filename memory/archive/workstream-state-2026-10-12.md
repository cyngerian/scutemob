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

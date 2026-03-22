# Workstream State

> Coordination file for parallel sessions. Read by `/start-session`, claimed by
> `/start-work`, released by `/end-session`. This file is the source of truth for
> which workstreams are actively being worked on.
>
> **Protocol**: Read before starting. Claim before coding. Release when done.

## Active Claims

| Workstream | Task | Status | Claimed | Notes |
|------------|------|--------|---------|-------|
| W1: Abilities | — | available | — | B16 complete (Dungeon + Ring); all abilities done |
| W2: TUI & Simulator | — | available | — | Phase 1 done; 6 UX fixes done; hardening pending |
| W3: LOW Remediation | — | available | — | **W3 LOW sprint DONE** (S1-S6): 83→29 open (119 closed total). TC-21 done. 2233 tests. |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | — | available | — | **Tier 1 COMPLETE** (A-01 through A-10 + A-31/A-37). Next: Tier 2 A-11 removal-destroy. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-22
**Workstream**: W6: Primitive + Card Authoring
**Task**: Phase 2 authoring — A-05 through A-10 (Tier 1 completion)

**Completed**:
- A-05 cost-reduction: 12 new cards + 5 DSL extensions (SpellCostFilter::HasColor, SpellCostFilter::InstantOrSorcery, SelfCostReduction::PerOpponent, LifeLostFromStarting, ConditionalPowerThreshold) — all with casting.rs enforcement
- A-06 scry-surveil: 7 new cards (Faerie Seer, Doom Whisperer, Woe Strider, Umbral Collar Zealot, Retreat to Coralhelm, Aqueous Form, Hermes)
- A-07 lifegain: 5 new cards (Jaddi Offshoot, Courser of Kruphix, Nadier's Nightblade, Bontu's Monument, Bloodchief Ascension)
- A-08 lifedrain: 6 new cards (Marauding Blight-Priest, Blood Seeker, Scrawling Crawler, Torment of Hailfire, Blind Obedience, Crypt Ghast)
- A-09 protection: 1 new card (Teferi's Protection)
- A-10 aura: 5 new + 1 existed (Wild Growth, Elvish Guidance, Animate Dead, Kasmina's Transmutation, Eaten by Piranhas)
- All cards reviewed (card-batch-reviewer agents), 8H+9M+10L findings — all HIGH fixed
- **Tier 1 COMPLETE**: A-01 through A-10 + A-31/A-37 pre-existing = 12 groups done
- Commits: d6634d6 (A-05), 2097db8 (A-06–A-10), 4d0999e (review fixes), 034385a (wave progress)
- 2281 tests, 0 clippy, clean workspace build

**Next**:
1. **A-11 removal-destroy** (48 cards, 4 sessions) — first Tier 2 group
2. Continue Tier 2 (A-11 through A-28), then Tier 3 (A-29 through A-42)
3. Direct authoring preferred for complex cards; agents for simple patterns

**Hazards**:
- Same systemic hazards (beast_within/generous_gift/swan_song controller_override, Shizo has_supertype, MDFC back faces)
- Direct authoring is faster for complex cards. bulk-card-author stalls ~50% on DSL research.
- For groups >10 cards, use bulk-card-author with explicit DSL snippets in prompt
- Bontu's Monument, Blood Seeker: compound SpellCostFilter (HasColor+HasCardType) DSL gap
- "that player" trigger target reference: DSL gap (Blood Seeker, Scrawling Crawler)
- WheneverYouCastSpell lacks spell type filter (noncreature, creature) — DSL gap

**Commit prefix used**: `W6-cards:`

## Handoff History

### 2026-03-22 — W6: Phase 2 authoring A-01 through A-04
- 52 new card defs authored (16 mana-creature + 33 mana-artifact + 3 mana-other).
- 5 groups verified pre-existing (body-only, land-etb-tapped, combat-keyword, mana-land).
- Commits: 10f81c0, 0903f6b. 2281 tests.

### 2026-03-22 — W6: Phase 1 close (F-4 sweep + F-5/F-6/F-7)
- 3 stale TODOs cleaned, 8 cards verified (7/8 PASS), build clean, committed 3bfe888.
- Phase 1 Fix COMPLETE. Next: Phase 2 authoring A-01.

### 2026-03-22 — W6: F-4 session 6 (11 now-expressible cards)
- 3 lands (ETB tapped + mana), 1 conditional mana, 1 conditional ETB, 1 equipment bounce, 1 creature ETB+mana, 1 creature ETB, 3 keyword additions. Review: 1M+1L fixed.
- 2281 tests.

### 2026-03-22 — W6: F-4 session 5 (12 land mana abilities)
- 8 pathway lands (mana tap) + 4 verge lands (conditional mana). Review: 2H fixed (stale oracle text).
- Commit: 8c2aded. 2281 tests.

### 2026-03-22 — W6: F-4 session 4 (18 card abilities)
- 10 new implementations + 6 prior unstaged + 2 stale cleanups. Review: 1H+1M fixed.
- Commit: e4cd042. 2281 tests.

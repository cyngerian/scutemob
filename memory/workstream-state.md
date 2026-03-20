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
| W3: LOW Remediation | — | available | — | **W3 LOW sprint DONE** (S1-S4 + S6): 50 LOWs closed (83→33 open). TC-21 done. 2233 tests. |
| W4: M10 Networking | — | not-started | — | After W1 completes |
| W5: Card Authoring | — | **RETIRED** | — | Replaced by W6. See `docs/primitive-card-plan.md` |
| W6: Primitive + Card Authoring | — | available | — | **ALL PBs DONE (PB-0 through PB-21)**. Phase 2 card authoring next |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-03-20
**Workstream**: W3: LOW Remediation
**Task**: S6: TC-21 PendingTrigger Option field migration to TriggerData

**Completed**:
- Migrated 19 Option fields from `PendingTrigger` struct into `TriggerData` variants
  (madness, miracle, modular, evolve, suspend, hideaway, partner_with, graft, backup,
  champion_etb, champion_ltb, soulbond, squad, gift, encore_activator, provoke)
- Removed fields from struct definition and `blank()` constructor in `stubs.rs`
- Removed 487 `fieldname: None,` lines from abilities.rs, turn_actions.rs, replacement.rs, resolution.rs
- Updated `hash.rs` to remove stale hash lines for removed fields
- Fixed accidental removal of `champion_exiled_card`/`gift_opponent` from `GameObject` struct literals in resolution.rs (6 token construction sites)
- Updated `partner_with.rs` test to check `&trigger.data` instead of `trigger.partner_with_name`
- MR-TC-21 closed, MR-M6-06 deferred (high refactor risk per remediation doc)
- 2233 tests, 0 clippy warnings
- Commit: 7e474d2

**Next**:
1. W3 is complete — TC-21 done, all HIGH/MEDIUM resolved, remaining ~33 LOWs are deferred
2. Top priority: W6 Phase 2 card authoring (~1,025 remaining cards)
3. `docs/workstream-coordination.md` Phase 3 checkboxes may need updating

**Hazards**:
- `flanking_blocker_id`, `rampage_n`, `renown_n`, `poisonous_n`, `poisonous_target_player`, `enlist_enlisted_creature`, `recover_cost`, `recover_card`, `ingest_target_player` still remain as Option fields on PendingTrigger — not migrated (these are specific named PTK variants, not in the session plan)
- ~30 remaining call sites still use `casting::can_pay_cost`/`casting::pay_cost` wrappers — functional but could be migrated to direct method calls opportunistically

**Commit prefix used**: `W3:`

## Handoff History

### 2026-03-19 — W6: PB-21 Fight & Bite
- PB-21 complete: Effect::Fight + Effect::Bite, 4 card defs fixed, 14 new tests, 2206 total
- ALL 22 PRIMITIVE BATCHES COMPLETE (PB-0 through PB-21)
- Next: Phase 2 card authoring (~1,025 remaining cards)

### 2026-03-19 — Exploratory: Rules audit + W3-LC plan
- Audited 4 abilities (Devoid, Flanking, Fabricate, Suspend) against CR rules
- Found 1 MEDIUM bug: Flanking reads base characteristics instead of layer-resolved (Humility breaks it)
- Identified 69 base-characteristic reads across 12 engine files that may have same issue
- Created `memory/w3-layer-audit.md` — 4-session audit plan (W3-LC S1-S4)
- Updated `docs/workstream-coordination.md` Phase 3 with W3-LC checkboxes
- **W3 and W6 are independent** — can run in parallel with no conflicts
- No code changes, no commit needed

### 2026-03-19 — W6: PB-18 review (FINAL)
- PB-18 retroactive review (20/20): Stax / restrictions, 10 cards — 2H 4M 1L fixed; 2M deferred (DSL gap)
- Engine: attack tax (combat.rs), mana ability restrictions (mana.rs), zone scope (abilities.rs), simulator filter
- Commit: ca13879
- **ALL RETROACTIVE REVIEWS COMPLETE**

### 2026-03-19 — W6: PB-17 review
- PB-17 retroactive review (19/20): Library search filters, 27 card defs — 4H 4M fixed; 1M deferred (DSL gap)
- Engine: shuffle_before_placing on SearchLibrary, CR 701.19→701.23 citations
- Commit: b72b0bd

### 2026-03-19 — W6: PB-16 review
- PB-16 retroactive review (18/20): Meld, 1 card — 1H 1M 1L fixed; 2M deferred (DSL gap)
- Commit: 6fce74c



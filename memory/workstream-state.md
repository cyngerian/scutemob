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
| W6: Primitive + Card Authoring | — | available | — | PB-S planned 2026-04-11; A-42 Tier 1 reclassified → PB-X; ready for PB-S implement next session |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-11
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-S plan + A-42 Tier 1 blocked reclassification

**Completed**:
- Attempted A-42 Tier 1 authoring (8 cards); 2 parallel `bulk-card-author` runs spun on DSL-gap research, wrote 0 files
- Diagnosed the blocker: 2026-04-10 re-triage verified individual filter fields but didn't trace the full primitive chain (effect → filter → layer → cost). Gaps found:
  - `EffectFilter::AllCreaturesExcludingSubtype` missing (blocks Crippling Fear, Eyeblight Massacre, Olivia's Wrath)
  - `LayerModification::ModifyBoth` takes `i32`, not `EffectAmount` — no dynamic P/T (blocks Olivia's Wrath)
  - `Cost::ExileSelf` missing (blocks Balthor the Defiled)
  - No `TapNCreatures` cost variant (blocks Heritage Druid — deferred to larger cost-framework PB)
  - Metallic Mimic "is the chosen type in addition" not verified (needs type-adding layer check)
- Reclassified 6 of 8 Tier 1 cards → new **PB-X** micro-PB bucket
- Updated `memory/card-authoring/a42-retriage-2026-04-10.md` with 2026-04-11 reclassification table
- Added PB-S + PB-X rows to `docs/project-status.md`
- Saved auto-memory `feedback_retriage_verification.md` (re-triage must trace full primitive chain; flag unverified as "Tier 1 (verify)")
- **PB-S plan written**: `memory/primitives/pb-plan-S.md` — GrantActivatedAbility via Layer 6 LayerModification::AddManaAbility + AddActivatedAbility, ~70 LOC engine, ~60 LOC card defs, ~200 LOC tests; unblocks Cryptolith Rite, Chromatic Lantern, Citanul Hierophants, Paradise Mantle, Enduring Vitality (5 full) + Song of Freyalise, Umbral Mantle (2 partial, other blockers remain); scope boundary: NOT Marvin's reflection pattern
- `memory/primitive-wip.md` → phase=plan, steps 1-4 checked, step 5 is "do not implement this session"

**Next session**:
1. `/implement-primitive` → implement phase for PB-S (runner executes plan)
2. After PB-S: plan + implement PB-X (micro — unblocks A-42 Tier 1 authoring)
3. Author A-42 Tier 1 once PB-X lands
4. Then PB-Q (ChooseColor), PB-R, etc. per revised slate

**Open questions flagged by PB-S planner** (resolve before implement):
1. Does `chars.abilities: Vector<AbilityInstance>` need parallel population, or only specialized vecs? (Planner recommends specialized only.)
2. Face-down creature + grant interaction test needed?
3. Hash version bump policy?
4. Include `mana_solver.rs` calc-chars fix in PB-S, or defer as LOW? (Planner recommends defer.)

**Hazards** (carried forward):
- Re-triage discipline: verify the full primitive chain, not single fields (see `feedback_retriage_verification.md`)
- PB-M deferred items: Isshin attack trigger doubling, Delney power-filtered doubling, Elesh Norn opponent ETB suppression, Drivnod activated ability
- Complete the Circuit: delayed copy trigger still TODO
- Forbidden Orchard: TargetPlayer → TargetOpponent (deferred to M10)
- Heritage Druid `TapNCreatures` cost — own PB, not in PB-X scope

**Commit prefix**: `W6-prim:` (primitive planning)

### 2026-04-10 — W6: A-42 re-triage + Tier 4 diagnosis (research-only)

**Completed**:
- A-42 re-triage: `memory/card-authoring/a42-retriage-2026-04-10.md` — corrected missing count 39→29 (filename heuristic missed 10 already-authored), verified 4 open DSL questions against source, revised tiering
- Tier 4 diagnosis: `memory/card-authoring/a42-tier4-diagnosis-2026-04-10.md` — diagnosed all 10 Tier 4 cards, re-bucketed (4a=0, 4b=8, 4c=2), identified shared gaps, sized each PB
- Angrath's Marauders verified correct (FromControllerSources filter is appropriate for "source you control")
- No code changes this session — pure research

**Key findings**:
- **PB-S (GrantActivatedAbility) is the highest-yield engine work in the entire codebase**: unblocks 8+ cards (Marvin + Cryptolith Rite, Citanul Hierophants, Chromatic Lantern, Paradise Mantle, Umbral Mantle, Enduring Vitality, Song of Freyalise)
- **PB-Q (ChooseColor) unblocks 9+ cards** across codebase, not just the 3 A-42 Tier 3 cards
- **Tier 4c collapsed from 10 cards to 2** (Patriarch's Bidding, Breach the Multiverse) — most of Tier 4 is cheap
- **Cheapest micro-PB**: PB-R ExchangeZones at ~60 LOC, unblocks Morality Shift + Time Spiral (partial) + Winds of Change + Timetwister

**Revised Tier 1 (8 cards, 0 engine work, ready to author)**:
Crippling Fear, Metallic Mimic, Obelisk of Urd, City on Fire, Eyeblight Massacre, Olivia's Wrath, Heritage Druid, Balthor the Defiled

**Next session** (priority order):
1. Author Tier 1 (8 cards) via `/author-wave` or direct — cheapest yield
2. **PB-S: GrantActivatedAbility** (~150-200 LOC) — highest total unblock
3. **PB-Q: ChooseColor** (medium scope) — second highest
4. **PB-R: ExchangeZones + ShuffleZonesIntoLibrary** (~60 LOC) — cheapest next engine work
5. **PB-T: Up-to-N targeting** (~100 LOC) — generic unblock
6. **PB-U: Trigger extensions** (Treasure Nabber, Ghyrson Starn, Roaming Throne, ~75 LOC)
7. **PB-V: DoubleCountersOnTarget** (~40 LOC) — combine with PB-T
8. **PB-W: Text-changing effects** (~100 LOC) — lowest yield, defer
9. Tier 4c deterministic fallbacks (Patriarch's Bidding, Breach the Multiverse) or defer to M10

**Hazards** (carried forward from prior sessions):
- `activated_ability_cost_reductions` index on channel lands may be off-by-one
- Cavern of Souls "can't be countered" deferred (needs CounterRestriction)
- Pitch-alt-costs (Force of Negation/Vigor) still blocked
- Forbidden Orchard: TargetPlayer should be TargetOpponent (deferred to M10)
- PB-M deferred: Isshin attack trigger doubling, Delney power-filtered doubling, Elesh Norn opponent ETB suppression, Drivnod activated ability
- Complete the Circuit: delayed copy trigger still TODO

**Commit prefix**: `W6-cards:` (authoring) or `W6-prim:` (engine)

## Handoff History

### 2026-04-10 — W6: PB-J + PB-M
- PB-J: CopySpellOnStack, ChangeTargets (CR 115.7a/d). 3 card fixes. 9 tests.
- PB-M: Panharmonicon trigger doubling (2 bug fixes, 2 new filters, 1 new card, 3 fixes, 5 tests). All HIGH batches complete. 2589 tests.

### 2026-04-09 — W6: PB-A + PB-B + PB-E
- PB-A: play from top of library. PB-B: play from graveyard. PB-E: mana doubling. 2575 tests.

### 2026-04-07 — W6: PB-A + PB-H + PB-L
- PB-A: play from top of library. PB-H: mass reanimate. PB-L: reveal/X effects. 2549 tests.

### 2026-04-06 — W6: PB-C + PB-F + PB-I
- PB-C: ExtraTurn + self_exile/self_shuffle. PB-F: TripleDamage, DamageTargetFilter. PB-I: FlashGrant, OpponentsCanOnlyCastAtSorcerySpeed. 2504 tests.

### 2026-04-04 — W6: PB-K + PB-D
- PB-K: land drops, Case mechanic. PB-D: chosen creature type, 8 fixes. 2474 tests.

### 2026-04-02 — W6: PB-N + PB-G
- PB-N: 19 misc card defs. PB-G: BounceAll + TargetFilter extensions + 4 cards. 2445 tests.

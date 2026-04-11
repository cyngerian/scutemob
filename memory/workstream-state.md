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
| W6: Primitive + Card Authoring | PB-X close + PB-Q plan | ACTIVE | 2026-04-11 | PB-X CLOSED (tracking files updated). Next in session: PB-Q (ChooseColor) plan phase via `/implement-primitive` |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-11 (third session of the day)
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-X plan → implement → review → fix (close phase deferred to next session)

**Completed**:
- **Plan phase** (Opus planner): `memory/primitives/pb-plan-X.md`. Held scope to 3 primitives, stop-and-flagged on Metallic Mimic (needs 4th primitive → parked as PB-Y), caught that Obelisk of Urd + City on Fire were already authorable (free wins from plan-phase full-chain verification), 5 open questions resolved by oversight before implement green-light.
- **Implement phase** (Sonnet runner): 3 primitives landed — `EffectFilter::AllCreaturesExcludingSubtype` + `AllCreaturesExcludingChosenSubtype` (disc 32/33), `LayerModification::ModifyBothDynamic { amount, negate: bool }` (disc 25, substituted at `Effect::ApplyContinuousEffect` per CR 608.2h — new variant, not migration; 76 existing ModifyBoth call sites untouched), `Cost::ExileSelf` + `ActivationCost.exile_self: bool` (disc 10, LKI via existing `embedded_effect` plumbing). Hash schema version bumped 1→2. All 6 Tier 1 cards authored fully (Crippling Fear, Eyeblight Massacre, Olivia's Wrath, Balthor, Obelisk, City on Fire). Balthor's `ReturnAllFromGraveyardToBattlefield` filter resolved affirmatively — filter supports per-player + color-filtered reanimate, no PB-Z parking needed.
- **Review phase** (Opus reviewer): 7 findings (1 HIGH, 3 MEDIUM, 3 LOW). C1 HIGH — Obelisk authored `ChooseCreatureType` as `TriggerCondition::WhenEntersBattlefield` instead of `ReplacementTrigger::WouldEnterBattlefield`; violated CR 614.12 and diverged from the in-codebase precedent (Urza's Incubator, Vanquisher's Banner, Morophon, Patchwork Banner, ~10 others). Observable bug: pump inactive during the trigger-resolution window. STOP-AND-FLAGGED to oversight before fix cycle per session discipline.
- **Fix phase** (Sonnet runner, sequential pass discipline per oversight): Pass 1 — C1 alone, full gates green in isolation (Obelisk rewritten to `AbilityDefinition::Replacement` mirroring Urza's Incubator pattern). Pass 2 — bundled E1 (10 CR citation rewrites: "701.10" → "118.12 + 406 + 602.2c"; 701.10 is actually "Double"), C2 (Balthor activated end-to-end test via `Command::ActivateAbility`), C3 (3 tests: `test_obelisk_of_urd_chosen_type_pump` with anti-C1 observability window shape, `test_city_on_fire_triples_damage`, `test_city_on_fire_does_not_triple_opponent_sources`), E2/E3 LOWs (doc expansion + dead capture cleanup). E4/C4 LOWs skipped per review guidance.
- **Standing rule established** (oversight): every new `LayerModification` variant must ship with at least one full-dispatch test (not just substitution unit test). Saved in `memory/conventions.md`.
- **Standing rule established** (oversight): "As ~ enters the battlefield, choose X" is a replacement effect per CR 614.12, NOT a triggered ability. Saved in `memory/gotchas-rules.md`.

**Test count**: 2600 → 2612 (implement, +12) → 2616 (fix, +4). All gates green after each phase.

**Cards unblocked**: 6 A-42 Tier 1 (Crippling Fear, Eyeblight Massacre, Olivia's Wrath, Balthor the Defiled, Obelisk of Urd, City on Fire).

**Commits**:
- `049b6802` — PB-X implement (39 files, +2580 / -62; 3 primitives, 6 cards, 12 tests)
- `10411bd8` — PB-X review fixes (9 files, +485 / -37; C1 HIGH + 3 MEDIUMs + 2 LOWs, +4 tests)

**Next session** (priority order):
1. **PB-X close phase**: update `docs/project-status.md` (PB-X → done, review → fixed), `memory/workstream-state.md`, CLAUDE.md Current State (2616 tests, PB-X done), and `memory/primitive-wip.md` (phase → closed). Commit as `W6-prim: PB-X close`.
2. **PB-Q** (ChooseColor) — next highest-yield primitive. Unblocks 9+ cards. Revised slate order: PB-Q → PB-R (ExchangeZones, ~60 LOC) → PB-T + PB-V (up-to-N targeting + DoubleCounters) → PB-U (trigger extensions) → PB-Y (Metallic Mimic) → PB-W (text-changing).
3. **PB-Y** (Metallic Mimic — `LayerModification::AddChosenCreatureType`) parked; do NOT schedule ahead of PB-Q (unblocks only 1 card vs PB-Q's 9+).

**Hazards** (carried forward + new):
- PB-X close phase left undone — remember to update project-status.md, CLAUDE.md, primitive-wip.md before starting PB-Q
- PB-Y (Metallic Mimic) parked — new deferred micro-PB in the slate
- Standing rule: "As ~ enters, choose X" = replacement effect per CR 614.12 (see `memory/gotchas-rules.md`)
- Standing rule: every new LayerModification variant ships with a full-dispatch test (see `memory/conventions.md`)
- Re-triage / full-chain verification discipline holds: `feedback_verify_full_chain.md`, `feedback_verify_cr_before_implement.md`, `feedback_retriage_verification.md`
- PB-S-L02..L06 residuals in abilities.rs — opportunistic or batch into W3-LC-residuals micro-PB
- Simulator mana_solver.rs:35 still reads base chars (PB-S-L01)
- PB-M deferred items (Isshin, Delney, Elesh Norn opponent ETB suppression, Drivnod activated ability)
- Complete the Circuit: delayed copy trigger still TODO
- Forbidden Orchard: TargetPlayer → TargetOpponent (deferred to M10)
- Heritage Druid `TapNCreatures` cost — cost-framework PB, not scheduled

**Commit prefix**: `W6-prim:`

### 2026-04-11 (second session) — W6: PB-S implement + review + fix cycle CLOSED

**Completed** (continuation of earlier PB-S plan session):
- **Implement phase** (runner stop-and-flagged on face-down test expectation; oversight verified CR 708.2, flipped test to "inherits", runner resumed): 2 new LayerModification variants (AddManaAbility + AddActivatedAbility), Layer 6 append semantics, ~80 LOC engine + 5 card defs + 2 TODO updates + 10 tests. Closed W3-LC deferred item in `handle_tap_for_mana` (mana.rs now reads calculated chars).
- **Review phase**: reviewer found 1 HIGH (hash `ActivatedAbility::once_per_turn` field gap — pre-existing, escalated by PB-S's discriminant 23), 0 MEDIUM, 3 LOW. File: `memory/primitives/pb-review-S.md`.
- **Fix cycle** (oversight-bundled H1 + L2 + mandatory spot-check):
  - H1: `hash.rs` — added `once_per_turn` to `HashInto for ActivatedAbility` (field 8/8, verified against struct)
  - L2 test added: `test_granted_once_per_turn_activated_ability_is_preserved_and_enforced` — exercises discriminant 23 (previously untested)
  - **NEW HIGH** (surfaced by L2): `abilities.rs:204` index validation read base → variant 23 unreachable at runtime. Fixed to use calculated chars.
  - **NEW HIGH** (caught by mandatory spot-check): `abilities.rs:478` summoning-sickness/haste check on tap-cost activated abilities read base — sibling W3-LC gap to the mana.rs fix. Fixed.
  - Spot-check residuals logged as LOWs: PB-S-L02 (channel/graveyard dispatch base read), L03 (sacrifice-self event emission), L04 (sacrifice-target event emission), L05 (indexed cost reduction), L06 (humility-inverse test gap). All in `docs/mtg-engine-low-issues-remediation.md`.
- **Tracking updates**: CLAUDE.md, `docs/project-status.md` (PB-S status → done), workstream-state.md, `memory/primitive-wip.md` (phase → fix-complete).
- **New auto-memory**: `feedback_verify_full_chain.md` — generalized verification rule covering triage/plan/impl/review, citing 3 appearances in this session (A-42 re-triage, H1 hash miss, reviewer Q6 miss).

**Test count**: 2589 (start of session) → 2600 (+11: 10 from implement, 1 from fix cycle L2). All tests green, clippy clean, workspace build clean, fmt clean.

**Cards unblocked**: 5 full (Cryptolith Rite, Chromatic Lantern, Citanul Hierophants, Paradise Mantle, Enduring Vitality) + 2 partial TODO updates (Song of Freyalise — Saga-blocked, Umbral Mantle — `{Q}`-blocked).

**Commits**:
- `b212c100` — PB-S plan + A-42 Tier 1 blocked reclassification
- `9dc9331a` — PB-S implement (17 files, +921 lines)
- `5b8496ab` — PB-S review fixes (6 files, +383 lines)

**Next session** (priority order, unchanged):
1. **PB-X**: micro-PB unblocking A-42 Tier 1 (`AllCreaturesExcludingSubtype` EffectFilter, dynamic P/T in LayerModification, `Cost::ExileSelf`) — ~100-150 LOC, unblocks 6 cards + likely others
2. Author A-42 Tier 1 after PB-X lands
3. PB-Q (ChooseColor)
4. PB-R (ExchangeZones, ~60 LOC)
5. PB-T, PB-U, PB-V, PB-W per slate

**Hazards** (carried forward + new):
- Re-triage discipline: verify full primitive chain (feedback_retriage_verification.md + feedback_verify_full_chain.md)
- Spot-check mandatory for any PB that fixes a dispatch pattern — walk every entry point for the subsystem, not just the file touched (feedback_verify_full_chain.md, #4)
- PB-S-L02..L06 residuals in abilities.rs — fix opportunistically or batch into a W3-LC-residuals micro-PB
- Simulator mana_solver.rs:35 still reads base chars (PB-S-L01) — bots undervalue granted mana
- PB-M deferred items (Isshin, Delney, Elesh Norn opponent ETB suppression, Drivnod activated ability)
- Complete the Circuit: delayed copy trigger still TODO
- Forbidden Orchard: TargetPlayer → TargetOpponent (deferred to M10)
- Heritage Druid `TapNCreatures` cost — own PB, not in PB-X scope

**Commit prefix**: `W6-prim:` (primitive work) or `W6-cards:` (authoring)

### 2026-04-11 (earlier) — W6: PB-S plan + A-42 Tier 1 reclassification

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

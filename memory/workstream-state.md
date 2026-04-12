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
| W6: Primitive + Card Authoring | — | available | — | PB queue re-prioritized (commit `70757770`). PB-N planner brief ready in `memory/primitive-wip.md` — next worker claims and runs Step 0 → 1 → 2. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-12 (third session)
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-Q4 full pipeline — plan, implement, review, fix, close

**Completed**:
- **PB-Q4 plan** (Opus planner): all 3 verification gates run. Gate 1 (Genju animate-land): PASS literal but Genju cycle deferred on a *separate* gap — no `WhenAttachedPermanentLeavesBattlefield` + self-graveyard-rebuy trigger. Gate 2 (Chained controller filter): FAIL — resolved by new `EnchantTarget::Filtered` variant + extending `matches_enchant_target` signature with both controllers. Gate 3 (Corrupted Roots disjunction): PASS via existing `TargetFilter.has_subtypes`. Plan: `memory/primitives/pb-plan-Q4.md`. 12 MANDATORY + 2 OPTIONAL tests numbered.
- **PB-Q4 implement** (`9c347754`): 9 engine files. New `EnchantFilter` struct (6 fields) in `state/types.rs` instead of plan's `Box<TargetFilter>` — resolves a real circular dep (`cards/card_definition.rs` imports from `state::*`). New `EnchantControllerConstraint` enum. Hash schema parity verified end-to-end. Cards: Awaken the Ancient, Chained to the Rocks, Ossification, Dimensional Exile (4 HIGH; both MEDIUM correctly deferred — Hot Springs needs prevention primitive, Earthlore needs `is_blocking` filter). 2625 → 2637 (+12 mandatory).
- **PB-Q4 review**: 0 HIGH / 1 MEDIUM / 3 LOW. All four oversight focus items VERIFIED (parity, hash, dispatch, circular dep). Test-skipping retro held — 12/12 mandatory tests delivered, no silent skips. Review file: `memory/primitives/pb-review-Q4.md`.
- **PB-Q4 fix pass** (closes PB-Q test-skipping retro thread): L2 (test 11 — Layer 5 SetColors registration + assertion) and L3 (test 12 — actual attack-with-Haste vs without, exercising the sickness override) tightened. 2637 → 2639. M1 + L1 deferred to LOW backlog as PB-Q4-M01 + PB-Q4-L01 (`docs/mtg-engine-low-issues-remediation.md`).
- **PB-Q4 close** (`0dd7288a`): tracking docs + memory + plan/review files committed.
- **Test count**: 2625 → 2639 (+14 net). Clippy clean, fmt clean, workspace builds clean.

**Next session**: TODO classification report (`460d5e67`, `chore: TODO classification report — 780 card defs categorized into 190 primitive groups`) is the **headline next item**. Use it to **re-prioritize the entire PB queue** before picking the next batch. The current priority order in CLAUDE.md (PB-R → PB-Q3 → PB-T+V → PB-U → PB-Y → PB-Q5 → PB-W) is pre-classification and may be wrong. Read the classification report first, then decide.

**Hazards**:
- **PB-Q4-M01 (MEDIUM, deferred)**: `EnchantFilter` and `TargetFilter` will diverge over time. The cycle is real, but moving `TargetFilter` into `state/` would unblock the plan's preferred shape. Decide when authoring the next non-land enchant target — don't let it rot.
- The new `nonbasic: bool` flag on `TargetFilter` is checked in exactly one site (`effects/mod.rs:6513-6521`) plus `enchant_filter_matches`. If a future PB adds another `TargetFilter` consumer, verify `nonbasic` dispatch.
- Genju cycle (5 cards) and Corrupted Roots / Spreading Algae / Uncontrolled Infestation (3 cards) are blocked on **two different missing trigger types**: rebuy-on-attached-leaves and whenever-attached-becomes-tapped. Both are candidate micro-PBs.
- `apply_mana_production_replacements` (PB-Q baseline) still untouched — keep it that way.
- TODO classification report exists at `460d5e67` but oversight has not yet read it — next session must read first, then re-prioritize.

**Commit prefix used**: `W6-prim:` (1 commit this session: `0dd7288a`; runner committed `9c347754` mid-session)

## Handoff History

### 2026-04-12 (second session) — W6: PB-Q close + PB-Q4 yield audit
- PB-Q close (`464d9e79`): deleted gauntlet_of_power.rs + utopia_sprawl.rs, reverted throne_of_eldraine.rs. Removed parked-only engine variants (`ReplacementManaSourceFilter::{BasicLand, EnchantedLand}`, `EffectFilter::AllCreaturesOfChosenColor`). 2627→2625. Fixed CR citation LOWs.
- Reviewer agent hardened: added oracle-vs-filter semantic gate as step 3 in `.claude/agents/primitive-impl-reviewer.md` (5th appearance of verify-existence-not-completeness failure mode).
- Reservations written: PB-Q2/Q3/Q4/Q5 in `docs/primitive-card-plan.md` Phase 1.7 + `docs/project-status.md`.
- Auto-memory: `feedback_pb_yield_calibration.md` — PB planners overcount in-scope cards by 2-3x; discount 40-50%.
- PB-Q4 yield audit (SQLite, not grep): direct LandSubtype yield 10 cards; bundled scope ~20 cards. Verdict: PB-Q4 #1 priority.
- Three verification gates queued for next session (Genju animate-land make-or-break; Chained controller filter; Corrupted Roots disjunction).

### 2026-04-11 (fourth session) — W6: PB-X close + PB-Q plan + implement
- PB-X close (`c502f8fc`). PB-Q plan caught 2 oversight roster errors via MCP. PB-Q implement (`880b7797`): `GameObject.chosen_color`, `ReplacementModification::ChooseColor` + `AddOneManaOfChosenColor`, `ChosenColorRef`, `ReplacementManaSourceFilter`, 2 EffectFilter variants, `Effect::AddManaOfChosenColor`, `apply_mana_production_replacements` refactor, hash sentinel 2→3. 6 cards, 11 tests, 2616→2627. Review deferred per context pressure.

### 2026-04-11 (third session) — W6: PB-X plan + implement + review + fix
- Plan (Opus): 3 primitives, scope held; stop-and-flagged Metallic Mimic → parked as PB-Y; Obelisk + City on Fire verified already-authorable. 5 open questions resolved by oversight.
- Implement: `EffectFilter::AllCreaturesExcludingSubtype` + `AllCreaturesExcludingChosenSubtype`, `LayerModification::ModifyBothDynamic { amount, negate }` substituted at `Effect::ApplyContinuousEffect` per CR 608.2h (new variant, 76 existing `ModifyBoth` sites untouched), `Cost::ExileSelf` + `ActivationCost.exile_self` via existing `embedded_effect` LKI plumbing. Hash schema 1→2. 6 cards authored (Crippling Fear, Eyeblight Massacre, Olivia's Wrath, Balthor, Obelisk, City on Fire).
- Review: 1 HIGH (C1 — Obelisk `ChooseCreatureType` authored as `TriggerCondition::WhenEntersBattlefield` instead of `ReplacementTrigger::WouldEnterBattlefield`; observable bug in trigger-resolution window), 3 MEDIUM, 3 LOW.
- Fix: sequential two-pass discipline. Pass 1 — C1 alone (Obelisk rewritten to `AbilityDefinition::Replacement` mirroring Urza's Incubator). Pass 2 — bundled E1 (10 CR citation rewrites: "701.10" → "118.12 + 406 + 602.2c"), C2 (Balthor activated e2e), C3 (Obelisk observability-window test + City on Fire tests), E2/E3 LOWs.
- Standing rules established: (a) "As ~ enters, choose X" = replacement effect per CR 614.12, saved to `memory/gotchas-rules.md`; (b) every new `LayerModification` variant ships with a full-dispatch test, saved to `memory/conventions.md`.
- Tests: 2600 → 2612 (+12 impl) → 2616 (+4 fix). Commits: `049b6802` (implement), `10411bd8` (fixes).

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


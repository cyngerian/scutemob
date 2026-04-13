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
| W6: Primitive + Card Authoring | PB-D: planner phase | **ACTIVE** | 2026-04-13 | Planning PB-D (TargetController::DamagedPlayer). PB-N closed; handoff below. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-13 (PB-N close session — worker ran plan → implement → review → fix → re-review → close pipeline under coordinator oversight)
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-N full pipeline — stale-TODO sweep + plan + implement + review + fix + aura experiment + close

**Completed**:
- **Step 0 stale-TODO sweep** (`fc83d9d0`): bootleggers_stash NEWLY AUTHORED (first `LayerModification::AddActivatedAbility` filtered grant on `LandsYouControl`); song_of_freyalise comment rewritten (blocker is Saga framework, not PB-S); throne_of_eldraine comments updated (PB-Q delivered ChooseColor pieces but PB-Q2 + PB-Q5 still block per no-wrong-game-state policy).
- **Step 1 PB-L Landfall pre-check**: confirmed PB-L is a real PB, not a stale sweep. `ETBTriggerFilter` has `creature_only` but no land/card_type filter; `TriggerEvent` has no land-typed variant; `khalni_heart_expedition` and `druid_class` explicitly TODO on it. Cheapest path: extend `ETBTriggerFilter` with `card_type_filter`. Yield re-discounted to 3-4 per trigger calibration.
- **PB-N plan** (Opus planner): dispatch verdict PASS-AS-FIELD-ADDITION (new `filter: Option<TargetFilter>` on two DSL variants + `triggering_creature_filter` on runtime `TriggeredAbilityDef` — mirrors existing `combat_damage_filter`). 8 mandatory + 2 optional tests numbered. Plan file: `memory/primitives/pb-plan-N.md`.
- **PB-N implement** (`d343e1ba` + `7e7d426a`): 7 engine files. Hash sentinel bumped 3→4 (now `pub const HASH_SCHEMA_VERSION` in `hash.rs`, exported from `lib.rs`). `combat_damage_filter` tightened to damage-only (latent semantic bug fix). 56 mechanical card-def backfills for unit → struct shape change. 4 newly authored cards + 9 tests. 2637 → 2646.
- **PB-N review**: 2 HIGH + 3 MEDIUM + 1 LOW. HIGHs: F1 Sanctum Seeker drained `total_lost` (wrong in multi-opponent), F2 Utvara Hellkite missed in roster (had a pre-existing TODO naming the primitive). MEDIUMs: F3 LKI test wedge invalid, F4 combat_damage_filter regression test invalid, F5 hash sentinel assertion too weak. All test-validity MEDIUMs treated as fix-phase HIGHs per new standing rule.
- **PB-N fix phase** (`0e5d7cf1`): F1 rewritten using `Sequence([ForEach(EachOpponent, LoseLife 1), GainLife(Controller, 1)])` (no new engine surface). F2 Utvara Hellkite filter added + TODO stripped + card-specific test — **PB-N yield bumped 4 → 5 cards**. F4 rewritten as `AnyCreatureYouControlAttacks + Ninja filter + Goblin attacker` (strictly discriminating). F5 tightened to `assert_eq!(HASH_SCHEMA_VERSION, 4u8)`. F6 cosmetic — rustfmt doesn't normalize the misalignment, logged as PB-N-L01. F3 STOP-AND-FLAGGED: prescribed wedge structurally unreachable.
- **F3 escalation + aura wedge experiment**: worker ran a 30-min coordinator-directed experiment to test whether an aura-based continuous-effect grant (via `EffectFilter::AttachedCreature`) would survive the LKI dispatch where the LayerModification wedge failed. Pre-experiment source read of `abilities.rs:4180-4202` predicted failure: the dispatch calls `calculate_characteristics(dying_obj_id)` on the graveyard object, re-runs all layer filters, and every battlefield-gated filter drops out (including `AttachedCreature`). The `.unwrap_or_else(|| dying_obj.characteristics.clone())` fallback is dead code. Experiment confirmed: sanity assertions passed (aura grant was visible while on battlefield, base Human preserved), but trigger did not fire after death. **Two independent data points** (LayerMod + aura) promoted the diagnosis from hypothesis to confirmed. Experiment test deleted post-run per directive.
- **PB-N re-review** (post-fix): all findings PASS or accepted. F3 accepted as "resolved by investigation, not by fix" per coordinator directive — Test 6 validates the new `triggering_creature_filter` dispatch path consumption (the load-bearing property), does NOT validate pre-death-vs-post-death LKI (structurally impossible pre-LKI-audit). New finding F-N1 (LOW): cosmetic misaligned `filter: None` indentation in 5 backfilled files — same issue as F6, folded in as PB-N-L01. Verdict: **READY FOR CLOSE**.
- **PB-N close commit** (following): aspirationally-wrong comment at `abilities.rs:4191-4193` replaced with `TODO(BASELINE-LKI-01)` pointing at the tracking LOW. BASELINE-LKI-01 + BASELINE-CLIPPY-01..06 + PB-N-L01 logged in `docs/mtg-engine-low-issues-remediation.md`. Four new standing rules written to `memory/conventions.md`. New gotcha #39 in `memory/gotchas-rules.md` (subtype-filter test wedge discipline). Two new auto-memory feedback files created: `feedback_planner_roster_recall.md` (roster-recall vs yield-overcount separation) and `feedback_escalation_report_behavior_not_cause.md`. `feedback_pb_yield_calibration.md` updated with category-specific yield rates. Planner agent prompt (`.claude/agents/primitive-impl-planner.md`) updated with mandatory step 3a — pre-existing TODO sweep. All docs updated (CLAUDE.md, project-status.md, primitive-card-plan.md Phase 1.8).
- **Test count**: 2637 → **2648** (+11 net: +9 impl, +2 fix). Build clean, fmt clean, tests green.
- **Clippy baseline** (corrected — previous "clippy clean" handoffs were wrong): `cargo clippy --all-targets -- -D warnings` shows ≥6 pre-existing errors across multiple test targets. Cargo's per-target bailout makes the visible count vary run-to-run. All documented individually as BASELINE-CLIPPY-01..06.

**Next session**: **Plan PB-D** (DamagedPlayer target filter). Revised yield 7-8 cards (filter-PB calibration unchanged). Fresh batch, no dependencies. PB-P (PowerOfCreature EffectAmount) is a viable swap if the next worker prefers EffectAmount. PB-L (Landfall trigger) drops to 3rd — trigger calibration + real primitive gap makes it less attractive than either alternative.

**Hazards**:
- **BASELINE-LKI-01** (LOW, cause identified, fix deferred): death-trigger dispatch at `abilities.rs:4180-4202` re-runs layer filters against graveyard objects via `calculate_characteristics`, dropping every battlefield-gated filter. Fix candidates: (a) dispatch reads `dying_obj.characteristics.clone()` directly; (b) teach `calculate_characteristics` to honor preserved chars for non-battlefield zones. **Needs a dedicated LKI-completeness audit session** — not just this one dispatch site. Audit scope: enumerate every battlefield-zone-guarded filter + every dispatch site that reads LKI via `calculate_characteristics` (replacement effects, "leaves the battlefield" triggers, LTB ability resolution are all candidates).
- **PB-N spawned micro-PB candidates (6)**: Najeela (controller-agnostic attack filter), Athreos (owner-not-controller death filter), Skullclamp (equipment-LKI death), Pashalik / Omnath Locus of Rage / Miara (self-OR-filtered death). Each needs a different dispatch shape. **Cataloged, not auto-promoted** — next oversight cycle decides.
- **Trigger-PB yield recalibration**: 15-25% now, not 50%. Filter PBs and EffectAmount PBs remain at 50-65%. See `feedback_pb_yield_calibration.md` category table. Apply this to PB-L's re-estimate (~3-4, not ~7).
- **Planner roster-recall miss**: PB-N missed Utvara Hellkite because the planner only ran MCP oracle lookup; the card had a pre-existing TODO naming the exact primitive. Step 3a in `.claude/agents/primitive-impl-planner.md` now makes this a mandatory pre-roster-finalize grep. Every future PB planner run must either produce a non-empty TODO sweep result or assert "TODO sweep: 0 cards" positively.
- **Handoff "clippy clean" lie**: every previous W6 handoff has said "clippy clean". That was never true with `--all-targets`. Next worker's end-session handoff should say something like "clippy: N pre-existing BASELINE-CLIPPY-0N warnings, no new from this session" instead.
- PB-Q4-M01 carryover: `EnchantFilter` vs `TargetFilter` divergence still open. Address at next non-land enchant target.
- PB-S residuals (L02-L06) still open in `abilities.rs`.

**Commit prefix used**: `W6-prim:` (close commit following this handoff; prior session commits: `fc83d9d0`, `d343e1ba`, `7e7d426a`, `0e5d7cf1`)

## Handoff History

### 2026-04-12 (third session) — W6: PB-Q4 full pipeline

- PB-Q4 plan (Opus) + implement (`9c347754`) + review (0 HIGH / 1 MEDIUM / 3 LOW) + fix (`0dd7288a`). New `EnchantFilter` struct (resolves circular dep vs plan's `Box<TargetFilter>`), `EnchantControllerConstraint` enum. 4 cards: Awaken the Ancient, Chained to the Rocks, Ossification, Dimensional Exile. Tests 2625 → 2639. Genju cycle + Corrupted Roots/Spreading Algae deferred — missing trigger types. PB-Q4-M01 + L01 logged.

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


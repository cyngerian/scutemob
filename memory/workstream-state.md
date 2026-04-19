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
| W6: Primitive + Card Authoring | PB-D implement | ACTIVE | 2026-04-16 | PB-D plan-complete, greenlit for implement. Plan: `memory/primitives/pb-plan-D.md`. |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-13 (PB-D planner session — Opus planner executed plan phase under coordinator oversight)
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-D plan phase (DamagedPlayer target-scope primitive for combat-damage triggers)

**Completed**:
- **Planner agent run** (Opus, ~35 min, 123 tool calls): ran first-actions list, read standing-rules memory files, claimed W6, executed Step 0 sweep + Step 1 PB-P pre-check + Step 2 full plan, wrote `memory/primitives/pb-plan-D.md`, advanced `memory/primitive-wip.md` to `phase=plan-complete`, and committed as `b9f43bf1 W6-prim: PB-D plan phase complete — 6 cards, PASS-AS-NEW-VARIANT`. (The agent hit an API overload on its post-commit report step — artifacts are intact on disk and in git; no resumption needed.)
- **Step 0 stale-TODO sweep**: SKIPPED with positive null — classification report `memory/card-authoring/todo-classification-2026-04-12.md` flags no DamagedPlayer-bucket cards as potentially stale post PB-S/X/Q/N. Recorded in plan preamble.
- **Step 1 PB-P pre-check**: PB-P is a real PB but narrower than its name suggests. `EffectAmount::PowerOf(EffectTarget)` already exists and covers the bulk of "power of creature" cases. The real gap is `EffectAmount::PowerOf` with a `SacrificedCreature` LKI target (Altar of Dementia, Greater Good — sacrifice-time power read). Queue impact TBD by oversight; worker did not act on the finding per instructions.
- **PB-D plan proper**: dispatch verdict **PASS-AS-NEW-VARIANT** — adds a fourth `TargetController` enum entry (`DamagedPlayer`) instead of a new TargetRequirement variant, new PlayerTarget variant, or new enum. Estimated ~10 new match arms across `casting.rs`, `abilities.rs`, `effects/mod.rs`, `hash.rs`. CR citations: 510.1, 510.3a, 603.2, 601.2c.
- **Roster verification** (MCP `lookup_card` on 15 classification candidates + 7 forced-adds via TODO sweep): **6 confirmed shippable** — 2 precision fixes (Throat Slitter, Sigil of Sleep — currently approximated as `Opponent`, wrong in multiplayer) + 4 newly authorable (Mistblade Shinobi, Alela Cunning Conqueror, Nature's Will, Balefire Dragon). 9 deferred for compound blockers, wrong bucket, or already implemented (enumerated in plan's "Deferred cards" section). Yield ≈40% — at the low end of the 50-65% filter-PB calibration band.
- **Test plan**: 7 mandatory + 2 optional tests numbered up front. Mandatory covers positive filter, negative filter, phase-boundary reset on `damage_received_this_turn`, hash parity via `assert_eq!(HASH_SCHEMA_VERSION, <bump>)`, multi-player isolation, real-card e2e (top-yield roster card).
- **BASELINE-LKI-01 verification**: confirmed structurally NOT a concern for PB-D. Player filters read `PlayerState.damage_received_this_turn`, which has no zone-change/layer-resolution dependency. Recorded in plan Risks section.
- **Stop-and-flag count**: 0 — no dispatch split required, no compound-blocker expansion, no hash policy ambiguity (default bump documented).

**Not done (deliberate — requires oversight greenlight)**:
- Implement phase. `memory/primitive-wip.md` halted at `phase=plan-complete`.

**Next session**: **Oversight greenlight review of `pb-plan-D.md`**, then `/implement-primitive` to run the implement phase. Confirm the hash sentinel bump policy (default: 4 → 5 on any wire-visible change per `memory/conventions.md`) before runner starts. After PB-D close, re-slate: PB-P needs oversight triage (real-but-narrow finding), PB-L still rank 3 at ~3-4 calibrated yield.

**Hazards**:
- **API overload risk**: the PB-D planner run hit an overload on its final report step (agent ID `aeeb21943656b1111`). Artifacts are intact, but if the next worker resumes an agent via `SendMessage` during overload windows, expect retries. Starting fresh agents is safer.
- **BASELINE-LKI-01** (carried from PB-N): `abilities.rs:4180-4202` re-runs layer filters against graveyard objects. Fix deferred, needs dedicated LKI-completeness audit session. Does NOT reach PB-D scope (verified).
- **BASELINE-CLIPPY-01..06** (carried from PB-N): `cargo clippy --all-targets -- -D warnings` shows ≥6 pre-existing errors. Report honestly in implement-phase end-session — "N pre-existing BASELINE-CLIPPY-0N warnings, no new from this session", not "clippy clean".
- **PB-N spawned micro-PB candidates (6)**: Najeela, Athreos, Skullclamp, Pashalik, Omnath Locus of Rage, Miara — cataloged in PB-N handoff, not auto-promoted.
- **PB-P narrow gap** (new this session): if oversight promotes PB-P, scope is `EffectAmount::PowerOf(SacrificedCreature)` LKI read only — not a general power-of-creature PB.
- **PB-Q4-M01 carryover**: `EnchantFilter` vs `TargetFilter` divergence still open. Address at next non-land enchant target.
- **PB-S residuals L02-L06**: still open in `abilities.rs`.
- **Trigger-PB yield recalibration**: 15-25% for triggers; 50-65% for filters/EffectAmount. PB-D landed at 40%, slightly below band — usual filter-PB variance, not a calibration miss.

**Commit prefix used**: `W6-prim:` (`b9f43bf1` — single plan-phase commit this session)

## Handoff History

### 2026-04-13 (PB-N close session) — W6: PB-N full pipeline

- Full pipeline (plan → implement → review → fix → re-review → close) under coordinator oversight. Step 0 stale-TODO sweep (`fc83d9d0`) shipped bootleggers_stash as first filtered `LayerModification::AddActivatedAbility` grant on `LandsYouControl`. PB-N plan verdict PASS-AS-FIELD-ADDITION (`filter: Option<TargetFilter>` + `triggering_creature_filter` mirroring `combat_damage_filter`). Implement (`d343e1ba`, `7e7d426a`): 7 engine files, hash sentinel 3→4 promoted to `pub const HASH_SCHEMA_VERSION`, `combat_damage_filter` tightened to damage-only (latent bug fix), 56 mechanical card-def backfills, 4 cards + 9 tests (2637 → 2646). Review found 2 HIGH + 3 MEDIUM + 1 LOW; fix phase (`0e5d7cf1`) rewrote Sanctum Seeker drain (no new engine surface), added Utvara Hellkite catch via TODO sweep (yield 4→5), tightened hash assertion, fixed combat_damage_filter regression test. F3 LKI test wedge stop-and-flagged as structurally unreachable — 30-min aura wedge experiment confirmed BASELINE-LKI-01 (death-trigger dispatch re-runs layer filters against graveyard objects, dropping battlefield-gated filters). Close commit logged BASELINE-LKI-01 + BASELINE-CLIPPY-01..06 + PB-N-L01 in remediation doc, added gotcha #39, created 2 new feedback memory files, updated primitive-impl-planner agent with mandatory step 3a (pre-existing TODO sweep). Tests 2637 → 2648. Clippy baseline correction: every prior "clippy clean" handoff was wrong with `--all-targets`; ≥6 pre-existing errors now logged.

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

**Commit prefix**: `W6-prim:` (primitive work) or `W6-cards:` (authoring)


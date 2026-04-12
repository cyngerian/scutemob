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
| W6: Primitive + Card Authoring | — | paused | — | PB-X CLOSED 2026-04-11. PB-Q implement DONE 2026-04-11 (commit 880b7797; 2627 tests; 6 cards). **phase: review** — next session runs `/implement-primitive` → review (do NOT re-run implement) |

**Status values**: `available` (free to claim), `ACTIVE` (session working on it),
`paused` (partially done, session ended mid-task), `not-started` (blocked/deferred),
`RETIRED` (replaced by another workstream)

## Last Handoff

**Date**: 2026-04-11 (fourth session of the day)
**Workstream**: W6: Primitive + Card Authoring
**Task**: PB-X close + PB-Q plan + PB-Q implement (review deferred per context-pressure rule)

**Completed**:
- **PB-X close phase** (commit `c502f8fc`): `docs/project-status.md` PB-X → done/fixed; CLAUDE.md Current State updated to 2616 tests + "PB-X closed, next PB-Q"; `memory/workstream-state.md` claim flipped; `memory/primitive-wip.md` PB-X phase → closed. Verified both new standing rules from the prior session are actually saved: `gotchas-rules.md:113` (CR 614.12 replacement-effect rule) and `conventions.md:53` (full-dispatch test rule for LayerModification variants).
- **PB-Q plan phase** (Opus planner): `memory/primitives/pb-plan-Q.md` — plan caught **two factual errors in the coordinator's card roster via MCP**: (1) Cavern of Souls is choose-creature-type, NOT choose-color (removed from scope); (2) Utopia Sprawl IS choose-a-color, NOT choose-a-basic-land-type (stays in scope, not bundled). Planner stop-and-flagged 10 open questions for oversight before implement green-light. Oversight answered all 10: activated-time deferred to PB-Q2 (Skrelv/Nykthos/Three Tree City/Throne's draw activation); Painter's Servant deferred (Q2); Gauntlet of Might deferred (Q3); mana doubling = Alt A with Q4 verification gate (grep for stackless-mana-trigger precedent before shipping); `Effect::AddManaOfChosenColor { amount }` new variant (Q6); per-trigger-event semantics (Q5); hash sentinel bump 2→3 (Q9); PB-Q2 to be reserved in primitive-card-plan.md (Q10 — NOT yet done).
- **New auto-memory**: `feedback_oversight_primitive_category_not_cards.md` — oversight names the primitive category; worker verifies card-level scope from oracle text (third consecutive session where oversight card rosters drifted and MCP-verification caught it).
- **PB-Q implement phase** (commit `880b7797`): 28 files, +2615 / -121. 2616 → **2627 tests**. All gates green (test/clippy/build/fmt). **Q4 verification gate resolved cleanly**: Mana Reflection already exists using `ManaWouldBeProduced { controller }` + `MultiplyMana` modification, so Alt A matched the existing precedent exactly — no engine divergence. Doc comment added at the new modification citing CR 605.3 (stackless mana abilities) + CR 603.2 (triggered abilities) explaining the CR-framing compromise. **Q7 verification gate resolved**: `resolution.rs` attaches auras before ETB replacements, so Utopia Sprawl's `chosen_color` gets set atomically with ETB — no ordering subtlety, no stop-and-flag needed.
- **Engine surface landed**: `GameObject.chosen_color: Option<Color>`; `ReplacementModification::ChooseColor(Color)` (parallel of existing `ChooseCreatureType` at `replacement.rs:1440`); `ReplacementModification::AddOneManaOfChosenColor`; `ChosenColorRef { SelfChosen, Fixed(Color) }` + `ReplacementManaSourceFilter { Any, BasicLand, AnyLand, EnchantedLand }`; `ManaWouldBeProduced` extended with `color_filter` + `source_filter`; `EffectFilter::CreaturesYouControlOfChosenColor` + `EffectFilter::AllCreaturesOfChosenColor`; `Effect::AddManaOfChosenColor { player, amount }`; `apply_mana_production_replacements` signature refactored from multiplier-only to `(multiplier, Vec<(ManaColor, u32)>)`; hash schema sentinel 2→3; `chosen_color` added to `HashInto for GameObject` (PB-S H1 discipline); 19 constructor updates across state/mod.rs, state/builder.rs, rules/resolution.rs; helpers.rs re-exports.
- **Cards landed (6 in-scope)**: NEW — `caged_sun.rs`, `gauntlet_of_power.rs`, `utopia_sprawl.rs`; PATCHED — `throne_of_eldraine.rs` (choose-color ETB + `{T}: Add 4 chosen color`), `temple_of_the_dragon_queen.rs` (choose-color ETB + `{T}: Add 1 chosen color` on existing EntersTapped).
- **Tests (11 new in `primitive_pb_q.rs`)**: both mandatory full-dispatch tests shipped — `test_caged_sun_full_dispatch_pumps_chosen_color_creatures` (EffectFilter dispatch via `calculate_characteristics`), `test_caged_sun_doubles_chosen_color_land_mana` (mana-doubling dispatch through `apply_mana_production_replacements`) — plus `test_chosen_color_hash_field_audit` for PB-S H1 defense. Tests cover majority/default-fallback/zone-change-reset/discrimination-filter/no-choice-no-pump paths.
- **Unplanned fix (flag for reviewer)**: deterministic tie-break added to `ChooseColor` fallback — `default_color` preferred when tied for max count, then highest `Color` discriminant. Needed because `HashMap` iteration would otherwise make test 5 flaky / break state-hash equality across equivalent states.

**Next**: Run `/implement-primitive` in the next session to execute PB-Q **review phase only** (plan + implement are done; do NOT re-run implement). `memory/primitive-wip.md` is parked at `phase: review` — unchanged by this end-session per oversight directive.

**Review focus list** (for the Opus reviewer — 8 items, worker's 6 + oversight's 2):
1. **Deterministic tie-break logic** in `ChooseColor` fallback — unplanned addition. Verify correctness under APNAP state-hash equality and determinism across `im-rs` HashMap iteration order. Confirm `default_color` precedence is sound for every card currently using it.
2. **`apply_mana_production_replacements` signature refactor** — ripple across all call sites. Verify every caller handles both the multiplier and the additions list correctly; no caller dropped the additions.
3. **`CreaturesYouControlOfChosenColor` vs `AllCreaturesOfChosenColor`** — Gauntlet of Power (all controllers) vs Caged Sun (you control) — confirm both variants are exercised in tests and the layer dispatch correctly reads `chosen_color` from the source permanent dynamically.
4. **19 `chosen_color: None` constructor updates** — any `GameObject` constructor missed? Grep `GameObject {` vs constructor sites, confirm exhaustive.
5. **Utopia Sprawl Aura attach + ETB replacement ordering** — `resolution.rs` was inspected at impl time (attach before ETB replacements), but verify the *actual* runtime behavior in a test; don't rely on code-path inference alone.
6. **Hash sentinel bump + `HashInto for GameObject` field count** — audit field count of the struct against the hash impl (PB-S H1 methodology). Confirm sentinel is actually 3u8.
7. **[oversight]** Q4 compromise documentation — verify the CR 605.3 + CR 603.2 comment actually exists at the `AddOneManaOfChosenColor` modification definition and explains the stackless-trigger framing clearly enough that a future reader understands why this is a replacement, not a trigger.
8. **[oversight]** `ReplacementManaSourceFilter` naming and scope — confirm the enum name doesn't collide with `ManaSourceFilter` in `card_definition.rs` (runner noted this at impl time) and the `BasicLand / AnyLand / EnchantedLand` scopes are sufficient for the three cards using them (Gauntlet basic-only, Caged Sun any-land, Utopia Sprawl enchanted-land-specific).

**Deferred**:
- **Q10 (PB-Q2 reservation)**: NOT yet written to `docs/primitive-card-plan.md`. Activated-time choose-color cards parked: Skrelv, Nykthos Shrine to Nyx, Three Tree City (third ability), Throne of Eldraine's `{3}{T}: Draw two cards` activation. Different plumbing (per-activation `EffectContext.chosen_color`, not `GameObject` field). Do this in next session's close phase or when PB-Q2 is picked up.
- **Painter's Servant**: Tier-1-verify, may need a Layer 5 `AddColorDynamic` modification — revisit after PB-Q review closes.
- **Gauntlet of Might**: static color filter (no choice) — different primitive, park cleanly; may fit near PB-W text-changing cluster or as its own micro.
- **`mana_reflection.rs` TODO comment**: references "spending restrictions"; existing behavior is correct, comment cleanup deferred.

**Hazards** (carry-forward + new):
- `memory/primitive-wip.md` MUST stay at `phase: review`. Do not advance it this session-end per oversight directive.
- **Q10 PB-Q2 reservation still pending** — remember to write it in next session's close phase.
- PB-Q2 semantic category: activated-time choose-color needs per-activation context plumbing (different from GameObject field), do NOT conflate with PB-Q.
- PB-Y (Metallic Mimic AddChosenCreatureType) still parked — do NOT schedule ahead of open PB-Q work.
- PB-S residuals L01..L06 still open (abilities.rs base-chars reads, mana_solver.rs:35) — opportunistic only.
- PB-M deferred items (Isshin attack trigger doubling, Delney power-filtered doubling, Elesh Norn opponent ETB suppression, Drivnod activated ability) — opportunistic.
- PB-X deferred close items already handled this session.
- `apply_mana_production_replacements` refactor is a load-bearing signature change — if review surfaces a regression, fix-cycle must re-verify every mana-production call site, not just the touched ones (full-chain discipline).
- Oversight guidance reframed: card-level scope comes from MCP verification, not from coordinator's memory — see `feedback_oversight_primitive_category_not_cards.md`.

**Commit prefix used**: `W6-prim:`

## Handoff History

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

### 2026-04-10 — W6: PB-J + PB-M
- PB-J: CopySpellOnStack, ChangeTargets (CR 115.7a/d). 3 card fixes. 9 tests.
- PB-M: Panharmonicon trigger doubling (2 bug fixes, 2 new filters, 1 new card, 3 fixes, 5 tests). All HIGH batches complete. 2589 tests.

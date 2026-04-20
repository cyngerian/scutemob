# Primitive WIP: PB-T — TargetRequirement::UpToN (optional target slots)

batch: PB-T
title: TargetRequirement::UpToN — optional target slots (CR 601.2c "up to N target(s)")
cards_unblocked_estimated: 22 raw candidates from classification report; filter-PB calibration 40-65% → ~9-14 BEFORE compound-blocker discount. Spot-sample already shows compound blockers (Tamiyo emblem grants, Endurance graveyard-to-library, Tyvar mana-ability grant) — expect aggressive discount. Worker/planner must run mandatory 22-card MCP sweep before finalizing roster.
cards_unblocked_confirmed_post_plan: 8-14 cards (8 core-CONFIRMED + 6 bonus via TODO sweep). 14 candidates DEFERRED for compound blockers (non-targeted untap-lands, SearchLibrary-N, delayed trigger riders, dynamic MV filters, etc.). See pb-plan-T.md Step 0 table for full breakdown.
started: 2026-04-20
phase: fix-complete
plan_file: memory/primitives/pb-plan-T.md (written 2026-04-19 by planner)
review_file: memory/primitives/pb-review-T.md (written 2026-04-20 by reviewer; verdict needs-fix — 1 HIGH, 5 MEDIUM, ~11 LOW)

## Task reference
- ESM task: scutemob-5
- Branch: feat/pb-t-implement-targetrequirementupton-optional-target-slots
- Acceptance criteria: 3482 (Step 0 sweep), 3483 (primitive added + HASH bump), 3484 (dispatch sites), 3485 (card defs), 3486 (tests), 3487 (cargo test/clippy/fmt), 3488 (delegation chain)

## Plan summary (from pb-plan-T.md)

- **Confirmed yield**: 8 robust CONFIRMED + 6 bonus (TODO sweep) = 8-14 card defs total, well above AC 3485 floor of ≥4.
- **Chosen shape**: **Shape A — enum wrapper** `TargetRequirement::UpToN { count: u32, inner: Box<TargetRequirement> }`. Rationale: most isomorphic with CR phrasing ("up to N target [something]"), minimal blast radius (single new variant), leverages existing robust `ctx.targets.get(idx) → Option` handling in `resolve_effect_target_list`, no cross-cutting Vec-shape refactor needed.
- **Dispatch unification verdict**: **PASS**. All 10 dispatch sites walked (2 beyond the 8 named in primitive-wip.md); 3 require code changes (DSL schema, hash, validator), 7 require no change because existing logic is UpToN-safe.
- **Mandatory tests**: **8 MANDATORY** (M1-M8: zero-target, partial, full, hash schema, partial-fizzle, regression, mixed mandatory+UpToN, wrong-type rejection) + **5 OPTIONAL** (O1-O5).
- **Deferred-card list**: abstergo_entertainment, blessed_alliance, buried_alive, cloud_of_faeries, frantic_search, glissa_sunslayer, mindbreak_trap, skullsnatcher, smugglers_surprise, snap, ancient_bronze_dragon (conditional), ajani_sleeper_agent, endurance (conditional), bridgeworks_battle (optional), wrenn_and_realmbreaker, tatyova_steward_of_tides, kaito_dancing_shadow, hammerhead_tyrant, skyclave_apparition, yawgmoth_thran_physician, bottomless_pool, carmen_cruel_skymarcher, sword_of_light_and_shadow, the_eternal_wanderer, ugin_the_spirit_dragon, gilded_drake, legolass_quick_reflexes, teferi_hero_of_dominaria — 28 cards DEFERRED with specific compound-blocker reasons (see plan Step 0 table).
- **Hash bump version**: **7 → 8** (current is 7 from PB-L, not 6 as original task description stated — PB-L shipped 2026-04-19 evening and bumped 6→7).
- **3 existing test files to update** from `assert_eq!(HASH_SCHEMA_VERSION, 7u8, ...)` to `8u8`: `pbp_power_of_sacrificed_creature.rs:782`, `pbn_subtype_filtered_triggers.rs:548`, `pbd_damaged_player_filter.rs:597`.
- **New LOWs logged during plan**: PB-T-L01 (loyalty ability targets not validated in `handle_activate_loyalty_ability`, pre-existing), PB-T-L02 (Sorin −6 reanimate rider deferred), PB-T-L03 (Tamiyo PreventUntap dependency — conditional).

## How this PB was selected

From the ESM task description + `docs/primitive-card-plan.md` (PB-T was rank-10 in yield terms, demoted from rank-1; see line 737). Gap: `targets: Vec<TargetRequirement>` is mandatory-only — no infrastructure for skippable target slots ("up to N target X"). The shape of the fix (enum wrapper `UpToN(count, inner)`, per-slot `Option<TargetRequirement>`, or a struct-field extension) is a worker/planner decision, not predetermined by title. Planner selected Shape A after dispatch walk confirmed no existing filter-re-indexing bug would be exacerbated.

## 22 raw candidates (from task description)

abstergo_entertainment, blessed_alliance, bridgeworks_battle, buried_alive, cloud_of_faeries, elder_deep_fiend, force_of_vigor, frantic_search, glissa_sunslayer, marang_river_regent, mindbreak_trap, skullsnatcher, smugglers_surprise, snap, sorin_lord_of_innistrad (+6 more — planner must identify via grep for TODO mentions of "up to" target phrasing)

## Additional candidates surfaced during plan (grep-discovered)

tyvar_kell, teferi_time_raveler, teferi_hero_of_dominaria, wrenn_and_realmbreaker, kogla_the_titan_ape, kaito_dancing_shadow, moonsnare_specialist, hammerhead_tyrant, skyclave_apparition, skemfar_elderhall, yawgmoth_thran_physician, bottomless_pool, carmen_cruel_skymarcher, sword_of_light_and_shadow, sword_of_sinew_and_steel, the_eternal_wanderer, ugin_the_spirit_dragon, endurance, gilded_drake, tatyova_steward_of_tides, legolass_quick_reflexes, tamiyo_field_researcher, teferi_temporal_archmage, tyvar_jubilant_brawler, ancient_bronze_dragon, ajani_sleeper_agent, basri_ket, elder_deep_fiend, force_of_vigor, marang_river_regent, sorin_lord_of_innistrad

## MANDATORY pre-plan steps for the planner

### Step 0: 22-card oracle-text sweep (MANDATORY)

For each of the 22 raw candidates (and any additional cards discovered via grep for "up to [N] target" TODOs in `crates/engine/src/cards/defs/`), look up oracle text via MCP `lookup_card` and classify:

- **CONFIRMED**: sole blocker is `TargetRequirement::UpToN` (or the worker's chosen shape). Oracle text matches the "up to N target X" pattern. No compound blockers.
- **DEFERRED + reason**: compound blocker exists (lists the specific other gap). Card is out of PB-T scope.

Record CONFIRMED count in plan preamble. Per `memory/feedback_pb_yield_calibration.md`, expect 40-65% yield BEFORE compound-blocker discount; expect aggressive additional discount afterward (spot-sample already shows Tamiyo/Endurance/Tyvar are all compound-blocked). Final CONFIRMED yield ≥4 per AC 3485.

### Step 1: CR research (mandatory)

- **CR 601.2c**: "The player announces his or her choice of an appropriate player, object, or zone for each target the spell requires. ... If the spell or ability requires a variable number of targets, the player chooses the number of targets."
- **CR 115**: targeting overview.
- **CR 115.1b**: "If an object requires any targets, the number of targets is specified in its rules text... If the spell or ability says 'up to [N],' the player chooses between zero and N targets (inclusive)."
- Verify whether a spell with zero declared targets is still a valid "targeting" spell (CR 115.1 subtleties about protection, hexproof, etc.).
- Confirm behavior when resolving an effect with fewer than maximum targets — the effect applies only to the declared targets, and per-target effects (ForEach) iterate only over those actually declared.

### Step 2: Engine architecture walk (MANDATORY — walk every dispatch site per `feedback_verify_full_chain.md`)

Trace the full dispatch chain:

1. **DSL schema** — `crates/engine/src/cards/card_definition.rs`: `TargetRequirement` enum (~line 2267), `targets: Vec<TargetRequirement>` fields on casting/activation variants (lines 225, 255, 280, 401, 413, 526, 719). Planner chooses shape:
   - **Shape A**: New variant `TargetRequirement::UpToN { count: u32, inner: Box<TargetRequirement> }` (enum-wrapper; most isomorphic with CR phrasing).
   - **Shape B**: Add `min_targets: u32` field alongside `max_targets` implicit in Vec length (struct-field extension — more invasive, bigger blast radius).
   - **Shape C**: Change `targets: Vec<TargetRequirement>` → `targets: Vec<TargetSlot>` where `TargetSlot = { required: bool, requirement: TargetRequirement }` (per-slot Option).
   - Planner **MUST** document chosen shape and rationale in the plan file.

2. **Target declaration (casting path)** — `crates/engine/src/rules/casting.rs`: the site where CastSpell's declared targets are validated against the TargetRequirement vec. For UpToN, the declared-targets vec may be shorter than the requirement vec; allow this. At declaration time, the player chooses how many targets (1..=N) for each UpToN slot.

3. **Target validation** — `validate_object_satisfies_requirement` (likely in `casting.rs` or `rules/targeting.rs` — planner must locate): for UpToN slots, each declared target must satisfy the inner requirement.

4. **Target resolution (effect path)** — `crates/engine/src/effects/mod.rs` ForEach + per-target dispatch sites that iterate `DeclaredTarget` vec. With UpToN, declared targets may be partial; existing iteration should Just Work if it already iterates `declared_targets` (not the static `targets: Vec<TargetRequirement>`). Planner must verify.

5. **Hash** — `crates/engine/src/state/hash.rs`: arm for `TargetRequirement` enum; add new variant's arm. Bump `HASH_SCHEMA_VERSION` (currently 7 from PB-L, not 6 as task description said — PB-L shipped 2026-04-19 evening) → 8.

6. **Replay harness legality check** — `crates/engine/src/testing/replay_harness.rs`: the site that validates test scripts' declared targets against the card's TargetRequirement vec. UpToN must allow zero-to-N targets.

7. **LegalActionProvider** — `crates/simulator/src/legal_actions.rs` (likely location): if the simulator generates cast/activate actions with target choices, UpToN slots need action-generation logic (either: generate one action per valid target-count, or represent target-count as a sub-choice).

8. **TargetProvider** — any test helper or harness code that enumerates valid targets for a given TargetRequirement. UpToN must delegate to the inner requirement's generator.

### Step 3: Dispatch unification verdict (MANDATORY GATE)

Worker MUST record one of:
- **PASS** — chosen shape (A/B/C) fully addresses all 8 dispatch sites with no cross-cutting refactor beyond the Vec-shape change.
- **SPLIT-REQUIRED** — discovery during walk requires multiple primitives; **STOP and flag to oversight**.

Per `memory/feedback_verify_full_chain.md`: verifying "TargetRequirement enum has a new variant" is NOT sufficient. Planner must walk the ACTUAL dispatch chain end-to-end (declare → validate → resolve → hash → replay) and prove each arm is either already UpToN-safe or is explicitly extended in the plan.

### Step 4: Re-scoping check (MANDATORY)

Is there existing partial mechanism (e.g. modal `ModeSelection`, or the existing min/max target fields on specific cards) that already covers >50% of the 22 candidates? If so, re-scope PB-T and STOP-AND-FLAG to oversight.

### Step 5: Card roster fixes (mandatory)

For each CONFIRMED card (≥4 per AC 3485):
- Replace placeholder/TODO with the new primitive shape.
- Verify the entire oracle text is covered (not just the target count — other gaps may remain).
- If any CONFIRMED card has a hidden compound blocker that surfaces during implement, DROP the card; do not expand PB-T surface.

### Step 6: Test plan (MANDATORY)

Tests in `crates/engine/tests/<new file>.rs` (name e.g. `pbt_up_to_n_targets.rs`). Each test cites CR 601.2c or relevant subrule. At minimum:
- **M1: Zero-target test** — player skips all optional slots (declares 0 targets for UpToN{3, Creature}); spell resolves without applying per-target effects to anyone.
- **M2: Partial-target test** — player picks 1 of up-to-3 targets; effect applies to the 1 chosen creature.
- **M3: Full-target test** — player picks N of N; effect applies to all.
- **M4: Hash determinism** — same declared-targets vec → same hash; schema bump verified.
- **M5: Target re-legality check** — if a mid-resolution target becomes illegal (zone change), only that one is dropped; others resolve. (Mirrors existing TargetRequirement semantics.)
- **M6: Regression sweep** — existing mandatory-target cards still work (Swords to Plowshares, Lightning Bolt sample runs).

OPTIONAL tests (O1-Ox) planner may add.

## Stop-and-flag triggers

- Dispatch unification gate fails (SPLIT-REQUIRED) → escalate to oversight, do not silently work around
- Compound blocker would expand PB-T surface beyond optional-target-slot semantics
- Existing partial mechanism (modal ModeSelection etc.) already covers >50% of roster → re-scope
- Any CONFIRMED card has a hidden compound blocker that surfaces during the implement phase → drop the card; do not expand PB-T scope
- BASELINE-CLIPPY-01..06 baseline warnings are not in scope to fix

## Out of scope for PB-T

- Modal "choose one — [effect] / [effect]" spells (already covered by `ModeSelection`)
- Spells with variable-number targets that are NOT "up to N" (e.g., "for each creature you control" type effects — already covered by `ForEach`)
- Fused targeting semantics for split cards (covered by `Fuse`)
- Changes to TargetFilter internals
- Changes to target-redirection / protection / shroud / hexproof semantics
- **Non-targeted "up to N X" variants** (untap up to N lands, exile any number of target spells, search library for up to N cards, put up to N cards from hand onto battlefield) — these have no "target" word and are CR-distinct from UpToN. Separate primitives.
- **Distribute-counters-among-targets** (Ajani Sleeper Agent −3) — CR 601.2d divide/distribute mechanic, separate primitive.
- **"Any number of target X"** (Mindbreak Trap) — unbounded is structurally different from bounded UpToN.
- **Dynamic-filter-against-triggering-context** (Hammerhead Tyrant's MV<=cast-spell's-MV, Carmen Cruel Skymarcher's MV<=own-power) — out of scope.
- **Planeswalker emblem static effects** (Sorin reanimate rider, Tamiyo +1 grant trigger, Ugin −10 put-from-hand) — separate primitives.
- **PB-T-L01**: `handle_activate_loyalty_ability` does not validate targets against TargetRequirement (pre-existing, unrelated to UpToN).

## Standing rules to honor

- `memory/conventions.md`: test-validity MEDIUMs = fix-phase HIGHs; hash sentinel pub const + assert_eq!; implement-phase default-to-defer; aspirationally-wrong comments are hazards
- `memory/gotchas-rules.md`: CR 601.2c, 115.1b, 400.7 (target object identity)
- `memory/feedback_verify_full_chain.md`: walk every dispatch site
- `memory/feedback_pb_yield_calibration.md`: filter PBs calibrate at 40-65% — discount aggressively

## Artifacts the planner must produce

- `memory/primitives/pb-plan-T.md` (full plan file)
- Updated `memory/primitive-wip.md` checklist (this file) with all planner steps checked
- A 1-paragraph summary at the top of pb-plan-T.md naming:
  confirmed yield (N cards), chosen shape (A/B/C with rationale), dispatch unification verdict, mandatory test count, deferred-card list, hash bump version

## Planner checklist (worker fills in)

- [x] Step 0: 22-card oracle-text sweep complete; CONFIRMED/DEFERRED+reason recorded — 22 raw + 23 grep-discovered = 45 classified; 8 core CONFIRMED + 6 bonus CONFIRMED + 28-31 DEFERRED (per plan Step 0 table)
- [x] Step 1: CR research notes captured (601.2c, 115.1b, 115, 400.7) — full CR text quoted in plan Step 1; 608.2b and 601.2d also referenced for partial-fizzle and divide-distribute gates
- [x] Step 2: engine architecture walk done; all 8 dispatch sites traced with file:line references — 10 sites total (2 beyond initial scope: loyalty-ability path, resolution.rs legal_targets filter). Each site has current-behavior + required-change + verdict
- [x] Step 3: dispatch unification verdict recorded (PASS or SPLIT-REQUIRED) — **PASS** (see plan Step 3)
- [x] Step 4: re-scoping check performed (no existing partial mechanism covers >50%) — 5 candidate mechanisms reviewed; none cover >5% of roster
- [x] Step 5: card roster confirmed (≥4 cards); each card-def fix described concretely — 14 cards with per-card fix sketches (8 core + 6 bonus); above floor of 4
- [x] Step 6: test plan numbered MANDATORY/OPTIONAL with zero-target + partial-target tests — 8 MANDATORY (M1-M8) + 5 OPTIONAL (O1-O5)
- [x] Plan file written: `memory/primitives/pb-plan-T.md`
- [x] Wip file phase advanced to `plan-complete` for runner handoff

## Implementation checklist (runner fills in)

- [x] Engine change 1: `TargetRequirement::UpToN { count: u32, inner: Box<TargetRequirement> }` variant added to `card_definition.rs:~2306` with CR-cited doc comment (601.2c / 115.1b)
- [x] Engine change 2: `target_count_range` helper added to `casting.rs`; `validate_targets_inner` rewritten with greedy-consume algorithm; unmapped-target rejection added post-mapping (fixes M8/O2c)
- [x] Engine change 3: recursive UpToN arm added to `validate_object_satisfies_requirement` (casting.rs ~5688) and `validate_player_satisfies_requirement` (casting.rs ~5533)
- [x] Engine change 4: NO CHANGE to target resolution sites (ForEach + per-target dispatch) in `effects/mod.rs` — verified UpToN-safe
- [x] Engine change 5: hash arm added in `state/hash.rs` (discriminant 17); HASH_SCHEMA_VERSION bumped 7→8; history comment added
- [x] Engine change 6: NO CHANGE to replay harness legality check — verified no pre-validation in `replay_harness.rs`
- [x] Engine change 7: NO CHANGE to LegalActionProvider (simulator bots already send empty targets)
- [x] Engine change 8: NO CHANGE to TargetProvider / test helpers (tests author SpellTargets directly)
- [x] All exhaustive matches updated: UpToN arm added to `abilities.rs` auto-target enumeration (only missing site); tools/replay-viewer + tools/tui verified to not match on TargetRequirement
- [x] 3 existing test files updated from `assert_eq!(HASH_SCHEMA_VERSION, 7u8, ...)` to `8u8` with PB-T reference: pbp_power_of_sacrificed_creature.rs:782, pbn_subtype_filtered_triggers.rs:548, pbd_damaged_player_filter.rs:597
- [x] Card defs updated (14 confirmed cards): elder_deep_fiend, force_of_vigor, marang_river_regent, sorin_lord_of_innistrad, basri_ket, tamiyo_field_researcher, teferi_temporal_archmage, tyvar_jubilant_brawler, tyvar_kell, teferi_time_raveler, kogla_the_titan_ape, moonsnare_specialist, skemfar_elderhall, sword_of_sinew_and_steel
- [x] Tests written: `crates/engine/tests/pbt_up_to_n_targets.rs` (8 MANDATORY M1-M8 + 2 OPTIONAL O1-O2 implemented; all 10/10 pass)
- [x] All existing tests pass: `cargo test --all` — 0 failures (269 test suites passing)
- [x] `cargo build --workspace` clean
- [x] `cargo fmt --check` clean
- [x] Clippy: 0 warnings — pre-existing collapsible_match in abilities.rs/casting.rs/mana.rs/replacement.rs/simulator/tui fixed with `#[allow]` at match or function level
- [x] HASH_SCHEMA_VERSION sentinel assertions updated in all 3 test files that reference it
- [x] TODOs resolved in card def files for all 14 confirmed cards; compound-blocker TODOs (Sorin reanimate L02, Tamiyo PreventUntap L03) retained with clear markers
- [x] PB-T-L01 (loyalty target validation), PB-T-L02 (Sorin reanimate rider), PB-T-L03 (Tamiyo PreventUntap if applicable) logged in `docs/mtg-engine-low-issues-remediation.md` — logged but PB-T-L01 description is factually wrong per reviewer D1

## Implementation Notes (runner)

- **Bug found + fixed mid-implement**: M8/O2c tests caught a gap in the greedy-consume algorithm — if a declared target didn't match any requirement slot, the validator would silently accept it. Added a post-mapping length check (`mapping.len() < targets.len()` → reject with "declared N targets but only M could be matched"). This fixed M8 and O2c. Captured in `validate_targets_inner` at casting.rs:5404-5410.
- **Collapsible-match clippy sweep**: the impl changes triggered newly-visible clippy warnings at existing sites (abilities.rs, casting.rs, mana.rs, replacement.rs, plus simulator + tui). All fixed with `#[allow(clippy::collapsible_match)]` attributes — NOT by restructuring the matches (out of scope per implement-phase-default-to-defer rule).

## Fix Phase Notes (fix runner)

- **E1 (HIGH)**: Replaced greedy-consume in `validate_targets_inner` (casting.rs) with two-pass best-fit algorithm. Pass 1 assigns each declared target to the first unmatched mandatory slot it satisfies (in slot order). Pass 2 assigns remaining targets to UpToN slots with remaining capacity. This correctly handles reverse-order declarations per CR 601.2c. Added M10 test (`test_pbt_up_to_n_reverse_order_declaration_succeeds`) for [artifact, planeswalker] against [UpToN{PW}, UpToN{Artifact}] shape.
- **E4 (MEDIUM)**: Rewrote `validate_targets` doc comment to describe two-pass best-fit semantics. Removed aspirationally-wrong "parallel indexing" claim. Also removed incorrect 115.1b citation from `target_count_range` helper comment.
- **T1 (MEDIUM, test-validity HIGH per conventions.md)**: Renamed M5 from `test_pbt_up_to_n_partial_fizzle_on_zone_change` to `test_pbt_up_to_n_partial_target_declaration_resolves` with accurate doc comment. Added new M9 test `test_pbt_up_to_n_partial_fizzle_on_zone_change` that genuinely exercises CR 608.2b zone-change partial fizzle: P1 casts UpToN{2} targeting A+B; P2 destroys A in response (Destroy Creature spell with `.with_types(vec![CardType::Instant])`); UpToN resolves — A illegal (graveyard) → skipped, B → tapped.
- **T2 (MEDIUM)**: Added O3 test `test_pbt_force_of_vigor_card_integration` using the real Force of Vigor card def loaded from registry. Partial (1-of-2) UpToN cast via `colorless: 2, green: 2` mana pool.
- **E2 (MEDIUM)**: Added UpToN arm in `check_triggers` auto-target outer match in abilities.rs. For player-inner UpToN, routes to player-picker. For permanent-inner UpToN, returns None (skip optional slot — 0 targets for triggers with optional targeting).
- **E3 (MEDIUM)**: Added doc comment on `TargetRequirement::UpToN` in card_definition.rs stating that `inner` MUST NOT itself be UpToN.
- **D1 (MEDIUM)**: Rewrote PB-T-L01 entry in docs/mtg-engine-low-issues-remediation.md to accurately describe the gap (zero target validation in `handle_activate_loyalty_ability` in engine.rs), naming all 6 affected PB-T cards and pointing to the fix approach.
- **LOW findings not fixed**: E5 (115.1b citation sweep — ~20 sites), E6 (pre-existing loyalty gap), C1-C4, D2-D6, T3-T4. These remain in docs/mtg-engine-low-issues-remediation.md for opportunistic cleanup.
- **All gates passed**: `cargo test --all` 0 failures (13/13 PB-T tests pass); `cargo build --workspace` clean; `cargo fmt --check` clean; clippy: 0 new lints introduced in modified files.

## Reviewer checklist

- [x] CR rules independently verified (601.2c, 115.1b, 115, 400.7) — verified via MCP; 115.1b is about Auras, NOT "up to N" semantics. See review finding E5 (pervasive misattribution).
- [x] Card oracle text verified via MCP for all confirmed cards — 14 cards MCP-verified; Sorin/Marang "other" filter gaps noted as LOW (C1/C2).
- [x] All 10 dispatch sites independently walked and confirmed UpToN-safe (especially sites 4, 6-10 that planner claims "no change required") — 9 of 10 confirmed; site 9 (loyalty activate) NOT UpToN-safe — see E6/D1.
- [x] Hash sentinel 7→8 verified in `state/hash.rs`; assertion updates verified in all 3 test files — confirmed; stale docstrings in pbp/pbd noted as LOWs (D2/D3).
- [x] Zero-target (M1) and partial-target (M2) tests genuinely exercise the new shape (not silent-skip pattern) — M1 confirms artifact survives; M2 confirms exactly 1 destroyed. Solid.
- [x] Hash-schema test (M4) asserts `HASH_SCHEMA_VERSION == 8u8` AND exercises 3+ distinct UpToN variants producing distinct hashes — confirmed 3 variants + 2 neighbor-disc comparisons.
- [x] Regression sweep: existing mandatory-target cards still resolve correctly (M6 passes, plus broader `cargo test --all` green) — M6 exercises 3 cases (0/2/1 targets); mandatory-1-target still enforced.
- [x] Greedy-consume validator algorithm doesn't over-consume (test M7 exercises mixed mandatory+UpToN layout; test M8 exercises wrong-type rejection) — M7/M8 good; BUT reverse-order declaration is silently rejected by greedy algo (HIGH E1).
- [x] Card defs match oracle text verbatim (via MCP re-lookup; no drift) — 14/14 match; "other" filter gaps on Sorin/Marang are LOW.
- [x] Tools (TUI, replay-viewer) exhaustive matches updated if necessary (expected: no change needed; verify anyway) — CONFIRMED zero TargetRequirement references in tools/ and simulator/.
- [x] No scope creep: Sorin reanimate rider, Force of Vigor pitch cost, Tamiyo PreventUntap (if missing) all stayed out of PB-T — confirmed; all retained as TODOs.
- [x] PB-T-L01/L02/L03 LOWs logged — PB-T-L02/L03 accurate; PB-T-L01 description is factually wrong (D1 MEDIUM).
- [x] Review file written: `memory/primitives/pb-review-T.md`
- [x] Wip phase advanced to `review-complete`

## Review verdict

**needs-fix** — 1 HIGH, 5 MEDIUM, ~11 LOW. See `memory/primitives/pb-review-T.md` for findings. HIGH (E1) must be addressed before merge: greedy-consume validator rejects CR-legal target declarations when the player declares targets out of slot order. MEDIUM set: T1 test-validity (M5 doesn't test what its name claims), D1 PB-T-L01 description is factually wrong, E2/E3 latent auto-target issues for UpToN{player} and nested UpToN, E4 aspirationally-wrong doc comment, T2 no card-integration test.

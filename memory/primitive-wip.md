# Primitive WIP: PB-P — EffectAmount::PowerOfSacrificedCreature (LKI)

batch: PB-P
title: EffectAmount::PowerOfSacrificedCreature — LKI-based read of a sacrificed creature's power at effect resolution
cards_unblocked_estimated: 3 confirmed (altar_of_dementia, greater_good, lifes_legacy); below filter-PB calibration (yield ~23%); accepted as narrow real-but-narrow primitive
started: 2026-04-19
phase: implement-complete
plan_file: memory/primitives/pb-plan-P.md (WRITTEN 2026-04-19)
implement_completed: 2026-04-19

## How this PB was selected

Re-triage from PB-D pre-check (memory/primitives/pb-plan-D.md and memory/workstream-state.md).
Original PB-P queue entry (`docs/primitive-card-plan.md` line 833) was a 13-card "PowerOfCreature
EffectAmount" filter PB. The PB-D planner's Step 1 sanity check found:

- `EffectAmount::PowerOf(EffectTarget)` already exists and is widely used (Swords to Plowshares,
  Souls' Majesty, Marwyn, Warstorm Surge, Eomer King of Rohan).
- The REAL gap is `EffectAmount::PowerOf(EffectTarget::SacrificedCreature)` — an LKI-based
  read at the moment of sacrifice — needed only by Altar of Dementia, Greater Good, and
  Life's Legacy (all three explicitly name this primitive in their TODOs).
- Most PB-P sample cards (Krenko Tin Street Kingpin, Master Biomancer, The Great Henge,
  Conclave Mentor, etc.) are blocked on entirely different gaps and out of PB-P narrow scope.

This wip file uses the narrow scope. Yield calibration is **23% (3/13)** — below the 40-65% filter-PB
calibration band per `memory/feedback_pb_yield_calibration.md`. Acceptable: this is a real primitive
that unblocks a small, well-defined card set and resolves a recognized DSL gap with explicit TODOs.

## Pre-triage table (worker has verified by reading every card def + grepping LKI plumbing)

| Card | Status | Reason |
|------|--------|--------|
| altar_of_dementia | **CONFIRMED** | sole blocker is `EffectAmount::PowerOfSacrificedCreature` (mill amount = sac creature's power); activated `Cost::Sacrifice(filter)` |
| greater_good | **CONFIRMED** | needs `EffectAmount::PowerOfSacrificedCreature`; "discard 3" comment claims `Effect::DiscardCards` is missing — **STALE**: `Effect::DiscardCards { player, count }` exists in `card_definition.rs:1197` and is implemented in `effects/mod.rs:518` |
| lifes_legacy | **CONFIRMED** | already plumbed `SpellAdditionalCost::SacrificeCreature`; sole gap is dynamic draw count via `EffectAmount::PowerOfSacrificedCreature` (currently `Fixed(1)` placeholder) |
| conclave_mentor | DEFERRED | needs `WhenThisDies` trigger + `EffectAmount::PowerOf(Source)` (different gap — not sacrifice-based) |
| jagged_scar_archers | DEFERRED | TODO claims "no PowerOf(Source)" — **STALE**, but TargetFilter "creature with flying" is also flagged (TargetFilter has `has_keywords` so this may also be stale; either way, out of PB-P scope) |
| krenko_tin_street_kingpin | DEFERRED | tokens count = source's power (different — not sacrificed) |
| master_biomancer | DEFERRED | dynamic ETB replacement (different DSL gap) |
| the_great_henge | DEFERRED | `SelfCostReduction::GreatestPower` (different DSL gap) |
| warstorm_surge | DEFERRED | already implemented via `EffectAmount::PowerOf(EffectTarget::TriggeringCreature)`; stale TODO comment can be cleaned in opportunistic sweep |
| ziatora_the_incinerator | DEFERRED | compound blocker: optional sacrifice as effect + reflexive trigger + power-based damage |
| ruthless_technomancer | DEFERRED | compound blocker: optional ETB sacrifice + variable `Cost::Sacrifice(X artifacts)` |
| miren_the_moaning_well | DEFERRED | needs **ToughnessOfSacrificedCreature** (different stat — out of PB-P narrow scope; candidate for a follow-up PB if ever needed) |
| diamond_valley | DEFERRED | same as Miren — `ToughnessOfSacrificedCreature` |
| birthing_pod | DEFERRED | dynamic search filter "MV = sacrificed creature's MV + 1" (different DSL gap) |

## Engine plumbing pre-survey (planner must verify)

- `EffectContext` (in `crates/engine/src/effects/mod.rs:48`) — does NOT currently carry `sacrificed_card_ids`. The planner must propose adding either:
  - `EffectContext.sacrificed_card_ids: Vec<ObjectId>` (LKI ID list, lookup in `state.objects` for power), OR
  - `EffectContext.sacrificed_powers: Vec<i32>` (capture power AT sacrifice time, before zone change), OR
  - A dedicated `EffectAmount::PowerOfSacrificedCreature` variant that reads from `EffectContext` directly.
- `AdditionalCost::Sacrifice(Vec<ObjectId>)` already preserves the IDs in `StackObject.additional_costs` for spells (read at `casting.rs:175` and `resolution.rs:1105`).
- For activated abilities, `Cost::Sacrifice(filter)` plumbing must be similarly verified — the planner walks the activated-ability sacrifice path (e.g., `Warren Soultrader`, `Vampiric Rites`, `Food Chain`, `Khalni Heart Expedition`) and confirms whether sacrificed IDs are already exposed to `EffectContext`.
- Key concern: **CR 608.2b LKI**. The sacrificed creature's characteristics MUST be read from the moment immediately before it left the battlefield, not from its graveyard state (which may differ if zone-change replacement effects, last-known anti-tampering, or other subsystems intervene). Planner must propose a clear capture point and document it.
- `EffectAmount::PowerOf(EffectTarget)` (existing, `effects/mod.rs:5805`) reads layer-resolved power from `state.objects` if zone is Battlefield, falling back to `obj.characteristics.power` otherwise. For PB-P this is **insufficient**: by resolve time, the creature is in graveyard and its layer-resolved power may differ from its on-battlefield LKI. Planner must justify whether to extend `PowerOf` (with a new EffectTarget variant) or add a dedicated `PowerOfSacrificedCreature` variant.
- Hash plumbing site: `crates/engine/src/state/hash.rs:4329` already handles `EffectAmount::PowerOf(target)`. New variant or new EffectTarget variant must be added to the hash dispatch.

## MANDATORY pre-plan steps for the planner

### Step 0: Stale-TODO sweep (mandatory)

Grep all card defs for "EffectAmount::PowerOfSacrificedCreature" or "PowerOf(SacrificedCreature)"
TODOs. Verify each surfaces in the CONFIRMED list above. Flag any miss as a forced add. Also
sweep for "warstorm_surge" stale TODO — the comment claims a gap that no longer exists; recommend
opportunistic cleanup (out of PB-P scope unless trivial).

### Step 1: CR research (mandatory)

- CR 701.16 — Sacrifice. Verify the timing of sacrifice as a cost (CR 117.1f, 601.2g) vs as
  an effect. Confirm that activated-ability sacrifice (Cost::Sacrifice) and spell additional-cost
  sacrifice (SpellAdditionalCost::SacrificeCreature) both happen as part of paying the cost and
  thus precede effect resolution.
- CR 608.2b — LKI. Confirm that the sacrificed creature's characteristics for resolution-time
  reads must use the values immediately before the sacrifice resolved (the "still on the
  battlefield" state).
- CR 400.7 — Object identity across zone change. Confirm that the engine's choice (preserve
  ObjectId across sacrifice OR snapshot characteristics at sacrifice time) is sound.
- CR 117.1f / 601.2g — Cost payment ordering relative to effect resolution.

### Step 2: Engine architecture walk (mandatory)

Trace the dispatch chain for both pathways:

1. **Spell additional cost path** (Life's Legacy):
   - Schema → `SpellAdditionalCost::SacrificeCreature` declared in `card_definition.rs`
   - Casting → `casting.rs:175,3155,3198` extracts `AdditionalCost::Sacrifice(ids)` and validates each ID against the filter
   - Resolution → `resolution.rs:1105` extracts the same `AdditionalCost::Sacrifice(ids)` from `StackObject.additional_costs` for resolution-time use
   - PROPOSED PB-P SITE: at resolution, set `EffectContext.sacrificed_card_ids` (or capture powers) before invoking effects

2. **Activated ability cost path** (Altar of Dementia, Greater Good):
   - Schema → `Cost::Sacrifice(TargetFilter)` in `card_definition.rs`
   - Activation → walk `crates/engine/src/rules/abilities.rs` for the activation cost-payment site (search for `Cost::Sacrifice(`)
   - Resolution → confirm whether sacrificed IDs flow into `EffectContext` via the activated-ability resolution path
   - PROPOSED PB-P SITE: at activation OR at effect-resolution, populate `EffectContext.sacrificed_card_ids`

3. **Hash dispatch** — `state/hash.rs:4329` for `EffectAmount::PowerOf(target)` plus the new variant arm

4. **Resolve dispatch** — `effects/mod.rs:5805` for `EffectAmount::PowerOf(target)` plus the new variant arm

### Step 3: Dispatch unification verdict (MANDATORY GATE)

Worker MUST record one of:
- **PASS-AS-NEW-EFFECT-AMOUNT-VARIANT**: add `EffectAmount::PowerOfSacrificedCreature` directly
- **PASS-AS-NEW-EFFECT-TARGET-VARIANT**: add `EffectTarget::SacrificedCreature` and reuse `EffectAmount::PowerOf(SacrificedCreature)`
- **SPLIT-REQUIRED**: discovery during walk requires multiple primitives; **STOP and flag to oversight**

Title and acceptance criteria use `EffectAmount::PowerOf(SacrificedCreature)` phrasing, suggesting
the EffectTarget variant route is preferred. Planner verifies feasibility (in particular: does
`PowerOf` resolution path correctly handle a non-battlefield LKI-style read?). If not, planner
recommends the dedicated EffectAmount variant route with rationale.

### Step 4: LKI capture-point decision

Planner proposes ONE of:
- **Capture-by-ID**: store `sacrificed_card_ids: Vec<ObjectId>` in `EffectContext`; rely on
  graveyard objects retaining same ObjectId and characteristics post-sacrifice (verify CR 400.7
  engine handling).
- **Capture-by-value**: snapshot power(s) AT sacrifice time, store `sacrificed_powers: Vec<i32>`
  in `EffectContext` (cleaner LKI semantics; avoids any post-sacrifice mutation risk).

Planner picks one with rationale and documents in the plan file.

### Step 5: Card roster fixes (mandatory)

Update each CONFIRMED card def. For Greater Good, ALSO clean the stale "Effect::DiscardCards
not in DSL" TODO. For Life's Legacy, replace `Fixed(1)` placeholder with the new variant.
For Altar of Dementia, author the activated ability from scratch (currently `abilities: vec![]`).

### Step 6: Test plan

Number every test MANDATORY or OPTIONAL with no silent skips. Required:
- M1: Altar of Dementia — sacrifice 5/5 creature, target player mills 5 (CR 701.16 + 608.2b)
- M2: Greater Good — sacrifice 3/3 creature, controller draws 3, then discards 3 (CR 608.2b + sequence ordering)
- M3: Life's Legacy — sacrifice 4/4 as additional cost on cast, draw 4 on resolve
- M4: LKI correctness — sacrifice a creature whose battlefield power was modified by an anthem (Glorious Anthem +1/+1); verify resolved amount uses LKI battlefield value, NOT graveyard base value
- M5: Multiple sacrifices in a single resolution (if the new variant supports vec semantics) — OR document why single-creature is sufficient
- M6: Hash determinism — same EffectAmount serializes to same hash; new variant does not collide with existing PowerOf(Source/etc.)
- M7: Backward compat — existing PowerOf(Source/DeclaredTarget/TriggeringCreature) cards still work (regression sweep on Swords to Plowshares, Souls' Majesty, Eomer)

OPTIONAL tests planner may add for edge coverage.

## Stop-and-flag triggers

- Dispatch unification gate fails (SPLIT-REQUIRED) → escalate to oversight, do not silently work around
- LKI capture point is structurally unsound (e.g., engine drops sacrificed object IDs before resolution and there is no clean place to snapshot power) → structural bug, escalate
- Hash sentinel bump policy unclear (default: bump 5→6, the PB-D bump went 4→5)
- Any CONFIRMED card has a hidden compound blocker that surfaces during the implement phase → drop the card; do not expand PB-P scope
- BASELINE-CLIPPY-01..06 baseline warnings are not in scope to fix

## Out of scope for PB-P

- `EffectAmount::ToughnessOfSacrificedCreature` (different stat — Miren, Diamond Valley) — separate future PB
- `EffectAmount::PowerOf(Source)` for activated-ability self-power reads (already exists; jagged_scar TODO is stale — opportunistic cleanup, not PB-P)
- Generic LKI infrastructure refactor — keep PB-P narrow to the sacrificed-creature read
- Reflexive triggers / optional sacrifice as effect (Ziatora) — different mechanic
- ETB replacement effects with dynamic counter counts (Master Biomancer)
- `Effect::DiscardCards` — already exists; Greater Good's stale TODO is incidentally cleaned during the card-def fix

## Standing rules to honor

- `memory/conventions.md`: test-validity MEDIUMs = fix-phase HIGHs; hash sentinel pub const + assert_eq!; implement-phase default-to-defer; aspirationally-wrong comments are hazards
- `memory/gotchas-rules.md`: CR 400.7 object identity; CR 608.2b LKI semantics
- `memory/feedback_verify_full_chain.md`: walk every dispatch site
- `memory/feedback_pb_yield_calibration.md`: filter PBs calibrate at 40-65% — PB-P at 23% is acknowledged narrow

## Artifacts the planner must produce

- `memory/primitives/pb-plan-P.md` (full plan file)
- Updated `memory/primitive-wip.md` checklist (this file) with all planner steps checked
- A 1-paragraph summary at the top of pb-plan-P.md naming:
  confirmed yield (3 cards), dispatch unification verdict, LKI capture decision, mandatory test
  count, deferred-card list, hash bump version

## Planner checklist (worker fills in)

- [x] Step 0: stale-TODO sweep complete; warstorm_surge note recorded (5 TODO hits found, 3 confirmed + 2 deferred + warstorm_surge confirmed already-implemented; recorded in pb-plan-P.md "Pre-existing TODO Sweep" section)
- [x] Step 1: CR research notes captured (701.16, 608.2b, 400.7, 117.1f, 601.2g) — full text excerpts in pb-plan-P.md "CR Rule Text" section
- [x] Step 2: engine architecture walk done; spell-cost path + activated-cost path + hash path traced — see pb-plan-P.md "Primitive Specification" + "Engine Changes" sections; verified `move_object_to_zone` kills OLD ObjectId at `state/mod.rs:409`, that NEW graveyard object inherits BASE characteristics (not LKI), and that activated abilities have NO existing additional_costs plumbing
- [x] Step 3: dispatch unification verdict recorded — **PASS-AS-NEW-EFFECT-AMOUNT-VARIANT** (NOT the title-suggested EffectTarget variant route); rationale: existing `PowerOf` resolution path requires live ObjectId and reads non-LKI base, both unfixable without a captured-integer side channel — at which point a dedicated EffectAmount variant is cleaner than overloading PowerOf
- [x] Step 4: LKI capture-point decision recorded — **CAPTURE-BY-VALUE** at every cost-payment site; capture-by-ID rejected as unsound under CR 400.7 (old ID dead) AND BASELINE-LKI-01 (new ID's calculate_characteristics drops battlefield-gated layer effects)
- [x] Step 5: card roster confirmed (3 cards) — Altar of Dementia (newly authored from `abilities: vec![]`), Greater Good (newly authored from `abilities: vec![]`, also cleans stale Effect::DiscardCards TODO), Life's Legacy (precision fix from `Fixed(1)` placeholder); per-card change plans in pb-plan-P.md "Card Definition Fixes"
- [x] Step 6: test plan numbered MANDATORY/OPTIONAL — **8 mandatory tests** (M1-M8) + 3 optional (O1-O3); M4 is the load-bearing LKI-correctness anchor (anthem-boosted creature sacrificed, captured value 3 vs. graveyard base 2)
- [x] Plan file written: `memory/primitives/pb-plan-P.md`
- [x] Wip file phase advanced to `plan-complete` for runner handoff

## Implementation checklist (runner fills in)

- [x] Engine change 1: `EffectAmount::PowerOfSacrificedCreature` variant added to `card_definition.rs`
- [x] Engine change 2: `AdditionalCost::Sacrifice` reshaped to struct `{ ids: Vec<ObjectId>, lki_powers: Vec<i32> }` in `card_definition.rs`
- [x] Engine change 3: `StackObject.sacrificed_creature_powers: Vec<i32>` field added (with `#[serde(default)]`)
- [x] Engine change 4: `EffectContext.sacrificed_creature_powers: Vec<i32>` scratch field added
- [x] Engine change 5: LKI capture-by-value wired at spell casting cost-payment site (`casting.rs`) — captures layer-resolved power before `move_object_to_zone`
- [x] Engine change 6: LKI capture-by-value wired at activated ability cost-payment site (`abilities.rs`) — populates `StackObject.sacrificed_creature_powers`
- [x] Engine change 7: `resolution.rs` spell path propagates `lki_powers` from `AdditionalCost::Sacrifice` into `EffectContext.sacrificed_creature_powers`
- [x] Engine change 8: `resolution.rs` activated-ability path propagates `StackObject.sacrificed_creature_powers` into `EffectContext.sacrificed_creature_powers`
- [x] Engine change 9: `EffectAmount::PowerOfSacrificedCreature` resolution arm added to `effects/mod.rs` (returns `sacrificed_creature_powers[0]` or 0)
- [x] Engine change 10: Hash arm added to `state/hash.rs` for new `EffectAmount` variant; `HASH_SCHEMA_VERSION` bumped 5→6
- [x] All exhaustive matches updated (cast_spell arm in `rules/*.rs`, EffectAmount arms in hash/effects)
- [x] Card def: `altar_of_dementia.rs` — authored from `abilities: vec![]`; activated ability with `Cost::Sacrifice` filter + `Effect::MillCards(PowerOfSacrificedCreature)`
- [x] Card def: `greater_good.rs` — authored from `abilities: vec![]`; activated ability + stale TODO cleaned
- [x] Card def: `lifes_legacy.rs` — `Fixed(1)` placeholder replaced with `PowerOfSacrificedCreature`
- [x] Tests written: `crates/engine/tests/pbp_power_of_sacrificed_creature.rs` — 9 active tests (M1-M8 + O3), 1 `#[ignore]` (O1 priority loop)
- [x] All existing tests pass: `cargo test --all` — 0 failures
- [x] `cargo build --workspace` clean
- [x] `cargo fmt --check` clean
- [x] Clippy: 8 pre-existing `collapsible_match` warnings in baseline (confirmed by stash check); 0 new warnings introduced by PB-P
- [x] HASH_SCHEMA_VERSION sentinel assertions updated in `pbd_damaged_player_filter.rs` and `pbn_subtype_filtered_triggers.rs` (5→6)
- [x] `AdditionalCost::Sacrifice` tuple→struct converted at all 43 test-file construction sites
- [x] `StackObject.sacrificed_creature_powers` added to all 15+ direct struct-literal constructions in test files

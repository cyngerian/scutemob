# Primitive WIP: PB-D — TargetController::DamagedPlayer

batch: PB-D
title: DamagedPlayer as a TargetController variant for "that player controls" scoping
cards_unblocked_estimated: ~15 classified, ~7-8 post-calibration estimate, **6 confirmed** post-roster-verification (filter-PB range 50-65%, actual ≈40%)
started: 2026-04-13 (planner session)
phase: plan-complete
plan_file: memory/primitives/pb-plan-D.md (WRITTEN 2026-04-13 by Opus planner session)

## How this PB was selected

Rank 1 of the post-PB-N re-discounted queue
(`docs/primitive-card-plan.md` Phase 1.8, "Post-PB-N queue re-discount"
table). Fresh batch, no dependencies. Filter-PB yield calibration
unchanged at 50-65% per `memory/feedback_pb_yield_calibration.md`.

## MANDATORY pre-plan steps for the worker

### Step 0: Stale-TODO sweep — SKIPPED (no stale candidates for this bucket)

The classification report (`memory/card-authoring/todo-classification-2026-04-12.md`
lines 22-32) flagged 3 potentially-stale TODOs for PB-N's pre-launch
sweep; all 3 were addressed in that sweep. Re-reading the
DamagedPlayer bucket (lines 119-125), **no new stale candidates are
flagged**. Recorded in plan preamble as "Step 0 sweep: no stale
candidates".

### Step 1: PB-P pre-check (5-minute sanity on rank-2 PB)

Before assuming PB-P (PowerOfCreature EffectAmount, rank 2) is a real
PB, grep for existing `EffectAmount::PowerOf*` variants. Report only,
do not act — oversight picks the next slate after PB-D closes.

**Result**: `EffectAmount::PowerOf(EffectTarget)` **already exists** and
is widely used (Swords to Plowshares, Souls' Majesty, Marwyn,
Warstorm Surge, Eomer King of Rohan). It accepts `Source`,
`DeclaredTarget`, `Self_`, and `TriggeringCreature` as targets. The
real PB-P gap is **narrower**: it is specifically
`EffectAmount::PowerOf(EffectTarget::SacrificedCreature)` — an
LKI-based read at the moment of sacrifice — needed by Altar of
Dementia and Greater Good (which explicitly name
`EffectAmount::PowerOfSacrificedCreature` in their TODOs). Several
PB-P sample cards (e.g. The Great Henge) are blocked on entirely
different things (SelfCostReduction::GreatestPower). **PB-P is not a
stale-TODO sweep** — it is a real but narrower primitive than the
queue name suggests. Documented in the PB-D plan preamble for
oversight's post-PB-D slate decision. No action taken.

### Step 2: PB-D plan proper

Standard `primitive-impl-planner` flow. Required artifacts:

1. **CR research** — walk the combat-damage trigger event model (CR
   510.1, 510.3a for the event; CR 603.2 for "whenever deals combat
   damage to a player" triggering; CR 601.2c for target selection at
   trigger-put-on-stack time). Verify that the "damaged player"
   identity is well-defined at target selection time (it is — it's on
   the triggering combat-damage event).

2. **Engine architecture study** — walk TargetController dispatch sites
   via Grep + Read. Prior-art field already plumbed end-to-end:
   - `PendingTrigger.damaged_player: Option<PlayerId>` (already set at
     trigger-queue time in `abilities.rs:4440..4978`)
   - `StackObject.damaged_player: Option<PlayerId>` (already populated
     at `abilities.rs:7200`)
   - `EffectContext.damaged_player: Option<PlayerId>` (already
     populated at `resolution.rs:2031,2099`)
   - `PlayerTarget::DamagedPlayer` resolves via
     `ctx.damaged_player.unwrap_or(ctx.controller)` in
     `effects/mod.rs:2895,2945`
   - `PlayerState.damage_received_this_turn: u32` exists but is
     **unrelated** to PB-D (it's the Bloodthirst cumulative-damage
     counter, reset at turn start in `turn_actions.rs:1499`). PB-D
     uses the per-event combat-damage binding via `PendingTrigger`,
     not the turn counter.

3. **Dispatch unification verification (MANDATORY GATE)**: the PB-D
   feature is a new `TargetController::DamagedPlayer` variant. Verdict
   must be recorded as PASS-AS-NEW-VARIANT, PASS-AS-FIELD-ADDITION, or
   SPLIT-REQUIRED. **Stop-and-flag if SPLIT-REQUIRED** before
   continuing.

4. **Card roster verification** — MCP lookup / Read + TODO sweep on
   the 15 classification candidates. Apply filter-PB calibration
   (50-65% yield). Drop compound-blocker cards. Record each with
   one-line status (CONFIRMED / DEFERRED + reason).

5. **Step 3a (MANDATORY)** — pre-existing TODO sweep.
   Per `feedback_planner_roster_recall.md` and the new mandatory step
   in `.claude/agents/primitive-impl-planner.md`, grep the card defs
   for any TODO naming `DamagedPlayer` or "that player controls" and
   mark every hit as a **forced add**. Record the sweep result
   positively ("N cards found" or "0 cards found") in the plan
   preamble.

6. **Test plan** — number every test MANDATORY or OPTIONAL with no
   silent skips. Each filter dispatch site must be exercised.

### Step 3: Standing rules to honor

- `memory/conventions.md`: test-validity MEDIUMs = fix-phase HIGHs;
  hash sentinel pub const + assert_eq!; implement-phase
  default-to-defer; aspirationally-wrong comments are hazards
- `memory/gotchas-rules.md` gotcha #39: subtype-filter test wedge
  discipline. **Note for PB-D: player filters are NOT zone-change
  sensitive** — the damaged player identity is bound on the
  `PendingTrigger` before the trigger enters the stack, and players
  don't change zones, so BASELINE-LKI-01 does NOT reach PB-D. Verify
  this assumption holds during plan walk.
- `memory/feedback_verify_full_chain.md` — walk every dispatch site
- `memory/feedback_planner_roster_recall.md` — MCP lookup + TODO grep
  are complementary, run both
- `memory/feedback_oversight_primitive_category_not_cards.md` —
  oversight named "DamagedPlayer target filter"; planner verifies
  card-level scope from oracle text
- `memory/feedback_pb_yield_calibration.md` — filter PBs calibrate at
  50-65% yield

## Stop-and-flag triggers (escalate to oversight, do not silently work around)

- Dispatch unification gate fails (split required)
- PB-P pre-check reveals it's stale-TODO territory (**report, don't act**)
- Any card in the roster has a compound blocker outside PB-D scope
  (drop the card; do not expand PB-D surface)
- `damage_received_this_turn` (or equivalent storage) reset semantics
  are wrong for the filter's needs (structural bug, escalate). **NOTE:
  PB-D does NOT use `damage_received_this_turn`** — it uses the
  PendingTrigger.damaged_player per-event binding.
- BASELINE-LKI-01 pattern reaches PB-D (it should NOT — player filters
  don't depend on layer-resolved-then-zone-changed characteristics —
  but if you find it does, stop)
- Hash sentinel bump policy unclear (default: bump 4→5, note in plan)

## Out of scope for PB-D

- PB-P (PowerOfCreature / PowerOfSacrificedCreature) — rank 2, separate
- PB-L (Landfall) — rank 3, separate
- Any life-loss-based filter (different primitive, future PB)
- Any new EffectFilter variant unless strictly required for one roster card
- BASELINE-LKI-01 investigation (dedicated audit session, not PB-D)
- Pre-existing clippy baseline warnings (BASELINE-CLIPPY-01..06)
- PlayerTarget::ControllerOfTriggeringCreature (different primitive — blocks Edric)
- PlayerTarget::TriggeringPermanentController (different primitive — blocks Horn of Greed, Blood Seeker)
- ForEach over "creatures controlled by declared target player" for spells (not triggers)
  — different primitive, blocks Polymorphist's Jest

## Planner checklist (worker fills in)

- [x] Step 0: no stale candidates for DamagedPlayer bucket — recorded positively in plan preamble
- [x] Step 1: PB-P pre-check complete — `EffectAmount::PowerOf` exists; real gap is `PowerOfSacrificedCreature`; PB-P is a real but narrower PB; not stale-sweep territory; report only, no action taken
- [x] Step 2.1: CR research notes captured — CR 510.1 (combat damage assignment), 510.3a (triggers from damage dealt), 603.2 (trigger event matching), 601.2c (target selection at trigger-stack-entry time). Combined with source-level observation that `PendingTrigger.damaged_player` is bound at trigger-queue time, before target selection.
- [x] Step 2.2: engine architecture walk done via Grep + Read. Dispatch sites mapped:
  - `TargetController` enum definition at `crates/engine/src/cards/card_definition.rs:2385-2392` (3 variants: Any, You, Opponent)
  - `TargetController` hash at `crates/engine/src/state/hash.rs:4137-4143`
  - TargetController match sites for filter dispatch:
    - `effects/mod.rs:869-873` (DestroyAll dispatch)
    - `effects/mod.rs:1050-1052` (ExileAll dispatch)
    - `effects/mod.rs:1155-1157` (DestroyAll second site)
    - `effects/mod.rs:5410-5412` (match-filter-controller path 1)
    - `effects/mod.rs:7203-7206` (ForEach EachPermanentMatching dispatch — load-bearing for Nature's Will + Balefire Dragon)
    - `casting.rs:5521-5526` (TargetCreatureWithFilter validation)
    - `casting.rs:5533-5538` (TargetPermanentWithFilter validation)
    - `abilities.rs:6461-6468` (triggered-ability auto-target for TargetCreatureWithFilter — load-bearing for Throat Slitter, Sigil of Sleep, Alela, Mistblade Shinobi)
    - `abilities.rs:6474-6482` (triggered-ability auto-target for TargetPermanentWithFilter)
  - PendingTrigger / StackObject / EffectContext `damaged_player` plumbing already end-to-end (set at `abilities.rs:4440`, `4501`, `4787`, `4964`, `4978`, `5017`, `7200`; consumed at `resolution.rs:2031,2099` and `effects/mod.rs:2895,2945`).
  - Hash / builder / replay_harness dispatch sites for TargetController: `state/hash.rs`, `testing/replay_harness.rs:2445,2776-2782` (match! patterns — fall-through safe but need a new arm if exhaustive).
- [x] Step 2.3: dispatch unification gate verdict — **PASS-AS-NEW-VARIANT** (single new `TargetController::DamagedPlayer` variant, ~10 match sites need an arm added). Mirrors the existing PlayerTarget::DamagedPlayer resolution pattern. Recorded in plan "Dispatch unification verdict" and "Engine Changes" sections.
- [x] Step 2.4: 15-card roster MCP/Read verification complete. Many classification entries are false positives (Memory Lapse, Leyline of the Void, The Eternal Wanderer) or compound-blocked (Marisi, Skullsnatcher, Ink-Eyes, Hellkite Tyrant, Dokuchi Silencer, Ragavan, Grenzo, Scalelord Reckoner). Some already implemented (Sword of Feast and Famine, Sword of Body and Mind, Sword of War and Peace, Lightning Army of One). **Confirmed yield: 6 cards** (Throat Slitter precision, Sigil of Sleep precision, Mistblade Shinobi new, Alela Cunning Conqueror partial, Nature's Will partial, Balefire Dragon new). Yield = 40% (below filter-PB range, slight under-performance flagged in plan Risks).
- [x] Step 2.5 / Step 3a: pre-existing TODO sweep run — 7 cards have TODOs explicitly naming DamagedPlayer / "TargetController::DamagedPlayer". 6 become forced-add confirmed entries; 1 (Marisi) is compound-blocked and deferred with reason. Recorded positively in plan preamble.
- [x] Plan file written: `memory/primitives/pb-plan-D.md`
- [x] Wip file phase advanced to `plan-complete` for oversight handoff

## Artifacts the planner must produce

- `memory/primitives/pb-plan-D.md` (full plan file)
- Updated `memory/primitive-wip.md` checklist (this file) with all planner steps checked
- A 1-paragraph summary at the top of pb-plan-D.md naming:
  confirmed yield, dispatch unification verdict, mandatory test count,
  deferred-card list, PB-P pre-check verdict, Step 0 sweep scope.

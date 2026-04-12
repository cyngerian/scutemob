# Primitive WIP: PB-N — SubtypeFilteredAttack + SubtypeFilteredDeath triggers

batch: PB-N
title: Combined subtype-filtered attack and death triggers (single dispatch site)
cards_unblocked_estimated: ~33 considered (18 attack + 15 death); ~20 yield expected post 60% calibration
started: 2026-04-12 (oversight session — re-prioritization)
phase: plan
plan_file: memory/primitives/pb-plan-N.md (TO BE WRITTEN by next worker session)
review_file: (TBD)

## How this PB was selected

Top of new data-driven priority queue (2026-04-12 re-slate, commit `70757770`).
See `docs/primitive-card-plan.md` Phase 1.8 and
`memory/card-authoring/todo-classification-2026-04-12.md` for the report.

PB-N combines classification ranks 1 (SubtypeFilteredAttack, 18 cards) and
3 (SubtypeFilteredDeath, 15 cards) because they share the same dispatch
pattern: a trigger condition that fires on a specific event (attack /
death) filtered by the triggering object's creature subtype. The planner
should verify in the first 10 minutes whether this combination holds — if
the dispatch sites are actually different, split into PB-N (attack) and a
follow-up PB.

## MANDATORY pre-plan steps for the worker

### Step 0: Stale-TODO sweep (do FIRST, before any planning work)

Three card defs were flagged in the classification report (line 26-32) as
potentially stale post PB-S / PB-Q. Initial oversight grep results:

| Card | TODO Line | Initial Verdict | Worker Action |
|------|-----------|-----------------|---------------|
| `song_of_freyalise` | "via PB-S LayerModification::AddManaAbility" | **NOT STALE** — comment is misleading. PB-S added grant-ability primitives but Saga chapter machinery is the real blocker. | Update comment wording to remove the PB-S misdirection; leave card blocked on Saga primitive. |
| `bootleggers_stash` | "Lands you control gain activated ability" | **LIKELY STALE-FIXABLE** — PB-S shipped `LayerModification::AddManaAbility` + `AddActivatedAbility`. Verify whether the existing grant supports "lands you control" filter (vs "permanents you control"). | If filter supports it, author the card. If not, log as a 1-line micro-PB candidate (filter extension) and move on. |
| `throne_of_eldraine` | "ChosenColor designation not in DSL" | **PARTIAL STALE** — PB-Q shipped ChosenColor as a designation. The static spending restriction is still PB-Q5 territory and the activation-time choice is PB-Q2 territory. | Update the card's TODO comments to reflect what's now expressible vs what's still blocked; cite PB-Q2 / PB-Q5 reservations. |

Commit the stale sweep as a separate prep commit BEFORE the PB-N plan
lands: `W6-prim: stale-TODO sweep — PB-N pre-launch (3 cards)`.

### Step 1: Verify PB-L is not collapsing into a stale-TODO sweep

Before assuming PB-L (Landfall, rank 4) is a real PB, grep the engine
for `WhenEntersBattlefield` + `EffectFilter::Land` patterns and check
whether the existing trigger condition + filter combo already covers
landfall. If yes, demote PB-L to a stale-TODO sweep instead of a full
PB. This is a 5-minute check the planner should do while still in PB-N
plan mode (no extra context cost). Report finding in the PB-N plan
preamble; do not act on it (let oversight call the next slate).

### Step 2: PB-N plan proper

Standard `primitive-impl-planner` flow. Required artifacts:

1. **CR research**: cite the trigger event sources (CR 603.2 for
   triggered abilities, CR 509.2 for declaring attackers as the event
   point for attack triggers, CR 700.4/704 for death events). Identify
   the exact intervening-if vs ETB ordering for filtered death triggers
   (CR 603.10 — zone-change triggers look back).

2. **Engine architecture study**: read the trigger-firing site that
   currently handles `WhenAttacks` and `WhenDies` (or whatever the
   nearest variants are). Use rust-analyzer to walk:
   - `TriggerCondition` enum definition + all match sites
   - `combat.rs` declare-attackers event emission
   - The death-triggered-ability fan-out site in `sba.rs` /
     `resolution.rs` / `zones.rs`
   - Hash dispatch for `TriggerCondition` (ensure new variant gets a
     hash arm)

3. **Dispatch unification verification (MANDATORY GATE)**: confirm the
   attack-side and death-side dispatch sites can take a single new
   variant (e.g. `TriggerCondition::SubtypeFilteredEvent { event,
   subtype }`) or whether they need two parallel variants. **Do not
   skip this gate** — if the verification fails, split the PB and
   stop-and-flag to oversight before continuing.

4. **Card roster verification**: MCP-look-up oracle text for all 33
   candidate cards from the classification report:
   - **Attack-side (18)**: aqueous_form, argentum_armor, battle_cry_goblin,
     bear_umbra, diamond_pick_axe, dreadhorde_invasion, dromoka_the_eternal,
     hellrider, hermes_overseer_of_elpis, kazuul_tyrant_of_the_cliffs,
     kolaghan_the_storms_fury, najeela_the_blade_blossom, quilled_charger,
     sanctum_seeker, shared_animosity, +3 more in classification report
   - **Death-side (15)**: anafenza_unyielding_lineage, athreos_god_of_passage,
     blade_of_the_bloodchief, crossway_troublemakers, luminous_broodmoth,
     marionette_apprentice, miara_thorn_of_the_glade, morbid_opportunist,
     omnath_locus_of_rage, pashalik_mons, patron_of_the_vein,
     serpents_soul_jar, skullclamp, teysa_orzhov_scion, thornbite_staff
   - For each card, confirm: (a) the trigger really is subtype-filtered
     (not just creature-typed), (b) the rest of the card body is
     authorable post-PB-N, (c) no compound blocker.
   - **Apply 60% yield discount** per `feedback_pb_yield_calibration.md`
     — expect ~20 cards actually shippable, not 33. Drop the rest into
     "deferred" with a one-line reason.

5. **Test plan**: number every test as MANDATORY or OPTIONAL up front.
   No silent skips (per PB-Q4 retro). Minimum mandatory:
   - One attack-side dispatch test (new variant fires on attack)
   - One death-side dispatch test (new variant fires on death)
   - One subtype-filter negative test (different subtype does not fire)
   - One LKI test (death-side reads pre-zone-change subtype, CR 603.10)
   - One hash parity test for the new variant
   - One real-card end-to-end (e.g., Shared Animosity attack or Skullclamp death)

6. **Standing rules to honor**:
   - `feedback_verify_full_chain.md` — walk every dispatch site, not just the file touched
   - `feedback_oversight_primitive_category_not_cards.md` — oversight named the category; you verify rosters from MCP
   - Every new layer/dispatch variant ships with a full-dispatch test (`memory/conventions.md`)
   - PB planner overcounts — discount 40-50% from ANY yield estimate

## Stop-and-flag triggers (escalate to oversight, do not silently work around)

- Dispatch unification gate fails (attack and death need separate sites)
- PB-L Landfall check reveals it's a stale-TODO sweep (no new info needed, but report it)
- Any card in the roster reveals a hidden compound blocker (e.g., needs a target filter that isn't the PB-N scope)
- Hash version bump policy unclear (we're at sentinel 3 post-PB-Q)

## Out of scope for PB-N

- PB-D (DamagedPlayer) — separate, queued next
- PB-L (Landfall) — separate, queued third (verify it's still a PB first)
- PB-P (PowerOfCreature) — separate, queued fourth
- Any non-subtype-filtered trigger condition
- Any new EffectFilter variant unless strictly required to author one of the verified roster cards

## Planner checklist (worker fills in)

- [ ] Step 0: stale-TODO sweep committed (separate commit, prefix `W6-prim:`)
- [ ] Step 1: PB-L landfall pre-check completed; finding noted in PB-N plan preamble
- [ ] Step 2.1: CR research notes captured
- [ ] Step 2.2: rust-analyzer walk of TriggerCondition dispatch sites done
- [ ] Step 2.3: dispatch unification gate verdict (PASS/FAIL/SPLIT) recorded
- [ ] Step 2.4: 33-card MCP roster verification complete with 60% yield discount applied
- [ ] Step 2.5: mandatory/optional test labels assigned with no silent skips
- [ ] Plan file written: `memory/primitives/pb-plan-N.md`
- [ ] Wip file phase advanced to `plan-complete` for oversight handoff

## Artifacts the planner must produce

- `memory/primitives/pb-plan-N.md` (full plan file)
- Updated `memory/primitive-wip.md` checklist (this file) with all planner steps checked
- A 1-paragraph summary at the top of pb-plan-N.md naming: confirmed yield, dispatch unification verdict, mandatory test count, deferred-card list

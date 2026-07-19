# Primitive WIP — PB-OS11 (IN PROGRESS) — CLOSES THE PB-OS QUEUE

<!-- last_updated: 2026-07-19 -->

**Task**: `scutemob-141` · branch `feat/pb-os11-final-queue-batch-cost-payment-lki-counters-oos-lki-`
**Phase**: review
**Plan file**: `memory/primitives/pb-plan-OS11.md`
**Review file**: `memory/primitives/pb-review-OS11.md`

## IMPLEMENT PHASE COMPLETE (2026-07-19)

- [x] **A-Change-1..7** — `ManaAbility.remove_counter: Option<(CounterType, u32)>` added
  (`crates/card-types/src/state/game_object.rs`); `ManaAbilityCost.remove_counter` +
  `Cost::RemoveCounter` acceptance + no-tap guard relaxation in
  `mana_ability_cost_components` + carried through `mana_ability_lowering`
  (`crates/engine/src/testing/replay_harness.rs`); pre-check + payment step (5b2 / 6d) in
  `handle_tap_for_mana` (`crates/engine/src/rules/mana.rs`), reusing
  `GameEvent::CounterRemoved`; `HashInto for ManaAbility` hashes the new field
  (`crates/engine/src/state/hash.rs`).
- [x] **A-Card-1** — `workhorse.rs` authored NEW → `Complete`.
- [x] **A-Backfill** — `gemstone_array` / `druids_repository` execution-verified and
  flipped `known_wrong` → `Complete` (see report; chosen-color mana confirmed via
  `TapForMana{chosen_color}`).
- [x] **B-Change-1..5** — `TriggerCondition::WheneverYouAttack` unit → struct `{ filter:
  Option<TargetFilter> }` (`crates/card-types/src/cards/card_definition.rs`);
  `HashInto` arm updated; `ControllerAttacks` filter branch added to
  `collect_triggers_for_event` (`crates/engine/src/rules/abilities.rs`); the
  WheneverYouAttack conversion site now threads `filter` into
  `triggering_creature_filter` (`replay_harness.rs`); all bare-unit construction sites
  migrated to `{ filter: None }` (5 card defs + `trigger_variants.rs`).
- [x] **B-Card-1/2/3** — `anim_pakal_thousandth_moon` (`known_wrong`→`Complete`),
  `general_kreat_the_boltbringer` (`partial`→`Complete`), `hermes_overseer_of_elpis`
  (`partial`→`Complete`).
- [x] **Wire** — HASH 62→63 (as predicted). PROTOCOL 25→26 (**deviates from the plan's
  "NO bump" prediction** — the `protocol_schema` machine gate demanded it: `ManaAbility`
  is inside the wire closure via `Characteristics.mana_abilities`, exactly the SR-34/36/37
  precedent the plan's own doc-comments describe; `TriggerCondition`/`TriggerEvent` are
  confirmed outside the closure per the OS10 note, so Part B stayed HASH-only as
  predicted). Gate machine-computed fingerprints applied verbatim (no guessed hex).
- [x] **Tests** — 19 new executing tests across
  `crates/engine/tests/primitives/pb_os11_remove_counter_mana_ability.rs` (9 tests, Part A)
  and `crates/engine/tests/primitives/pb_os11_batch_filtered_attack_trigger.rs` (10 tests,
  Part B); both registered in `crates/engine/tests/primitives/main.rs`.
- [x] **Gate bumps riding along**: `bare_lookup_ratchet` ceilings raised
  `abilities.rs` 74→75, `mana.rs` 7→8 (both NONSWALLOW predicate reads, documented
  inline); `completeness_deviation_scan` ALLOWLIST gained `anim_pakal_thousandth_moon`
  (accepted LKI-edge deviation language) and its marker floor lowered 672→662 (3 more
  defs flipped to Complete).
- [x] **Full verification green**: `cargo build --workspace`, `cargo test --all`
  (all suites, 0 failures), `cargo clippy --workspace --all-targets -- -D warnings`
  (0 warnings), `cargo fmt --all -- --check`, `tools/check-defs-fmt.sh` (all clean).

Next: primitive-impl-reviewer reads `memory/primitives/pb-plan-OS11.md` + this file and
writes `memory/primitives/pb-review-OS11.md`.

## PLAN COMPLETE (2026-07-19) — both premises reframed against MCP source; scope grew via TODO sweep

**Read `memory/primitives/pb-plan-OS11.md` — it supersedes the pre-plan chain-verification below
where they differ.** Two headline corrections the runner MUST honor:

- **OOS-LKI-3 premise was STALE.** MCP `lookup_card "Workhorse"` = *"This creature enters with four
  +1/+1 counters on it. / Remove a +1/+1 counter from this creature: Add {C}."* — **NO sacrifice, NO
  X-mana.** The pre-plan `SacrificedCreatureLki`-counter-capture design (below, items 1) is **MOOT**
  — no card in the corpus needs it. **Reframed** to the real gap Workhorse exercises: a
  `Cost::RemoveCounter` **mana** ability with no `{T}` cannot be lowered to a true mana ability
  (CR 605.1a) — the exact gap `druids_repository.rs` / `gemstone_array.rs` `known_wrong` notes call
  out. Fix = `ManaAbility.remove_counter` field + accept `Cost::RemoveCounter` in
  `mana_ability_cost_components` + relax the no-tap guard + pay in `handle_tap_for_mana` (reuse the
  existing `GameEvent::CounterRemoved`). Workhorse authored NEW → Complete (fixed {C}, so it dodges
  the any-color color bug). Opportunistic backfill: gemstone_array / druids_repository may flip
  known_wrong→Complete (execution-verify; the color bug resolves on the lowered any-color path).

- **OOS-TS-1 re-scope was STALE.** `TargetFilter.exclude_subtypes` already EXISTS and is enforced in
  `matches_filter`. And `WheneverCreatureYouControlAttacks{filter}` is the WRONG trigger — it fires
  once PER matching attacker (over-triggers). Anim Pakal is a **batch** trigger firing ONCE. Real fix
  = **filter on the once-firing batch trigger**: `TriggerCondition::WheneverYouAttack` unit→struct
  `{ filter: Option<TargetFilter> }`, applied via a new `ControllerAttacks` branch in
  `collect_triggers_for_event` that reads `state.combat.attackers` and fires once iff ≥1 declared
  attacker matches.

- **TODO-sweep gate (mandatory) = 2 FORCED ADDS** beyond the 2-card brief: `general_kreat_the_boltbringer`
  (Goblins) and `hermes_overseer_of_elpis` (Birds) both self-identify in source as needing this exact
  batch-filtered-attack primitive. Final roster: **workhorse (NEW) + anim_pakal + general_kreat +
  hermes = 4 guaranteed clean flips** (+ up to 2 backfill). `najeela` stays ENGINE-BLOCKED (needs more).

- **Wire CORRECTED: HASH 62→63 only; expect NO PROTOCOL bump.** `TriggerCondition` is outside the
  wire closure (protocol.rs v25/PB-OS10 note) and `ManaAbility` field additions are HASH-only by
  precedent (SR-34 v41, PB-EF8 v51). Runner confirms via `protocol_schema` gate — it should PASS
  UNCHANGED; bump PROTOCOL 25→26 ONLY if the machine fingerprint actually moves.

---

## Scope — two independent singleton fixes (OOS-LKI-3 + OOS-TS-1)

Source: `memory/primitives/oos-retriage-plan-2026-07-18.md` §3 (PB-OS11 entry), §2 (re-scope),
§5 (dispatch-loop notes binding). Candidates: `workhorse`, `anim_pakal_thousandth_moon`.

### Chain-verification findings (done pre-plan — SUPERSEDED IN PART by the plan; see corrections above)

1. **OOS-LKI-3** — `SacrificedCreatureLki` (PB-EF10, `card-types/src/state/types.rs:211`) carries
   `{ power, toughness, mana_value }` — **NOT counters**. [SUPERSEDED: modern Workhorse does not
   sacrifice; the SacrificedCreatureLki path is not used. See plan Part A — RemoveCounter mana-ability
   lowering.]
2. **OOS-TS-1** — `TargetFilter.exclude_subtypes: Vec<SubType>` **ALREADY EXISTS**
   (`card_definition.rs:3132`) and is **enforced in `matches_filter`** (`effects/mod.rs:8856`).
   [CORRECTED: use `WheneverYouAttack { filter }` (batch, fires once), NOT
   `WheneverCreatureYouControlAttacks{filter}` (per-creature, over-triggers). See plan Part B.]

### Wire

[CORRECTED — see PLAN COMPLETE above.] **HASH 62→63 only; expect NO PROTOCOL bump.**

### Guardrails (§5)
- No gated-stub effects in authoring (barred from `Complete`).
- Probe by execution, not source-tracing (SR-34/36): each flipped card needs an executing test
  (this INCLUDES the gemstone/druids backfill flip decision).
- Verify oracle text directly via MCP for both cards. [DONE — both verified; Workhorse premise was stale.]

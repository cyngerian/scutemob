# PB-DX35 — implementation plan

Read `memory/primitives/pb-DX35-execution-notes.md` §0 FIRST. It carries the baseline, the wire
prediction (written before any production line) and the two design decisions. **Nothing in this
plan may change a fingerprint, a version constant or a sentinel.** If a schema gate reddens,
STOP and report — do not edit a number.

Standing rules that bind every step here:
* A site list in a seed row, a memo cell or in this plan is a **FLOOR**. Re-derive it.
* SR-36: enumerate `mtg_card_defs::all_cards()` for every roster. **Never grep source for a
  population.**
* A comment is a claim. If a comment you touch asserts something this batch makes false,
  rewrite it in the same commit.
* Every new gate/probe must be proven RED by an executed revert. Record the row.

---

## HALF B — `OOS-DX4-5`: make `Effect::LookAtTopThenPlace.optional` real

### B1. The engine change (`crates/engine/src/effects/mod.rs`, the `LookAtTopThenPlace` arm)

The arm currently destructures `optional: _` (with a comment saying it is inert). Change it to
bind `optional` and honour it.

Today the winner is picked as:

```rust
let mut placed_id: Option<ObjectId> = None;
if placement_allowed {
    placed_id = top_ids.iter().copied().filter(|&id| { ...filter + caps... }).min_by_key(|id| id.0);
}
```

Replace with: build the matching set as a `Vec<ObjectId>` **sorted ascending by `ObjectId`**
(`top_ids` is `Zone::top_n` order, i.e. TOP-FIRST, not id order — so an explicit sort is
required and `candidates.first()` then equals today's `min_by_key(|id| id.0)` EXACTLY; this
equality is the whole behaviour-preservation argument and a probe must pin it). Then:

* `optional == false` **or** `candidates.is_empty()` → `placed_id = candidates.first().copied()`
  (today's behaviour, byte-for-byte).
* `optional == true` and candidates non-empty → ask
  `EffectChoiceQuestion::ChooseObject { candidates: candidates.clone(), count: 1, up_to: true }`
  through `ask_or_consume_effect_choice(state, ctx, p, ..)` — the SAME helper the `place_cost`
  branch three lines up already uses, addressed to `p` (the looking player), not
  `ctx.controller`.
  * `Some(EffectChoiceAnswer::ChooseObject { chosen })` → `placed_id = chosen.first().copied()`
    (`None`/empty = the DECLINE).
  * `Some(other)` → `debug_assert!(false, ...)` + fall back to `candidates.first().copied()`,
    mirroring the `place_cost` branch's own mismatch arm.
  * `None` (suspended) → **`break`**, not `continue` — identical to the `place_cost` branch and
    for the identical reason (a later payer must not run their look; the wrapper is about to
    discard the whole pass).

**Do NOT add a determined-answer short-circuit for `candidates.len() == 1`.**
`resolve_pending_object_choices` has one, and it is correct there because `!up_to` makes a
single candidate the only legal SET. Here `up_to: true` means declining is always a second legal
answer, so a lone candidate is still a real choice. State this in the code comment.

**Ask ordering with `place_cost` is load-bearing and must be preserved**: the cost question is
asked first and its payment sets `ctx.sacrificed_creature_lki`, which parameterises
`filter.max_cmc_amount` and therefore the candidate set. `birthing_ritual` is the only corpus
def with both, and it asks BOTH questions in one resolution — CR-correct, its printed text is
*"Then you may sacrifice a creature. If you do, **you may put** a creature card ... onto the
battlefield"* (MCP, verbatim). A probe must drive both questions in that order.

`default_effect_choice_answer`'s `ChooseObject` arm already returns `candidates.take(count)` =
the first candidate = today's winner, so **no change to it is owed**; verify and say so.

### B2. Comments this change falsifies — rewrite ALL of them in the same commit

1. The arm's own `optional: _` comment (*"is not read by this M7 deterministic executor …
   Currently inert, not a live gate"*).
2. `crates/card-types/src/state/stubs.rs`, `EffectChoiceQuestion::ChooseObject`'s doc:
   *"this one names **public** information: every id is a permanent on the battlefield or a card
   in a graveyard, both public zones"*. **False after this batch** — `LookAtTopThenPlace` hands
   it LIBRARY ids. Rewrite to state the two populations and why redaction is still correct
   (`GameEvent::EffectChoiceRequired` is `private_to(player)`, the same channel `SearchLibrary` /
   `Scry` / `Surveil` already carry library ids on).
3. `tools/play-server/src/view.rs`'s `ChooseObject` arm comment (*"`candidates` name PUBLIC
   objects (battlefield permanents / graveyard cards)"*). Same correction. **No code change is
   owed there** — it already renders through `question_cards`, the channel `SearchLibrary` uses
   for library cards; verify that and say so rather than assuming it.
4. `crates/engine/tests/primitives/pb_dp9_effect_choice.rs::test_dp9_mana_ability_gate`'s
   justification for the `LookAtTopThenPlace` needle (*"five corpus defs carry it and only one
   sets a `place_cost`, so this gate would flag a mana ability containing any of the five … it
   is over-wide, deliberately"*). The needle itself does **not** change; its reason does — after
   this batch all five genuinely ask, so it is over-wide only for a hypothetical
   `optional: false, place_cost: None` def, of which the corpus has zero.
5. All five card defs' in-source notes that say `optional` is inert / `optional: _` is
   destructured away (`birthing_ritual`, `growing_rites_of_itlimoc`, `grisly_salvage`,
   `satyr_wayfinder`, `risen_reef`). These are **comment-only** edits — no
   `Completeness` marker moves (all five are already `Complete`), so no seeded fixture is
   re-dealt for Half B's sake.

### B3. `core::decision_site_walk`

Row `look_at_top_or_route` moves `AutoChosen` → `Served { by: "PB-DX35", residual: [...] }`.
Its `AutoChosen` reason is precisely what this batch deletes. **Careful**: the row is COMPOUND —
it also covers `Effect::RevealAndRoute`, whose CR 401.4 "in any order" choice is NOT served by
this batch. So either split the row or state the surviving `RevealAndRoute` residual explicitly
in the `Served.residual` list with its own seed. Decide, state the decision, and make the row's
`site` string true.

### B4. Half B probes (new file
`crates/engine/tests/primitives/pb_dx35_optional_placement.rs`, registered in the group `mod`)

Every probe asserts by RESOLUTION EFFECT, never by the offer.

* `t1` — `optional: false` (synthetic def) places the winner and asks NOTHING
  (`state.pending_effect_choice.is_none()` after a full resolution).
* `t2` — `optional: true` with candidates ASKS, and the DEFAULT answer places the same card
  `t1` places. This is the behaviour-preservation pin.
* `t3` — the DECLINE (`chosen: []`) leaves the card unplaced and routes it to `rest_to`. Assert
  the card's ZONE, not the absence of an event.
* `t4` — `optional: true` with an EMPTY candidate set asks nothing.
* `t5` — `candidates.first()` == today's `min_by_key(|id| id.0)` when `top_ids` is in a
  DIFFERENT order from ObjectId order. Build the fixture so top-first order and ascending-id
  order genuinely disagree, or this probe is vacuous — say so in its doc.
* `t6` — `birthing_ritual`'s two questions in one resolution, in order (PayOptionalCost then
  ChooseObject), driven on the real def.
* `t7` — the choice is a real CHOICE of WHICH, not only whether: with two matching cards,
  answering with the SECOND one places the second one.
* `t8` — `risen_reef` declined puts the land in HAND (its `rest_to`), which is the printed
  *"If you don't put the card onto the battlefield, put it into your hand."*

### B5. Half B reachability (AC 7328's UI-4 standard) — a NON-DEFAULT (decline) answer through
all three channels, each asserted by resolution effect:

* `LocalGame` / `HumanChoice` — new `crates/simulator/tests/pb_dx35_optional_placement_channel.rs`.
* `POST /api/game/action` — a real HTTP drive in `tools/play-server/src/main.rs`'s
  `#[cfg(test)]` module. If a play-server session cannot be steered onto one of the five defs,
  say EXACTLY which combination is untested and why (PB-DX45 had to disclose this).
* the bot path — assert `StubProvider` needs no change rather than assuming it.

### B6. Consumer enumeration (AC 7328)

Enumerate EVERY consumer of `EffectChoiceQuestion` / `EffectChoiceAnswer` and confirm each
already handles `ChooseObject` — engine `handle_answer_effect_choice`, `default_effect_choice_answer`,
`view-model`, `play-server` `view.rs` + `api.rs` `validate_decision_params`, the frontend picker,
`simulator` `legal_actions.rs` / `params.rs` / bots, TUI, replay viewer. Put the enumeration in the
execution notes with file:line. Confirm `validate_decision_params` dispatches on `question` alone
with no wildcard (PB-DX45's obligation 8) — it should already; the task is to VERIFY, not assume.

---

## HALF A — `OOS-DX4-2`: slice `ModeSelection.mode_targets` by the chosen mode on the TRIGGER path

Read execution-notes §0.3 (the CR 700.2b decision) and §0.5 (the dispatch map) before starting.
The four requirement sites and the two modes sites are enumerated there with file:line.

### A1. One shared arithmetic (`crates/engine/src/rules/abilities.rs`)

Add ONE function and make the three hand-rolled copies call it. Suggested shape:

```rust
/// CR 700.2b + CR 700.2c/700.2f + CR 601.2c — the mode(s) a modal triggered ability
/// is put on the stack with, and the target requirements those modes announce.
///
/// `None` = CR 700.2b's "If no mode is chosen, the ability is removed from the stack."
pub(crate) struct TriggerModalPlan {
    pub modes_chosen: Vec<usize>,
    pub requirements: Vec<TargetRequirement>,
}
pub(crate) fn trigger_modal_plan(state: &GameState, trigger: &PendingTrigger)
    -> Option<TriggerModalPlan>
```

Behaviour, in this order:

1. Look the ability up ONCE, kind-dispatched exactly as sites 1/2/3 already do
   (`Normal` → runtime `obj.characteristics.triggered_abilities[ability_index]` for `targets`;
   `CardDefETB` → registry `def.effective_abilities(obj.is_transformed)[ability_index]`).
   Any other kind → `Some(TriggerModalPlan { modes_chosen: vec![], requirements: vec![] })`,
   i.e. today's `vec![]`.
   Use `state.fizzle_object` (CR 113.7a LKI), matching site 3's existing choice, NOT
   `state.objects.get` — and say in the doc that sites 1 and 2 used the non-LKI lookup, so
   unifying them on the LKI one is a **deliberate** widening. Prove it changes nothing that
   matters, or say what it changes.
2. Look the `ModeSelection` up from the REGISTRY (both existing sites do; execution-notes §0.5
   says why that is the incumbent and what it costs). `None` ⇒ non-modal ⇒
   `modes_chosen: vec![]`, `requirements: <flat targets>` — today's behaviour exactly.
3. Modal with `mode_targets: None` ⇒ `modes_chosen: vec![0]` when `modes.len() > 0`
   (today's value, unchanged — with a FLAT list every mode announces the same requirements, so
   CR 700.2b legality cannot differ by mode and the existing CR 603.3d slot check already
   removes the trigger when it is unsatisfiable), `requirements: <flat targets>`.
   **This arm is what keeps every non-repaired corpus def byte-identical, and a probe must pin
   that.**
4. Modal with `mode_targets: Some(mt)` ⇒ CR 700.2b legality-aware choice:
   for `idx in 0..modes.modes.len()`, compute
   `casting::per_mode_target_requirements(ms, &[idx])` — **the SAME helper `handle_cast_spell`
   and `rules::queries::spell_target_requirements` call**, which is the shared arithmetic the
   criterion demands — and call that mode LEGAL iff every requirement it names yields a
   non-empty `trigger_target_candidates(..).candidates` OR that slot is `optional`. Take the
   FIRST legal index.
   * a legal mode exists ⇒ `modes_chosen: vec![idx]`, `requirements: <that mode's slice>`.
   * none legal and `min_modes == 0` ⇒ `modes_chosen: vec![]`, `requirements: vec![]`
     ("choose up to one" and chose zero — CR 700.2b permits it; the ability resolves with no
     effect).
   * none legal and `min_modes >= 1` ⇒ **`None`** (CR 700.2b removal).
5. `max_modes > 1` combined with `mode_targets: Some(_)` is UNSUPPORTED, exactly as
   `abilities.rs:455` already hard-rejects it on the activated path. Zero corpus members
   (measured: all 7 modal triggered abilities have `max_modes: 1`). Handle it fail-safe
   (choose one mode) with a `debug_assert!`, and **gate the zero population by roster** so it
   cannot grow silently.

### A2. The consumers — all four, and the modes assignment must use the SAME value

* Site 1 (`trigger_target_requirements`, `abilities.rs:~8806`) → `plan.requirements`.
* Site 2 (`ability_targets`, `abilities.rs:~8929`) → `plan.requirements`; a `None` plan is a
  CR 700.2b removal and must take the SAME path the CR 603.3d "no legal choice" branch takes.
* Site 3 (`trigger_ability_target_requirements`, `abilities.rs:~10352`) → `plan.requirements`.
  It re-derives on the ANSWER path; state why re-derivation is stable (the admission gate
  admits only the answer command and `Concede` while a trigger-target choice is pending —
  VERIFY that in source, do not assume it).
* Site D — the `modes_chosen` assignment at `abilities.rs:~9855-9887`. **Delete the
  registry re-lookup and the hard-coded `vec![0]` in both arms** and assign
  `plan.modes_chosen`. If the plan is recomputed rather than threaded, the two must be the
  same call; prefer computing the plan ONCE near the top of the per-trigger loop and threading
  it, so "one arithmetic" is structural rather than coincidental.
* `rules/resolution.rs:~2351-2390` — leave the registry modes lookup, but note two things in
  source: (a) it is the second half of the pair §0.5 measures as misaligned for three defs, and
  (b) with `max_modes: 1` corpus-wide the chosen mode's slice sits at offset 0 of
  `stack_obj.targets`, so `EffectTarget::DeclaredTarget { index: N }` inside a mode must be
  **rebased to 0** in any def this batch re-shapes. **Verify by execution** that no per-mode
  offset loop is needed on the trigger path at `max_modes: 1`; if one IS needed, mirror
  `resolution.rs`'s spell-side offset loop rather than writing a third copy.
  **Also fix**: a modal trigger with an EMPTY `modes_chosen` currently falls through to the
  runtime `effect`, which for the three `WhenDies`/`WhenAttacks`/`WhenBlocks` lowering arms is
  **mode 0 pre-resolved** — so "chose zero modes" silently executes mode 0. Make a modal
  ability with no chosen mode resolve with NO effect, and pin it.

### A3. Card defs

* `shambling_ghast.rs` — flat `targets: vec![]`; `mode_targets: Some(vec![vec![], vec![<the
  opponent-creature requirement>]])` (mode 0 = Treasure, mode 1 = -1/-1; the def's mode order is
  deliberately reversed from print and its own comment says so — do not reorder). Mode 1's
  `EffectFilter::DeclaredTarget { index: 0 }` already reads slot 0 and stays 0.
  **Marker `partial` → `Complete`**, note rewritten.
* `retreat_to_kazandu.rs` — flat `targets: vec![]`; `mode_targets: Some(vec![vec![TargetCreature],
  vec![]])`. Mode 0's `DeclaredTarget { index: 0 }` stays 0. Stays `Complete` (0 flip).
* `retreat_to_coralhelm.rs` — same shape (mode 0 `[TargetCreature]`, mode 1 `[]`). Stays
  `known_wrong` for its unrelated tap/untap blocker; update the note to say the mode-target half
  is now correct.
* `hullbreaker_horror.rs`, `glissa_sunslayer.rs`, `junji_the_midnight_sky.rs` — **DO NOT
  re-shape.** Re-adjudicate the marker text to name the index-space blocker (execution-notes
  §0.5) and cite the new seed. Re-shaping them would arm `OOS-DX4-2`'s own stated trap.
* `felidar_retreat.rs` — not in the population (flat `targets` is empty); leave it, and say so.

### A4. Half A probes (new `crates/engine/tests/primitives/pb_dx35_modal_trigger_targets.rs`)

* `t1` — **the CR 603.3d trap, the criterion's headline**: `retreat_to_kazandu` on a board with
  NO creature. The trigger must RESOLVE and gain 2 life. Assert the life total, not the event.
  At the merge base this trigger is removed and life is unchanged.
* `t2` — same def, WITH a legal creature: mode 0 is chosen, a target IS announced and the
  +1/+1 counter lands on that creature. **This is the "drop-the-requirement trap is red" probe**
  — a revert that makes the plan return `vec![]` for `mode_targets` defs must redden it.
* `t3` — `shambling_ghast` dies with no opponent creature: a Treasure token exists afterwards.
* `t4` — `shambling_ghast` dies WITH an opponent creature: still a Treasure (mode 0 is legal, so
  CR 700.2b's first-legal choice picks it) and NO target is announced, because mode 0's slice is
  empty. Pin the announcement count.
* `t5` — a synthetic modal trigger whose mode 0 needs a target and mode 1 does not, with
  `min_modes: 1`: with no candidate for mode 0 the plan picks mode 1. Pin `modes_chosen`.
* `t6` — the same synthetic with `min_modes: 0` and NO legal mode: `modes_chosen` empty and the
  ability resolves with **no effect** (the A2 fix).
* `t7` — a synthetic with `min_modes: 1` and NO legal mode: CR 700.2b removal (the trigger is
  not on the stack).
* `t8` — `mode_targets: None` modal trigger is byte-identical to the merge base
  (`modes_chosen == vec![0]`, flat requirements). The backward-compat pin.
* `t9` — sites 1/2/3 agree BY VALUE on the same trigger (the differential pin that stops the
  three copies re-diverging).

### A5. Half A roster gates (new `crates/engine/tests/core/pb_dx35_modal_trigger_roster.rs`)

All derived from `mtg_card_defs::all_cards()` (SR-36), all PRINTING their populations
(`t_census_report`), never transcribing them:

* `r1` — the 7 modal triggered abilities, by name, with their `mode_targets` state.
* `r2` — **the index-space alignment census** (execution-notes §0.5): for every modal triggered
  ability, registry index vs the lowered runtime index, and the misaligned set pinned at exactly
  `{hullbreaker_horror, glissa_sunslayer, junji_the_midnight_sky}` — so a new member cannot join
  silently and the day someone lowers `modes` the gate says the set emptied. Derive the runtime
  index by CALLING the lowering, not by counting `AbilityDefinition::Triggered` entries.
* `r3` — every misaligned member is non-`Complete` (the zero-deck-legal-blast-radius claim,
  gated rather than asserted in prose).
* `r4` — `max_modes: 1` for every modal triggered ability (A1 step 5's premise).
* `r5` — no def combines a NONEMPTY flat `targets` with `mode_targets: Some(_)` on a Triggered
  ability (the author invariant the cast path already enforces at `casting.rs:3848`).
* `r6` — the defect population (nonempty flat `targets` + at least one mode that names no
  declared target) is exactly the members this batch repaired plus the three it filed.
* `r7` — the `modal_trigger` `decision_site_walk` row's `site` string does not still claim
  `modes_chosen = vec![0]` in both arms.

### A6. Half A channel probe

`crates/simulator/tests/pb_dx35_modal_trigger_channel.rs` — drive `retreat_to_kazandu`'s landfall
trigger end-to-end through `LocalGame` with a real land drop, on a board with no creature, and
assert the life total. The engine-level probe is not the channel probe; PB-DX43's lesson
(`kaito_shizuki`: existence is never sufficiency) applies.


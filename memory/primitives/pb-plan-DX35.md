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

## HALF A — see §A below (filled in after the stage-0 dispatch map lands)

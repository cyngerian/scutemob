# PB-DX35 — execution notes

**Task**: `scutemob-227`. v4 queue rank 12 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 12).
**Seeds**: `OOS-DX4-2` (Half A) + `OOS-DX4-5` (Half B); siblings `OOS-DP10-5`, `OOS-DX8-3`, PB-DX9's row.

---

## §0 — Stage 0: measured BEFORE any production line changed

### §0.1 Baseline

Full-workspace `cargo test --workspace --no-fail-fast` on this branch, tree clean, **before any edit**:

```
5,058 passed / 0 failed / 5 ignored — 61 result-producing targets
```

This **reproduces PB-DX51's published close pin exactly** (5,058 / 0 / 5, 61 targets). Log:
`scratchpad/pbdx35/baseline.log`.

Current fingerprints at the merge base: **HASH 82**, **PROTOCOL 41**.

### §0.2 The wire prediction — written before any production line changed

Both halves are predicted **NONE / NONE**, and the reasons are structural rather than hopeful.
The stop condition is stated with each: if the named premise turns out false, the batch takes ONE
bump per fingerprint for the whole PB (never two) and re-derives the number from the failing
gate's own output rather than inventing it.

#### Half A (`OOS-DX4-2`, modal trigger targets) — predicted **PROTOCOL none / HASH none**

The memo's cell reads *"none-if-registry / both-if-lowered (MED)"*. The reachability trace that
decides it:

* `ModeSelection` (and therefore `mode_targets`) hangs off
  `AbilityDefinition::Triggered`, i.e. off `CardDefinition`. `CardDefinition` is
  **excluded** from the PROTOCOL closure (`CLOSURE_MUST_NOT_CONTAIN`), and
  `GameState::card_registry` is `#[serde(skip)]` for the declaration digest — the same two
  facts PB-DX18 relied on when `AbilityDefinition::Splice` gained a field and PROTOCOL did
  not move.
* The runtime type `Characteristics::triggered_abilities: Vec<TriggeredAbilityDef>` **has no
  `modes` field at all** (`crates/card-types/src/state/game_object.rs:908-966`, read field by
  field). So the incumbent source of a modal trigger's `ModeSelection` is *already* the
  registry: both `rules/abilities.rs` (~:9845) and `rules/resolution.rs` (~:2350) read
  `state.card_registry` → `def.effective_abilities(..)` for it today.
* Therefore **reading `mode_targets` from the registry adds no type, no variant and no field**,
  and the prediction is NONE for both fingerprints.
* **The counterfactual is stated because it is the live alternative**: lowering
  `modes`/`mode_targets` into `TriggeredAbilityDef` would add a field to a type reachable from
  `Characteristics`, which is a PROTOCOL closure root — that is the "both" branch, and it is
  the branch this batch does **not** take.

**Stop condition**: if `protocol_schema` or `hash_schema` reddens, the registry-read premise is
false somewhere and the design is re-examined before any number is edited.

#### Half B (`OOS-DX4-5`, the inert `optional` flag) — predicted **PROTOCOL none / HASH none**

The question variant is the decision, and it is settled by CR text rather than by convenience.
Three candidates were considered:

1. **A new `EffectChoiceQuestion::Confirm`-shaped variant** — a costless yes/no.
   *Rejected.* It is a new variant on a type inside the wire closure, so it costs BOTH bumps,
   and it answers a *narrower* question than the cards ask.
2. **A zero-cost `PayOptionalCost`** — `Cost::Mana(default)`.
   *Rejected as dishonest.* `can_pay_optional_cost` is trivially true for a free cost, the
   client would render "Pay {0}?", and the task brief names this exact trap (DP-12). A question
   whose payload lies to the renderer is worse than no question.
3. **`EffectChoiceQuestion::ChooseObject { candidates, count: 1, up_to: true }`** — CHOSEN.
   This is what the five printed cards literally say. All five read *"You may put **a**
   [creature/land] card from among them into your hand / onto the battlefield"* (MCP,
   verbatim, §0.3): that is CR 608.2's *choose up to one*, which is the exact contract
   `ChooseObject` was built for by PB-DX28 (CR 115.10 untargeted resolution-time choice).
   It carries BOTH halves of the printed decision — **whether** and **which** — where a bare
   confirm carries only the first, and the v4 memo's own row 15 cell already records the
   judgement: *"**DX4-5 now none** (HIGH) — PB-DX28's `EffectChoiceQuestion::ChooseObject
   { up_to: true }` already expresses it"*.

   No new variant, no new field ⇒ **NONE for both fingerprints**.

**The one thing reuse costs, stated rather than discovered later**: `ChooseObject`'s own doc
comment claims *"this one names **public** information: every id is a permanent on the
battlefield or a card in a graveyard"*. `LookAtTopThenPlace` hands it **library** ids, so that
sentence becomes false the moment this batch ships. Redaction is unaffected —
`GameEvent::EffectChoiceRequired` is `private_to(player)`, the same channel `SearchLibrary`,
`Scry` and `Surveil` already carry hidden library ids on — but **a false comment is this
queue's most-repeated defect**, so the doc is corrected in the same commit as the code rather
than left for a later batch to trip over.

**Stop condition**: if either gate reddens, ONE bump per fingerprint is taken for the whole PB
(both halves together), read off the failing gate's own output.

#### Consequence if the prediction holds

No sentinel re-pin and no `*_SCHEMA_HISTORY` row are **owed**. `history_is_append_only` and
`frozen_prefix_is_pinned` are still executed and must be green; that they were not edited is the
claim, and the gates are the evidence.

### §0.3 The mode-choice channel decision for triggers — STATED, with its consequences

`AC 7327` requires this decision to be made and stated. **This batch does NOT give a modal
triggered ability's mode choice a human channel; the `decision_site_walk` `modal_trigger` row
stays `AutoChosen`.** What changes is that the engine's automatic choice becomes
**CR 700.2b-legal**, which it is not today.

**CR 700.2b, verbatim** (MCP):

> The controller of a modal triggered ability chooses the mode(s) as part of putting that
> ability on the stack. **If one of the modes would be illegal (due to an inability to choose
> legal targets, for example), that mode can't be chosen.** If no mode is chosen, the ability
> is removed from the stack. (See rule 603.3c.)

Today `flush_sorted` writes `stack_obj.modes_chosen = vec![0]` in **both** arms of its
`min_modes` branch, unconditionally — so the engine picks mode 0 even when mode 0 is exactly the
mode CR 700.2b forbids it to pick. **"Scope the targets to mode 0" — the option the brief names
— is therefore not available**: it would leave `retreat_to_kazandu` unable to gain 2 life with no
creature on the board, which is the defect, and it would make the two predicted `partial →
Complete` flips false claims. Legality-aware auto-choice is not scope creep here; it is the
minimum CR-correct behaviour, and it is what makes AC 7327's mandated probe (*a mode chosen
WITHOUT a target resolves when the other mode's target is absent*) pass.

**Gate consequences taken**:

* `core::decision_site_walk`'s `modal_trigger` row keeps class `AutoChosen` — the *controller*
  still does not choose — but its `site` string (*"modes_chosen = vec![0] in both the
  min_modes==0 and !=0 arms"*) becomes false and is rewritten.
* No new `EffectChoiceQuestion` and no new `Command` ⇒ the `decision_gate` ratchet and the
  CR 605.4a mana-ability needle list (`test_dp9_mana_ability_gate`) are **not** owed a change by
  Half A. (Half B's own needle situation is §0.4.)
* The human channel is the residual and is **filed**, not silently dropped.

### §0.4 Half B's gate consequences

`LookAtTopThenPlace` is **already** in `test_dp9_mana_ability_gate`'s needle list — PB-DX45 added
it as a deliberately OVER-wide needle for the `place_cost` field. After this batch the needle
stops being over-wide for the five `optional: true` defs (they now genuinely ask) and stays
over-wide only for a hypothetical `optional: false, place_cost: None` def, of which the corpus has
**zero**. **The needle does not change; its justifying comment does**, and that comment is
rewritten rather than left asserting a population that has moved.

`core::decision_site_walk`'s `look_at_top_or_route` row moves `AutoChosen` → `Served`, because its
stated reason (*"LookAtTopThenPlace's `optional` field is inert by construction (OOS-DP10-5)"*)
is exactly what this batch deletes.

### §0.5 The Half A dispatch map — measured at stage 0, and it moves the batch

Everything here was read out of the tree before any production line changed. It refutes two
published cells and finds a live defect no document names.

#### The four target-requirement derivation sites (the memo names none of them)

| # | site | consumed for |
|---|---|---|
| 1 | `rules/abilities.rs:8806-8841` (`trigger_target_requirements`, inside `flush_sorted`) | `has_ability_targets` + `StackObject.target_requirements` |
| 2 | `rules/abilities.rs:8929-8971` (`ability_targets`) | the CR 603.3d slot derivation and the "remove this trigger" decision |
| 3 | `rules/abilities.rs:10352-10388` (`fn trigger_ability_target_requirements`) | the CR 601.2c cross-slot distinctness check on the answer path |
| 4 | `rules/mana.rs:900-921` | decides whether a `WhenTappedForMana` ability is targeted (and so must use the stack); queues `CardDefETB` **precisely so the registry index space matches** |

Sites 1-3 are three hand-rolled copies of one lookup — this is the "ONE shared arithmetic"
AC 7327 asks for, and it is three, not one. Site 4 is a different question on a different kind
and is deliberately **not** unified (it decides *whether targeted*, not *which targets*).

#### The two `ModeSelection` read sites, and the index space they use

`rules/abilities.rs:9855-9887` and `rules/resolution.rs:2351-2390`. **Both read the REGISTRY**
(`state.card_registry` → `def.effective_abilities(..).get(ability_index)`), because
`TriggeredAbilityDef` has **no `modes` field at all**
(`crates/card-types/src/state/game_object.rs:908-966`, read field by field).

#### THE FINDING: three of the seven modal triggered abilities look their modes up in the WRONG INDEX SPACE, and no document names it

All three of the seed's named cards queue as `PendingTriggerKind::Normal`, so
`PendingTrigger.ability_index` indexes the **runtime** `Characteristics::triggered_abilities`
vec. The modes lookup indexes the **registry** `CardDefinition::abilities` list. Those two agree
only when no non-`Triggered` ability precedes the modal one.

Census over all 7 corpus modal triggered abilities:

| def | registry idx | runtime idx | aligned? | consequence today |
|---|---|---|---|---|
| `felidar_retreat` | 0 | 0 | YES | — |
| `retreat_to_coralhelm` | 0 | 0 | YES | — |
| `retreat_to_kazandu` | 0 | 0 | YES | — |
| `shambling_ghast` | 0 | 0 | YES | — |
| `hullbreaker_horror` | **1** (Keyword(Flash) first) | 0 | **NO** | registry `.get(0)` is a `Keyword`, so `modes = None`, `modes_chosen` stays empty and the trigger resolves the runtime `effect` — which is `Effect::Nothing`. **The whole modal ability is a no-op.** |
| `glissa_sunslayer` | **2** (FirstStrike, Deathtouch first) | 0 | **NO** | same — `effect: Effect::Nothing`, whole ability a no-op |
| `junji_the_midnight_sky` | **2** (Flying, Menace first) | 0 | **NO** | same lookup failure, DIFFERENT symptom: `WhenDies` is one of the three lowering arms that pre-resolve `modes.first()` into `effect`, so junji silently executes **mode 0 forever** and the mode choice is a fiction |

**Blast radius is ZERO deck-legal cards**: all three misaligned defs are non-`Complete`
(`hullbreaker_horror` `partial`, `glissa_sunslayer` `partial`, `junji_the_midnight_sky`
`known_wrong`), so `validate_deck` refuses every one of them. That is why this is FILED rather
than fixed here, and the reason is stated as a measurement rather than as a scope preference.

**Why it is not fixed in this batch.** The only structural fix is to lower `modes` into
`TriggeredAbilityDef`, which is the memo's own "both-if-lowered" branch. Measured cost:
`TriggeredAbilityDef` has no `Default` derive and **190 struct literals across 44 files**
construct it exhaustively, so a field addition is 190 mechanical edits **plus** one PROTOCOL and
one HASH bump plus the full sentinel/history ceremony — on top of two halves that already carry
a census, a channel and a reachability matrix. It also changes behaviour simultaneously for
three defs whose modal dispatch has never once run. That is its own batch by this queue's own
sizing, and it is filed with the census above so the next dispatcher inherits the measurement
rather than the surprise.

#### Consequence 1 — the memo's 2-flip cell is REFUTED, and the seed row's own trap is why

v4 §4 row 12 and the task brief both predict *"2 real flips (`shambling_ghast`,
`hullbreaker_horror`)"*. **The measured answer is ONE.**

`OOS-DX4-2`'s row warns, verbatim, that *"moving the targets into `mode_targets` looks like the
CR 601.2c-correct repair and would silently DROP the requirement instead of scoping it, because
nothing reads the field."* For `hullbreaker_horror` that trap is **still armed after this
batch** — not because the trigger path now ignores `mode_targets` (it does not), but because the
modes lookup cannot find its `ModeSelection` at all, so the slice falls back to the flat list,
which the repair would have emptied. Repairing that def here would convert a trigger that is
usually *removed* into one that always *resolves doing nothing*. **So `hullbreaker_horror` is
re-adjudicated and NOT re-shaped**: it keeps `partial`, and its marker is rewritten to name the
blocker that actually survives.

The batch's own thesis, applied to the batch: a repair that looks right is measured before it is
made.

#### Consequence 2 — the wire prediction §0.2 STANDS, unrevised

Half A reads `mode_targets` off the registry, exactly as the incumbent modes lookup already
does. No type, no variant, no field. **PROTOCOL none / HASH none** for both halves, as committed
in `c6646052` before any production line. The lowering counterfactual is named above and is the
branch not taken.

#### Predicted flips, NAMED before regeneration

* `shambling_ghast` **`partial` → `Complete`** — its marker names exactly this defect and
  nothing else survives it. Its residual (the controller cannot *choose* mode 1) is the
  corpus-wide `modal_trigger` AutoChosen row, which does not demote defs — `felidar_retreat` and
  `retreat_to_kazandu` are `Complete` carrying it.
* `retreat_to_kazandu` — repaired in place, **stays `Complete`, 0 flip**. It is the
  live-wrong deck-legal member: printed *"choose one — • Put a +1/+1 counter on target creature.
  • You gain 2 life."*, authored with a FLAT `TargetCreature`, so with an empty board CR 603.3d
  removes the trigger and the controller cannot take the mode that needs no target at all.
* `retreat_to_coralhelm` — repaired in place, **stays `known_wrong`, 0 flip** (its blocker is
  the unrelated "tap or untap modelled as untap only").
* `hullbreaker_horror`, `glissa_sunslayer`, `junji_the_midnight_sky` — **NOT re-shaped**, markers
  re-adjudicated to name the index-space blocker, seed filed.

Net: **1 flip, named**. Coverage predicted **1,137 → 1,138 / 1,803 = 63.1% → 63.1%**.

## §B — Half B: post-code record (append-only; §0 above is the pre-code prediction and stays
untouched)

Baseline going into Half B (measured on this worktree after Half A committed, `dfd6e1ce`):
**5,076 passed / 0 failed / 5 ignored, 62 result-producing targets.**

### B1. The engine change

`crates/engine/src/effects/mod.rs`, the `LookAtTopThenPlace` arm (~line 6595 pre-edit): `optional`
is now bound (not `optional: _`), the candidate-finding block is unchanged in its `filter`/cap
logic but now **collects into a `Vec<ObjectId>` and explicitly `sort_by_key(|id| id.0)`s it**
(`top_ids` is `Zone::top_n` order, i.e. top-first — the reverse of ascending-id order for any
2+-candidate window built the ordinary way, so the sort is load-bearing, not defensive). The
winner is then decided by:

* `!optional || candidates.is_empty()` → `candidates.first().copied()` — byte-identical to the
  pre-batch `min_by_key(|id| id.0)` winner (proven equal by construction: same filter/caps, same
  set, `.first()` of an ascending sort == `min_by_key`).
* otherwise → `ask_or_consume_effect_choice(state, ctx, p, EffectChoiceQuestion::ChooseObject {
  candidates: candidates.clone(), count: 1, up_to: true })`, addressed to `p` (the looking
  player), not `ctx.controller` — matches the `place_cost` ask three lines up. `Some(ChooseObject
  { chosen })` → `chosen.first().copied()`; `Some(other)` → `debug_assert!` + fall back to
  `candidates.first()`; `None` (suspended) → `break`, not `continue`, mirroring `place_cost`'s own
  suspend arm and for the identical reason.
* **No determined-answer short-circuit for `candidates.len() == 1`**, per the plan — a code
  comment states why (`up_to: true` makes declining a real second answer even for one candidate).

`crates/card-types/src/cards/card_definition.rs`'s `LookAtTopThenPlace.optional` field doc
(the M7/PB-OS8 "reserved for M10+ interactive decline" sentence) was also corrected — it is a
card-author-facing claim, not just an internal comment, and the plan's "a comment is a claim" rule
does not stop at `effects/mod.rs`'s file boundary.

### B2. Comment corrections — SIX sites, not five

The plan named five sites (the arm's own comment, `stubs.rs`'s `ChooseObject` doc, `view.rs`'s
`ChooseObject` arm, `test_dp9_mana_ability_gate`'s needle justification, and the five card defs).
**Two more were found during B6's consumer enumeration and are the SAME false-comment class**:

1. `effects/mod.rs`'s `optional: _` comment — rewritten (see B1).
2. `crates/card-types/src/state/stubs.rs`'s `EffectChoiceQuestion::ChooseObject` doc — rewritten to
   state both populations (public battlefield/graveyard objects since PB-DX28, library ids since
   PB-DX35) and why redaction is unaffected either way (`GameEvent::EffectChoiceRequired` is
   `private_to(player)`, the same channel `SearchLibrary`/`Scry`/`Surveil` already carry hidden
   library ids on).
3. `tools/play-server/src/view.rs`'s `ChooseObject` arm comment — rewritten; **verified, not
   assumed, that no code change was owed**: it already renders through `question_cards`, the same
   channel `SearchLibrary` uses for library cards.
4. `crates/engine/tests/primitives/pb_dp9_effect_choice.rs::test_dp9_mana_ability_gate` — the
   `LookAtTopThenPlace` needle's justification rewritten (needle unchanged; it is over-wide only
   for a hypothetical `optional: false, place_cost: None` def, of which the corpus has zero, now
   stated instead of the pre-batch "five carry it, one sets place_cost" framing).
5. The five card defs (`birthing_ritual`, `growing_rites_of_itlimoc`, `grisly_salvage`,
   `satyr_wayfinder`, `risen_reef`) — all comment-only, verified by `git diff` line-by-line (every
   `+`/`-` line starts with `//`), no `Completeness` marker moved.
6. **NOT in the plan, found by B6**: `crates/engine/src/testing/replay_harness.rs`'s
   `auto_answer_blocking_decisions` `ChooseObject` arm ("Unlike the four arms above, `candidates`
   names PUBLIC objects... no hidden-zone caveat applies") and
   `tools/tui/src/play/app.rs`'s `EffectChoiceRequired` formatter's `ChooseObject` arm (identical
   claim). Both rewritten with the same correction as #2/#3; both conclusions (the harness has
   omniscient access so nothing leaks; the TUI formatter prints the class label only regardless)
   were already correct and stay unchanged — only the STATED REASON was false.

### B3. `core::decision_site_walk` — SPLIT, not residual-stated

Chose **split** over "state the surviving residual in `Served.residual`": the row's predicate
(`json_contains_variant(v, "LookAtTopThenPlace") || json_contains_variant(v, "RevealAndRoute")`)
is an OR over two DSL variants with independently-moving decision status after this batch, and
`decision_gate.rs`'s `every_baseline_entry_is_live_and_necessary` (T5) computes `auto_chosen_row_
hits` PER ROW — leaving the row compound and reclassifying it `Served` would have made the
`RevealAndRoute` members (`Chaos Warp`, `Coiling Oracle`, `Goblin Ringleader`, `Sylvan Messenger`)
silently stop needing a `BASELINE` entry even though their CR 401.4 "any order" choice is still
engine-made — the exact kind of false "served" claim this whole queue exists to catch.

Shipped: `p_look_at_top_or_route` renamed in spirit (kept its id, narrowed to
`json_contains_variant(v, "RevealAndRoute")` only) and stays `AutoChosen`, site string corrected.
New row `look_at_top_then_place_optional` (predicate `p_look_at_top_then_place`, only
`LookAtTopThenPlace`), class `Served { by: "PB-DX35", residual: &[] }` — empty residual because the
served `ChooseObject` question carries BOTH halves of CR 118.12's decision (whether and which), so
nothing is left over for this specific row. The surviving `RevealAndRoute` gap is filed as
**`OOS-DX35-1`** (registry row appended to `docs/audits/decision-point-audit.md`, after
`OOS-DX51-7`) rather than merely named in a code comment, per dispatch hygiene 5.

Consequences, all re-observed by executing the gate rather than computed:
* `BASELINE` (`decision_gate.rs`): the five LookAtTopThenPlace entries removed (`Birthing Ritual`,
  `Growing Rites of Itlimoc`, `Grisly Salvage`, `Satyr Wayfinder`, `Risen Reef`) — none of the five
  hits any OTHER `AutoChosen` row, so they leave the `BASELINE` table entirely, not just this row.
  The four `RevealAndRoute` entries (`Chaos Warp`, `Coiling Oracle`, `Goblin Ringleader`,
  `Sylvan Messenger`) are UNCHANGED.
* `MAX_AUTO_CHOSEN_COMPLETE_UNION`: **72 → 67**, read off `T6`'s own printed number (matches the
  arithmetic `72 − 5`, which is a check here, not the source, per the constant's own standing
  rule).
* `positive_value_for_row`'s `"look_at_top_or_route"` arm now constructs `Effect::RevealAndRoute`
  (was `Effect::LookAtTopThenPlace`); a new `"look_at_top_then_place_optional"` arm constructs
  `Effect::LookAtTopThenPlace { optional: true, .. }`.
* `served_rows_still_have_their_hooks` (T8): `("look_at_top_then_place_optional", 1)` added to its
  explicit per-id floor list (the pattern PB-DX45's `may_pay_then_effect` established but never
  joined that specific list — this batch's row DOES join it, since a hook floor costs nothing and
  proves the row is not dark).
* `crates/simulator/src/decision_coverage.rs`: `look_at_top_then_place_optional` joins
  `OBSERVABLE_ROW_IDS` (now 7); `look_at_top_or_route`'s `UNOBSERVABLE_ROW_IDS` reason string
  narrowed to RevealAndRoute. Module doc counts corrected (22→23 rows named in ROWS' own doc
  header — a PRE-EXISTING stale count found while touching this file: the live `ROWS.len()` was
  already 22 before this batch touched anything relevant, one short of what two doc paragraphs
  claimed elsewhere; not chased further, out of scope, but the two numbers this batch's own edit
  touches — "Five/Seven rows observable", "Seventeen/Sixteen rows unobservable" — are corrected).
* **`row_id_for` needed a real code change, not just a doc fix** — `EffectChoiceQuestion::
  ChooseObject` is now asked by TWO unrelated primitives (PB-DX28's untargeted resolution-time
  choice, which the pre-existing code correctly maps to `None` because it sits outside the audit's
  ROWS taxonomy entirely, and PB-DX35's placement, which IS one of the ROWS). The two cannot be
  told apart by the question's own shape (identical `{candidates, count, up_to}`). Disambiguated by
  ZONE: `LookAtTopThenPlace`'s candidates resolve entirely to `ZoneId::Library(_)` (a hidden zone);
  PB-DX28's resolve to Battlefield/Graveyard (public). A nonempty, all-Library candidate set maps
  to `"look_at_top_then_place_optional"`; anything else (including empty, which every determined
  short-circuit produces before either site asks) stays `None`. Proven reachable by a NEW fixture,
  `look_at_top_then_place_state()`, in `crates/simulator/tests/pb_dx32_fuzz_output.rs`'s
  `test_dx32_row_id_for_covers_every_observable_row` (T6.2) — a `PendingEffectChoice` whose sole
  candidate is a REAL object minted first in a fresh two-player build (`ObjectId(1)`, deterministic
  from `GameState::next_object_id`'s `timestamp_counter` starting at 0), living in
  `ZoneId::Library(p1)`.
* **`test_dx32_a_fuzz_run_reaches_at_least_one_served_row` (T6.3) moved, and was RE-OBSERVED, not
  silently re-tuned** (this test's own module doc calls exactly that out as the required
  discipline): the reached set gained `look_at_top_then_place_optional`
  (`may_pay_then_effect` stays never-reached, unmoved). Re-run twice, identical partition both
  times. No A/B was owed (unlike PB-DX18's entry in this same test, which attributed a MOVEMENT in
  an EXISTING row) — this is a SEVENTH row entering the set because the row is NEW, with the six
  existing rows' own reachability provably untouched (identical `DecisionCoverage` entries either
  side of the new one).

### B4. Half B probes — `crates/engine/tests/primitives/pb_dx35_optional_placement.rs`

All 9 tests (t1–t8 + a version-sentinel) pass. One-line summary + revert row for each:

| test | asserts | reddens under |
|---|---|---|
| `t1` | `optional: false` places the winner, raises no question | CONTROL — green under every revert (pins the untouched arm) |
| `t2` | `optional: true`, default answer reproduces `t1`'s winner | CONTROL — green under a full revert too (the equivalence IS the property; `t3`/`t6`/`t7`/`t8` are what prove the ask itself is real) |
| `t3` | decline (`chosen: []`) leaves the card unplaced, routes it to `rest_to` (asserted by ZONE) | full revert to the pre-batch inert design (R1) |
| `t4` | `optional: true`, empty candidate set, no question raised | CONTROL — green under every revert (pre-existing `continue`/empty-set path) |
| `t5` | `candidates.first()` (post-sort) == the pre-batch `min_by_key` winner, on a fixture where `Zone::top_n`'s top-first order and ascending-id order are proven (by an explicit pre-effect assertion) to disagree | R2 — deleting ONLY the `candidates.sort_by_key` call (not a full revert; `t5` stays green under R1 because `min_by_key` was already correct pre-batch) |
| `t6` | Birthing Ritual (the corpus's only two-question member) asks `PayOptionalCost` THEN `ChooseObject`, in that order — driven on the REAL def's effect extracted from `all_cards()` | R1 (order becomes `["PayOptionalCost"]` only — the `ChooseObject` half never asked) |
| `t7` | answering with the SECOND (higher-id) candidate places the second, not the deterministic default | R1 and R2 (both reproduce a wrong winner) |
| `t8` | Risen Reef declined puts the card into HAND (its printed `rest_to`), driven through the full `process_command` cast→resolve→trigger→resolve pipeline, not a direct-executor shortcut | R1 (`pending_effect_choice()` is never `Some`, the `.expect(..)` panics) |

R1 = full revert of the B1 diff (`git checkout -- crates/engine/src/effects/mod.rs` against the
pre-Half-B tree). R2 = R1's tree with only `candidates.sort_by_key(|id| id.0);` deleted (compiles
after also dropping the now-unnecessary `mut` on `candidates`'s binding). Both executed; both
restored from a full-file backup before continuing; the full suite (`t1`–`t8` + sentinel)
re-verified green after each restore.

### B5. Reachability — three channels, each with a NON-DEFAULT (decline) answer, asserted by
resolution effect

* **`LocalGame`/`HumanChoice`** — `crates/simulator/tests/pb_dx35_optional_placement_channel.rs`
  (new file, 3 tests: `c1` decline, `c2` accept, `c3` bot path). Subject: Risen Reef, cast with
  `auto_tap: true` through `LegalAction::CastSpell`, driven to its own ETB's `ChooseObject` offer.
  `c1` asserts the fixture Swamp ends in `ZoneId::Hand(p1)` on decline; `c2` asserts battlefield
  TAPPED on accept; `c3` asserts `StubProvider.legal_actions` offers exactly one
  `AnswerEffectChoice` for the outstanding question with **no code change needed** (verified by
  reading `legal_actions.rs`'s `EffectChoice` arm: it is fully generic over `EffectChoiceQuestion`,
  builds the offer from `default_effect_choice_answer(&question)` with no per-variant match at
  all). All three reddened under a full R1 revert (the offer is never reached; drive loop exhausts
  its guard against `Halted(MaxTurns)`), then were restored green.
* **`POST /api/game/action`** — `tools/play-server/src/main.rs`'s `#[cfg(test)]` module, 3 new
  `#[tokio::test(flavor = "multi_thread")]` fns (`test_dx35_the_choose_object_offer_over_http`,
  `..._a_declined_..._over_http`, `..._an_accepted_..._over_http`), modelled on the `dx45h_*`
  precedent. Subject: **Satyr Wayfinder**, not Risen Reef — its printed dig is `count: Fixed(4)`,
  and the fixture's 99-card deck is dominated by Forests, so ALL FOUR top-of-library cards are
  legal Land candidates (a genuine "which of several" choice, not merely "whether one"). Fixed
  deck (1 Satyr Wayfinder + 98 Forest), commander `old-gnawbone` (`{5}{G}{G}`, the `DX45H_
  COMMANDER` unreachable-in-window trick, reused verbatim). **Seed 1 found by an executed scan
  over seeds 0..200** (a throwaway `#[test]` fn written, run, and DELETED once seed 1's hand —
  `[Satyr Wayfinder, Forest×6]` — was confirmed; not guessed, not carried over from `DX45H_SEED`,
  since a different deck composition shuffles differently even at the same seed). `api::validate_
  decision_params`'s `ChooseObject` arm and `view::blocking_decision_view`'s `ChooseObject` arm
  needed **ZERO code changes** — both already handle the variant generically since PB-DX28,
  verified by reading both (`api.rs`'s arm's own count/up_to check is unconditional on the
  question's population; `view.rs`'s arm renders through `question_cards` unconditionally) and by
  executing all three tests. **Zone assertions use COUNT DELTAS plus a per-id retirement check,
  not "the same id moved"**: CR 400.7 mints a new object id on every cross-zone move (library →
  hand/graveyard is cross-zone, unlike PB-DX15a's same-zone case), so the ORIGINAL candidate ids
  are provably gone (`dx35h_object_retired`) and the destination zone's population count moves by
  the expected amount (`dx35h_zone_count` before/after). All three reddened under a full R1 revert
  (the offer is never reached within the drive's step budget), then were restored green.
* **the bot path** — asserted, not assumed, at BOTH the simulator level (`c3` above,
  `StubProvider` needs no change) and structurally (`legal_actions.rs`'s single `EffectChoice` arm
  is the ONLY offer-construction site for any `EffectChoiceQuestion`, so "no change for
  `ChooseObject`'s new producer" is a property of that one arm, not something that could differ
  per-question elsewhere in the crate).

`git diff main..HEAD --numstat -- tools/play-server/frontend` is **EMPTY**; `npm run build` is N/A
and is stated as such rather than skipped silently.

### B6. Consumer enumeration (`EffectChoiceQuestion` / `EffectChoiceAnswer`)

Every file in the workspace referencing either type (`grep -rl`, excluding `tests/`), with its
disposition:

| file | what it does | change owed |
|---|---|---|
| `crates/engine/src/effects/mod.rs` | executor (`ChooseObject` construction, `handle_answer_effect_choice` validation, `resolve_pending_object_choices`, `default_effect_choice_answer`) | B1 (construction site only); validation and default-answer arms unchanged, verified pre-existing and correct |
| `crates/card-types/src/cards/card_definition.rs` | `LookAtTopThenPlace.optional` field doc | B1 (doc correction) |
| `crates/card-types/src/state/mod.rs` | re-export | none |
| `crates/card-types/src/state/stubs.rs` | type declarations (`EffectChoiceQuestion`/`Answer` enums, `PendingEffectChoice`) | B2 #2 (doc correction only; no field/variant change) |
| `crates/card-types/src/state/types.rs` | unrelated (`Cost`/other type used inside the enum) | none |
| `crates/engine/src/lib.rs` | re-export | none |
| `crates/engine/src/rules/command.rs` | `Command::AnswerEffectChoice { answer: EffectChoiceAnswer }` field type | none — opaque field |
| `crates/engine/src/rules/engine.rs` | `BlockingDecision::EffectChoice`, doc citing the wire-cost precedent | none — no per-variant match |
| `crates/engine/src/rules/events.rs` | `GameEvent::EffectChoiceRequired { question, .. }` field type | none — opaque field, `private_to` unaffected |
| `crates/engine/src/rules/protocol.rs` | wire-closure type registration + HISTORY doc | none — confirmed no bump owed (§0.2/B-throughout) |
| `crates/engine/src/rules/resolution.rs` | `MutateOnTop` construction (unrelated primitive) + a doctest fixture using `SearchLibrary` | none |
| `crates/engine/src/state/hash.rs` | `HashInto for EffectChoiceQuestion`/`EffectChoiceAnswer`, both already carry a `ChooseObject` arm (PB-DX28, discriminant 4) hashing exactly `{candidates, count, up_to}` — unchanged fields | none — read, verified, no edit |
| `crates/engine/src/testing/replay_harness.rs` | `auto_answer_blocking_decisions`'s `ChooseObject` arm (golden-script driver) | B2 #6 (doc correction; behaviour already correct — omniscient test driver, not a redaction boundary) |
| `crates/engine/src/testing/script_schema.rs` | doc comments citing the variant name | none — prose only, not false |
| `crates/simulator/src/decision_coverage.rs` | `OBSERVABLE_ROW_IDS`/`UNOBSERVABLE_ROW_IDS`, `row_id_for` | B3 (row list + disambiguation logic) |
| `crates/simulator/src/legal_actions.rs` | `EffectChoice` offer construction — fully generic, no per-variant match | none — verified by reading, confirmed by `c3` |
| `crates/simulator/src/params.rs` | `ActionParams.effect_choice_answer: Option<EffectChoiceAnswer>` — opaque pass-through to `Command::AnswerEffectChoice` | none |
| `tools/play-server/src/api.rs` | `validate_decision_params`'s `ChooseObject` arm (candidate-id membership + count/up_to check) | none — verified generic over candidate POPULATION, not per-zone; confirmed by all 3 HTTP tests |
| `tools/play-server/src/main.rs` | test-only (this batch's own new HTTP tests + the pre-existing DX45 section) | B5 (3 new tests) |
| `tools/play-server/src/view.rs` | `blocking_decision_view`'s `ChooseObject` arm (`question_cards` → `AnswerShapeView::PickN`) | B2 #3 (doc correction only) |
| `tools/tui/src/play/app.rs` | `EffectChoiceRequired` log-line formatter's `ChooseObject` arm (class label only, no ids ever printed) | B2 #6 (doc correction only) |

**Not a consumer, checked and confirmed**: `crates/view-model/` and `tools/replay-viewer/` —
`grep -rl "EffectChoiceQuestion\|EffectChoiceAnswer"` over both returns EMPTY. Neither the
view-model crate nor the replay viewer's Rust backend touches this type at all (StackObjectKind /
KeywordAbility are the exhaustive-match surfaces those two own, per CLAUDE.md's gotcha list, and
neither is a `KeywordAbility` or `StackObjectKind` change). The frontend's `DiscardPicker.svelte`
mentions `ChooseObject` once, in a doc comment describing the wire shape (not a false claim — no
change owed).

### Wire prediction — CONFIRMED, gate-executed

`HASH_SCHEMA_VERSION` and `PROTOCOL_VERSION` both read **82 / 41**, unmoved from the merge-base
value recorded in §0.1, matching §0.2's prediction exactly. `core::hash_schema` (36/36) and
`core::protocol_schema` (17/17) both green, executed after the FULL Half B diff (not merely
inspected). No sentinel re-pin owed and none taken.

### Full-workspace test count

`cargo test --workspace --no-fail-fast` (log: `scratchpad/pbdx35/full_test_halfb.log`):
**5,091 passed / 0 failed / 5 ignored, 63 result-producing targets** (62 → 63: one new simulator
test binary, `pb_dx35_optional_placement_channel.rs` — the flat-directory-is-a-target convention
`pb_dx32_fuzz_output.rs`'s own module doc cites). Delta from Half A's own close (5,076): **+15**,
itemised by construction rather than by a set-diff (every addition is a `#[test]` this batch wrote
and named above): 9 in the new `pb_dx35_optional_placement.rs`, 3 in the new
`pb_dx35_optional_placement_channel.rs`, 3 in `tools/play-server/src/main.rs`'s `#[cfg(test)]`
module. Zero renames, zero removals — confirmed by construction (no existing `#[test]` fn was
deleted or renamed; `pb_os8_look_at_top_then_place.rs`'s seven repaired tests each kept their name
and gained an `execute_effect` → `execute_effect_with_default_choices` call-site swap only, because
Half B's own behaviour change (asking rather than silently placing) made their PRE-EXISTING
assertions reachable only through the abort-and-replay loop — reproduced as a failure BEFORE the
fix, itemised below).

### A ripple the plan did not name: `pb_os8_look_at_top_then_place.rs` needed repair, not just
Half B's own new file

Seven of that file's PRE-EXISTING tests called `execute_effect` (the bare, non-suspending
executor) directly with `optional: true` and a real (nonempty) candidate set. Once B1 shipped,
each of those seven started SUSPENDING instead of placing (a real `ChooseObject` ask with no
banked answer), and all seven went RED — reproduced by executing `cargo test` immediately after B1
landed, before any test-file edit: `test_look_place_onto_battlefield_fires_etb`,
`test_look_place_at_most_one_even_when_two_match`, `test_look_place_min_and_max_equal_exact_mv`,
`test_look_place_truncates_at_top_n_leaves_out_of_window_match_untouched`,
`test_look_place_rest_to_bottom_positional_order`, `test_look_place_creature_to_hand_growing_rites`,
`test_look_place_cost_sacrifice_gates_and_parameterizes`. Fixed by swapping their `execute_effect`
call to `execute_effect_with_default_choices` — the SAME behaviour-preservation argument `t2`
pins, applied to seven pre-existing fixtures rather than a new one — and, for the sacrifice+
placement test, DELETING a now-redundant manual `bank_effect_choice_answer(PayOptionalCost)` call
whose purpose (answering the `place_cost` ask) is now subsumed by the SAME default-answer loop
that also answers the new `ChooseObject` ask, in the printed order (`t6`'s own finding). Three
tests (`test_look_place_no_match_leaves_all_bottomed`, `test_look_place_cost_declined_when_
unpayable_skips_placement`, `test_look_place_empty_library_places_nothing_and_skips_cost`) needed
NO change — each has either an empty candidate set or `placement_allowed == false`, so neither
path this batch touches is reached. Zero test NAMES changed in this file; zero assertions
weakened — this is disclosed as a ripple, not folded silently into "9 additions".

### Card defs — 5 comment-only edits, 0 `Completeness` marker moves

`birthing_ritual.rs`, `growing_rites_of_itlimoc.rs`, `grisly_salvage.rs`, `satyr_wayfinder.rs`,
`risen_reef.rs`. Every changed line in all five starts with `//` — verified by a Python line-level
diff scan, not eyeballed. `python3 tools/authoring-report.py` re-run to PROVE (not assume) the
predicted 0-flip: clean **1,138** (unmoved from Half A's own close), matching `CORPUS_COMPLETE`
in `pb_dx32_fuzz_output.rs`. The regeneration's self-dating churn (`docs/authoring-status.md`,
`docs/authoring-status-missing.txt`, `docs/authoring-status-prev.json`) was reverted with
`git checkout --`, per the standing convention — the doc's tracked content was already one
regeneration stale (dated `2026-09-02`, from PB-DX45) going into this task, which is reported
rather than silently fixed, since re-dating it is the coordinator's close-out call, not this
batch's.

### Gates that fired on THIS batch's own work

* `core::decision_gate::runtime_decision_coverage_roster_matches_rows` — reddened on the first
  attempt (the new row wasn't yet in `decision_coverage.rs`'s two lists); fixed by adding it;
  re-observed green.
* `core::decision_gate::auto_chosen_complete_union_is_ratcheted` (T6) — reddened (72 pinned, 67
  measured); the constant lowered to 67 with its own history paragraph, per the file's standing
  rule (read off T6's printed number, arithmetic stated as a check not the source).
* `core::decision_gate::every_baseline_entry_is_live_and_necessary` (T5) — would have reddened had
  the five stale `BASELINE` entries been left in place after the row split (verified this by
  reasoning from the function's own logic, not by leaving the bug in to watch it fire, since the
  entries were removed in the same edit that split the row).
* `simulator::pb_dx32_fuzz_output::test_dx32_row_id_for_covers_every_observable_row` (T6.2) —
  reddened until the new fixture (`look_at_top_then_place_state`) was added; re-observed green.
* `simulator::pb_dx32_fuzz_output::test_dx32_a_fuzz_run_reaches_at_least_one_served_row` (T6.3) —
  reddened (a real re-observation of a fuzz-shaped run, not a bug); re-pinned per its own module
  doc's discipline, with the new doc paragraph stating the cause (a new row entering the set) so a
  future reader does not mistake it for a trajectory change the way PB-DX18's neighbouring entry
  in the same test genuinely was.
* `cargo fmt --check` — reddened twice during authoring (stray trailing comma; a few lines needing
  rewrap); both fixed by `cargo fmt` on the specific files, re-verified clean workspace-wide.

No gate outside this list fired. `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo build --workspace`, and `tools/check-defs-fmt.sh` (1,803 defs) all clean against the FINAL
tree.

---

## §C — Coordinator close-out (post-code; §0 above is the pre-code prediction and stays as written)

### C1. The delta, byte-exact — and the one thing the byte-exact method could not see

Both logs taken with `--workspace --no-fail-fast` to a file, never piped to `tail`. Set difference
computed in Python over the exact `^test (.+?) \.\.\. (ok|ignored)` capture — **never `sort` +
`comm`**, which fabricates a removal under a UTF-8 locale (`OOS-DX20b-5`, obeyed rather than
re-learned).

```
baseline  5,058 / 0 / 5   61 result-producing targets
final     5,097 / 0 / 5   63 result-producing targets   (+2 binaries)
delta     39 additions / 0 leavers / 0 removals / 0 renames
count delta 39  ==  name-set delta 39   <- the reconciliation, see below
(re-taken AFTER the /review fix cycle, which added r8; dispatch hygiene 8)
```

**The first run of that delta reported 32 additions against a count delta of 33, and the
discrepancy is the finding.** Half B's bot-path channel probe was named
`c3_the_bot_path_is_offered_and_answers_the_same_action`, which
`crates/simulator/tests/pb_dx45_optional_cost_channel.rs:293` already used. `tests/` compiles one
binary per file, so both compiled and both ran — legal Rust, and a real measurement defect: a set
difference over test NAMES collapses the pair. Renamed
(`c3_dx35_the_bot_path_is_offered_and_answers_the_same_choose_object`), after which the two figures
reconcile. **The check that separates them costs one line — compare the name-set delta against the
`passed + ignored` delta — and nothing in the tree enforces it.** Filed as `OOS-DX35-8`.

The 39: 8 in `primitives::pb_dx35_modal_trigger_targets`, 8 in
`core::pb_dx35_modal_trigger_roster` (r1-r7 + `t_census_report`), 1 internal unit test in
`rules::abilities` (t9 — site 3 is a bare private `fn`, unreachable from an integration test, which
is disclosed in both files), 1 in `simulator::pb_dx35_modal_trigger_channel`, 9 in
`primitives::pb_dx35_optional_placement`, 3 in `simulator::pb_dx35_optional_placement_channel`,
3 HTTP probes in `tools/play-server/src/main.rs`, 5 in the close-out's
`core::pb_dx35_optional_placement_roster` (b1-b4 + `t_census_report`), and 1 added by the
`/review` fix cycle (`r8`).

**The reconciliation is now part of the measurement, not a one-off repair.** The final run's count
delta (**39**) and name-set delta (**39**) are equal and the duplicate-name scan returns EMPTY —
which is the check `OOS-DX35-8` proposes, executed on this batch's own final numbers rather than
only on the run that exposed the problem.

### C2. Line counts, measured

| area | +/− |
|---|---|
| `crates/engine/src` | **+722 / −356** (`rules/abilities.rs` +636/−337, `effects/mod.rs` +62/−12, `rules/resolution.rs` +13/−3, `testing/replay_harness.rs` +11/−4) |
| `crates/card-types/src` | +6 / −2 |
| `crates/card-defs` | +212 / −100 (12 defs; **3 re-shaped, 9 comment-only**) |
| `crates/simulator/src` | +51 / −13 |
| **`crates/view-model`** | **0 / 0** |
| `tools/` | +385 / −9 — `play-server/src/main.rs` **+366 / −0, entirely inside `#[cfg(test)]`**, plus 6 doc-comment lines in `view.rs` and 4 in `tui/play/app.rs` |

`rules/abilities.rs`'s figure is inflated by a `cargo fmt` reflow of the shortened target-derivation
chain: `git diff -w` gives **+460 / −161**, and the difference is stated rather than left to look
like 600 lines of new logic.

**`npm run build` was NOT run and that is stated rather than omitted**: it is N/A here, because
`git diff main..HEAD --numstat -- tools/play-server/frontend` is **EMPTY** and `node_modules` is
absent from this worktree. `tools/` is **not** zero, and a first draft of this line would have
implied it was.

### C3. `OOS-DP10-5`'s own sweep, EXECUTED — the table

Nine batches inherited the instruction *"Sweep for others not yet found"* without running it.
Method: every `field: _` discard in an `Effect` / `EffectAmount` / `AbilityDefinition` destructure
across `crates/engine/src/effects/` and `crates/engine/src/rules/`, **each one READ** to decide
whether the value is consumed anywhere else — because a discard at one site is not a defect if
another site consumes it.

| site | field | verdict |
|---|---|---|
| `effects/mod.rs:6607` | `LookAtTopThenPlace.optional` | the seed itself — **CLOSED by this batch** |
| `effects/mod.rs:4195` | `SearchLibrary.reveal` | the sibling the row already names (`OOS-DP9-9`) — still open, untouched |
| `effects/mod.rs:4987` | **`CounterUnlessPays.cost`** | **NEW, REAL, and the largest.** Delegates to `Effect::CounterSpell` and drops the cost, so CR 118.12a's *"unless its controller pays"* is never offered. Its in-source justification — *"the payer never has an incentive to voluntarily tax themselves without interactive choice"* — is **false on its face**: the payer is the OPPONENT whose spell is on the stack. Population **7, all `Complete` by derive, all deck-legal**. Filed `OOS-DX35-3` |
| `effects/mod.rs:9001` | `EffectAmount::CardCount.player` | not a defect — `zone: ZoneTarget` already carries the owner, so the field is redundant rather than dropped. Recorded, not filed |
| `rules/replacement.rs:2812` | `Replacement.unless_condition` | **checked and CLEAN** — the value IS consumed, at `replacement.rs:2044-2049`; `:2812` is the registration loop. **This is the row that proves the sweep read each discard rather than counting them** |
| `rules/layers.rs:1726-1737` | 12 `Characteristics` fields | not a defect — a deliberate exhaustive destructure (PB-DX43's `clear_all_abilities`, written so a new field is a compile error) |

### C4. The seed-ID collision, and why it is recorded rather than tidied away

PB-DX35 ran its two halves as two delegated implementations. **Both independently allocated
`OOS-DX35-1`** — Half A for the index-space defect, Half B for the `RevealAndRoute` residual — and
neither could see the other, because a seed ID is allocated against a registry neither had written
to yet. Found at close-out by grepping the ID rather than by trusting either report (dispatch
hygiene 5: the registry is ground truth).

Resolved by giving the number to the **majority-cited** meaning: the index-space defect carries 12
in-source cites, the `RevealAndRoute` residual carries 1. The latter became `-2` and its single
cite in `decision_site_walk.rs` was repointed in the same commit — `OOS-M11-10`'s renumbering
orphaned 30 in-source cites under a note asserting it had not, and letting the majority keep the
number is the cheap way not to repeat that.

### C5. The census corrected TWO of this document's own figures

`core::pb_dx35_optional_placement_roster::t_census_report` was written to satisfy the criterion's
*"PRINTED by a test"*, and its first run refuted two figures the coordinator had already committed
to the registry:

1. The corpus-wide "you may" population was published as **213 / 90**. The true figures are
   **365 / 165**. The 213 came from a throwaway Python script whose `oracle_text` extractor did not
   join Rust's `\`-newline line continuations, so it silently truncated every multi-line oracle
   string. Both registry rows corrected.
2. The pinned MDFC's def NAME is `Turntimber Symbiosis // Turntimber, Serpentine Wood`, not the
   file stem `turntimber_symbiosis`. A list pinned from file names is not a list pinned from
   `all_cards()`.

**PB-DX8's rule — publish the figure, do not transcribe it — caught its own author.** Every Half B
population figure in this document and in CLAUDE.md is now read off that test's output.

The oracle axis is a **conjunction**, not a needle, and the reason is measured rather than argued:
the naive `"you may put"` returns **29** defs, of which **18** are the unrelated *"you may put a
land card **from your hand** onto the battlefield"* landfall family. Two independent derivations
(a throwaway script over the def files, and the test over `all_cards()`) returned the same
**5 carriers + 11 inverse members**; the agreement is the evidence, not either run alone.

### C6. Revert matrix — `core::pb_dx35_optional_placement_roster` (added at close-out)

All executed, each reddening **exactly one** gate, tree restored green afterwards. The Half A and
Half B matrices are in §A/§B and in the implementers' own test docs.

| row | revert | reddens |
|---|---|---|
| V1 | drop `"Risen Reef"` from `CARRIERS` | `b1` only |
| V2 | plant `optional: false` on `risen_reef.rs` (a REAL def, not a fixture) | `b2` only |
| V3 | drop `"Six"` from `INVERSE_MEMBERS` | `b3` only |
| V4 | lower `MAX_YOU_MAY_DEFS` 400 → 300 | `b4` only |
| V5 | blind the axis's look-verb list (`b3`'s non-vacuity floor) | `b3` only |

V5 is the row worth reading: it proves the floor is load-bearing rather than decorative. Without
it, an axis that stopped recognising the printed shape would report **0** inverse members and
`b3` would fail on the pin — but for the wrong reason, and a future author "fixing" it by emptying
`INVERSE_MEMBERS` would ship a gate that measures nothing. The floor asserts the axis still finds
all **5** defs that DO use the primitive, which is what makes the inverse count meaningful.

### C7. Fuzz — NOT A/B'd, and the reason is attribution, not effort

`shambling_ghast` `partial` → `Complete` moves `CORPUS_COMPLETE` **1,137 → 1,138**, which re-deals
every seeded fuzz game. A merge-base fuzz A/B across that boundary measures trajectory
reindexing, not this batch — `OOS-DX21-6`'s lesson, which PB-DX51 spent a third ablation run
proving. What was done instead: `simulator::pb_dx32_fuzz_output`'s gate config was **re-observed by
execution**, and `COMMANDER_POOL` was **re-measured and found UNCHANGED at 90** rather than
reasoned about (`Shambling Ghast` is a Creature but not `SuperType::Legendary`, so it was never a
commander candidate). The T6.3 served-row partition moved and was re-observed, gaining the new
`look_at_top_then_place_optional` row — an improvement, and attributed.

### C8. Benches — MEASURED, seven runs, and the verdict is NO REGRESSION

Matched-set A/B against merge base `90c5499c` in an isolated `git worktree` with its own
`CARGO_TARGET_DIR`, HEAD in a second one. **The same-code repeatability band was measured FIRST**
(PB-DX18's lesson), from two runs of the identical merge-base code:

| bench | base 1 | base 2 | same-code band | HEAD 1 | HEAD 2 | verdict |
|---|---:|---:|---:|---:|---:|---|
| `priority_cycle_4p` | 24.529 | 24.689 | 0.65% | 24.163 | 25.023 | intervals overlap |
| `priority_cycle_6p` | 39.099 | 38.708 | 1.00% | 38.067 | 38.831 | intervals overlap |
| `sba_check` | 15.126 | 14.802 | **2.14%** | 15.217 | 14.742 | intervals overlap |
| `full_turn_4p` | 216.00 | 217.33 | 0.62% | 217.47 | 218.36 | intervals overlap |
| `full_turn_6p` | 345.32 | 343.86 | 0.42% | 341.62 | 345.25 | intervals overlap |
| `board_wipe_4p` | 119.33 | 121.57 | 1.88% | **135.15** | 118.68 | see below |

(µs, criterion's point estimate.)

**`board_wipe_4p`'s HEAD run 1 is an outlier and it was killed by execution, not by averaging.**
135.15 µs against a base range of 119.33-121.57 reads as a +12% spike, and taking the mean of the
two HEAD runs would have published a tidy, meaningless **+5.37%**. Instead a THIRD run was taken on
each side, on the same quiet machine, after both pairs: **HEAD 116.73**, **base 120.23**. Both
subsequent HEAD runs bracket below every base run, so run 1 was contended and is discarded with its
reason stated rather than folded into an average.

**Verdict: no regression on any bench, and no speed-up is claimed either.** Every base-vs-HEAD
criterion interval overlaps, and the effect sizes are all inside a same-code band that reaches
2.14% on `sba_check`. **The apparent 1.4-3% improvement on `board_wipe_4p` is deliberately NOT
claimed** — PB-DX51's discipline: `sba_check` and the priority cycles are controls here (nothing
this batch touches is on the SBA loop or the priority-pass path) and they move the same order, so a
uniform shift is a build/layout artefact of two separate compilations rather than a property of the
change.

There **is** a mechanism that would explain no regression, and it is stated as a reason to expect
the measurement rather than as a claim about it: `flush_sorted` previously performed **two**
independent target-requirement lookups per trigger (sites 1 and 2, each walking to the object and
into the registry or the runtime vec) and now performs **one** shared `trigger_modal_plan`. The
per-mode candidate derivation — the only genuinely new work — runs **only** when
`mode_targets.is_some()`, which is true for exactly 3 corpus defs and 0 of the benched fixtures.

---

## §D — The `/review` fix cycle: 9 issues, all 9 taken, four gate defeats PROVED by execution

The reviewer had a shell and used it. Its own summary is the fair one: *"the engine work is sound,
CR-faithful and honestly measured … but the batch's gates are weaker than its prose."* Every
published figure it re-derived reproduced — the 5,096 delta, all six census populations, every
line count in §C2 — and it then defeated four gates.

| # | sev | finding | disposition |
|---|---|---|---|
| 1 | MEDIUM | **`r7` defeated TWICE.** (a) A reword — *"trigger_modal_plan (PB-DX35) -- hard-codes mode 0 in both the min_modes==0 and min_modes!=0 arms"* re-asserts the exact false claim in different words **while still naming `trigger_modal_plan`**, so the negative needle missed and the positive floor passed. (b) The original needle split across a Rust `\`-newline: the RENDERED string is byte-identical to the pre-batch lie, the SOURCE no longer contains it contiguously. That is `OOS-DX51-6`'s class verbatim, in the batch that inherited the lesson. | **TAKEN.** Re-keyed on the MECHANISM in three conjuncts: the row text is NORMALISED (line continuations collapsed) before matching; a deliberately over-collecting denylist of hard-code assertions; and the positive half now demands the row cite **CR 700.2b** as well as name the function, so a sentence that names it only in order to restate the lie cannot pass. **The denylist needed a NEGATION guard and finding that out cost one red run** — the row's own honest phrasing is *"…, **not** always mode 0"*. Residual STATED: a contrived double negative evades it; both executed defeats do not. Both re-run: RED. |
| 2 | MEDIUM | **`t9` defeated.** It is the only gate behind AC 7327's headline "ONE shared arithmetic", and both its cases drove `PendingTriggerKind::Normal`. The reviewer re-planted the original `OOS-DX4-2` defect in site 3 behind `if trigger.kind == CardDefETB` — a hand-rolled fifth copy — and **the entire `mtg-engine` crate stayed green**. *A differential probe proves agreement on the branches it drives and nothing about the branches it does not.* | **TAKEN, twice over.** `t9` gains **case C** on the `CardDefETB` branch (with a non-vacuity assertion that both kinds address the same ability on that fixture, so the comparison cannot pass for the wrong reason); and the reviewer's own stronger suggestion is taken as **`r8`**, a MECHANISM gate asserting every triggered-target extraction in `crates/engine/src/rules/` lives inside `trigger_modal_plan`, with an allowlist whose reasons are re-checked in source. The defeat is RED under BOTH. |
| 3 | MEDIUM | **Dispatch hygiene 8**: `memory/workstream-state.md` and the v4 memo's row still published 5,091 / +33, superseded by this branch's own `c5094719`. The workstream-state cell is the worse one — it sits under *"every one re-run by the coordinator"*. | **TAKEN.** Both corrected, **and then corrected AGAIN** to 5,097 / +39 when this fix cycle added `r8`. The cell now discloses all three figures and the reason, because *a re-verification claim attached to a figure a later commit supersedes is worse than no claim*. |
| 4 | MEDIUM-LOW | **`b2` defeated.** Its per-def fold was `.any(optional == true)`, so a SECOND `LookAtTopThenPlace` node with `optional: false` added to `grisly_salvage` (a real deck-legal `Complete` carrier) inside a `Sequence` left `b1` and `b2` GREEN — while that node keeps the pre-batch take-when-able behaviour on a printed "you may", i.e. the defect this batch closed, alive on a `Complete` def. `b2`'s own docstring was then false about a green tree. | **TAKEN.** Fold changed to `.all(..)`; a per-def NODE COUNT added to the struct, asserted at exactly 1 for every carrier, and printed by `t_census_report` — because *an `all` fold is only as informative as the node count it folds over*. Defeat re-run: RED. |
| 5 | MEDIUM-LOW | **The trigger path mirrors ONE of the TWO author invariants its peers hard-reject.** `casting.rs:3856` and `abilities.rs:481-486` both refuse `UpToN` inside `mode_targets`; `r5` mirrored only the flat-targets rule, **naming one of two rules that sit five lines apart in the same file**. Planted `UpToN` into `retreat_to_kazandu`'s mode-0 slice: all 8 roster gates GREEN, and the behaviour genuinely wrong — an `UpToN` slot is `optional`, so the mode is judged unconditionally legal and CR 700.2b's fall-through dies. | **TAKEN on both ends.** `r5` gains the `UpToN` conjunct (with a new `mode_targets_contain_up_to_n` axis), and `trigger_modal_mode_is_legal` gains a **fail-closed** arm: a trigger is not a command and has nothing to reject to, so an `UpToN` slice makes the mode ILLEGAL and CR 700.2b falls through to one that is not. |
| 6 | LOW | **A false stated reason in a card-def comment** — `felidar_retreat.rs` said *"both modes take no `DeclaredTarget`"*; mode 1 has `DeclaredTarget { index: 0 }` inside a `ForEach`. The CONCLUSION (out of population) is right; the reason is what the next batch reuses. | **TAKEN.** Rewritten to name the real separator (the flat `targets` list) and to say explicitly that the `DeclaredTarget` is a `ForEach` iteration binding, so a reader grepping for `DeclaredTarget` does not find a contradiction. |
| 7 | LOW | **`shambling_ghast`'s new `Complete` note implies a board-sensitivity it does not have** — its DSL mode 0 (Treasure) has an EMPTY requirement slice, so it is first-legal in *every* board state and the -1/-1 mode is unreachable always, which is what `t4` asserts. The note borrowed `retreat_to_kazandu`'s genuinely board-sensitive shape. | **TAKEN.** Rewritten to say what the fix actually changes (the -1/-1 target no longer leaks onto the Treasure mode, so the trigger RESOLVES instead of being removed) and to name the surviving agency gap and its seed. |
| 8 | LOW | **CR 700.2b mode legality is decided PER SLOT** and does not consult `forced_answer_breaks_distinctness`, so a mode with a candidate per slot but no legal COMBINATION is chosen and then removed. Zero corpus exposure. | **TAKEN.** Stated as a residual in `trigger_modal_mode_is_legal`'s own doc and filed as **`OOS-DX35-10`**. |
| 9 | NIT | **Two filed seeds had no in-source anchor** (`OOS-DX35-7`, `OOS-DX35-9`) — *"a residual filed without an anchor is the one nobody finds again"*. | **TAKEN.** `-9` anchored on `trigger_modal_plan`'s doc (the LKI widening it describes); `-7` anchored on `retreat_to_coralhelm`'s mode 0, where the costless "may" lives. |

### The fix cycle's own finding: `r8` refuted its author in its first run

`r8`'s doc, as first written, said *"exactly THREE such extractions exist in `rules/`, and
`rules/mana.rs`'s site 4 does not match — it never extracts `targets` at all"*. **It does**:
`mana.rs:821-828` destructures `targets` straight out of an `AbilityDefinition::Triggered` pattern.
The throwaway script behind that sentence searched for `AbilityDefinition::Triggered {` **with the
brace**, and `mana.rs` puts the brace on the next line. The population is **SIX**, not three.

Site 4 is allowlisted rather than unified, and its exemption is narrow and **re-checked in
source**: it uses the binding at exactly one place, `targets.is_empty()` — a presence test
deciding whether the ability must use the stack (CR 605.5a) — and the gate asserts that call still
exists AND that `targets.get(` / `targets.iter()` / `targets[` / `mode_targets` do **not** appear
in the file. *An allowlist whose reason nothing verifies is a comment.*

### Post-fix-cycle numbers, RE-TAKEN

Tests **5,097 / 0 / 5**, **63** targets, byte-exact **39 additions / 0 leavers / 0 removals /
0 renames**, count delta **39** == name-set delta **39**, duplicate scan EMPTY. HASH **82** /
PROTOCOL **41** still unmoved. `clippy --workspace --all-targets -D warnings`, `cargo fmt --check`,
`tools/check-defs-fmt.sh` (1,803 defs) and `cargo build --workspace` all clean against the FINAL
tree. Coverage re-regenerated: **1,138 / 518 / 147 identical**, so the fix cycle moved no marker;
self-dating churn reverted. Seed range corrected `-1..9` → **`-1..10`** on every headline surface
by re-checking against the registry AFTER the fix cycle.

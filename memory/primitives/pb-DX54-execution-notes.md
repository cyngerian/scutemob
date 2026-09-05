# PB-DX54 — execution notes (`scutemob-232`)

Seed **OOS-DX25c-6**; riders **OOS-DX25-4**, **OOS-DX25b-4**. v4 queue rank 17.

---

## §0.1 — Baseline (measured BEFORE any edit)

`cargo test --workspace --no-fail-fast` to a file:

```
5,210 passed / 0 failed / 5 ignored     67 result-producing targets
```

**Reproduces PB-DX53's published close pin EXACTLY** (5,210 / 0 / 5, 67 targets). No
correction is owed and `OOS-DX51-5`'s non-reproducing-pin failure does not recur — the
fourth consecutive batch in which an inherited pin reproduces.

---

## §0.2 — CR research, and a CITE CORRECTION owed to this task's own framing

**Misdirection, ruling 2004-10-04** (MCP, verbatim):

> *"You can choose to make a spell on the stack target this spell (if such a target choice
> would be legal had the spell been cast while this spell was on the stack). The new target
> for the deflected spell is not chosen until this spell resolves. **This spell is still on
> the stack when new targets are selected for the spell.**"*

and, the other half of the same ruling set:

> *"You can't make a spell which is on the stack target itself."*

So the ruling asks for exactly two things at once: the RESOLVING spell must be visible as a
redirect candidate, and self-targeting must still be refused.

**The CR basis is CR 608.2n, not CR 608.2m — and the seed row, the v4 memo row and
acceptance criterion 7379 all cite 608.2m.** Checked against the rules server rather than
inherited:

* **CR 608.2n** — *"As the final part of an instant or sorcery spell's resolution, the spell
  is put into its owner's graveyard. As the final part of an ability's resolution, the
  ability is removed from the stack and ceases to exist."* This is the rule that says the
  entry is still on the stack for the whole of the resolution, and CR 608.2's own preamble
  reinforces it: *"The steps described in rule 608.2n and 608.2p are followed last."*
* **CR 608.2m** — *"If an instant spell, sorcery spell, or ability that can legally resolve
  **leaves the stack once it starts to resolve**, it will continue to resolve fully."* That
  is about an object removed by SOMETHING ELSE mid-resolution (a Stifle-shaped effect, a
  counter that lands during a suspended resolution). It says nothing about when the
  resolving object's own departure happens, so it cannot be the warrant for this fix.

PB-DX52's own narrative already cites CR 608.2n correctly for *"an ability ceases to
exist"*; the mis-cite entered at `OOS-DX25c-6`'s filing and propagated into the memo row and
the dispatch AC. Corrected in every surface this batch writes (`OOS-DX54-1`).

---

## §0.3 — WIRE PREDICTION, PER OPTION, WRITTEN BEFORE ANY PRODUCTION LINE

Ground verified by reading the two gates' own source (and confirmed by executing them
green at the merge base, which is what proves the exclusion is live rather than declared):

* `crates/engine/tests/core/protocol_schema.rs:116` —
  `CLOSURE_MUST_NOT_CONTAIN = ["GameState", "PlayerState", "StackObject", "CardDefinition"]`.
  **`StackObject` is excluded as well as `GameState`**, which matters for option B.
* `crates/engine/tests/core/hash_schema.rs` — `decl_fingerprint` is a source scan of
  `GameState`'s **serde** type closure, so any field added to `GameState` moves it.

| Option | HASH | PROTOCOL | Reason |
|---|---|---|---|
| **A — resolve-in-place** (the pop moves to the end of `resolve_top_of_stack_inner`) | **UNMOVED** | **UNMOVED** | No type, no variant and no field is added anywhere. `git diff` over `state/hash.rs` and `rules/protocol.rs` is EMPTY; the change is a control-flow move plus reads of data already hashed. |
| **B — shadow entry** (`GameState.resolving_stack_object: Option<StackObject>`) | **+1** | **UNMOVED** | `GameState` is in PROTOCOL's `CLOSURE_MUST_NOT_CONTAIN`, and so is `StackObject`, so neither the container nor the payload can reach the wire closure — the PB-DX51 `CombatState.had_attackers` precedent exactly (HASH 81→82, PROTOCOL 41 unmoved). HASH moves because `decl_fingerprint` scans `GameState`'s serde shape. |
| **C — a new `EffectChoiceQuestion` variant** (what rider `OOS-DX25b-4` would need) | **+1** | **+1** | `EffectChoiceQuestion` is on the wire through `GameEvent::EffectChoiceRequired` and `Command::AnswerEffectChoice`, and inside `GameState` through `pending_effect_choice`. This is the PB-DX45 precedent (PROTOCOL 38→39 / HASH 77→78, one bump each). |

**Prediction of record, made before any production line changed: option A is expected to
be chosen and to move NEITHER gate.** If it is chosen, both gates are executed and their
UNMOVED result published with this counterfactual stated; if the measurement in §0.4 forces
option B instead, HASH bumps once and PROTOCOL does not.

Closure type counts predicted UNCHANGED at **98** (PROTOCOL) / **132** (HASH) under every
option except C, which adds one variant of an existing type and therefore also leaves both
counts unchanged.

---

## §0.4 — BLAST RADIUS, MEASURED BY EXECUTION (AC 7379's own requirement)

The acceptance criterion says the design is *"settled at stage 0 with the blast radius of
each MEASURED by executing the suite with the pop moved"*. It was, before the design was
chosen and before a single line of the real fix was written.

**Scaffold**: `resolve_top_of_stack_inner` renamed to `resolve_top_of_stack_body`, its
`pop_back()` changed to `back().cloned()`, and a wrapper that removes the entry BY ID after
the body returns (`.iter().position(|so| so.id == rid)`).

**Result — `cargo test --workspace --no-fail-fast`:**

```
5,207 passed / 3 failed / 5 ignored     67 result-producing targets
```

**All three failures are SOURCE gates. ZERO behavioural tests moved.**

| Failing test | Why it fired |
|---|---|
| `core::pb_dx48_announcement_site_roster::r1_call_site_census_is_pinned` | keyed on `func: "resolve_top_of_stack_inner"`; the scaffold RENAMED that function |
| `core::pb_eng2_targets_announced::every_announcement_site_is_classified` | same key, same cause |
| `core::pb_dx52_stack_target_roster::r1a_no_reopened_find_or_position_scan_of_stack_objects` | the scaffold's `.position(\|so\| so.id == rid)` genuinely re-open-codes the announced-id-to-stack-entry resolution that gate exists to forbid |

All three are the gate being **right**, and all three are avoidable rather than
allowlistable — which is what the shipped shape does (see §1).

## §0.5 — DESIGN: resolve-in-place, and why not the shadow entry

| | resolve-in-place | shadow entry on `GameState` |
|---|---|---|
| wire | **HASH unmoved / PROTOCOL unmoved** | HASH **+1** / PROTOCOL unmoved |
| behavioural blast radius | **0 tests** (measured, §0.4) | 0 by construction — nothing else can see the field |
| call sites changed | 1 peek + 3 departure calls | `stack_index_for_announced_target` returns `Option<usize>`, an index into a vector the shadow is **not in**, so its return type must become an enum and all **6** consumers (`Effect::CounterSpell`, `Effect::ChangeTargets`, `Effect::CopySpellOnStack`, `casting.rs`'s two single-target arms, `validate_stack_object_satisfies_requirement`) must learn the new shape |
| CR fidelity | CR 608.2n verbatim: the object really is on the stack until the final part of its resolution | the object is on the stack *for targeting* and off it for everything else — a second, partial answer to "is it on the stack" |

**Chosen: resolve-in-place.** It is cheaper on every axis measured, and the axis where it
could have been more expensive — behavioural blast radius — came back at zero.

The wire prediction for the chosen option (**both gates UNMOVED**) is the one committed in
§0.3 before any production line changed.

**Both alternatives' wire costs were VERIFIED BY EXECUTION at stage 0, not inferred:**

* `StackObject` planted into `hash_schema.rs`'s `CLOSURE_MUST_NOT_CONTAIN` → `hash_schema::
  state_closure_is_not_vacuous_and_bounded` **FAILS** (*"StackObject entered the GameState
  serde closure"*), while `protocol_schema.rs` already lists `StackObject` and its gate is
  **green** at HEAD. So option B is HASH-only, exactly as predicted.
* `EffectChoiceQuestion` planted into BOTH lists → **both** gates fail
  (*"EffectChoiceQuestion entered the GameState serde closure"* / *"…entered the
  Command/GameEvent closure"*). So option C — the variant rider `OOS-DX25b-4` would need —
  moves BOTH, exactly as predicted.

Both plants were reverted by restoring byte-exact copies; `git diff` over
`crates/engine/tests/core/` is empty.

---

## §1 — The shipped shape, and the design the suite could NOT have refuted

`resolve_top_of_stack_inner` PEEKS (`state.stack_objects.back().cloned()`). The entry
departs through one new function, `depart_resolving_stack_entry`, called at **three** sites:

1. the CR 608.2b **fizzle tail**, immediately before `sba::check_and_apply_sbas`;
2. the **main tail**, immediately before `check_triggers_with_timing` (CR 608.2p) →
   `check_and_apply_sbas` (CR 704.3) → `grant_priority_to_active_player` (CR 117.3b);
3. an idempotent **backstop** in `resolve_top_of_stack`, after `resolve_top_of_stack_inner`
   returns.

### Why NOT the function boundary — and this is the half no test would have told us

The §0.4 scaffold removed the entry only at the function boundary, and **every behavioural
test in the workspace stayed green**. It is still wrong, on two SBAs that read
`state.stack_objects` from inside `check_and_apply_sbas`:

* **CR 714.4** (`rules/sba.rs`, the Saga sacrifice): *"…and it isn't the source of a chapter
  ability that has triggered but not yet left the stack"*. CR 704.3 checks SBAs when a
  player would receive priority, i.e. AFTER CR 608.2n. A resolving FINAL chapter ability
  that had not yet departed would postpone its own Saga's sacrifice by a whole SBA round.
* **CR 309.6** (`rules/sba.rs`, dungeon removal): the same shape, for a `RoomAbility` still
  on the stack.

Stated as a reason rather than discovered by the suite, because the suite is measurably
incapable of discovering it — that is what "5,207 / 3, zero behavioural" means.

### Why a backstop, and why it is idempotent

**Four** paths return from `resolve_top_of_stack_inner` before either departure site: three
ability-fizzle / intervening-if returns (`AbilityResolved` with no target, Offspring's
CR 603.4 check, Gift's CR 603.4 check) and the CR 608.2d suspension. None of the four runs a
trigger, SBA or priority step, so the ORDER is unobservable on those paths — but the
departure is still owed. Discharging it in one place rather than at four `return` statements
is PB-DP8's rule (*a guard that returns early inherits the obligation of the statements it
skipped*) made structural: a FIFTH such return cannot forget it.

Idempotence is **CR 608.2m**, and this is the one place in this batch where 608.2m is the
right cite: an effect resolving during this very resolution can now legitimately remove the
entry — the resolving spell is a reachable `Effect::CounterSpell` victim for the first time —
and CR 608.2m says it *"will continue to resolve fully"* anyway. "Already gone" is therefore
a LEGAL state at every departure site, and the correct response is to do nothing.

### The lookup obeys `r1a` rather than being allowlisted around it

`stack_index_for_announced_target`'s first clause is `so.id == announced`, and the id passed
here is a stack-ENTRY id read from `state.stack_objects.back()` moments earlier. Its second
clause compares against a CARD id, and the two spaces are disjoint by construction (both
minted from the one monotone `timestamp_counter`), so it cannot fire. The shared function
therefore resolves exactly the entry meant, and `pb_dx52_stack_target_roster::r1a` stays
green with no allowlist entry added — the gate is obeyed, not respelled around. (Respelling
it as `retain(|so| so.id != rid)` would also have satisfied the needle, and would have been
exactly the *"a gate you edit prose to satisfy has stopped measuring"* failure PB-DX52 names.)

### Suite after the engine change alone

**5,210 / 0 / 5** — unmoved from the pre-edit baseline. No gate fired.

---

## §2 — Consumer audit (AC 7379): every resolution-time reader of `state.stack_objects`

Derived by grepping every `stack_objects` read across `crates/engine/src`,
`crates/simulator/src`, `crates/view-model/src` and `tools/`, then classifying each by its
ENCLOSING FUNCTION — command-time or resolution-time. A `handle_*` function reached from
`process_command` is command-time by construction: the entry is already gone.

| Consumer | When it runs | Verdict |
|---|---|---|
| `casting.rs`'s three timing reads inside `handle_cast_spell` — sorcery speed (`StackNotEmpty`), Plot (CR 702.170d), Teferi (CR 101.2) | `handle_cast_spell`, whose **only** caller is `rules/engine.rs`'s `Command::CastSpell` arm | command-time. **A cast during resolution — cascade, CR 608.2g — does NOT go through `handle_cast_spell`**; `resolution.rs` builds the stack object directly and says so in its own comment (*"WITHOUT ever calling `handle_cast_spell`"*). Unaffected. |
| `handle_play_land`, `handle_plot_card`, `handle_suspend_card`, `handle_bring_companion`, `handle_activate_ability`, `handle_unearth_card`, `handle_embalm_card`, `handle_eternalize_card`, `handle_encore_card`, `handle_saddle_mount`, `handle_scavenge_card`, `handle_activate_craft`, `handle_activate_loyalty_ability`, `handle_level_up_class` | all inside `handle_*` command handlers | command-time. Unaffected. |
| `handle_all_passed` | decides whether to resolve at all, BEFORE `resolve_top_of_stack` | unaffected. |
| `discharge_effect_choice_on_concede` | `Command::Concede` handler | command-time. Unaffected. |
| `GameState::maybe_clear_lki_objects` (SR-13) | THREE call sites — `finish_stack_resolution` (AFTER `resolve_top_of_stack` returns), `handle_all_passed`'s stack-empty branch, and `reset_turn_state` | all command-time. **Checked specifically because a stack that is non-empty during resolution would suppress the LKI clear**; it cannot be reached there. Unaffected. |
| `loop_detection::compute_mandatory_state_hash` | reached only via `check_for_mandatory_loop`, whose three call sites are two in `engine.rs`'s stack-EMPTY branch and one in `abilities.rs::run_flush_resume_obligations`, gated to `EnterStepPriority`/`EnterStepCleanup` | never computed during a resolution. **Checked specifically because the entry's id is fresh per iteration and would have defeated CR 726 loop detection had it been folded in.** Unaffected. |
| `sba.rs`'s two stack reads — CR 714.4 Saga sacrifice, CR 309.6 dungeon removal | inside `check_and_apply_sbas`, called from resolution's own tails | **AFFECTED, and this is why the departure point is where it is.** See §1. |
| `Effect::CounterSpell`, `Effect::CopySpellOnStack`, `Effect::ChangeTargets` | resolution-time, through `stack_index_for_announced_target` | **AFFECTED, and this is the fix.** `ChangeTargets` is the seed; the other two now see the resolving entry too — see the "never double-seen / never resolved twice" analysis below. |
| the **six** `exists_on_stack` liveness reads (PB-DX52's `r1` population) — 3 in `effects/mod.rs`, 1 each in `casting.rs`, `abilities.rs` and `resolution.rs`, re-counted at HEAD rather than transcribed | resolution-time and cast-time | affected only in the direction of ACCEPTING the resolving entry as live, which CR 608.2n makes correct. Unreachable in practice for the resolving object itself, because CR 601.2c self-exclusion refuses it at announcement (`t4`). |
| `copy.rs`'s three pushes and `resolution.rs`'s two — copy / cascade / storm / suspend | resolution-time, all `push_back` | unaffected in ORDER: a push during resolution lands ABOVE the resolving entry, which is CR 608.2g verbatim (*"That spell becomes the topmost object on the stack, and the currently resolving spell or ability continues to resolve"*). Before this batch it also landed on top, because the entry had been popped — so the topmost-ness is identical and only the entry BELOW it differs. |
| `crates/simulator/src/invariants.rs` `check_stack_consistency`, `crates/view-model` `stack_kind_info`, `tools/tui/.../stack_view.rs`, `tools/play-server`, `tools/replay-viewer` | all read a state at a COMMAND BOUNDARY | the entry is gone by then. Unaffected — and this is what makes "never double-seen" true for every external observer by construction rather than by care. |

**Never double-seen**: the entry is in `state.stack_objects` exactly once (it was never
copied out and back — it is the same element, never popped), and the local `stack_obj` the
body reads is a CLONE, not a second vector element. **Never resolved twice**: nothing calls
`resolve_top_of_stack` re-entrantly — its only two callers are `handle_all_passed` and
`effects::handle_answer_effect_choice`'s CR 608.2d replay tail, both command handlers.

---

## §3 — Riders (AC 7380), each decided with the reason posted as a task comment

### `OOS-DX25-4` — **TAKEN**

The row's own prescribed fix shape, verbatim: *"a sibling `source_of(&StackObjectKind) ->
Option<ObjectId>` in `state::stack_registry`, exhaustive like `card_in_stack_zone`, consumed
by both paths."* PB-DX52 built the helper (it was a live requirement on the retarget path,
`OOS-DX25c-3`); this batch consumes it, which is all the row had left.

Both counter paths — `effects/mod.rs`'s `Effect::CounterSpell` and
`resolution.rs::counter_stack_object` — carried a **byte-identical** two-arm `match` naming
`ActivatedAbility` and `TriggeredAbility` and falling through `_ => None` for the other 23
kinds, so a Stifle-shaped counter removed the entry and reported nothing to the event log.
PB-DX48 made four of those kinds (`ForecastAbility`, `ScavengeAbility`, `LoyaltyAbility`,
`KeywordTrigger`) reachable from Ward, so the silence was live rather than theoretical.

`GameEvent::SpellCountered` now names a source for **20 of 25** kinds. The five that still
name none — `EmbalmAbility`, `EternalizeAbility`, `EncoreAbility`, `ScavengeAbility`,
`RoomAbility` — are a **measured absence** already documented on `source_of` itself (CR 400.7
retired the card that was the cost; CR 309.4c puts a Room's generator in the command zone),
not a missing arm. This is a diagnostics change and not a state one: no card moves
differently and no zone diverges, exactly as the row says.

### `OOS-DX25b-4` — **DECLINED, and re-filed with the exact missing variant and its measured wire cost**

AC 7380 offers both: take it, *"or [it] is explicitly re-filed with the exact missing
question variant and its wire cost"*. Re-filed, on cost measured rather than estimated:

* **The missing variant** is `EffectChoiceQuestion::ChooseNewTargets { stack_object:
  ObjectId, per_index_candidates: Vec<Vec<Target>> }` with the matching
  `EffectChoiceAnswer::ChooseNewTargets { chosen: Vec<Option<Target>> }` — `Option` per index
  because CR 115.7d is *"you MAY choose new targets"*, so declining a single index is a legal
  answer where CR 115.7a's `must_change` has no such option. `retarget_candidates` +
  `validate_targets_inner` already compute the candidate sets; what does not exist is the
  question, the answer, and their consumers.
* **Wire cost, VERIFIED BY EXECUTION** (§0.5): `EffectChoiceQuestion` is in BOTH closures, so
  a new variant moves **HASH +1 and PROTOCOL +1**. AC 7381 budgets *"ONE bump each at most"*,
  which this would exactly consume — for a rider, in a batch whose own subject moves neither.
* **Consumer cost, MEASURED not estimated**: `grep -rn "EffectChoiceQuestion::"` over
  production source (tests excluded) finds **~110 sites across 14 files** —
  `effects/mod.rs` (32), `tools/play-server/src/api.rs` (15), `view.rs` (10), `main.rs` (9),
  `simulator/src/decision_coverage.rs` (8), `state/hash.rs` (8), `tools/tui/src/play/app.rs`
  (7), `testing/replay_harness.rs` (7), `simulator/src/params.rs` (4),
  `card-types/src/cards/card_definition.rs` (4), and five more. That is PB-DX45's
  eight-consumer shape at more than ten times the scale, plus a new picker component in the
  Svelte frontend.

So it is a batch of its own, and it is **not** dead weight in the meantime: `must_change:
false` leaves `deflecting_swat` a deterministic no-op, which is under-permission (the player
never gets a decision CR 115.7d gives them), not a wrong outcome.

**One correction to the row, and it is this batch's own subject**: the row says the seed is
about the RESOLUTION being a no-op while *"the announcement half now works"*. Both halves are
now more reachable than the row records — PB-DX52 widened the announcement to
`TargetSpellOrAbility`, and PB-DX54 makes the RESOLVING redirector itself a legal new target
for a `must_change: true` victim. Neither moves `deflecting_swat`, because its own
`must_change` is `false`; the row is updated to say so rather than left implying PB-DX54
touched it.

### T7 / T8's route-around docs (AC 7380)

`crates/engine/tests/primitives/pb_dx25c_retarget_legality.rs`:

* **T7** (`t7_misdirection_is_itself_a_legal_candidate`) carries a paragraph headed
  *"Discovered structural fact, worth recording because it decided this test's shape"* which
  states that `TargetSpellWithSingleTarget` / `TargetSpellOrAbilityWithSingleTarget` *"CANNOT
  observe the actively-resolving spell as a candidate"*. **That is false at HEAD** and is
  rewritten to say what is now true, with the reason the fixture keeps its
  `TargetSpellWithFilter` shape anyway (it is the only probe in the tree covering the FILTER
  variant, which PB-DX54's own probes do not).
* **T8** (`t8_self_targeting_is_still_refused`) carries a paragraph explaining why it needs
  **FOUR** stack objects rather than three: the fourth exists *only* because the resolving
  Misdirection was invisible and so could not serve as the alternative self-exclusion is
  measured against. With the fix it can, so PB-DX54's own `t4` is the three-object version
  and T8's doc records that its fourth object is now redundant-but-harmless coverage rather
  than a workaround.

---

## §4 — Census and coverage (AC 7381)

### The census, PRINTED by `core::pb_dx54_resolving_entry_roster::r6`

Union **11** defs across the two axes. Declared axis (the three `TargetRequirement` variants
AC 7381 names, decided by `decision_site_walk::def_contains_variant` — **not** by a substring
scan over `format!("{def:#?}")`, which counts a `Completeness` note as a declarer;
`OOS-DX53-2`, and the mechanism that separates them is that `def_contains_variant` matches a
unit variant's serialized name EXACTLY):

| def | declared | completeness |
|---|---|---|
| Misdirection | `TargetSpellWithSingleTarget` | **Complete** |
| Bolt Bend | `TargetSpellOrAbilityWithSingleTarget` | **Complete** |
| Untimely Malfunction | `TargetSpellOrAbilityWithSingleTarget` | `Partial` |
| Deflecting Swat | `TargetSpellOrAbility` | **Complete** |

**`OOS-DX25c-6`'s "2 deck-legal `Complete`" cell REPRODUCES**, and this is worth saying
plainly after four consecutive batches in which a yield cell turned out to be a floor. The two
are Misdirection and Bolt Bend. `Untimely Malfunction` declares one of the two affected
requirements and is `Partial`, so its deck-legal exposure is zero; `Deflecting Swat` declares
the THIRD variant, which was never blind (see below).

**Why only the two single-target requirements were blind**, re-derived rather than inherited:
`plan_target_change` validates candidates against the **VICTIM's** `target_requirements`, and
only `TargetSpellWithSingleTarget` / `TargetSpellOrAbilityWithSingleTarget` consult
`state.stack_objects` (through `stack_index_for_announced_target`, to count the candidate's
targets and classify it as a spell). `TargetSpell`, `TargetSpellWithFilter` and
`TargetSpellOrAbility` decide the object branch on `obj.zone == ZoneId::Stack` alone
(`casting.rs`'s `TargetSpell | TargetSpellWithFilter | TargetSpellOrAbility` arm), and the resolving spell's CARD never left `ZoneId::Stack` — which is
exactly why PB-DX25c's T7 could route around the defect with `TargetSpellWithFilter`.

### The INVERSE ORACLE axis found a sibling gap no document names

**7 printed-only** defs, and `declared-only` is **0**. Five of the seven print *"choose new
targets"* for a **COPY** (CR 707.10's *"you may choose new targets for the copy"*), not for
CR 115.7's retarget: `Rings of Brighthearth`, `Illusionist's Bracers`, `Complete the Circuit`,
`Flusterstorm`, `Train of Thought`. `Effect::CopySpellOnStack`'s own doc says
*"choose-new-targets deferred to M10"* — so the copy family has the SAME under-permission
`OOS-DX25b-4` describes for `must_change: false`, through a different effect, and no registry
row names it. Filed as **`OOS-DX54-2`**.

*The two axes do not nest*, for the fifth batch running (PB-DX26 → PB-DX43 → PB-DX35 →
PB-DX53 → here), asserted by `r6` rather than stated.

### Coverage flips: **0, predicted with the reason BEFORE any regeneration**

`git diff --numstat <merge-base>..HEAD` over `crates/card-defs` and
`crates/card-types/src/cards` is **EMPTY** — this batch edits no card definition of any kind,
so no `Completeness` marker can move, `CORPUS_COMPLETE` cannot move, and no seeded fixture can
be re-dealt (`OOS-CARDS2-3`'s budget checked and found not owed). The regeneration is run
anyway, against the FINAL tree, rather than the empty-diff shortcut being taken as the answer.

---

## §5 — The roster (`core::pb_dx54_resolving_entry_roster`), 9 gates

| gate | what it pins |
|---|---|
| `r1` | `resolve_top_of_stack_inner` does not pop the entry off the front of its own resolution — keyed on the MECHANISM (any mutable route to a `pop_back`/`pop_front` on the stack, direct or `let`-aliased), not on one spelling, because `OOS-DX51-6` defeated three successive drafts of a same-shape gate with exactly one statement of indirection |
| `r1b` | that detector, proven on synthetic input in BOTH directions — it fires on the direct and the aliased pop, and does NOT fire on a pop of something that is not the stack (otherwise `r1` is "any pop anywhere" and measures nothing) |
| `r2` | **the design decision the whole suite could not refute**: every `check_and_apply_sbas(` / `grant_priority_to_active_player(` inside `resolve_top_of_stack_inner` is preceded by a departure, plus exact counts (2 inside, 1 backstop in the wrapper) |
| `r2b` | that ordering detector, proven on synthetic input in both directions — it fires on the function-boundary shape and stays quiet on the correct one |
| `r3` | the departure resolves its entry through `stack_index_for_announced_target` and re-open-codes no `.position(` / `.find(` / `.retain(` / `.iter()` of its own |
| `r4` | **rider `OOS-DX25-4`, behaviourally**: `counter_stack_object` names a source for a `LoyaltyAbility` — one of the 23 kinds the old two-arm `match` fell through |
| `r4b` | the same on the OTHER counter path, `Effect::CounterSpell`, because the two carried byte-identical copies of the defect |
| `r5` | the CONSUMER ROSTER behind `r2`'s argument: `sba.rs` reads the stack at exactly 2 decision sites, pinned by count AND by their two CR cites |
| `r6` | the census, PRINTED |

**`r2`'s first draft was wrong and its own failure is what corrected it.** The window was set
at 1,200 bytes by taste; the real distances, measured afterwards, are 169 / 466 / 520 and
**1,605** bytes. Widened to 4,000 — and the SECOND measurement, the one that stops the window
failing open, is recorded beside it: the two departure calls sit **464,693 bytes apart**, so no
window of this size can let one tail vouch for the other's site. `OOS-DX39-8`'s shape (an
over-wide detector turning a verdict into a non-verdict) avoided by measuring both bounds rather
than one.

**`r4`/`r4b` are behavioural, not source, on purpose** — `OOS-DX52-2`: a row that reddens only a
source gate is telling you the behaviour has no probe. Both drive a **real
`Command::ActivateLoyaltyAbility`** rather than a hand-built `StackObject`, so the classification
is proven on a shape production can actually produce (`OOS-DX47-4`'s naked-`ObjectSpec` lesson).

### Coverage regeneration, RUN rather than shortcut

`python3 tools/authoring-report.py` against the tree:

```
1,803 files | clean 1,140 (63.2%) | todo 516 | empty 147
```

**Identical to PB-DX53's close in every bucket** — clean 1,140, todo 516, empty 147 — so
coverage is **UNMOVED at 1,140/1,803 = 63.2%** with **0 flips**, exactly as predicted before the
run. The only diff against the committed report is self-dating churn (the generation timestamp,
the branch/SHA line, and the delta columns, which read `+1 / -1` on the committed copy because
PB-DX53's flip is what they are measuring against); all three generated files reverted.

The empty card-def diff made the shortcut available and it was **not taken** — the regeneration
was run anyway, which is the standing rule in this queue.

---

## §6 — Wire: **HASH 85 / PROTOCOL 44, BOTH UNMOVED — ZERO bumps for the whole PB**

Gate-executed, not assumed: `cargo test -p mtg-engine --test core -- hash_schema
protocol_schema` is **53 passed / 0 failed**, which includes `declaration_fingerprint_is_pinned`,
`stream_fingerprint_is_pinned`, `protocol_schema_fingerprint_is_pinned`,
`history_is_append_only` and `frozen_prefix_is_pinned` on both sides.

**Closure type counts, MEASURED by raising each gate's `MIN_CLOSURE_TYPES` to 9999 and reading
the gate's own panic text** (never transcribed from the previous batch — PB-DX8's rule):

* PROTOCOL: *"protocol closure is only **98** types"*
* HASH: *"GameState serde closure is only **132** types"*

Both **unchanged**, exactly as §0.3 predicted. Both floors restored; `git diff` over
`crates/engine/tests/core/` is empty.

**The counterfactual, stated because "unmoved" is only informative beside what would have
moved it** (§0.3, verified by execution at stage 0):

* the rejected **shadow-entry** design — `GameState.resolving_stack_object: Option<StackObject>`
  — would have been **HASH +1 / PROTOCOL unmoved**. `StackObject` planted into `hash_schema.rs`'s
  `CLOSURE_MUST_NOT_CONTAIN` fails that gate; `protocol_schema.rs` already lists `StackObject`
  *and* `GameState` and is green at HEAD.
* the declined rider **`OOS-DX25b-4`** would have been **+1 on BOTH**: `EffectChoiceQuestion`
  planted into both lists fails both.

**Nothing was owed and nothing was done**: `git diff` over `crates/engine/src/state/hash.rs` and
`crates/engine/src/rules/protocol.rs` is **EMPTY**, so there was no sentinel to re-pin, no
survivor scan to run on either axis, no `OOS-DX18-3` over-replacement read to take, no history
row to append and no `FROZEN_HISTORY_PREFIX_DIGEST` to re-pin. The two append-only gates were
executed anyway, green, as the evidence that none was owed rather than as a claim that none was.

---

## §7 — What this batch did NOT do, stated rather than omitted

* **`npm run build` was NOT run**, and it is N/A rather than skipped:
  `git diff --numstat <merge-base>..HEAD -- tools/` is **EMPTY**, so no frontend or play-server
  line moves, and `node_modules` is absent from this worktree. Unlike PB-DX52, no acceptance
  criterion predicted otherwise.
* **The fuzzer was not A/B'd**, and the reason is a reason rather than a measurement dressed as
  one: no `Completeness` marker moved anywhere (the card-def diff is empty), so no seeded fixture
  is re-dealt and `OOS-CARDS2-3`'s usual budget does not apply. What WOULD have justified one —
  a change to the order or identity of stack entries a fuzz trajectory sees — is exactly what
  the consumer audit in §2 shows does not happen at any command boundary.
* **`OOS-DX25b-4` was declined**, with the reason and the measured cost in §3.
* **The tracked zero-byte `{}` file on `main` was left in place** (`OOS-DX54-3`), because a
  main-scope tidy inside a correctness batch is an unexplained diff at collect.

---

## §8 — Probes, and three corrections the coordinator made to delegated output

### The files

* `crates/engine/tests/primitives/pb_dx54_resolving_entry_target_space.rs` — t1..t7 (8 tests)
* `crates/simulator/tests/pb_dx54_resolving_redirect_channel.rs` — c1..c3 plus one sentinel
* `crates/engine/tests/core/pb_dx54_resolving_entry_roster.rs` — r1..r6 (9 tests), §5

### Correction 1 — an EMPTY `#[test]` was removed, and the reason is a number

The probe agent shipped `t7b_cr_714_4_same_command_sacrifice_is_confounded_by_a_different_bug`
as a `#[test]` whose entire body was a comment. Its DOC was excellent — a careful record of a
real, out-of-scope defect with four rejected alternative constructions — and is preserved
verbatim in the file's module header. The WRAPPER was wrong: a test that asserts nothing always
passes, contributes no coverage, and **adds +1 to this batch's own reported test delta for a row
that tests nothing**, which corrupts the one figure every later batch inherits as its baseline.
The finding is filed as **`OOS-DX54-4`** instead — which is where a real defect with no probe
belongs, on PB-DX49's own `OOS-DX49-1` precedent (*a probe asserting today's behaviour would
have to be inverted by whoever fixes it, and nothing this batch touched is on that path*).

### Correction 2 — the coordinator's own edit deleted a PASSING test, and that is recorded

Removing that wrapper, the coordinator's cut ran back to the wrong section banner and took
`t7_non_final_chapters_resolve_normally_with_correct_departure_timing` and two helpers with it.
Recovered from the agent's transcript and re-verified green. Written down because a silent
recovery is how a deleted test becomes a permanently missing one — and because the same
over-wide-cut shape is what `OOS-DX18-3` filed about a sentinel re-pin.

### Correction 3 — `t5`'s headline assertion message OVERCLAIMED

Its message read *"not zero (pre-fix, the popped entry made this arithmetic answer 0, which is
the defect)"*. The count is taken **before** `resolve_top_of_stack` is called, when the entry is
on the stack under BOTH revisions — pre-fix it answered 1 there too, and only answered 0 INSIDE
the resolution, which no assertion in this file can observe directly. Reworded to state that it
is a PRECONDITION whose value is that the `TargetsChanged` assertion below it cannot be
satisfied by a double-count artefact. The probe's discrimination was always in that second
assertion; only the prose was wrong.

### The defect the probe that could not be built found (`OOS-DX54-4`)

CR 714.4's exemption — *"…isn't the source of a chapter ability that has **triggered** but not
yet left the stack"* — is checked against `state.stack_objects` alone, while `enter_step` queues
the chapter trigger, THEN runs SBAs, THEN flushes. So a Saga is sacrificed one mechanism before
its FINAL chapter reaches the stack, and that chapter resolves sourceless and does nothing.
Observed in one command's event slice (`CounterAdded {Lore, 3}` → `PermanentDestroyed` →
`AbilityTriggered` → `AbilityResolved` with no effect event), with chapters I and II resolving
correctly on the same fixture, which is what isolates it to the final chapter. **Pre-existing,
proven structurally**: `git diff <merge-base>..HEAD` over `sba.rs`, `turn_actions.rs`,
`engine.rs`, `replacement.rs` and `saga.rs` is EMPTY, and the sacrifice happens at step entry,
outside any resolution, so this batch's departure point is never reached in that trace.

### Two claims in the channel brief that the channel agent found FALSE, reported not worked around

1. `ObjectSpec::with_mana_cost(ManaCost { blue: 1, .. })` does **not** make an object Blue —
   `legal_actions::eligible_pitch_cards` reads a separate `colors` field a naked `ObjectSpec`
   never derives from `mana_cost`. Every pitch-fodder object needs an explicit
   `.with_colors(vec![Color::Blue])`. Named in neither the brief nor either reference file.
2. `LocalGame::submit`'s returned events do NOT surface bot-driven resolutions: `advance()`
   stops only for a human seat, so a bot's pass and everything it triggers runs inside
   `advance()` and never reaches the caller. `c1` reads `game.journal_since(cursor)` instead,
   with the mechanism documented rather than only the fix.

# PB-DX28 — part 1 execution notes (§2 owner axis + §3 `EffectTarget::DamagedPlayer`)

Scope of this run: `pb-plan-DX28.md` §2 and §3 ONLY. §1 (untargeted-choice channel) and §4
(allowlist retirement) are separate runs — not touched here.

---

# PB-DX28 — part 2 execution notes (§1, the untargeted-choice channel)

Scope: `pb-plan-DX28.md` §1 ONLY, per `pb-plan-DX28-part2.md`'s consumer-surface census and
derivation rule. §4 (allowlist retirement) NOT touched except for one line removed as a direct,
mechanical fallout of this run's own migration (see "What §4 forced" below). Starting commit:
`6aeb2008` (part 1 SHIPPED and committed). Tree left DIRTY, uncommitted, per instructions.

## Files touched

**DSL / engine**:
- `crates/card-types/src/cards/card_definition.rs` — `ChoiceZone` enum, `EffectTarget::ChosenObject`
  variant.
- `crates/card-types/src/state/stubs.rs` — `EffectChoiceQuestion::ChooseObject`,
  `EffectChoiceAnswer::ChooseObject`.
- `crates/card-types/src/cards/helpers.rs`, `crates/card-types/src/cards/mod.rs`,
  `crates/engine/src/cards/mod.rs`, `crates/engine/src/lib.rs` — `ChoiceZone` re-export chain
  (mirrors part 1's `TargetOwner` chain).
- `crates/engine/src/effects/mod.rs` — `EffectContext.chosen_objects` field (+ 2 constructors, 2
  `ForEach` sub-context rebuilds); `filter_matches_object_untargeted` (the dedicated
  filter-only predicate, NOT `casting::validate_targets_inner`); `resolve_pending_object_choices`
  (the pre-pass, called at the top of `execute_effect_inner`); `resolve_effect_target_list_indexed`'s
  new `ChosenObject` arm (fail-closed `debug_assert`); `default_effect_choice_answer`'s new arm;
  `handle_answer_effect_choice`'s variant-agreement match + new per-variant legality arm.
- `crates/engine/src/state/hash.rs` — `EffectTarget::ChosenObject` (discriminant 13),
  `impl HashInto for ChoiceZone`, `EffectChoiceQuestion::ChooseObject` / `EffectChoiceAnswer::
  ChooseObject` (discriminant 4 each).
- `crates/engine/src/testing/replay_harness.rs` — new `ChooseObject` arm in `answer_effect_choice`.
- `crates/engine/src/testing/script_schema.rs` — `EffectChoiceScriptAnswer.chosen: Vec<String>`.
- `crates/engine/src/rules/abilities.rs` — one `chosen_objects: Vec::new()` backfill (a
  hand-built `EffectContext` literal for a condition check).

**Consumer surfaces** (part2 plan §2, every line re-grepped before editing, all moved since the
2026-08-14 measurement — confirmed by diffing against what the plan cited):
- `crates/simulator/src/decision_coverage.rs` — `row_id_for`'s `EffectChoiceQuestion` match
  restructured `.map` → `.and_then`, new `ChooseObject => None` arm (see "R4 of part2 plan §2"
  below — NOT a new decision-audit ROW).
- `crates/simulator/src/legal_actions.rs` — VERIFIED generic, 0 lines changed
  (`LegalAction::AnswerEffectChoice` calls `default_effect_choice_answer` uniformly).
- `crates/engine/src/rules/engine.rs` — VERIFIED generic, 0 lines changed (`BlockingDecision::
  EffectChoice` carries no per-variant code).
- `crates/simulator/src/params.rs` — VERIFIED generic, 0 lines changed (`Command::AnswerEffectChoice`
  construction reads `params.effect_choice_answer` uniformly).
- `tools/play-server/src/view.rs` — `AnswerShapeView::PickN` gains `min_count: usize`; the
  Discard arm sets `min_count: *count as usize` (unchanged behaviour); new `ChooseObject` match
  arm, routed through the EXISTING `question_cards` channel (no 4th raw `GameState` read —
  `test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places` stays green unmoved).
- `tools/play-server/src/api.rs` — `question_kind` new arm; `validate_decision_params`'s
  `(question, answer)` match new arm (full CR 115.10/608.2 legality: dedup, subset, exact-`min(
  count,len)`-when-`!up_to`, `<=count`-when-`up_to`).
- `tools/play-server/frontend/src/lib/DiscardPicker.svelte` — new `minCount` prop (default
  `count`, so every pre-PB-DX28 caller is behaviourally unchanged), `canConfirm` widened to a
  range, header/button text updated for the `up_to` case.
- `tools/play-server/frontend/src/lib/ActionBar.svelte` — `minCount={currentShape.min_count}`
  passed on the `PickN` branch only (the `Subset` branch has no `min_count` field and is untouched).
- `tools/tui/src/play/app.rs` — new `EffectChoiceQuestion::ChooseObject => "choose object"` arm
  in the event-log formatter (found by `cargo build --workspace`, not named in either plan file).

**Tests**:
- `crates/engine/tests/core/pb_dx28_chosen_object_roster.rs` (NEW) — R1-R4.
- `crates/engine/tests/primitives/pb_dx28_untargeted_choice.rs` (NEW) — T1-T10 behavioural probes.
- `crates/engine/tests/core/main.rs`, `crates/engine/tests/primitives/main.rs` — `mod` registration.
- `crates/engine/tests/primitives/pb_dp9_effect_choice.rs` — `test_dp9_mana_ability_gate` extended
  to 5 channels (`"ChosenObject"` needle added).
- **Pre-existing tests repaired as DIRECT, MECHANICAL fallout of the migration** (all 4 reasoned
  through below): `crates/engine/tests/core/completeness_deviation_scan.rs`,
  `crates/engine/tests/core/decision_gate.rs`, `crates/engine/tests/primitives/
  pb_dp8_trigger_target_choice.rs`, `crates/engine/tests/primitives/pb_dx4_baseline_triage.rs`.
- `crates/engine/tests/primitives/primitive_pb37.rs` — 1-line `chosen_objects: Vec::new()`
  backfill for a hand-built `EffectContext` literal.

## The 17-def migration table

| def | ability | before | after |
|---|---|---|---|
| 10 Karoos | ETB Triggered | `targets: [TargetPermanentWithFilter(Land+You)]`, `MoveZone{DeclaredTarget{0}}`, `Hand.owner = OwnerOf(DeclaredTarget{0})` | `targets: []`, `MoveZone{ChosenObject{Battlefield, Land+You, 1, false}}`, `Hand.owner = OwnerOf(`the same `ChosenObject` value`)` |
| `shrieking_drake`, `whitemane_lion` | ETB Triggered | `targets: [TargetCreatureWithFilter(You)]` (implicit creature check) | `targets: []`, `ChosenObject{Battlefield, Creature+You, 1, false}` (creature-ness now EXPLICIT via `has_card_type`, since the untargeted predicate has no separate "must be a creature" branch the way `TargetCreatureWithFilter`'s `!is_creature` guard does) |
| `sword_of_truth_and_justice` | combat-damage Triggered, `AddCounter` half only | `targets: [TargetCreatureWithFilter(You)]` on the ability | `targets: []`, `ChosenObject{Battlefield, Creature+You, 1, false}` on `AddCounter.target`. Equip {2}'s OWN `TargetCreatureWithFilter(You)` is UNTOUCHED (a real CR 702.6a target) |
| `cloud_of_faeries` | ETB Triggered | `targets: [UpToN{2, TargetLand}]`, 2× `UntapPermanent{DeclaredTarget}` | `targets: []`, ONE `UntapPermanent{ChosenObject{Battlefield, Land, 2, true}}` (no `You` — printed "lands", not "lands you control") |
| `frantic_search` | Spell | `targets: [UpToN{3, TargetLand}]`, 3× `UntapPermanent{DeclaredTarget}` | `targets: []`, ONE `UntapPermanent{ChosenObject{Battlefield, Land, 3, true}}` |
| `rewind` | Spell | `targets: [TargetSpell, UpToN{4, TargetLand}]`, `CounterSpell{DeclaredTarget{0}}` + 4× `UntapPermanent{DeclaredTarget}` | `targets: [TargetSpell]` (slot 0 KEPT, real printed target), `CounterSpell{DeclaredTarget{0}}` UNCHANGED, ONE `UntapPermanent{ChosenObject{Battlefield, Land, 4, true}}` replaces slot 1 + the 4 `UntapPermanent`s. The def's long "pooled indexing" comment REWRITTEN (it described a mechanism that no longer exists) |
| `takenuma_abandoned_mire` | Channel Activated | `targets: [TargetCardInYourGraveyard(Creature\|Planeswalker)]`, `MoveZone{DeclaredTarget{0}}` | `targets: []`, `MoveZone{ChosenObject{YourGraveyard, Creature\|Planeswalker, 1, false}}` |

All 17 stay `Completeness::Complete`. **Zero completeness markers changed.**

## Consumer-surface census — what I actually verified, with line numbers AT THIS COMMIT

Every line number the two plan files cited was **stale** (both plan files say so explicitly,
`OOS-DX6-5`) — re-grepped, not trusted:

- `crates/simulator/src/decision_coverage.rs:256-276` (`row_id_for`) — the ONE site part2 plan §2
  named that needed real design judgment, not a mechanical arm. See "R4" below for the decision
  taken (NOT a new decision-audit ROW).
- `crates/simulator/src/legal_actions.rs:519-532` (the `EffectChoice` arm inside the pending-decision
  dispatch) — confirmed generic via `default_effect_choice_answer(&question)`, 0 lines changed.
- `crates/engine/src/rules/engine.rs` `BlockingDecision::EffectChoice` (3 sites, 161/199/253 in the
  plan's stale measurement) — confirmed generic, 0 lines changed.
- `tools/play-server/src/view.rs` — the `EffectChoiceQuestion` match inside
  `LegalAction::AnswerEffectChoice`'s rendering arm (originally cited ~2116, actually at this
  commit inside a larger match after PB-DX23/DX27 growth) — found by re-grep, not by trusting the
  line number.
- `tools/play-server/src/api.rs` — `question_kind` (originally ~912) and `validate_decision_params`
  (originally ~524-586) — both re-grepped, both moved from the plan's cited lines.
- **`tools/tui/src/play/app.rs`** — a SIXTH consumer surface, in NEITHER plan file, found only by
  `cargo build --workspace` (the TUI's event-log formatter has its own exhaustive
  `EffectChoiceQuestion` match, `app.rs:642` at this commit). The dispatch brief's own framing —
  "the auto-target picker turned out to be two functions rather than one" as part 1's example of a
  short site list — repeated here on a SIXTH consumer, not a second function at an already-named
  site. **A `cargo build --workspace` compile error is a stronger census method than either plan
  file's own reading of the source, because it cannot miss an exhaustive match.**

## R4 (the inverse-axis roster gate) — measured convergence, not a re-derivation of the planner's

`pb-plan-DX28-part2.md` §3 asks for "no `Complete` def still pairs `slots > 'target'-word-count`
outside a named allowlist ... This is the census, frozen." The FIRST draft of this gate — the
literal reading of plan §0's Axis A ("sum every declared `TargetRequirement` slot ... compare with
the number of `\"target\"` occurrences") — measured **40 false positives** against the live corpus.
Every one was chased to a real, nameable cause and fixed in the GATE (not worked around by
allowlisting the symptom):

1. **Case sensitivity** (40 → 12): `"Target creature ..."` at a sentence start was invisible to a
   case-SENSITIVE `"target"` match. Fixed: `.to_lowercase()`.
2. **`UpToN` weighted by its own `count` field** (12 → 8, net, after #1): `elder_deep_fiend.rs`'s
   "tap up to four target permanents" is ONE printed "target" word for a REAL `UpToN{count:4}`
   slot; weighting the slot side by 4 made a genuinely CR-correct target read as 4-slots-vs-1-word.
   Fixed: `UpToN` contributes exactly 1 slot, however large its `count` — it is one element of
   `targets: Vec<TargetRequirement>`, which is the DSL's own notion of "a slot".
3. **`UpToN.inner` double-counted** (8 → 2): both the bare-string counter (unit-variant `inner`,
   e.g. `TargetPermanent`) and the object-key counter (filter-carrying `inner`, e.g.
   `force_of_vigor.rs`'s `TargetPermanentWithFilter`) were counting the WRAPPED requirement a
   SECOND time on top of the UpToN slot itself. Fixed: both counters skip any match whose direct
   parent JSON key is `"inner"`.
4. **DFC back faces have their OWN `oracle_text`, not folded into the front's** (2 → 1):
   `thaumatic_compass.rs` // Spires of Orazca prints "target attacking creature" on the BACK face
   only; `def.oracle_text` alone undercounted. Fixed: the word count reads `def.oracle_text` +
   `back_face.oracle_text` + `adventure_face.oracle_text` — the plan's own phrase, "the COMBINED
   oracle text", taken literally.
5. **`"Connive // Concoct"` (1 → 0, by DEFERRAL, not a bug fix)**: Concoct's half — "Surveil 3, then
   return a creature card from your graveyard to the battlefield" — prints NO "target" and is
   authored as a real `TargetCardInYourGraveyard`. This is a GENUINE, PREVIOUSLY-UNKNOWN 18th
   `OOS-DX4-6` member, found by this batch's own gate, that neither plan file names. **NOT migrated
   in this run** — the 17-member roster was the plan's own reviewed scope (R1 pins it at exactly
   17), and migrating an 18th, un-reviewed member here would be the "I'll just fix one more" scope
   creep `memory/conventions.md`'s "Implement-phase default-to-defer" rule exists to stop. Recorded
   in `SLOT_COUNT_REFUTED` with the reason spelled out, NOT silently swallowed into a
   "refuted" bucket it does not belong in — filed here as the batch's own finding for a follow-up.

Final state: R4 passes with **zero unexplained violations** and **one explained, deferred one**
(`Connive // Concoct`). The equip-carrying-Equipment allowlist entries (10 named cards, matching
plan §0.1) turned out to be REDUNDANT once `AttachEquipment` was excluded structurally (CR 702.6a's
implicit, never-printed target) — kept anyway as a second, independent floor.

## Fallout the migration itself forced (not additive scope — required by §1 alone)

- **`completeness_deviation_scan.rs`'s `sword_of_truth_and_justice` ALLOWLIST entry went dead.**
  The entry existed to allowlist a deviation this batch's own migration CLOSES. The gate's own
  message said so verbatim ("no longer matches any deviation needle ... remove it"). Removed —
  exactly that one entry, nothing else in `ALLOWLIST` (that is §4's job, explicitly out of scope
  here; `staff_of_compleation` and `nether_traitor`'s entries are untouched).
- **`decision_gate.rs`'s `triggered_targets` row census dropped 74 → 60** (measured, not
  back-derived): 13 defs (10 Karoos + `shrieking_drake` + `whitemane_lion` +
  `sword_of_truth_and_justice`) lost their non-empty `targets` list on the Triggered node the
  `triggered_targets` predicate counts. Floor re-pinned to the measured value with the reason
  stated in the assertion message.
- **`pb_dp8_trigger_target_choice.rs::test_dp8_up_to_n_accepts_n_targets_not_one`** cited
  `Cloud of Faeries` as one of its two motivating oracle examples for the `UpToN`-accepts-N-not-1
  fix. Cloud of Faeries no longer uses `UpToN` (by DESIGN — "untap up to two lands" was itself an
  instance of the class this batch fixes). Test's oracle census narrowed to `Elder Deep-Fiend`
  alone; doc comment corrected to say so, not silently trimmed.
- **`pb_dx4_baseline_triage.rs::sword_of_truth_and_justice_targets_only_your_creature`** — a
  PRE-EXISTING test that went RED on arrival and encoded the NOW-SUPERSEDED fix (PB-DX4's
  controller-axis repair on a REAL `TargetCreatureWithFilter`, which this batch's own plan names
  as `OOS-DX4-6`'s live instance). Rewritten to assert the SAME controller/self-exclusion facts on
  the NEW primitive's filter (`EffectTarget::ChosenObject`'s `TargetFilter`), not weakened —
  documented in the test's own doc comment as a rewrite, with the reason.

## Revert matrix

All executed against LIVE source (`crates/engine/src/effects/mod.rs`), rebuilt, observed red,
restored, reconfirmed green with a full `cargo test --workspace --no-fail-fast` (571 passed / 2
failed [the two version gates, unmoved] / 0 ignored, identical before and after every revert).

| # | Revert | Site | Probes covered | Observed |
|---|---|---|---|---|
| R1 | Disabled `resolve_pending_object_choices`'s CALL at the top of `execute_effect_inner` (a `const` bool guard) | `execute_effect_inner`'s entry | T1-T7, T9, T10 (9 of 10) | **RED**: every one hit the fail-closed `debug_assert!(false, "... was resolved with no banked answer ...")` in `resolve_effect_target_list_indexed`'s `ChosenObject` arm, naming the exact `ChosenObject` value unresolved. T8 (stack-shape only, no resolution) correctly UNAFFECTED — the one test that doesn't touch this path. |
| R2 | `filter_matches_object_untargeted` rejects any candidate carrying `Hexproof`/`Shroud` (simulating the pre-batch defect: routing through full CR 115 legality) | `filter_matches_object_untargeted`'s body | T1, T2 | **RED**, precisely: `t1_hexproofed_land_is_eligible...` and `t2_shrouded_creature_is_eligible...`, both on "the choice must be ASKED" (their SECOND-candidate setup no longer suspends because the hexproof/shroud candidate is excluded, collapsing the "ask" case to a determined 1-candidate case). Every OTHER test unaffected — precise, single-property discrimination. |
| R3 | `resolve_pending_object_choices`'s `determined` short-circuit forced to `None` unconditionally (never auto-resolve) | `resolve_pending_object_choices`'s determined-answer computation | T3, T4, T5, T7, T10 (5 of 10) | **RED**, precisely the determined-shape-dependent tests: T3 ("must resolve immediately, not fizzle and not suspend"), T4 ("zero candidates is DETERMINED"), T5 ("zero candidates ... still DETERMINED"), T7 (graveyard mill-count assertion, since the Sequence's second half never got a chance to auto-apply before the first `pending_effect_choice` check its own harness doesn't drive further), T10 ("one eligible card is DETERMINED"). T1/T2/T6/T8/T9 (which either expect an ASK already, or don't touch resolution) correctly UNAFFECTED. |

Zero UNDISCRIMINATED rows — every probe that touches the primitive was individually reddened by a
revert of the exact mechanism it exercises; the one test that does NOT touch resolution (T8) is
correctly unaffected by both resolution-side reverts, which is itself informative (T8 tests a
DIFFERENT property — the trigger's declared-targets list at STACK-PUSH time, CR 603.3d — not
resolution behaviour, and the revert matrix proves that separation is real, not assumed).

## Live digests, taken from the gates' own output (NOT bumped — coordinator's job)

- `hash_schema::declaration_fingerprint_is_pinned`: pinned (unmoved, HASH_SCHEMA_VERSION 75)
  `e8ca51103996c3094a0c6c1e1107511e2f98719e15cf0fe15f1726cc730f4ca5`; **LIVE**
  `06208006f9fb87b49e3f15b1132f4dbf2656da44a47895d2ea58e88aa97348e0` (131 types, up from part 1's
  130 — `ChoiceZone` + the two `ChooseObject` variants + `EffectTarget::ChosenObject` all reachable
  from `GameState`'s closure via `embedded_effect: Option<Box<Effect>>` etc).
- `protocol_schema::protocol_schema_fingerprint_is_pinned`: pinned (unmoved, PROTOCOL_VERSION 36)
  `bdd02df0eb7f84f0a957852a7e0944affa7e0f7c8de1348990ad53d1c5e73f62`; **LIVE**
  `03c5a4ac138556dd27c63a00088624287070a6107d382220b16c67b0df3d00a3` (98 types, up from part 1's
  97). Digest UNCHANGED between the engine-changes commit and the card-def-migration commit (card
  defs are data, not new types — confirmed by re-running the gate after EVERY edit phase, not
  assumed).
- Neither `HASH_SCHEMA_VERSION`, `PROTOCOL_VERSION`, nor either pinned fingerprint constant was
  edited in this run, per instruction.

## What the plan files got wrong (both are claims like any other)

1. **`pb-plan-DX28-part2.md` §2's consumer-surface table names 5 files; the real count is 6.**
   `tools/tui/src/play/app.rs` carries its OWN exhaustive `EffectChoiceQuestion` match (the TUI's
   event-log formatter) and is in NEITHER plan file. Found only by `cargo build --workspace`.
2. **The plan's `pb-plan-DX28.md` §1.4 "Supported arms" list (`MoveZone`, `AddCounter`,
   `UntapPermanent`) is correct as the closed corpus population, but its claim that reaching an
   unsupported arm is caught ONLY by the roster gate (R3) undersells it** — this run also wired a
   runtime `debug_assert!` fail-closed path (in `resolve_effect_target_list_indexed`'s
   `ChosenObject` arm) that fires INDEPENDENTLY of R3 the moment an 18th, unsupported use is
   authored and actually resolved, which is exactly what R1 revert-proof R1 above exercises.
3. **Neither plan file anticipated the SR-25 bare-lookup ratchet reacting to
   `filter_matches_object_untargeted`'s first-draft `state.objects.get(&id)`.** Fixed by routing
   through `state.expect_object(id)` (SR-4 engine-bug classification, since every caller passes an
   `id` freshly drawn from the SAME `state.objects` walk) — mirrors part 1's own identical finding
   on `EffectTarget::DamagedPlayer`'s resolver, so this is the SAME class recurring, not a new one.
4. **Plan §0.1's refutation list undersold its own completeness.** It refutes the equip-carrying
   Equipment population by NAME (10 cards); the real, general reason (CR 702.6a's implicit,
   never-printed target) covers the WHOLE Equip-carrying population structurally, which R4's design
   now encodes directly (`AttachEquipment` node subtraction) rather than by an ever-growing name
   list — the 10 names are now redundant, not load-bearing.
5. **The plan's §1.6 probe list did not anticipate the determined-short-circuit needing its OWN
   dedicated tests beyond "cloud_of_faeries: up_to lets the chooser take fewer".** T5 (the
   zero-candidate `up_to` case) surfaced a design point neither plan file states explicitly:
   `up_to` does NOT short-circuit merely because `candidates.len() < count` (only on the EMPTY
   set) — a 1-candidate "up to 2" choice is genuinely asked, because the player could still
   legally choose zero. This is CR-correct (confirmed by reading the shipped
   `resolve_pending_object_choices` code, not assumed) and is now the ONLY place either plan
   document — or this note — states it in so many words.

## Deferred, not fixed (report per instructions)

`"Connive // Concoct"`'s Concoct half is a genuine, previously-unfiled 18th `OOS-DX4-6` member
(see R4 §5 above). Recorded in `SLOT_COUNT_REFUTED` with the deferral reason inline; NOT migrated;
NOT silently absorbed into a "refuted" classification it does not belong in.

## Starting point

Resumed from `memory/primitives/pb-DX28-RESUME.md` at commit `e5ee1994` ("mechanical — owner:
None at all 46 WheneverCreatureDies sites"), which did not compile: 7 errors (1 `E0432` unresolved
import, 3 `E0027` missing-field patterns, 1 `E0063` missing-fields initializer, 2 `E0004`
non-exhaustive-match) exactly as the dispatch brief predicted. All 7 fixed by implementing
behaviour, not by adding wildcards — see "Engine changes" below.

## Engine changes (files touched, with line references as of this commit)

- `crates/card-types/src/cards/mod.rs`, `crates/card-types/src/cards/helpers.rs`,
  `crates/engine/src/cards/mod.rs`, `crates/engine/src/lib.rs` (already had it),
  `crates/engine/src/state/hash.rs`, `crates/engine/src/testing/replay_harness.rs`:
  `TargetOwner` was declared in `card_definition.rs` by the WIP commit but not re-exported through
  four of the five re-export chains card defs / engine internals / tests actually import it
  through. Added to each.
- `crates/engine/src/effects/mod.rs`:
  - `resolve_effect_target_list_indexed`: new `EffectTarget::DamagedPlayer` arm, resolving from
    `ctx.damaged_player` via `state.expect_player` (not a bare `.players.get`, to keep the SR-25
    bare-lookup ratchet at its pinned ceiling — see "Bare-lookup ratchet" below). Resolves to the
    EMPTY set when no damaged-player context exists, matching every other single-player
    `EffectTarget` arm in this resolver (`TriggeringCreature`/`EquippedCreature`/
    `LastCreatedPermanent`) rather than falling back to `ctx.controller` the way
    `PlayerTarget::DamagedPlayer` does — deliberate, and stated in the type's own doc comment.
  - `filter_states_a_quality`: added `qualities.owner = TargetOwner::default();` to the exclusion
    list, with the CR 701.23b rationale in-line.
- `crates/engine/src/rules/casting.rs`, `validate_object_satisfies_requirement`: added a
  `passes_owner` check (mirroring `passes_controller`) to all FOUR filter-carrying arms —
  `TargetCreatureWithFilter`, `TargetPermanentWithFilter`, `TargetCardInYourGraveyard`,
  `TargetCardInGraveyard` — matching `filter.owner` against `obj.owner`/`caster`.
- `crates/engine/src/rules/abilities.rs`:
  - `trigger_battlefield_target_matches`: `passes_owner` added to the `TargetCreatureWithFilter`
    and `TargetPermanentWithFilter` arms, matching `f.owner` against `obj.owner`/`trigger.controller`
    ("you" = the ability's controller, same convention as `passes_controller` in this function).
  - `trigger_target_candidates`: `owner_ok` added to both `TargetCardInYourGraveyard` and
    `TargetCardInGraveyard` filter closures.
  - The `AnyCreatureDies` BATTLEFIELD dispatch loop (`collect_graveyard_carddef_triggers`'s
    battlefield sibling, ~line 4960): `dying_is_token` and the new `dying_owner` now share ONE
    `state.objects.get(&dying_obj_id)` call (`dying_obj_snapshot`) rather than two separate bare
    lookups — see "Bare-lookup ratchet". `df.owner_you`/`df.owner_opponent` checked against
    `dying_owner` vs `obj.owner` (the watching permanent's own owner).
  - `collect_graveyard_carddef_triggers`'s `WheneverCreatureDies` arm (the GRAVEYARD dispatch
    site, ~line 7395): destructured the new `owner` field as `owner_scope` (avoiding a shadow of
    the outer `owner` variable, which is the trigger source's own owner from
    `ZoneId::Graveyard(owner)`). `dying_owner` read via `state.fizzle_object(*new_grave_id)` (a
    rules-correct fizzle classification, not `expect_*` — the card may have left the graveyard
    between the death event and this dispatch) and compared against `owner` (the trigger source's
    owner). The PRE-EXISTING comment above the `controller`/`death_scope` field ("do NOT read the
    dying object's owner here instead, that would give the SAME DSL field two different
    meanings") is about that OTHER field and was left untouched — it does not apply to the new,
    separate `owner` field.
- `crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs`: `TARGET_FILTER_FIELDS`
  (33rd field, `owner`) — see "Unplanned failure" below.
- Four test files fixed for the mechanical `E0063`/`E0027` fallout of the new `TargetFilter.owner`
  / `DeathTriggerFilter.owner_you`/`owner_opponent` / `TriggerCondition::WheneverCreatureDies.owner`
  fields: `crates/engine/tests/rules/creature_triggers.rs`,
  `crates/engine/tests/rules/etb_trigger_subtype_filter.rs`,
  `crates/engine/tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs`,
  `crates/engine/tests/primitives/pbn_subtype_filtered_triggers.rs`.

## Card-def repairs (exactly the three named in the dispatch brief, no others)

- `crates/card-defs/src/defs/staff_of_compleation.rs`: `TargetPermanentWithFilter` now carries
  `owner: TargetOwner::You` and DROPS `controller: TargetController::You`. In-def comment rewritten
  (it used to record the gap as unfixable — now false).
- `crates/card-defs/src/defs/nether_traitor.rs`: `controller: None, owner: Some(TargetOwner::You)`.
  In-def comment rewritten (used to claim "the DSL has no owner-scoped death trigger" — now false —
  AND corrected the note's own `fecundity` citation, which the plan's §0.2 census found wrong:
  `fecundity`'s gap is a CONTROLLER gap [`PlayerTarget::ControllerOf(TriggeringCreature)`], not an
  ownership approximation, per `fecundity.rs`'s own `partial` note. `fecundity.rs` itself was NOT
  edited — that's `nether_traitor`'s citation of it being wrong, not `fecundity`'s own content).
- `crates/card-defs/src/defs/sword_of_war_and_peace.rs`: `targets: vec![]`; both
  `DeclaredTarget { index: 0 }` (the `DealDamage.target`) and `PlayerTarget::DeclaredTarget { index:
  0 }` (the `ZoneTarget::Hand.owner` and the `EffectAmount::CardCount.player`) replaced with
  `EffectTarget::DamagedPlayer` / `PlayerTarget::DamagedPlayer`. Comment rewritten to note the old
  comment already claimed `ctx.damaged_player` resolution while the code used a declared target —
  the PB-DX27 stale-note class, live in a def this batch's own census found.

All three stay `Completeness::Complete`. No completeness marker was changed anywhere.

## Enforcement-site census actually verified (not just the brief's list)

- `casting::validate_object_satisfies_requirement`: verified by reading the whole function,
  confirmed exactly four filter-carrying `TargetRequirement` arms exist
  (`TargetCreatureWithFilter`, `TargetPermanentWithFilter`, `TargetCardInYourGraveyard`,
  `TargetCardInGraveyard`); all four now carry `passes_owner`.
- `rules::abilities`'s triggered-ability auto-target picker: TWO functions, not one —
  `trigger_battlefield_target_matches` (battlefield family, boolean predicate) AND
  `trigger_target_candidates` (candidate ENUMERATION, separate function, two graveyard-arm
  closures). The brief named the first; the second was found by reading the whole file's
  `TargetFilter`-consuming surface and is equally load-bearing (it is what populates a picker /
  bot candidate list, not just what validates a submitted answer). Both fixed.
- `rules::queries::spell_target_requirements` / `legal_targets_per_slot`: VERIFIED, not assumed —
  read `legal_targets_per_slot` (`queries.rs:214-259`) and confirmed it delegates EVERY candidate
  through `casting::validate_targets_inner` → `validate_object_satisfies_requirement`, the exact
  function already fixed. No separate owner-axis code exists in `queries.rs`; PB-DX20's "the offer
  layer and the cast path are one arithmetic" claim holds for this axis too.
- `filter_states_a_quality` (`effects/mod.rs`): confirmed as the brief specified, `owner` added to
  the exclusion list.

No enforcement site was found that the brief's list missed (unlike the four preceding DX-family
batches this brief itself warned about) — but see "Unplanned failure" below for a REACTIVE gap the
brief's list also missed: a THIRD, pre-existing structural gate (`pb_dx42a_continuous_condition_
roster.rs`) that keys on `TargetFilter`'s exact field COUNT and would have silently gone blind the
moment `owner` was added, with no compile error and no test failure pointing at the cause without
investigation.

## Unplanned failure found and fixed: `pb_dx42a_continuous_condition_roster::t6`

Reproduced red at HEAD (not predicted by the brief) after all 7 build errors were fixed. Root
cause: `TARGET_FILTER_FIELDS`, a hand-maintained `&[&str]` fingerprint used by
`object_field_set_equals` to recognize a serialized JSON node as "a `TargetFilter`" by EXACT field
SET equality (`m.len() != fields.len()` short-circuits first). Adding `owner` as `TargetFilter`'s
33rd field changed every `TargetFilter` node's serialized shape; the fingerprint (still 32 entries)
stopped matching ANYTHING corpus-wide, so axis 2 (`subtree_contains_target_filter`) silently went
from `{2 members}` to `{}` with zero card-def or condition-dispatch code touched. Confirmed by
diffing against a pristine `c5b9e459` worktree (`git worktree add`), where the same test passes.
Fixed by adding `"owner"` to the constant and updating its doc comment with the mechanism, so the
next field addition finds the explanation already written rather than re-deriving it. `t9`
(structural: the fingerprint must literally match the struct declaration, read from source) confirms
the fix rather than needing a hand check.

## SR-25 bare-lookup ratchet: kept at its pinned ceiling, not raised

Two new bare `.objects.get(&id)` calls this batch's dispatch logic needed would have raised
`src/effects/mod.rs` 108→109 and `src/rules/abilities.rs` 75→77. Both avoided without adding a
raised-ceiling exception:

- `effects/mod.rs`'s new `EffectTarget::DamagedPlayer` arm uses `state.expect_player(dp)` (an
  `engine-bug` classification — `state.players` never loses entries, CR 800.4a removes objects not
  players) instead of a bare `state.players.get(&dp)`, matching the SAME function's existing
  `AttackTarget` arm's convention two cases above it.
- `abilities.rs`'s battlefield `AnyCreatureDies` loop: the existing `dying_is_token` bare lookup and
  the new `dying_owner` read now share ONE `state.objects.get(&dying_obj_id)` call
  (`dying_obj_snapshot`) instead of two separate ones.
- `abilities.rs`'s graveyard-zone dispatch: `dying_owner` uses `state.fizzle_object(*new_grave_id)`
  (not counted by the ratchet's needle at all — `fizzle_object(` is a named helper, not a bare
  `.objects.get(`), reusing the SAME classification the two checks immediately below it
  (`nontoken_only`, `filter`) already use for the SAME id.

`bare_lookup_ratchet::bare_lookup_counts_are_pinned` passes unmoved at its pre-batch ceilings
(108 / 75 / 34) — verified by executing the gate, not by arithmetic.

## Version gates — numbers taken from the gates' own output, not predicted

Both fail, as the plan's §5 wire-impact prediction said they must (`TargetFilter` gains a field,
`TriggerCondition::WheneverCreatureDies` gains a field, `EffectTarget` gains a variant — all three
inside the `Command`/`GameEvent` closure or the `GameState` closure). **Not bumped in this run** —
left for the coordinator per the dispatch brief.

- `hash_schema::declaration_fingerprint_is_pinned`: current pinned `d73666c9...` (36/36); LIVE
  digest `e8ca5110...`.
- `protocol_schema::protocol_schema_fingerprint_is_pinned`: current pinned `bdd02df0...`
  (PROTOCOL 36); LIVE digest `686d14e4...`.

Full failure text (verbatim, from the final `--workspace --no-fail-fast` run):

```
---- hash_schema::declaration_fingerprint_is_pinned stdout ----
The serialized shape of the GameState type closure (130 types) has changed.
  left:  "d73666c948e7b3fe09934d87896585e5a514f559d373076197143461e1312818"
  right: "e8ca51103996c3094a0c6c1e1107511e2f98719e15cf0fe15f1726cc730f4ca5"

---- protocol_schema::protocol_schema_fingerprint_is_pinned stdout ----
The serialized shape of the Command/GameEvent type closure (97 types) has changed.
Currently PROTOCOL_VERSION 36.
  left:  "bdd02df0eb7f84f0a957852a7e0944affa7e0f7c8de1348990ad53d1c5e73f62"
  right: "686d14e4e028f7d1148958ae58fcc17a9f359ed46c4835a864199895077f5f04"
```

## Tests: `crates/engine/tests/primitives/pb_dx28_owner_axis.rs` (12 tests)

Registered in `crates/engine/tests/primitives/main.rs`. Four sections:

- **A** (casting-path `TargetFilter.owner`, `t1`-`t4`): synthetic `{T}: Destroy target permanent
  [owner scope].` activated ability, `Command::ActivateAbility`, `Ok`/`Err(InvalidTarget)`.
- **B** (battlefield `DeathTriggerFilter.owner_you`/`owner_opponent`, `t5`-`t8`): raw
  `TriggeredAbilityDef` attached via `ObjectSpec::with_triggered_ability` (bypasses card-def
  lowering, exercises the dispatch site directly) except `t8`, which goes through the REAL
  card-def lowering (`build_face_ability_vectors`) to exercise `Some(TargetOwner::Any)` as an
  actual enum value, not as two hand-set bools.
- **C** (graveyard-zone dispatch, `t9`-`t10`): the REAL `nether_traitor()` def from `all_cards()`,
  `check_and_apply_sbas` + `state.pending_triggers()` — mirrors
  `pb_dx24_trigger_zone_and_index_spaces.rs`'s fixture pattern exactly, varying owner vs controller
  of the dying creature independently.
- **D** (`EffectTarget::DamagedPlayer`, `t11`-`t12`): `t11` is a synthetic self-referential
  "whenever this deals combat damage to a player, deal N to that player" creature in a 4-player
  game, attacking p3 directly (turn order p1,p2,p3,p4) — isolates the primitive from Equip
  mechanics. `t12` is the card-integration test: the REAL `sword_of_war_and_peace()` def, equipped
  (Equip {2}, ability index derived from `all_cards()`, never hard-coded) and attacking p3, checking
  both `EffectTarget::DamagedPlayer` (damage) and `PlayerTarget::DamagedPlayer` (the `ZoneTarget::
  Hand.owner` / `EffectAmount::CardCount.player` reads) resolve correctly, plus the unrelated
  `PlayerTarget::Controller` life-gain amount for completeness. Required
  `register_static_continuous_effects` after direct battlefield placement (the same
  `GameStateBuilder::object()` gotcha `cards1_equip_target_repair.rs` already documents for
  Skullclamp) so the Sword's +2/+2 static actually applies.

## Revert matrix — every probe, executed red then restored

All six reverts were applied to LIVE source, run against the full `pb_dx28_owner_axis` module,
observed red, then restored verbatim and re-confirmed green with a final full-workspace run
(4,615 passed / 2 failed [the two version gates, unmoved] / 5 ignored — identical before and
after the revert exercise).

| # | Revert | File:site | Probes covered | Observed |
|---|---|---|---|---|
| R1 | `TargetOwner::You`/`Opponent` compare `obj.controller` instead of `obj.owner` (the pre-batch `TargetController` approximation, reproduced verbatim) | `casting.rs`, `TargetPermanentWithFilter` arm | t1, t2, t3 | **RED**: t1 `Err` (expected `Ok`), t2 `Ok` (expected `Err`), t3 both halves flipped. t4 unaffected (control). |
| R2 | `TargetOwner::Any => false` | `casting.rs`, same arm | t4 | **RED**: both accepts became `Err`. t1-t3 unaffected. |
| R3 | `df.owner_you`/`df.owner_opponent` compare `dying_controller`/`obj.controller` instead of `dying_owner`/`obj.owner` | `abilities.rs`, battlefield `AnyCreatureDies` loop | t5, t6, t7 | **RED**: t5 0 triggers (expected 1), t6 1 trigger (expected 0), t7 both halves flipped. t8-t12 unaffected. |
| R4 | Lowering maps `Some(TargetOwner::Any)` onto `owner_you: true` ("Any" mistaken for "You") | `testing/replay_harness.rs`, `build_face_triggered_abilities`'s `WheneverCreatureDies` arm | t8 | **RED**: the P2-owned-fodder half (0 triggers, expected 1). The P1-owned half stayed green — expected: from the watcher's own perspective P1-owned coincidentally still satisfies the incorrectly-widened "You" match, which is itself informative (proves the revert is doing exactly what it claims, not something coarser). Others unaffected. |
| R5 | `dying_owner` reads pre-death `*death_controller` instead of the post-death graveyard object's real `.owner` | `abilities.rs`, `collect_graveyard_carddef_triggers`'s `WheneverCreatureDies` arm | t9, t10 | **RED**: t9 0 triggers (expected 1), t10 1 trigger (expected 0). t1-t8, t11-t12 unaffected. |
| R6 | `EffectTarget::DamagedPlayer` resolves to `ctx.controller` unconditionally instead of `ctx.damaged_player` | `effects/mod.rs`, `resolve_effect_target_list_indexed` | t11, t12 | **RED**: t11 p3 life 38 (expected 35); t12 p3 life 36 (expected 35). All other tests unaffected. |

Zero UNDISCRIMINATED rows — every probe was individually reddened by a revert of the exact code it
exercises.

## What the plan got right, and one correction

- The plan's §2/§3 design (two-bool `DeathTriggerFilter` decomposition rather than a stored
  `TargetOwner`, module-dependency-direction rationale; `EffectTarget::DamagedPlayer` resolving to
  EMPTY rather than falling back to controller; the exact four casting.rs arms; the two
  `trigger_target_candidates`/`trigger_battlefield_target_matches` picker sites) all held exactly as
  specified — nothing needed re-deriving.
- One correction: the plan's §2.1 enforcement-site list did not separately call out that
  `rules::abilities`'s "auto-target picker" is actually TWO functions
  (`trigger_battlefield_target_matches` the predicate, `trigger_target_candidates` the enumerator),
  and only named the graveyard family generically ("`trigger_target_candidates`'s two graveyard
  arms"). Both were verified present and both needed the fix; this is a documentation gap in the
  plan, not a missed site — the plan's own "verify, do not assume" instruction is what caught it.
- The `pb_dx42a_continuous_condition_roster.rs` failure was NOT anticipated by the plan at all
  (§5's wire-impact section discusses `TargetFilter` gaining a field only from the hashing/protocol
  angle, not from this OTHER hand-maintained structural fingerprint). Filed here as a durable
  lesson for future `TargetFilter` field additions, not as a plan defect — the plan had no way to
  know this second, independent gate existed without reading it, and it now says so explicitly in
  its own doc comment.

## Definition-of-done checklist

- `cargo build --workspace`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean. `tools/check-defs-fmt.sh`: clean (1,803 defs).
- Full `cargo test --workspace --no-fail-fast`: **4,615 passed / 2 failed / 5 ignored** — the 2
  failures are `hash_schema::declaration_fingerprint_is_pinned` and
  `protocol_schema::protocol_schema_fingerprint_is_pinned`, both EXPECTED (wire change, left for
  the coordinator's bump) and both gate-executed, numbers quoted above verbatim.
- No pre-existing test went red for any reason other than the two version gates. The
  `pb_dx42a_continuous_condition_roster::t6` failure surfaced DURING this run (reproduced at HEAD
  before this run's own tests existed) and was fixed as part of this run's engine work, not
  weakened or glossed.
- Tree left DIRTY, uncommitted, per instructions.

---

# PB-DX28 — coordinator addendum: the 18th member, migrated rather than deferred

Part 2's run found an 18th `OOS-DX4-6` member — `Connive // Concoct`'s Concoct half — with its own
R4 inverse axis, *after* the roster had been pinned at 17, and **deferred** it on scope-discipline
grounds (`memory/conventions.md`'s implement-phase default-to-defer). That reasoning is sound in
general and wrong here. Reversed by the coordinator, with the argument stated so the next reader
can disagree with it on the merits:

* `connive.rs` declares **no `completeness` field**, so it derives `Complete` and is deck-legal.
  It is not a latent member.
* Its shape is *identical* to `takenuma_abandoned_mire`'s, which this batch already migrated:
  a printed "return **a creature card from your graveyard**" with no "target", authored as a real
  `TargetCardInYourGraveyard`. The migration is the same four lines.
* This batch **closes** `OOS-DX4-6`. Closing a class while a known deck-legal `Complete` member
  keeps the defective shape closes it on a false premise — which is the precise failure this
  queue has recorded against itself repeatedly (`OOS-DX3-1`'s "cheap standing sweep" closure is
  the batch immediately before this one).
* AC 6448 says registry member lists are **floors**. A floor written by this batch's own plan is
  still a floor. Deferring here would make "18 ≥ 14" a fact the batch discovered and then declined
  to act on.

## What the reversal cost, and what it caught

Migrating it reddened **two** rows of part 2's own roster gate, and the second is the interesting
one:

| row | why it reddened | resolution |
|---|---|---|
| `r1_chosen_object_roster_is_pinned` | a hardcoded `CHOSEN_OBJECT_MEMBERS.len() == 17` floor | re-pinned to 18, with the reason for the difference written into the constant's doc rather than just the number changed |
| `r3_migration_is_complete_not_additive` | **the walk could not SEE the member.** R3 enumerated `["Triggered", "Spell", "Activated"]` nodes. A split card's half is an `AbilityDefinition::Fuse`, which is none of the three, so R3 reported "R1 found a ChosenObject but no node carries it" | variant list widened to include `Fuse`, `LoyaltyAbility` and `SagaChapter`, with the mechanism written into the function's doc |

R3's gap is `seed-rerank-2026-08-02.md` §2.7's hazard — a short variant list dropping a nesting
site in silence — **occurring inside a gate written by the batch that cites that hazard**, and it
was invisible for as long as no member used the missing variant. Had the 18th member been deferred,
R3 would have shipped with a hole nothing could have found, because the only def that exercises it
would have been the one deliberately left out of scope.

## Probe

`t11_concoct_surveils_then_returns_a_chosen_creature_with_no_declared_target`
(`crates/engine/tests/primitives/pb_dx28_untargeted_choice.rs`). The card had **zero** behavioural
coverage before it — `crates/engine/tests/mechanics_a_d/connive.rs` exercises the connive KEYWORD
(CR 702.163), not this card — so the migration would otherwise have rested on a roster pin alone,
which asserts a def's shape and nothing about whether that shape executes. That is the failure
PB-DX27's `/review` recorded against three headline defs, in this same queue, one batch ago.

The probe puts TWO eligible creature cards plus one ineligible Instant in the controller's
graveyard, so it exercises the real question rather than the determined short-circuit, and asserts
the answer space is exactly the two creatures.

**Revert row (executed):** restore `connive.rs` to its pre-migration shape → `t11` **RED**, then
restored and re-confirmed green (11/11 in the probe file, 5/5 in the roster gate). Recorded
precisely: the revert trips `t11`'s in-finder `targets.is_empty()` assertion, so this row
discriminates on the **declaration**; the behavioural half (the `ChooseObject` question and its
candidate set) is what the green run proves. Under the reverted shape `MoveZone` would read
`DeclaredTarget { index: 0 }` against an empty `ctx.targets` and move nothing, so the probe would
fail either way — but it fails at the earlier assertion, and saying so is cheaper than implying a
behavioural discrimination this row does not perform.

## AC 6451 — allowlist retirement, and the proof it did not go blind

Both `completeness_deviation_scan.rs` `ALLOWLIST` entries covering these classes are **removed**:
`sword_of_truth_and_justice` (forced dead by part 2's own migration — the gate said so itself) and
`staff_of_compleation` + `nether_traitor` (removed here). The three defs now carry **no exemption
of any kind**, which is a stronger outcome than rewriting their reasons.

Removing them first turned the scan RED, naming `staff_of_compleation`, `nether_traitor` and
`connive` — because part 1's and this addendum's own rewritten in-def comments used the words the
scan hunts for (`approximat`, `partial`, `should`, `deferred`). That is the scan working: those
words are how a def *claims an open gap*, and these defs no longer have one. The comments were
reworded to state the same CR facts in closed-defect language, and the scan went green with no
allowlist row and no `RECORDED_BASELINE` row. The alternative — a PB-DX27-style "record of a
refuted claim" entry — was available and was **not** taken: an entry is an exemption, and these
defs need none.

**The scan is still live on both classes, proven by execution rather than argued** (a removal
argued is how an allowlist rots — this file's own `path_to_exile` precedent):

| planted instance | def | scan result |
|---|---|---|
| owner class — "printed 'you own' … best available **approximation**, since TargetFilter has no owner axis" | `staff_of_compleation` | **RED**, `Offenders: ["staff_of_compleation"]` |
| untargeted-choice class — "the printed choice is untargeted, but it is **modeled as** a real target requirement — an accepted **deviation**" | `azorius_chancery` | **RED**, `Offenders: ["azorius_chancery"]` |

Both plants reverted; `completeness_deviation_scan` back to **12/12 green**.

---

# AC 6448 — the census, and its full disposition table

Method, both axes, neither one a grep of the registry's member names:

* **Axis A — slot arithmetic, `all_cards()` walk.** For every def, sum every declared
  `TargetRequirement` slot across `Activated` / `Triggered` / `Spell` / `LoyaltyAbility` /
  `SagaChapter` / `ClassLevel` and both extra faces, and compare against the number of `"target"`
  occurrences in the combined oracle text (front + back + adventure). `slots > words` is the
  candidate signal. 50 rows, 34 of them `Complete`.
* **Axis B — inverse, printed text first.** Scan every def's printed oracle text for ownership
  needles (`you own`, `opponent owns`, `owned by`, `into your graveyard`, `its owner controls`, …)
  with **no reference to what the def declares**. 63 rows.

Both are **floors**, and one of them was proven to be one *within this batch*: axis A missed
`Connive // Concoct`, which the shipped R4 gate then found. The recall bound is filed as
`OOS-DX28-8`.

## Untargeted-choice class (`OOS-DX4-6`) — 18 `Complete` members + 1 player-axis sibling

| # | def | printed clause (no "target" — CR 115.10) | was | now |
|---|---|---|---|---|
| 1-10 | the ten Karoos | "return **a land you control** to its owner's hand" | `TargetPermanentWithFilter(Land+You)` | `ChosenObject{Battlefield, Land+You, 1, false}` |
| 11 | `cloud_of_faeries` | "untap **up to two lands**" | `UpToN{2, TargetLand}` + 2 `UntapPermanent` | one `UntapPermanent{ChosenObject{…,2,true}}` |
| 12 | `frantic_search` | "untap **up to three lands**" | `UpToN{3}` + 3 `UntapPermanent` | one `UntapPermanent{ChosenObject{…,3,true}}` |
| 13 | `rewind` | "Counter target spell. Untap **up to four lands**." | slot 0 `TargetSpell` + slot 1 `UpToN{4}` | slot 0 KEPT (a real printed target); slot 1 migrated |
| 14 | `shrieking_drake` | "return **a creature you control**" | `TargetCreatureWithFilter(You)` | `ChosenObject{Battlefield, Creature+You, 1, false}` |
| 15 | `whitemane_lion` | same | same | same |
| 16 | `sword_of_truth_and_justice` | "put a +1/+1 counter on **a creature you control**" | `TargetCreatureWithFilter(You)` | `ChosenObject` on `AddCounter.target` |
| 17 | `takenuma_abandoned_mire` | "return **a creature or planeswalker card from your graveyard**" | `TargetCardInYourGraveyard` | `ChosenObject{YourGraveyard, …}` |
| 18 | `connive` (Concoct half) | "return **a creature card from your graveyard** to the battlefield" | `TargetCardInYourGraveyard` | `ChosenObject{YourGraveyard, …}` |
| — | `sword_of_war_and_peace` | "deals damage to **that player**" | `TargetRequirement::TargetPlayer` | `EffectTarget::DamagedPlayer`, `targets: vec![]` |

**Refuted** (axis A surfaced them; adjudication cleared each): every `Equip`-carrying Equipment —
CR 702.6a's granted ability *does* say "target creature you control" and PB-DX26 authored it
deliberately, so the printed line is only the cost; `curtains_call` ("Destroy **two** target
creatures"), `huddle_up` ("**Two** target players"), `victimize` ("Choose **two** target creature
cards") — one `"target"` word, two real slots; and the trigger halves of
`sword_of_fire_and_ice` / `sword_of_light_and_shadow` / `sword_of_sinew_and_steel`, which genuinely
print "target" / "any target".

## Owner class (`OOS-DX4-1`) — 2 `Complete` members, and three refutations that matter more

| def | printed clause | was | disposition |
|---|---|---|---|
| `staff_of_compleation` | "Destroy target permanent **you own**" (CR 108.3) | `TargetController::You` (CR 109.4) | REPAIRED — `owner: You`, and `controller: You` **removed** |
| `nether_traitor` | "put **into your graveyard**" (CR 404.3) | `WheneverCreatureDies{controller: Some(You)}` | REPAIRED — `controller: None, owner: Some(You)` |

Members that are **not** `Complete`, listed so the class is counted rather than the cards:
`athreos_god_of_passage` (`partial`, and its own note already names this gap), `hellkite_courser`
(`partial`), `maskwood_nexus` (`partial`), `mishra_claimed_by_gix` (`partial`),
`leyline_of_the_void` (`known_wrong`).

**REFUTED, and each refutation is load-bearing:**

* **The six mutate defs** — `brokkos_apex_of_forever`, `gemrazer`, `sea_dasher_octopus`,
  `necropanther` (all `Complete`), plus `mindleecher` and `nethroi_apex_of_death` — print "put it
  over or under target non-Human creature **you own**" (CR 702.140a). Not members: `casting.rs`
  checks `target_obj.owner != player` **open-coded**, outside `TargetFilter`. This is the largest
  group the census found and it is entirely clean.
* **`fecundity` is not a member, and `nether_traitor`'s own note said it was.** Its printed clause
  is "**that creature's controller** may draw a card" — a controller gap
  (`PlayerTarget::ControllerOf(TriggeringCreature)`), exactly as `fecundity.rs`'s own marker note
  already said. The citation was wrong and is corrected in place; `fecundity.rs` itself is
  untouched, because the error was in the def that cited it.
* **`hanweir_battlements`** ("If you both **own and control** this land and …") — `Effect::Meld`
  checks `obj.owner == controller && obj.controller == controller`. Correct today.

## Seeds filed

`OOS-DX28-1..8` in `docs/audits/decision-point-audit.md` (the registry was grepped for the ID
prefix before filing — dispatch hygiene 5 — and returned 0). `OOS-DX4-6` and `OOS-DX4-1` both
CLOSED there, each row carrying the corrections to its own original claims.

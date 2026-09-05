# PB-DX57 — execution notes

*v4 queue rank 21, task 3 of 5 of the SECOND user-approved chain.*
*Branch `feat/pb-dx57-the-gate-widening-cluster-six-gates-that-report-succ`; merge base `8604207e`.*

---

## §0. Stage 0 — measured before any test line changed

### §0.1 The inherited test pin REPRODUCES

`cargo test --workspace --no-fail-fast` on this branch **before any edit**:

```
72 result-producing targets
5,316 passed / 0 failed / 5 ignored
```

PB-DX56's published close pin is **5,316 / 0 / 5 on 72 targets**. It reproduces **exactly**, with no
correction owed — the **ninth** consecutive batch in which an inherited pin reproduces
(`OOS-DX51-5`'s non-reproducing-pin failure has not recurred since PB-DX51).

### §0.2 Wire prediction — PER HALF, in writing, before any line of this batch existed

**Predicted: `HASH_SCHEMA_VERSION` 85 UNMOVED and `PROTOCOL_SCHEMA_VERSION` 44 UNMOVED — ZERO bumps
for the whole PB.** Predicted separately for each half, with the mechanism rather than the
conclusion:

**Half A** (`OOS-DX28-1` fingerprint class + `OOS-DX28-5` shared target-declaring enumeration +
`OOS-ADJ-2` verify-only). Every line lands in `crates/engine/tests/`. Both gates hash a *type
closure* rooted at `Command` / `GameEvent` / `Effect` / `Characteristics` (PROTOCOL) and at
`GameState`'s hashed declarations (HASH). An integration test is not in either closure and cannot be:
it declares no production type, adds no variant and adds no field. A test that READS a production
declaration out of source at runtime (the `declared(..)` idiom) reads it as **text**, so it does not
even link the type into a new position.

**Half B** (`OOS-DX26-3` order roster + `OOS-DX21-7` vacuous-probe rewrites + `OOS-DX28-6` mechanism
notes). Same argument for the two roster/probe halves. The one place this half touches
non-test source is **card-def comment repairs**, and a comment is not a declaration: it is discarded
by the lexer, contributes no bytes to any `HashInto` impl, and appears in no serialized payload. A
compiled `Completeness::partial("...")` note IS a string literal rather than a comment — so the
stop-condition below covers it explicitly.

**Stop-condition, stated in advance**: if the `OOS-DX28-6` census turns up a stale note whose repair
cannot be expressed without editing a `Completeness` *marker kind* (not its prose), or whose LIVE
defect can only be fixed by an engine edit, the batch does **not** take it — it FILES it, per the
task's own 0-engine-lines constraint — and this prediction stands. If any prediction here is
refuted, the refutation is reported in this file rather than the prediction quietly edited.

**Counterfactual, to be verified by execution rather than asserted**: "unmoved" only means something
beside what would have moved it. Both gates' `CLOSURE_MUST_NOT_CONTAIN` lists will be planted with a
type each half would have had to touch had it needed a field, and each plant executed.

### §0.3 Coverage prediction

**0 flips**, predicted with the reason before regeneration: this batch authors no card text and
changes no `Completeness` marker KIND. Card-def edits are **comment-only**; the authoring report's
buckets are marker-driven, so a comment cannot move one. `git diff` over the `Completeness::` marker
lines will be checked directly rather than inferred from an unchanged total (PB-DX26's lesson that a
stable COUNT is not a stable SET).

### §0.4 Benches and `npm run build`

Both **N/A**, with the reason rather than by omission: this is a test-only batch (0 engine lines is
an acceptance criterion), so no benched path can move, and `tools/` is untouched.

---

*(Sections filled in as the batch proceeds.)*

---

## §1. `OOS-ADJ-2` — closed by PB-DX42b, VERIFIED HERE BY EXECUTION, not redone

The v4 memo's row 21 carried `OOS-ADJ-2` as a member of this batch and then struck it:
**✅ TAKEN AS A RIDER BY PB-DX42b (`scutemob-233`, 2026-09-05), both halves, both revert-proven.**
The registry row agrees. This batch therefore does **not** redo the widening; the acceptance
criterion asks only that the closure be *verified by execution*, and it is — **on both halves,
because the widened `t7` carries two independent assertions and only one of them is what the
criterion names.**

**Verification 1 — the DECLARATION half (the assertion the criterion names).** A **ninth**
layer-querying variant was planted into `Condition::required_characteristic_layer`'s fixed
`=> Some(EffectLayer::TypeChange)` arm in `crates/card-types/src/cards/card_definition.rs`
(`Condition::HaveTwoOrMoreOpponents`, moved out of the `None` arm so the plant compiles without an
unreachable-pattern warning — under `-D warnings` an unreachable pattern is a build failure, and a
build failure is a NON-verdict rather than a red, `OOS-DX39-8`). Result:

```
t7_non_target_filter_layer_querying_variants_absent_from_population ... FAILED
  left:  {..., "OpponentControlsMoreLandsThanYou"}                       (8, the pinned list)
  right: {..., "HaveTwoOrMoreOpponents", "OpponentControlsMoreLandsThanYou"}  (9, the declaration)
```

RED, **by NAME**, on the set-equality against the arm parsed out of source. The other nine tests in
the file stayed green, which is the right shape: this half is about the gate's own coverage, not
about the corpus.

**Verification 2 — the POPULATION half, which is what the gate is FOR.** `rancor.rs`'s two
`ContinuousEffectDef`s were given `condition: Some(Condition::ControlLegendaryCreature)` — a corpus
def joining the population through one of the eight non-`TargetFilter` variants. Result: `t7` RED
again, on its *other* assertion, naming the variant.

**The green rows under verification 2 are the finding, not the omission.**
`t6_two_axes_agree_on_the_conditioned_population` and `t5_layer_querying_set_is_pinned` both stayed
**GREEN** with two new layer-querying members in the corpus. That is exactly the blindness
`OOS-ADJ-2` was filed about and `t7` was widened to close: axis 2 recognises a member by finding a
`TargetFilter` node in its subtree, and these eight variants **carry no `TargetFilter`**, so the
structural axis cannot see them and the two axes agree *vacuously*. **`t7` is the only thing in the
tree that reddens**, which is the strongest possible statement that the widening is load-bearing.

**A methodological note recorded because it nearly published a false green.** The first attempt at
verification 2 used a needle occurring **twice** in `rancor.rs`; the patch helper refused to apply
it (fail-closed, printing `PATCH AMBIGUOUS`) — and `cargo test` then ran anyway and printed **10
passed**. That run is a **NON-VERDICT**, not a pass. It is `OOS-DX39-8`'s shape one direction over,
and PB-DX53's *"an earlier R1 patch that never applied printed seven greens that were the UNMODIFIED
tree"* verbatim. Every plant in this batch is therefore checked for APPLICATION (re-read the bytes)
before any test result is read.

Both files restored and verified byte-identical by `cmp`.

**Recorded disposition**: `OOS-ADJ-2` — **closed by PB-DX42b, verified by execution here (both
halves), not redone.**

---

## §2. `OOS-DX28-5` — the shared target-declaring enumeration, and the seed's own instance had ALREADY REGROWN

### §2.1 The ground truth, derived rather than typed

`pub enum AbilityDefinition` carries **68** variants, of which **8** declare a `targets` field:

```
Activated, Aftermath, Fuse, LoyaltyAbility, SagaChapter, Spell, Splice, Triggered
```

`pb_dx28_chosen_object_roster::ability_target_shapes` walked **six**. It omitted **`Aftermath`**
and **`Splice`**.

### §2.2 The finding: the list went stale WITHIN ONE BATCH of being widened

This is the seed measured rather than predicted, and the two omissions have different causes:

* **`Aftermath.targets`** has existed since the variant did. It was in no draft of the list — a
  straight omission.
* **`Splice.targets` DID NOT EXIST when PB-DX28 wrote the six.** **PB-DX18** (`OOS-M11-5`,
  `scutemob-225`) added it for CR 702.47a — *"copy this card's text box onto that spell"*, so a
  spliced spell requires the spliced card's targets and CR 601.2c makes those a real announcement.
  Nothing in the tree reddened.

PB-DX28's own R3 doc said the two extra entries it added were *"included for completeness — the
point of listing them is that the day one does, this row sees it."* The row did not see it, because
a seventh target-declaring variant arrived through the ordinary act of authoring a rule and a
hand-written list has no way to notice.

**And both are LIVE, not latent.** Axis 2 (below) observes all **8** variants carrying `targets`
keys in the real corpus — so `Aftermath` and `Splice` nodes with declared targets exist today and
R3's walk could not see them.

### §2.3 What shipped, and why a derivation rather than a longer list

`crates/engine/tests/core/pb_dx57_ability_target_variants.rs` derives the set from `pub enum
AbilityDefinition`'s own declaration. **A pinned literal checked against the declaration is the
right repair when the list encodes a JUDGEMENT** — that is `t7`'s case one file over, where *"which
variants query a characteristic layer"* is a semantic claim about eight names. **Here the list
encodes no judgement**: *"declares a `targets` field"* is a syntactic property of the declaration,
so a literal adds a second place to be wrong and buys nothing.

A derivation can still break silently, so it is guarded on **two independent axes and a floor**,
never on itself:

* **`d1`** — the parse reached the whole enum (≥ 60 variants), every parsed name is a plausible Rust
  identifier, and the `targets:` classifier produced a **non-degenerate split** (a classifier that
  says *everything* declares targets passes a bare `>= 8` floor, and `d2`'s floor alone could not
  tell the two apart).
* **`d2`** — a raise-only floor of 8, PLUS a second-method re-check that each derived name really
  carries the field in its own declaration window (the floor is blind to over-reporting by
  construction).
* **`d3`** — **axis 2**: serde-walk `all_cards()` and observe which variant nodes actually carry a
  `"targets"` key. Axis 2 knows nothing about axis 1's regex, so the two can disagree; `d3` asserts
  axis 2 ⊆ axis 1 and **PRINTS** the residual rather than asserting it empty (an unused variant is
  not a defect, and asserting it empty would redden on the ordinary act of adding a variant before
  authoring a card for it). Measured: axis 1 = 8, axis 2 = **8**, residual **0**.
* **`d4`** — the historical six are a strict subset of the declared set, phrased as a subset
  relation rather than as *"the missing two are Aftermath and Splice"*, so the record survives a
  later variant arriving and does not re-create a hand-maintained list inside the module written to
  remove one.

A parser note worth carrying: the first draft split the enum body on `,` **before** stripping line
comments, and the enum's doc comments are English prose. It yielded **204** "variants" including
`the`, `it` and `CR` against a true 68 — `OOS-DX32-6`'s *a text scan cannot tell code from a
comment* arriving inside a parser rather than inside a gate. `d1`'s identifier-shape assertion is
what catches it and is kept for that reason.

### §2.4 The consumer is load-bearing, PROVEN BY EXECUTION — and the blindness is UNDER-checking

`ability_target_shapes` now calls `target_declaring_ability_variants()`.

The naive plant does not demonstrate anything, and saying so is the point: R3 opens with
`assert!(!chosen_object_nodes.is_empty(), "R1 found a ChosenObject but no node carries it")`, so a
def whose *only* `ChosenObject` sits in an omitted variant reddens on that message — which is
exactly how PB-DX28 discovered `Fuse`. **The residual blindness is a def with BOTH a visible
target-declaring node AND an invisible one**: the `!is_empty()` check is satisfied by the visible
one and the additive-migration violation on the invisible one is never examined.

Planted exactly that. `frantic_search` (already a `CHOSEN_OBJECT_MEMBERS` entry) gained a second
ability — an `AbilityDefinition::Splice` carrying **both** `targets: vec![TargetRequirement::
TargetCreature]` **and** a `ChosenObject` in its effect, i.e. the double-counting additive migration
R3 exists to refuse.

| tree state | `r3` | note |
|---|---|---|
| historical six-item list, plant in | **GREEN** | and `r4` RED — by a DIFFERENT mechanism (oracle-slot subtraction), so the first attempt over-stated R3's blindness |
| historical six, plant in, **one `"target"` added to the oracle text** | **GREEN — and so is EVERY OTHER ROW IN THE FILE (6/6 pass)** | R4 neutralised, so R3's blindness stands alone |
| derived enumeration, same plant | **RED**, `left: 1, right: 0`, naming Frantic Search | |

The middle row is the one worth reading: **the whole roster reports success while the defect it
exists to refuse is sitting in the corpus.** That the first attempt tripped `r4` and had to be
narrowed is recorded rather than discarded — *"all rows RED" is a true sentence the wrong assertion
can produce* (PB-DX48), and its converse is that a red row can be evidence about a different gate.

Every file restored, `cmp` byte-identical.

---

## §3. `OOS-DX28-6` — the sibling class, measured, and the answer is a ZERO

### §3.1 The census

Stage 0 derived the population mechanically rather than by reading defs. The corpus splits into a
PROSE surface (**15,394** line comments + **663** `Completeness` notes; **0** block comments) and a
CODE surface (comments **and string literals** stripped — a `Completeness::partial("… Effect::Foo …")`
note is a string LITERAL, and `OOS-DX53-2` records a census that read 5 where the truth was 4 for
exactly that reason).

**593 of the 663 notes — 89.4% — span multiple lines**, and a naive extractor that does not join
`\`-continuations silently truncates every one of them (`OOS-DX35`). The extractor was reconciled
against a raw grep and **was wrong once**: 662 against 663, because `darksteel_colossus.rs:63` puts a
six-line comment BETWEEN `Completeness::known_wrong(` and its string. Fixed; 663/663.

The vocabulary was derived **from the prose** by a closure that does not iterate (PB-DX8's lesson:
*iterated bootstrapping DRIFTS, and a vocabulary learned from the DSL's own ground truth is
self-blinding on the target*), and the CamelCase axis was filtered against a **declared** dictionary
parsed from `card-types` — 92 enums / 954 variants — never against corpus usage.

### §3.2 The result

**34 hits across 33 files. 22 are real mechanism claims and every one is CONFIRMED TRUE. Zero
stale. ZERO LIVE DEFECTS.** `sword_of_war_and_peace` appears to have been the only instance of the
exact shape on the axes a derivation can see, and PB-DX28's repair of it is present at `:55-67` /
`:74-95` and was re-verified.

This is stated as a **result**, not as a clean bill of health. The recall bound is measured, not
estimated: **33.7% of resolution-verb sentences name no identifier at all** and are structurally
invisible — the seed's own case written in plain English would not be seen.

### §3.3 The known-positive replay, and what it says about the obvious first draft

The seed's verbatim pre-repair sentence fires against reconstructed pre-PB-DX28 code and is silent
at HEAD. **The load-bearing detail: that sentence contains no `::` at all**, so a qualified-token
(`Enum::Variant`) derivation — the obvious first draft, and the closest to the task brief's own
suggested vocabulary — **would have missed the seed's own instance.** The BARE-identifier axis is
what carries the known positive.

**That prediction was reproduced by execution rather than taken on trust.** The first draft of
`m4_the_gate_fires_on_the_seeds_own_pre_repair_sentence` failed on its first run against exactly
that gate, before any of this was written down.

### §3.4 The one repair, and why it survived three previous sweeps

`well_of_lost_dreams.rs` — `inert`, comment-only, no marker moved, 0 coverage flips. Three claims
repaired:

* Clause **(c)** of its `Completeness::inert` note said *"'you may pay' has no interactive
  expression"* — **falsified by PB-DX45 three days earlier** (`EffectChoiceQuestion::PayOptionalCost`,
  `stubs.rs:1030`). A **seventh** "pay when able" residue that PB-DX45's own `/review` sweep did not
  reach. Narrowed rather than deleted: the channel exists, and the surviving blocker is that
  `PayOptionalCost` carries a FIXED `Cost`, not a cap.
* Two `// TODO`s said `TriggerCondition::WhenYouGainLife` does not exist — which the def's **own
  compiled note already calls false**. They survived PB-DX27's blocker sweep, PB-DX8's
  `completeness_deviation_scan` and this census's own qualified-token pass for one reason: **they
  MISSPELL the identifier** (`WhenYouGainLife` for `WheneverYouGainLife`), so it is not in any
  declared dictionary and no needle set keyed on identifiers can see it.

**The generalisation is worth more than the instance and is filed**: *a needle set keyed on
identifiers is blind to a claim that gets the identifier's spelling wrong, and getting the spelling
wrong is correlated with being wrong about the claim.*

### §3.5 The ratchet, and why its polarity is inverted

`crates/engine/tests/core/pb_dx57_mechanism_note_ratchet.rs`, six tests.

The census's first proposal was to key on resolution VERBS and subtract the non-assertions. **Its
own §6.2 then defeated that design**: *"filter (d) is the load-bearing weakness and no lengthening
of the marker list repairs it"* — a stale claim can carry a negation word in a neighbouring clause
while asserting the defect in its own, and one of the measured false-positive shapes carries **no
negation word at all**. So the shipped gate enumerates a small set of **ASSERTIVE FRAMES**:
**enumerating what may fire fails CLOSED; enumerating what may not fails OPEN.** That is PB-DX53's
`/review` repair (*"enumerating what may mutate a container is unbounded… enumerating the 8 READ
methods is short and fails closed"*) applied to a prose classifier.

Measured at HEAD: **56** assertive-frame sentences across 1,803 defs, **23** naming a declared
identifier, **4** live offenders — and *the gap between 56 and 23 IS the gate's recall bound*, which
`m3` prints rather than hides.

All four offenders are adjudicated in `RECORDED_OFFENDERS` with the verdict quoted, in **three
distinct false-positive shapes**:

| def | shape | verdict |
|---|---|---|
| `chandra_flamecaller` (×2 identifiers) | **rejected-alternative rationale** — *"a naive `DiscardCards{HandSize}+DrawCards{HandSize}` reads 0 after the hand is already emptied"* explains why the def uses `Effect::WheelHand` INSTEAD | TRUE |
| `elenda_the_dusk_rose` | **the comment names the RUNTIME identifier the DECLARED one lowers to** — `TriggerCondition::WhenDies` lowers to `trigger_on: TriggerEvent::SelfDies` at `replay_harness.rs:2677`, verified by reading the site | TRUE |
| `fecundity` | **prospective** — *"Residual AFTER REWIRE: … `ControllerOf` reads the graveyard object"* is a claim about a rewire not yet made, inside a note that says so | TRUE |

They are **recorded, not filtered**, because Census C §6.3 is right that *a gate which makes an
author delete a true sentence to go green has stopped measuring* — PB-DX54's `r3` finding stated
inside this file's own subject matter.

**A prediction of this batch's own was refuted by the corpus and is recorded rather than deleted.**
The authorability filter (an enum no def's code ever names is engine plumbing) was expected to
remove `TriggerEvent` and so to remove the `elenda` hit. It does not: `basri_ket.rs:78` and
`ajani_sleeper_agent.rs:69` construct `TriggerEvent` variants in real emblem trigger specs, so the
enum genuinely IS authorable. The hit is recorded on its merits instead.

### §3.6 A tautology this batch wrote and deleted

`m6`'s first draft closed with `assert!(AMBIGUOUS_BARE.is_empty() || !AMBIGUOUS_BARE.is_empty(), ..)`.
That is `A || !A` — **`t9_fingerprints_match_their_structs`'s ORIGINAL defect
(`intersection.is_none() || len != len`) reproduced by hand, inside the batch whose subject is
assertions that cannot fail.** Deleted rather than reworded, with the note left at the site.

---

## §4. `OOS-DX21-7` — the sweep, the repairs, and the site the sweep missed

### §4.1 The census

| step | | count |
|---|---|---|
| 1 | files mentioning `process_command` | **387** |
| 2 | raw call sites | **3,115** |
| 3 | of those, TEST files | **369** |
| 4 | of those, files with an error expectation — *the honest search space* | **191** |
| 5 | test FUNCTIONS whose `Err` provably came from a `process_command` result | **534** |
| 6 | of those, with ≥1 `assert` after the `Err` — **the read set, all read in full** | **216** |
| 7 | shape A (`X.clone()` argument) sites / files | 46 / 19 |
| 8 | shape B (value snapshot bound before the call) sites / files | 22 / 8 |
| 9 | **VACUOUS after reading** | **17 sites / 15 fns / 9 files** |

**Zero AMBIGUOUS** — every candidate resolved by reading binding provenance in its own function.

**No shared helper wraps the shape.** Only four non-test helpers call `process_command` and handle
an error, and none makes an absence-of-mutation claim — one of them,
`scripts/harness_equivalence.rs:853`, uses the mechanism *knowingly and correctly*. So the defect was
17 hand-written instances rather than one helper multiplied: **there was nothing to fix once and
everything to fix once each.**

**The concentration is the tell.** `pb_dp7` / `pb_dp8` / `pb_dp9` are three blocking-decision batches
written to one template and held **9 of the 17**; `pb_dp8` had **exactly one** of its tests repaired
(`:359`), with a doc comment that is the best statement of `OOS-DX21-7` in the repository — while its
own siblings and the whole of `pb_dp7`/`pb_dp9` were left. *The lesson was learned once and not
carried across the file, let alone the batch.*

**Three vacuous tests DOCUMENT the mechanism in their own comments and assert anyway.**
`primitive_sr34_composite_mana_costs.rs:267` says *"`process_command` takes `GameState` by value, so
a rejected command's mutations (if any had happened) are unobservable"* — and then asserts on the
clone 27 lines later.

### §4.2 The load-bearing constraint on the repair

`process_command` dispatches **44** handlers over 45 `Command` variants. **34 are `pub`**; **10 are
private `fn` in `rules/engine.rs`** (`handle_pass_priority`, `handle_activate_loyalty_ability`,
`handle_concede`, `handle_transform`, `handle_turn_face_up`, `handle_level_up_class`,
`handle_activate_craft`, `handle_pay_echo`, `handle_pay_recover`, `handle_pay_cumulative_upkeep`),
`blocking_decision` is `pub(crate)`, and `LocalGame` is not an engine dev-dependency. **For those ten
commands the direct-handler rewrite does not compile from an integration test**, which is a fact
about the repair and not an excuse: it is why one row is class C and why the structural fix is filed.

### §4.3 The three repair classes, all 17 sites, 19 guard-removal experiments, 19 RED

* **Class A — admission-gate rows** (5). `process_command` returns `Err(BlockedByPendingDecision)`
  **before the `match` on `command` runs at all**, so no handler executes and the property is true by
  construction — *there is no handler to call*. Repair is DELETION of the tautology plus the positive
  control that discriminates: the ALLOWED command from the named player IS admitted and DOES move the
  hash. Guard removed: the admitting clause for that command.
* **Class B — handler-validation rows** (11). Direct-handler `&mut state` with an ACCEPTED control, so
  the probe cannot be satisfied by an engine that mutates nothing ever. `pb_dp7:257` was a **one-token
  fix**: `&mut state.clone()` → `&mut state`.
* **Class C — the unbuildable row** (1). `loyalty_target_validation.rs`'s handler is private.
  Rewritten to an observable, **with the residual in the test's own doc**, and the handler was NOT
  made `pub`, because that is an engine line.

**The load-bearing detail of the proofs**: R1-R6 and R16's guard removals were designed so the error
assertion the OLD test already had stays GREEN and only the repaired receiver-reading assertion
fails. That is the direct demonstration that each repair ADDED discrimination rather than restating
it. **Zero `#[test]` added, renamed or deleted** — 157 == 157 by byte-exact name-set difference over
the nine modules.

### §4.4 THE 18TH SITE, found by this batch's own gate, and the sweep called its own count a FLOOR

`rules/commander.rs::test_companion_rejected_when_not_in_command_zone`. Its comment reads:

> *"The action failed atomically: the state the caller keeps is unchanged — no mana spent, action not
> marked used."*

…and then asserts on the ORIGINAL `state` after `process_command(state.clone(), ..)`. Repaired to
`rules::commander::handle_bring_companion(&mut state, ..)` (`pub`, `commander.rs:1028`).

**Proven by a COMPLEMENTARY PAIR rather than by one red row.** MR-M9-13's guard is *"locate the
companion in the command zone BEFORE paying any cost"*. Move that lookup BELOW the mana payment:

| form | verdict |
|---|---|
| repaired (`&mut state`) | **RED** — `assertion left == right failed: mana must not be deducted on the state the REJECTED handler actually held` |
| original (`process_command(state.clone(), ..)`) | **GREEN** |

The defect is in the tree and the old test says nothing about it. Both files restored `cmp`-identical.

### §4.5 The gate, and what it does NOT claim

`crates/engine/tests/core/pb_dx57_vacuous_rejection_gate.rs`. Keyed **per ASSERTION** on binding
provenance, because per-FILE is green on the three files holding 9 of the 17 and per-FUNCTION is
green on `pb_dp7`'s `&mut state.clone()` — PB-DX50's `r3` instantiated twice on real data. Comments
and string literals stripped; `use ... as` aliases resolved; the `&mut` exemption requires a **bare
identifier**, never an expression.

**It ships labelled a RATCHET, not a proof.** The sweep's adversarial section found seven bypasses,
of which **two are not closable at the source level**: a helper wrapper (the test then contains no
`process_command`, no `.clone()` and no `Err` token — and it is exactly what a later batch does on
noticing 17 sites repeat) and a macro. Saying so in the module doc rather than letting the name imply
more is `OOS-DX49-6`'s own lesson, applied to this batch's own gate.

**The bypass-proof repair is stated and deliberately NOT taken**: split `process_command` into a
`pub process_command_mut(&mut GameState, Command)` — its body is already `let mut state = state;`
followed by `&mut`-dispatch — plus the by-value wrapper. That makes the property falsifiable for all
45 command variants **including the 10 whose handlers are private and for which no such test can be
written today**. It is an engine change and this is a 0-engine-lines batch, so it is FILED.

### §4.6 The gate over-fired twice and both were caught by ADJUDICATING hits, not by reading a count

* It attributed **any later `Err` in the function** to the flagged call. Three false positives, all
  the same real shape: a `.clone()` handed to a call that SUCCEEDS (`.unwrap()`), with a separate,
  correctly by-value rejection elsewhere in the function.
* It had **no rebinding awareness**. `let (state, _) = process_command(state, good).unwrap();` after
  a rejection means every later assertion reads the ACCEPTED path — sound. *Dozens of the 216
  functions the sweep read in full do exactly this*, which is why that sweep had to READ them.

Both narrowed; both now carry a synthetic control in `v3`. **The first draft's shape-A detector was
also dead**: it anchored the statement at the CALL, so `let r = process_command(..)` never showed its
own `let` and the binding-then-Err-check arm never ran — caught by `v3`'s synthetic case, which is
the whole reason the self-test uses synthetic input rather than the corpus.

### §4.7 Two disclosed positive controls, RECORDED rather than rewritten or filtered

PB-DX21's two second-declaration probes read a `state` that a PREVIOUS **accepted**
`process_command` produced, and each assertion's own message says *"positive control"*. That is the
second sound idiom. They are recorded with the adjudication because the distinction the gate would
need — *"is this comparing to a pre-call snapshot, or to a value the accepted path established"* — is
a claim about INTENT, and the gate strips comments and so cannot read the disclosure that makes them
sound. Encoding the judgement once, with the reason, is honest; teaching the scanner to guess would
make it fail OPEN on the real shape.

---

## §5. The wire — predicted NONE per half, executed UNMOVED, and the counterfactual is informative in BOTH directions

**Executed against the final tree**: `hash_schema` **36/36**, `protocol_schema` **17/17**,
`history_is_append_only` and `frozen_prefix_is_pinned` green on **both** gates.
`HASH_SCHEMA_VERSION` **85**, `PROTOCOL_VERSION` **44** — both **UNMOVED**, exactly as predicted per
half in `fb8e53c0` before any test line existed. `git diff` over `state/hash.rs` and
`rules/protocol.rs` is **EMPTY**, so no sentinel re-pin, no survivor scan, no history row and no
frozen-prefix re-pin were owed; the two append-only gates were executed anyway, green, as the
evidence that none was owed rather than as a claim.

**The counterfactual, verified by execution**, because "unmoved" only means something beside what
would have moved it. Each type planted in **both** gates' `CLOSURE_MUST_NOT_CONTAIN`, one at a time,
both gates executed, then restored (`git diff` over both files empty):

| planted type | HASH | PROTOCOL | what it says |
|---|---|---|---|
| `TargetFilter` | **FAILS** | **FAILS** | on BOTH wires |
| `Condition` | **FAILS** | **FAILS** | on BOTH wires |
| `AbilityDefinition` | passes | passes | on **NEITHER** |

The first two rows are the counterfactual proper: `TargetFilter` and `Condition` are the very types
the `OOS-DX28-1` fingerprint work is *about*, and had any repair here needed to STORE the pinned
information on one of them rather than derive it at run time, it would have cost **+1 HASH and +1
PROTOCOL** plus a ~49-file sentinel re-pin. Deriving it costs nothing. That measurement is the reason
for the design, not a preference.

**The third row is the finding.** `AbilityDefinition` is in neither closure, and the reason is
structural: it is reachable only through `CardDefinition`, which **both** lists exclude. That
reproduces PB-DX18's own recorded observation (*"`AbilityDefinition::Splice` gained a field, and
PROTOCOL still did not move, because `AbilityDefinition` is reachable only through `CardDefinition`,
which the same list excludes"*) — **and it is exactly why `OOS-DX28-5`'s hand-written list could rot
silently.** When PB-DX18 added `Splice.targets`, there was nothing anywhere in the tree that would
have said anything: not the compiler (every construction site uses `..Default::default()` and
`#[serde(default)]` covers deserialization — `OOS-DX20b-2`), not the wire gates (the type is off both
closures), and not the walk that depended on the list (a hand-written literal). *Three independent
mechanisms that each look like they would catch it, and all three are structurally blind to the same
edit.*

---

## §6. `OOS-DX28-1` — the class, enumerated and pinned

### §6.1 The census: 35 members

From ~150 classified candidates and ~110 individually named rejected near-misses (needle lists for
source-text gates, file/function allowlists, expected-value pins, ratchet ceilings, card-name lists
— none of which mirrors a declaration). **22 UNPINNED, 13 ALREADY-PINNED.** Printed by
`core::pb_dx57_fingerprint_census::c3` rather than left in a memo, because a memo cannot notice when
one of its rows stops being true; `c1` re-checks every row's needle still exists (`OOS-DX52-1`), and
`c2` ratchets the derived slice-const population (229 across 52 files) so a new candidate cannot
join in silence.

**The `const` axis is a CEILING; the INLINE axis is a FLOOR.** Four members are an inline
`for x in [..]` with no keyword to anchor a grep on — including the seed's own instance.

**Two methodology findings, both this class happening to the instrument that measures it:**

* a `static` grep returned **0** while `pub static ROWS` existed, because it anchored on a bare
  `static` — `OOS-DX20b-5` reproduced inside a census whose subject is exactly that, and caught by
  re-running with a second spelling rather than by reading;
* **a `const` whose TYPE is a struct slice hides its string literals from every `&[&str]` grep**
  (`&[Row]`, `&[UnreadField]`, `&[NamingSiteRow]`, `&[ReachRow]`) — four members are that shape and
  all four were found only by reading the file.

### §6.2 Three members were ALREADY STALE, and a fourth was found by the repairs

1. **`pb_dx28`'s inline `AbilityDefinition` list** — 6 of 8 (`OOS-DX28-5`, §2).
2. **`SIMPLE_TARGET_VARIANTS` + `FILTER_TARGET_VARIANTS` + the inline `"UpToN"`** covered **21 of
   22** `TargetRequirement` variants. `TargetSpellOrAbility` was in none of them — and because R4
   is a **SUBTRACTION** (`slots > words`), the gap does not merely under-report: **a real
   over-declaration on the same def CANCELS against it**. Live on one deck-legal `Complete` def
   (`deflecting_swat`), measured after the repair at 1 slot / 1 word, so the cancellation happened
   not to be load-bearing on today's corpus — stated that way rather than rounded either up or down.
3. **`LAND_TYPE_CONFERRING_VARIANTS`** covered 3 of the 4 `LayerModification` variants carrying a
   `SubType`, missing `SetCreatureTypes` while listing `SetCardTypes`, which cannot name a subtype
   at all. Corpus exposure measured at **ZERO**, so the gap was in the census's REACH, not yet in
   its answer.
4. **Found by the repairs, not the census: `NEW_TRIGGER_EVENTS` is a gate's NEEDLE SET** and was
   short by `EquippedCreatureDealsCombatDamageToPlayer` — so that gate was not scanning the
   dispatcher's eighth event for a second dispatcher at all.

### §6.3 One shared parser, and its own cross-check caught it twice

`core::pb_dx57_declared_source` replaces what would have been a **nineteenth** hand-written
declaration parser; the tree already held five, each with its own anchoring rules and its own bugs.
It was wrong three times and each was caught by execution, never by review:

* **3 of 8** on its first run — five `AbilityDefinition` variants carry `#[serde(default)]` above
  the `targets` field and the naive extractor read `#` as the field's first character. Caught by
  `p4`'s by-value cross-check against the sibling derivation, which reaches the same answer by a
  different code path.
* **LINE-based field extraction** — `pub basic: bool, pub nonbasic: bool,` on one line contributed
  only its first field, and the resulting failure message said *"the declaration no longer has
  `nonbasic`"*, **the opposite of the truth**. Found by another agent's plant against a COPY.
  Comma-chunked; `p6` pins it; verified on a real declaration (8 vs 7).
* **RAW IDENTIFIERS** — `pub r#type: bool` parsed to the empty string and the field was dropped
  **in silence**. `p1`–`p6` were all green under the plant. Fixed, made **fail-CLOSED** (a `pub`
  chunk that yields no name is now a panic, because a dropped field is invisible to every consumer
  at once), and `p7` pins it. Both cross-target copies fixed the same way.

### §6.4 Two members deliberately NOT closed

Both live in `#[cfg(test)]` modules under `tools/`, which **this batch's own 0-engine-lines
criterion requires to be an EMPTY diff**. The two acceptance criteria are in tension and the
explicit diff constraint wins; the gap is filed as `OOS-DX57-3` with both exact repairs, and
`c4_the_two_unfixed_members_are_named_and_still_present` records it **inside the census test**, so a
reader of the census sees it rather than only a reader of a memo.

---

## §7. The adversarial pass — TEN defeats across two rounds, all re-keyed and re-executed RED

Every gate this batch wrote or re-keyed was handed to a **second agent, briefed with the gate's doc
sentence and explicitly not its implementation**, to attack by execution. This is the queue's
recurring lesson made procedural: *a revert-proof written by the same author from the same mental
model exercises the inputs that author already thought of* (`OOS-DX54-6`).

**Round 1 defeated 5 of 5. Round 2 defeated 5 more, three of them completely (whole test target
green).** Not one gate survived its first adversary.

| # | gate | the defeat | the re-key |
|---|---|---|---|
| 1 | face-down maker derivation | keyed on the assigned `FaceDownKind`, so a third genuine site assigning an EXISTING kind **deduped away** and a floor of 2 was satisfied by a set of 2 | re-keyed onto the enclosing `Effect::` arm — **and that was STILL GREEN**, because the roster only moves when a corpus DEF uses the channel; closed with an exact-set pin |
| 2 | vacuous-rejection gate | the **snapshot-before-move** form: no `.clone()` argument, no helper, no macro — *the more natural way to write the test* | shape C added, with a synthetic control |
| 3 | mechanism-note ratchet | eight plain-English assertive verbs, sharpest being **`snapshots` — the verb in the gate's own recorded offender** | frames 11 → 19 (monotone, which is why the polarity is fail-closed) |
| 4 | shared target-declaring enumeration | **nothing enforced "every walk calls it"** — a second walk with its own six-element list left everything green | `d5`, a consumer gate |
| 5 | shared declaration parser | reported as prefix-vulnerable — **RE-ATTRIBUTED**, see below | four OTHER parsers repaired; `p5` forbids the form |
| 6 | `t12` / `unread_init_fields` / the canonical parser | a **raw identifier** (`pub r#type`) is dropped in silence; two whole test targets stayed green | `r#`-aware and **fail-closed**, in all three copies; `p7` |
| 7 | `decision_gate::every_effect_variant_is_classified` | **laundering**: move a variant between two lists carrying OPPOSITE claims and the partition is invariant; whole `core` target green | the candidate set pinned BY NAME in the dangerous direction only |
| 8 | `pb_dx39::r3b` | same shape — misfiling `ExileSelf` keeps the partition valid; whole `core` target green | source-moving side pinned BY NAME |
| 9 | `pb_dx45::r2b` | the derivation is **syntactic** (`=> false`), so a semantically-false body passes | **not closable at source level** — stated as a residual, with the measurement that seven behavioural probes DO redden |
| 10 | `cards2::r7b` | two consistent lists can shrink **together** | stated residual; `r7`'s aggregate floor is what holds |

### §7.1 The re-attribution, recorded because accepting it would have been easier

Round 1 reported the shared parser prefix-vulnerable. **It is not** — its needle carries the trailing
` {` — and re-executing the decoy plant against it returns the real 17 fields. What the adversary
actually found is better: **four OTHER declaration lookups in the tree use a bare prefix needle**,
so which declaration they read depends on declaration ORDER. Proven by execution: a decoy carrying
the same field names left the **whole `core` target green with every field-set pin checking the
decoy**. Not contrived — `PendingTriggerTargets` already exists in that file, and `pub enum Effect`
is a prefix of `EffectTarget`, `EffectAmount`, `EffectFilter`, `EffectLayer` and `EffectDuration`.
**And the load-bearing part is about the DEFENCE**: this module's doc leans on *"panics on an empty
parse, so a caller can never accidentally compare against nothing"* — and a prefix-shadowed parse is
**non-empty and wrong**, the one failure mode that promise cannot see. All four repaired; `p5`
forbids the form; `OOS-DX57-5`.

### §7.2 Two of this batch's own gates over-fired, and were narrowed by ADJUDICATING hits

The same discipline pointed inward. The rejection gate attributed **any** later `Err` in a function
to the flagged call (three false positives, all a `.clone()` handed to a call that SUCCEEDS) and had
no **rebinding** awareness (an assertion after `let (state, _) = process_command(state, good)` reads
the ACCEPTED path — *dozens* of the 216 swept functions do exactly that). `p5`'s first draft reported
**five** offenders, all `.expect(..)` MESSAGES and **zero** real. Every one was found by opening a
flagged line rather than reading the count.

---

## §8. The `/review` — 16 findings, all 16 taken, and FOUR of them were this batch's own gates

The reviewer had a shell, reproduced every headline number independently (5,363 / 0 / 6 across 72
result lines; an empty engine diff; HASH 85 / PROTOCOL 44; 63.2% coverage; every touched registry row
splitting at exactly 4 cells) — and then **defeated four more of this batch's gates by execution**,
which is the class the batch exists to close, arriving for the third time.

### §8.1 The four gate defeats

* **`p6`/`p7` gated a COPY.** Both pasted `declared_struct_fields`' body inline instead of calling
  it, so **reverting the function to the line-based form this batch's own commit `7811ad36` fixed
  left `p6` GREEN and the whole `core` target at 830 passed.** That is PB-DX50's `r3` *inverted* — a
  gate on a copy of a predicate says nothing about the predicate — inside the batch that closes that
  class. Body extracted as `struct_fields_from_body`; both plants now redden.
* **`v1` had FIVE natural bypasses**, one of which contradicted the module doc's own stated key: the
  `&mut` exemption was computed over the **whole function body**, so one unrelated
  `warm_up(&mut state)` exempted every assertion in a test — while the doc four screens up says
  *"gating per FUNCTION is green … So the key is per ASSERTION."* And the repair idiom this batch
  shipped IS `handle_x(&mut state, ..)`, so a **partly-repaired function was unpoliced by
  construction**. Also: the hoisted clone (`let cloned = state.clone(); process_command(cloned, ..)`
  — no helper, no macro, and the more natural spelling), `.to_owned()` through the blanket impl, and
  a hand-typed `STATE_READS` missing `pending_triggers` (41 uses), `objects_in_zone` (112),
  `pending_effect_choice` (74) and `object` (37). All closed, each with a synthetic control.
* **The mechanism ratchet discharged a stale claim on a SUBSTRING**, in the commit that ships
  `has_token` for exactly this. The dictionary is full of prefix pairs, and a def declaring
  `AddManaAnyColor` discharged a false claim about `AddMana`. Fixed — **and the fix immediately
  surfaced a sixth real row on its first run**, `olivias_wrath`, whose own `ModifyBothDynamic`
  contains `ModifyBoth` and had been discharging its (true) note against itself.
* **`r2`'s pin coverage was frozen and unenforced**: the loop iterates `PINNED_ORDER` and never
  walks the live set, and **262 of the 530 pinned rows pin an EMPTY activated list, which is a
  prefix of anything**. A def ABSENT from the pin could gain two activated abilities and have them
  SWAPPED with all 30 roster tests green. Today's population is complete, so this was decay rather
  than a live miss — but the pin is only regenerated when it fails, so covered fraction would shrink
  monotonically as cards are authored, which is the exact activity `OOS-DX26-3` polices. Closed by a
  COVERAGE assertion, RED under the replanted defeat and naming the def.

### §8.2 Three published figures did not reproduce, and one was a bound the next batch is told to reuse

* §3.5's *"56 assertive-frame sentences … 23 naming a declared identifier … 4 live offenders — and
  the gap between 56 and 23 IS the gate's recall bound"* was taken **before** the adversarial
  widening and never re-taken. At HEAD `m3` prints **91 / 35**, and the recorded set holds **6**.
  Corrected. **PB-DX28's re-take MEDIUM, committed by this batch inside a paragraph whose whole
  purpose is to hand a measured bound to the next reader.**
* `SLICE_CONST_CEILING` was set from a **stage-0** measurement (229/52) and the final tree reads
  **234/53** — this batch added five of its own — so the ceiling left 6 headroom under a doc saying
  *"a new one cannot join in silence"*. Re-measured, ceiling 240 → 236, and the FLOOR tightened
  200 → 228 on the same `OOS-DX47` argument (*a ratchet's slack IS its blind spot*).
* `m2` asserted `>= 50 && >= 800` under a message reading *"(floor 60 / 700)"*, so a maintainer
  correcting the code to match the message would have **reddened the enum axis and loosened the
  variant axis**. The floors are now interpolated into their own message: a number written twice is
  a number that can disagree with itself.

### §8.3 Two scope findings that are one batch old in this repository

`v1` walked **one crate** while the sweep it cites is a workspace figure — **55 files containing
`process_command` were outside it**, and the reviewer planted the seed verbatim in
`crates/simulator/tests` and watched every gate stay green. `p5` had the same one-crate scope while
its heading says *"the test tree"*, and `crates/simulator/tests/pb_dx55_activation_auto_tap.rs:818`
already does `.find("pub enum Command {")`, compliant only by its author's care. **This is PB-DX48's
`SITE_SRCS` defeat and PB-DX49's workspace-walk repair, one batch old and not carried across** — and
the module doc's *"three lessons this gate is built to survive"* did not list it. Both now walk
`crates/*/{tests,src}` plus `tools/`: 468 files → **2,492**. Both defeats re-executed RED.

Latent rather than live when found, and said so: the reviewer ran the scanner verbatim over the
other roots and measured **0 sites in each**.

### §8.4 The allowlist finding, and the fix that had to be fixed twice

Three allowlists said their reasons were checked; all three checked only `reason.len()`. Proven for
`v4`: deleting the words *"positive control"* from the assertion message its reason QUOTES left the
gate green. The repair re-checks the quoted evidence **in the named test's body** — and its own
first draft was file-scoped, which stayed green because the file holds two such labels, and its
second draft searched the COMMENT-STRIPPED body, which fails on the clean tree because the label
lives in a string literal. **Both were found by re-executing the reviewer's defeat against the fix
rather than assuming it landed**, which is the same discipline that caught the face-down re-key in
round 1.

### §8.5 Everything else taken

`RECORDED_VACUOUS`'s doc said *"empty is the state"* while holding two rows (a reader auditing
*"were any allowlisted?"* was told no); the mechanism ratchet's **fifth** recall bound
(sentence-scoping: a claim split across two sentences is invisible) was undisclosed; `m4`, the
known-positive replay, re-implemented the pipeline instead of calling it, so repairing `offenders()`
would have left the seed replay testing the old rule; `r4`'s failure message claimed a
wrong-ability check it does not perform (it checks RANGE); and the card-def note cited
`stubs.rs:1030` without its crate.

### §8.6 What the adversarial record does and does not cover — stated because the criterion asks

Criterion 7409 asks for at least three bypass attempts per gate. **Two independent adversarial
rounds plus the `/review` attacked eleven gates and defeated ten; the remaining ~20 gates this batch
wrote or re-keyed were NOT individually attacked**, and that is a gap rather than a pass. The ones
attacked were chosen by the adversaries themselves as most likely to fall, which is the right
selection rule and not the same as coverage. Named so the next reader knows which claims rest on
execution and which rest on care: **attacked and defeated** — `r5d`, `v1`, `m1`, `d1`–`d4`, `p1`–`p4`,
`t12`, `unread_init_fields`, `decision_gate::every_effect_variant_is_classified`, `r3b`, `r2b`,
`r7b`, `p6`, `p7`, `r2`; **attacked and survived** — `r2d`, `pb_eng2::stack_push_variants…`,
`declared_enum_variants` (survived 2 attempts each, reported as *"survived N attempts"* and never as
"cannot be bypassed"); **not attacked** — `c1`–`c4`, `v2`–`v4`, `m2`, `m3`, `m5`, `m6`, `p5`, `r1`,
`r3`, `r4`, and the re-keyed pins in `pb_dx20b_enchant_line_roster`, `pb_dx36_deals_damage_roster`,
`pb_dx43_land_type_roster`, `pb_dx48_announcement_site_roster`, `pb_dx49_saga_blanking_roster`,
`pb_rs1_roster_sweep`. Each of those has an executed plant proving it discriminates; none has an
adversary's attempt to evade it.

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

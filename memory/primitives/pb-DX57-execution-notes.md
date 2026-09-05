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

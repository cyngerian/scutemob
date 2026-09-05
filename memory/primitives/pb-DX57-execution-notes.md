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

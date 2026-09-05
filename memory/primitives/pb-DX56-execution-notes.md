# PB-DX56 execution notes (`scutemob-235`)

> v4 queue rank 20, task 2 of 5 of the SECOND user-approved chain.
> Seeds: **OOS-FB1-1** *(prerequisite)* → **OOS-DX32-1** + **OOS-DX22-8**.

---

## §0 Stage 0 — measured and predicted BEFORE any production line

### 0.1 Pre-edit full-workspace baseline (measured, not remembered)

`cargo test --workspace --no-fail-fast` to a file, on this branch **before any edit**:

```
result lines (targets): 72
passed/failed/ignored: 5287 0 5
extracted test lines: 5292 distinct: 5292
duplicate names: 0
```

**Reproduces PB-DX55's close pin exactly** — 5,287 / 0 / 5 on **72** result-producing
targets — the **seventh** consecutive batch in which an inherited pin reproduces with no
correction owed. The extraction regex is deliberately **not** end-anchored
(`OOS-DX42b-6`), so an `#[ignore = "reason"]` test whose line reads `... ignored, <reason>`
is still extracted; the duplicate-name scan the byte-exact method is structurally blind to
(`OOS-DX35-8`) is **EMPTY** (5,292 lines / 5,292 distinct).

### 0.4 Wire prediction — PER HALF, in writing, before any production line

**Prediction: HASH 85 and PROTOCOL 44 BOTH UNMOVED — zero bumps for the whole PB.**

Stated per half with the reason, not as a preference:

* **Half A — `OOS-FB1-1`, the diagnosability tooling.** Everything it touches lives in
  `crates/simulator` and `crates/simulator/src/bin/fuzzer.rs`:
  `InvariantViolation` gains an `evidence` field; `LocalGame`/`GameResult` gain a bounded
  command-history ring; `CrashReport.command_history` stops being `Vec::new()`; the
  in-flight tombstone is a filesystem write in the binary. **Neither gate walks
  `crates/simulator` at all** — `hash_schema.rs` and `protocol_schema.rs` live in
  `crates/engine/tests/core` and close over the engine's
  `Command` / `GameEvent` / `Effect` / `Characteristics` roots. A simulator-side struct is
  not reachable from any of the four. Predicted movement: **none, on either gate.**
* **Half B — the `OOS-DX22-8` engine fix.** The defect is a **dangling `ObjectId` left in
  an already-hashed field**: `GameObject.attached_to` has existed and been hashed since
  long before this batch. A fix that changes **when** that field is cleared adds no type,
  no variant and no field, and `state/hash.rs` hashes the field's VALUE, not the moment it
  was written. Predicted movement: **none, on either gate.**
  **Stop-condition, stated in advance**: if the fix turns out to require a new field or a
  new type, this batch STOPS and posts `COORDINATOR` before re-predicting — it does not
  quietly take a bump the prediction did not cover.
* **Half C — the `OOS-DX32-1` disposition.** Both branches are wire-neutral for the same
  two reasons above: a transient split is a `crates/simulator` bucket change, and an
  engine-side repair of `turn.priority_holder` writes an already-hashed field.
  Predicted movement: **none, on either gate.**

**Counterfactual, to be VERIFIED BY EXECUTION at stage 0 rather than asserted** — "unmoved"
only means something beside what would have moved it. Recorded when run.

### 0.5 Coverage prediction

**0 flips, coverage UNMOVED at 1,140/1,803 = 63.2%.** Reason: this batch authors no card
text and repairs no card-def blocker — it changes the fuzzer's instrumentation and (at
most) one engine attachment/priority path. No `Completeness` marker can move, because no
def's expressible-ness changes. To be confirmed by regeneration rather than by the
empty-diff shortcut.

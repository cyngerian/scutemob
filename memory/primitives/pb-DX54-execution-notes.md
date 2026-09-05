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

# PB-DX44 — the casts you cannot make

> Task `scutemob-215`. v4 queue rank 2 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 2,
> §2.4). Closes `OOS-DX29-3` / `-9` / `-12` / `-14`.
>
> This file is written **incrementally, in execution order**. §1 (wire predictions) was written
> and committed BEFORE any source line changed — that ordering is the point of the section, and
> the commit that adds it touches nothing under `crates/` or `tools/`.

---

## 0. Baseline, measured on this branch BEFORE any edit

`cargo test --workspace --no-fail-fast` to a file, at `3fb932ef` (branch fork point):

| figure | value |
|---|---|
| passing | **4,753** |
| failing | **0** |
| ignored | **5** |
| result-producing targets | **50** |
| distinct test NAMEs captured | **4,757** (0 duplicates) |

Reproduces PB-DX43's close pin (4,753 / 0 / 5, 50 targets) exactly. The 4,757-name set is
retained for the end-of-batch set-diff, so the delta is itemised by NAME rather than by
arithmetic (PB-DX7's rule).

---

## 1. Wire predictions — WRITTEN BEFORE ANY CODE CHANGE

The v4 memo publishes four wire cells for this row, each with a confidence. AC 6730 requires the
prediction to exist before the code and makes a mismatch a stop-and-re-scope event rather than a
number to be adjusted after the fact. PB-DX27 is the precedent that makes this worth doing
honestly: its brief predicted "expected wire impact NONE" and the gate refuted it.

| half | memo prediction | **this batch's prediction** | reasoning, stated before the edit |
|---|---|---|---|
| **Spree mode costs** (`OOS-DX29-14`) | none, HIGH | **PROTOCOL none / HASH none** | The whole change is `crates/simulator/src/legal_actions.rs::effective_cast_cost_with_additional` plus its callers. The chosen modes must *reach* that function, which is a signature change on a **simulator** function — `LegalAction`, `ActionParams` and `AdditionalCostPlan` are all simulator types, outside the `Command`/`GameEvent`/`Effect`/`Characteristics` closure `protocol_schema` walks. `ModeSelection.mode_costs` is already hashed (`hash.rs:6894`) and already read by `casting.rs`; nothing about it changes. |
| **Fuse targets** (`OOS-DX29-12`) | none, HIGH | **PROTOCOL none / HASH none** | `casting.rs` concatenates the existing `AbilityDefinition::Fuse { targets }` into the requirement list it already derives. `StackObject.target_requirements` is an existing hashed `Vec<TargetRequirement>` (PB-DX25c) whose **content** grows, not its type. `legal_actions::fused_right_half_declares_targets` is a simulator predicate being deleted. No type, no variant, no field. |
| **Half selector** (`OOS-DX29-9`) | **PROTOCOL**, HIGH | **PROTOCOL BUMP / HASH BUMP** | Two moves, and the memo names only the first. (a) `CastSpellData` gains a half selector field → it is `Command::CastSpell`'s payload, squarely inside the PROTOCOL closure. (b) Resolution must know which half to execute, and the precedent set for every other cast-mode flag (`was_overloaded`, `was_bargained`, `was_cleaved`, `cast_with_aftermath`, `was_cast_as_adventure`) is a field on `StackObject` — which is hashed. **So a HASH bump is predicted too, and the memo's cell is short by that half.** Recorded here as a prediction, not as a discovery after the gate. |
| **Pitch** (`OOS-DX29-3`) | none, HIGH | **PROTOCOL none / HASH none** | `CastSpellData.alt_cost` and the pitch payment path (`casting.rs:4209-4260`) both already exist and are already on the wire; `AltCostKind::Pitch` is an existing variant. What is missing is entirely above the engine: `params.rs` hard-codes `alt_cost: None`, `ActionParams` has no channel to carry the choice, and the offer layer emits no pitch alternative. `ActionParams` is a simulator type; the play-server request DTO is a play-server type. |

**Aggregate prediction: exactly ONE PROTOCOL bump and ONE HASH bump for the whole PB**, both
owned by the half-selector half. Every other half is wire-neutral.

**Stop condition, stated in advance**: if `protocol_schema` reports a bump the half-selector does
not explain, or reports **no** bump at all, the batch stops and re-scopes rather than re-writing
this table. Same for `hash_schema`.

### 1.1 Why the half selector is a field and not an `AdditionalCost`

`OOS-DX29-9`'s own row says the fix is "a half-selector on the cast action, not another
additional cost", and the CR agrees: CR 702.102a and CR 709.4 make each half of a split card a
spell with its own characteristics that you choose *at announcement* (CR 601.2a), which is not
what CR 118 additional costs are. Modelling it as an `AdditionalCost` would also put it in the
same bag as `AdditionalCost::Fuse`, and the two must be mutually exclusive — a "fused right half"
is not a thing.

### 1.2 The target-index hazard, named before it is hit

`turn.rs`'s right half declares `EffectTarget::DeclaredTarget { index: 1 }` — a **globally
offset** index, correct for a fused cast where the left half's one target occupies index 0. A
right-half-**only** cast announces one target, which lands at index 0, and the effect would read
index 1 and find nothing. This is a real consequence of the def-authoring convention
`resolution.rs:338-345` documents, and any half-selector that ignores it ships a spell that
resolves at nothing — the silent-wrong-game-state failure, not a refusal. Handling is designed in
§3 and gated by a roster assertion over every `AbilityDefinition::Fuse` carrier.

# PB-DX42b — wire prediction, written BEFORE any production line changed

**Stage 0. Nothing in `crates/*/src` has been edited at the time this file is committed**
(`git diff --numstat main..HEAD -- crates/` is empty except this file's sibling task list, which
lives in `memory/`).

## The prediction

| gate | at merge base | predicted after | confidence |
|---|---|---|---|
| `HASH_SCHEMA_VERSION` | **85** | **85 — UNMOVED** | high |
| `PROTOCOL_VERSION` | **44** | **44 — UNMOVED** | high |
| hash closure type count | **132** | **132 — UNMOVED** | high |
| protocol closure type count | **98** | **98 — UNMOVED** | high |

Both closure counts are **MEASURED at the merge base**, not transcribed from PB-DX54: each gate's
`MIN_CLOSURE_TYPES` was temporarily raised to `9999` and the figure read out of the gate's own
panic text (`protocol closure is only 98 types`, `GameState serde closure is only 132 types`),
then both files restored byte-exactly (`git diff --stat` empty).

## The reason, stated rather than asserted

Everything this batch adds is **call-stack state or a computed function**, and neither is a
serialized declaration:

1. `CharacteristicEvalContext` — a `BTreeSet<EffectId>` plus a layer bound, constructed at the top
   of a `calculate_characteristics` walk and dropped when it returns. It is not a field of
   `GameState`, not a field of any hashed type, not `Serialize`, and not reachable from
   `Command` / `GameEvent` / `Effect` / `Characteristics`. It replaces a `thread_local!` depth
   counter that was equally invisible to both gates (PROTOCOL 33 / HASH 70 were gate-executed
   unmoved when `scutemob-184` introduced it).
2. `TargetFilter::required_characteristic_layer(&self) -> EffectLayer` — a **method in an `impl`
   block**, computed per filter INSTANCE. The declaration scan digests the normalized declaration
   text of `pub enum` / `struct` / `type` items; an `impl` method is none of those.
3. Splitting `is_effect_active` into `is_effect_duration_active` + `is_effect_condition_satisfied`,
   and adding `calculate_characteristics_through` — free functions. No type, no variant, no field.

## The counterfactual, VERIFIED BY EXECUTION at stage 0

"Unmoved" only means something beside what would have moved it. The obvious alternative design —
**store** the required layer on `TargetFilter` as an `EffectLayer` field instead of computing it —
was priced by planting each name in both gates' `CLOSURE_MUST_NOT_CONTAIN` and running them:

| planted name | `hash_schema` | `protocol_schema` |
|---|---|---|
| `TargetFilter` | **FAILS** — *"TargetFilter entered the GameState serde closure"* | **FAILS** — *"TargetFilter entered the Command/GameEvent closure"* |
| `EffectLayer` | **FAILS** — *"EffectLayer entered the GameState serde closure"* | **FAILS** — *"EffectLayer entered the Command/GameEvent closure"* |

Both the container and the field type are already in **both** closures, so a stored field would
have moved **both** fingerprints — **+1 HASH and +1 PROTOCOL**, plus a sentinel re-pin across ~49
files and two history rows. Computing it per instance costs **zero** on both wires. That is the
measurement behind design choice 2, not a preference.

A second counterfactual is stated with its ground rather than probed, because the type does not
exist yet to plant: putting `CharacteristicEvalContext` on `GameState` would move **HASH only** —
`GameState` is the hash root and is listed verbatim in `protocol_schema.rs`'s
`CLOSURE_MUST_NOT_CONTAIN` (`["GameState", "PlayerState", "StackObject", "CardDefinition"]`,
`:116-117`). That is also why it is not going there: it is call-stack state, and Architecture
Invariant 2 makes `GameState` the wrong home for "where in the engine's own execution we are".

## Stop condition

If either gate moves, or moves in a way the three items above do not explain, stop and report
rather than bumping. Both gates are to be **EXECUTED** against the final tree, never predicted
twice.

# PB-DX49 — every Saga site reads the printed def; a blanked Saga is still sacrificed

Task `scutemob-220`; v4 queue rank 7 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 7,
derivation §1g). Closes the **engine** half of corner-case audit #36 (the audit's last open GAP).

Seeds: **OOS-RR4-1** (subject), **OOS-RR4-3** (rider, doc rot).

---

## 0. Wire prediction — WRITTEN BEFORE ANY CODE CHANGED

**Prediction: PROTOCOL 39 UNMOVED / HASH 78 UNMOVED.** Stated with its reason rather than
asserted:

- `PROTOCOL_SCHEMA_FINGERPRINT` closes over the `Command` / `GameEvent` / `Effect` /
  `Characteristics` type closure. This batch adds **free functions and one engine-internal
  struct that is not reachable from any of those four roots** (it is a return value of a
  read-only query, never a field of a `Command`, an `Effect` payload or an event). No enum
  gains a variant; no struct in the closure gains a field.
- `HASH_SCHEMA_FINGERPRINT` hashes **declarations** of the hashed state types. This batch adds
  no field to `GameState`, `Object`, `CombatState` or any hashed type: the whole change is a
  *read* of `state.continuous_effects` (already hashed) and `obj.status.face_down` (already
  hashed) at five decision points.
- No history row is owed and no `FROZEN_HISTORY_PREFIX_DIGEST` re-pin is owed, on either gate.

The counterfactual is stated because §1g's row says it: **lowering `AbilityDefinition::SagaChapter`
into `Characteristics` would move BOTH fingerprints** (`Characteristics` is a PROTOCOL root and a
hashed type). The continuous-effect-scan design was mandated for exactly that reason and is what
ships. Gate-computed result recorded in §9.

## 0b. Pre-edit baseline — measured on this branch BEFORE any edit

`cargo test --workspace --no-fail-fast` to a file:
**4,900 passed / 0 failed / 5 ignored**, **57** result-producing targets, residual list empty.
This **reproduces PB-DX48's close pin exactly** (`4,900 / 0 / 5`, 57 targets).
Name set captured for the by-NAME delta (4,905 lines = 4,900 ok + 5 ignored).

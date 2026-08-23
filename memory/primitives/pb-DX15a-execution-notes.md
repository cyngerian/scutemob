# PB-DX15a — execution notes

**Task**: `scutemob-216` · v4 queue rank 3 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 3)
**Seeds**: `OOS-DP9-8` (CR 608.2e/701.22c/701.23i APNAP) + `OOS-DP9-11` (CR 400.7 same-zone renumber)
**Riders named by the memo**: `OOS-DX24-1`, `OOS-DX24-7`
**Explicitly NOT taken**: `OOS-DP9-16` (parked — unreachable by construction; there is no PB-DX15b)

---

## §0 — Wire prediction, written BEFORE any code changed

> This section was written and committed before a single non-test source line moved.
> Commit order is the evidence: this file's first commit precedes every code commit
> on the branch. The v4 memo's row 3 cell says **none (HIGH)** for both halves.

**Prediction: `PROTOCOL_SCHEMA_FINGERPRINT` UNMOVED and `HASH_SCHEMA_VERSION` UNMOVED,
for both halves.** Derivation, per half, rather than inherited from the memo:

**APNAP half (`OOS-DP9-8`).** The change reorders the *iteration* of an existing
`Vec<PlayerId>` produced by `effects::resolve_player_target_list` and its siblings. It
adds no type, no enum variant and no struct field. `PlayerTarget` is unchanged;
`EffectChoiceQuestion` / `EffectChoiceAnswer` are unchanged; no `Command` or `GameEvent`
gains or loses a shape. `rules::abilities::apnap_order` already exists and is already
called from `rules/engine.rs:1617` and `rules/abilities.rs:8374`, so nothing new becomes
reachable from a closure root. **HASH**: `hash.rs` hashes declared *shapes*; a different
ordering of the same player ids changes the runtime `public_state_hash` **value** on
affected games but not the *schema*, and `HASH_SCHEMA_VERSION` gates the schema.

**Same-zone half (`OOS-DP9-11`).** Replacing a same-zone `move_object_to_zone` /
`move_object_to_bottom_of_zone` with a `Zone::reposition_within`-style permutation
mutates `GameState.zones` (an existing field of an existing type) and *declines* to
mutate `objects` / `timestamp_counter`. `Zone` is unchanged. No new type.

**Stop condition (binding).** If either gate moves, that is a signal to **stop and
re-scope**, not to edit the pin. Both gates are executed after the implement phase and
the measured value is recorded in §7 below, taken from the gate's own output.

**What WILL move, and is budgeted rather than discovered:**
- Golden scripts whose per-step assertions read `ObjectId`s across a same-zone reorder,
  or whose event order is per-player (APNAP reorders the questions/events).
- SR-9b per-step fingerprints, for both reasons: fewer `ObjectId`s minted (the same-zone
  half) and a different event order (the APNAP half).
- Any seeded fixture whose shuffle/coin-flip outcome depends on `timestamp_counter`,
  because the same-zone half stops consuming values from it.

Every moved pin is listed **by name with its CR reason** in §6. A pin repaired by
weakening an assertion is a defect, not a repair.

---

## §1 — Census (both populations are FLOORS — dispatch hygiene 6)

*(filled in by stage 0; both populations are PRINTED by a test, not transcribed)*

## §2 — APNAP half

## §3 — Same-zone half

## §4 — Rider dispositions (`OOS-DX24-1`, `OOS-DX24-7`)

## §5 — Revert matrix

## §6 — Moved pins, by name, with the CR reason

## §7 — Gates, measured


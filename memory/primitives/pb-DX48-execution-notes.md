# PB-DX48 — Ward never fires on a triggered ability (OOS-ENG2-1 ≡ OOS-ENG2-2, + OOS-ENG2-3)

Task `scutemob-219`, branch `feat/pb-dx48-ward-never-fires-on-a-triggered-ability-cr-70221a-ta`.
v4 queue rank 6 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 6; re-verification §2.5).

---

## §0 — WIRE PREDICTION, written BEFORE any code changed

**Prediction: PROTOCOL 39 UNMOVED / HASH 78 UNMOVED.**

Committed at `git rev-parse HEAD` = `7eb0b2e0` (merge of `scutemob-218`), with **zero** lines of
`crates/`, `tools/` source changed at the moment of writing (`git status` clean).

**Reason, stated rather than asserted:**

* `GameEvent::PermanentTargeted` already exists (`rules/events.rs:767`, discriminant 69) and is
  already inside the `Command`/`GameEvent`/`Effect` wire closure — it is emitted at three sites
  today. Emitting the **same variant with the same three fields** at more sites adds no type, no
  variant and no field, so `PROTOCOL_SCHEMA_FINGERPRINT` cannot move. This is the identical
  argument PB-DX47 used for a *suppression*, run in the other direction.
* `HASH` hashes **declarations**, not event volume. `state/hash.rs:5484` already hashes
  `GameEvent::PermanentTargeted`'s three fields; nothing about this batch adds a hashed field, a
  hashed struct, or a new enum member. No `GameState` field is added.
* **The one thing that could have moved it, checked explicitly**: the CR 603.3b re-dispatch hook
  (§3) pushes `PendingTrigger`s into the existing `state.pending_triggers` field. That field is
  already hashed and its element type is unchanged, so the *declaration* is unchanged; only the
  runtime population differs, which a schema fingerprint does not see.

**Stop condition (pre-committed):** if either gate moves, STOP and read the bump off the failing
gate's own output rather than inventing one — do not edit a pin to make a prediction true.

## §0b — MOVEMENT BUDGET, written BEFORE any code changed

The ENG-2 handoff warns that this fix "will move fuzz and golden parity — budget for that", and
the v4 memo row repeats it. Recorded here in advance so an EMPTY moved-pin list has to be
*explained* rather than quietly enjoyed (PB-DX15a's lesson: a paid-and-unclaimed budget is
reported, not dropped):

* golden-script assertion paths that count events;
* SR-9b per-step stream fingerprints (`stream_fingerprint_is_pinned`);
* `UI3_SPLIT_COMBAT_SEED` and the other seeded constants in `tools/play-server`;
* fuzz violation counts.

Every movement is to be listed **by NAME with its CR reason**; no assertion is to be weakened to
absorb one.


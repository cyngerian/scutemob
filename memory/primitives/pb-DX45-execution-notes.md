# PB-DX45 — `Effect::MayPayThenEffect` is pay-when-able (CR 118.12)

**Task**: `scutemob-217` · v4 queue rank 4 (`memory/primitives/seed-rerank-2026-08-14.md` §4 row 4)
**Seeds**: `OOS-DX24-9` ≡ `OOS-DX27-5` — the same defect filed twice, five days apart, neither row
citing the other (v4 memo §1d).

---

## §0 — Predictions, written BEFORE any code changed

> This section is written at the top of the batch and is **never edited afterwards**. Everything
> below §0 may correct it; §0 itself stands as the record of what was predicted.
> Baseline commit: `a9671666`. Pre-edit constants read from source: **PROTOCOL 38**
> (`protocol.rs:412`, fingerprint `50e69006…205a27`), **HASH 77** (`hash.rs:863`).

### §0.1 Wire prediction (AC 7244)

| axis | prediction | reasoning, traced not guessed |
|---|---|---|
| **PROTOCOL** | **38 → 39**, ONE bump for the whole PB | `EffectChoiceQuestion` is reachable from `GameEvent::EffectChoiceRequired` and `EffectChoiceAnswer` from `Command::AnswerEffectChoice`; both are therefore inside the `PROTOCOL_ROOTS` closure (`protocol_schema.rs:74`). Adding a variant to either changes the serialized shape of an in-closure type, so the fingerprint moves. |
| **PROTOCOL closure type count** | **98 → 98 (UNCHANGED)** | The new variants carry only `Cost` (already in-closure via `Effect::MayPayThenEffect { cost, .. }`) and `bool`. No new type enters the closure — unlike PB-DX28, whose `ChoiceZone`/`TargetOwner` were genuinely new members and moved 96 → 98. |
| **HASH** | **77 → 78**, ONE bump for the whole PB | `AnsweredEffectChoice { question, answer }` is reachable from `GameState.effect_choice_answers` and is folded into `public_state_hash`; `hash.rs` has a `HashInto` impl for both enums with one arm per variant. A new arm changes the hash schema. |
| **`hash_schema` / `protocol_schema` gate behaviour** | both go **RED first**, then are re-pinned from **their own output** | Never predicted numerically. The fingerprints below are transcribed from the failing gates, never invented (PB-DX8's "publish the figure, do not transcribe it" rule; PB-DX28's execution notes quoted two fingerprints that had never existed). |
| **`history_is_append_only`** | green after appending ONE row to each of `PROTOCOL_HISTORY` and `HASH_SCHEMA_HISTORY` | rows appended, never edited |
| **`frozen_prefix_is_pinned`** | RED until `FROZEN_HISTORY_PREFIX_DIGEST` is re-pinned in **both** gate files | version 38 / hash 77 join the frozen prefix when 39 / 78 ship |
| **sentinels** | re-pinned **by symbol**, not by hand-copied literal | the SR-27/SR-8 procedure |

**Stop condition, stated in advance**: if either gate moves in a way this table does not explain —
or does **not** move at all — stop and re-read rather than edit a pin (v4 memo's inherited
addendum; the PB-DP2/DP3 precedent where two predicted bumps were falsified).

### §0.2 Coverage / completeness flips predicted (AC 7242)

See §3 for the policy ruling and the named flip list. Written before regeneration.

### §0.3 Population prediction (AC 7243)

The memo's **11 deck-legal `Complete` defs** is treated as a **FLOOR** (dispatch hygiene 6). §2
records the inverse-method census at HEAD.

# SR-24 — Bounding `capture_lki_snapshot` cost

<!-- last_updated: 2026-07-16 -->

> Re-audit task `scutemob-79`. Measure the LKI-snapshot capture on the mass-departure
> path, then gate it if it earns a gate. Same perf axis as the deferred LOWs
> MR-M1-18 / MR-M6-14 and the `sba_check` / `full_turn_4p` benches.

## The finding

`GameState::capture_lki_snapshot` (`crates/engine/src/state/mod.rs`) runs on **every**
battlefield departure — lands, tokens, auras included — performing a
`calculate_characteristics` layer evaluation plus a full `GameObject` clone into the
hashed `lki_objects` map. SR-13 added it with a correctness-only review, so nothing
measured it. A 4-player board wipe destroys dozens of permanents in one SBA batch, and
if the stack and pending-trigger queues are empty the whole map is discarded at the next
`handle_all_passed` (`maybe_clear_lki_objects`) — so the work is pure waste in the common
case.

## Measurement

New criterion bench `board_wipe_4p` (`crates/engine/benches/engine_perf.rs`): a 4-player
board of **40** vanilla 2/2 creatures, each carrying lethal marked damage, swept by one
`check_and_apply_sbas` call. Every creature leaves the battlefield, so the sweep exercises
40 `capture_lki_snapshot` calls. Vanilla creatures have no dies-triggers, so no ability
lands on the stack — the measurement is pure departure work.

Measured on this dev box (AMD Ryzen 7 7800X3D), `--bench engine_perf`, median of 100
samples, by temporarily stubbing parts of `capture_lki_snapshot`:

| Variant | board_wipe_4p (40 departures) | capture share of path |
|---|---|---|
| Unconditional capture (SR-13 baseline) | **~118 µs** | — |
| Capture fully stubbed off | ~98 µs | **~17%** total |
| `calculate_characteristics` only, no clone+insert | ~104 µs | clone+insert ≈ **~12%** |
| — implied `calculate_characteristics` component | — | ≈ **~5%** |

So the capture was ~17% of the board-wipe path, split ~5% layer-eval + ~12% clone+insert.
The **clone + `OrdMap` insert is the dominant, safely-skippable half.**

## Decision: gate the store on relevant-keyword presence

17% is above the ~5% "leave it" threshold, so a gate is warranted. The store — not the
whole capture — is gated:

```
if let Some(chars) = calculate_characteristics(self, object_id) {
    if no keyword in {Wither, Infect, Deathtouch, Lifelink} ⊆ chars.keywords { return; }
    // clone + insert
}
```

**Why this is correctness-safe.** The `lki_objects` store has exactly two readers, both in
`crates/engine/src/effects/mod.rs`:

- `damage_source_characteristics` — reads the snapshot's `characteristics` *only* to test
  `keywords.contains(&{Wither, Infect, Deathtouch, Lifelink})` (CR 702.80c / 702.90e /
  702.2 / 702.15b, applied to a departed source via CR 608.2h / 113.7a).
- `damage_source_controller` — reads the snapshot's `controller`, and is called *only*
  when the source has Lifelink.

A departing permanent whose layer-resolved characteristics carry none of those four
keywords can therefore never change an outcome through this store. Skipping its snapshot is
observationally invisible.

**Why `calculate_characteristics` stays.** A relevant keyword can be *granted* by a
continuous effect (Tainted Strike's infect, Basilisk Collar's deathtouch/lifelink), which
is only visible after layer resolution — so the gate keys on the layer-resolved `chars`,
not the base keywords, and the layer eval cannot be short-circuited. This is the residual
~5%.

**Why it is timing-independent (and the SR-13 field-doc caution is respected).** The gate
does *not* consult `stack_objects` or `pending_triggers`. The SR-13 docs deliberately kept
capture ungated on a non-empty stack because a damage ability can be a *pending* trigger —
or not yet queued at all (a "when this dies, it deals damage" trigger is created only after
the death move) — when its source leaves. Gating on queue emptiness would drop those. Gating
on the departing object's own keywords does not: a keyworded source is captured whether or
not anything is queued yet.

Result: `board_wipe_4p` **~118 µs → ~105 µs (~11% faster)**; the residual is the layer eval,
now ~5% of the path.

## Hash / compatibility reasoning

`lki_objects` is hashed into `public_state_hash` (`hash.rs`), so the gate changes the
*contents* of a hashed field for keyword-less departures. But it touches **no `HashInto`
impl and no serde shape** — the field is still hashed the same way, the same bytes over the
same fixture. Consequences, all verified green:

- **SR-17 `decl_fingerprint`** (serde-closure source scan): unchanged — no field, type, or
  serde attribute moved. `hash_schema::declaration_fingerprint_is_pinned` passes.
- **SR-17 `stream_fingerprint`** (hash bytes over the canonical fixture): unchanged — the
  fixture is built directly, never through `move_object_to_zone`, so the gate code never
  runs while hashing it. The `hash_schema` stream pins pass.
- **`HASH_SCHEMA_VERSION`**: **no bump.** The bump checklist (`hash.rs` header) is for
  changes to *what `HashInto` feeds* or the serde shape; this is neither. It is a
  runtime-behavior change to a hashed field's values, in the same class as any gameplay
  correctness fix — which the engine does not version-bump for.
- **Determinism / loop detection (CR 104.4b)**: the gate is deterministic (a pure function
  of the departing object's resolved keywords), so two runs of the same game still agree,
  and `state_hashing` dual-instance tests pass.
- **Cross-version replays**: a replay recorded under pre-gate code would carry `lki_objects`
  snapshots for keyword-less departures that post-gate code omits, so its stored hashes
  would diverge — but this is pre-alpha with no stored replay-log fixtures (SR-10), and the
  affected entries are transient (cleared at the next `maybe_clear_lki_objects`). Practical
  impact: nil.

## Verification

- `sr13_lki_damage_source` (the SR-13 correctness suite): **green** — all four keyword
  sources (wither / infect / deathtouch / lifelink) still capture and apply from a dead
  source.
- New unit tests in `state/diagnostics.rs`: `successful_battlefield_departure_still_captures_lki`
  (keyworded source captured with empty stack+pending — timing independence),
  `keywordless_departure_is_not_captured` (the gate skips vanilla).
- `hash_schema` (both SR-17 fingerprints), `state_hashing`: green.

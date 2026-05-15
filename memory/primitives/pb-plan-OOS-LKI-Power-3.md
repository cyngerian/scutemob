# OOS-LKI-Power-3 — Hash `pre_lba_power` on 4 `GameEvent` variants (HASH 23 → 24)

**Task**: scutemob-29
**Branch**: `feat/oos-lki-power-3-hash-prelbapower-on-4-gameevent-variants-has`
**Filed**: 2026-05-15
**Seed origin**: `memory/primitives/pb-retriage-CC.md` § OOS-LKI-Power-3 (filed by PB-LKI-Power planner 2026-05-13; reaffirmed by `memory/primitives/pb-review-LKI-Power.md` row 17a STOP-AND-FLAG and row E2 sub-bullet on OOS-LKI-Power-3).
**CR**: CR 603.10a (LBA triggers look back in time); CR 113.7a (LKI on stack); CR 122.2 (counters cease on zone change); CR 400.7 (zone change → new object).

## Goal

Close the determinism gap noted in OOS-LKI-Power-3: the four LBA-event variants
on `GameEvent` (`AuraFellOff`, `ObjectExiled`, `PermanentDestroyed`,
`ObjectReturnedToHand`) currently destructure with `..` in their `HashInto`
arms, so their `pre_lba_counters` (PB-LKI-CC) and `pre_lba_power` (PB-LKI-Power)
LKI payloads are not folded into the state hash. `CreatureDied` already hashes
both `pre_death_counters` and `pre_death_power` at `state/hash.rs:3463-3473`.
This batch makes the four sibling arms symmetric.

The seed at `pb-retriage-CC.md:621-635` describes the inconsistency on BOTH
axes (counters + power); the AC text foregrounds `pre_lba_power` because that
is the field PB-LKI-Power added, but eliminating `..` necessarily forces a
decision on `pre_lba_counters` too. **Decision: hash both, closing the full
OOS-LKI-Power-3 seed in one HASH bump.** Leaving `pre_lba_counters` destructured
but unhashed would re-open the same kind of inconsistency the seed was filed
to retire.

## Scope (in)

1. `crates/engine/src/state/hash.rs` — `HashInto for GameEvent`, four arms:
   - `AuraFellOff` (currently disc `29u8`, line ~3483)
   - `ObjectExiled` (currently disc `41u8`, line ~3570)
   - `PermanentDestroyed` (currently disc `42u8`, line ~3581)
   - `ObjectReturnedToHand` (currently disc `50u8`, line ~3640)

   Replace each `..` destructure with explicit `pre_lba_counters` /
   `pre_lba_power` bindings; hash `pre_lba_counters` (deterministic
   `im::OrdMap` iteration; mirror the loop from `CreatureDied` at lines
   3468-3471) followed by `pre_lba_power.hash_into(hasher)` (mirror line 3473).

2. `HASH_SCHEMA_VERSION` bump 23 → 24 at `state/hash.rs:165`, with a new
   history entry "24" in the comment block (around `state/hash.rs:95-165`)
   citing OOS-LKI-Power-3 and CR 603.10a.

3. Sentinel sweep across the 18 test files that currently assert
   `HASH_SCHEMA_VERSION == 23u8`. Bump to `24u8` and uniformly rewrite the
   accompanying assertion message to cite OOS-LKI-Power-3 (replacing
   PB-EWC-D's message inherited by each file).

4. Determinism test. New test file
   `crates/engine/tests/primitive_pb_oos_lki_power_3.rs` (separate file is
   cleaner than appending to `primitive_pb_lki_power.rs`; future re-traces
   can sweep `oos_lki_power_3` as a single grep token). Asserts:
   - Sub-test α: two `GameEvent::AuraFellOff` values differing only in
     `pre_lba_power` (`None` vs `Some(2)` vs `Some(5)`) produce three
     pairwise-distinct hashes. Discriminates Option tag-byte + value.
   - Sub-test β: same for `GameEvent::PermanentDestroyed`. (Pairs ≥ 2 of the
     4 variants per AC #3940.)
   - Sub-test γ: `pre_lba_counters` is also now hashed — same-value-differ-in-
     counters control case on `GameEvent::ObjectExiled` produces distinct
     hashes. (Defensive; if we ever scope back to power-only, drop this
     sub-test.)
   - Sub-test δ: `HASH_SCHEMA_VERSION == 24u8` sentinel inside the same file
     (so the test forces a fail if someone bumps the version without updating
     this file's own assertion).

## Scope (out, with rationale)

- **Toughness LKI** (OOS-LKI-Power-1) — not yet implemented at the
  `EffectAmount`/`EffectContext` layer. No `pre_lba_toughness` field exists
  on the four GameEvent payloads. No hash arm to update.
- **AnyCreatureDies LKI-power** (OOS-LKI-Power-4) — separate axis (different
  PendingTrigger field would be needed); the `GameEvent::CreatureDied` arm
  already hashes `pre_death_power`. Untouched here.
- **Non-creature SBA path LKI capture** (OOS-LKI-Power-5) — pre-existing
  seed (`pb-retriage-CC.md:662-683`). Per AC #3943, worker decides whether
  to fold in. **Decision: DEFER, do not fold in.** Rationale: the four sites
  (`abilities.rs:890`, `sba.rs:733/854/1170`) are an event-emission concern
  (capture-time correctness), not a hash-arm consistency concern. Bundling
  them with the hash bump expands blast radius from "deterministic state
  hash" (replay correctness) to "LBA snapshot at non-creature SBA" (live
  game-state correctness for layer-4 animated edge cases). The existing
  OOS-LKI-Power-5 seed at `pb-retriage-CC.md:662` already enumerates the
  four sites and the fix recipe; keeping it filed costs nothing and lets
  the next animated-non-creature card surface (currently zero confirmed)
  drive the change with proper card-yield justification. A sub-bullet
  cross-reference will be added under OOS-LKI-Power-5 noting "blocked on
  the same v24 hash bump shipped by OOS-LKI-Power-3" so future work can
  ship symmetric changes in one HASH bump.

## Dispatch / chain audit

`GameEvent` payloads (`events.rs:239-256, 373-395, 396-417, 488-510`) already
carry `pre_lba_counters: OrdMap<CounterType, u32>` and
`pre_lba_power: Option<i32>` with `#[serde(default)]` (added by PB-LKI-CC
and PB-LKI-Power respectively; backward-compatible deserialization).

`HashInto<GameEvent>` is the only consumer of `pre_lba_*` for the hash. The
emitting sites (sba.rs, abilities.rs, replacement.rs, casting.rs, mana.rs,
resolution.rs, turn_actions.rs, engine.rs) DO populate the fields per the
PB-LKI-CC / PB-LKI-Power plumbing tables (see
`pb-review-LKI-Power.md` plumbing trace, Steps 6-8). No emit-site changes
needed for this batch.

Downstream consumers of the GameEvent hash:
- `state/hash.rs::HashInto<GameState>` walks `state.events: Vector<GameEvent>`
  and hashes each. Confirmed by `grep -n 'events\.hash_into\|for ev in' hash.rs`.
- Replay determinism gates (sentinel-version comparisons) inside test files
  pin `HASH_SCHEMA_VERSION` and recompute hashes — covered by the sentinel
  sweep.

No `Command`, `Effect`, `EffectAmount`, `PendingTrigger`, or `StackObject`
changes — those payloads already hash correctly (per PB-LKI-CC and
PB-LKI-Power); only the four sibling `GameEvent` arms drift.

## Wire-format / replay impact

HASH 23 → 24 invalidates pre-PB-OOS-LKI-Power-3 replay sentinels (intentional
— that is the entire point of the bump). The `#[serde(default)]` markers on
the four `pre_lba_power` fields ensure pre-bump serialized events still
deserialize cleanly (the field defaults to `None`, which is then hashed —
which is fine; replays that didn't carry `pre_lba_power` will now
deterministically hash as if every event had `pre_lba_power: None`, the
same value the old `..` skip implicitly produced).

## Implementation steps (worker)

1. Edit `state/hash.rs`:
   1a. Replace `..` in `AuraFellOff` arm with explicit destructure +
       hash both fields (loop for counters + line for power).
   1b. Same for `ObjectExiled` (also has `player` already destructured).
   1c. Same for `PermanentDestroyed`.
   1d. Same for `ObjectReturnedToHand`.
   1e. Add history-comment entry "v24: PB-OOS-LKI-Power-3 (2026-05-15) — …"
       in the comment block around line 95-165.
   1f. Bump `HASH_SCHEMA_VERSION` 23 → 24 at line 165.
2. Bulk sentinel sweep across 18 files (`23u8` → `24u8`, and unify the
   assertion message to cite OOS-LKI-Power-3). The current message text
   reads "PB-EWC-D bumped HASH_SCHEMA_VERSION 22→23 (new ObjectFilter::
   CreatureControlledByOfSubtype variant + bind_object_filter
   OwnedByOpponentsOf rebind). If you bumped again, update this test and
   state/hash.rs history." Replace with "OOS-LKI-Power-3 bumped
   HASH_SCHEMA_VERSION 23→24 (4 GameEvent LBA variants now hash
   pre_lba_counters + pre_lba_power per CR 603.10a). If you bumped again,
   update this test and state/hash.rs history."
3. Create `crates/engine/tests/primitive_pb_oos_lki_power_3.rs` with the
   four-sub-test determinism check (α, β, γ, δ).
4. Update `pb-retriage-CC.md` OOS-LKI-Power-3 entry — mark CLOSED 2026-05-15
   with HASH 24 + scutemob-29 ref. Add cross-reference sub-bullet to
   OOS-LKI-Power-5 noting "deferred per scutemob-29 plan; will reuse HASH 24
   if shipped before another bump, else bumps independently."
5. Run gates: `cargo test --workspace` / `cargo clippy --workspace
   --all-targets -- -D warnings` / `cargo fmt --all --check` /
   `cargo build --workspace`.
6. Spawn `primitive-impl-reviewer` (Opus). Write findings to
   `memory/primitives/pb-review-OOS-LKI-Power-3.md`. Resolve any
   HIGH / MEDIUM before signal-ready.
7. ESM: `task satisfy` for criteria 3937-3943; `task signal-ready`;
   `session end`.

## Risks

- **R1** — Some sentinel sites may have variant assertion-message wording
  (e.g. legacy PB-X messages not yet swept to PB-EWC-D's wording). Sweep is
  bulk via grep; verify by re-grepping `HASH_SCHEMA_VERSION,\s*23u8` after
  the bulk edit returns zero hits.
- **R2** — `pre_lba_counters` is `im::OrdMap<CounterType, u32>`. Verify the
  `CreatureDied` arm's `for (ct, count) in pre_death_counters.iter()` pattern
  works for the same `OrdMap<CounterType, u32>` type (it does — line 3468-3471
  is the exact template). Deterministic iteration order is guaranteed by
  `im::OrdMap`'s `Ord` key requirement.
- **R3** — `#[serde(default)]` on `pre_lba_power: Option<i32>` means
  pre-PB-LKI-Power serialized events deserialize to `pre_lba_power: None`.
  Pre-bump replay hashes that EXCLUDED the field (via `..`) and post-bump
  hashes that INCLUDE `pre_lba_power: None` will differ by exactly the
  Option tag byte (0u8). This is the entire point of the bump — flag in
  the history entry. No mitigation needed; deliberate.
- **R4** — Sub-bullet decision: deferring OOS-LKI-Power-5 means the next
  animated-non-creature card will require a fresh HASH bump (or piggy-back
  if it arrives before another bump). Acceptable; documented above.

## Test plan

The new file `primitive_pb_oos_lki_power_3.rs` is the focused determinism
canary. Other gates (workspace test, clippy, fmt, build) catch regression
of the four arms. No game-state behavior changes; only state-hash
output changes. Replay determinism tests (sentinel-version asserts in 18
files) cover the schema bump.

## Acceptance-criterion map

| ESM AC | Coverage |
|--------|----------|
| 3937 (engine surface) | Step 1 + Step 1a-1d |
| 3938 (HASH bump + history) | Step 1e-1f |
| 3939 (sentinel sweep + uniform message) | Step 2 |
| 3940 (determinism test, ≥2 variants) | Step 3 (covers 3 variants α/β/γ, plus δ sentinel) |
| 3941 (plan + review) | this file + Step 6 |
| 3942 (gates) | Step 5 |
| 3943 (sub-bullet decision documented) | "Scope (out)" + Step 4 cross-reference |

# Primitive Batch Review: PB-OOS-LKI-Power-3 — Hash `pre_lba_counters` + `pre_lba_power` on 4 `GameEvent` LBA variants

**Date**: 2026-05-15
**Reviewer**: primitive-impl-reviewer (Opus)
**ESM task**: scutemob-29
**Commit reviewed**: `be658f2a`
**Branch**: `feat/oos-lki-power-3-hash-prelbapower-on-4-gameevent-variants-has`
**CR Rules**: CR 603.10a (LBA / leaves-graveyard / public-zone-to-hand-or-library triggers look back in time); CR 113.7a (LKI used when source no longer in expected zone); CR 122.2 (counters cease on zone change); CR 400.7 (zone change → new object)
**Engine files reviewed**: `crates/engine/src/state/hash.rs`
**Test files reviewed**: `crates/engine/tests/primitive_pb_oos_lki_power_3.rs` (new) + 18 sentinel-sweep files
**Card defs reviewed**: none (engine-consistency PB; no card defs in scope)

## Verdict: PASS-WITH-NITS

The implementation is correct and complete on every load-bearing axis. All four
`GameEvent` LBA hash arms (`AuraFellOff` disc 29, `ObjectExiled` disc 41,
`PermanentDestroyed` disc 42, `ObjectReturnedToHand` disc 50) now explicitly
destructure and deterministically hash both `pre_lba_counters` and
`pre_lba_power`, byte-for-byte mirroring the `CreatureDied` template (disc 27,
lines 3469-3487). No `..` remains in those arms. Discriminant bytes are
unchanged. `HASH_SCHEMA_VERSION` is bumped 23→24 with an accurate v24 history
entry citing OOS-LKI-Power-3 + CR 603.10a. The sentinel sweep is clean: zero
`23u8` sites remain, 19 `24u8` sites carry a uniform OOS-LKI-Power-3 assertion
message. The new determinism test discriminates the Option tag byte
(None vs Some(0)), distinct payload values, and the counter axis (empty vs
populated `OrdMap`), and covers all four variants. Scope is clean — no emit-site
changes, no card defs, no new enum variants. The retriage doc is updated
(OOS-LKI-Power-3 marked CLOSED; OOS-LKI-Power-5 cross-reference sub-bullet added).
Backward-compat reasoning (R3) is sound: all four payloads carry
`#[serde(default)]`.

The only findings are LOW pre-existing-debt nits inherited (not worsened) by the
sweep: three sentinel test functions retain stale `_is_NN` names and four retain
stale doc comments describing prior bumps. The plan's Step 2 explicitly scoped
the sweep to the `23u8` literal and the assertion message only — fn names and
doc comments were out of scope — so this is not a deviation from plan, but it
leaves a small latent-confusion footgun worth a one-line note.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | LOW | `tests/primitive_pb_ewcd.rs:142`, `primitive_pb_eat.rs:141`, `primitive_pb_xs_e.rs:158` | **Stale sentinel test function names.** Functions are named `test_pb_ewcd_hash_schema_version_is_23`, `test_pb_eat_hash_schema_version_is_21`, `test_pbxse_hash_schema_version_is_20` but now assert `24u8`. Pre-existing debt (e.g. `_is_21` was already wrong when PB-EWC-D bumped to 23); not introduced or worsened here. **Fix:** opportunistically rename to a version-agnostic form (cf. `primitive_pb_xs.rs`'s `test_pbxs_hash_schema_version_matches_live_sentinel` and `primitive_pb_xa2.rs`'s `test_pb_hash_schema_version_live_sentinel`, both already generic). Not blocking. |
| E2 | LOW | `tests/primitive_pb_ewcd.rs:137-140`, `primitive_pb_eat.rs:134-139`, `primitive_pb_ewc.rs:394`, `primitive_pb_xa2.rs:100-105` | **Stale sentinel doc comments.** Doc comments above the swept sentinel asserts still narrate the historical bump that originally created the test (e.g. "PB-EWC-D bumped HASH_SCHEMA_VERSION from 22 to 23..."), which now contradicts the `24u8` assertion body. Pre-existing; the sweep correctly updated the assertion *message* (the load-bearing text) per plan Step 2. **Fix:** optionally trim these doc comments to a version-agnostic sentence. Not blocking. |
| E3 | LOW | `tests/pbt_up_to_n_targets.rs:408-409` | **Stale inline comment above sentinel.** Comment reads "sentinel must be 15 (PB-LKI-CC bump from PB-TS's 14...)" above an assert that now checks `24u8`. Pre-existing legacy text, far out of sync. **Fix:** opportunistically delete or update the two comment lines. Not blocking. |

No HIGH or MEDIUM findings.

## Card Definition Findings

None — this PB touches no card definitions. (Scope-correct: OOS-LKI-Power-3 is an
engine hash-consistency batch with zero card yield, as documented in
`pb-retriage-CC.md:633`.)

## Detailed Verification (Checklist Items 1-10)

### 1. CR correctness — VERIFIED
CR 603.10a verified via MCP: "Some zone-change triggers look back in time. These
are leaves-the-battlefield abilities, abilities that trigger when a card leaves a
graveyard, and abilities that trigger when an object that all players can see is
put into a hand or library." The four affected events are exactly the
zone-change events that feed LBA / public-to-hand triggers — `AuraFellOff`,
`PermanentDestroyed`, `ObjectExiled` cover battlefield-leaves; `ObjectReturnedToHand`
covers the public-zone-to-hand case. CR 113.7a verified: confirms LKI is used
when the source is no longer in its expected zone. The hash.rs v24 history entry
(lines 165-177) and the four arm comments ("CR 603.10a: hash LKI ... snapshot")
cite CR 603.10a accurately. The premise — LKI-bearing event fields must be folded
into the deterministic state hash or replays diverge — is sound: `HashInto<GameState>`
walks `state.events` and hashes each `GameEvent`, so any unhashed payload field
that differs between two otherwise-identical event streams produces a hash
collision and a replay-determinism false-positive.

Nit on the test-file doc comment (`primitive_pb_oos_lki_power_3.rs:13-18`): the
CR 603.10a / 113.7a quotes are paraphrased, not verbatim CR text. The paraphrase
is faithful in substance and the arm comments in `hash.rs` cite correctly, so
this is not even a LOW finding — noted for completeness only.

### 2. Hash arms — VERIFIED
All four arms read correctly:
- `AuraFellOff` (hash.rs:3496-3512): destructures `object_id, new_grave_id,
  pre_lba_counters, pre_lba_power`; disc `29u8`; counter loop `for (ct, count) in
  pre_lba_counters.iter()`; `pre_lba_power.hash_into(hasher)`.
- `ObjectExiled` (hash.rs:3591-3609): destructures `player, object_id,
  new_exile_id, pre_lba_counters, pre_lba_power`; disc `41u8`; counter loop +
  power line.
- `PermanentDestroyed` (hash.rs:3610-3626): destructures `object_id,
  new_grave_id, pre_lba_counters, pre_lba_power`; disc `42u8`; counter loop +
  power line.
- `ObjectReturnedToHand` (hash.rs:3677-3695): destructures `player, object_id,
  new_hand_id, pre_lba_counters, pre_lba_power`; disc `50u8`; counter loop +
  power line.

(a) No `..` remains in any of the four arms. (b) `pre_lba_counters` is hashed via
the deterministic `for (ct, count) in ...iter()` loop — `im::OrdMap` guarantees
sorted iteration order by its `Ord` key requirement, so the loop is
order-deterministic (R2 confirmed). (c) `pre_lba_power.hash_into(hasher)` hashes
the `Option` (tag byte + payload). (d) Loop-then-power ordering exactly matches
the `CreatureDied` template at hash.rs:3481-3486. (e) The pre-existing
discriminant byte and all other field hashes (`object_id`, `new_*_id`, `player`)
are preserved and emitted before the new fields, identical to before. Discriminant
bytes confirmed UNCHANGED: 29 / 41 / 42 / 50.

### 3. No `..` left — VERIFIED
Inspected the four arms directly; none uses `..`. The plan correctly notes other
`GameEvent` arms with genuinely-fieldless or fully-destructured patterns are
fine — those are out of scope and were not touched.

### 4. HASH bump — VERIFIED
`HASH_SCHEMA_VERSION = 24` at hash.rs:178. A v24 history-comment entry exists at
hash.rs:165-177, citing OOS-LKI-Power-3, naming all four variants, citing
CR 603.10a, and explaining the `#[serde(default)]` backward-compat behavior. The
entry is accurate and complete.

### 5. Sentinel sweep — VERIFIED
`grep -rn 'HASH_SCHEMA_VERSION,\s*23u8' crates/engine/tests/` returns ZERO hits.
`grep` for `24u8` returns 19 sites across 18 files: the 18 swept files (one each)
plus `pbt_up_to_n_targets.rs` which has 2, plus the new test file's own δ
sentinel — totaling 19, matching the plan's expectation. All 19 carry the
identical uniform assertion message: "OOS-LKI-Power-3 bumped HASH_SCHEMA_VERSION
23→24 (4 GameEvent LBA variants now hash pre_lba_counters + pre_lba_power per
CR 603.10a). If you bumped again, update this test and state/hash.rs history."
Spot-checked `primitive_pb_xs.rs:70`, `primitive_pb_ewcd.rs:145`,
`primitive_pb_eat.rs:144`, `primitive_pb_xa2.rs:110`, `pbt_up_to_n_targets.rs:412`
— message text is byte-identical. R1 (verify zero `23u8` hits after sweep) is
satisfied. (See findings E1/E2/E3 for the stale fn-name / doc-comment debt the
sweep did not touch — out of plan scope.)

### 6. Determinism test — VERIFIED
`crates/engine/tests/primitive_pb_oos_lki_power_3.rs` is a single combined test
`test_pb_oos_lki_power_3_lba_variants_hash_pre_lba_fields` with four sub-blocks:
- **δ**: asserts `HASH_SCHEMA_VERSION == 24u8` (collocated sentinel).
- **α** (`AuraFellOff`): four fixtures — `None`, `Some(0)`, `Some(2)`, `Some(5)`.
  Asserts `None ≠ Some(0)` (discriminates the Option tag byte), `Some(2) ≠ Some(5)`
  (discriminates payload value), `None ≠ Some(2)` (three-way).
- **β** (`PermanentDestroyed`): `None`, `Some(0)`, `Some(4)` — same three pairwise
  asserts. Covers a second of the four variants per AC #3940.
- **γ** (`ObjectExiled`): empty `OrdMap` vs `OrdMap{PlusOnePlusOne: 3}` — asserts
  the counter axis is also folded into the hash. `CounterType::PlusOnePlusOne`
  verified as a real variant (`state/types.rs:239`).
- **Bonus** (`ObjectReturnedToHand`): `None` vs `Some(3)` `pre_lba_power` — covers
  the fourth variant.

All four variants are exercised (exceeds the AC #3940 ≥2 requirement). The
None-vs-Some(0) pairs specifically catch the failure mode where the tag byte is
dropped (a `Some(0)` payload hashing identically to `None`). Tests cite CR 603.10a
and CR 113.7a in the module and per-block comments. Assertions check the right
thing (pairwise hash distinctness on `hash_event`, which calls the engine's
`HashInto` impl). Compilation deps (`blake3::Hasher`, `im::OrdMap`,
`mtg_engine::state::hash::HashInto`, `CounterType`, `GameEvent`, `ObjectId`,
`PlayerId`, `HASH_SCHEMA_VERSION`) are all valid public exports.

### 7. Scope discipline — VERIFIED
The diff is hash-arm + sentinel sweep + new test + memory docs ONLY. The four
`pre_lba_power` *emit/snapshot* sites (`abilities.rs:890`, `sba.rs` capture sites)
are NOT touched — those belong to OOS-LKI-Power-5 and are explicitly deferred
(plan "Scope (out)"). No card defs. No `Command`/`Effect`/`EffectAmount`/
`PendingTrigger`/`StackObject` changes. No engine logic changes — only state-hash
*output* changes.

### 8. Backward compat — VERIFIED
All four `GameEvent` variants carry `#[serde(default)]` on both `pre_lba_counters`
and `pre_lba_power` (confirmed at `rules/events.rs:245-255, 384-394, 406-416,
499-509`). Pre-bump serialized events deserialize cleanly with
`pre_lba_power: None` and an empty `pre_lba_counters` `OrdMap`. The plan's R3
reasoning is sound: pre-bump hashes excluded these fields via `..`; post-bump
hashes include `pre_lba_power: None` (tag byte 0) — the hash output changes,
which is the *intent* of the schema bump, and no live game-state behavior changes
(only hash output). The v24 history entry documents this explicitly.

### 9. Sub-bullet decision (AC #3943) — VERIFIED
The plan documents the explicit decision to DEFER OOS-LKI-Power-5 ("Scope (out)",
lines 74-88: "Decision: DEFER, do not fold in", with blast-radius rationale).
`pb-retriage-CC.md` is updated: OOS-LKI-Power-3 marked "**Status**: CLOSED by
PB-OOS-LKI-Power-3 (scutemob-29, 2026-05-15). HASH 23→24..." (line 634), and
OOS-LKI-Power-5 has a cross-reference sub-bullet (line 682): "Cross-ref: blocked
on the same v24 hash bump shipped by OOS-LKI-Power-3 (scutemob-29). When a real
animated-non-creature card surfaces, capture the four sites and bump HASH 24→25..."

### 10. Exhaustive-match risk — VERIFIED (by reasoning)
The diff adds no new enum variants — no `StackObjectKind`, no `KeywordAbility`, no
`GameEvent` variant. It only changes the body of four existing `HashInto for
GameEvent` match arms. Therefore the exhaustive matches in TUI `stack_view.rs` and
replay-viewer `view_model.rs` are unaffected and require no new arms. No
compilation impact on downstream crates.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 603.10a (LBA / public-to-hand triggers look back in time) | Yes — 4 GameEvent LBA arms hash their LKI payloads | Yes — `primitive_pb_oos_lki_power_3.rs` α/β/γ/bonus | All four LBA event variants covered |
| 113.7a (LKI used when source left zone) | Yes — `pre_lba_power`/`pre_lba_counters` are the LKI carriers | Indirectly (the test asserts hash folding, not trigger resolution) | Trigger-resolution behavior owned by PB-LKI-Power/PB-LKI-CC; this PB only closes the hash gap |
| 122.2 (counters cease on zone change) | N/A — captured pre-zone-change by emit sites (PB-LKI-CC) | γ sub-test exercises the captured-counter hash | Emit-side correctness is PB-LKI-CC, unchanged here |
| 400.7 (zone change → new object) | N/A — invariant context, no code path here | — | Cited as background only |

## Acceptance-Criteria Map (ESM AC 3937-3943)

| ESM AC | Description | Status | Evidence |
|--------|-------------|--------|----------|
| 3937 | Engine surface — 4 GameEvent LBA arms hash `pre_lba_*` | SATISFIED | hash.rs:3496-3512, 3591-3609, 3610-3626, 3677-3695 — all `..`-free, counter loop + power line |
| 3938 | HASH bump 23→24 + history entry | SATISFIED | `HASH_SCHEMA_VERSION = 24` (hash.rs:178); v24 history entry (hash.rs:165-177) cites OOS-LKI-Power-3 + CR 603.10a |
| 3939 | Sentinel sweep + uniform message | SATISFIED | 0× `23u8`, 19× `24u8`, byte-identical OOS-LKI-Power-3 assertion message across all sites |
| 3940 | Determinism test, ≥2 variants | SATISFIED (exceeds) | `primitive_pb_oos_lki_power_3.rs` covers all 4 variants + Option tag-byte + counter axis |
| 3941 | Plan + review | SATISFIED | `pb-plan-OOS-LKI-Power-3.md` + this file |
| 3942 | Gates (test/clippy/fmt/build) | NOT VERIFIED BY REVIEWER | Reviewer is read-only; runner must confirm `cargo test/clippy/fmt/build --workspace` clean before signal-ready |
| 3943 | Sub-bullet decision documented | SATISFIED | Plan "Scope (out)" defers OOS-LKI-Power-5; `pb-retriage-CC.md` marks -3 CLOSED + adds -5 cross-ref sub-bullet |

## Reviewer Checklist

- [x] CR 603.10a and CR 113.7a independently verified via mtg-rules MCP
- [x] All 4 hash arms destructure explicitly, no `..` remaining
- [x] Counter loop is deterministic (`im::OrdMap` Ord-keyed iteration)
- [x] Loop+power ordering mirrors `CreatureDied` template
- [x] Discriminant bytes unchanged (29 / 41 / 42 / 50)
- [x] `HASH_SCHEMA_VERSION = 24` + accurate v24 history entry
- [x] Sentinel sweep: 0× `23u8`, 19× `24u8`, uniform message
- [x] Determinism test discriminates tag byte, value, counter axis; covers all 4 variants; cites CR
- [x] Scope: no emit-site changes, no card defs, no new enum variants
- [x] Backward compat: `#[serde(default)]` on all 4 payloads × 2 fields
- [x] Retriage doc updated (CLOSED + cross-ref)
- [x] No exhaustive-match downstream impact
- [ ] Build/test/clippy/fmt gates — **runner must confirm** (reviewer is read-only)

## Action Required Before signal-ready

- None blocking. Findings E1/E2/E3 are LOW pre-existing debt and may be deferred
  or fixed opportunistically; they do not gate this PB.
- Runner must independently confirm AC #3942 (workspace test/clippy/fmt/build all
  green) — not verifiable by a read-only reviewer.

## Resolution (worker, 2026-05-15, commit follows be658f2a)

All three LOW findings were fixed opportunistically — they are all inside the
sentinel files the PB already swept, so resolving them makes the sweep genuinely
uniform (the spirit of AC #3939) rather than leaving fn names / comments that
contradict the `24u8` assertion body.

- **E1 RESOLVED** — stale sentinel test fn names renamed to version-agnostic
  `..._hash_schema_version_live_sentinel` form. Fixed the 3 flagged
  (`primitive_pb_ewcd.rs`, `primitive_pb_eat.rs`, `primitive_pb_xs_e.rs`) plus
  3 more found during the sweep that the review did not enumerate
  (`primitive_pb_ewc.rs` `_is_18`, `primitive_pb_lki_cc.rs` `_is_15`,
  `effect_sacrifice_permanents_filter.rs` `_is_15`, and
  `pbt_up_to_n_targets.rs` `_sentinel_is_15_regression`).
- **E2 RESOLVED** — stale sentinel doc comments narrating prior bumps replaced
  with a version-agnostic one-liner ("HASH_SCHEMA_VERSION live sentinel — fails
  if the schema version drifts ...") in `primitive_pb_ewcd.rs`,
  `primitive_pb_eat.rs`, `primitive_pb_ewc.rs`, `primitive_pb_xa2.rs`,
  `primitive_pb_lki_cc.rs`.
- **E3 RESOLVED** — `pbt_up_to_n_targets.rs:408-409` stale inline comment
  ("sentinel must be 15 ...") replaced with a version-agnostic note.

Not changed (genuinely accurate historical records, out of finding scope): the
module-level `//!` header docs that state what each PB historically bumped
(e.g. `primitive_pb_xs.rs:12` "PB-XS bumped 18→19"), and
`primitive_pb_ts.rs:358`'s history-framed doc comment. These describe each
PB's own contribution accurately and are not contradicted by the live `24u8`
assertion.

**Final verdict: PASS** — all findings resolved; no open items.

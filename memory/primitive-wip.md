# Primitive batch WIP — ENG-2 (prior batches are stale history; see their own plan files)

**Batch**: ENG-2 — targets in the event log: an announcement-time target event, viewer-gated
**Seeds**: `OOS-G7-1` (no `GameEvent` carries targets) — G7 of
`memory/playtest-triage-2026-08-02b.md`, row 7 of its successor table
**Task**: `scutemob-193` · **Branch**: `feat/eng-2-targets-in-the-event-log-an-announcement-time-target-e`
**Phase**: fix (stage E — version-gate bump — DONE)

**Baselines measured on this branch before any edit**: PROTOCOL **34**, HASH **71**
(`rules/protocol.rs::PROTOCOL_VERSION`, `state/hash.rs::HASH_SCHEMA_VERSION`). The triage says
"PROTOCOL currently 33" — that is stale; ENG-1 (merge `a3b5e56b`) moved both after the triage
was written. Both new values must be read from the failing gates' own output, never predicted.

- [x] **Stage E — PROTOCOL/HASH version bump (gate-computed)**: `PROTOCOL_VERSION` 34 -> 35
  (fingerprint `7a5fc4b0…386b`), `HASH_SCHEMA_VERSION` 71 -> 72 (decl_fingerprint
  `6cb06c10…9329`, stream_fingerprint `c5786aaa…eed5c` — read from the gate AFTER the version
  bump, per the `HASH_SCHEMA_VERSION` byte folded into the stream; this is the v69/PB-DX1
  version-sentinel-byte-only case, not a payload-bytes case, since `canonical_fixture()` has no
  `PendingTrigger.triggering_event` populated with `TargetsAnnounced`). Both `PROTOCOL_HISTORY`
  and `HASH_SCHEMA_HISTORY` rows appended (never edited in place); both `FROZEN_HISTORY_PREFIX_DIGEST`
  constants and both version sentinels re-pinned. All 45 `HASH_SCHEMA_VERSION` and 11
  `PROTOCOL_VERSION` scattered live sentinels re-pinned **by symbol** (two double-checked via
  grep -A1 for multi-line survivors: `pb_dx2_command_gates.rs` and `pb_dp5_pending_draw_choice.rs`
  both split the assertion across three lines and were caught). Full workspace:
  **4,341 passed / 0 failed / 5 ignored**; `cargo fmt --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, and `tools/check-defs-fmt.sh` all clean. Commit:
  `scutemob-193: ENG-2 stage E — PROTOCOL 34→35, HASH 71→72 (gate-computed)`.

## Plan file

`memory/primitives/pb-plan-ENG2.md`

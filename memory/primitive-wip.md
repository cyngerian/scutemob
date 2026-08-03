# Primitive batch WIP — ENG-2 (prior batches are stale history; see their own plan files)

**Batch**: ENG-2 — targets in the event log: an announcement-time target event, viewer-gated
**Seeds**: `OOS-G7-1` (no `GameEvent` carries targets) — G7 of
`memory/playtest-triage-2026-08-02b.md`, row 7 of its successor table
**Task**: `scutemob-193` · **Branch**: `feat/eng-2-targets-in-the-event-log-an-announcement-time-target-e`
**Phase**: plan

**Baselines measured on this branch before any edit**: PROTOCOL **34**, HASH **71**
(`rules/protocol.rs::PROTOCOL_VERSION`, `state/hash.rs::HASH_SCHEMA_VERSION`). The triage says
"PROTOCOL currently 33" — that is stale; ENG-1 (merge `a3b5e56b`) moved both after the triage
was written. Both new values must be read from the failing gates' own output, never predicted.

## Plan file

`memory/primitives/pb-plan-ENG2.md`

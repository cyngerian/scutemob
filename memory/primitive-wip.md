# Primitive WIP — IDLE (no PB in progress)

<!-- last_updated: 2026-07-19 -->

No primitive batch is currently in flight. Last completed: **PB-OS4** (`scutemob-130`,
merge `7ee96913`, SHIPPED NARROWED — see `memory/primitives/pb-plan-OS4.md` /
`pb-review-OS4.md` and the queue plan).

**Active queue**: `memory/primitives/oos-retriage-plan-2026-07-18.md` (PB-OS5..OS11 pending;
**OOS-OS4-2 face-aware ability gathering is a correctness candidate that should be triaged
ahead of OS5** — it may implicate shipped PB-EF5 TransformSelf `Complete` markers).
**PB dispatches are gated on DOCB-2/3 (`scutemob-132`/`133`) collecting.**

Start the next PB via /dispatch per the queue plan (coordinator) — /implement-primitive
picks up this file as its state.

# Primitive batch WIP — PB-DX4

**Batch**: PB-DX4 — triage the 97-entry decision `BASELINE` against oracle text
**Seed**: OOS-DP10-8 (`docs/audits/decision-point-audit.md` §8.1)
**Task**: `scutemob-168` · **Branch**: `feat/pb-dx4-triage-the-97-entry-decision-baseline-against-oracle-`
**Phase**: CLOSED (implement → review → fix cycle all complete)

## Process deviation, recorded rather than hidden

**There is no `memory/primitives/pb-plan-DX4.md`.** This batch ran without a
`primitive-impl-planner` plan file (review Finding 13). The dispatch brief in
`memory/primitives/seed-rerank-2026-07-27.md` §"Dispatch briefs" was detailed enough to scope
the work, and the batch's shape — read 97 defs against oracle text, classify, fix or demote —
needed decomposition into parallel reader batches rather than an architecture plan.

Two consequences worth noting for whoever reads this next, because both cost something:

1. The class-B/class-D **standard was never written down before the readers started**, only
   described in the dispatch prompt. Seven sub-agents then applied it inconsistently — batch 2
   and batch 6 split on the identical costless-"may" shape — which is why the published split
   (84/13) has to be read as "at least 13, by non-uniform readers" rather than as a
   measurement. A plan file would have been the natural place to fix the standard first.
2. Nothing pre-committed a falsifiable yield prediction, so the queue row's "0 flips" estimate
   went unchallenged until the demotions were already in hand (the real answer was 6).

## Outcome

Split **84 class-B / 13 class-D**. Six defs repaired in place and still `Complete`; six
demoted; one (`staff_of_compleation`) allowlisted on class precedent. Also closed **OOS-M11-6**
incidentally. Coverage 1,143 → **1,137**; tests 4,040 → **4,048**; 0 engine lines; PROTOCOL 32 /
HASH 69 unmoved.

Durable record: `memory/primitives/pb-dx4-baseline-triage.md`
Review: `memory/primitives/pb-review-DX4.md` (2 HIGH / 5 MEDIUM / 6 LOW — all 13 applied)
New seeds: **OOS-DX4-1 … OOS-DX4-6**

**Next**: PB-DX5 (OOS-OS7-2 — CR 611.2c affected-set snapshot).

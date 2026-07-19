# Primitive WIP: PB-OS4 — return-transformed / enters-transformed as a NEW object (OOS-EF5-3)

batch: OS4
task: scutemob-130
branch: feat/pb-os4-return-transformed-enters-transformed-as-a-new-object
started: 2026-07-19
phase: plan

Plan: `memory/primitives/pb-plan-OS4.md`. Review: `memory/primitives/pb-review-OS4.md`.

## Brief (THE PLAN IS `memory/primitives/oos-retriage-plan-2026-07-18.md` §3 PB-OS4; canonical finding `memory/primitives/ef-batch-plan-2026-07-17.md` §9 OOS-EF5-3)

CAPABILITY, **highest yield** of the capability group. First capability dispatch off the PB-OS
queue (correctness group PB-OS1..OS3 complete).

A permanent that dies / is exiled and RETURNS to the battlefield already on its back face is a
**NEW object entering transformed** (CR 712.18) — a fundamentally different mechanism than the
in-place `Effect::TransformSelf` that PB-EF5 shipped (which keeps the same `ObjectId`). CR 400.7:
the returned permanent is a new object; the old `ObjectId` is dead; auras/counters do NOT carry;
"when this dies" triggers reference the OLD object.

## Fix shape (per plan §3 / OOS-EF5-3)
A `ReturnTransformed`/`enters_transformed` flag on the zone-change/return effect
(`Effect::MoveZone` or a dedicated `Effect::ReturnTransformed`) threaded through the return path so
the new object enters with **back-face** characteristics, layer-resolved; PLUS Saga-chapter
integration for Fable. **New wire type → PROTOCOL bump** (one bump for the whole PB, machine-forced
by SR-8 gates; justify in close-out — do NOT fight the gate).

## Candidates (4 — EACH verified vs oracle text via MCP BEFORE impl; PB-EF5 caught 2 mis-filed cards this way; honest yield ~2-3)
- **edgar_charmed_groom** — dies → delayed trigger returns it to the battlefield transformed at the
  next end step.
- **fable_of_the_mirror_breaker** — Saga chapter III: exile, return transformed (riskiest — Saga
  integration).
- **nicol_bolas_the_ravager** — `{4}{U}{B}{R}`: exile self, return transformed at next end step.
- **grist_voracious_larva** — re-verified via MCP (plan table was stale): "Whenever Grist or another
  creature you control enters, if it entered from your graveyard or you cast it from your graveyard,
  you may pay {G}. If you do, exile Grist, then return it to the battlefield transformed under its
  owner's control." — identical return-transformed mechanism, NOT a TransformSelf case.

Discounted ship **~2-3** of 4 — honest yield beats forced flips; a card with a distinct 2nd blocker
stays truthfully marked with the blocker NAMED (PB-EF5 precedent).

## Mandatory tests
- **New-object identity (CR 400.7)**: old ObjectId dead; auras/counters do NOT carry; a "when this
  dies" trigger references the OLD object. Pin CR 400.7 by test.
- **Enters-transformed characteristics (CR 712.18)**: the returned object has back-face
  characteristics, **layer-resolved** (calculate_characteristics, not raw def read).
- **Delayed-trigger timing**: return happens at the **next end step**, not immediately (edgar,
  nicol_bolas).
- **Saga chapter ordering** for Fable if shipped.
- **Decoys must fail on exactly the field under test** (SR-34/36 probe-by-execution).

## Wire bump (AC 5040)
Single PROTOCOL bump (with HASH if forced) for the whole PB, justified in close-out per SR-8;
update the sentinel tests + history rows. Do NOT fight the gate.

## Close-out (AC 5041)
Close (or honestly narrow) **OOS-EF5-3** in `oos-retriage-plan-2026-07-18.md` §3 PB-OS4 (SHIPPED
banner + table strike) and `ef-batch-plan-2026-07-17.md` §9 (CLOSED banner). Update shipped-card
header comments. Update this WIP status. Non-shipped cards keep their real named blocker.

## Steps
- [ ] 1. Plan — primitive-impl-planner → pb-plan-OS4.md (verify 4 cards vs MCP; design primitive + Saga; justify PROTOCOL bump)
- [ ] 2. Implement engine change (return-transformed threading; Saga integration for Fable) — primitive-impl-runner
- [ ] 3. Flip card defs (only genuinely-correct ones; others keep named blocker)
- [ ] 4. Tests (identity CR 400.7, characteristics CR 712.18, timing, Saga)
- [ ] 5. PROTOCOL/HASH bump + sentinel/history rows updated
- [ ] 6. Review — primitive-impl-reviewer → pb-review-OS4.md; fix cycle if findings
- [ ] 7. Green gates: build/test/clippy/fmt + check-defs-fmt.sh
- [ ] 8. Close OOS-EF5-3 in plan + source docs; /review; Completion Sequence

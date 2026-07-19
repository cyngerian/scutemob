# Primitive WIP: PB-OS4 — return-transformed / enters-transformed as a NEW object (OOS-EF5-3)

batch: OS4
task: scutemob-130
branch: feat/pb-os4-return-transformed-enters-transformed-as-a-new-object
started: 2026-07-19
phase: implement

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

## Plan outcome (2026-07-19)
- **Design**: 2 dedicated unit `Effect` variants — `ExileSourceAndReturnTransformed` (immediate; fable ch. III) + `ReturnSourceToBattlefieldTransformedNextEndStep` (delayed; edgar dies→end-step) — plus new `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed`. Reuses craft return path (engine.rs:1422-1441), ExileWithDelayedReturn idiom, SagaChapter. NOT a flag on MoveZone (blast radius).
- **Honest yield ~2**: fable_of_the_mirror_breaker + edgar_charmed_groom → Complete. nicol_bolas_the_ravager + grist_voracious_larva **STAY OUT** — named blocker: planeswalker back face + `CardFace` has no `starting_loyalty` → 0-loyalty PW dies to SBA 704.5i. File follow-up seed **OOS-OS4-1**. grist also needs entered-from-graveyard trigger condition.
- **Brief correction**: nicol_bolas returns IMMEDIATELY (not next end step) — moot (stays out) but recorded.
- **Wire**: PROTOCOL 18→19, HASH 55→56 machine-forced (3 new enum variants in SR-8 closure). Effect hash discriminants 94/95. `DelayedTriggerAction` matched at 4 hash.rs sites — all need new arm.

## Steps
- [x] 1. Plan — primitive-impl-planner → pb-plan-OS4.md (verified 4 cards; design + Saga; PROTOCOL/HASH bump justified)
- [x] 2. Implement engine change (return-transformed threading; Saga integration for Fable) — primitive-impl-runner
  - Added `Effect::ExileSourceAndReturnTransformed` + `Effect::ReturnSourceToBattlefieldTransformedNextEndStep` (card_definition.rs, near TransformSelf); `DelayedTriggerAction::ReturnFromGraveyardToBattlefieldTransformed` (stubs.rs). Immediate executor arm (effects/mod.rs, mirrors craft return path) + delayed registration arm; dispatch arm in resolution.rs (mirrors ReturnFromExileToBattlefield). Hash: Effect discriminants 94/95; DelayedTriggerAction local discriminant 5 at all 4 sites. `cargo check --workspace` clean (no TUI/replay-viewer match gaps — confirmed, matches plan's Change 9 prediction).
- [ ] 3. Flip card defs (only genuinely-correct ones; others keep named blocker)
- [ ] 4. Tests (identity CR 400.7, characteristics CR 712.18, timing, Saga)
- [x] 5. PROTOCOL/HASH bump + sentinel/history rows updated (2 commits — see divergence note below)
  - **DIVERGENCE (STOP-and-flag, proceeded per "follow the text" authorization)**: at authoring time, `cards.sqlite` oracle text for Edgar, Charmed Groom showed NO "at the beginning of the next end step" clause — "When Edgar, Charmed Groom dies, return it to the battlefield transformed under its owner's control" resolves IMMEDIATELY on the WhenDies trigger, with no exile step either (unlike Fable). Neither of the plan's two effects fit this shape. Added a THIRD unit effect, `Effect::ReturnSourceToBattlefieldTransformed` (immediate, no exile — mirrors Persist/Undying's `Effect::MoveZone{Source->Battlefield}` idiom but also flips to back face + registers statics/ETB). This makes `ReturnSourceToBattlefieldTransformedNextEndStep` (the delayed variant) currently unused by any roster card — kept anyway since it's a real, distinct CR 603.7 primitive, tested standalone per plan §8.
  - PROTOCOL_VERSION 18→19 (`rules/protocol.rs`), fingerprint `1d0dc7b8d5ea44129090b873826d798e84dd7698d1b2170214b66d65d2543e05`, FROZEN_HISTORY_PREFIX_DIGEST re-pinned to `427628738bef89b1a939590242978b532810bfbaea7f44b8d07ce6275c07b6c1`.
  - HASH_SCHEMA_VERSION 55→56 (`state/hash.rs`), decl `d8752059bb71f8c104ab76caf4995055dd9bdd2e8fe5c298e79cb3dbecaa2b98`, stream `46da56438f4951cb7b3eb76ed35fa966ffa738b6449eb611459c77359ba455ee`, FROZEN_HISTORY_PREFIX_DIGEST re-pinned to `4f1b8eba2e9cfb60cf8e7aed5d56f774b09d959352fc911af9016b0b39ac2bb2`.
  - All values copied verbatim from the failing gate tests' expected output (never hand-computed), per SR-8/SR-17 doctrine.
  - Bulk-updated ~35 scattered sentinel assertions (`HASH_SCHEMA_VERSION, 55u8` → `56u8`; `PROTOCOL_VERSION, 18` → `19`) across `crates/engine/tests/`.
  - SR-25 `bare_lookup_ratchet` gate caught 2 new bare `.objects.get(` lookups in the new `effects/mod.rs` executor + 1 in the new `resolution.rs` dispatch arm; converted to `fizzle_object`/`expect_object` (no ceiling bump needed — kept at 107/102).
  - `cargo test -p mtg-engine --test core`: 424 passed, 0 failed.
- [ ] 6. Review — primitive-impl-reviewer → pb-review-OS4.md; fix cycle if findings
- [ ] 7. Green gates: build/test/clippy/fmt + check-defs-fmt.sh
- [ ] 8. Close OOS-EF5-3 in plan + source docs; /review; Completion Sequence

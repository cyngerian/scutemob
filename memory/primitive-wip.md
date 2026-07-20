# Primitive WIP — PB-RS3 (OOS-OS9-1) · PLAN

<!-- last_updated: 2026-07-20 -->

- **PB**: PB-RS3 — `AtBeginningOfCombat` card-def sweep (`begin_combat` collects emblem triggers only)
- **Task**: `scutemob-145`
- **Branch**: `feat/pb-rs3-atbeginningofcombat-card-def-sweep-begincombat-collec`
- **Class**: CORRECTNESS (live-wrong on a `Complete`-by-default card; Invariant #9)
- **Phase**: plan
- **Binding spec**: `memory/primitives/rider-seed-triage-2026-07-19.md` §2.3 (chain notes) + §3 (R3 row)
- **Plan file**: `memory/primitives/pb-plan-RS3.md`
- **Review file**: `memory/primitives/pb-review-RS3.md`
- **Wire expectation**: **NO PROTOCOL bump, NO HASH bump.** This PB adds a collection call inside an
  existing turn-based action; it introduces no `Command` field, no `Effect` variant, no
  `HashInto`-reachable shape change. If a bump IS forced by the machine gate, that is a
  **stop-and-re-scope signal** (AC 5127) — do not silently re-pin the fingerprint.

## The chain (from triage §2.3 — verify each hop before acting)

Exactly **one** broken hop. Hops 6-8 all work and are exercised by four sibling sweeps.

1. Engine-side `AtBeginningOfCombat` occurrences are only two `HashInto` arms
   (`state/hash.rs:3175`, `:5726`) and the emblem call at `rules/turn_actions.rs:1689-1698`.
2. **BROKEN**: `begin_combat` (`turn_actions.rs:1684-1703`) builds `CombatState`, collects
   **emblem** triggers only (CR 114.4), and returns `Vec::new()`. **There is no card-def scan.**
3. Queue → stack: `abilities.rs:8251-8258` — works.
4. Resolution + intervening-if: `resolution.rs:2018-2048`, `:2185-2219` — works.
5. `Condition::YouControlYourCommander` — works (shipped by PB-OS9).

**Four-times-proven sibling template** (the repair is a fifth copy):
`turn_actions.rs:296-301`, `:443-455`, `:507-519`, `:710-722`.

## Cards in scope (triage §3 R3 row — verify against `all_cards()`, never grep, per SR-36)

- **`helm_of_the_host.rs`** — has **no `completeness` field** ⇒ `Complete` by `#[default]`
  (`card_definition.rs:199-200`). Its only non-Equip ability is the `AtBeginningOfCombat`
  `CreateTokenCopy` at `:27-42`. **It enters real games and silently does nothing** — a corrupted
  replay history per Invariant #9. This is the live-wrong card; the probe test targets it.
- **`loyal_apprentice.rs`** — `partial` → `Complete` expected.
- **`siege_gang_lieutenant.rs`** — `partial` → `Complete` expected. Carries the intervening-if
  condition (test both directions).

Triage §3 discounted ship: **2 flips + helm_of_the_host repaired.** Honor
`feedback_pb_yield_calibration` — do not inflate.

## Standing hazard to assess (AC 5127)

`helm_of_the_host` was live-wrong *because* an omitted `completeness` field defaults to `Complete`.
Assess how widespread that pattern is across the corpus (defs with no explicit marker) and **file a
seed if widespread**. This is a class-level integrity question, not a helm-specific one.

## Steps

- [x] 0. Probe test written FIRST (helm_of_the_host token copy at begin combat), verified FAILING
      against pre-fix HEAD (`left: 0, right: 1`, no tokens created), before any production edit.
      Test: `crates/engine/tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs`.
- [x] 1. Plan phase (`primitive-impl-planner`) → `memory/primitives/pb-plan-RS3.md`.
- [x] 2. Card-def `AtBeginningOfCombat` sweep added to `begin_combat` (ENGINE SWEEP step, this
      session). `crates/engine/src/rules/turn_actions.rs` — sweep adapted verbatim from S3
      (`postcombat_main_actions`), inserted between the `CombatState` init and the existing emblem
      block, ordered before it, and placed **outside** the `if state.combat.is_none()` guard.
      `ability_index` enumerates `def.effective_abilities(obj.is_transformed)` (CardDefETB
      namespace, not the dense `triggered_abilities` namespace — §3c trap avoided). Controller
      filter: early `return None` when `controller != active`. Push via
      `PendingTrigger { ability_index, ..PendingTrigger::blank(obj_id, controller,
      PendingTriggerKind::CardDefETB) }` (SR-7 compliant). Incidental comment defect at
      `turn_actions.rs:689-690` (aspirational `AtBeginningOfEachEndStep` claim) also fixed per §1a
      — no behavior change. Probe test now PASSES. `cargo build --workspace` clean (no exhaustive-
      match gaps — this PB adds no enum variant). Full `cargo test --workspace` green, 0 failures,
      0 regressions. PROTOCOL 27 / HASH 63 confirmed unchanged (grep-verified, no touch to
      `hash.rs` or `protocol.rs`). **Card-def completeness flips (helm_of_the_host,
      loyal_apprentice, siege_gang_lieutenant) and the remaining mandatory tests (Tests 2-8,
      roster sweep) are NOT done by this step — deferred to the next step per session scope.**
- [ ] 3. `helm_of_the_host` verified vs oracle via MCP; explicit completeness marker added.
- [ ] 4. `loyal_apprentice` + `siege_gang_lieutenant` verified vs oracle, flipped `partial` →
      `Complete`, each with an end-to-end behavior test citing CR.
- [ ] 5. Mandatory tests: helm probe (inverted to permanent regression), lieutenant condition both
      directions, APNAP multi-player ordering, emblem + card-def coexistence (no double / no drop),
      extra-combat behavior with CR citation.
- [ ] 6. Full `all_cards()` roster sweep for `AtBeginningOfCombat`; roster reported in close-out.
- [ ] 7. PROTOCOL/HASH confirmed unchanged; `cargo build --workspace` clean.
- [ ] 8. Full gates: `cargo test --all`, `clippy -D warnings`, `cargo fmt --check` **and**
      `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt` checks none of the 1,804 defs).
- [ ] 9. `primitive-impl-reviewer` pass with every finding dispositioned.

## Prior state

PB-RS1 SHIPPED (`scutemob-143`, merge `56697a00`). PB-RS2 SHIPPED (`scutemob-144`, merge
`86176ff7`; PROTOCOL 26→27, HASH 63; filed OOS-RS2-1 post-`/review`). The R1..R11 ranked queue
lives in `memory/primitives/rider-seed-triage-2026-07-19.md` §3.

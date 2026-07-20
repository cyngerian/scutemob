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
- [x] 3. `helm_of_the_host` — oracle re-verified via MCP (faithful, unmodified translation);
      explicit `completeness: Completeness::Complete,` added (was `Complete` only by
      `#[default]`). Reviewer-endorsed (`memory/card-authoring/review-pb-rs3-roster.md`).
- [x] 4. `loyal_apprentice` + `siege_gang_lieutenant` — oracle re-verified via MCP; flipped
      `partial` → `Complete`. Stale "STILL BLOCKED" comment blocks replaced with a PB-RS3
      closure note on each file; haste-fallback rationale preserved. Both flips are
      reviewer-endorsed **conditional on F3** (MEDIUM, engine-wide, pre-existing): `intervening_if`
      is checked only at resolution (`resolution.rs:2125-2135`), never at queue time, though CR
      603.4 requires both — this is a standing engine-wide convention (documented at
      `turn_actions.rs:265-266`), affects every already-shipped `Complete` intervening-if card, and
      is explicitly recorded in both card notes rather than silently overclaimed. Filed as a seed
      per the review (not fixed here — out of PB-RS3 scope, touches every trigger sweep).
      **End-to-end behavior tests for these two cards are deferred to step 5/6 (Tests 2-8 + roster
      sweep), per this session's explicit scope boundary — not written in this step.**
      `legion_warboss` note amended to name BOTH live gaps (Mentor keyword absent; token's
      "attacks this combat if able" unimplemented) — explicitly did NOT add
      `MustAttackEachCombat` to `TokenSpec.keywords` (would over-restrict every later combat).
      Stays `partial`. `mirage_phalanx` `known_wrong` note amended: now wrong in BOTH directions
      (unpaired → wrongly self-copies every combat; paired → still under-produces, grant to the
      OTHER paired creature not modeled). Verified via grep: no golden script or test fixture
      constructs Mirage Phalanx via `ObjectSpec` (only a comment reference in
      `pb_os9_lieutenant_commander_control.rs`) — containment via `known_wrong` + `validate_deck`
      (SR-2) holds, zero exposure. Stays `known_wrong`.
      **`goblin_rabblemaster` PROBE (F-Rabble, HIGH finding)**: the def's stated blocker ("needs a
      new subtype-filtered must-attack `GameRestriction` variant") was misframed — the engine
      already implements must-attack via `KeywordAbility::MustAttackEachCombat`, read from
      layer-resolved characteristics (`expect_characteristics`) at `combat.rs:378-390`, not the
      object's own printed keyword list. Wrote probe test
      `crates/engine/tests/primitives/pb_rs3_rabblemaster_mustattack_probe.rs`
      (`test_addkeyword_mustattack_grant_composes_for_non_source_object`): built a mock
      Rabblemaster-shaped `Static` ability (`AddKeyword(MustAttackEachCombat)` +
      `OtherCreaturesYouControlWithSubtype("Goblin")` + `WhileSourceOnBattlefield`), registered it
      via `register_static_continuous_effects` (the `pb_os4b_face_aware_abilities.rs` pattern,
      since `GameStateBuilder` does not replay ETB), and drove it through the FULL enforcement
      path (`Command::DeclareAttackers`), not just a characteristics snapshot. **PROBE RESULT:
      YES, it composes cleanly** — the granted keyword reaches the non-source Goblin's
      layer-resolved characteristics, the source itself correctly does NOT get the keyword (CR
      "other"), and `DeclareAttackers` correctly rejects a declaration that omits the forced
      Goblin while accepting one that includes it. Sanity-checked the probe itself is
      non-vacuous: temporarily disabled the `register_static_continuous_effects` call and
      confirmed the test fails (then restored). **No engine change needed.** Authored the real
      ability onto `goblin_rabblemaster.rs` (identical shape to `galadhrim_brigade.rs` /
      `camellia_the_seedmiser.rs`, swapping the modification for `AddKeyword(MustAttackEachCombat)`)
      and flipped `partial` → `Complete` — **legitimate third flip**, per plan §5c authorization
      for a clean composition. `cargo check -p mtg-card-defs` clean.
- [ ] 5. Mandatory tests: helm probe (inverted to permanent regression), lieutenant condition both
      directions, APNAP multi-player ordering, emblem + card-def coexistence (no double / no drop),
      extra-combat behavior with CR citation. **NOT done in this step — deferred per session scope
      (card-def step only; Tests 2-8 + roster sweep are a later step's job).**
- [ ] 6. Full `all_cards()` roster sweep for `AtBeginningOfCombat`; roster reported in close-out.
- [ ] 7. PROTOCOL/HASH confirmed unchanged; `cargo build --workspace` clean.
- [ ] 8. Full gates: `cargo test --all`, `clippy -D warnings`, `cargo fmt --check` **and**
      `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt` checks none of the 1,804 defs).
- [ ] 9. `primitive-impl-reviewer` pass with every finding dispositioned.

## Prior state

PB-RS1 SHIPPED (`scutemob-143`, merge `56697a00`). PB-RS2 SHIPPED (`scutemob-144`, merge
`86176ff7`; PROTOCOL 26→27, HASH 63; filed OOS-RS2-1 post-`/review`). The R1..R11 ranked queue
lives in `memory/primitives/rider-seed-triage-2026-07-19.md` §3.

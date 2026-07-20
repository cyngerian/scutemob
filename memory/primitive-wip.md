# Primitive WIP — PB-RS3 (OOS-OS9-1) · PLAN

<!-- last_updated: 2026-07-20 -->

- **PB**: PB-RS3 — `AtBeginningOfCombat` card-def sweep (`begin_combat` collects emblem triggers only)
- **Task**: `scutemob-145`
- **Branch**: `feat/pb-rs3-atbeginningofcombat-card-def-sweep-begincombat-collec`
- **Class**: CORRECTNESS (live-wrong on a `Complete`-by-default card; Invariant #9)
- **Phase**: review (steps 0-8 done — engine sweep, card-def flips, mandatory tests 2-8,
  roster sweep, wire/gate verification all complete; step 9 `primitive-impl-reviewer` pass
  is the only remaining step)
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
      and flipped `partial` → `Complete` — **legitimate third flip**, authorized by the roster
      reviewer's F-Rabble finding plus this purpose-built probe, NOT by plan §5c (§5c is titled
      "The other three roster members — do NOT flip" and explicitly predicts Rabblemaster "stays
      `partial`; the surviving blocker is the subtype-filtered forced-attack `GameRestriction`" —
      the flip overrides that prediction, it does not follow from it; corrected per PB-RS3 review
      Finding 4, which caught the original record citing an authority that said the opposite).
      `cargo check -p mtg-card-defs` clean.
- [x] 5. Mandatory tests 2-8 written in
      `crates/engine/tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs` (registered,
      already present, in `primitives/main.rs`): Test 2 index-space discriminator
      (`test_loyal_apprentice_trigger_uses_carddef_ability_index_namespace`), Tests 3a/3b
      siege_gang intervening-if both directions (holds / fails-when-commander-removed),
      Test 5 APNAP/controller scoping (4-player), Test 6 emblem+card-def coexistence
      (Basri Ket emblem + Helm, no double/no drop, queue-order pinned), Test 7 extra-combat
      refire (CR 506.1/603.2), Test 8 unattached-Helm negative edge (CR 702.6). All 8 tests
      in the file pass. `pb_os9_lieutenant_commander_control.rs`'s file-level doc comment and
      the Siege-Gang test's doc comment corrected (sweep now shipped; that file's own tests
      still isolate resolution-only, cross-referenced to the new end-to-end tests).
      **Every new test's discrimination verified empirically** (temporarily broke the cited
      production code, confirmed FAIL with the predicted message, reverted, confirmed PASS —
      see close-out report for the full before/after transcript per test). One correction
      made during verification: Test 7's doc comment originally claimed to guard the
      `state.combat.is_none()` nesting trap (R2); empirically, nesting the sweep in that
      guard does NOT reproduce a failure in this harness (because `end_combat` unconditionally
      resets `state.combat = None` before the redirect), so the comment was corrected to
      describe what was actually verified (an R4-shaped "skip on repeat entry" mutation, which
      DOES fail the test as predicted).
- [x] 6. Full `all_cards()` roster sweep written: `crates/engine/tests/core/pb_rs3_combat_trigger_roster.rs`
      (registered in `core/main.rs`). Enumerates `all_cards()` (SR-36), walks
      `serde_json::to_value(&def)` recursively, scoped to the `trigger_condition` JSON key
      (a bare `contains_key`/string-value walk was tried first and found a real false
      positive — Basri Ket's emblem `trigger_on: TriggerEvent::AtBeginningOfCombat` serializes
      to the identical bare string as `TriggerCondition::AtBeginningOfCombat`; fixed by scoping
      the match to the `trigger_condition` field name). **Roster: exactly 6**, matching the
      plan's predicted roster: Helm of the Host, Loyal Apprentice, Siege-Gang Lieutenant,
      Goblin Rabblemaster, Legion Warboss, Mirage Phalanx. Basri Ket confirmed excluded
      (emblem path, not card-def path) — asserted directly. Completeness pinned per member:
      **4 Complete** (Helm of the Host, Loyal Apprentice, Siege-Gang Lieutenant, **Goblin
      Rabblemaster** — this last one is real information that diverges from the plan §7
      table's prediction, which expected Rabblemaster to stay `partial`; step 4's F-Rabble
      probe legitimately flipped it, a third flip beyond the plan's predicted two), 1 Partial
      (Legion Warboss), 1 KnownWrong (Mirage Phalanx). Non-vacuity floor (`>= 6`) and all
      completeness pins verified to discriminate (temporarily broke the field-name match,
      confirmed roster collapses to 0 and the assertion fails with the predicted message,
      reverted, confirmed pass).
- [x] 7. PROTOCOL/HASH confirmed unchanged: PROTOCOL_VERSION == 27, HASH_SCHEMA_VERSION == 63
      (grep-verified; `git diff --stat` on `protocol.rs`/`hash.rs` empty). `cargo build
      --workspace` clean.
- [x] 8. Full gates green: `cargo test --all` (all suites, 0 failed), `cargo clippy
      --all-targets -- -D warnings` (clean), `cargo fmt --check` (clean — one file needed
      `cargo fmt` applied, a multi-line `all.push(...)` call reformatted; re-verified clean
      and re-ran the full suite after), `tools/check-defs-fmt.sh` (1804 defs, clean). No
      remaining TODOs in the flipped card defs (helm_of_the_host, loyal_apprentice,
      siege_gang_lieutenant, goblin_rabblemaster).
- [x] 9. `primitive-impl-reviewer` pass with every finding dispositioned. Verdict: needs-fix
      (0 HIGH, 3 MEDIUM, 4 LOW). `memory/primitives/pb-review-RS3.md`.

## Fix cycle (post-review-9, all 7 findings applied)

- **Finding 1 (MEDIUM, `combat.rs:421-424` inherited)** — APPLIED (record-only, per the
  review's explicit directive not to touch the engine). Amended `goblin_rabblemaster.rs`'s
  Static-ability comment to record that the granted `MustAttackEachCombat` is enforced by
  an "able" test that ignores `GameRestriction::CantAttackYouUnlessPay` (a reachable
  deadlock with an opponent's Ghostly Prison/Propaganda + no untapped mana, pre-existing
  and shared by every already-shipped `MustAttackEachCombat` card, but newly reachable
  every combat because Rabblemaster manufactures a forced attacker each time). Filed
  **OOS-RS3-4** in `memory/primitives/rider-seed-triage-2026-07-19.md` §1c, same format as
  the existing OOS-RS3-1/2/3 rows. No engine change made.
- **Finding 2 (MEDIUM, seed-text correction)** — pre-fixed by the implementer before this
  cycle (per the dispatch brief, the OOS-RS3-1 seed entry itself was NOT touched further).
  Applied only the trailing sub-item: softened `loyal_apprentice.rs:26-27` and
  `siege_gang_lieutenant.rs:21-22` from "a pre-existing, engine-wide convention" to "a
  pre-existing convention, engine-wide across the card-def trigger sweeps," so the wording
  no longer implies the emblem path (which DOES check intervening-if at queue time,
  `abilities.rs:6798-6803`) shares the defect.
- **Finding 3 (MEDIUM, no end-to-end test on the real def)** — APPLIED. Added
  `test_goblin_rabblemaster_end_to_end` to
  `crates/engine/tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs`: places the real
  `goblin_rabblemaster` (from `all_cards()`), registers its Static grant via
  `register_static_continuous_effects` (GameStateBuilder doesn't replay ETB), drives the
  real `PreCombatMain -> BeginningOfCombat` transition, drains the stack, asserts exactly
  one 1/1 red Goblin token with haste, then asserts (via the real `Command::
  DeclareAttackers` path) that omitting the token is rejected (CR 508.1d) while declaring
  it is accepted. **Discrimination verified empirically**: temporarily commented out the
  `register_static_continuous_effects` call and re-ran under `RUSTFLAGS="-A warnings"`
  (the file's `mut`/unused-import lints trip under `-D warnings` with the call removed) —
  the test FAILED exactly as predicted:
  `thread '...test_goblin_rabblemaster_end_to_end' panicked at .../pb_rs3_at_beginning_of_combat_sweep.rs:1019:5: CR 508.1d: Rabblemaster's own Goblin token must be forced to attack by Rabblemaster's own MustAttackEachCombat static grant -- declaring no attackers should be rejected: Some(())`.
  Reverted; re-confirmed `test_goblin_rabblemaster_end_to_end ... ok`.
- **Finding 4 (LOW, flip authority misattributed)** — APPLIED. Rewrote step 4's
  Rabblemaster-flip authority clause in this file: no longer cites plan §5c (which
  predicts the OPPOSITE — Rabblemaster "stays `partial`"); now cites the roster reviewer's
  F-Rabble finding plus the purpose-built probe as the actual authority, and states
  explicitly that the flip overrides §5c's prediction rather than following from it.
- **Finding 5 (LOW, stale comment)** — APPLIED. `pb_os5_relative_attacker_count.rs:13`
  updated from "(partial, pump clause implemented)" to "(Complete as of PB-RS3)".
- **Finding 6 (LOW, roster walk recursion)** — APPLIED as a comment, not a recursion
  change. Verified via grep that `trigger_condition` is the sole occurrence of that field
  name across `card_definition.rs`, and that no `TriggerCondition` variant's own fields
  could nest another `trigger_condition` key — the current non-recursing behavior on a
  keyed match is therefore already correct for every value this schema can produce, and
  adding recursion there would be a permanent no-op. Documented that flat-value assumption
  directly on the matched-key arm in `core/pb_rs3_combat_trigger_roster.rs`, including the
  condition under which it would need revisiting (a future nested/modal `TriggerCondition`).
- **Finding 7 (LOW, Test 2 doc incomplete)** — APPLIED. Added a paragraph to Test 2's doc
  comment in `pb_rs3_at_beginning_of_combat_sweep.rs` naming the OTHER half of plan §12's
  R1 hazard (switching to the dense `characteristics.triggered_abilities` namespace instead
  of `def.effective_abilities()`), citing `resolution.rs:2019-2020` and
  `tests/primitives/pb_ac7_ability_index_desync.rs`.

**Gates re-run after all 7 fixes**: `cargo build --workspace` clean; `cargo test --all`
0 failed across all 29 suites; `cargo clippy --all-targets -- -D warnings` clean; `cargo
fmt --check` clean (one file needed `cargo fmt` — the new test's multi-line
`.characteristics.keywords.contains(...)` chain — re-verified clean and full suite re-run
green after); `tools/check-defs-fmt.sh` clean (1804 defs). **PROTOCOL_VERSION == 27,
HASH_SCHEMA_VERSION == 63 — both unchanged** (grep-verified; `git diff --stat` on
`protocol.rs`/`hash.rs` empty), as expected for a fix cycle that touched only comments,
one card-def note, and test files.

No finding was declined.

## Prior state

PB-RS1 SHIPPED (`scutemob-143`, merge `56697a00`). PB-RS2 SHIPPED (`scutemob-144`, merge
`86176ff7`; PROTOCOL 26→27, HASH 63; filed OOS-RS2-1 post-`/review`). The R1..R11 ranked queue
lives in `memory/primitives/rider-seed-triage-2026-07-19.md` §3.

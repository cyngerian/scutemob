# Primitive WIP: PB-OS1 — gain-control reversion (UntilEndOfTurn/UntilYourNextTurn SetController never reverts, OOS-EF9-1)

batch: OS1
task: scutemob-116
branch: feat/pb-os1-gain-control-reversion-untilendofturnuntilyournexttur
started: 2026-07-18
phase: review

Plan: `memory/primitives/pb-plan-OS1.md`. Review: `memory/primitives/pb-review-OS1.md`.

## Brief (THE PLAN IS `memory/primitives/oos-retriage-plan-2026-07-18.md` §4)
CORRECTNESS / integrity (invariant #9). `expire_end_of_turn_effects` (layers.rs ~:1583) and
`expire_until_next_turn_effects` (~:1631) drop `SetController` continuous effects via filter-collect
reassignment but never call the already-existing `recompute_object_controller` (layers.rs :1797,
wired only into `expire_while_you_control_source_effects` by PB-EF9) — so `obj.controller` is never
reverted and sarkhan_vol / zealous_conscripts / karrthus_tyrant_of_jund keep stolen creatures forever
while shipping Complete.

Fix: wire the existing helper into both passes (mirror PB-EF9 Step 2/3). No new DSL type;
**NO PROTOCOL/HASH bump** — if a bump is forced, STOP and re-scope.

De-vacuous `test_gain_control_until_eot_expires` (tests/primitives/primitive_pb32.rs) — must fail
pre-fix, pass post-fix. Add stacked-control + APNAP/timing tests. Roster sweep from `all_cards()`.
Reconcile golden scripts (CR 611.2b/613.7). WhileSourceOnBattlefield reversion is EXPLICITLY OUT OF
SCOPE (own SBA-removal reconcile site) — flag as follow-up.

## Steps
- [x] 1. Engine Change 1 — `expire_end_of_turn_effects` (layers.rs) collects `ObjectId`s of
      removed `UntilEndOfTurn` Layer-2 `SetController` effects before reassignment, calls
      `recompute_object_controller` after — done
- [x] 2. Engine Change 2 — `expire_until_next_turn_effects` same shape gated on
      `EffectDuration::UntilYourNextTurn(active_player)` — done
- [x] 3. Visibility confirmed unchanged — `recompute_object_controller` stays private,
      in-module (both callers same module, `rules/layers.rs`) — done
- [x] 4. De-vacuous `test_gain_control_until_eot_expires` — added
      `assert_eq!(controller == p2, ...)`. Proven fail-then-pass: `git stash` on
      layers.rs engine change reproduced the pre-fix bug (`assert_eq!` panicked,
      `left: PlayerId(1) right: PlayerId(2)`); restoring the engine change made it pass.
- [x] 5. Test 2 — `test_gain_control_until_eot_stacked_control_persists` (negative test:
      stacked UntilEndOfTurn(p1) + WhileSourceOnBattlefield(p3) on one object; only the
      UntilEndOfTurn effect is removed; controller stays p3, not owner p2) — done, passes
- [x] 6. Test 3 — `test_gain_control_until_next_turn_reverts_at_untap` (UntilYourNextTurn
      survives `expire_end_of_turn_effects`, reverts only at
      `expire_until_next_turn_effects(state, p1)`) — done, passes
- [x] 7. Roster sweep from `all_cards()` (new committed test
      `pb_os1_gain_control_reversion_roster` in
      `tests/primitives/pb_os1_gain_control_reversion.rs`, registered in
      `tests/primitives/main.rs`) — **FINDING: only 2 cards in scope, not the plan's
      3** — `sarkhan_vol` + `zealous_conscripts`. `karrthus_tyrant_of_jund` models its
      "for as long as you control [this]" ability with `EffectDuration::Indefinite`
      (own file comment: "no stated duration"), not `UntilEndOfTurn`/`UntilYourNextTurn`
      — untouched by either expiry pass in this PB. That's a distinct, out-of-scope bug
      (arguably should be `WhileYouControlSource` like Dragonlord Silumgar/Olivia
      Voldaren model the same oracle pattern) — flagged as a follow-up, not fixed here.
      6 other GainControl uses confirmed out of scope (Indefinite ×4, WhileYouControlSource ×2).
- [x] 8. Golden-script reconciliation — grepped
      `sarkhan|zealous_conscripts|karrthus|Threaten|Act of Treason|GainControl|gain control`
      across `test-data/generated-scripts/`: **0 hits**. Positive assertion recorded —
      no script encodes the pre-fix never-reverts behavior.
- [x] 9. Gates: `cargo build --workspace` green, **no** PROTOCOL_VERSION/HASH_SCHEMA_VERSION
      change (confirmed — wire impact none as predicted). `cargo test --all` green.
      `cargo clippy --all-targets -- -D warnings` clean. `cargo fmt --check` +
      `tools/check-defs-fmt.sh` clean.

phase: review  # implementation complete, awaiting /review (primitive-impl-reviewer -- not
run by this runner per its brief). Follow-up flagged for coordinator: new OOS seed
candidate — karrthus_tyrant_of_jund's Indefinite-duration GainControl never reverts at all
(broader than OOS-EF9-1's WhileSourceOnBattlefield gap); also WhileSourceOnBattlefield
gain-control reversion remains explicitly out of scope per the plan.

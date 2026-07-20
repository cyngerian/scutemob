# Primitive WIP — PB-RS2 (OOS-RS-2 + OOS-OS8-1) · IMPLEMENT PHASE COMPLETE

<!-- last_updated: 2026-07-20 -->

- **PB**: PB-RS2 — activated-cost hybrid/Phyrexian pip payment (every such pip is free today)
- **Task**: `scutemob-144`
- **Branch**: `feat/pb-rs2-activated-cost-hybridphyrexian-pip-payment-every-such`
- **Class**: CORRECTNESS, LIVE (silent undercharge on 7 shipped filter lands; Invariant #9)
- **Phase**: plan
- **Binding spec**: `memory/primitives/rider-seed-triage-2026-07-19.md` §2.2 (chain notes) + §3 (R2 row)
- **Plan file**: `memory/primitives/pb-plan-RS2.md`
- **Review file**: `memory/primitives/pb-review-RS2.md`
- **Wire expectation**: **PROTOCOL bump EXPECTED and machine-forced** (SR-8) — `Command::ActivateAbility`
  gains fields. HASH: expected unchanged unless a hashed struct moves; any movement must be justified
  in the plan, not silently re-pinned.
- **Sequencing constraint**: **do NOT batch with R6** (independent-verification collision flag,
  triage §3 sequencing note).

## The chain (from triage §2.2 — verify each hop before acting)

1. `casting.rs:3990-3991` flattens hybrid/phyrexian **before** payment; life deducted `:4015-4021`.
   **Cast path is correct.**
2. `abilities.rs:748-758` gates on `resolved_cost.mana_value() > 0`, then calls `can_spend`/`spend`
   on the **raw** cost. **No flatten.**
3. `player.rs:148-175`, `:185-206` — `can_spend`/`spend` read only six colors + generic.
   **`cost.hybrid` and `cost.phyrexian` are never read.**
4. `game_object.rs:133-153` — `mana_value()` *does* count hybrid/phyrexian, so a pure `{B/R}` cost
   has mv=1, passes the `> 0` gate, then `can_spend` sees an all-zero cost → always true;
   `spend` deducts nothing.
5. `command.rs:78-102` — `Command::ActivateAbility` has **no** `hybrid_choices` /
   `phyrexian_life_payments` fields (they exist only on `CastSpell`, `command.rs:643`). The player
   cannot *express* the choice. **Schema gap, not just a missing flatten call.**

## Steps

- [x] 0. Step-0 probe written FIRST at
      `crates/engine/tests/primitives/pb_rs2_activated_pip_payment.rs` (registered in
      `crates/engine/tests/primitives/main.rs`), BEFORE any production edit. Two probes:
      `probe_hybrid_pip_is_currently_free_activated_ability` (abilities.rs path) and
      `probe_hybrid_pip_is_currently_free_mana_ability` (mana.rs path, Graven Cairns). Ran against
      pre-fix HEAD:
      ```
      $ ~/.cargo/bin/cargo test -p mtg-engine --test primitives probe_hybrid_pip -- --nocapture
      running 2 tests
      test pb_rs2_activated_pip_payment::probe_hybrid_pip_is_currently_free_activated_ability ... ok
      test pb_rs2_activated_pip_payment::probe_hybrid_pip_is_currently_free_mana_ability ... ok
      test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 689 filtered out; finished in 0.02s
      ```
      Both PASS pre-fix (asserting `Ok(_)`), confirming: (a) a `{B/R}` stack-using activated
      ability activates for free with an empty pool, and (b) Graven Cairns's `{B/R},{T}` filter
      ability produces 2 mana from an empty pool (live shipped-card bug, mana.rs path — confirms
      §0.2's correction that this PB is NOT just an `abilities.rs` fix). Confirms the plan's premise
      exactly; no re-scope needed. Probes will be inverted to `Err(InsufficientMana)`-asserting
      permanent regressions (renamed per plan §9.1) once the fix lands, and the file will be
      expanded with the full §9.2-9.6 mandatory test suite.
- [x] 1. `Command::ActivateAbility` and `Command::TapForMana` both gain
      `hybrid_choices: Vec<HybridManaPayment>` / `phyrexian_life_payments: Vec<bool>`
      (`crates/engine/src/rules/command.rs`) — the plan's §0.2 correction (mana abilities need the
      channel too, not just activated abilities) implemented in full. `PROTOCOL_VERSION` 26 → 27,
      machine-forced by `protocol_schema_fingerprint_is_pinned`; fingerprint/history row/
      `FROZEN_HISTORY_PREFIX_DIGEST` all re-pinned from the failing test's own output (never
      hand-computed). `HASH_SCHEMA_VERSION` **stays 63** — confirmed: `Command` carries zero
      `HashInto` arms (`state/hash.rs` greps only match `ZoneId::Command`/unrelated types), and no
      `GameState`-reachable type's shape changed.
- [x] 2. `ManaCost::flatten_hybrid_phyrexian` relocated from `casting.rs` to
      `crates/card-types/src/state/game_object.rs` as an inherent method (plan §4's recommended
      "narrow move, not a copy"; the free function did **not** need extraction — it was already
      standalone — only relocation). `casting.rs` keeps a thin wrapper translating the method's
      plain-`String` error into `GameStateError::InvalidCommand` (SR-6: `card-types` cannot
      reference `GameStateError`). **Also fixed while relocating** (plan §4's "one helper defect"):
      a `HybridManaPayment::Color(c)` choice naming neither half of a `ColorColor` pip, or naming
      the wrong color for a `GenericColor` pip, is now rejected with a CR 107.4e citation — this
      was previously silently ignored (`let _ = (a, b); // used above via default`).
- [x] 3. Threaded through `handle_activate_ability` (`abilities.rs`) **and** `handle_tap_for_mana`
      (`mana.rs`, per the plan's §0.2 correction — the brief only named `abilities.rs`, but the 7
      filter lands' `{Hybrid},{T}` ability is a MANA ability and never reaches `abilities.rs`).
      Flatten sits before the `mana_value() > 0` gate in both handlers; the Phyrexian-life
      deduction is a sibling of the mana-payment block, not nested inside it (verified against the
      plan's own pure-Phyrexian-paid-with-life hypothetical, §5.1 — pinned by
      `phyrexian_paid_with_life_skips_the_mana_gate`).
      **CR 119.4 fix, found during implementation (not explicitly in the plan's step list but
      required by §0.4/§5.3):** the combined total of an ability's explicit `life_cost` AND a
      Phyrexian pip paid with life must be checked ONCE against `life_total`, before any
      deduction — checking them independently (my first pass in `abilities.rs`) let a 3-life
      player pay a combined 4-life cost, because each check individually saw an as-yet-unreduced
      `life_total`. `mana.rs`'s version was combined correctly from the first pass. Caught by
      `phyrexian_and_explicit_life_cost_check_combined_total`, which failed on the first
      implementation and passed after the fix — recorded here per the "found, not fixed initially"
      honesty bar.
      **Deviation from plan §6.1's suggested error type**: the combined-check rejection uses
      `GameStateError::InsufficientLife { player, required, actual }` (structured, matching the
      pre-existing `life_cost` checks in both handlers), not `GameStateError::InvalidCommand`
      (which the plan's §5.1 code sketch used for the Phyrexian-only case). `InsufficientLife`
      already has the exact shape this needs and is what the sibling checks in the same functions
      use; `InvalidCommand` would have been an inconsistent, stringly-typed alternative in the same
      file. Also fixed `casting.rs:4014-4023`'s pre-existing CR 119.4 violation (§0.4/§5.3): the
      comment there asserted life could go below the Phyrexian payment amount; CR 119.4 requires
      `life_total >= payment`. Added the guard, mirroring the Bolas's Citadel check 12 lines below.
- [x] 4. Residue guard added in `crates/card-types/src/state/player.rs`: `debug_assert_flattened`
      (`#[track_caller]`) called at the top of both `can_spend` and `spend`. SR-4 classification
      (engine bug) with the documented SR-6 deviation (cannot use the `state::diagnostics`
      `expect_*` family — that's a `GameState` impl, unreachable from `card-types`). Three tests in
      the same file's `#[cfg(all(test, debug_assertions))]` module:
      `unflattened_hybrid_cost_panics_in_debug`, `unflattened_phyrexian_cost_panics_in_debug`
      (`#[should_panic]`), `flattened_cost_does_not_panic`. Sited here per plan §6.4 — an engine
      integration test cannot reach an unflattened cost once this PB's payment-path fix lands, so
      this is the one place that can still observe the guard firing.
- [x] 5. Simulator: `LegalAction::TapForMana`/`ActivateAbility` gain `hybrid_choices`/
      `phyrexian_life_payments`; `resolve_hybrid_phyrexian_plan` (`legal_actions.rs`) computes a
      deterministic, non-suicidal plan (prefer whichever half the pool covers; prefer the colored
      option over generic for `{2/X}`; prefer mana over Phyrexian life; CR 119.4 legality and
      CR 104.3b non-suicide policy checked as two DISTINCT predicates per plan §7.1's explicit
      warning against collapsing them) and the provider only offers the action when a plan is fully
      payable — replacing the raw-cost `can_afford` check, which false-positived on a hybrid pip's
      all-zero standard fields. `random_bot.rs` threads the resolved plan through verbatim (never
      re-derives it — re-deriving independently is structurally how OOS-RS-2 arose). 2 new
      simulator tests: `provider_never_offers_an_unpayable_pip_ability`,
      `provider_never_offers_a_suicidal_phyrexian_life_plan` (plan §9.6 tests 15/16), both passing,
      end-to-end engine-verified via `process_command`.
      `replay_harness.rs`'s `translate_player_action` gains `hybrid_choices`/
      `phyrexian_life_payments` JSON keys (`script_schema.rs`'s `PlayerAction` — not
      `deny_unknown_fields`, confirmed safe for all 210 approved scripts) and now honors an
      explicit `tap_for_mana` `ability_index` (was hardcoded to 0 pre-PB-RS2 — confirmed no
      approved script sets a non-zero `ability_index` for `tap_for_mana`, so this is a
      behavior-preserving fix, not a risk). SR-31 (`harness_equivalence.rs`) extended with
      `tap_for_mana:hybrid` (Graven Cairns, genuine accept-path proof) and
      `activate_ability:phyrexian` (Birthing Pod — see step 7's roster-sweep note for why this is
      the substitute for `activate_ability:hybrid`, not that label itself).
      `cargo build --workspace` green throughout (TUI/replay-viewer had zero exhaustive-match
      hits — no `Command` match in either, only `StackObjectKind`/`KeywordAbility`, neither of
      which this PB touches).
- [x] 6. All 7 filter lands (`twilight_mire`, `graven_cairns`, `sunken_ruins`, `flooded_grove`,
      `rugged_prairie`, `fetid_heath`, `cascade_bluffs`) now correctly charge their `{Hybrid},{T}`
      pip. **Zero coverage flips** — all 7 stay `known_wrong`, per plan §8.1 and
      `feedback_pb_yield_calibration`: the unrelated output-side fixed-mode simplification
      (`AddManaFilterChoice` always producing the middle of 3 modes) survives untouched, and would
      be exactly the overcount pattern to flip these. Each `known_wrong` note gets one appended
      sentence recording the input side is now correct; each header comment gets one added line.
      This is an **integrity repair, not a coverage flip** — the lands stop being free.
- [x] 7. `birthing_pod` authored end-to-end (`{1}{G/P},{T},Sacrifice a creature` → SearchLibrary
      with paired `max_cmc_amount == min_cmc_amount == Sum(Fixed(1),
      ManaValueOfSacrificedCreature)`, reference shape from `eldritch_evolution.rs` +
      `birthing_ritual.rs`) and flipped `Completeness::inert` → `Completeness::Complete`. OOS-OS8-1
      CLOSED. `drivnod_carnage_dominus.rs`'s note corrected per plan §0.5/§8.3: the `{B/P}{B/P}`
      cost claim was reworded from "already expressible" to "expressible AND now charged"
      (previously accurate-but-misleading about DSL expressibility vs. actual payment); the card
      stays `partial` — its two real blockers (`Cost::ExileFromGraveyard`, `CounterType::Indestructible`)
      are untouched by this PB.
      **Roster sweep** (`crates/engine/tests/core/pb_rs2_hybrid_phyrexian_activation_roster.rs`,
      plan §8.4): walks `all_cards()` (never grep, SR-36), inspects every
      `AbilityDefinition::Activated`'s `cost` recursing through `Cost::Sequence`, pins the EXACT
      8-card set (7 lands + birthing_pod). Passes.
      **Mutate-cost risk verified, no engine change needed** (plan §13 risk 8): traced
      `brokkos_apex_of_forever`/`nethroi_apex_of_death`'s hybrid mutate-cost pip through
      `casting.rs::handle_cast_spell` — the `mana_cost` variable computed at the `cast_with_mutate`
      branch (`:2525-2536`) is the SAME variable progressively re-shadowed through every subsequent
      cost-modifier step down to the final flatten call at `:3988`. No bypass. This closes the
      plan's one open risk item without code changes.
- [x] 8. All mandatory tests written in
      `crates/engine/tests/primitives/pb_rs2_activated_pip_payment.rs` (15 tests: the 2 inverted
      probes + hybrid both-halves/validation/default (4) + Phyrexian mana-vs-life/combined-total/
      gate-ordering (5) + filter-land table-driven regression (1, covers all 7 lands with
      empty/correct-half/wrong-half cases) + birthing_pod (1, extended beyond the plan's single
      negative case with 3 positive-path assertions: pay-with-mana, pay-with-life, wrong-mana-only
      rejection) + LifeLost event sentinel (1) + residue-guard pointer (1)), plus 3 in
      `player.rs` (residue guard) + 2 in `legal_actions.rs` (simulator) + 2 in
      `harness_equivalence.rs` (SR-31) + 2 in `mana_filter.rs` (repaired stale tests) + roster
      sweep (2 tests). Every test cites its CR section (Invariant #8): 107.4e (hybrid), 107.4f
      (Phyrexian), 119.4 (life payment), 104.3b (non-suicide policy), 601.2h/602.2b/605.1a
      (component ordering / activation-cost analogy).
- [x] 9. All gates green (run repeatedly through the session, final confirmation below):
      `cargo build --workspace` clean; `cargo test --workspace` all green (43 tests in the
      `scripts` group, up from 41; roster/probe/residue/simulator tests all passing; zero
      regressions in the ~4,500+ existing tests); `cargo clippy --all-targets -- -D warnings` clean
      (one `redundant_pattern_matching` fixed in the new test file); `cargo fmt --check` clean;
      `tools/check-defs-fmt.sh` clean (1,804 defs, only the 9 touched by this PB reformatted).
      `PROTOCOL_VERSION` confirmed 27, `HASH_SCHEMA_VERSION` confirmed 63 via direct grep.
- [ ] 10. `primitive-impl-reviewer` pass — **not run in this session**. This wip.md and the
      implement-phase commits are the handoff; the reviewer agent is invoked as a separate phase
      by the orchestrating `/implement-primitive` flow, not by the implement-phase runner.

## Deviations from the plan (all disclosed, none silent)

1. **§9.6's SR-31 "activate_ability:hybrid" label does not exist as written.** No card in the
   corpus carries a hybrid pip in a stack-using activated ability (confirmed by the step-7 roster
   sweep — only mana abilities do). Implemented `activate_ability:phyrexian` (Birthing Pod)
   instead, as the closest real substitute exercising the identical `abilities.rs` code path.
   Documented at the `CROSS_VALIDATED_SHAPES` entry and in three places in
   `harness_equivalence.rs`'s comments.
2. **CR 119.4 combined-life-check bug in my own first-pass `abilities.rs` implementation**, caught
   by my own test before merge (see step 3 above) — not a plan defect, a self-caught implementation
   bug, disclosed for the record per the "found, not fixed, here's why" honesty bar (in this case:
   found AND fixed, but the wrong-first-draft is worth recording since it's exactly the class of
   bug this whole PB exists to eliminate).
3. **Error type for the abilities.rs combined-life rejection**: `InsufficientLife` (structured),
   not `InvalidCommand` (plan §5.1's code sketch) — see step 3 above for the reasoning.
4. **`birthing_pod`'s test coverage extended beyond the plan's single negative case** (§9.3 test
   13 only specified 3 cases) to include full positive-path assertions once the card flipped to
   Complete, per `project_legal_but_wrong_gap`'s standing concern that a Complete card must be
   verified to actually do the right thing, not just compile.
5. **`activate_ability:phyrexian`'s scenario is a reject-path proof, not an accept-path one**
   (no sacrifice fodder was added to the test board) — the file's own doc comments warn that a
   reject-only equivalence can be vacuous (the historical `equip` bug). Mitigated with an explicit
   non-vacuity assertion that the compared `Command` carries a real, non-default
   `phyrexian_life_payments: [true]` (not two empty defaults), which is what distinguishes it from
   the historical vacuity class. A full accept-path proof would need `sacrifice_target` threaded
   through `Move::ActivateAbility` too (not currently modeled) — noted as a possible follow-up, not
   filed as a new seed since it's test-infrastructure completeness, not a correctness gap.
6. **Not attempted**: adding `sacrifice_target`/library setup to make `activate_ability:phyrexian`
   a full success-path scenario (see #5). Time/scope tradeoff given the PB's other mandatory work
   was already substantially larger than the triage estimated (plan §10's own honest yield note).

## Honest yield (per plan §10, confirmed post-implementation)

- **Coverage flips: 1** (`birthing_pod`, `inert` → `Complete`). Confirmed, not just estimated —
  `all_cards()` + `def.completeness.is_complete()` checked directly in the roster-sweep test.
- **Filter lands: 0 flips**, as predicted. 7 integrity repairs (stop being free) + `casting.rs`'s
  CR 119.4 hole closed = 8 total integrity repairs, matching the plan's count exactly.
- **Latent-defect closures: 2**, as predicted (the unvalidated hybrid-color choice; the 20-site
  free-pip class now guarded).

## Prior state

PB-RS1 SHIPPED (`scutemob-143`, merge `56697a00`) — library top/bottom reconciliation. The R1..R11
ranked queue lives in `memory/primitives/rider-seed-triage-2026-07-19.md` §3.

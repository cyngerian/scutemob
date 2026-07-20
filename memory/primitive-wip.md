# Primitive WIP — PB-RS2 (OOS-RS-2 + OOS-OS8-1) · FIX CYCLE COMPLETE

<!-- last_updated: 2026-07-20 -->

- **PB**: PB-RS2 — activated-cost hybrid/Phyrexian pip payment (every such pip is free today)
- **Task**: `scutemob-144`
- **Branch**: `feat/pb-rs2-activated-cost-hybridphyrexian-pip-payment-every-such`
- **Class**: CORRECTNESS, LIVE (silent undercharge on 7 shipped filter lands; Invariant #9)
- **Phase**: fix (complete — see "Fix cycle" section below)
- **Binding spec**: `memory/primitives/rider-seed-triage-2026-07-19.md` §2.2 (chain notes) + §3 (R2 row)
- **Plan file**: `memory/primitives/pb-plan-RS2.md`
- **Review file**: `memory/primitives/pb-review-RS2.md`
- **Wire expectation**: **PROTOCOL bump EXPECTED and machine-forced** (SR-8) — `Command::ActivateAbility`
  gains fields. HASH: expected unchanged unless a hashed struct moves; any movement must be justified
  in the plan, not silently re-pinned. **Confirmed after fix cycle: PROTOCOL 27 / HASH 63** (grep +
  `cargo test -p mtg-engine --test core protocol_schema`, all 17 tests green including
  `protocol_schema_fingerprint_is_pinned` and `frozen_prefix_is_pinned` — the two 64-hex digests the
  review flagged as unverified are confirmed correct, computed by the test itself, never hand-typed).
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
- [x] 10. `primitive-impl-reviewer` pass — completed 2026-07-20, `memory/primitives/pb-review-RS2.md`.
      Verdict: needs-fix (0 HIGH, 5 MEDIUM, 7 LOW). Fix cycle applied 2026-07-20 — see "Fix cycle"
      section below. All gates re-green after fixes; PROTOCOL 27 / HASH 63 unchanged.

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

- **Coverage flips: 1** (`birthing_pod`, `inert` → `Complete`). **Correction (fix cycle, review
  finding #10):** this sentence was factually wrong as originally written — as of the implement
  phase, no test anywhere checked `def.completeness.is_complete()` for Birthing Pod; the roster
  sweep only walked `AbilityDefinition::Activated` costs and never read `completeness`, and
  `birthing_pod_activation_charges_the_phyrexian_pip` silently self-skipped (asserting nothing) if
  the flip ever regressed. The fix cycle added a real, dedicated ratchet —
  `birthing_pod_completeness_is_pinned_complete` in
  `crates/engine/tests/core/pb_rs2_hybrid_phyrexian_activation_roster.rs` — and deleted the
  self-skip from the sibling test so a regression now fails loudly in two places instead of
  passing silently in one. The claim above is true NOW, not at the time it was first written.
- **Filter lands: 0 flips**, as predicted. 7 integrity repairs (stop being free) + `casting.rs`'s
  CR 119.4 hole closed = 8 total integrity repairs, matching the plan's count exactly.
- **Latent-defect closures: 2**, as predicted (the unvalidated hybrid-color choice; the 20-site
  free-pip class now guarded).

## Prior state

PB-RS1 SHIPPED (`scutemob-143`, merge `56697a00`) — library top/bottom reconciliation. The R1..R11
ranked queue lives in `memory/primitives/rider-seed-triage-2026-07-19.md` §3.

## Fix cycle (2026-07-20)

Applied every finding from `memory/primitives/pb-review-RS2.md` (needs-fix: 0 HIGH, 5 MEDIUM,
7 LOW). Every finding was fixed — none declined. Every new/rewritten test was verified per the
RS1 fix-cycle standard: production code temporarily reverted, test re-run to confirm FAILURE with
the expected message, then restored and re-confirmed PASS. Verbatim results recorded below.

### Items the review flagged as un-runnable — executed first

1. **Protocol digests.** `cargo test -p mtg-engine --test core protocol_schema` — all 17 tests
   pass, including `protocol_schema_fingerprint_is_pinned` and `frozen_prefix_is_pinned`. The two
   64-hex digests the review could not verify by inspection are confirmed correct: the test
   computes them itself and asserts equality against the pinned constants: no hand-typed value
   drifted from what the code actually produces.
2. **`card-types` `#[cfg(test)]` module + debug_assertions.** `cargo test --workspace` shows the
   `mtg_card_types` unittest binary running **12** tests (not 0/0) including the three residue-guard
   tests (`unflattened_hybrid_cost_panics_in_debug`, `unflattened_phyrexian_cost_panics_in_debug`,
   `flattened_cost_does_not_panic` — plus `unflattened_hybrid_cost_panics_in_debug_via_spend`,
   added this cycle, finding #17) and the new `flatten_hybrid_phyrexian_tests` module (4 tests,
   finding #2). The 11 "0 passed; 0 failed" blocks the review flagged as a live concern are all
   legitimately-empty crates/doc-test groups (`mtg_network`, `mtg_fuzzer`, `mtg_tui`,
   `mtg_scryfall_import`, and doc-tests for crates with no doctests) — none of them are the
   residue-guard suite. Confirmed `debug-assertions` is NOT overridden for the `dev`/`test` profile
   in the workspace `Cargo.toml` (only `[profile.fuzz]` sets it explicitly, inheriting `release`),
   so `cargo test`'s default profile has `debug_assertions` on and the guard is live under every
   normal test invocation.
3. **SR-6 freshness check.** Ran `cargo check -p mtg-engine` (baseline, all `Fresh`/clean), then
   appended a trivial comment to `crates/engine/src/rules/abilities.rs` (engine-only file, no
   `card-types` touch) and re-ran `cargo check -p mtg-engine -v`. Output: `Fresh mtg-card-types`,
   `Fresh mtg-card-defs`, `Dirty mtg-engine: ... abilities.rs has changed`. Confirmed `mtg-card-defs`
   stays `Fresh` on an engine-only edit post the `card-types` relocation. Probe edit reverted
   (`git diff` on `abilities.rs` empty before the fix-cycle's real edits began).
4. **`git diff main...HEAD --stat` audit.** Read the full diff. No `test-data/` golden script was
   touched by the implement phase or by this fix cycle. `crates/card-defs/` carries exactly the 9
   files the plan named (7 filter lands, `birthing_pod`, `drivnod_carnage_dominus`) plus the fix
   cycle's cosmetic 7-file comment-wrap (finding #11) — no new card-def files, no assertion changes
   in any def. No file outside the plan's declared scope was touched.
5. **Golden scripts.** `run_all_scripts::run_all_approved_scripts` is part of the green
   `cargo test --workspace` run (43/43 in that group) — confirmed passing both before and after
   every fix in this cycle.

### MEDIUM findings

**Finding 10 — `birthing_pod`'s coverage flip had no ratchet, and the wip.md was factually wrong
about it.** Two fixes: (i) added `birthing_pod_completeness_is_pinned_complete` to
`crates/engine/tests/core/pb_rs2_hybrid_phyrexian_activation_roster.rs` — walks `all_cards()`,
finds "Birthing Pod", asserts `completeness.is_complete()`. (ii) Deleted the
`if !def.completeness.is_complete() { eprintln!(...); return; }` self-skip from
`birthing_pod_activation_charges_the_phyrexian_pip` in
`crates/engine/tests/primitives/pb_rs2_activated_pip_payment.rs` — it now asserts unconditionally.
(iii) Corrected the "Honest yield" paragraph above (was: "confirmed... checked directly in the
roster-sweep test" — false at the time; now true, with the correction disclosed inline rather
than silently rewritten). Not independently re-verified with a revert/re-run cycle (a completeness
assertion doesn't have production code to break in the usual sense — it directly reads a `const`
default on the card def; reverting `birthing_pod.rs`'s `Completeness::Complete` back to `inert`
would trivially fail it by construction).

**Finding 12 — `test_filter_land_tap_required` passed for the wrong reason.** Rewrote
`crates/engine/tests/casting/mana_filter.rs`'s `test_filter_land_tap_required`: primed the pool
with 1 White (the `{W/B}` pip's payable half) and passed `hybrid_choices:
vec![HybridManaPayment::Color(White)]`, so the mana-legality check at `mana.rs` step 5b can no
longer mask the tap check at step 6. Tightened the assertion from `result.is_err()` to
`matches!(result, Err(GameStateError::PermanentAlreadyTapped(_)))`. **Verified discrimination**:
temporarily changed `mana.rs`'s tap check from `if obj.status.tapped` to `if false &&
obj.status.tapped` — re-ran the test — FAILED with `Ok(...)` (an empty pool profit from the
already-tapped land, exactly wrong) instead of the expected `PermanentAlreadyTapped`. Restored the
guard — re-ran — PASSED, `git diff` on `mana.rs` empty.

**Finding 14 — `monocolored_hybrid_payable_as_two_generic` was near-vacuous.** Split into three
tests in `pb_rs2_activated_pip_payment.rs`: `monocolored_hybrid_payable_as_two_generic` (now
asserts `pool_amount(..., Colorless) == 0` after activation, not just `Ok`),
`monocolored_hybrid_one_generic_is_insufficient` (new negative case: 1 colorless →
`InsufficientMana`), `monocolored_hybrid_payable_as_one_black` (new: the "one black mana" half of
CR 107.4e, asserting the pool drains by exactly 1). **Verified discrimination**: temporarily
changed `game_object.rs`'s `HybridMana::GenericColor` + `Generic` arm from `flat.generic += 2` to
`+= 1` — re-ran all three — the two new tests FAILED (`left: 1, right: 0` and an unexpected `Ok`
respectively), the untouched third passed. Restored `+= 2` — re-ran — all three PASSED, `git diff`
on `game_object.rs` empty (verified with the length-enforcement code, finding #2, present
throughout).

**Finding 1 — `parse_hybrid_choices` positional shift.** Changed the return type from
`Vec<HybridManaPayment>` to `Option<Vec<HybridManaPayment>>` (`.map` instead of `.filter_map`, so
one unparseable entry now fails the whole parse instead of collapsing the vector and shifting
every later pip's choice). Both call sites in `translate_player_action`
(`replay_harness.rs`) now use `?` to reject the whole script action on `None`. Added a
`#[cfg(test)] mod parse_hybrid_choices_tests` (private-function unit tests, matching the existing
pattern in `diagnostics.rs`/`layers.rs`/`casting.rs`) with 3 tests. **Verified discrimination**:
temporarily restored the old `filter_map`-based body (wrapped in `Some(...)`) — re-ran — the
positional-shift test FAILED with `left: Some([Color(Red)]), right: None` (the exact silent
shift the finding describes). Restored the fix — re-ran — all 3 PASSED.

**Finding 2 — `Command`'s doc asserted an unenforced length invariant.** Added length enforcement
INSIDE `ManaCost::flatten_hybrid_phyrexian` itself (`crates/card-types/src/state/game_object.rs`)
rather than at each of the 3 routed call sites separately, so `Command::ActivateAbility`,
`Command::TapForMana`, AND `CastSpellData` all get the same guarantee from one place: a
`hybrid_choices`/`phyrexian_life_payments` vector SHORTER than the pip count still gets the
documented per-pip default (unchanged, deliberate), but a vector LONGER than the pip count is now
`Err`, not silently ignored past the pip count. Reworded all three doc comments (`command.rs`) to
state the actual (now-enforced) contract instead of an aspirational one. Added 4 tests in a new
`flatten_hybrid_phyrexian_tests` module in `game_object.rs`. **Verified discrimination**:
temporarily deleted both new `if ... .len() > ... { return Err(...) }` blocks — re-ran — the two
over-long tests FAILED with `Ok(...)` where `Err` was expected. Restored — re-ran — all 4 PASSED.

### LOW findings

- **Finding 3** — `mana.rs`'s `.expect("flat_mana_cost is Some only when ability.mana_cost is
  Some")` replaced: bound the original `ability.mana_cost` in the same `if let` as `flat_mana_cost`
  (`if let (Some(ref flat_cost), Some(ref orig_cost)) = (&flat_mana_cost, &ability.mana_cost)`)
  instead of re-deriving it with an assertion. No `.expect()` remains on this path.
- **Finding 4** — Both `.expect("has_pip_cost checked Some")` sites in `legal_actions.rs` replaced
  with `let Some(mc) = ... else { continue };`, using the loop's existing `continue` escape hatch
  instead of a runtime assertion.
- **Finding 5** — `resolve_hybrid_phyrexian_plan`'s pool-only hybrid-half preference could produce
  a false negative when the pool covers neither half but an untapped source can pay one of them
  (the heuristic reads the pool; `can_afford` also consults untapped sources via the mana solver).
  Refactored into `build_hybrid_choices(cost, pool, flip)` (the `flip` param inverts every
  preference) + `try_hybrid_phyrexian_plan(...)` (the affordability/legality check, extracted so
  it can be tried twice); `resolve_hybrid_phyrexian_plan` now tries the pool-preferred plan first,
  then the flipped-hybrid-half plan as a fallback before giving up. Added
  `provider_offers_the_payable_hybrid_half_when_only_the_other_is_in_pool_preference` (a `{B/R}`
  filter land with an empty pool + an untapped Mountain — the Red half is payable via the
  Mountain even though the pool-preference heuristic defaults to Black). **Verified
  discrimination**: temporarily short-circuited the fallback branch to `None` — re-ran — the new
  test FAILED (`action.is_some()` assertion, got `None`). Restored — re-ran — all 6
  `mtg-simulator` lib tests PASSED.
- **Finding 6** — Disclosure-only, no code change: the plan's §11.6 out-of-scope item
  (`abilities.rs`'s missing CR 119.4 check on `life_cost`) was taken during implementation and is
  disclosed here explicitly, since the original wip.md (step 3) disclosed only the
  Phyrexian-combined half of that fix, not that the plain `life_cost` check was also newly added
  in both `abilities.rs` branches (`:698-707`, `:786-796`). Correct and welcome per the review;
  this entry closes the under-disclosure.
- **Finding 7** — Added a "Release note" paragraph to `debug_assert_flattened`'s doc comment in
  `crates/card-types/src/state/player.rs` stating explicitly that `debug_assert!` is a no-op in
  release and that release correctness rests on the three call sites flattening, not on this guard.
- **Finding 8** — `abilities.rs` and `mana.rs` now call the inherent
  `ManaCost::flatten_hybrid_phyrexian` directly (mapping the `String` error to
  `GameStateError::InvalidCommand` locally, as `legal_actions.rs` already did) instead of routing
  through `super::casting::flatten_hybrid_phyrexian` / `crate::rules::casting::...`. The
  `casting.rs` wrapper itself is untouched (still used by `casting.rs`'s own call site and by
  `crates/engine/tests/casting/mana_costs.rs`, which imports it directly).
- **Finding 9** — Mirrored `chosen_color`'s strictness in `handle_tap_for_mana`: a non-empty
  `hybrid_choices`/`phyrexian_life_payments` supplied for a mana ability whose cost has no such pip
  is now rejected with `InvalidCommand`, instead of silently ignored. (`abilities.rs`/
  `ActivateAbility` has no `chosen_color` field to be asymmetric with, so no equivalent change was
  needed there — the finding's cited asymmetry is specific to `mana.rs`.) Added
  `extraneous_hybrid_choices_on_a_pip_free_mana_ability_is_rejected`. **Verified discrimination**:
  temporarily removed both new guard blocks — re-ran — the new test FAILED (`Ok(...)` instead of
  `InvalidCommand`). Restored — re-ran — PASSED, `git diff` on `mana.rs` clean before the finding's
  real (retained) edit.
- **Finding 11** — Wrapped the ~217-char single-line PB-RS2 comment in all 7 filter-land defs to
  the files' prevailing ~85-char width (3 lines each). `tools/check-defs-fmt.sh` clean after.
- **Finding 15** — Deleted the empty-bodied `#[test] fn
  residue_guard_test_lives_in_card_types_player_rs() {}` from `pb_rs2_activated_pip_payment.rs`
  and replaced it with a plain `//` module comment carrying the same pointer. Test count in that
  file decreases by 1 (offset by the new tests added for other findings).
- **Finding 16** — Added a fourth case to `filter_land_charges_its_hybrid_pip`'s per-land block:
  `half_b` primed in pool, `hybrid_choices: [Color(half_b)]` → `Ok`, asserting the same +1 net-delta
  invariant the `half_a` case already proved. Previously only `half_a` was exercised on the accept
  path. **Verified discrimination**: temporarily forced the `ColorColor` flatten arm to ignore the
  caller's choice and always pick `*a` — re-ran — the new half_b case FAILED with
  `InsufficientMana` (Twilight Mire, first case in the table). Restored — re-ran — PASSED, `git
  diff` on `game_object.rs` shows pure additions only (`git diff | grep '^-'` matches only the
  diff header).
- **Finding 17** — Added `unflattened_hybrid_cost_panics_in_debug_via_spend` to `player.rs`'s test
  module — calls `ManaPool::spend` directly (the guard's other copy, previously untested in
  isolation; only `can_spend`'s copy had a dedicated test). Confirmed the panic fires at
  `player.rs:191` (inside `spend`, not `can_spend`), proving it exercises the intended call site.
- **Finding 18** — Added `assert_eq!(pool_amount(&state, p(1), ManaColor::Red), 0)` to the `{R}`
  half of `hybrid_activated_cost_payable_with_either_half`, matching the `{B}` half's existing
  drain assertion.

### Gates (final, all green)

`cargo build --workspace` clean. `cargo test --workspace` all green (0 failures across every
group; `mtg_card_types` lib 12 passed, `mtg_engine` lib 27 passed, `mtg_simulator` lib 6 passed,
`primitives` group 705 passed 1 ignored, `core` group 428 passed, `casting` group 147 passed,
`scripts` group 43 passed — full breakdown recorded in this session's `cargo test` output).
`cargo clippy --all-targets -- -D warnings` clean (two `items-after-test-module` lints surfaced
and fixed by relocating the two new `#[cfg(test)]` modules — `game_object.rs`'s
`flatten_hybrid_phyrexian_tests` and `replay_harness.rs`'s `parse_hybrid_choices_tests` — to the
end of their files, after the last non-test item, matching every other test module's position in
those files). `cargo fmt --check` clean (after one `cargo fmt` pass; 3 files auto-reformatted:
`mana.rs`, `replay_harness.rs`, `legal_actions.rs`). `tools/check-defs-fmt.sh` clean (1,804 defs).
`PROTOCOL_VERSION` confirmed 27, `HASH_SCHEMA_VERSION` confirmed 63 via direct grep — unchanged by
this fix cycle (no new `Command` field, no `HashInto`-reachable type touched).

### Declined findings

None. All 12 findings (5 MEDIUM + 7 LOW) were applied.

## Post-`/review` record correction (2026-07-20) — a FOURTH unflattened payment site survives

`/review` (Opus, all 5 ACs PASS) found that this PB's narrative of "one flatten
implementation, three call paths, all routed through it" is **incomplete**. There is a
fourth `can_spend`/`spend` call site that this PB did NOT route through the helper:

**`crates/engine/src/rules/engine.rs:1582-1587` (`TurnFaceUp`)** pays a **raw**
`def.mana_cost` (or morph cost) with no flatten — the identical OOS-RS-2 bug class.

**Verified reachable, not theoretical**: `TurnFaceUpMethod::ManaCost` lets a Manifested or
Cloaked permanent be turned face up for its mana cost, and that card can be any creature card.
`crates/card-defs/src/defs/kitchen_finks.rs:8-15` is a `Completeness`-shipped creature whose
`mana_cost` is `{1}{G/W}{G/W}` (two `HybridMana::ColorColor` pips). Manifested and flipped for
its mana cost today, both hybrid pips are charged **free** — the player pays `{1}`.

The AC 5120 residue guard *would* catch this, but only under `debug_assertions` and only if a
test drives that path; none does, so it stays silent in release.

**Disposition: found, NOT fixed — deliberately out of scope.** It is outside all five ACs
(which name `ActivateAbility` and, by the plan's §0.2 correction, `TapForMana`), it is
pre-existing rather than introduced here, and folding a third command's payment path into this
PB after the review + fix cycle had already closed would be exactly the unrequested scope creep
the RS1 close-out warned against. **Filed as OOS-RS2-1** in
`memory/primitives/rider-seed-triage-2026-07-19.md` §1c.

**The corrected claim of record**: PB-RS2 routes **three of four** engine payment sites through
the single `ManaCost::flatten_hybrid_phyrexian` implementation. `TurnFaceUp` is the fourth and
is deferred to OOS-RS2-1. Anyone reading "no second open-coded copy" (AC 5119) as "every
payment site in the engine now flattens" is misreading it.

# M11-local Fix Session Plan

<!-- last_updated: 2026-08-02 -->

## ✅ STATUS: WORKED AND CLOSED — 2026-08-02, `scutemob-173` close-out

Both sessions ran inside `scutemob-173` itself rather than as separate tasks, because
M11-local's close-out was already the work in flight (a kitty crash split it across two
sittings — commit `c2013efa` is the resume point). **10 of 10 HIGH/MEDIUM done. 1 of the
4 in-session LOWs done, 3 deferred with reasons stated at the item.** Checkboxes below
are accurate; the "Not in a session" LOWs are untouched, as intended.

**Two items are closed on *part* of what this plan asked for**, and the reasoning lives
in `docs/mtg-engine-milestone-reviews.md` § "Fix phase — what was taken, and the two
things deliberately left" rather than being implied by a tick: MR-M11-04 took the
narrowing, not the companion gate; MR-M11-06 took the filing, not the code half.

**Correction to this plan's own header**: it says `HASH_SCHEMA_VERSION` 69 must be
unmoved. The constant is **70** — PB-DX5 (`scutemob-170`) moved it on the parallel W6
track before this branch forked, so M11-local inherited 70 and left it there. The
requirement (unmoved *by this work*) was met; the number in the plan was stale, the same
drift the plan's own MR-M11-12 item is about.

Source: `docs/mtg-engine-milestone-reviews.md` § "M11-local: Web Client & Local Play
(First Playable)" — 1 HIGH / 9 MEDIUM / 8 LOW / 3 INFO, reviewed 2026-08-01.

**Scope rule for both sessions**: M11-local made no engine change beyond a read-only
query module, and neither session below needs one. If a fix appears to require an
engine edit, a new `Command`/`GameEvent`/`Effect` variant, or a `GameState` field,
**stop and flag** — that is a wire change (SR-8) and belongs in the PB-DX queue,
not here. `PROTOCOL_VERSION` 32 and `HASH_SCHEMA_VERSION` 69 must be unmoved at the
end of each session; run the `core` group's `protocol_schema` / `hash_schema`
sentinels to confirm rather than assuming.

**Do not start the play-server or replay-viewer HTTP binary.** An agent context that
does gets SIGKILL/137 (`memory/gotchas-infra.md`, plan §7 constraint 1). Every HTTP
check goes through `tower::ServiceExt::oneshot` against `build_router(..)`. The crate
machine-enforces this: `test_no_socket_symbol_appears_in_the_test_region` will redden,
naming the file, if a test names `TcpListener` / `axum::serve` / `bind` / the serving
entry point below a `#[cfg(test)]` cut.

---

## Session 1 — Architecture Invariant 7: close the two bypasses and make the gate honest

Theme: everything that leaves the process for a seat. Touches
`tools/play-server/src/{view.rs,api.rs,main.rs}`, `tools/play-server/README.md`,
`memory/decisions.md`.

- [x] **DONE** MR-M11-01 (**HIGH**) — `GameSummary.seed` reconstructs every hidden zone.
      `(seed, players, mulligan_count)` + the fixed `config_for` inputs rebuild every
      seat's opening hand and library order through `setup::build_initial_state`.
      Gate the field off the seat view (keep `players` / `mulligan_count`, which the
      client renders), leave it on `BugReportView`, and record it as an explicit
      exception with an M10a re-scope note if it is kept anywhere on `SeatView`.
      Add a test that the seat payload does not carry it.
- [x] **DONE** MR-M11-08 (MEDIUM) — `GameOverView.violations` / `.reason` are raw `Debug`
      strings injected outside both redaction chokepoints (`view.rs::game_over_view`,
      `halted_view`). Reduce to check names + counts, or route through an entitlement
      pass. Live instance: `invariants::check_no_orphaned_tokens` interpolates
      `obj.characteristics.name`.
- [x] **DONE (narrowing only)** MR-M11-04 (MEDIUM) — the Invariant-7 source gate
      (`test_production_code_never_builds_an_omniscient_view`) cannot see a route that
      serializes engine types directly, which is how `BugReportView` shipped green.
      Either narrow its failure message to what it checks, or add a companion gate
      pinning the set of handlers returning a non-`SeatView` body to exactly
      `{get_report, get_healthz}`.
- [x] **DONE** MR-M11-03 (MEDIUM) — README documents neither `GET /api/game/report` nor its
      Invariant-7 exception, while `view.rs::BugReportView`'s doc claims it does. Add
      the Routes-table row and a named exception paragraph to "Hidden information",
      both pointing at `memory/decisions.md` §5 and the M10a obligation.
- [x] **DONE (one deviation, stated)** MR-M11-05 (MEDIUM) — omitting `params` and sending
      `"params": {}` are not equivalent (`auto_tap` false vs true). Hand-write `impl
      Default for ActionParamsDto` with `auto_tap: true`; pin both spellings with one
      `oneshot` test; note it in the README's route table.
      **Deviation**: what shipped is a plain `#[test]` at the deserialization boundary,
      not a `oneshot` test. That is where the divergence actually lived — both spellings
      are parsed and lowered through the real `From<ActionParamsDto> for ActionParams` and
      compared structurally — so it pins the property directly rather than through a route
      that would only observe its consequence. The test additionally pins that the agreed
      `auto_tap` is `true`, so a re-`derive(Default)` reddens even though it would leave
      the two spellings agreeing; a `oneshot` equality test would not have caught that.
- [ ] **DEFERRED** MR-M11-17 (LOW, same file family) — `event_kind` serializes the whole
      event to read one key, on every event of every request. Replace with a `match`
      returning `&'static str`, **preserving the reads-no-payload-field property** that is
      what makes it leak-proof.
      **Deferred at close-out**: this is not the one-line swap the item implies. The
      value of doing it is the per-variant `text` arms ("a poor game log" is the other
      half of the finding), which is authoring player-facing prose for dozens of
      `GameEvent` variants — a UI-quality task, and one whose natural home is the M13 log
      work rather than a milestone-close fix cycle. The leak-proof property the item
      rightly protects is what makes deferring safe: the catch-all renders the bare
      variant name and cannot leak. Cost is a per-event `to_value` on a
      request/response cadence, which MR-M11-18 already frames as an M10a re-measure.

**Gate for this session**: `cargo test -p play-server -p mtg-view-model`, plus the
two source gates green and *proven non-vacuous by mutation* if either is edited.

---

## Session 2 — Seed hygiene, unusable capabilities, and simulator correctness

Theme: `crates/simulator` + the seed inventory. Touches
`crates/simulator/src/{params.rs,local_game.rs,heuristic_bot.rs,mana_solver.rs,invariants.rs,driver.rs}`,
`tools/play-server/src/session.rs`, `docs/audits/decision-point-audit.md`.

- [x] **DONE** MR-M11-02 (MEDIUM) — `OOS-M11-7` (CR 704.3: SBAs not checked on every priority
      grant) and `OOS-M11-9` (CR 508.1: `handle_declare_attackers` has no
      already-declared guard) are named as filed in shipped source and exist in no
      inventory. Add both as rows in `docs/audits/decision-point-audit.md` §8.1 with
      the CR cite, the observation that produced each, and the engine-vs-simulator
      classification. **File only — do not fix; both are engine changes.**
- [x] **DONE (filing only — see below)** MR-M11-06 (MEDIUM) — a human cannot use a targeted loyalty ability.
      Add `LegalAction::ActivateLoyaltyAbility` to `action_to_command_with_params`'
      parameterization allowlist and forward `params.targets` / `params.x_value`
      (`Command::ActivateLoyaltyAbility` already has both fields); add it to
      `view.rs::action_target_requirements` and `target_query_source`. File the seed
      the in-source comment promised. Check `ActivateBloodrush` / `CastWithMutate` /
      `CastMorphFaceDown` in the same pass and say explicitly which stay hard-coded.
- [x] **DONE** MR-M11-07 (MEDIUM) — `{X}` announced through the API is unpayable
      (`OOS-M11-8`). Include `cast.x_value` in the cost `auto_tap_commands_for` solves
      for (CR 107.3b), **or** disable `ValuePrompt`'s X input with the reason. Pin
      whichever with a test; the current workaround (tap manually first) is not
      reachable from this UI.
- [x] **DONE** MR-M11-09 (MEDIUM) — the S8 repeat cap disables bot attacks in every extra
      combat phase (CR 506.5 / 508.1; `aurelia_the_warleader` is `Complete` and
      deck-legal). Reset `repeats_this_turn` on combat-phase entry rather than on
      `turn_number`, or exempt `DeclareAttackers` when `turn.in_extra_combat`. Do not
      simply remove the cap — it is the only thing preventing the `OOS-M11-9` stall.
- [x] **DONE** MR-M11-10 (MEDIUM) — a kept hand can be re-dealt. Add `kept: bool` to
      `PlaySession`, set it from `post_mulligan { take: false }`, and answer
      `409 not_pregame` afterwards (CR 103.5: the choice is terminal). `summary.pregame`
      then means what `PlayApp.svelte`'s client-side `keptHand` currently infers.
- [ ] **DEFERRED** MR-M11-13 (LOW) — `unreachable!()` in `GameDriver::run_game`. Return a
      `GameResult` with `GameDriverError::EngineError` and keep the invariant as a
      `debug_assert!`.
      **Deferred at close-out**: `GameDriver` is the fuzzer's entry point and the arm is
      provably unreachable there (`human_seats` is empty, so `advance()` cannot return
      `AwaitingHuman`). Converting a panic into a `GameResult` on that path changes what
      the fuzzer reports for a state it cannot reach, for no behavioural gain, at the one
      moment the branch's headline evidence is a measured 500-game fuzz parity. Correct
      to take when something else is already touching `driver.rs`.
- [ ] **DEFERRED** MR-M11-14 (LOW) — `check_stack_consistency` no longer rejects two
      non-copy `Spell` stack objects claiming the same `source_object` (CR 400.7 makes it
      impossible). Count instead of set-insert and report a duplicate.
      **Deferred at close-out, on this plan's own advice**: the Session 2 gate below names
      MR-M11-14 as one of the two items that can perturb the 500-game fuzz parity, and
      that parity run *is* criterion 5977's evidence — re-earning it is not a
      milestone-close activity for a LOW. It also strictly *adds* a violation class, so
      the risk is one-directional and real: this check is the one S8 rewrote to remove
      501 false positives, and re-widening it needs its own measured run.
- [x] **DONE** MR-M11-12 (LOW) — `auto_tap_commands_for` cites a `mana_solver.rs` doc
      sentence that does not exist. Add the pool / `OOS-M11-2` note to `mana_solver.rs`'s
      module doc, or re-point the cite.
      **Taken rather than deferred** — it is the lying-cite class, doc-only, zero risk to
      the fuzz parity, and the fix is strictly better than the re-point: `mana_solver.rs`
      now documents *both* halves of `OOS-M11-2`, and the second (it reads
      `obj.characteristics.mana_abilities` raw rather than through
      `calculate_characteristics`) was verified at the read site, not copied from
      CLAUDE.md.

**Gate for this session**: `cargo test -p mtg-simulator -p play-server`, then the
500-game fuzz parity command from `memory/m11/s8-fuzz-parity.md`
(`--games 500 --seed 12345 --max-turns 40 --verbose`) diffed against the pre-session
build. MR-M11-09 and MR-M11-14 are the two that can perturb it; a difference in
turns/commands/outcome stops the session (plan §8 R11).

---

## Not in a session

`MR-M11-11`, `MR-M11-15`, `MR-M11-16`, `MR-M11-18` are LOW and independent — take
them opportunistically. `MR-M11-16` (a `DeckSource::Fixed` `oneshot` test for
`force_of_vigor`'s `UpToN { count: 2 }` slot and for a modal spell with `mode_targets`)
is the highest-value of the four: it converts two "right by construction and
unexercised" README limitations into pinned behaviour without needing a frontend
harness.

`MR-M11-19` / `MR-M11-20` / `MR-M11-21` are INFO — no action.

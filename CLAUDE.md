# CLAUDE.md — MTG Commander Rules Engine

> **This file is the primary context document for Claude Code sessions.** Read this before
> doing anything. It tells you where the project is, what the architecture looks like,
> and what to watch out for.
>
> **Update this file** at the completion of each milestone or when major design decisions
> change. The "Current State" section should always reflect reality.

---

## Current State

> Detailed PB-by-PB handoffs, hazards, and seed inventories live in `memory/workstream-state.md`.
> Worker sessions: append detail there, not here. CLAUDE.md tracks current snapshot only.

> **Formatting rule (2026-08-02)**: wrap prose at ~100 characters (semantic line breaks) and NEVER
> grow an existing line — git merges at line granularity, so single-line mega-bullets make every
> parallel CLAUDE.md edit an unresolvable whole-line conflict. Close-outs append a NEW short delta
> and rotate detail to the monthly archive.

- **Active Milestone**: **M11-local is DONE — the engine is first-playable** (closed 2026-08-01 by
  `scutemob-173`; all 8 sessions of `memory/m11-session-plan.md` shipped as
  `scutemob-147`/`161`/`163`/`165`/`167`/`169`/`171`/`173`). A human occupies one seat of a 4-player
  Commander game in a browser against three simulator bots, with no networking: `LocalGame`
  (steppable driver, `crates/simulator`) → `setup.rs` (deterministic seeded pregame through the real
  `validate_deck`) → `crates/view-model` (seat-redacted view models, Architecture Invariant 7
  chokepoint) → `tools/play-server` (axum on 3040, 6 routes, Svelte 5 frontend). **Wire-neutral end
  to end**: no new `Command`/`GameEvent`/`Effect` variant in any session, and the milestone's only
  engine addition is the read-only `crates/engine/src/rules/queries.rs` (S3). PROTOCOL **32** / HASH
  **70** at close, both moved by the W6 track and never by M11-local. **What it does NOT deliver,
  stated plainly**: card images are fetched from Scryfall over the network rather than cached (M14);
  the bug-report artefact has no free-text description field; no automated test spans browser +
  game, because no frontend test harness exists (plan §8 R7, revisit at M13); `StubProvider`
  enumerates no Adventure, alt-cost or Convoke/Improvise/Delve casts (R4). **The active track is now
  the PB-DX correctness queue alone** (`memory/primitives/seed-rerank-2026-08-02.md` §4 — v3,
  `scutemob-182`; PB-DX19 shipped `451e3517`, PB-DX22 shipped `95f53b78`;
  PB-DX32 shipped `scutemob-197`/`685aa1c4` (promoted per feedback doc §2.3, user-approved 2026-08-03);
  PB-DX20 shipped `scutemob-198`/`ecd7b119` 2026-08-04;
  PB-DX21 shipped `scutemob-200` 2026-08-04 (ranks 1-4 all shipped);
  PB-DX23 shipped `scutemob-201` 2026-08-05 (rank 5);
  PB-DX24 shipped `scutemob-202` 2026-08-05 (rank 6);
  PB-DX25 shipped `scutemob-203` 2026-08-05 (rank 7);
  PB-DX25b shipped `scutemob-204` 2026-08-06 (rank 7b, INSERTED 2026-08-06 user-approved);
  PB-DX25c shipped `scutemob-205` 2026-08-06 (rank 7c, INSERTED 2026-08-06 user-approved —
  closed OOS-DX25b-3, the CR 115.7a redirect-legality gap);
  PB-DX26 shipped `scutemob-206` 2026-08-11 (rank 8 — closed OOS-CARDS1-3, OOS-CARDS1-1 and
  OOS-DX3b-1);
  PB-DX7 shipped `scutemob-207` 2026-08-11 (rank 9 — closed OOS-DP7-11, OOS-DP9-13,
  and both riders OOS-DP10-1 and OOS-DP9-10's residual);
  PB-DX8 shipped `scutemob-208` 2026-08-12 (rank 10 — FILED and closed OOS-CARDS2-7,
  RECORDED OOS-DP10-9, shipped the PB-DX42a rider); **next dispatch: PB-DX27** (rank 11);
  the playtest-successor run 174–181
  AND the triage-2 successor run 187–194 both completed 2026-08-02 — triage 2 is fully closed,
  8/8 rows shipped. **FEEDBACK-1 SHIPPED** (`scutemob-192`, merge `d55e74cc`, doc-only):
  `docs/mtg-engine-feedback-engineering.md` is the alpha feedback-loop strategy — 8 ranked
  proposals, dispatch order the coordinator's; PROTOCOL **35** / HASH **72** as of ENG-2).
  The roadmap's next milestone candidate is **M10-pre → M10a** — *not
  started, and not to be started without direction.* Full session-by-session M11 narrative:
  `memory/archive/claude-md-changelog-2026-08.md`; S8 handoff and durable lessons:
  `memory/workstream-state.md`. **Seeds this milestone left open**: **OOS-M11-2**
  (cost MODIFIERS and CR 106.12 restricted mana only, as of SIM-2 — its pool half closed in S3,
  its commander-tax half by SIM-1, and its layer-resolution half by SIM-2, which found that half
  live-wrong on **face-down** permanents rather than theoretical), **OOS-M11-3** / **OOS-DP3-9**
  (the fuzzer is not run-to-run deterministic in
  very long games and stack-overflows at `--max-turns 200`; pre-existing, reproduced on pristine
  merge-base code by S8 — **and SIM-2 diagnosed a mechanism**, `OOS-SIM2-6`: an unbounded
  `calculate_characteristics` recursion that `indomitable_archangel` makes unconditional),
  **DE-NOISED by SIM-3** (`scutemob-177`) — this seed's `stack_consistency` half is WITHDRAWN,
  measured: the check was a false positive by construction and accounted for **90.3%** of a
  5-game fuzz run's entire violation volume (9,719 → 938). Its determinism and stack-overflow
  halves stand; read every pre-2026-08-02 `stack_consistency` count as a spell-and-ability
  census, not a defect count,
  **OOS-M11-7** (CR 704.3 SBAs are checked on step entry and at resolution,
  not on every priority grant, so a token sacrificed as a mana cost lingers in the graveyard until
  the next of those — self-healing, never wrong at rest), **OOS-M11-9** (`handle_declare_attackers`
  has no "already declared this combat" guard; CR 508.1 makes it a once-per-combat turn-based
  action, and with a vigilant attacker the engine will accept re-declaration without limit)
  — **CLOSED 2026-08-04 by PB-DX21** (`scutemob-200`), so this milestone's last open combat-side
  seed is gone.
  **CLOSED by M11-local**: OOS-M11-1 (PB-DP2), OOS-M11-4 (PB-DP8), OOS-M11-6 (PB-DX4), OOS-M11-8
  (S8). **Milestone review DONE and its fix cycle closed** (`docs/mtg-engine-milestone-reviews.md`,
  MR-M11-01..21): 1 HIGH + 9 MEDIUM all closed; of 8 LOW, 1 closed and 7 left open. The reviewer's
  `memory/m11-fix-session-plan.md` had scoped **four** LOWs into its sessions rather than leaving
  all eight to opportunity, so "LOW needs no fix phase" was only half the account: MR-M11-12 was
  taken (a doc cite pointing at a sentence that does not exist) and MR-M11-13/14/17 deferred with
  the reason at each item — MR-M11-14 on the plan's own advice, since its gate names that item as
  one of the two that can perturb the 500-game fuzz parity the branch's acceptance evidence rests
  on. The HIGH is worth carrying past the milestone — `GameSummary.seed` shipped on **every** seat
  payload for three sessions and *rebuilds* every bot's opening hand and library order
  (`build_initial_state` is deterministic in its config alone), while **both** Invariant-7 gates
  stayed green, because one searches the body for card **names** and the other scans source for
  omniscient **view-model entry points**, and a seed is neither: **a redaction gate checks the
  channel it was written for, and a new channel is invisible to it.** Three gates for three channels
  now, tabled in the play-server README. Also from the close-out: three fixes had landed without the
  test their finding asked for (now added, each proven to discriminate by execution — and the first
  revert *did not compile*, the S8 `{X}` lesson recurring inside the same task); **`OOS-M11-10`**
  filed for the loyalty-ability targeting gap whose in-source comment had promised a filing for
  three sessions; and the reviews doc's `HASH 69` corrected to **70** in four places — the claim was
  true, the number was stale, PB-DX5 moved it on the parallel W6 track before this branch forked.
- **Card Authoring Campaign** (continuous, was M12): plan
  `memory/card-authoring/campaign-plan-2026-05-16.md` §0. **Live coverage: 1,133/1,803 = 62.8%**
  (unmoved by PB-DX26 — one flip up and one honest flip down cancelled, 2026-08-11)
  (PB-DX4's 6 honest demotions outweigh its 6 in-place repairs — the number went *down* because the
  corpus got *truer*) — regenerate with `tools/authoring-report.py`; `docs/authoring-status.md` is
  the canonical, self-dating source. **Current queue state: the PB-OS queue is COMPLETE; the PB-DP
  suite is COMPLETE (DP1..DP10, `scutemob-149..158`); the PB-RS queue is **RETIRED** — the re-rank
  ran as `scutemob-159`, and the authoritative queue is now
  `memory/primitives/seed-rerank-2026-08-02.md` §4 (v3), **PB-DX7..PB-DX41**, correctness-first;
  `seed-rerank-2026-07-27.md` §4 is banner'd SUPERSEDED (its §1-§3 remain canonical).
  RS5..RS11 are each dispositioned there (R5 retired; R6→PB-DX5, R7→PB-DX13, R8→PB-DX12, R9→PB-DX16,
  R10→PB-DX14, R11→PB-DX17) and `rider-seed-triage-2026-07-19.md` §3/§5 must not be claimed from.
  **PB-DX1..PB-DX6 ALL SHIPPED** (`scutemob-160`/`162`/`164`/`166`/`168`/`170`/`172`; full
  narratives in `memory/archive/claude-md-changelog-2026-08.md`, per-batch handoffs in
  `memory/workstream-state.md`). PROTOCOL **35** / HASH **72** (as of ENG-2, `scutemob-193`).
  **PB-DX19 SHIPPED** (`scutemob-184`, `451e3517`) and **PB-DX20 SHIPPED** (`scutemob-198`,
  `ecd7b119`) and **PB-DX21 SHIPPED** (`scutemob-200`) and **PB-DX23 SHIPPED**
  (`scutemob-201`) and **PB-DX24 SHIPPED** (`scutemob-202`) — **next dispatch: PB-DX25** (rank 7;
  brief in `memory/primitives/seed-rerank-2026-08-02.md` §4; re-word OOS-DX19-2 per OOS-ADJ-3
  before any DX42b dispatch). **PB-DX7 is no longer next** — it survives at
  rank 9; eight new entries outrank it. Older queue history (the PB-OS,
  PB-RS and PB-DP chains) is rotated to the 2026-08 archive.
  **PB-DX20 SHIPPED** (`scutemob-198`; v3 queue rank 2).
  **PB-DX21 SHIPPED** (`scutemob-200`; v3 queue rank 3) — v3 ranks **1-4 are all shipped**.
  **PB-DX23 SHIPPED** (`scutemob-201`; v3 queue rank 5) and **PB-DX24 SHIPPED**
  (`scutemob-202`; v3 queue rank 6) — v3 ranks **1-6 are all shipped**.
  **PB-DX25 SHIPPED** (`scutemob-203`; v3 queue rank 7) — v3 ranks **1-7 are all shipped**.
  **PB-DX25b SHIPPED** (`scutemob-204`; v3 queue rank 7b) — **OOS-DX25-3 CLOSED**; ranks
  **1-7b are all shipped**. **PB-DX25c SHIPPED** (`scutemob-205`; v3 queue rank 7c, INSERTED
  2026-08-06 user-approved — closed **OOS-DX25b-3**, the CR 115.7a redirect-legality gap; row
  in the v3 memo §4) — ranks **1-7c are all shipped**.
  **PB-DX26 SHIPPED** (`scutemob-206`; v3 queue rank 8 — **OOS-CARDS1-3**, **OOS-CARDS1-1**
  and **OOS-DX3b-1** all CLOSED) — ranks **1-8 are all shipped**.
  Live coverage NET UNMOVED at **1,133/1,803 = 62.8%** — one flip up
  (`sword_of_body_and_mind`) and one honest flip down (`the_reaver_cleaver`).
  **PB-DX7 SHIPPED** (`scutemob-207`; v3 queue rank 9 — **OOS-DP7-11** and **OOS-DP9-13**
  CLOSED, plus both riders **OOS-DP10-1** and **OOS-DP9-10**'s residual) — ranks
  **1-9 are all shipped**. Coverage unmoved at
  **62.8%**, proven by regeneration. PROTOCOL **35** / HASH **74** as of PB-DX7
  (both gate-executed, both unmoved by it).
  **PB-DX8 SHIPPED** (`scutemob-208`; v3 queue rank 10 — **OOS-CARDS2-7** FILED *and* CLOSED
  (it had no registry row at all until this batch wrote one), **OOS-DP10-9** **RECORDED, not
  closed**, and the **PB-DX42a** rider shipped per adjudication §5.1) — ranks **1-10 are all
  shipped**, so **next dispatch: PB-DX27** (rank 11). Coverage unmoved at **62.8%**, proven by
  regeneration; PROTOCOL **35** / HASH **74** gate-executed and unmoved.
- **Tests (delta 2026-08-12, PB-DX8 + fix cycle)**: **4,561 / 0 / 5** full-workspace on branch
  `scutemob-208` (+34 over the **4,527** baseline measured on this branch BEFORE any edit),
  `--workspace --no-fail-fast` to a file, 46 result-producing targets, residual list empty.
  **Delta itemised by test NAME with zero removals**, by set-diffing the two run logs: 17 in the
  new `crates/engine/tests/core/pb_dx8_oracle_decision_cross_check.rs`, 10 in the new
  `crates/engine/tests/core/pb_dx42a_continuous_condition_roster.rs`, and 7 added to
  `crates/engine/tests/core/completeness_deviation_scan.rs` (5 → 12). Three of the 34 are the
  `/review` fix cycle's own additions.
  **PROTOCOL 35 / HASH 74 both unmoved**, gate-executed (`hash_schema` 36/36,
  `protocol_schema` 17/17). **0 source lines and 0 card-def edits of ANY kind** —
  `git diff --numstat` over `crates/engine/src`, `crates/card-types/src`,
  `crates/card-defs/src`, `crates/view-model/src`, `crates/simulator/src` and `tools/` is
  empty, so unlike PB-DX7 no per-line comment audit was owed. Coverage unmoved at
  **1,133/1,803 = 62.8%**, proven by regeneration with the self-dating churn reverted.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs).
- **Tests (delta 2026-08-12, PB-DX7 + fix cycle)**: **4,527 / 0 / 5** full-workspace on branch
  `scutemob-207` (+19 over the **4,508** baseline measured on this branch BEFORE any edit),
  `--workspace --no-fail-fast` to a file, 46 result-producing targets, residual list empty.
  **The delta is itemised by test NAME, not by arithmetic**: 4 on the rider commit
  (`decision_gate::pb_dp9_roster_walks_agree_by_value` plus the three in the new
  `crates/engine/tests/core/unordered_iteration_ratchet.rs`) and 15 in
  `crates/engine/tests/core/hash_schema.rs` (21 → 36 across the whole task). **PROTOCOL 35 / HASH 74 both unmoved**,
  gate-executed (`hash_schema` 33/33, `protocol_schema` 17/17) — no genuinely-unhashed
  field was found, so no bump was warranted and none was taken. **0 non-comment lines in
  `crates/engine/src/state/hash.rs`** — verified line-by-line in python, because
  `grep -E '^[+-]//'` returns a **false positive on indented comments** and would have
  under-reported the check. 3 files touched, **0 card-def edits**; coverage unmoved at
  **1,133/1,803 = 62.8%**, proven by regeneration with the self-dating churn reverted.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs).
- **Tests (delta 2026-08-11, PB-DX26 + fix cycle)**: **4,508 / 0 / 5** full-workspace on
  branch `scutemob-206` (+17 over the **4,491** baseline measured on this branch BEFORE any
  edit — 6 gates in the new `crates/engine/tests/core/pb_dx26_attach_keyword_roster.rs`
  (R1-R6) and 11 probes in the new `crates/engine/tests/primitives/pb_dx26_equip_surface.rs`
  (T1-T9 plus the fix cycle's T10/T11, which pay the corpus's only coloured equip cost
  `{1}{W}` and its only zero cost `{0}` for real rather than comparing them statically);
  every other change is a modification of an existing test, not an addition), `--workspace
  --no-fail-fast` to a file, 46 result-producing targets, residual list empty.
  **PROTOCOL 35 / HASH 74 both unmoved**, gate-executed (`hash_schema` 21/21,
  `protocol_schema` 17/17). **0 engine-source lines** — `git diff --numstat` over
  `crates/engine/src`, `crates/card-types/src`, `crates/view-model/src` and
  `crates/simulator/src` is empty. **`tools/` is not zero**, and the first draft of this
  line implied it was (review Finding L11): `tools/play-server/src/main.rs` moves
  **+~50 -~24**, entirely inside its `#[cfg(test)]` module — the
  `UI3_SPLIT_COMBAT_SEED` constant and its doc. **Coverage NET ZERO at 1,133/1,803 =
  62.8%**, regenerated: one flip UP (`sword_of_body_and_mind`, its only blocker being
  the Equip {2} this batch authored) and one honest flip DOWN (`the_reaver_cleaver`,
  review Finding 7 — derive-`Complete` with no marker anyone had ever ruled on, while
  the trigger it grants under-fires against "a player **or planeswalker**").
  **A stable count is not a stable deal**: the two moves cancelled in
  `CORPUS_COMPLETE` and not in the SET, so the fuzz pool holds a different card and
  `UI3_SPLIT_COMBAT_SEED` had to be re-observed **twice** (21 → 28 → 26), each time by
  an executed sweep, while the constant that normally shouts about a pool change
  stayed green. `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt
  --check` clean, `tools/check-defs-fmt.sh` clean (1,803 defs).
- **Tests (delta 2026-08-06, PB-DX25c fix cycle 2)**: **4,491 / 0 / 5** full-workspace on
  branch `scutemob-205` (+4 over the **4,487** fix-cycle-1 SETTLED pin — 4 new probes:
  `t7b_plain_target_spell_victim_cannot_redirect_onto_its_own_card`,
  `bb1_bolt_bend_object_branch_lands_only_on_a_legal_creature_never_a_land`,
  `bb2_bolt_bend_object_branch_no_legal_target_leaves_targets_unchanged`,
  `s1b_bot_driven_misdirection_object_branch_redirects_legally`), `--workspace
  --no-fail-fast` to a file, 46 result-producing targets, residual list empty.
  **PROTOCOL 35 / HASH 74 both unmoved**, gate-executed (`hash_schema` 21/21,
  `protocol_schema` 17/17). Coverage unmoved **1,133/1,803 = 62.8%**, proven by
  regeneration, self-dating churn reverted. `clippy --workspace --all-targets -- -D
  warnings` clean, `cargo fmt --check` clean (one fmt pass applied to the 2 new test
  files), `tools/check-defs-fmt.sh` clean (1,803 defs). Closed **OOS-DX25c-5** (a
  `TargetSpell`/`TargetSpellWithFilter` victim could be redirected onto its own card) with
  a two-line `self_id` guard, proven cast-path-neutral and red by an executed revert.
- **Tests (delta 2026-08-06, PB-DX25b)**: **4,469 / 0 / 5** full-workspace on branch
  `scutemob-204` (+17 over the **4,452** baseline measured on this branch BEFORE any edit —
  10 probes in the new `crates/engine/tests/primitives/pb_dx25b_announced_stack_target_
  space.rs` and 7 gates in the new `crates/engine/tests/core/pb_dx25b_announced_target_
  roster.rs`; the repaired fixtures in `casting.rs`, `pb_ef11_spell_single_target.rs` and
  `copy_redirect.rs` are modifications, not additions), `--workspace --no-fail-fast` to a
  file, residual list empty. **PROTOCOL 35 / HASH 73 both unmoved**, gate-executed after the
  fix cycle as well. Coverage unmoved **1,133/1,803 = 62.8%**, proven by regeneration; all 4
  card-def edits comment-only, verified per-line. Earlier pins below.
- **Tests (delta 2026-08-05, PB-DX25)**: **4,452 / 0 / 5** full-workspace on branch
  `scutemob-203` (+17 over the **4,435** baseline measured on this branch BEFORE any edit —
  T1-T7 in the new `crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs`,
  G1-G4 in the new `crates/engine/tests/core/pb_dx25_stack_registry_roster.rs`, and 2 in the
  new `crates/simulator/tests/pb_dx25_counter_on_mutate_is_consistent.rs`),
  `--workspace --no-fail-fast` to a file, residual list empty.
  **PROTOCOL 35 / HASH 73 both unmoved**, gate-executed after the `abilities.rs`/`casting.rs`
  edits as well. Benches within noise (`full_turn_4p` 214-215 µs). Earlier pins below.
- **Tests (delta 2026-08-05, PB-DX24)**: **4,435 / 0 / 5** full-workspace on branch
  `scutemob-202` (+22 over the **4,413** baseline measured on this branch BEFORE any edit —
  17 probes in the new `crates/engine/tests/primitives/pb_dx24_trigger_zone_and_index_
  spaces.rs` and 5 gates in the new `crates/engine/tests/core/pb_dx24_trigger_zone_
  roster.rs`), `--workspace --no-fail-fast` to a file, residual list empty.
  **PROTOCOL 35 / HASH 73 both unmoved**, gate-executed. Benches within noise
  (`full_turn_4p` 221.5-223.5 µs). Earlier pins below.
- **Tests (delta 2026-08-05, PB-DX23)**: **4,413 / 0 / 5** full-workspace on branch
  `scutemob-201` (+15 over the **4,398** baseline measured on this branch BEFORE any edit —
  1 mandatory probe + 6 more in the new `crates/simulator/tests/pb_dx23_dredge_answer_
  channel.rs`, 7 in the new `crates/engine/tests/primitives/pb_dx23_dredge_tail_and_
  query.rs`, 1 play-server HTTP probe), `--workspace --no-fail-fast` to a file,
  residual list empty. **PROTOCOL 35 / HASH 73 both unmoved**, gate-executed. Earlier
  pins below.
- **Tests (delta 2026-08-04, PB-DX21)**: **4,398 / 0 / 5** full-workspace on branch
  `scutemob-200` (+10 over the **4,388** baseline measured on this branch BEFORE any edit —
  9 probes in the new `crates/engine/tests/primitives/pb_dx21_declare_attackers_once_per_
  combat.rs` + 1 simulator offer-suppression probe), `--workspace --no-fail-fast` to a file,
  residual list empty. **PROTOCOL 35 unmoved / HASH 72 → 73**, both gate-executed. Earlier
  pins below.
- **Tests (delta 2026-08-04, PB-DX20)**: **4,388 / 0 / 5** full-workspace on branch
  `scutemob-198` (+15 over the **4,373** baseline measured on this branch BEFORE any edit —
  14 probes in the new `crates/engine/tests/primitives/pb_dx20_keyword_carried_target_
  requirements.rs` + 1 play-server HTTP probe), `--workspace --no-fail-fast` to a file,
  residual list empty. **PROTOCOL 35 / HASH 72 unmoved**, both gate-executed. Earlier pins below.
- **Tests (delta 2026-08-03, PB-DX32)**: **4,373 / 0 / 5** full-workspace on branch
  `scutemob-197` (+15 over PB-DX22's 4,358, re-measured on this branch BEFORE any edit as its
  own baseline — 14 probes in the new `crates/simulator/tests/pb_dx32_fuzz_output.rs`, 1 in
  `sim5_bot_cast_discipline.rs`, plus one test appended to `crates/engine/tests/core/
  decision_gate.rs`), measured with `--workspace --no-fail-fast` to a file, residual list
  empty, and re-run independently after the fix cycle. **PROTOCOL 35 / HASH 72 unmoved**,
  both gate-executed. Earlier pins below.
- **Tests (delta 2026-08-03, PB-DX22)**: **4,358 / 0 / 5** full-workspace on branch
  `scutemob-196` (+13 over UI-6's 4,345, which this branch re-measured as its own pre-edit
  baseline — 12 probes in the new `crates/simulator/tests/pb_dx22_fuzz_instrument.rs` + 1
  CR 903.9b probe added to `crates/simulator/tests/local_game.rs`), measured with
  `--workspace --no-fail-fast` to a file, residual list empty. **PROTOCOL 35 / HASH 72
  unmoved**, both gate-executed. Earlier pins below.
- **Tests (delta 2026-08-02, UI-6)**: **4,345 / 0 / 5** full-workspace on branch
  `scutemob-194` (+4 over ENG-2's 4,341, measured on that branch BEFORE any edit — 2 play-server
  HTTP probes + 1 frontend source gate + 1 review-cycle restriction probe; the Invariant-7 gate
  was renamed, not added), measured with `--workspace --no-fail-fast` to a file, residual list
  empty. **PROTOCOL 35 / HASH 72 unmoved**, both gate-executed. Earlier pins below.
- **Tests (delta 2026-08-02, ENG-2)**: **4,341 / 0 / 5** full-workspace on branch
  `scutemob-193` (+11 over ENG-1's 4,330 — 9 engine probes + 1 view-model redaction probe + 1
  play-server HTTP probe), measured with `--workspace --no-fail-fast` to a file, residual list
  empty. **PROTOCOL 34 → 35, HASH 71 → 72**, both gate-computed. Earlier pins below.
- **Tests (delta 2026-08-02, ENG-1)**: **4,330 / 0 / 5** full-workspace on branch
  `scutemob-191` (+13 over a 4,317 baseline measured on that branch BEFORE any edit — 11 engine
  tests + 2 play-server probes), measured with `--workspace --no-fail-fast` to a file.
  **PROTOCOL 33 → 34, HASH 70 → 71**, both gate-computed. Earlier pins below.
- **Tests (delta 2026-08-02, UI-5)**: **4,317 / 0 / 5** full-workspace on branch
  `scutemob-190` (+4 over SIM-6's 4,313 — the four new frontend source gates), measured with
  `--workspace --no-fail-fast` to a file. Earlier pins below.
- **Tests (delta 2026-08-02, SIM-6)**: **4,313 / 0 / 5** full-workspace on branch
  `scutemob-189` (+18 over SIM-5's 4,295: 11 simulator + 7 play-server), measured with
  `--workspace --no-fail-fast` to a file. Earlier pins below.
- **Tests (delta 2026-08-02, second session)**: **4,281 / 0 / 5** full-workspace at the PB-DX19
  collect (`451e3517`); UI-4 (`b031d39e`) adds +2 play-server gates (57 green, targeted run) —
  nominal 4,283, full-tree re-measure at next collect. Earlier pin below.
- **Tests**: **4,263 passing / 0 failing / 5 ignored** on main at the wave-4 collect
  (`b76b1df4`, 2026-08-02) — the full playtest-successor run 174–181 landed +139 over the 4,124
  S8+DX6 baseline. Per-batch branch pins for the run are rotated to
  `memory/archive/claude-md-changelog-2026-08.md`. Earlier pin:
  **4,124 passing / 0 failing / 5 ignored** on main at the S8+DX6 collect (`cb0755bf`),
  measured on the combined tree — consistent with the disjoint branch pins (4,097 `scutemob-173`,
  4,099 `scutemob-172`). Branch-pin detail: **4,099 passing / 0 failing / 5 ignored** on branch
  `scutemob-172` at PB-DX6 close (+33 over the **4,066** merge-base baseline at `f20823b1`, measured
  on this branch before any edit. Split across the batch's six implement stages plus the fix cycle:
  the probe file `crates/engine/tests/primitives/pb_dx6_unflattened_payment_sites.rs` (stage-0
  observations converted to historical records, T1/T2/T4 turn-face-up, T3/T5/T6/T7 attack tax, T8/T9
  the copy-major order pin, T10 query-vs-charge parity, T11 the all-Phyrexian zero-mana-value case,
  T12 the wire sentinel), the permanent roster gate
  `crates/engine/tests/core/pb_dx6_turn_face_up_and_attack_tax_roster.rs` (R1–R4 with non-vacuity
  floors, since R2 and R4 are pinned **empty** and that is the shape that rots silently), four
  simulator tests proving the attack-tax plan is built at command-construction time, and two
  `mtg-card-types` unit tests for the residue predicate. The fix cycle's delta is exactly **+1** —
  the new discriminating order-pin fixture; every other repair was a 1:1 rename or an assertion
  correction. **PROTOCOL 32 → 33** computed from the failing gate's own output, both histories
  appended to with no shipped row edited, 13 sentinels re-pinned by **symbol** and then confirmed by
  a full `--workspace --no-fail-fast` run — whose residual list was **empty**, unlike PB-DX5's two
  multi-line survivors, which is a fact about this batch's sentinel population and not about the
  procedure. **HASH confirmed unmoved at 70** by executing `--test core hash_schema`, not by
  predicting it. Coverage unmoved at **1,137/1,804 = 63.0%** — 0 completeness flips, pre-committed
  in the plan and confirmed by an empty `git diff` over `crates/card-defs` and a regeneration whose
  report body came back byte-identical. Benches within noise: `full_turn_4p` 220–222 µs,
  `priority_cycle_4p` 25.5–26.0 µs — expected, since the flatten runs once per declaration, not per
  attacker.) Earlier pins: rotated to `memory/archive/claude-md-changelog-2026-08.md`.
  — and `fmt` here means `cargo fmt --check` **plus** `tools/check-defs-fmt.sh`, which is the only one
  of the two that looks at the 1,798 card defs (SR-35)
- **CI**: **LIVE and green** since 2026-07-10 (SR-1, merge `e9742dc2`) — single Ubuntu job (fmt +
  clippy + `build --workspace` + full tests) on push/PR to main + workflow_dispatch; rust-cache@v2,
  45m timeout. **Toolchain pinned (SR-11, `scutemob-63`)**: `rust-toolchain.toml` pins exact stable
  `1.95.0` and CI reads that `channel` from the file (no more floating to latest stable), so local
  `clippy -D warnings` is an authoritative CI preview. SR remediation track: original SR-1..16 all
  DONE 2026-07-10; a 2026-07-11 re-audit of the remediated baseline filed **SR-17..SR-32**, all DONE
  2026-07-14..16 (16/16 collected; full record: `docs/sr-remediation-plan.md`).
- **Abilities**: ~199 validated; 42/42 P1; 17/17 P2; 40/40 P3; 95/95 P4 implemented (9
  permanent-n/a; 1 deferred: Banding)
- **Primitives**: PB-0..PB-37 + named-letter chain
  (PB-A/B/E/J/M/S/X/Q/Q4/N/D/P/L/T/SFT/CC-{W,B,C,A}/TS/LKI-CC/CD/LKI-Power/EWC/XS/XS-E/XA/EAT/XA2/EWC-D)
  all DONE. PB-Q2/Q3/Q5 reserved.
- **Open primitive seeds**: fully retriaged 2026-07-18 (`scutemob-115`) — 65 distinct seeds
  chain-verified: **23 resolved/stale** (10 newly found silently closed by the EF/EWC/EAT/AC9 waves
  — e.g. OOS-XS-3, OOS-LKI-Power-2, OOS-TS-3/4), **16 active candidates ranked into PB-OS1..OS11**,
  7 deferred (Battle subsystem, Super Nova, protection-from-color, AC7 one-offs), 24
  dormant-0-yield. Canonical inventory + queue: `memory/primitives/oos-retriage-plan-2026-07-18.md`
  (supersedes `pb-retriage-CC.md`'s status banners).
- **Known issues**: 0 HIGH; 2 MEDIUM (pre-M8 deferred to M10+); **6 LOW open** (4 M10-gated:
  MR-M8-11, MR-B16-04/05/06; 2 permanent perf: MR-M1-18, MR-M6-14). Full:
  `docs/mtg-engine-milestone-reviews.md`.
- **Strategic Review**: `docs/mtg-engine-strategic-review.md` (2026-03-07) — **fully applied
  2026-07-26** (`scutemob-147`); all 9 action items resolved (8 done, #9 obsolete — the doc it
  targeted was retired). M11 decoupled → M11-local, M10 split into M10a/M10b, M12 downscoped,
  **web-first decided**. Its Finding 2 premise ("Tauri can't build on headless Debian") is corrected
  in-doc as stale. The doc is now a historical record, not a pending-changes list — and so, since
  2026-08-01, is `memory/m11-session-plan.md` (M11-local COMPLETE). The live plan is
  `docs/mtg-engine-roadmap.md` plus the PB-DX queue in `memory/primitives/seed-rerank-2026-08-02.md`
  §4 (v3; `seed-rerank-2026-07-27.md` §4 is SUPERSEDED, its §1-§3 still canonical).

### Machine-enforced invariants (full text: `docs/engine-invariants.md`)

> The standing invariant/gate bullets that used to live here moved to
> **`docs/engine-invariants.md`** on 2026-07-18 (they are permanent engineering
> constraints, not a rolling snapshot). One-line pointers remain below; read the matching
> section of that doc before touching the subsystem it guards. See also the nine
> non-negotiable **Architecture Invariants** further down this file.

- **SR-2** — Invariant #9 is machine-enforced: `CardDefinition.completeness` markers (62 inert / 570
  partial / 97 known-wrong per `scutemob-88`); `validate_deck` rejects non-`Complete` cards; new
  defs must be `Complete` or carry a marked note. → `docs/engine-invariants.md`
- **SR-3** — Invariant #3 is machine-enforced: `GameState` is sealed `pub(crate)`; the only mutation
  path is a `Command` through `process_command`; `cargo build --workspace` is the seal gate. →
  `docs/engine-invariants.md`
- **SR-4** — Silent failures in `effects/mod.rs` + `rules/resolution.rs` are classified LKI-fizzle
  vs engine-bug (`state::diagnostics` `expect_*` vs `lki_*`); new code there must pick a side. →
  `docs/engine-invariants.md`
- **SR-5** — Every `KeywordAbility` variant declares where its behavior lives
  (`state::keyword_registry::handling`, exhaustive; adding a variant is a compile error until
  classified). → `docs/engine-invariants.md`
- **SR-6** — Card defs compile in isolation from the engine: `mtg-card-defs` depends on `card-types`
  only, never the engine; touching an engine file leaves the 1,798 defs `Fresh`. →
  `docs/engine-invariants.md`
- **SR-7** — `PendingTrigger` is built through `PendingTrigger::blank` only; per-kind payload lives
  in `data: Option<TriggerData>`; new per-kind state goes in a `TriggerData` variant. →
  `docs/engine-invariants.md`
- **SR-35** — The card-def corpus is format-checked by `tools/check-defs-fmt.sh`, **not** `cargo
  fmt` (which checks zero of the defs and still exits 0); run the script or `cargo test --all`. →
  `docs/engine-invariants.md`
- **SR-8** — Serialized `Command`/`GameEvent`/replay-log streams carry a version tag; strict
  lockstep; `PROTOCOL_SCHEMA_FINGERPRINT` machine-checks the wire closure (adding an `Effect`
  variant is a wire change). → `docs/engine-invariants.md`
- **SR-9a** — Integration tests are 9 targets, not 297 binaries (`crates/engine/tests/<group>/`);
  never add a top-level `tests/*.rs`; a dropped `mod` line silently deletes coverage and the gate
  catches it. → `docs/engine-invariants.md`
- **SR-9c** — The golden-script corpus is triaged (208 approved / 63 retired / 0 pending; the gate
  checks the PARTITION, not these values, so re-measure rather than trust them) and cannot
  skip silently; a new assertion path must be implemented in `check_assertions`. →
  `docs/engine-invariants.md`
- **SR-9b** — The JSON-script regime and the direct-`Command` regime cross-validate on a per-step
  fingerprint; `build_initial_state` is deterministic (`sorted_zone_entries`). →
  `docs/engine-invariants.md`
- **SR-36** — An activation cost is only paid if some code pays it: `AddManaScaled` + `life_cost`
  payment paths, disjoint by construction; enumerate `all_cards()` for rosters, never grep source. →
  `docs/engine-invariants.md`
- **SR-37** — A def's PRINTED fields (mana cost, P/T, type line, ability-embedded costs, and
  oracle text) are diffed against the card from a committed Scryfall fixture; `completeness`
  never checked any of them, and 39 were wrong.
  `tools/card-field-dump` → `tools/refresh-card-fidelity-fixture.py` →
  `core::cards2_printed_field_fidelity` R1–R8 (the only place equality is decided). →
  `docs/engine-invariants.md`

### Changelog & history

- **Full PB/SR narrative** ("Last shipped" + the reverse-chronological "Last Updated" log) lives in
  **`memory/archive/claude-md-changelog-2026-07.md`** — moved there verbatim on 2026-07-18 (DOC-1v2)
  so Current State stays a true snapshot. **August 2026 opened
  `memory/archive/claude-md-changelog-2026-08.md`**, whose first entry is the verbatim
  session-by-session M11-local narrative, archived at milestone close (`scutemob-173`). PB-by-PB
  handoffs also live in `memory/workstream-state.md`; the ESM task record and git log carry the
  rest.
- **Recurrence rule** — future `/collect` and milestone-close bookkeeping appends its detailed PB/SR
  narrative to that archive file (newest first), and updates only a one-paragraph snapshot delta
  here. Start a new dated archive (`claude-md-changelog-YYYY-MM.md`) when the month turns over.
- **Last Updated**: 2026-08-12 — **PB-DX8 SHIPPED** (`scutemob-208`; v3 queue rank 10 —
  **OOS-CARDS2-7** FILED *and* CLOSED, **OOS-DP10-9** **RECORDED not closed**, rider
  **PB-DX42a** SHIPPED). A gate can only see the vocabulary it was given. Three files, all
  tests: the new oracle-text-vs-DSL cross-check, the new `ContinuousEffectDef` roster, and a
  rewritten `completeness_deviation_scan`. **Both vocabularies are DERIVED, and it took three
  measured failures to find a derivation that works** — iterated bootstrapping DRIFTS
  (`battlefield`, `library`, `graveyard` enter by iteration 3); single-pass lift returns object
  nouns; and **a vocabulary learned from the DSL's own ground truth is self-blinding on the
  target**, dropping `may` entirely, because "you may" is precisely the class the DSL cannot
  encode. §2.6's rule needs a companion clause: **derive the category from the thing being
  checked, not from the thing that already handles it correctly.** What shipped is a
  morphological closure, which cannot drift because it does not iterate. **The channel split is
  load-bearing**: a `choose`-shaped construct does not discharge a printed "you may", and
  collapsing them is what let Smuggler's Copter pass `decision_gate.rs` — which saw it only
  through the incidental `Effect::DiscardCards` inside the same unconditional `Sequence`.
  Measured: `may` **287** oracle-positive / **72** effectively-`Complete` with nothing able to
  express it, `choose` 116/2, `up_to` 70/10; union **80**, frozen mechanically (documented as
  such at write time, the PB-DP10 correction) and ratcheted; 6 reasoned
  `RECORDED_STRUCTURAL_EVIDENCE` suppressions covering **24** real defs — and those figures are
  now **printed by `t_reconciliation_report`**, not transcribed, because the first draft
  published `285` and `18`, both measured against the *pre-fix front-face-only* oracle axis and
  never re-run after the multi-face widening corrected them. **Fail-closed proven
  END-TO-END on a real def** — `lightning_bolt.rs` given "You may draw a card.", both gates RED
  naming card/channel/CR and the union 80→81, restored GREEN. **The seed's 35-def floor
  reproduces EXACTLY at HEAD**, but the derived set is NOT a superset of it — the two are
  non-nested (14 seed-only, 10 derived-only), union **45** — and the reason is the batch's second
  headline: `todo`/`deferred` live in `// TODO` COMMENTS, not compiled `Completeness` notes, so a
  derivation keyed on ONE declaration construct is short by exactly the failure mode
  `OOS-CARDS2-7` names, **reproduced inside the fix for it**.
  **The batch committed its own subject matter three times and execution caught every one**: the
  oracle axis read `def.oracle_text` alone while `CardFace` carries its OWN (blind to every
  transformed face and Adventure half — found by the inverse-method census, not by a test);
  the deviation scan matched whole SOURCE while its needles were derived from PROSE, so
  `drawcards` (== `Effect::DrawCards`) scored **203 files / 127 unmarked / 37% precision**
  against the derivation's own **20/1/95%** and would have blown the freeze past 150 silently;
  and a population ratchet filtered its own denominator down to the roster it was checking and
  **could never redden**, caught only because revert row V4 had to demonstrate red. That last is
  the **third instance in three batches** — PB-DX7's V14 and this batch's own
  `t_optional_false_is_not_evidence` are the others — and the common cause is worth naming:
  **a checker whose reference set is derived from the thing it checks can never disagree with
  it.** 30 revert rows, 29 RED, **1 honestly UNDISCRIMINATED** (`PROSE_FIELDS`, inherited rather
  than earned, disclosed in the module doc). Corrections carried back: the brief's Tier-A dedupe
  note was imprecise in both directions, and the adjudication's structural layer-querying axis is
  **not** a general proxy (`Condition::ControlLandWithSubtypes` reaches
  `characteristics_for_condition` with no `TargetFilter`) — so `t7` pins the coincidence and
  **PB-DX42b's rank argument inherits the caveat**. Tests **4,558** (+31, itemised by name);
  coverage unmoved at **62.8%** by regeneration; PROTOCOL **35** / HASH **74** gate-executed and
  unmoved; **0 source lines, 0 card-def edits**. Seeds: **OOS-CARDS2-7 filed and CLOSED**,
  **OOS-DX8-1..8** filed.
  **The `/review` cycle found 4 MEDIUM / 6 LOW and all 10 were taken — and the reviewer, who had
  a shell and used it, DEFEATED two of the three gates by execution.** (1) Narrowing the deviation
  scan to `//` comments silently dropped `/* */` blocks: the byte-identical sentence reddened as a
  line comment and left every test green as a block comment — `OOS-DX32-6`'s class, latent only
  because the corpus happens to carry zero such comments today. (2) Evidence is scoped to the
  **def**, not the **clause**, so a printed "may" appended to a `Complete` def whose
  `optional: true` belongs to an unrelated clause is invisible — **24** `Complete` defs are
  exempted by a single piece of evidence, now stated as a second recall bound rather than
  discovered later. Also taken: a doc comment citing a test **that did not exist** (the precise
  failure the same file cites `decision_gate.rs`'s precedent for); a
  **compile-time-tautological** test that compared two `const` array lengths and then asserted
  `A || true` (rewritten to parse the struct declaration; a one-field desync now reddens **5**
  tests where it reddened none); six Tier-A needles that are ordinary English clearing the
  concentration floor on base rate, kept and stated as a precision bound rather than tuned away;
  and a count ceiling whose comment claimed a per-entry promise it cannot keep — corrected, with
  the note that `decision_gate.rs`'s identical construct carries the same overclaim.
  Full handoff: `memory/workstream-state.md`; measurements, the 30-row revert matrix and the
  fix-cycle table: `memory/primitives/pb-DX8-execution-notes.md`.
- **Prior**: 2026-08-11 — **PB-DX7 SHIPPED** (`scutemob-207`; v3 queue rank 9,
  closing `OOS-DP7-11`, `OOS-DP9-13`, and both riders `OOS-DP10-1` and `OOS-DP9-10`'s
  residual). A gate that reported success while checking nothing. **Both holes were
  REPRODUCED at HEAD before anything changed** — deleting a live field from the
  path-qualified `MergedComponent` impl left **all 21 gates green, including
  `stream_fingerprint_is_pinned`** (no seed row claims that half: the canonical fixture
  carries no merged component), and the enum demo was green **and** `clippy -D warnings`
  clean, because `..` silences `unused_variables`. Part A keys `hashinto_impl_bodies()` on
  the bare name with **zero call sites renamed**, so the hole cannot reopen by spelling.
  **The durable lesson is about briefs, not code: a scope that is true about a subset reads
  exactly like a scope that is complete.** The brief scoped the enum half to "the 10
  path-qualified enums" — a true sentence, and irrelevant, since path qualification has
  nothing to do with the enum half and **all 79** hashed enums were outside the struct gate.
  Obeying it would have closed `OOS-DP9-13` on paper with 69 enums uncovered. Final scope:
  **79 enums / 1,252 variants / 1,097 variant fields**. **Three further holes of the same
  family surfaced only by widening, each found by refusing a plausible "this is fine"**:
  the coverage predicate could not tell `self.f.hash_into()` from `self.f.is_some()
  .hash_into()`, so 4 sites passed as covered while their values never reached the hasher
  (`OOS-DX7-2`, closed by new `PARTIALLY_HASHED` categories — the enum half was nearly
  omitted as a scope call, which would have left two halves of one file disagreeing about
  coverage on the same field name); `Effect` reuses **9 discriminants across 18 variants**
  while its comments called them unique, and the first disposition ("subsequent field bytes
  differ") was an assertion of the same shape that let `OOS-SIM2-6` survive 4.5 months —
  settled instead by an **executed** pairwise-distinctness experiment over all 18, plus a
  ratchet so a 10th cannot appear (`OOS-DX7-1`); and **`GameState` was carved out of the
  field gate entirely**, with 3 of its 45 fields reaching no hash and no stated exclusion
  list (`OOS-DX7-3`). **34 revert rows (24 numbered + 10 in the fix cycle), all executed red
  then restored, none
  UNDISCRIMINATED — and two caught real bugs before shipping**: V14 exposed a **false
  negative in the new dead-entry checker itself** (it searched for the literal tuple-index
  string `"0"` instead of the actual pattern binding, passing GREEN where it should have
  failed RED — this batch's own subject matter recurring inside its implementation, caught
  only because every row had to *demonstrate* red), and V18 forced an artificial digest
  collision rather than assuming the detector fired. Tests **4,527** (+19, itemised by test
  name); coverage unmoved at **62.8%** by regeneration; PROTOCOL **35** / HASH **74**
  gate-executed and unmoved; **0 non-comment `hash.rs` lines**, 0 card-def edits.
  **The `/review` cycle found 2 HIGH / 5 MEDIUM / 9 LOW, all 16 taken, and both HIGHs were
  this batch committing its own subject matter — verified by EXECUTION, since the reviewer had
  no shell.** (1) The new unordered-container ratchet counted the literal `HashSet<` spelling,
  which is the type-ANNOTATION form and the **minority** idiom in this tree: `casting.rs` has
  **0** annotations and **9** constructions and was pinned at ceiling **0**. Appending the exact
  `OOS-DP9-10` defect to `layers.rs` with `HashMap::new()` + `into_iter().max_by_key()` left all
  three tests **green** — and V5 had reddened only because its probe used the one spelling the
  gate could see, i.e. **the gate was proven with the single input it handled**. Needle widened
  to whole-token; **27 across 6 files → 85 across 9**, each of the 85 traced to a named variable
  and classified (the extra 58 are imports, parameter restatements, `.clone()`s and
  empty-literal arguments — no new hazard). The module doc had called type-inferred construction
  "deliberately obscure code review would reject"; it is ordinary Rust. (2) `FieldCoverage::Full`
  meant "the token appears", not "the value is hashed": `let _ = may_fail_to_find;` on the
  seed's **own card** was 33/33 green and clippy-clean with the field gone from the stream —
  **verbatim `OOS-DP9-13`'s sentence, so that closure did not hold when first claimed.** The
  fail-open `else` is now a fail-closed `Unverified`. Also taken: an empty arm
  (`GiftType::Food => {}`) passed both the enum gate and the discriminant ratchet; the Named
  branch accepted `_` bindings while Tuple rejected them; a hand-hashed struct with no `HashInto`
  hit the `else { continue }` Part A existed to remove; the GameState gate used the very matcher
  this batch had diagnosed; and **both `PARTIALLY_HASHED_VARIANT_FIELDS` citations pointed at the
  impl header rather than the arms, with `ActivatedAbility` carrying no in-source comment at all
  — a reason asserting documentation that did not exist, approved by the coordinator without
  opening the lines.** **The M5 fix's own first draft then repeated H2**: it used bare presence
  and its revert proof PASSED when it should have failed, caught before shipping — the third
  instance in one batch. Two reviewer recommendations were **declined with reasons rather than
  buried**: the `OOS-DP9-10` rider stays (deferring leaves a wrong count and a gate that
  green-lights its own defect) and the 18-sample digest experiment stays (it is the only executed
  evidence behind `OOS-DX7-1`). Both defeats re-executed against the fixed gates by the
  coordinator: RED.
  **Corrections carried back into the rows themselves**: the seed's cite
  `hash_schema.rs:1540-1541` names the wrong symbol; the implement phase's "26 revert rows"
  is **24**; `OOS-DP10-1`'s "cross-checked BY VALUE" was a **floor** check with one floor
  *below* the live count, so a one-def divergence passed in silence; and a premise the
  **coordinator** asserted — that `HASH_SCHEMA_HISTORY` rows inherited the discriminant
  error — was checked and found **false**, so a clean-result note was recorded rather than a
  correction invented to fit the instruction. **A brief, a comment and a coordinator's
  message are each a claim like any other.** Full handoff: `memory/workstream-state.md`;
  measurements and revert matrix: `memory/primitives/pb-DX7-execution-notes.md`.
- **Prior**: 2026-08-11 — **PB-DX26 SHIPPED** (`scutemob-206`; v3 queue rank 8,
  closing `OOS-CARDS1-3`, `OOS-CARDS1-1` and `OOS-DX3b-1`). A printed ability that did not
  exist. `keyword_registry.rs`'s `K::Equip` arm is a `KeywordHandling::Marker` and a marker
  **synthesises nothing**, so 21 defs carrying only
  `AbilityDefinition::Keyword(KeywordAbility::Equip)` had no `ActivatedAbility` at all — no
  offer, no index, no `Command` that could reach one. Where `OOS-M11-10(equip)` was "the
  picker never asks for a target", this is **"there is no action to pick"**, one link sooner
  and on a larger population: **10 of the 21 were deck-legal `Complete`**, nine by the
  `#[default]` derive. All 21 now carry the MCP-verified printed ability (CR 702.6a target
  creature you control, CR 702.6d sorcery speed) **beside a retained keyword marker**;
  `darksteel_garrison` gets CR 702.67a's `TargetPermanentWithFilter(Land + You)` —
  explicitly not the equip repair's creature filter; `guardian_project` gets
  `is_nontoken: true` and stays `known_wrong` on the name-uniqueness half alone.
  **The "~4-6 flips" estimate was wrong in an instructive direction**: the ten deck-legal
  defs were ALREADY `Complete`, so repairing them flips nothing, and the batch's single flip
  came from an eleventh def nobody was counting (`sword_of_body_and_mind`, whose `partial`
  note named the missing Equip {2} as its only blocker). **A card-def-only batch is not
  automatically an index-neutral one**: `Command::ActivateAbility { ability_index }` indexes
  activated abilities in declaration order, and the first pass silently renumbered Umezawa's
  Jitte's PB-EF7 modal ability 0 → 1 — caught only by this batch's own `t3` (`OOS-DX26-3`).
  **The inverse census earned its keep**: R4/R5 start from the printed TYPE LINE rather than
  the keyword marker and found `quietus_spike` + `sting_the_glinting_dagger`, which print
  "Equip {N}" and carry neither marker nor ability, so neither the seed's grep nor R1's
  `all_cards()` walk could ever see them (both `Inert`, 0 deck-legal blast radius —
  `OOS-DX26-1`). `cards1_equip_target_roster` R1 re-pinned **17 → 38** and its `Effect` match
  made **recursive** over all ten nesting sites — the §2.7 hazard, proven live by revert row
  V6b; `t7b` strengthened from a name-set pin to a requirement-shape pin. Revert matrix: **15
  rows — 13 RED as required, 1 CONTROL (V6a, must be green), 1 UNDISCRIMINATED** (V4b,
  shadowed by `OOS-DX20-7`'s legacy guard, disclosed in the test's own doc rather than
  glossed). **Review: 1 HIGH / 6 MEDIUM / 11 LOW, all 18 taken**, and its two sharpest were
  this batch's own failure modes recurring inside it — a `Complete` def declaring a MANDATORY
  target for a printed "up to one target" (`sword_of_light_and_shadow`, which lost its
  unconditional life gain under CR 603.3d), and **an eleventh `Effect` nesting site that was
  already in the enum while the new gate claimed to be exhaustive** (`RollDice`'s
  `Vec<(u32, u32, Effect)>`, invisible to a `Box`/`Vec` count — and the residual the gate
  DID state named a form that would have fired it). Tests **4,508** (+17); coverage net
  unmoved at **62.8%**; PROTOCOL **35** / HASH **74** gate-executed and unmoved. Seeds: three
  CLOSED, **OOS-DX26-1..6** filed. Durable lesson: **a roster derived from a keyword marker
  measures the marker, not the printed card** — the fix for a short census is a second axis,
  not a better grep.
  Full handoff: `memory/workstream-state.md`; measurements and revert matrix:
  `memory/primitives/pb-DX26-fail-before-2026-08-11.md`.
- **Prior**: 2026-08-06 — **PB-DX25c SHIPPED** (`scutemob-205`; v3 queue rank 7c,
  closing `OOS-DX25b-3`). A spell you can retarget is a spell you can retarget LEGALLY.
  New `StackObject.target_requirements` (hashed) + `rules::retarget::plan_target_change`
  delegate the whole "which object or player may become the new target" decision to
  `casting::validate_targets_inner` — the same collective arithmetic a real cast is
  checked against (CR 115.3/115.7e/115.7a all-or-nothing) — closing BOTH the object
  branch (the filed defect) and an independently-reachable player-branch defect the
  filing missed (no `TargetOpponent` check, `has_lost`-only not `has_conceded`).
  Fail-closed on a missing requirement list. `t9_object_target_redirect_ignores_the_
  original_requirement` inverted (renamed `...obeys_the_original_requirement`) with a
  new `t9b` sibling proving the fix isn't "never redirect". Two structural findings
  surfaced only by executing tests: `TargetSpellWithSingleTarget`/`TargetSpellOrAbility
  WithSingleTarget` cannot observe the ACTIVELY-RESOLVING spell as a redirect candidate
  (its own `StackObject` entry is popped before its effect runs — resolution.rs's own
  documented order), and `StubProvider`'s offer layer reads `obj.characteristics.
  mana_cost` directly rather than the registry def, a third instance of the "ObjectSpec
  ::card() is naked" gotcha. Tests **4,491** (+22); coverage unmoved **1,133/1,803 =
  62.8%**, proven by regeneration; PROTOCOL **35** / HASH **73 → 74** gate-executed.
  **Correction (fix cycle 2, this bullet was stale by one)**: the chooser-first row
  (V9) was CLOSED in fix cycle 1 by `t3b_chooser_first_preference_beats_seat_order` —
  **16 of 19 revert-matrix rows discriminate; 3 remain honestly UNDISCRIMINATED**
  (V3 final-set re-validation, V7 `has_conceded` — both shadowed by a redundant
  downstream check — and V13 copy propagation, blocked behind `OOS-DX25b-2`). Fix
  cycle 2 CLOSED **OOS-DX25c-5** (a `TargetSpell`/`TargetSpellWithFilter` victim could
  be redirected onto its OWN card): a two-line `self_id` guard mirroring the two
  single-target arms, proven cast-path-neutral (cast-time validation runs before the
  CR 400.7 zone-move id change) and red by an executed revert; new probes
  `t7b_plain_target_spell_victim_cannot_redirect_onto_its_own_card`,
  `bb1_bolt_bend_object_branch_lands_only_on_a_legal_creature_never_a_land`,
  `bb2_bolt_bend_object_branch_no_legal_target_leaves_targets_unchanged` (closing
  AC 6303's `bolt_bend` object-branch gap) and `s1b_bot_driven_misdirection_object_
  branch_redirects_legally` (closing AC 6304's object-branch gap; S1 alone only
  ever exercised the player branch). Seeds: **OOS-DX25b-3 CLOSED**; filed
  **OOS-DX25c-1..6**, of which **OOS-DX25c-5 is now CLOSED** and **OOS-DX25c-6
  stays open** (the resolving spell's own `StackObject` entry is popped before its
  effect runs, so Misdirection can still never target ITSELF as its own new
  victim card). Full handoff: `memory/workstream-state.md`; measurements and
  revert matrix: `memory/primitives/pb-DX25c-execution-notes.md`.
- **Prior**: 2026-08-06 — **PB-DX25b SHIPPED** (`scutemob-204`; v3 queue rank 7b).
  A spell you can target is a spell you can retarget. `validate_object_satisfies_requirement`
  resolved the announced id through `state.objects` — the **card** — and then compared it to
  `so.id`, a **stack-entry** id. `next_object_id` mints both namespaces from one monotone
  counter, so the comparison **type-checks and can never match**: `misdirection` and
  `bolt_bend` were `Complete`, deck-legal, and unable to resolve a legal target ever.
  **The brief's "validation-site only" was short by three sites, and obeying it would have
  shipped a strictly worse cast than HEAD**: `Effect::ChangeTargets` — the effect *both*
  cards use — matches the same announced id against stack-entry ids at three more places, so
  a validation-only fix takes the mana, announces the target, and then **silently does
  nothing** at resolution. `Effect::CopySpellOnStack` is a fourth site (latent) and
  `Effect::CounterSpell` had open-coded the correct rule as a fifth. **The fix is
  structural**: one `state::stack_registry::stack_index_for_announced_target` beside PB-DX25's
  `card_in_stack_zone`, encoding `so.id == announced || (!so.is_copy && card_in_stack_zone(..)
  == Some(announced))` ONCE and consumed by all five. The `!so.is_copy` guard is load-bearing
  **twice** — disambiguation (a copy's `kind` is cloned wholesale, so one card id would match
  the original *and* every copy) and the CR 702.99c cipher-copy exile leak. The `is_spell`
  guard is **kept though now production-unreachable**: it is the only thing distinguishing the
  two requirements, and deleting it becomes CR-wrong the day `OOS-DX25b-1` closes.
  **The existing tests were green while testing a fiction** — `make_test_stack_spell` built
  `StackObject { id, kind: Spell { source_object: id } }`, collapsing the two id spaces into a
  configuration no real cast can produce; three test files carried that fixture and
  `tests/rules/copy_redirect.rs` still has eight more, now disclosed. Review 1 HIGH / 5 MEDIUM
  / 6 LOW, **all 12 taken**, and **the HIGH was a plan deliverable the implement phase silently
  dropped** — the CR 115.7a wrong-way-round probe — with an execution note that misstated the
  plan as having deferred it; the plan deferred the *fix*, not the *probe*. **The reviewer also
  defeated the new R5 gate three ways**; two are now caught and the third is stated as a
  permanent structural residual in the gate's own doc rather than papered over — a gate that
  overclaims its reach being this batch's own subject matter. The reviewer re-derived the
  census by the inverse method and confirmed **no sixth site**. Tests **4,469 / 0 / 5** (+17);
  coverage unmoved **1,133/1,803 = 62.8%**, proven by regeneration; PROTOCOL **35** / HASH
  **73** gate-executed and unmoved; 0 simulator, view-model, card-types or `tools/` lines, and
  all 4 card-def edits comment-only. Seeds: **OOS-DX25-3 CLOSED** (its row now carries four
  corrections to its own claims); filed **OOS-DX25b-1..5**, of which **OOS-DX25b-3 is LIVE on
  the same two `Complete` defs** — this batch is what makes CR 115.7a's unchecked object-target
  redirect reachable, so a Misdirected "destroy target creature" now destroys a basic land.
  Durable lesson: **a fixture that collapses two id spaces makes a test green by removing the
  only condition under which the code is wrong** — and the enumeration lesson recurred, the
  brief, the plan and the batch's own notes each being short about a different thing.
  Full handoff: `memory/workstream-state.md`; measurements and revert matrix:
  `memory/primitives/pb-DX25b-execution-notes.md`.
- **Prior**: 2026-08-05 — **PB-DX25 SHIPPED** (`scutemob-203`; v3 queue rank 7).
  A countered spell is countered, whichever shape it arrived in. `Effect::CounterSpell`
  decided "does this stack object own a card" by matching the **variant name**, so
  `MutatingCreatureSpell` fell through a `_ =>` catch-all. **The seed and the queue row both
  ranked the three shapes backwards, and the batch's own review then did the same thing to the
  batch.** (a), filed as the stranding, was **never independently live** — Ward cannot reach a
  mutate spell, because the mutate target rides in `AdditionalCost::Mutate` and never enters
  `spell_targets` (`OOS-DX25-1`) — so **(a) is what fixing (c) ALONE would have created**, a
  permanent `ZoneId::Stack` leak in place of a silent no-op. **A "just fix the lookup" change was
  strictly worse than HEAD**, which is why (c) and (a) had to land in one commit. (b) is
  unreachable **three** ways, not the memo's one. And (c) is worse than "silent no-op" sounds:
  `TargetSpell` validates against the **card**, which a mutate spell really does have in
  `ZoneId::Stack`, so the engine **offers the target, takes the mana, and does nothing**.
  **The fix is structural**: one engine-side `state::stack_registry::card_in_stack_zone`,
  exhaustive over all 27 kinds with **no wildcard**, consumed by **both** counter paths
  (`Effect::CounterSpell` and `resolution.rs::counter_stack_object`) — a 28th card-carrying
  variant is now a compile error until classified. **The simulator's `stack_card_of` is
  deliberately NOT unified with it**: `check_stack_consistency` exists to catch the engine
  getting this classification wrong, and a verifier that reads the engine's own answer goes
  **silent** on exactly the defect it was written for. Both sides stay exhaustive independently;
  a behavioural probe proves they agree without coupling them. Review 0 HIGH / 6 MEDIUM / 3 LOW
  + 7 folded notes, **all taken**, and its three sharpest findings were **this batch's own
  failure mode recurring inside it**: the plan's "FOUR classification sites" census was short by
  two — and one of the two (`abilities.rs:6736`) was wrong in the **same direction** as the
  defect being fixed while its sibling one function over was right; the SR-36 roster measured
  **48** live-wrong pairs by walking `Effect::CounterSpell` alone, blind to
  `Effect::CounterUnlessPays` **delegating into the same arm** (`mana_leak`, `mana_tithe`,
  `make_disappear`, all `Complete`) — the real figure is **66**; and T6's advertised
  "non-vacuity" compared a hand-written fixture to **itself**. Durable lesson: **an enumeration
  is only as wide as the variant list it walks, and an exhaustive match proves nothing about the
  callers that never ask it.** Tests **4,452 / 0 / 5** (+17); coverage unmoved
  **1,133/1,803 = 62.8%**, proven by regeneration; PROTOCOL **35** / HASH **73** gate-executed
  and unmoved; 0 card-def, card-types, view-model or `tools/` lines. Seeds: **OOS-SIM3-5
  CLOSED** (its row now carries four corrections to its own claims); filed **OOS-DX25-1..6**,
  of which **OOS-DX25-3** is **LIVE on two `Complete` deck-legal defs** — `misdirection` and
  `bolt_bend` can never resolve a legal target, the same id-space confusion one function over,
  behind negative tests that pass **vacuously** because the requirement refuses everything.
  Full handoff: `memory/workstream-state.md`; measurements and revert matrix:
  `memory/primitives/pb-DX25-execution-notes.md`.
- **Prior**: 2026-08-05 — **PB-DX24 SHIPPED** (`scutemob-202`; v3 queue rank 6).
  A zone-scoped ability finally functions in its zone. `AbilityDefinition::Triggered`
  carries `trigger_zone`; the runtime `TriggeredAbilityDef` has no home for it, so **33 of
  the lowering's 34 arms swallowed it** and `nether_traitor` (`Complete`, deck-legal) had
  its graveyard ability installed on the **battlefield object** — functioning from exactly
  the wrong zone. CR **113.6m** is load-bearing. **The brief and the queue row were both
  short by a whole half, and the "one-line, wire-neutral" fix alone would have shipped a
  card that fires NOWHERE**: `collect_graveyard_carddef_triggers` had a single `fires` arm
  (`PermanentEnteredBattlefield`, written for Bloodghast), so a `WheneverCreatureDies`
  graveyard trigger had **no dispatch path at all** — criterion 6205's "both directions" is
  what forced the discovery. **Uniformity is structural, not 33 more `continue`s**: the
  trigger-lowering region becomes `build_face_triggered_abilities`, filtered **once** at its
  single call site through `lowers_onto_the_battlefield` (exhaustive on `TriggerZone`), with
  the old per-arm guard **deleted** so there is one mechanism; two source gates fail if a
  41st arm re-swallows it, and the gate's comment-stripping was itself proven load-bearing
  by executing both variants. The new death arm mirrors the battlefield arm clause for
  clause (CR 108.4a owner-as-controller, CR 400.7 `exclude_self` on the **graveyard** id —
  a battlefield-only comparison fails **open, silently** — CR 111.7, CR 603.10a/613.1d) plus
  a CR 603.10a look-back guard applied to **that arm only**, because ETB triggers are not in
  603.10a's list. **`OOS-DX1-4`'s "6 latent queue sites" was right for the wrong reason**:
  the SR-36 enumeration measured **0** corpus defs with any Q-shape on a back face, so all
  seven are latent and every probe is synthetic. **Q5's rule was wrong in the plan AND in
  the review**: not CR 712.2 (face symbols) but CR **712.16**; CR 712.15 makes the site
  reachable, and CR **712.15a** ("turned face up → its **front** face up") makes the
  front-face read **CR-correct** rather than merely unreachable. Review 0 HIGH / 6 MEDIUM /
  7 LOW, all 13 taken, and **two were the coordinator's own**: a seed row claimed
  "live-wrong on 2 `Complete` defs" when both doublers are `partial`, and framed a
  **pre-existing** class (the ETB arms have had a graveyard-sourced pairing since PB-35) as
  new — re-measured, **no pairing is deck-legal on both halves in either direction**. Also
  caught: **a gate that was green while the invariant it pinned was already violated**
  (it scanned for a literal `is_transformed = true`; `face.rs:97` writes a computed bool,
  which the batch's own probes assert). Tests **4,435 / 0 / 5** (+22); coverage unmoved
  **1,133/1,803 = 62.8%** (comment-only card-def edit, proven by regeneration);
  PROTOCOL **35** / HASH **73** gate-executed and unmoved. Seeds: **OOS-DX1-3** and
  **OOS-DX1-4** CLOSED (each row also corrects its own original claims); filed
  **OOS-DX24-1..9**, of which **OOS-DX24-9** is **LIVE on a `Complete` def** (CR 118.12's
  optional cost is engine-chosen; the class is the pre-existing DP-19 shape, but this batch
  is what makes the instance reachable). Durable lesson: **a guard, a gate and a claim each
  have a subject, and "it passes" only tells you about the subject it actually has** — three
  findings were one shape, each true about what it examined and wrong about what it was
  taken to mean. Full handoff: `memory/workstream-state.md`; measurements and revert matrix:
  `memory/primitives/pb-DX24-execution-notes.md`.
- **Prior**: 2026-08-05 — **PB-DX23 SHIPPED** (`scutemob-201`; v3 queue rank 5).
  Dredge is answerable, by anyone. `grep -rn "ChooseDredge" crates/simulator/src/ tools/`
  returned **zero**: the engine had the command, the event and a gated handler, and
  **nothing could reach any of it**. Not a lost option — a permanent draw-cadence
  corruption, and the probe **measured** it rather than arguing it: on a real 2-player
  `LocalGame`, both bot seats, no state pokes, **2** dredge offers fired, **1** card was
  drawn where **2** were owed, and **1** `PendingDraw` survived to the halt. Each turn's
  draw defers and the *next* turn's discharges the stale entry — forever one behind, off a
  library reordered a full turn later. `LegalAction::ChooseDredge { card, mill }` is now
  emitted, mapped, scored and labelled; `rules::queries::dredge_options` makes the offer
  and the engine's own scan **one arithmetic** (the PB-DX20 shape) — and **the SR-5 keyword
  registry caught what the brief missed**, `queries.rs` being a `Dredge` handling site.
  **The brief was short by one site and its naive fix would have shipped a new bug**: there
  are **three** `offer_dredge: false` resume sites, not two, and an *unconditional* tail
  flip makes the REOPENED `OOS-DX2-3` **live from the corpus's only dredge card** — the
  implicit discharge's tail pushes one entry, the outer call pushes another, two
  dredge-originated entries. The flag is threaded; the exact trace is pinned. **Why PB-DP5
  §3.3 does not extend to the tail**: it argues about the SAME draw event; CR 121.2 makes
  "draw three" three draws and CR 614.11a/121.6b resume the sequence, so each resumed draw
  is its own fresh "would draw". **The brief's UI prescription was one layer off** — the
  choice lives in the `LegalAction` (the `PayEcho` shape), so **zero frontend production
  lines**; a `BlockingDecision` variant would be CR-wrong (CR 702.52a is "you **may**") and
  a HASH bump. **The review found the batch's own overclaim, and it was this batch's own
  failure mode**: the suppression rule was documented as removing the decline-forever loop
  *"structurally"*, but it is keyed on the **graveyard** while the answered entry is keyed
  **FIFO** — the same shape of claim `OOS-DX2-3` was wrongly closed on, made inside the
  batch dispatched not to repeat it. Fixed with a third conjunct whose limits are stated,
  not glossed. The reviewer's suggested bot-side repeat cap was **declined on precedent**:
  PB-DX21 deleted exactly that shape. Tests **4,413 / 0 / 5** (+15); coverage unmoved
  **1,133/1,803 = 62.8%** (comment-only card-def edit). Review 0 HIGH / 4 MEDIUM / 9 LOW,
  all 13 taken. Durable lesson: **a guard keyed on one thing cannot police a decision keyed
  on another** — both halves were right about their own subject and the pair was wrong.
  Seeds: **OOS-DX2-5** and **OOS-DX2-2** CLOSED; **OOS-DX2-7** RECORDED as an AUTO-CHOSEN
  audit row (still open); **OOS-DX2-3** STAYS REOPENED, pin byte-unedited; filed
  **OOS-DX23-1..4, -6, -7, -8**. Full handoff: `memory/workstream-state.md`; measurements
  and revert matrix: `memory/primitives/pb-DX23-execution-notes.md`.
- **Prior**: 2026-08-04 — **PB-DX21 SHIPPED** (`scutemob-200`; v3 queue rank 3).
  Declaring attackers is once per combat. CR 508.1 makes it a turn-based action;
  `handle_declare_attackers` guarded on step, active player, priority and per-attacker legality
  and **on nothing else**, so a second `DeclareAttackers` reran the whole body. **Three
  consequences, not the seed's one** — the map is `insert`ed into, so a repeated id **moves that
  creature's attack target mid-combat**; a fresh `AttackersDeclared` + `check_triggers` +
  `flush_pending_triggers` **re-fires every attack trigger** (the one a human hits first); and
  `attackers_declared_this_turn` is clobbered, killing `windbrisk_heights`/`legions_landing`'s
  raid gate. **A fourth was found**: each accepted declaration resets `players_passed`, so a
  re-declaring client holds the CR 117.4 pass-round open with **no attacker changing**.
  **The brief's preferred one-liner would have shipped a new bug.** It said prefer reading
  `combat.attackers` to avoid a HASH bump; refuted three ways — CR 508.1a's "*if any*" + CR 508.8
  make an **empty** declaration a completed one (and `params.rs:474` sends exactly that), and
  **CR 508.4/506.3 "put onto the battlefield attacking" inserts straight into `combat.attackers`
  at four sites** without any declaration, so that guard would have refused a player's **first,
  legal** declaration. `CombatState` gains `attackers_declared: bool`, hashed; HASH **72 → 73**
  gate-computed, PROTOCOL **35** gate-executed unmoved. **CR 509.1a verified covered, NOT
  widened.** Both client-side mitigations deleted **with their mechanism** — the bot's
  `RepeatKey::DeclareAttackers` cap and the playthrough's `PolicyState` — which forced
  `legal_actions.rs` to suppress the offer (SR-38); the S8 playthrough's *"a rejection means the
  offer was wrong"* assertion, green **with no cap**, is the closure proof. Tests **4,398 / 0 / 5**
  (+10); coverage unmoved **1,133/1,803 = 62.8%** (comment-only card-def edits). Review 0 HIGH /
  7 MEDIUM / 8 LOW, all 15 taken — the two that mattered: a **card-def comment asserted a defect
  the card does not have** (`legions_landing` is a CR 508.3d *per-declaration* trigger; CR 508.6
  was mis-cited, and following the note would have **regressed** it — `OOS-DX21-1` re-scoped to
  `windbrisk_heights` alone), and **four probes were reading state their failing call never
  touched** — `process_command`'s `Err` arm carries no `GameState`, so "the rejection mutated
  nothing" is structurally vacuous through it; T4's CR 117.4 pin was repaired to the direct-handler
  idiom and re-watched failing. Durable lesson: **a guard keyed on a collection cannot tell "chose
  nothing" from "has not chosen", and cannot tell your own writes from someone else's.** Seeds:
  **OOS-M11-9 CLOSED**; filed **OOS-DX21-1..7**. Full handoff: `memory/workstream-state.md`;
  revert matrix and measurements: `memory/primitives/pb-DX21-execution-notes.md`.
- **Prior**: 2026-08-04 — **PB-DX20 SHIPPED** (`scutemob-198`; v3 queue rank 2).
  The offer layer can now see a keyword-carried target requirement. An Aura's CR 303.4a
  requirement lives in `KeywordAbility::Enchant`, which `casting.rs` special-cased and
  `spell_target_requirements` could not see, so 13 deck-legal `Complete` Auras rendered a
  zero-target action and 422'd on click. One **total** derivation now serves both sides —
  `casting::enchant_target_to_requirement` (exhaustive over all 9 `EnchantTarget` variants,
  no wildcard arm) + `aura_spell_target_requirements`, consumed by `handle_cast_spell` AND
  `rules::queries::spell_target_requirements`, so the two are one arithmetic rather than two
  that agree. **No new `TargetRequirement` variant.** **The brief's prescription was one layer
  off**: it named `legal_actions.rs`, which took 0 lines — the browser reads the engine query,
  and `tools/play-server` needed 0 production lines. The CR 303.4a gate is KEPT deliberately:
  it is the **SBA's own** predicate, and cast-vs-SBA agreement is a different property from
  offer-vs-cast agreement. Reconfigure (`OOS-CARDS1-2`) gets CR 702.151a's *another* target
  creature you control (`exclude_self: true` — the equip repair was NOT copied); its live
  symptom was worse than its row said — a zero-target attach **paid the mana and fizzled in
  silence**. `KNOWN_FALSE_OFFERS` is deleted with its whole mechanism; any refusal in that
  driver is now fatal, which is what proves the closure. **The SR-5 keyword registry caught
  what two green targeted test runs missed** — `queries.rs` is an Enchant handling site.
  Tests **4,388 / 0 / 5** (+15); 0 card-def lines, coverage unmoved **1,133/1,803 = 62.8%**;
  PROTOCOL **35** / HASH **72** gate-executed and unmoved. Review 1 HIGH / 5 MEDIUM / 7 LOW,
  all 13 taken — **the HIGH is not the primitive** (the reviewer re-derived its 9-variant
  equivalence by hand and found it exact) but a card def inside the batch's own 13:
  `imprisoned_in_the_moon` declares `Permanent` for a printed "creature, land, or
  planeswalker", unreachable before this batch and human-reachable after it; filed
  `OOS-DX20-10` with a wrong-way-round roster pin, not fixed, because `EnchantFilter` has no
  OR over card types and adding one moves HASH. Durable lesson: **a differential probe between
  two consumers of one function proves consistency, not correctness.** Seeds:
  **OOS-CARDS2-4** + **OOS-CARDS1-2** CLOSED; filed **OOS-DX20-1..10**. Full handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-08-03 — **PB-DX32 SHIPPED** (`scutemob-197`, merge `685aa1c4`; v3 queue rank 19,
  promoted per FEEDBACK-1 §2.3, user-approved). The fuzzer's OUTPUT now means something:
  `GameResult` carries an SR-38 rejection channel + a promoted waste tally, both behind
  ratchets pinned at measured values; the CR 704.3 orphaned-token class is split off as
  transient and answered by a strictly stronger end-state check, so hard violations go
  **426 → 125** and crash files **16 → 6**; a source gate keys a runtime decision-point
  counter to `decision_site_walk.rs`'s `ROWS`. **Every pre-PB-DX22 fuzz number was
  re-measured at HEAD first** — `OOS-SIM3-4`'s "929 of 938" was both stale and a sample.
  **Criterion (c) is met literally, not colloquially**: `--stop-on-error` still halts, now
  on undiagnosed `player_consistency` (26.8% of a run, `OOS-DX32-1`), deliberately NOT
  suppressed. Tests **4,373 / 0 / 5** (+15); 0 engine-source lines, 0 card-def lines, 0
  wire, `tools/` exactly `+1 -0`; PROTOCOL **35** / HASH **72** gate-executed and unmoved;
  coverage unmoved **1,133/1,803 = 62.8%**. Seeds: **OOS-SIM3-3**, **OOS-SIM3-4**,
  **OOS-CARDS2-3** CLOSED, **OOS-SIM3-2** PARTIAL (#10 served at run scope; #11 SBA
  idempotency still unwritten); filed **OOS-DX32-1..10**. Review 0 HIGH / 8 MEDIUM / 10 LOW,
  all 18 taken — **`OOS-DX32-6` was proven by experiment, not argued**: a `/* */`-wrapped
  roster row left the compiled roster while the gate AND all 12 probes stayed green. Full
  handoff: `memory/workstream-state.md`.
- **Prior**: 2026-08-03 — **PB-DX22 SHIPPED** (`scutemob-196`; v3 queue rank 4,
  FEEDBACK-1 §2.1 row 1). The fuzzer is an instrument: it shuffles every library from the
  game's own seeded RNG (CR 103.3) and registers commanders (CR 903.6) via the new shared
  `crates/simulator/src/fuzz_setup.rs`, so the first cast moves **turn 143-154 → 3-29**
  (library-only band **5-29**) and CR 903.8 / 903.9a / 903.10a are fuzzed for the first time.
  **Every recorded fuzz seed before this merge is dead** (`OOS-DX22-7`); no play-server pin
  moved (78/0 — `session.rs` builds through the untouched `setup.rs`).
  Tests **4,358 / 0 / 5** (+13); 0 engine lines, 0 card-def lines, 0 `tools/` lines; PROTOCOL
  **35** / HASH **72** gate-executed and unmoved; coverage unmoved **1,133/1,803 = 62.8%**.
  Seeds: **OOS-UI2-1**, **OOS-SIM3-1**, **OOS-SIM1-4** CLOSED; filed **OOS-DX22-1..13** —
  of which **OOS-DX22-8** is the first real defect the repaired instrument found
  (pre-existing `attachment_validity`, seed 5 turn 88). Full handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-08-03 — **FEEDBACK-1 SHIPPED** (`scutemob-192`, merge `d55e74cc`;
  doc-only, the user-directed fresh-session planning task).
  `docs/mtg-engine-feedback-engineering.md`: 14-channel inventory, 8-row ranked proposal table
  (rows 1/3 are the queued PB-DX22/PB-DX32 — reconciled, not duplicated; PB-DX32 argued for
  promotion), alpha-loop ownership table, 18 from/to corrections. Headline: of 17 functional
  playtest findings only 2 are engine-layer; the crash→seed→replay pipeline does not exist
  (`fuzzer.rs:270`); the decision-point gate already exists (`decision_gate.rs`, ratchet 80) —
  extend, don't rebuild; the rejection channel is bot-path-only (`local_game.rs:564`).
  Tests unmoved (0 code lines, 1 doc file); PROTOCOL **35** / HASH **72** by construction.
  Seeds: **OOS-FB1-1..9** specified in doc §5; **FILED** by `scutemob-195` (2026-08-03) and
  again by `scutemob-199` (2026-08-04, this bullet's stale "NOT filed" the cause) —
  **deduplicated 2026-08-05**, the chain-verified set kept; see the registry banner.
  Full handoff: ESM task comments on `scutemob-192` (scutemob-183 pattern).
- **Prior**: 2026-08-02 — **UI-6 SHIPPED** (`scutemob-194`, merge `dd5cb47d`; **G9** of
  `memory/playtest-triage-2026-08-02b.md`, row 8 — the **last** row of its successor table, which
  is now fully dispatched).
  The browser shows a searcher their whole library, look-only: `AnswerShapeView::PickOne` gains
  `all_cards` (a play-server DTO, no wire change) while `candidates` stays exactly the engine's
  answer space (SR-38). Sorted by NAME so CR 701.23e's shuffle is not disclosed, and narrowed by
  CR 121.1 when a search-restriction replacement applies. `SearchPicker` is a scrollable checkable
  list; a look-only row is a plain `div` with a visible tag, not a disabled button.
  **The Invariant-7 raw-read gate is deliberately re-pinned 2 → 3** and is now a needle SET —
  measured: with the channel in the tree `.objects()` is still 2, so the old single-needle gate
  would have stayed green (MR-M11-01, a second time in the same file). Five zero-pins close the
  synonym bypasses, two of them added after a revert defeated the first draft with one.
  Tests **4,345 / 0 / 5** (+4); 0 engine lines, 0 card-def lines, coverage unmoved
  **1,133/1,803 = 62.8%**; PROTOCOL **35** / HASH **72** gate-executed and unmoved.
  Seeds: **OOS-UI6-1..6** filed. Full handoff: `memory/workstream-state.md`.
- **Prior**: 2026-08-02 — **ENG-2 SHIPPED** (`scutemob-193`, merge `4ab68fdc`; **G7** of
  `memory/playtest-triage-2026-08-02b.md`, row 7 of its successor table).
  Targets reach the event log: one additive `GameEvent::TargetsAnnounced` (discriminant 132) fires
  at announcement time from all 12 stack-push sites (CR 601.2c/602.2b/603.3d), player targets
  public and object targets through `event_view`'s existing `card_or` gate (Invariant 7).
  Tests **4,341 / 0 / 5** (+11); 0 card-def lines, 0 play-server source lines, 0 play-frontend
  lines, coverage unmoved **1,133/1,803 = 62.8%**; PROTOCOL **34 → 35** / HASH **71 → 72**.
  Seeds: **OOS-G7-1 CLOSED**; filed **OOS-ENG2-1, -2, -3, -6, -7, -8, -9**; **OOS-ENG2-4** and
  **OOS-ENG2-5** filed and CLOSED by their own riders. Successor candidate:
  **OOS-ENG2-1** + **OOS-ENG2-2** together (Ward never fires on a triggered ability).
  Full handoff: `memory/workstream-state.md`.
- **Prior**: 2026-08-02 — **ENG-1 SHIPPED** (`scutemob-191`, merge `a3b5e56b`; **G3** of
  `memory/playtest-triage-2026-08-02b.md`, row 6 of its successor table).
  Effect-driven discard is a real player choice: `Effect::DiscardCards` suspends into a new
  `EffectChoiceQuestion::Discard` (CR 701.9b) instead of auto-picking the lowest `ObjectId`.
  Tests **4,330 / 0 / 5** (+13); 0 card-def lines, coverage unmoved **1,133/1,803 = 62.8%**;
  PROTOCOL **34** / HASH **71**; `MAX_AUTO_CHOSEN_COMPLETE_UNION` **91 → 80**.
  Seeds: **OOS-G3-1 CLOSED**; filed **OOS-ENG1-1, -2, -3, -4, -6, -7, -8, -9, -10** (no `-5`,
  deliberately unused) + **OOS-G3-2**. Successor candidate: **OOS-ENG1-9**.
  Full handoff: `memory/workstream-state.md`.
- **Prior**: 2026-08-02 — **UI-5 SHIPPED** (`scutemob-190`, merge `08dc4e6a`; rows G8 + G10-G13 of
  `memory/playtest-triage-2026-08-02b.md`, the whole UX half of that triage). Frontend only:
  **0 engine lines** (`git diff main..HEAD -- crates/` empty), 0 wire change, PROTOCOL **33** /
  HASH **70** gate-executed. **Concede leaves the priority row** for the header beside "New
  game", behind a two-step confirm, filtered out of *both* action groups and disabled with a
  **visible** reason (a native `title` never opens on a disabled button); pickers say **Back**.
  **`TapForMana` is collapsed into `mana sources (N)`, one row per source NAME with a count —
  never hidden**, and the gate asserts both sides, because auto-tap covers casts alone
  (`auto_tap_commands_for`) so this is the only human channel for activation costs, echo,
  cumulative upkeep and CR 608.2g floating. **`cardTooltip` grows a caption** and every native
  `title` on a tooltip-anchored card element is gone — the nine the triage named **plus ~10 on
  the badges nested inside those anchors**, the identical collision over a smaller hit area.
  **Lands render below Artifacts/Enchantments** (Lands moved down, so no other pair reordered;
  artifact lands stay with lands, documented at the classifier) and **same-name lands stack**
  on `(name, tapped)` plus every other distinguishing field, with the click path decided
  (representative + caller-side fallback) rather than left undefined. **The shared-`$viewer`
  rule, stated once and applied three times**: edit in place, and where the two surfaces want
  opposite things express it as a PROP — `stackLands` defaults off, because the replay viewer
  is a step debugger and stacking deletes objects you are stepping to inspect. **4 gates, 9
  reverts executed red**; the G11 gate is per-ELEMENT (a tag walk over each `use:cardTooltip`
  anchor) so `title` stays legal on real controls, and **its own first run found a bug in
  itself** — prose in a module doc reported a tag that does not exist. Browser: **24/24** live
  (decline path AND a real concede; an open picker showing Back; a `Swamp×3` tapped beside a
  `Swamp×2` untapped; a stack click that acted, 819→820), plus **10/10** mounting the shared
  components against a fixture — a working proof-of-concept of the deferred R7 harness, recipe
  in the handoff. **The `/review` cycle found 8 and all 8 were taken, two of them real G8
  defects**: the armed confirmation **survived the decision it was armed against** (`Concede`
  is on every decision, so the disarm guard essentially never fired — the accidental-concede
  class G8 exists to close, reintroduced by the guard meant to prevent it), and the header
  button was a **silent dead control while a picker was open** (`beginChain` refuses on
  `chainOpen`, which `PlayApp` could not see) — the same shape UI-4 was dispatched to fix. Both
  proven real by revert. Tests **4,317 / 0 / 5** (+4). Seeds **OOS-UI5-1..4**. Full handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-08-02 — **SIM-6 SHIPPED** (`scutemob-189`, merge `ee99929d`; row 4 of the
  `memory/playtest-triage-2026-08-02b.md` successor table). **G4 CLOSED, both components.**
  `LegalAction::ActivateAbility` gains an `ActivationCostPlan`; the offer is suppressed when
  the eligible sacrifice/discard set is empty (SR-38, mirroring `offerable_cast_plan`); the
  choice is forwarded through `params.rs` — which hardcoded `sacrifice_target: None` /
  `discard_card: None`, the whole 422 — with the plan's own default as the bot fallback; the
  browser renders and validates it; the TUI's hand-built command routes through the one
  mapping table. **0 engine lines**, 0 wire changes, PROTOCOL **33** / HASH **70**
  gate-executed. **8 card defs** printing "Sacrifice ANOTHER …" carried `exclude_self: false`
  and would have started sacrificing themselves the moment the channel opened; coverage
  unmoved at **1,133/1,803 = 62.8%**. **Three findings the brief did not predict**: (1) the
  brief's "~135 of 166 refusals are your subject" is **refuted by measurement** — **zero** of
  the 166 is a sacrifice/discard refusal; they are 95 `InsufficientMana` + 40 unmet
  `activation_condition`; (2) with the channel open a heuristic bot ate two of its own
  creatures per turn (caught by the UI-3 seeded fixture going red), so bots now score an
  object-naming activation below `PassPriority`; (3) **the browser verification found a live
  422 of its own** — the offer loop mirrored none of CR 302.6 / CR 602.5b / CR 118.3, and
  `activated_ability_is_activatable` (the non-mana sibling of SIM-2's
  `tap_ability_is_activatable`) closes all 40 condition refusals. A/B **166 → 113**. Three
  browser flows verified live with a NON-DEFAULT answer each (Yahenni in response to a
  Dismember, with itself correctly absent from the picker; Altar of Dementia cost+target in
  one chain; Rummaging Goblin discard). Seeds **OOS-SIM6-1..5** — **`OOS-SIM6-3` is the
  successor**: auto-tap covers `CastSpell` alone, so 62 of the 113 remaining refusals, and a
  human's mana-cost activation in the browser, are still 422s. Tests **4,313 / 0 / 5**
  (+18). **The `/review` cycle found 5 LOW and all 5 were taken**, two of them real
  coverage holes: the discard channel had no HTTP probe (added, on a new mono-red
  fixture whose ability is `{T}`-only so it cannot fail for `OOS-SIM6-3`'s reason), and
  an `additional_costs` array sent on an `ActivateAbility` was dropped in silence — the
  mirror image of a guard this batch had just added in the same function.
  Full handoff: `memory/workstream-state.md`.
- **Prior**: 2026-08-02 — **SIM-5 SHIPPED** (`scutemob-188`, merge `e185a2ff`; row 3 of
  the `memory/playtest-triage-2026-08-02b.md` successor table). **G5 CLOSED in its (1)/(2)/(3)
  halves; (4) offer-suppression DEFERRED with measurements as `OOS-SIM5-4`** — it would have
  suppressed 1 of 166 refusals, does not cover the Aura family (needs an engine query;
  `get_enchant_target` is `pub(crate)`), and post-(1) an unsatisfiable offer costs nothing.
  What shipped: the bot path routes through the same atomic `apply_sequence` as the human path
  (a rejected cast commits zero taps); **bots announce targets for the first time ever** — new
  `crates/simulator/src/targeting.rs::plan_targets`, every legality decision delegated to
  `rules/queries.rs`, deterministic first-legal candidate by design (no RNG, so no recorded seed
  moves); and rejections are recorded (`RejectedCommand`, capped 256 retained /
  uncapped count, exported on `GET /api/game/report`) instead of discarded. **A/B on seeds
  0/7/42, 25 turns, 4 heuristic bots: wasted taps 20/15/10 → 0/0/0, `ManaPoolsEmptied`
  10/15/5 → 0/1/0** (the one residual traced in-journal to greedy-solver slack `OOS-SIM2-1`,
  a cast that SUCCEEDED — not a wasted plan); first journal-verified targeted bot casts
  (`Doom Blade` → creature, `Glacial Ray` → player, 7 total). **The new rejection channel
  immediately paid for itself**: 166 refusals classified, **~135 of them the activation-cost
  payment channel — SIM-6's exact subject**, measured before its dispatch. Seeds
  **OOS-SIM5-1..5** filed (1: lowest-ObjectId targeting often self-targets; 3: bot blocker
  refusals; 5: modal per-mode target slices unqueryable). 0 engine lines, PROTOCOL **33** /
  HASH **70** gate-executed. Tests **4,295 / 0 / 5** (+5), every new gate proven red by revert.
  Full handoff: `memory/workstream-state.md`.
- **Prior**: 2026-08-02 — **SIM-4 SHIPPED** (`scutemob-187`, merge `dcb1fe55`; row 2 of
  the `memory/playtest-triage-2026-08-02b.md` successor table, first of the seven remaining rows
  dispatched one at a time). **G2 CLOSED**: a browser mulligan re-rolled all four seats' decks
  AND commanders (CR 103.5/903.6) because the session kept `DeckSource::RandomPerSeat` and
  `redeal`'s perturbed seed re-ran the whole recipe. **The brief's fix was built, measured, and
  replaced**: two-pass `resolve_decks` moves the single-RNG stream (seat 2's decklist depends on
  seat 1's shuffle) and reddened 7 seeded tests, so the ship is `setup::dealt_decks(&state,&cfg)`
  (`setup.rs:238`) — read the *dealt* multiset back out of the built state and pin
  `cfg.decks = Fixed(dealt)` in `session::new_game` (`session.rs:240`); `redeal` needed zero
  changes and no seeded table moved. The mandatory gate exists
  (`test_redeal_preserves_every_seats_deck_and_commander`, plus two play-server probes proven
  red by executing the revert) — and the handoff notes the simulator gate ALONE could never
  catch this class: the defect lived in what the session *stored*, and a gate on the primitive
  does not gate the caller's argument. 0 engine lines (`git diff -- crates/engine/` empty),
  PROTOCOL **33** / HASH **70** by construction. Tests **4,290 / 0 / 5** (+7). Browser-verified
  headless (seed 187187, two mulligans, four command zones stable, hand redraws). Seed
  dispositions: **OOS-G2-1 CLOSED** (redeal no longer re-validates a recipe), **OOS-G2-2
  narrowed** (README repro procedure corrected), **OOS-G2-3 unchanged** (out of scope). New:
  **OOS-SIM4-1** (redeal still accepts a recipe config silently — TUI `app.rs:132` would
  reintroduce G2 verbatim), **OOS-SIM4-2** (pre-existing: engine offers an Aura `CastSpell` it
  then rejects, CR 303.4a), **OOS-SIM4-3** (`dealt_decks` can't represent partner commanders).
  Per-seat RNG streams deferred with reasons (needs per-seat mulligan counts; would move the
  `UI1_SEED`/`UI2_SEED`/`SIM1_SEED` fixtures five shipped flows rest on). Full handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-08-02 — **ADJUDICATION SHIPPED** (`scutemob-186`, merge `8b069ae2`,
  doc-only): the external recursion review (`docs/audits/mtg-characteristics-recursion-findings.md`)
  is adjudicated in `docs/audits/mtg-characteristics-recursion-adjudication.md`. **The task's framed
  conflict was stale on arrival** — PB-DX19's own review had already replaced the raw
  base-characteristics fix with `characteristics_for_condition` (thread-local depth guard), so the
  external doc's live objection is its thread-local rejection, not its printed-characteristics one.
  Measured from `all_cards()`: the layer-querying-condition population is **exactly 1 card**
  (`indomitable_archangel`); the shipped deviation is live-wrong on **7** deck-legal `Complete`
  pairs plus the CR 708.2a face-down class. CR verdict: the external doc's durable architecture
  (layer-bounded queries) is CR-correct; its immediate treat-as-inactive patch has **no CR
  warrant** (CR 613.8b prescribes timestamp order, never inactivity). **Disposition: accept in
  substance, reject the sequencing, split** — **PB-DX42b briefed at rank 13** of the v3 queue
  (authority: adjudication §5; the v3 memo's §4 table is NOT yet re-rowed), **PB-DX42a** (corpus
  roster gate) offered as a rider on PB-DX8 (rank 10). Seeds **OOS-ADJ-1..7** filed in adjudication
  §6 (registry-of-record for them until rowed into the audit registry); **OOS-ADJ-7** is an
  unrelated live-wrong find — `blood_moon`/`magus_of_the_moon` strip the Artifact card type from
  artifact lands, contradicting the printed cards (should ride PB-DX27, not DX42).
- **Prior**: 2026-08-02 — **UI-4 SHIPPED** (`scutemob-185`; G1 of
  `memory/playtest-triage-2026-08-02b.md`). **The browser's Confirm button was dead in all three
  template-copying pickers, and five CR flows the project believed shipped had never worked**:
  library search (701.23), scry (701.22a), surveil (701.25a), sacrifice additional costs (118.8),
  Squad (702.157a). Cause exactly as triaged — `structuredClone()` on a Svelte 5 `$state` proxy
  throws `DataCloneError` out of the click handler, leaving the DOM untouched. **Confirmed in
  headless Chromium against a live game before any edit** (picker open, 0 POSTs, 0 error strip,
  `command_count` unmoved), then fixed with a new `plainClone.svelte.js` (`$state.snapshot`) at all
  three sites. Picker failures now reach the error strip by two independent paths — a per-picker
  `try/catch` → `onError` → `stores.reportClientError`, and a `window` `error`/`unhandledrejection`
  net armed from `main.js` that covers the five pickers with no `try` (Svelte's `<svelte:boundary>`
  does **not** catch handler errors). Two source gates added in `tools/play-server/src/main.rs`,
  each proven red by executing a revert. **All five flows re-verified in the browser with a
  NON-DEFAULT answer each**, so game state distinguishes the human's choice from the engine's
  default. 9 source files + 4 doc files; **0 engine lines and 0 simulator lines**
  (`git diff main..HEAD --numstat -- crates/` is empty), **0 wire change** — PROTOCOL **33** /
  HASH **70** untouched by construction. Tests **4,265 / 0 / 5** on branch (+2 = the two gates).
  **The `/review` cycle found 5 LOW and all 5 were taken**, two of them real holes: the gate
  walked only `frontend/src/` and missed the `$viewer` shared library that `vite.config.js`
  compiles into the same bundle (now walked, proven red by planting a call in `cardTooltip.js`),
  and the pickers' malformed-template guards still bailed in silence — the same symptom from a
  second cause (now reported). All three picker types were re-verified in the browser *after* the
  fix cycle.
  **The R7 frontend harness is proposed, not built** — `memory/workstream-state.md` carries the
  two-tier design, the "fixtures must wrap the template in `$state()`" rule without which a harness
  would have passed green against this bug, the CI Node gap, and four known-good
  (seed → card → flow) tuples so nobody re-scans 2,400 seeds.
- **Prior**: 2026-08-02 — **PB-DX19 SHIPPED** (`scutemob-184`), first of the v3 queue:
  **OOS-SIM2-6 (the registry's only HIGH) and OOS-SIM2-5 both CLOSED**, and **OOS-DP3-9 /
  OOS-M11-3's stack-overflow half closes with them** on a 0/15 → 15/15 A/B (the pre-fix aborts
  were not individually backtrace-classified — strong evidence, not proof). The recursion
  (`calculate_characteristics` → `is_effect_active` → `check_static_condition` →
  `expect_characteristics` → back) is broken by the brief's pre-decided base-characteristics read.
  **Two premises of the seed were wrong and are corrected in its row**: the recursion is not a
  property of the object being calculated or of its zone — `calculate_characteristics` evaluates
  **every** conditional effect on **every** call — so a probe on the Archangel's own
  characteristics, and one with Metalcraft OFF, crashed identically. The in-source comment had
  argued termination from exactly that disproved invariant and demoted the fix to a *performance*
  note; **that comment, not the code, is why a HIGH survived 4.5 months**, and it now carries the
  mechanism. **The mandatory experiment is decisive**: `mtg-fuzzer --games 15 --seed 1` under
  `[profile.fuzz]` went from SIGABRT with **0 of 15** games completed to **15 completed** at avg
  **189** turns — and the abort was *immediate*, so OOS-DP3-9's "game-length-dependent" reading was
  a decks-drawn artefact. **OOS-SIM2-5 undercounted its own scope 4×**: sixteen edits, not four —
  ten `+=` sites (incl. the ±1/+1 counter path every game runs), six negations, and **two `as i32`
  counter widenings**, the last being the one that mattered, since **an `as` cast is not checked
  arithmetic even under `overflow-checks`** and wrapped the counter's SIGN in every profile. Its
  probe is the only one that fails by assertion, not panic. Scope, by a stated rule rather than a
  bare number (non-comment added lines carrying a checked-arithmetic construct, `layers.rs` +
  `effects/mod.rs`): **14 `saturating_add`, 6 `saturating_neg`, 4 `i32::try_from`, 3 `try_into`,
  2 `saturating_sub` = 29**. Three different counts were published before this one; the rule is
  given so the next reader can re-derive it instead of trusting it. **The fix's cost is real and is
  pinned, not remembered**: `blinkmoth_nexus`/`inkmoth_nexus` are `Complete`-by-derive colourless
  lands that animate into *artifacts*, so an animated Nexus no longer feeds Metalcraft though CR
  613.1d says it must — asserted wrong-way-round by
  `deviation_animated_nexus_does_not_count_toward_metalcraft`, which tells the successor batch to
  **invert** it. **The batch's first fix was itself a HIGH regression, caught by review and fixed
  here.** `check_static_condition` is a **shared** evaluator: five callers reach it, only
  `is_effect_active` closes a cycle, so reading base characteristics unconditionally broke the four
  safe callers to fix the one dangerous one — `garruks_uprising`'s `min_power` intervening-if
  silently stops firing on a counter-pumped creature (CR 613.4c), `bloodline_keeper` rejects a
  changeling (CR 702.73a), and `mox_opal` **over**-counts a face-down manifest (CR 708.2a, the
  false-positive direction nobody looked for). **None was visible to 4,274 passing tests** — no
  fixture put a counter-pumped or type-changed permanent through a condition filter. The repair is
  a re-entrancy guard, `rules::layers::characteristics_for_condition` behind an RAII
  `LayerWalkGuard`: base inside the walk, layer-resolved outside it. It decides by SHAPE, so it
  **also closes `OOS-DX19-1`** — the ten sibling sites — which the leaf-edit fix would have got
  wrong in the other direction, several being *correct* as layer-resolved. **That closure was
  claimed once before it was true**: the routing was done by pattern-replacement and missed three
  sites spelling the call `expect_characteristics(state, id)` rather than `(state, obj.id)`, and
  the re-review reproduced the original SIGABRT through one of them on a tree that already said
  CLOSED. All 14 sites now route through the helper, and a source gate
  (`no_condition_evaluator_resolves_characteristics_directly`) fails if any condition evaluator
  ever resolves characteristics directly again — the convention is now machine-checked, which is
  the only reason the closure is trustworthy. The deviation's scope is the layer walk
  alone. Seeds **OOS-DX19-1..4** filed. PROTOCOL **33** / HASH **70**
  gate-executed and unmoved. Tests **4,281 / 0 / 5** (+18, measured twice with a forced rebuild). Coverage **unmoved** — proven by
  regenerating `tools/authoring-report.py` to a byte-identical body, *not* by an empty card-defs
  diff, since the brief itself mandated the `greymond_avacyns_stalwart` note edit (that note had
  been instructing future authors to build a second instance of this exact HIGH). `cargo fmt`
  passed the greymond edit and **`tools/check-defs-fmt.sh` caught it** — SR-35, again. Full memo:
  `memory/primitives/pb-plan-DX19.md`; handoff in `memory/workstream-state.md`.
- **Prior**: 2026-08-02 — **SEED RE-RANK v3 SHIPPED** (`scutemob-182`, doc-only):
  `memory/primitives/seed-rerank-2026-08-02.md` is the authoritative queue; v2's §4 is banner'd
  SUPERSEDED (its §1-§3 stay canonical). **Census: 80 rows / 79 distinct IDs filed after
  2026-07-27 — twice the brief's ~40**, because v2's census closed 2026-07-31 and every PB-DX
  batch shipped 2026-08-01, so v2 never saw PB-DX1..DX5's 29 seeds or `OOS-M11-5..10`. Every row
  chain-verified against HEAD. **11 closures verified in code** (one, `OOS-UI2-3`, closed *further*
  than recorded — its third cause was `OOS-M11-2`'s `can_afford` half, so that seed's residue is
  now cost MODIFIERS + CR 106.12 only). **Next dispatch is PB-DX19, not PB-DX7**: `OOS-SIM2-6`,
  the registry's only HIGH, is an unbounded `calculate_characteristics` recursion
  (`layers.rs:46` → `:565` → `effects/mod.rs:10259` → `layers.rs:478`) that hard-aborts the process
  from **one** deck-legal `Complete` card (`indomitable_archangel`) and has been live 4.5 months
  behind a comment arguing termination from the wrong invariant and a test that names the card
  while hand-building `condition: None`. **One line fixes it**, and the correct precedent
  (`layers.rs:2304-2310`) was already in the tree. **Four seeds filed "latent" are live-wrong** on
  deck-legal `Complete` cards (`golgari_grave_troll`, `retreat_to_kazandu`, the ten Karoo bounce
  lands, `nether_traitor`). The `#[default] Completeness::Complete` derive explains five of the
  eight defs found this way — but `nether_traitor`, `qarsi_sadist` and `voldaren_epicure` declare
  `Complete` **explicitly**, so the shared mechanism is not the derive, it is that **nobody
  looked**. 965 of 1,803 defs never declare a marker and nothing reviews that
  population — filed as `OOS-RR3-1`. Also: `OOS-CARDS2-4` makes **13 `Complete` Auras** unplayable
  in the browser on first contact (the offer layer cannot see a `KeywordAbility::Enchant`-carried
  requirement); `OOS-M11-9` re-fires attack triggers and mutates attack targets when a human clicks
  attack twice; and `OOS-UI2-1` + `OOS-SIM3-1` reconcile — the fuzzer's first non-land is personal
  draw ~35-40, so "never casts" is `--max-turns 80` and "casts from turn 143" is the default cap.
  PB-DX7..DX18 keep their numbers and scopes at new ranks (PB-DX8 and PB-DX18 widened); PB-DX7
  drops to rank 9. **Full memo: `memory/primitives/seed-rerank-2026-08-02.md`.**
- **Prior**: 2026-08-02 — **THE PLAYTEST-SUCCESSOR RUN IS COMPLETE: scutemob-174..181 all
  SHIPPED in one coordinated session** (four waves of two workers; merges `f28df527` UI-1,
  `d04f42a1` CARDS-1, `83bfdba5` SIM-1, `8cad9c36` CARDS-2, `b30c99f4` SIM-2, `f40c9fb9` UI-2,
  `a23f0be0` SIM-3, `b76b1df4` UI-3). **The 2026-08-02 playtest triage is fully closed — F1–F10,
  OPEN = none.** Highlights: the browser answers blocking decisions (UI-1) and additional costs
  (UI-2, Sacrifice + Squad); SR-37 printed-field fidelity gate exists and repaired 45 wrong costs
  (CARDS-2); the mana solver counts MANA not sources and solves the residual against the pool
  (SIM-2); commanders are castable from the command zone with tax-aware auto-tap (SIM-1);
  `stack_consistency` no longer false-positives (SIM-3, 8,781→0 in fuzz A/B); 17 equip defs carry
  their CR 702.6a target (CARDS-1); 4-board layout fixed (UI-3). PROTOCOL **33** / HASH **70**
  unmoved by every batch, gate-executed each time. Coverage **62.8%** (1,133/1,803) after CARDS-2's
  honest demotions. Two cross-branch reconciliations happened at collect, not in any worker: the
  UI-2/SIM-2 bot-path conflict (one auto-tap path, both semantics) and UI-2's F4 pin flipped 0→1 by
  its own written instruction when SIM-2 closed F4 in parallel. **Standing findings worth reading
  before trusting old evidence**: `OOS-UI2-1` — the fuzzer has NEVER cast a spell (every historical
  "fuzz parity" claim is about a land-only game); `OOS-SIM2-6` (HIGH) — unbounded
  `calculate_characteristics` recursion, a hard crash reachable from a legal deck
  (`indomitable_archangel`); `OOS-SIM2-5` — silent i32 P/T wrap in release. Seeds filed:
  OOS-SIM1-1..4, OOS-SIM2-1..7, OOS-UI2-1..5, OOS-CARDS1-1..3, OOS-CARDS2-1..11, OOS-SIM3-*.
  **Full per-batch narratives: `memory/archive/claude-md-changelog-2026-08.md`** (rotated at this
  collect per the recurrence rule); handoffs in `memory/workstream-state.md`.
- **Prior**: 2026-08-02 — **PB-DX6 SHIPPED** (`scutemob-172`, merge `cb0755bf`; redone from
  scratch after the wave-7 crash, staged 0/A-F): **OOS-RS2-1 + OOS-DP4-1 CLOSED** —
  `handle_turn_face_up` paid a raw unflattened `def.mana_cost` in **all three** `TurnFaceUpMethod`
  arms (the brief named one), so every hybrid/Phyrexian pip in a face-up flip was free in release (a
  manifested `kitchen_finks` flipped for `{1}`); `Command::DeclareAttackers` gains the two PB-RS2
  payment fields so a hybrid/Phyrexian CR 508.1h attack tax is payable;
  `ManaPool::can_spend`/`spend` now fail **closed** on unflattened residue in release. PROTOCOL 32
  -> **33** (the predicted single bump, gate-computed with the full sentinel re-pin), HASH **70**
  unmoved; tests 4,099 on branch, **4,124 / 0** measured on main at the combined S8+DX6 collect.
  Seeds OOS-DX6-1..5 filed; next: **PB-DX7**. Same day: **M11-local closed** (2026-08-01 archive
  entry) and the first-human-playtest triage landed (`memory/playtest-triage-2026-08-02.md`;
  successor tasks `scutemob-174..181`). **Full narratives:
  `memory/archive/claude-md-changelog-2026-08.md`** — this bullet holds only the latest delta, per
  the recurrence rule.

### What Exists (M0-M9.5 + Engine Core Complete + all P3/P4 abilities)

- `cards/`: CardDefinition framework (30+ Effect primitives), ~1,798 card defs across hand-authored
  + templated waves; CardRegistry
- `effects/`: Full effect execution engine (DealDamage, GainLife, DrawCards, ExileObject,
  CreateToken, SearchLibrary, ForEach, Conditional, Scry, Surveil, DrainLife, Goad, Fight, etc.)
- `rules/`: Turn structure, priority, stack, SBAs, dependency-based layer system, combat, casting
  (Convoke/Improvise/Delve/Evoke/Kicker/Morph/Disturb alt costs), resolution, ETB trigger queueing
  (CR 603.3/603.4), ETB & global replacements, prevention, Commander (deck validation, command zone,
  tax, zone-return SBA, mulligan, companion, partner variants), protection (DEBT), copy (Layer 1 +
  storm + cascade), loop detection (CR 104.4b), Enchant, suspend, Mutate (CR 702.140), Transform/DFC
  (CR 701.28/712), Daybound/Nightbound, Craft, Morph/Megamorph/Disguise/Manifest/Cloak; Type
  Consolidation refactor complete (CastSpell 32→13, SOK ~20, AbilDef 55)
- `testing/`: Replay harness (`crates/engine/src/testing/replay_harness.rs` — public, shared with
  replay viewer), ~112 approved scripts, ~1934 harness tests, 6-player suite, 54 property invariants
- `benches/`: criterion (priority_cycle_4p 23µs, sba_check 14µs, full_turn_4p 205µs)
- `tools/replay-viewer/`: axum + Svelte 5, 5 API endpoints, 12 components, diff highlighting, keyboard nav
- 36 corner cases: 32 COVERED, 4 GAP, 0 DEFERRED

---

## Project Overview

We are building an MTG rules engine targeting **Commander format** (4-player multiplayer) with
**networked play**. The engine is written in **Rust**, the desktop app uses **Tauri v2** with a
**Svelte** frontend.

The engine is a standalone library crate with no UI or network dependencies. It can be tested
entirely in isolation. The network layer wraps the engine. The Tauri app wraps the network layer.

### Primary Documents

| Document | Location | Purpose |
|----------|----------|---------|
| Architecture & Testing Strategy | `docs/mtg-engine-architecture.md` | Why decisions were made; system design; testing approach |
| Engine Invariants & Gates | `docs/engine-invariants.md` | Full text of the machine-enforced SR gates (SR-2/3/4/5/6/7/8/9a/9b/9c/35/36); read the matching section before touching the subsystem it guards |
| Development Roadmap | `docs/mtg-engine-roadmap.md` | What to build and in what order; milestone definitions |
| Game Script Strategy | `docs/mtg-engine-game-scripts.md` | Engine-independent test script generation, JSON schema, replay harness design |
| Corner Case Reference | `docs/mtg-engine-corner-cases.md` | 36 known difficult interactions the engine must handle correctly |
| Corner Case Audit | `docs/mtg-engine-corner-case-audit.md` | Living correctness ledger: coverage status, card def gaps, deferred items |
| Network Security Strategy | `docs/mtg-engine-network-security.md` | **Deferred P2P upgrade path** — not the active M10 plan. M10 uses a centralized server. |
| Milestone Code Reviews | `docs/mtg-engine-milestone-reviews.md` | Per-milestone code review findings, file inventories, issue tracking |
| Replay Viewer Design | `docs/mtg-engine-replay-viewer.md` | M9.5 game state stepper: architecture, API, Svelte components, shared-component strategy |
| Ability Coverage Audit | `docs/mtg-engine-ability-coverage.md` | Keyword and pattern coverage tracking |
| LOW Issues Remediation | `docs/mtg-engine-low-issues-remediation.md` | **HISTORICAL (2026-02-28 snapshot; "~68 open LOW" is stale, ~6 remain).** Live LOW tally: "Current State → Known issues" above + `docs/mtg-engine-milestone-reviews.md` |
| Workstream Coordination | `docs/workstream-coordination.md` | **HISTORICAL — retired W1–W6 model (frozen 2026-03-08).** For what to work on: "Current State" above + `memory/primitives/oos-retriage-plan-2026-07-18.md` |
| Ability Batch Plan | `docs/ability-batch-plan.md` | **HISTORICAL — campaign COMPLETE.** Live tally: "Current State → Abilities" above; detail `docs/mtg-engine-ability-coverage.md` |
| Card Pipeline & Scaling | `docs/mtg-engine-card-pipeline.md` | Card definition organization, Rust DSL rationale, scaling strategy (112 → 27k), authoring pipeline |
| Strategic Review | `docs/mtg-engine-strategic-review.md` | 2026-03-07 project review: path-to-playable compression, M10/M11/M12 restructuring, action items. **All 9 resolved 2026-07-26** — historical record now; the structure it argued for lives in the roadmap |
| M11-local Session Plan | `memory/m11-session-plan.md` | The active first-playable plan: 8 sessions, crate-by-crate scope, the steppable-driver decision, hidden-info chokepoints, risks |
| Card Authoring Operations | `docs/card-authoring-operations.md` | **HISTORICAL — 2026-03-21 runbook, superseded.** Active campaign: `memory/card-authoring/campaign-plan-2026-05-16.md`; live coverage `docs/authoring-status.md`. (Its "Authoring Order" section is still cited by the Wave Process below.) |
| Runtime Integrity | `docs/mtg-engine-runtime-integrity.md` | Watchdog, recovery, bug reporting — pre-alpha requirement |
| Feedback Engineering | `docs/mtg-engine-feedback-engineering.md` | Alpha feedback-loop strategy: channel inventory, 8 ranked buildout proposals, alpha-pipeline ownership (2026-08-03, dispatch-ready) |
| Type Consolidation Plan | `docs/mtg-engine-type-consolidation.md` | Pre-M10 refactoring: CastSpell, SOK triggers, AbilityDef, Designations — 8 sessions |
| Cleanup Retention Policy | `docs/cleanup-retention-policy.md` | Two-tier ladder, year-month archive convention, /cleanup skill protocol |
| This file | `CLAUDE.md` | Current project state; session context |

**Read the architecture doc before implementing anything.**

### Secondary Documents & Task Records

Not primary context, but every one is reachable from here. Load on demand for the stated purpose.

| Document | Location | Purpose |
|----------|----------|---------|
| Authoring status (generated) | `docs/authoring-status.md` + `docs/authoring-status-guide.md` | **Canonical card-health source** — regenerated by `tools/authoring-report.py`, self-dating; prefer over any hand-maintained count |
| Engine explanation | `docs/engine_explanation.md` | Narrative walkthrough of the engine for a newcomer |
| Protocol versioning policy | `docs/mtg-engine-protocol-versioning.md` | Wire versioning policy behind SR-8 (also linked from `docs/engine-invariants.md`) |
| Simulator & bots | `docs/mtg-engine-simulator.md` | RandomBot / HeuristicBot / GameDriver / LegalActionProvider design |
| TUI plan | `docs/mtg-engine-tui-plan.md` | Terminal UI dashboard plan |
| Interaction gaps | `docs/mtg-engine-interaction-gaps.md` | Catalogue of known unresolved rules-interaction gaps |
| Project status (RETIRED) | `docs/project-status.md` | **🚫 RETIRED 2026-07-18, do not use or regenerate.** Successors: `docs/authoring-status.md` (card health) + "Current State" above (everything else) |
| Primitive/card plan (HISTORICAL) | `docs/primitive-card-plan.md` | March primitive/card plan — **banner'd historical**; active queue `memory/primitives/oos-retriage-plan-2026-07-18.md`, coverage `docs/authoring-status.md` |
| DSL gap closure (HISTORICAL) | `docs/dsl-gap-closure-plan.md` | March DRAFT — **banner'd superseded** by the EF/OS queues; audit `memory/card-authoring/dsl-gap-audit-2026-05-16.md` |
| SR remediation record | `docs/sr-remediation-plan.md` | Full SR-1..32 remediation log |
| SR task-record audits | `docs/sr-4-silent-failure-audit.md`, `docs/sr-5-keyword-catchall-audit.md`, `docs/sr-9a-test-consolidation.md`, `docs/sr-14-silent-failure-audit-rules.md`, `docs/sr-15-dispatch-enum-catchall-audit.md`, `docs/sr-24-lki-capture-cost.md` | Per-SR method/scope records referenced by the matching gate in `docs/engine-invariants.md` |
| Audit program | `docs/audits/README.md` + `docs/audits/methodology.md` | Index and method for the standing audit program |
| Standing audits | `docs/audits/layer-bypass-audit.md`, `docs/audits/event-log-diagnosability.md`, `docs/audits/stress-test-scenarios.md`, `docs/audits/decision-point-audit.md` | Specific audits (note: layer-bypass "9 HIGH" are its own M10-scheduled class, distinct from the 0-HIGH engine tally; **decision-point audit (2026-07-26, `scutemob-148`) found 5 Tier-0 correctness findings DP-1..DP-5 — incl. priority-after-cast CR 117.3c violation — and a ranked PB-DP1..DP10 insertion list, unranked vs the RS queue as of collection**) |
| Changelog archive | `memory/archive/claude-md-changelog-2026-07.md` | Verbatim PB/SR history moved out of this file's Current State (see "Changelog & history" above) |

### Additional Skills (beyond the ESM/session ones listed below)

- `/crew` — multi-agent orchestration helper.
- `/new-doc` — scaffold a new managed doc.
- `/next-ability` — pick and set up the next ability to implement.
- `/remedy` — SR remediation track driver (agent `sr-coordinator`; does not touch workstream-state).
- `/start-stepper` — launch the replay-viewer game-state stepper.

(Session/workflow skills — `/start`, `/dispatch`, `/collect`, `/eot`, `/task`, `/done`, `/spawn`,
`/status` — are in "Quick Start" below; per-task skills like `/implement-primitive`,
`/author-wave`, `/cleanup`, `/audit-cards` appear in the "When to Load What" table.)

---

## When to Load What

Before starting work, check which files apply to your task:

| Task | Load before starting |
|------|----------------------|
| Understanding / modifying a machine-enforced gate (any SR-N invariant) | `docs/engine-invariants.md` (the SR-2/3/4/5/6/7/8/9a/9b/9c/35/36 gate reference) |
| Touching any file in `rules/` | `memory/gotchas-rules.md` |
| Touching any file in `state/`, `cards/`, `effects/` | `memory/gotchas-infra.md` |
| Writing or modifying tests | `memory/gotchas-infra.md` (testing gotchas) |
| Writing new code or tests | `memory/conventions.md` |
| Questioning a design decision | `memory/decisions.md` |
| Implementing a new subsystem | `docs/mtg-engine-corner-cases.md` (full) |
| Checking correctness gaps | `docs/mtg-engine-corner-case-audit.md` |
| Starting a new milestone | Use `/start-milestone <N>` — reads only the relevant roadmap section via Grep+offset, never the full file. |
| Writing golden tests | `docs/mtg-engine-game-scripts.md` |
| Implementing network features (M10+) | `docs/mtg-engine-roadmap.md` M10 section (centralized server); `docs/mtg-engine-network-security.md` only for deferred P2P upgrade |
| Implementing replay viewer (M9.5) | `docs/mtg-engine-replay-viewer.md` |
| Implementing a keyword ability | `docs/mtg-engine-ability-coverage.md` |
| Checking ability gaps | Use `/audit-abilities` or `/ability-status` |
| Implementing a single ability end-to-end | Use `/implement-ability` — orchestrates plan → implement → review → fix → card → script → close |
| End-of-milestone cleanup pass | Use `/cleanup` — reads `docs/cleanup-retention-policy.md` and runs Gate A → B → dry-run → execute |
| Fixing LOW issues | `docs/mtg-engine-milestone-reviews.md` (live issue index; ~6 LOW remain). `docs/mtg-engine-low-issues-remediation.md` is a HISTORICAL 2026-02-28 snapshot — risk-tier framework still useful, counts stale |
| Authoring card definitions | `memory/card-authoring/campaign-plan-2026-05-16.md` (active campaign, §0 authoritative); `docs/mtg-engine-card-pipeline.md` (DSL reference). `docs/card-authoring-operations.md` is HISTORICAL — its "Authoring Order" section still valid, see Wave Process below |
| Triaging card defs for TODOs | Use `/triage-cards` — scans defs, reclassifies blocked sessions, consolidates review findings |
| Authoring a group of cards | Use `/author-wave <group>` — orchestrates author → review → fix → commit for one group |
| Auditing all card defs | Use `/audit-cards` — scans for TODOs, empty abilities, known-issue patterns, certifies completion |
| Type consolidation refactoring | `docs/mtg-engine-type-consolidation.md` (COMPLETE 2026-03-09 — historical record of the refactor, not an active plan) |
| Working on the play client / local play (M11-local is **COMPLETE** — this is maintenance, not milestone work) | `tools/play-server/README.md` (routes, limitations, hidden-info rules) + `docs/mtg-engine-simulator.md` §"Phase 3b" + `memory/workstream-state.md`'s S8 handoff. `memory/m11-session-plan.md` is now a historical record with its own COMPLETE banner |
| Planning M10a/M10b or the card-scaling track | `docs/mtg-engine-roadmap.md` (restructured 2026-07-26 — read the milestone section itself). `docs/mtg-engine-strategic-review.md` is now a historical record of *why* that structure exists, not a pending-changes list |
| Deciding what to work on / coordinating workstreams | "Current State" above (active milestone + queue) + `memory/primitives/oos-retriage-plan-2026-07-18.md` (ranked queue). `docs/workstream-coordination.md` is HISTORICAL (retired W1–W6 model) — do not use to pick work |

Use `/review-subsystem <name>` to load the right file and see open issues in one step.

---

## Card Authoring Wave Process

The remaining A-29+ groups are ordered into three waves by engine risk level.
**Follow this order** — see the "Authoring Order and Engine Risk Assessment" section of
`docs/card-authoring-operations.md` for the full breakdown. (That doc is banner'd HISTORICAL,
but this specific ordering section remains the valid reference for the wave sequence.)

1. **Wave A** (A-29, A-32, A-33, A-34, A-35, A-39): Safe to author now. Minor/no engine changes.
2. **Wave B** (A-38, A-42): Re-triage each group first — split authorable cards from blocked ones.
3. **Wave C** (A-30, A-36, A-40, A-41): Blocked on significant engine work. Treat as PB-style batch.

**Engine review checkpoints**: After each wave completes, batch-review all engine
changes before starting the next wave. Run `git diff <pre-wave-commit>..HEAD -- crates/engine/src/`
and review the accumulated engine additions. Fix any issues found. This is a single
review pass per wave, not per-session — but it is **mandatory** before advancing to
the next wave. The PB pipeline had plan → implement → review → fix; the authoring
pipeline adds engine code inline without review, so these checkpoints catch that gap.

---

## Architecture Invariants

These are non-negotiable. If a change would violate any of these, stop and reconsider.

1. **Engine is a pure library.** No IO, no network, no filesystem access, no async runtime
   in the engine crate. It takes commands in and emits state changes out. Everything else
   is the caller's responsibility.

2. **Game state is immutable.** Use `im-rs` persistent data structures. State transitions
   produce new states; old states are retained for undo/replay. Never mutate state in place.

3. **All player actions are Commands.** There is no way to change game state except through
   the Command enum. This enables networking, replay, and deterministic testing.

4. **All state changes are Events.** The engine emits Events describing what happened.
   The network layer broadcasts these. The UI consumes these. Events are the single
   source of truth for "what happened."

5. **Multiplayer-first.** Priority, triggers, combat — everything is designed for N players.
   1v1 is N=2, not a special case.

6. **Commander-first.** The command zone, commander tax, commander damage, color identity —
   these are core features, not bolted-on extensions.

7. **Hidden information is enforced.** The engine knows everything. The centralized server
   filters events before broadcasting — private events go only to the relevant player via
   `GameEvent::private_to() -> Option<PlayerId>`. Never expose another player's hand or
   library order to the wrong client. (P2P + Mental Poker is a deferred upgrade path —
   see `docs/mtg-engine-network-security.md`.)

8. **Tests cite their rules source.** Every test references the CR section or known
   interaction it validates. Untraceable tests are technical debt.

9. **Every card in a game must have a `CardDefinition` before the game starts.** The deck
   builder enforces this. No mid-game discovery, no graceful degradation during play. The
   rewind/replay/pause system depends on a complete and accurate state history from turn 1 —
   a card whose abilities silently never fired produces a corrupted history that cannot be
   rewound to correctly. Unimplemented cards are surfaced at deck-building time with clear
   messaging, not silently ignored at game time.

---

## MCP Resources
- **Rules search**: query by rule number ("613.8") or concept ("dependency continuous effects")
- **Card lookup**: query by exact card name for oracle text, types, rulings
- **Rulings search**: query by interaction concept ("copy effect on double-faced card")
- **rust-analyzer**: semantic code navigation — hover, definition, references, implementations,
  incoming/outgoing calls, workspace symbols. Call `rust_analyzer_stop` when done to free ~2.5GB
  RAM. First call triggers ~70s indexing warmup. Results default to 50 max; pass `limit` to
  override. See your auto-memory MEMORY.md index (rust-analyzer MCP Server section) for details.

---

## Critical Gotchas

These 3 apply to nearly every session. All other gotchas are in `memory/gotchas-rules.md` and `memory/gotchas-infra.md`.

- **Object identity (CR 400.7)**: When an object changes zones, it becomes a NEW object.
  The old ObjectId is dead. Auras fall off. "When this dies" triggers reference the old
  object. This is the #1 source of bugs in MTG engines.
- **Replacement effects are NOT triggers.** They modify events as they happen. They don't
  use the stack. Getting this wrong breaks the entire event system.
- **SBAs are checked as a batch, not individually.** All applicable SBAs happen simultaneously.
  Then triggers from all of them go on the stack together (in APNAP order).

---

## Agents

Seventeen project-scoped agents in `.claude/agents/` encode milestone, ability, primitive, and card authoring workflows:

| Agent | Model | RA | Trigger | Purpose |
|-------|-------|----|---------|---------|
| `rules-implementation-planner` | Opus | yes | "plan M9 implementation" | Session plan with architecture, CR refs, session breakdown |
| `session-runner` | Sonnet | — | "run session 1" / "next session" | Execute one implementation session from the plan |
| `milestone-reviewer` | Opus | yes | "review milestone M9" | Structured code review with HIGH/MEDIUM/LOW findings; creates fix-session-plan |
| `fix-session-runner` | Sonnet | — | "run fix session 3" | Execute 5-8 fixes, run tests, close issues |
| `card-definition-author` | Sonnet | — | "add card definition for X" | Translate oracle text to CardDefinition DSL |
| `bulk-card-author` | Sonnet | — | "author session 5" | Write batch of 8-20 card defs from authoring plan |
| `card-batch-reviewer` | Opus | — | "review cards batch 5" | Review 5 card defs against oracle text |
| `card-fix-applicator` | Sonnet | — | "apply fixes from review" | Apply review findings to card def files, verify build |
| `cr-coverage-auditor` | Sonnet | — | "check CR coverage for 614" | Audit test/script coverage for CR sections |
| `game-script-generator` | Sonnet | — | "generate script for X interaction" | JSON game scripts for replay harness |
| `ability-coverage-auditor` | Opus | — | `/audit-abilities` | Scan engine + card defs + scripts → refresh ability coverage doc |
| `ability-impl-planner` | Opus | yes | `/implement-ability` (plan phase) | CR research, study similar abilities, write implementation plan |
| `ability-impl-runner` | Sonnet | — | `/implement-ability` (implement/fix phase) | Execute steps 1-4 (enum, enforcement, triggers, tests), apply fixes |
| `ability-impl-reviewer` | Opus | yes | `/implement-ability` (review phase) | Verify implementation against CR, check edge cases, write findings |
| `primitive-impl-planner` | Opus | yes | `/implement-primitive` (plan phase) | CR research, study engine architecture, write PB plan |
| `primitive-impl-runner` | Sonnet | — | `/implement-primitive` (implement/fix phase) | Engine changes, card def fixes, tests, apply review fixes |
| `primitive-impl-reviewer` | Opus | yes | `/implement-primitive` (review phase) | Verify engine + card defs against CR/oracle text, write findings |

---

## Session & Workstream Protocol

- `/start` — bootstrap ESM, check local state, orient (also covers what `/start-session` used to do
  — workstream state is loaded via `esm project bootstrap` and the auto-memory MEMORY.md index)
- `/start-work W1-B3` — claim a workstream before coding (prevents parallel collisions)
- `/eot` — end-of-turn / end-of-session: ESM session close + workstream-state rotation + memory
  routing (replaces `/end` + `/end-session`)
- State file: `memory/workstream-state.md` (shared across sessions)
- Conventions: `memory/conventions.md` | Decisions: `memory/decisions.md`
- Dev environment: `.claude/CLAUDE.local.md`

### Commit Prefix Convention

| Workstream | Prefix | Example |
|------------|--------|---------|
| W1: Abilities | `W1-B<N>:` | `W1-B3: implement Ninjutsu` |
| W2: TUI & Simulator | `W2:` | `W2: fix blocker declaration` |
| W3: LOW Remediation | `W3:` | `W3: add debug_assert to sba.rs` |
| W4: M10 Networking | `W4:` | `W4: add GameServer skeleton` |
| W6: Card Authoring | `W6-cards:` | `W6-cards: author Skullclamp, Blood Artist` |
| W6: Primitives | `W6-prim:` | `W6-prim: add exclude_self enforcement` |
| SR remediation | `SR-<N>:` | `SR-9a: consolidate test binaries` |
| Cross-cutting | `chore:` | `chore: update workstream-state` |

---

## Milestone Completion Checklist

When completing a milestone:

- [ ] All deliverables checked off in the roadmap
- [ ] All acceptance criteria met
- [ ] All tests pass: `cargo test --all`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35 — `cargo fmt`
      checks none of the 1,798 card defs and still exits 0; the script is the only thing
      that checks them. `cargo test --all` runs it too, via `core card_defs_fmt`.)
- [ ] Performance benchmarks run (if applicable to this milestone)
- [ ] Update "Current State" section of this file
- [ ] Update "Active Milestone" to the next milestone
- [ ] Check off completed deliverables in `docs/mtg-engine-roadmap.md`
- [ ] Update relevant memory topic files (`memory/gotchas-rules.md`, `memory/gotchas-infra.md`,
  `memory/conventions.md`, `memory/decisions.md`) with new learnings
- [ ] Review all new/changed files and update `docs/mtg-engine-milestone-reviews.md`:
  - Add file inventory with line counts
  - List CR sections implemented
  - Record findings (bugs, enforcement gaps, test gaps) with severity and issue IDs
  - Place deferred issues in the correct future milestone stub
  - Update the cross-milestone issue index and statistics
- [ ] Commit: `M<N>: milestone complete — <summary>`
- [ ] **Code review → fix phase** (if any HIGH or MEDIUM findings):
  - Run the `milestone-reviewer` agent (Opus) — writes findings to `docs/mtg-engine-milestone-reviews.md`
    and creates `memory/m<N>-fix-session-plan.md` grouping issues into sessions of 5-8 fixes each
  - Work through fix sessions with the `fix-session-runner` agent (Sonnet):
    reads `memory/m<N>-fix-session-plan.md` → applies fixes → `cargo test --all` → `cargo clippy -- -D warnings` → closes issues in reviews doc → commit
  - When all sessions complete, update "Current State" and advance to the next milestone
  - LOW-only findings do not require a fix phase; collect them in the reviews doc and address
    opportunistically

---

# Scutemob MTG Engine — ESM-Managed Project

This project is managed by ESM (External State Machine). Use the `esm` CLI and slash commands to interact with it.

## Quick Start

Use these slash commands to manage your ESM session:

- **`/start`** — Begin a session. Bootstraps context from ESM, starts session tracking, orients you.
- **`/dispatch <title>`** — **Primary workflow.** Create a task, worktree, and auto-launch a worker
  in a kitty pane. Use this for all implementation work.
- **`/status`** — Quick snapshot of tasks, sessions, and fleet-wide context.
- **`/collect [task_id]`** — Collect a finished worker's work: merge worktree to main, clean up.
- **`/task <title>`** — Create a task and work on it yourself (for small, self-assigned work only).
- **`/done [task_id]`** — Complete a self-assigned task: transition to done, merge branch to main.
- **`/spawn <title>`** — Like /dispatch, but you launch the worker manually.
- **`/eot`** — End-of-turn / end-of-session: ESM close + workstream-state rotation + memory routing.
  **Use this instead of `/end`** for scutemob — `/end` still works but skips the project-specific
  bookkeeping.

**Every session must begin with `/start`** (or manually running `esm project bootstrap scutemob` + `esm session start`).

## Worker Detection

If `.esm/worker.md` exists in the working directory, **you are a worker agent**. Read it
immediately and follow its task/acceptance criteria. The rest of this CLAUDE.md still applies.

## Workflow Rules

1. **Bootstrap first**: `/start` (or `esm project bootstrap scutemob && esm session start --project
   scutemob --agent primary`).
2. **An `in_progress` task must exist before writing code.** Lifecycle: `backlog → in_progress →
   in_review → done` (or `blocked` from either active state).
3. **Branch protocol**: feature branch per task; attest `working_branch=<full-name>` on transition;
   `/done` (self-assigned) or `/collect` (dispatched) merges to main.
4. **Tests are mandatory.** Write alongside implementation. Must pass before `in_review`.
5. **Acceptance criteria**: `esm task satisfy <task_id> <criterion_id> --by <agent>` for each before
   signaling ready.
6. **Task comments are short status lines** — `Completed: X. Next: Y.` / `Blocked: X. Tried: Y.` /
   `Decision: X. Reason: Y.` Detailed design notes belong in `docs/` or `memory/`, not comments.
7. **Dispatch, don't implement.** Coordinator creates tasks and dispatches workers via `/dispatch`
   for PB / ability / card-authoring work. Only implement inline for trivial fixes (<10 lines) or
   when explicitly told.

ESM CLI reference: `esm --help` or `esm <command> --help`. Sessions without a heartbeat for 10 minutes are auto-ended.

## Required Attestations

When transitioning to `in_progress`:
- `branch_exists`: "true"
- `acceptance_criteria_defined`: "true"
- `working_branch`: "<branch-name>"

When transitioning to `in_review`:
- `tests_passing`: "true"
- `implementation_complete`: "true"

When transitioning to `done`:
- `review_complete`: "true"

When transitioning to `blocked`:
- `blocked_reason`: describe what you need before you can continue

Unblocking requires admin approval — you cannot unblock yourself.

## Advisory Mode

ESM runs in **advisory mode** by default. The hook will warn you about scope violations and missing tasks, but won't block your work. Warnings appear in stderr — pay attention to them.

If this project uses **blocking mode**, scope violations will be denied. Check the project's `enforcement_mode` setting.

## Documentation Management

If `.claude/docs.yaml` exists, this project uses ESM documentation management.
Managed docs have a `<!-- last_updated: YYYY-MM-DD -->` comment that tracks freshness.

- **`/docs status`** — Quick health overview of all managed docs
- **`/docs check`** — Audit docs for drift (checks triggers against git history)
- **`/docs init`** — Interactive setup: scan existing docs, detect features, scaffold new ones

When you update a managed doc, always update the `<!-- last_updated: YYYY-MM-DD -->`
comment to today's date. Only update it for substantive changes — not typo fixes.

The `/done` and `/eot` skills automatically check for stale docs based on which
files you changed. Follow their recommendations or dismiss with a reason.

## Project Info

- **ESM Project ID**: `scutemob`
- **Agent ID**: `primary`
- **ESM Server**: `http://tower:8765`

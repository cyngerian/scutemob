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
  RECORDED OOS-DP10-9, shipped the PB-DX42a rider);
  PB-DX27 shipped `scutemob-209` 2026-08-13 (rank 11 — closed OOS-CARDS2-8/-10/-11, OOS-RR3-2
  and rider OOS-ADJ-7, all five registry-rowed for the first time; PROTOCOL 36 / HASH 75);
  PB-DX28 shipped `scutemob-210` 2026-08-14 (rank 12 — closed OOS-DX4-6 and OOS-DX4-1);
  PB-DX29 shipped `scutemob-211` 2026-08-14 (rank 13 — closed OOS-M11-10(loyalty) and
  OOS-UI2-4; PB-DX42b was passed over, its rank premise recorded false by OOS-DX27-9);
  **SEED RE-RANK v4 SHIPPED** (`scutemob-212`, 2026-08-14, doc-only) — the authoritative queue
  is now `memory/primitives/seed-rerank-2026-08-14.md` §4; v3's §4 is banner'd SUPERSEDED
  (its §1-§3 remain canonical). **Next dispatch: PB-DX43** (CR 305.6/305.7 intrinsic mana
  abilities — OOS-DX27-1 + OOS-DX27-10, live-wrong on 3 deck-legal `Complete` format staples);
  **not** PB-DX18, which sits at v4 rank 10;
  **↻ PB-DX43 SHIPPED** (`scutemob-213`, 2026-08-14; v4 rank 1 — **OOS-DX27-1** and
  **OOS-DX27-10** both CLOSED);
  **↻ PB-DX44 SHIPPED** (`scutemob-215`, 2026-08-15; v4 rank 2 — **OOS-DX29-9**, **OOS-DX29-12**
  and **OOS-DX29-14** CLOSED, **OOS-DX29-3** NARROWED: its pitch half closed, its graveyard half
  deferred-and-measured);
  **↻ PB-DX15a SHIPPED** (`scutemob-216`, 2026-08-23; v4 rank 3 — **OOS-DP9-8** and
  **OOS-DP9-11** both CLOSED; rider **OOS-DX24-1** CLOSED and rider **OOS-DX24-7** RE-OPENED —
  the `/review` fix cycle inverted BOTH first-draft rider verdicts, see the narrative below;
  **OOS-DP9-16** parked as directed);
  **↻ PB-DX45 SHIPPED** (`scutemob-217`, 2026-09-02; v4 rank 4 — **OOS-DX24-9** ≡ **OOS-DX27-5**
  CLOSED as ONE defect, cross-cited);
  **↻ PB-DX47 SHIPPED** (`scutemob-218`, 2026-09-02; v4 rank 5 — **OOS-DX24-4** CLOSED, the
  probe-first outcome being the LARGE one: the double-push was REAL, not a dedup);
  **↻ PB-DX48 SHIPPED** (`scutemob-219`, 2026-09-02; v4 rank 6 — **OOS-ENG2-1** ≡ **OOS-ENG2-2**
  FILED *and* CLOSED and **OOS-ENG2-3** FILED and NARROWED, none of the three having had a
  registry row);
  **↻ PB-DX49 SHIPPED** (`scutemob-220`, 2026-09-03; v4 rank 7 — **OOS-RR4-1** and rider
  **OOS-RR4-3** both CLOSED; corner case **#36 GAP → PARTIAL**, the engine half of the
  corner-case audit's last open GAP).
  **↻ PB-DX50 SHIPPED** (`scutemob-221`, 2026-09-03; v4 rank 8 — **OOS-DX25-1** and
  **OOS-DX29-2** both CLOSED, each row corrected against three of its own claims).
  **↻ PB-DX20b SHIPPED** (`scutemob-222`, 2026-09-03; v4 rank 9 — **OOS-DX20-10** and
  **OOS-DX20-5** CLOSED as ONE defect, cross-cited; a third census member repaired that no
  document named).
  **↻ PB-DX18 SHIPPED** (`scutemob-225`, 2026-09-04; v4 rank 10 — **OOS-DP2-7**, **OOS-DP2-4**,
  **OOS-DP2-8**, **OOS-DX2-4**, **OOS-DX2-1** and **OOS-M11-5** ALL CLOSED, six seeds in one
  batch).
  **↻ PB-DX51 SHIPPED** (`scutemob-226`, 2026-09-04; v4 rank 11 — **OOS-DX21-4**, **OOS-DX21-2**
  and rider **OOS-DX21-5** ALL CLOSED; live on 2 deck-legal `Complete` defs).
  **↻ PB-DX35 SHIPPED** (`scutemob-227`, 2026-09-04; v4 rank 12 — **OOS-DX4-2** and
  **OOS-DX4-5** both CLOSED, plus **OOS-DP10-5** CLOSED and **OOS-DX8-3** updated).
  **↻ PB-DX36 SHIPPED** (`scutemob-228`, 2026-09-04; v4 rank 13 — **OOS-CARDS2-6** FILED
  (it had no registry row) and **CLOSED**, both halves).
  **↻ PB-DX52 SHIPPED** (`scutemob-229`, 2026-09-04; v4 rank 14 — **OOS-DX25b-1** and
  **OOS-DX25b-5** CLOSED, plus **OOS-DX25c-3** CLOSED as a third that this batch would
  otherwise have turned into a live CR 702.16b defect).
  **↻ PB-DX39 SHIPPED** (`scutemob-230`, 2026-09-05; v4 rank 15 — **OOS-DX5-3** and
  **OOS-DX5-7**'s residual CLOSED, each row corrected against three and four of its own
  claims; the class is 15× the two seeds and one deck-legal member is still broken
  one link upstream, stated rather than rounded up).
  **↻ PB-DX53 SHIPPED** (`scutemob-231`, 2026-09-05; v4 rank 16 — **OOS-DX21-1** CLOSED, its row
  corrected against three of its own claims; the class is **2** deck-legal members, not the row's
  1, and the second is a card whose defect was that its ability was MISSING).
  **↻ PB-DX54 SHIPPED** (`scutemob-232`, 2026-09-05; v4 rank 17 — **OOS-DX25c-6** CLOSED, plus
  rider **OOS-DX25-4** CLOSED and rider **OOS-DX25b-4** DECLINED with a measured wire cost; the
  row's "2 deck-legal `Complete`" cell REPRODUCES, the first yield cell in five batches that is
  not a floor, and the row's CR cite is wrong — CR 608.2n, not 608.2m).
  **↻ PB-DX42b SHIPPED** (`scutemob-233`, 2026-09-05; v4 rank 18 — **OOS-ADJ-1** ≡
  **OOS-DX19-2** FILED *and* CLOSED as ONE defect, plus **OOS-DX19-1**'s residue and
  **OOS-DX19-4** closed BY CONSTRUCTION, and the rank-21 rider **OOS-ADJ-2** taken in both
  halves; NEITHER of the two headline seeds had a registry row until this batch wrote one).
  **RANKS 1-18 ARE ALL SHIPPED AND NO FURTHER DISPATCH IS AUTHORISED** — rank 18 was the LAST
  **↻ 2026-09-05, later: SUPERSEDED on the no-further-dispatch point — the user approved a SECOND
  five-task chain, v4 ranks 19-23 (PB-DX55 → PB-DX56 → PB-DX57 → PB-DX9 → PB-DX38), sequential,
  collect-before-next, exactly five. Next dispatch: PB-DX55 (rank 19).**
  **↻ PB-DX55 SHIPPED** (`scutemob-234`, 2026-09-05; v4 rank 19, task 1 of 5 of the SECOND
  chain — **OOS-SIM6-3**, **OOS-SIM5-3** and **OOS-SIM5-5** ALL FILED (none had a registry row)
  *and* CLOSED, plus the rider **OOS-DX51-3** CLOSED; the bot refusal surface goes **70 → 9**
  and every survivor is one unrelated class. **Next dispatch: PB-DX56 (rank 20).**)
  **↻ PB-DX56 SHIPPED** (`scutemob-235`, 2026-09-05; v4 rank 20, task 2 of 5 of the SECOND
  chain — **OOS-FB1-1** *(the prerequisite)*, **OOS-DX32-1** and **OOS-DX22-8** ALL CLOSED,
  plus the rider **OOS-DP9-19(b)** CLOSED; the fuzzer's HARD bucket goes **291 → 0** on the
  standard invocation, so `--stop-on-error` no longer halts on an undiagnosed class.
  **Next dispatch: PB-DX57 (rank 21).**)
  of the user-approved five-task chain (ranks 14-18), and `feedback_queue_autonomous_chaining`
  was RETRACTED 2026-08-01, so rank 19 (`PB-DX55`) needs explicit user approval.
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
  `memory/card-authoring/campaign-plan-2026-05-16.md` §0. **Live coverage: 1,140/1,803 = 63.2%**
  (**UNMOVED by PB-DX55, 2026-09-05 — 0 flips and 0 card-def edits of ANY kind**: `git diff
  --numstat` over `crates/card-defs` and `crates/card-types/src/cards` is EMPTY, so the
  empty-diff shortcut was available and the regeneration was run anyway, confirming every bucket
  identical (clean 1,140 / todo 516 / empty 147). The reason 0 is the right number rather than a
  lucky one: this batch authors no card text — it repairs the offer, query and funding layers
  that sit BETWEEN a `Complete` def and a client, which is why it closes 61 bot refusals while
  moving no marker.*)*
  *(historical: **1,140/1,803 = 63.2%**
  (**UNMOVED by PB-DX42b, 2026-09-05 — 0 flips, predicted with the reason PER DEF before any
  code changed and confirmed in every bucket (clean 1,140 / todo 516 / empty 147 identical).
  **3 card-def edits, all comment-only** — `indomitable_archangel`'s wrong CR cite, the
  `greymond_avacyns_stalwart` blocker note this batch FALSIFIES, and the SR-35 reflow of the
  latter — and `git diff` over the `Completeness::` marker lines in `crates/card-defs` is
  **EMPTY**, so the `CORPUS_COMPLETE` SET is unmoved as well as its count and
  `OOS-CARDS2-3`'s re-deal budget was checked and found not owed. The marker was checked by
  `git diff` over the marker rather than inferred from the unchanged total — PB-DX26's lesson
  that a stable COUNT is not a stable SET**)
  *(historical: **1,140/1,803 = 63.2%**
  (**UNMOVED by PB-DX54, 2026-09-05 — 0 flips, 0 card-def edits of any kind**)*
  *(historical: **1,140/1,803 = 63.2%**
  (**PB-DX53, 2026-09-05 — ONE flip, `minas_tirith` `partial` → `Complete`, NAMED before any code;
  it is a THIRD member of the turn-scoped class that no document in the chain names, found by the
  INVERSE ORACLE axis because its ability was unauthored and a declared-axis census structurally
  cannot see a card whose defect is that it is missing. Its `ENGINE-BLOCKED` note demanded
  `Condition::AttackedWithNCreatures(2)` — an identifier that had existed since PB-OS6, so the note
  was FALSE at HEAD**)
  *(historical: **1,139/1,803 = 63.2%**
  (**UNMOVED by PB-DX52, 2026-09-04 — 0 flips, predicted and reasoned per def before
  regeneration; no `Completeness` marker moved anywhere, so the `CORPUS_COMPLETE` SET is
  unmoved too and no seeded fixture was re-dealt**)
  *(historical: **1,139/1,803 = 63.2%**
  (PB-DX36, 2026-09-04 — one flip, `exalted_angel`, from the new `WhenDealsDamage` trigger +
  `EffectAmount::DamageDealt`)*
  *(historical: **1,138/1,803 = 63.1%**
  (PB-DX35, 2026-09-04 — one flip, `shambling_ghast`, from the CR 700.2b per-mode target scoping)
  *(historical: **1,137/1,803 = 63.1%**
  (PB-DX45, 2026-09-02 — one flip, `vampire_gourmand`, from the CR 118.12 policy re-adjudication)*
  *(historical: **1,133/1,803 = 62.8%**
  (unmoved by PB-DX26 — one flip up and one honest flip down cancelled, 2026-08-11)*
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
  **PB-DX27 SHIPPED** (`scutemob-209`; v3 queue rank 11 — **OOS-CARDS2-8**, **OOS-CARDS2-10**,
  **OOS-CARDS2-11**, **OOS-RR3-2** and the rider **OOS-ADJ-7** all FILED *and* CLOSED; none of
  the five had a registry row before this batch wrote one) — ranks **1-11 are all shipped**,
  so **next dispatch: PB-DX28** (rank 12). Coverage **62.8% → 63.0%** (1,133 → **1,136**);
  PROTOCOL **35 → 36** / HASH **74 → 75**, both gate-computed. Filed **OOS-DX27-1..10**.
  **↻ 2026-09-02 — PB-DX45 SHIPPED** (`scutemob-217`; v4 rank 4 — **OOS-DX24-9** ≡ **OOS-DX27-5**
  CLOSED as ONE defect). Coverage **62.8% → 63.0% → 63.1%** (1,136 → **1,137**) on the single
  predicted flip `vampire_gourmand`; PROTOCOL **38 → 39** / HASH **77 → 78**, both gate-computed.
  **↻ 2026-09-02 — PB-DX47 SHIPPED** (`scutemob-218`; v4 rank 5 — **OOS-DX24-4** CLOSED).
  Coverage unmoved at **1,137/1,803 = 63.1%**, **0 flips**, **0 card-def edits of any kind**;
  PROTOCOL **39** / HASH **78** both gate-executed and UNMOVED. Filed **OOS-DX47-1..7**.
  **↻ 2026-09-02 — PB-DX48 SHIPPED** (`scutemob-219`; v4 rank 6 — **OOS-ENG2-1** ≡ **OOS-ENG2-2**
  CLOSED, **OOS-ENG2-3** NARROWED). Coverage unmoved at **1,137/1,803 = 63.1%**, **0 flips**,
  **0 card-def edits of any kind**; PROTOCOL **39** / HASH **78** both gate-executed and UNMOVED.
  Filed **OOS-DX48-1..7**.
  **↻ 2026-09-03 — PB-DX49 SHIPPED** (`scutemob-220`; v4 rank 7 — **OOS-RR4-1** CLOSED and rider
  **OOS-RR4-3** CLOSED). Coverage unmoved at **1,137/1,803 = 63.1%**, **0 flips**, **0 card-def
  edits of any kind**; PROTOCOL **39** / HASH **78** both gate-executed and UNMOVED, predicted in
  writing before any code. Filed **OOS-DX49-1..9**.
  **↻ 2026-09-03 — PB-DX50 SHIPPED** (`scutemob-221`; v4 rank 8 — **OOS-DX25-1** and
  **OOS-DX29-2** both CLOSED). Coverage unmoved at **1,137/1,803 = 63.1%**, **0 flips**,
  **0 card-def edits of any kind**; **PROTOCOL 39 → 40 / HASH 78 → 79**, ONE bump each, both
  gate-computed and both predicted PER HALF in writing before any code — the legality half moved
  neither fingerprint and the timing half moved each once. Filed **OOS-DX50-1..11**.
  **↻ 2026-09-03 — PB-DX20b SHIPPED** (`scutemob-222`; v4 rank 9 — **OOS-DX20-10** ≡
  **OOS-DX20-5** CLOSED as ONE defect). Coverage unmoved at **1,137/1,803 = 63.1%**, **0 flips**
  as predicted, **3 card-def edits** (`imprisoned_in_the_moon`, `kayas_ghostform`,
  `breath_of_fury` — all keyword-declaration changes, **no `Completeness` marker moved**, so the
  `CORPUS_COMPLETE` SET is unmoved and no seeded fixture was re-dealt); **PROTOCOL 40 → 41 /
  HASH 79 → 80**, ONE bump each, both gate-computed and both predicted in writing before any
  code, type counts predicted unchanged and confirmed at 98 / 131. Filed **OOS-DX20b-1..7** (`-6` and `-7` by the `/review` fix cycle, after the first draft of these lines said `-1..5` — dispatch hygiene 8's exact case, caught by re-checking this cell against the registry AFTER the fix cycle rather than before it).
  **↻ PB-DX18 SHIPPED** (`scutemob-225`, 2026-09-04; v4 rank 10 — **OOS-DP2-7**, **OOS-DP2-4**,
  **OOS-DP2-8**, **OOS-DX2-4**, **OOS-DX2-1** and **OOS-M11-5** ALL CLOSED, six seeds in one
  batch).
  **↻ PB-DX51 SHIPPED** (`scutemob-226`, 2026-09-04; v4 rank 11 — **OOS-DX21-4**, **OOS-DX21-2**
  and rider **OOS-DX21-5** ALL CLOSED; live on 2 deck-legal `Complete` defs).
  **↻ 2026-09-04 — PB-DX35 SHIPPED** (`scutemob-227`; v4 rank 12 — **OOS-DX4-2** and
  **OOS-DX4-5** both CLOSED, plus **OOS-DP10-5** CLOSED and **OOS-DX8-3** updated). Coverage
  **63.1% → 63.1%** (1,137 → **1,138**) on the single predicted flip `shambling_ghast`, NAMED
  before any code; **12 card-def edits of which 9 are comment-only**, and the ONE marker move
  took `CORPUS_COMPLETE` 1137 → 1138 with `COMMANDER_POOL` re-measured UNCHANGED at 90.
  **PROTOCOL 41 / HASH 82 both UNMOVED — zero bumps for the whole PB**, both gate-executed and
  both predicted per half in writing before any production line. Filed **OOS-DX35-1..10**.
  **↻ PB-DX36 SHIPPED** (`scutemob-228`, 2026-09-04; v4 rank 13 — **OOS-CARDS2-6** FILED
  (it had no registry row) and **CLOSED**, both halves).
  **↻ PB-DX52 SHIPPED** (`scutemob-229`, 2026-09-04; v4 rank 14 — **OOS-DX25b-1** and
  **OOS-DX25b-5** CLOSED, plus **OOS-DX25c-3** CLOSED as a third that this batch would
  otherwise have turned into a live CR 702.16b defect).
  **↻ PB-DX39 SHIPPED** (`scutemob-230`, 2026-09-05; v4 rank 15 — **OOS-DX5-3** and
  **OOS-DX5-7**'s residual CLOSED, each row corrected against three and four of its own
  claims; the class is 15× the two seeds and one deck-legal member is still broken
  one link upstream, stated rather than rounded up).
  **↻ PB-DX53 SHIPPED** (`scutemob-231`, 2026-09-05; v4 rank 16 — **OOS-DX21-1** CLOSED, its row
  corrected against three of its own claims; the class is **2** deck-legal members, not the row's
  1, and the second is a card whose defect was that its ability was MISSING).
  **↻ PB-DX54 SHIPPED** (`scutemob-232`, 2026-09-05; v4 rank 17 — **OOS-DX25c-6** CLOSED, plus
  rider **OOS-DX25-4** CLOSED and rider **OOS-DX25b-4** DECLINED with a measured wire cost; the
  row's "2 deck-legal `Complete`" cell REPRODUCES, the first yield cell in five batches that is
  not a floor, and the row's CR cite is wrong — CR 608.2n, not 608.2m).
  **↻ PB-DX42b SHIPPED** (`scutemob-233`, 2026-09-05; v4 rank 18 — **OOS-ADJ-1** ≡
  **OOS-DX19-2** FILED *and* CLOSED as ONE defect, plus **OOS-DX19-1**'s residue and
  **OOS-DX19-4** closed BY CONSTRUCTION, and the rank-21 rider **OOS-ADJ-2** taken in both
  halves; NEITHER of the two headline seeds had a registry row until this batch wrote one).
  **RANKS 1-18 ARE ALL SHIPPED AND NO FURTHER DISPATCH IS AUTHORISED** — rank 18 was the LAST
  **↻ 2026-09-05, later: SUPERSEDED on the no-further-dispatch point — the user approved a SECOND
  five-task chain, v4 ranks 19-23 (PB-DX55 → PB-DX56 → PB-DX57 → PB-DX9 → PB-DX38), sequential,
  collect-before-next, exactly five. Next dispatch: PB-DX55 (rank 19).**
  **↻ PB-DX55 SHIPPED** (`scutemob-234`, 2026-09-05; v4 rank 19, task 1 of 5 of the SECOND
  chain — **OOS-SIM6-3**, **OOS-SIM5-3** and **OOS-SIM5-5** ALL FILED (none had a registry row)
  *and* CLOSED, plus the rider **OOS-DX51-3** CLOSED; the bot refusal surface goes **70 → 9**
  and every survivor is one unrelated class. **Next dispatch: PB-DX56 (rank 20).**)
  **↻ PB-DX56 SHIPPED** (`scutemob-235`, 2026-09-05; v4 rank 20, task 2 of 5 of the SECOND
  chain — **OOS-FB1-1** *(the prerequisite)*, **OOS-DX32-1** and **OOS-DX22-8** ALL CLOSED,
  plus the rider **OOS-DP9-19(b)** CLOSED; the fuzzer's HARD bucket goes **291 → 0** on the
  standard invocation, so `--stop-on-error` no longer halts on an undiagnosed class.
  **Next dispatch: PB-DX57 (rank 21).**)
  of the user-approved five-task chain (ranks 14-18), and `feedback_queue_autonomous_chaining`
  was RETRACTED 2026-08-01, so rank 19 (`PB-DX55`) needs explicit user approval.
  **↻ 2026-08-14 — QUEUE RE-RANKED (v4, `scutemob-212`)**: every "next dispatch" line above this
  one is historical. The authoritative queue is `memory/primitives/seed-rerank-2026-08-14.md`
  **§4**; `seed-rerank-2026-08-02.md` §4 is banner'd SUPERSEDED (its §1-§3 stay canonical).
  **Next dispatch: PB-DX43.** (**↻ SHIPPED `scutemob-213` 2026-08-14; next is PB-DX44.**)
  Census: **208** post-v3 seed IDs by a published derivation rule
  (2.6× v3's 80), of which **61 have no registry row** — plus 7 more behind standing rows, so the
  registry's blind spot is **68**. Verdicts: 25 CLOSED / 45 QUEUE / 32 RIDER / 63 PARKED / 43
  DESIGN-RECORD. **PB-DX42b re-decided, not carried** — `OOS-DX27-9`'s "rank premise falsified"
  does not hold on the deck-legal axis the rank used, so it keeps its scope at rank 18.
  Filed **OOS-RR4-1..3** for the user-directed Blood Moon / Urza's Saga flag, now discharged.
- **Tests (delta 2026-09-05, PB-DX56 + `/review` fix cycle)**: **5,316 / 0 / 5** full-workspace on
  branch `scutemob-235` (+29 over the **5,287** baseline, measured on this branch BEFORE any edit
  and **reproducing PB-DX55's close pin exactly** — the **eighth** consecutive batch in which an
  inherited pin reproduces with no correction owed), `--workspace --no-fail-fast` to a file,
  **72** result-producing targets (unmoved), residual list empty.
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs** —
  never `sort` + `comm` (`OOS-DX20b-5`), with the extraction regex deliberately NOT end-anchored
  (`OOS-DX42b-6`), and **RE-TAKEN AFTER the `/review` fix cycle** (dispatch hygiene 8, the ninth
  batch running): **29 additions, 0 leavers, 0 removals, 0 renames.** Count delta 29 == name-set
  delta 29, duplicate-name scan **EMPTY on both runs** (5,292 / 5,292 distinct; 5,321 / 5,321).
  **"0 leavers" must NOT be read as "nothing was touched"** — two tests were edited IN PLACE with
  their names unchanged, so the name-set delta is structurally blind to both:
  `mechanics_e_l::extra_turns::test_extra_turn_eliminated_player_skipped`, whose docstring
  **DOCUMENTED the CR 800.4k violation F2 closes as expected behaviour** (corrected, and the
  correction is strictly stronger — `assert_ne!` becomes `assert_eq!`), and
  `t_check_all_prepends_state_context_before_the_checks_own_evidence`, repointed when the class
  split.
  **HASH 85 / PROTOCOL 44 BOTH UNMOVED — ZERO bumps for the whole PB**, gate-executed
  (`hash_schema` 36/36, `protocol_schema` 17/17) and **predicted PER HALF in writing before any
  production line** (`1f16e2c9`), with a stated stop-condition had the engine fix needed a field.
  `git diff` over `state/hash.rs` and `rules/protocol.rs` is **EMPTY**, so no sentinel re-pin, no
  survivor scan, no history row and no frozen-prefix re-pin were owed.
  **The counterfactual is VERIFIED BY EXECUTION**: planting `GameObject` and `TurnState` — the two
  types whose already-existing fields F1/F2/F3 write — in each gate's `CLOSURE_MUST_NOT_CONTAIN`
  **FAILS HASH** and leaves **PROTOCOL green** (both are reachable only through `GameState`, which
  that list excludes), reproducing PB-DX51's `CombatState` asymmetry. *(**And §0.4a's stronger
  claim about the simulator half is REFUTED and corrected**: `CLOSURE_MUST_NOT_CONTAIN` is a list
  of type-NAME STRINGS, so planting `"GameResult"` compiles and the gate passes 36/36 — the
  counterfactual is **expressible and vacuous**, not "unexpressible". The conclusion survives on
  the dependency graph; the claimed epistemic strength did not. `OOS-DX56-15`.)*
  Coverage **UNMOVED at 1,140/1,803 = 63.2%** by regeneration, **0 flips** predicted with the
  reason before any code, self-dating churn reverted; **0 card-def edits of any kind**, so the
  empty-diff shortcut was available and the regeneration was run anyway.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `cargo build --workspace` clean (the SR-3 seal
  gate) — all against the FINAL tree, and `cargo fmt` FIRED there once and was answered.
  **`npm run build` NOT run — N/A with the reason**: `git diff main..HEAD --numstat --
  tools/play-server/frontend` is **EMPTY** (the only `tools/` change is one
  `..Default::default()` inside a `#[cfg(test)]` `GameResult` literal) and `node_modules` is
  absent from this worktree.
  **Benches NOT measured, and the reason is a mechanism bound checked by execution**:
  `crates/engine/benches/engine_perf.rs` contains **zero** occurrences of any symbol the engine
  half touches (`detach_from_host_on_departure`, `advance_turn`,
  `grant_priority_to_active_player`, `extra_turns`, `attachments`); F1 adds one `Option` test and,
  only when `Some`, one `retain` over a `Vector` that is empty for every unattached permanent, on
  the zone-move path, which none of the six benches drives.
  **Engine lines**: `crates/engine/src` **+82 / −20** across four files — `state/mod.rs` +39/−0,
  `rules/turn_structure.rs` +21/−1, `rules/engine.rs` +16/−11, `rules/priority.rs` +6/−8 (two doc
  inventories that became false the moment F3 landed). **`crates/card-types`, `crates/card-defs`
  and `crates/view-model` are all EXACTLY 0.**
  **FUZZ: HARD 291 → 0 on the standard invocation**, so `--stop-on-error` no longer halts. A/B'd
  against merge base `e0da3cc9` in an isolated worktree with its own `CARGO_TARGET_DIR` (deleted
  after — dispatch hygiene 11): the merge-base run and this batch's PRE-EDIT run **differ in
  EXACTLY ONE LINE, the wall clock**. Every per-class raw count and per-class game list is
  IDENTICAL across the boundary (189 / 102 / 553), TRANSIENT 553 → **844 = 553 + 189 + 102
  exactly**, wins / draws / errors / avg turns unchanged at 20 / 0 / 0 / **122.0** — so the whole
  movement is the RECLASSIFICATION plus three new hard checks measuring zero, and **the three
  engine fixes are trajectory-neutral by measurement**, which is why no seeded pin moved. PB-DX32
  gate config re-observed **13/13 green** with **no ratchet constant touched** — answered, not
  loosened.
  **Revert / bypass matrix: 5 revert rows + 8 executed bypass plants + 3 more the `/review` found
  — ELEVEN gates on this batch defeated by execution, all eleven now RED**, every file restored
  byte-exactly. **R-E is the row worth reading**: with F1 reverted the new `attachment_symmetry`
  check measures **10,290 raw / 7 distinct across 5 of 20 games** — ~1,470 checkpoints per
  condition — against `attachment_validity`'s 102 / ~13, ~8. **Two orders of magnitude on the same
  run from the same stateless per-command checker is what settles "at rest" versus "transient",
  and the census wrote that prediction down first.** **THREE plants produced NON-VERDICTS that
  looked like passes** — two build failures (`-D warnings` dead code; a `break` outside a loop)
  and two that silently failed to APPLY after `cargo fmt` rewrapped their target line —
  `OOS-DX39-8`'s shape three times in one batch, reported rather than counted.
  Filed **OOS-DX56-1..15** (`-6` through `-15` by the `/review` fix cycle — dispatch hygiene 8's
  exact case for the eighth batch running).
- **Tests (delta 2026-09-05, PB-DX55 + `/review` fix cycle)**: **5,287 / 0 / 5** full-workspace on branch
  `scutemob-234` (+44 over the **5,243** baseline, measured on the MERGE BASE in its own worktree
  and **reproducing PB-DX42b's close pin exactly** — the sixth consecutive batch in which an
  inherited pin reproduces with no correction owed), `--workspace --no-fail-fast` to a file,
  **72** result-producing targets (69 → 72: three new test binaries), residual list empty.
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs —
  never `sort` + `comm` (`OOS-DX20b-5`), and with the extraction regex deliberately NOT
  end-anchored (`OOS-DX42b-6`, so an `#[ignore = "reason"]` test whose line reads
  `... ignored, <reason>` is still extracted), and **RE-TAKEN AFTER the `/review` fix cycle rather
  than before it** (dispatch hygiene 8 — the cycle added `r6`, so the pre-cycle figure of 43 is
  superseded by this line rather than left standing beside it): **44 additions, 0 leavers, 0
  removals, 0 renames.** Count delta 44 == name-set delta 44, and the duplicate-name scan the byte-exact
  method is structurally blind to (`OOS-DX35-8`) is **EMPTY on both runs** (5,248 lines / 5,248
  distinct; 5,292 / 5,292).
  **HASH 85 / PROTOCOL 44 BOTH UNMOVED — ZERO bumps for the whole PB**, gate-executed
  (`hash_schema` 36/36, `protocol_schema` 17/17) and **predicted in writing PER HALF before any
  production line** (`6a03181a`). Closure type counts **MEASURED** at **132 / 98** by raising
  each gate's `MIN_CLOSURE_TYPES` to 9999 and reading its own panic text. `git diff` over
  `state/hash.rs` and `rules/protocol.rs` is **EMPTY**, so no sentinel re-pin, no survivor scan,
  no history row and no frozen-prefix re-pin were owed.
  **The counterfactual is VERIFIED BY EXECUTION, because "unmoved" only means something beside
  what would have moved it**: planting `TargetRequirement`, `ModeSelection` and `AttackTarget` in
  each gate's `CLOSURE_MUST_NOT_CONTAIN` **FAILS BOTH GATES** every time — i.e. every type this
  batch's new query surfaces traffic in was **already on both wires**, which is exactly why
  returning them adds nothing. `CombatState` fails HASH and **passes** PROTOCOL (reachable only
  through `GameState`, which that list excludes), reproducing PB-DX51's finding. The load-bearing
  fact for the modal half is that **`Command::ActivateAbility` already carried `modes_chosen`**
  (`command.rs:124`), so no command field was added; `rules/queries.rs` is a read-only query
  module and is off-wire.
  Coverage **UNMOVED at 1,140/1,803 = 63.2%** by regeneration, **0 flips**, self-dating churn
  reverted; **0 card-def edits of any kind**, so the shortcut was available and the regeneration
  was run anyway.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `cargo build --workspace` clean (the SR-3 seal
  gate) — all against the FINAL tree, **and `clippy` FIRED there**, on `doc_lazy_continuation` in
  a doc line opening `1.` — an ordered-list item that makes the next line its lazy continuation,
  PB-DX39's own case one punctuation mark over; reworded with the reason recorded at the line.
  **TWO standing gates also FIRED and both were answered rather than weakened**: SR-5's
  `keyword_registry` site roster TWICE (`legal_actions.rs` reads Vigilance for the funding
  exclusion, `random_bot.rs` reads Menace for the prune), and SR-25's `bare_lookup_ratchet`,
  whose `combat.rs` ceiling genuinely FELL 16 → 14 from the extraction and was **lowered** rather
  than left stale-high (PB-DX49's rule that a stale-high ceiling is slack a regression hides in).
  **`npm run build` was NOT run, and unlike the last several batches that is a GAP rather than an
  N/A**: the frontend DOES move here — `BlockerPicker.svelte` **+28 / −11** (each row's `<select>`
  is now driven by that blocker's own `legalBlocks` slice instead of the flat cross product) and
  `ActionBar.svelte` +1 — so the criterion is live, and `node_modules` is absent from this
  worktree (`test -d` executed), so the build cannot run at all. The Svelte changes ship
  **unbuilt**, covered only by the Rust-side gates and `api.rs`'s pair validation. Filed as
  `OOS-DX55-7` rather than folded into the usual N/A sentence.
  **Engine lines**: `crates/engine/src` **+548 / −514**, of which `rules/combat.rs` is
  **+343 / −474** — a **NET REDUCTION of 131 lines**, which is the second hand-rolled copy
  collapsing into the first; `crates/simulator/src` **+900 / −119**; `tools/` **+517 / −43**; and
  **`crates/card-types`, `crates/card-defs` and `crates/view-model` are all EXACTLY 0**.
  **Benches: NOT measured, and the reason is a mechanism bound checked by execution rather than
  an estimate.** `crates/engine/benches/engine_perf.rs` contains **zero** occurrences of any
  symbol this batch touched (`check_block_pair`, `legal_blocks`, `ability_target_requirements`,
  `per_mode_target_requirements`, `command_mana_cost`, `auto_tap_commands_for`,
  `handle_declare_blockers`), and its only mention of `DeclareBlockers` is a doc line whose very
  next sentence is *"No attackers are declared."* — so the extracted predicate is never called on
  any benched path. Everything else changed is in `crates/simulator` and `tools/`, which the
  engine benches do not link.
  **Fuzz: A/B'd against the merge base on the PB-DX32 gate config, and the headline is a ZERO.**
  Merge base `70cd2487` built in its own worktree with its own `CARGO_TARGET_DIR`: T2.2's
  rejection rate goes **5 / 2,713 = 1.843‰ → 0 / 2,717 = 0.000‰**, and T6.3's REACHED decision
  rows go **4 of 7 → 6 of 7** (`look_at_top_then_place_optional` and `surveil` JOIN — before this
  batch bots could not pay activation costs, so the resolution paths behind those rows were
  starved — and Half 2's trajectory shift takes `may_pay_then_effect` back out; both attributed
  by executed ablation, and `decision_site_walk`'s partition is untouched). **The zero forced
  T2.2 to move its seeds** to `[6, 7, 10]`, because its own `total_rejections > 0` non-vacuity
  floor cannot coexist with a measured zero and a gate whose floor is unsatisfiable has stopped
  discriminating — **and moving the seeds is the right repair for T2.2 and the wrong place to
  leave the result**, so the zero is pinned where it happened by a new
  `test_dx55_the_historical_gate_seeds_now_produce_zero_bot_rejections` asserting `== 0` on
  `[1, 2, 3]` under a `total_commands >= 2,150` floor (a bot that stopped acting also reports
  zero). A ceiling of zero is the strongest ratchet that file can hold. Five further seeded pins
  were re-observed and each attributed by an executed ablation rather than re-tuned
  (`UI3_SPLIT_COMBAT_SEED` 26 → 47, `pb_dx22_fuzz_instrument`'s `SEED` 1 → 6, T2.1's seed 1 → 16,
  T2.2's triple, T6.3's set) — `OOS-DX21-6`'s blast radius, which `OOS-DX51-3`'s row predicted
  and which came due here.
  **Revert matrix: 13 rows, EXECUTED BY THE COORDINATOR rather than accepted from the delegated
  reports, every file restored byte-exactly (`cmp`), with a CONTROL row (R0, no patch, green).**
  R1 and R2 are **precise complements** (the funding widening and the self-tap exclusion each
  load-bearing). **R4's zero is the row worth reading and it is NOT a coverage gap**: removing
  `legal_blocks`' attacking-player fast path reddens nothing, and that is structural — an
  attacker always attacks somebody else, so `check_block_pair`'s `CrossPlayerBlock` arm already
  refuses every pair the attacking player could name. Settled by a complementary pair rather than
  argued: fast path AND that arm removed together (R4b) reddens 3, the arm alone (R5) reddens
  exactly the cross-player probe. The row that actually carries `OOS-DX51-3` is **R6**, the offer
  consuming the query instead of a raw battlefield scan.
  **R9 IS A GATE DEFEAT THAT SUCCEEDED, and it is this batch's durable half.** The
  block-legality gate decides *"exactly one per-pair predicate exists"* by a threshold of 5 of 9
  markers, and **all nine are EXOTIC** — horsemanship, skulk, shadow, intimidate, fear, the
  `CantBeBlockedExceptBy` internals, landwalk, protection. Planting a five-guard hand-rolled
  predicate in `combat.rs` itself — controller, tapped, `CantBlock`, flying/reach, protection,
  i.e. someone answering *"can this block that?"* for one local purpose — scored **1 of 9** and
  left the gate **GREEN**. Its own `r2`/`r3` self-defeats could not see it because both plant a
  WHOLESALE renamed copy of the real body, which carries all nine by construction: `OOS-DX54-6`
  verbatim, *a self-test written by the same author from the same mental model exercises the
  inputs that author already thought of*. **Nobody hand-rolls their way to horsemanship.** Closed
  by a second axis on the COMMON guards whose threshold was MEASURED before the code was written
  (real predicate 8 of 8, nearest other 2, nothing else above one), with the plant kept as a test
  that ALSO asserts the exotic axis still misses it. `OOS-DX55-3`.
  **AND THE REVERT HARNESS ITSELF WAS WRONG BEFORE ANY OF THAT.** It restored patched files with
  `shutil.copy2`, **which preserves the source mtime**, so a restored file looked OLDER than the
  artefact compiled from the patched version, `cargo` did not rebuild, and **every row after the
  first measured the PREVIOUS row's binary**. Nothing said so: `git status` reported the tree
  clean and `cmp` reported every file restored byte-exactly, and both were true. Found only
  because a probe failed in isolation on a tree that had just passed the full suite and no source
  difference could explain it. Measured cost: the first matrix reported R6 at **3** red and R7 at
  **2**; after the fix they are **2** and **1**. The whole matrix was re-executed from the
  control row. **`OOS-DX39-8`'s over-wide build detector turned a verdict into a NON-verdict,
  which is loud; this turns a verdict into a DIFFERENT verdict, which is not** (`OOS-DX55-4`).
  Filed **OOS-DX55-1..10** (`-9` and `-10` by the `/review` fix cycle — dispatch hygiene 8's
  exact case for the seventh batch running, caught by re-checking this cell against the registry
  AFTER the cycle rather than before it).
- **Tests (delta 2026-09-05, PB-DX42b + `/review` fix cycle)**: **5,243 / 0 / 5** full-workspace
  on branch `scutemob-233` (+12 over the **5,231** baseline, measured on this branch BEFORE any edit and
  **reproducing PB-DX54's close pin exactly** — the fifth consecutive batch in which an inherited
  pin reproduces with no correction owed), `--workspace --no-fail-fast` to a file, **69**
  result-producing targets (68 → 69: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs — never
  `sort` + `comm` (`OOS-DX20b-5`), and **RE-TAKEN AFTER the `/review` fix cycle rather than before
  it** (dispatch hygiene 8 — the cycle added two probes, so the pre-cycle figure of 13 is
  superseded by this line rather than left standing beside it): **15 additions, 3 leavers,
  0 removals**, and **all three leavers
  are disclosed and none is a removal** — every one is a rename this batch was instructed to make:
  `deviation_animated_nexus_does_not_count_toward_metalcraft` →
  `nexus_animated_by_a_continuous_effect_now_counts_toward_metalcraft` (the INVERSION its own
  message demanded); `the_deviation_is_scoped_to_the_layer_walk_only` →
  `characteristics_for_condition_gives_full_resolution_outside_any_walk` (the ambient flag it read
  no longer exists); and `t7_control_land_with_subtypes_absent_from_population` →
  `t7_non_target_filter_layer_querying_variants_absent_from_population` (the `OOS-ADJ-2` rider's
  1 → 8 widening). Honest reading: **12 genuine additions and 3 mandated renames.**
  **The first draft of this cell said 12 and 2 and was missing the third** — an enumeration error
  inside the one cell whose whole purpose is enumeration, caught by RUNNING the set difference
  rather than by transcribing what the batch remembered doing. Count delta 12 == name-set delta 12,
  and the duplicate-name scan the byte-exact method is structurally blind to (`OOS-DX35-8`) is
  **EMPTY on both runs** (5,236 / 5,236 distinct; 5,248 / 5,248) — **and the first draft of those
  four numbers was 5,235 / 5,245, off by one each, because the extraction regex was ANCHORED at
  the end of the line (`\.\.\. (ok|FAILED|ignored)$`) and the corpus contains one
  `#[ignore = "reason"]` test whose line reads `... ignored, <reason>`. The additions and leavers
  were unaffected (none of the 16 is an ignored test) but the DUPLICATE SCAN was blind to that one
  name, which is the only thing that scan exists to see. `OOS-DX42b-6`: an end-anchored test-line
  regex silently drops every `#[ignore = "..."]` test, and it is a duplicate-name scan's blind
  spot that it cannot report a name it never extracted.**
  **A THIRTEENTH TEST EXISTS AND DOES NOT APPEAR IN ANY OF THESE FIGURES, which is stated rather
  than left for a later batch to trip over**: `same_layer_self_reference_is_suppressed_not_resolved`
  is `#[cfg(not(debug_assertions))]` — the labelled deviation's wrong-way-round pin cannot be a
  debug test, because the CR 613.1d `debug_assert!` fires first on any same-layer self-reference —
  so it never compiles into the debug binary, never runs in CI, and no count delta will ever
  include it. Verified by executing `cargo test --release`. Filed as **`OOS-DX42b-4`**.
  **HASH 85 / PROTOCOL 44 BOTH UNMOVED — ZERO bumps for the whole PB**, gate-executed
  (`hash_schema::` **34** + `protocol_schema::` **17** = **51/51**, including
  `history_is_append_only` and `frozen_prefix_is_pinned` on both sides; the `/review` reported 53
  from a BARE-SUBSTRING filter, which also matches two tests OUTSIDE those modules that merely
  mention the name — SR-36's shape applied to a test filter, so the module-scoped 51 is the figure
  and the looser one is recorded here rather than silently discarded) and **predicted
  in writing before any production line** (`d90b7994`). Closure type counts **MEASURED** at
  **98 / 132** by raising each gate's `MIN_CLOSURE_TYPES` to 9999 and reading its own panic text,
  never transcribed. `git diff` over `state/hash.rs` and `rules/protocol.rs` is **EMPTY**, so no
  sentinel re-pin, no survivor scan, no `OOS-DX18-3` over-replacement read, no history row and no
  frozen-prefix re-pin were owed; the two append-only gates were executed anyway, green, as the
  evidence that none was owed rather than as a claim.
  **The counterfactual is VERIFIED BY EXECUTION at stage 0**: planting `TargetFilter` — and
  separately `EffectLayer` — in each gate's `CLOSURE_MUST_NOT_CONTAIN` fails **BOTH** gates each
  time, so the rejected design (STORE the required layer on `TargetFilter` instead of computing it)
  costs **+1 HASH and +1 PROTOCOL** plus a ~49-file sentinel re-pin, where computing it per
  instance costs nothing. That measurement is the reason for the design, not a preference.
  Coverage **UNMOVED at 1,140/1,803 = 63.2%** by regeneration, **0 flips** predicted with the
  reason per def before any code changed and confirmed in every bucket (clean 1,140 / todo 516 /
  empty 147 identical), self-dating churn reverted; **3 card-def edits, all comment-only**, with
  `git diff` over the `Completeness::` markers **EMPTY**.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `cargo build --workspace` clean (the SR-3 seal
  gate) — all against the FINAL tree, **and TWO standing card-def gates FIRED there**, both on this
  batch's own comment edits and both answered rather than dodged: PB-DX8's
  `completeness_deviation_scan` on `indomitable_archangel` (answered by ALLOWLIST with the contract
  widening STATED, on the `bolt_bend` precedent — rewording to dodge the needle was rejected
  because *a gate you edit prose to satisfy has stopped measuring*), and SR-35's
  `tools/check-defs-fmt.sh` on the `greymond` rewrite, which `cargo fmt --check` passes and always
  will.
  **`npm run build` was NOT run and that is stated rather than omitted**: N/A, because
  `git diff --numstat <merge-base>..HEAD -- tools/` is **EMPTY** and `node_modules` is absent.
  **Engine lines** (re-taken against the FINAL tree rather than transcribed from a mid-batch
  figure — PB-DX28's re-take MEDIUM, and the first draft of this cell said `+400 / −144` from an
  estimate rather than a measurement): `crates/engine/src` **+558 / −187** across `rules/layers.rs`
  (+436/−142), `effects/mod.rs` (+112/−31), `rules/engine.rs` (+9/−13) and `lib.rs` (+1/−1);
  `crates/card-types/src` **+209 / −0** (the two `required_characteristic_layer` impls, one of them
  an exhaustive 33-field destructure and the other an exhaustive 53-variant match);
  `crates/card-defs` **+26 / −8**, all comment-only across two files; and
  **`crates/view-model`, `crates/simulator/src` and `tools/` are all EXACTLY 0** — every consumer
  of characteristics already called `calculate_characteristics`, whose signature does not change,
  and `check_condition`'s public signature was deliberately preserved across its 63 call sites.
  **Benches: a REAL regression, FOUND AND REMOVED rather than published.** The first A/B measured
  `sba_check` **+6.76% with NON-OVERLAPPING criterion intervals** against a same-code band of
  **0.32%** on that bench. Cause 1: the bounded layer list was `.filter(..).collect::<Vec<_>>()` —
  a **heap allocation on every `calculate_characteristics` call**, i.e. one per battlefield
  permanent per SBA check, where the pre-batch code used a stack array (+6.76% → +2.98%). Cause 2:
  `abilities_are_blanked` constructed a fresh eval context per effect inside an
  O(permanents × effects) sweep; hoisting it is observationally identical because `InFlightGuard`
  removes on drop (+2.98% → +1.29%). **Final: 3 merge-base runs and 5 HEAD runs on a quiet machine,
  same-code band measured FIRST, EVERY criterion interval OVERLAPS**, deltas +0.05 / +1.20 / +1.29 /
  −0.31 / +0.48 / +1.51%, each smaller than HEAD's own same-code spread on that bench — **no
  regression demonstrated, and nothing claimed in either direction.** **No struct grew**, so no
  `size_of` is owed, which is the same fact the wire prediction rests on.
  **Fuzz: NEUTRAL BY MEASUREMENT and the output is byte-identical.** The PB-DX32 gate config's
  per-seed rows are byte-identical before and after; the wider matched A/B
  (`--games 20 --seed 1 --max-turns 200`, merge base in its own worktree with its own
  `CARGO_TARGET_DIR`) differs in **EXACTLY ONE LINE — the wall clock** (1.9s vs 2.1s). Every
  violation count, per-seed band and histogram row is identical: HARD **88 / distinct 4** across
  5 of 20 games, TRANSIENT **210 / distinct 44**, rejections **2,189 / 94,770 = 23.098‰**. No
  movement, so **no ablation was owed**.
  **Revert matrix: 9 rows, EXECUTED BY THE COORDINATOR rather than accepted from the delegated
  reports, all three source files restored byte-exactly (`cmp`).** **R1 and R2 are precise
  complements** — R1 (restore the deviation) reddens 6 including both channel directions, R2 (the
  DEPTH-COUNTER revert) reddens exactly the nesting probe and leaves every channel probe green,
  which is the only way to show the bounded query and the `EffectId` KEYING are each load-bearing.
  **R3 is the row worth reading, and it came back GREEN AT THE TIME IT WAS RUN.** *(It no longer
  does: the `/review` defeated the source gate that R3 was reddening, the gate was re-keyed on the
  mechanism, and at HEAD deleting the conjunct reddens it. The sentence below describes the state
  in which the finding was made.)* Deleting the activity sweep's layer
  bound — the adjudication's OWN §3.2(iii) load-bearing precondition, the one it says is *"stated
  here because it is stated nowhere else"* — reddens **nothing**, and that is structural rather
  than a missing test: a later-layer effect cannot change an earlier layer's output, which is the
  very fact that makes bounding semantically free, and the `in_flight` backstop absorbs the rest.
  **Settled by a complementary pair, the first execution of §3.2(iii)'s claim**: sweep bound
  PRESENT + backstop REMOVED runs **23/23 green** (termination IS by construction), and sweep bound
  REMOVED + backstop REMOVED **aborts with `fatal runtime error: stack overflow` (SIGABRT)** —
  `OOS-SIM2-6`'s original crash. R4/R4b are the same shape one axis over. **R7 was the second
  coverage measurement** — restoring `OOS-DX42b-1` reddened only a VOCABULARY gate, so the
  behaviour had no probe; both gaps are closed by probes that are RED under their own rows.
  Filed **OOS-DX42b-1..7** (`-6` and `-7` by the `/review` fix cycle — dispatch hygiene 8's exact
  case for the sixth batch running, caught by re-checking this cell against the registry AFTER the
  cycle rather than before it), plus **`OOS-ADJ-1`** and **`OOS-ADJ-2`** FILED for the first time —
  six of the adjudication's seven `OOS-ADJ-*` seeds had never been registered.
- **Tests (delta 2026-09-05, PB-DX54 + `/review` fix cycle)**: **5,231 / 0 / 5**
  full-workspace on branch `scutemob-232` (+21 over the **5,210** baseline, measured on this
  branch BEFORE any edit and **reproducing PB-DX53's close pin exactly** — the fourth
  consecutive batch in which an inherited pin reproduces with no correction owed),
  `--workspace --no-fail-fast` to a file, **68**
  result-producing targets (67 → 68: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs — never
  `sort` + `comm` (`OOS-DX20b-5`): 21 additions, 0 leavers, 0 removals, 0 renames.** Count delta
  21 == name-set delta 21, and the duplicate-name scan the byte-exact method is structurally blind
  to (`OOS-DX35-8`) is **EMPTY on both runs** (5,210 / 5,210 distinct; 5,231 / 5,231).
  **HASH 85 / PROTOCOL 44 BOTH UNMOVED — ZERO bumps for the whole PB**, gate-executed (53/53
  including `declaration_fingerprint_is_pinned`, `stream_fingerprint_is_pinned`,
  `history_is_append_only` and `frozen_prefix_is_pinned` on both sides) and **predicted PER OPTION
  in writing before any production line** (`54415c25`). Closure type counts **MEASURED** at
  **98 / 132** by raising each gate's `MIN_CLOSURE_TYPES` to 9999 and reading its own panic text,
  never transcribed from PB-DX53. `git diff` over `state/hash.rs` and `rules/protocol.rs` is
  **EMPTY**, so no sentinel re-pin, no survivor scan on either axis, no `OOS-DX18-3`
  over-replacement read, no history row and no frozen-prefix re-pin were owed; the two append-only
  gates were executed anyway, green, as the evidence that none was owed rather than as a claim.
  **The counterfactual is stated because "unmoved" only means something beside what would have
  moved it, and all three legs were VERIFIED BY EXECUTION at stage 0**: planting `StackObject` in
  `hash_schema.rs`'s `CLOSURE_MUST_NOT_CONTAIN` fails that gate while `protocol_schema.rs` already
  lists both `StackObject` and `GameState` and stays green (so the rejected shadow-entry design is
  HASH-only), and planting `EffectChoiceQuestion` fails BOTH (so the declined rider `OOS-DX25b-4`
  is +1 on each).
  Coverage **UNMOVED at 1,140/1,803 = 63.2%** by regeneration, **0 flips** predicted with the
  reason before any regeneration and confirmed in every bucket (clean 1,140 / todo 516 / empty 147
  identical), self-dating churn reverted; **0 card-def edits of any kind** — `git diff --numstat`
  over `crates/card-defs` and `crates/card-types/src/cards` is empty, so the shortcut was available
  and the regeneration was run anyway.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `cargo build --workspace` clean (the SR-3 seal
  gate) — all against the FINAL tree, **and `clippy` FIRED there**, on `useless_conversion` in the
  `/review` fix cycle's own `r5` widening; fixed and recorded rather than swept, because "clean
  against the FINAL tree" is only worth something if the final tree is the one checked. The suite
  and its delta were **RE-TAKEN AFTER the fix cycle** (dispatch hygiene 8) and are unchanged, the
  cycle having edited three test files and added no `#[test]`.
  **`npm run build` was NOT run and that is stated rather than
  omitted**: N/A, because `git diff --numstat <merge-base>..HEAD -- tools/` is **EMPTY** and
  `node_modules` is absent; unlike PB-DX52 no acceptance criterion predicted otherwise.
  **Engine lines** (re-taken against the FINAL tree rather than transcribed from a mid-batch
  figure — PB-DX28's re-take MEDIUM): `crates/engine/src` **+209 / −81**, and it is exactly two
  files — `rules/resolution.rs` **+192 / −59** and `effects/mod.rs` **+17 / −22**;
  **`crates/card-types`, `crates/card-defs`, `crates/view-model`,
  `crates/simulator/src` and `tools/` are all EXACTLY 0** — every consumer of the CR 608.2n
  departure point lives in those two engine files, measured before the design was chosen rather
  than asserted after.
  **Revert matrix: 7 rows, EXECUTED BY THE COORDINATOR rather than accepted from the delegated
  reports, all three source files restored byte-exactly (`cmp`)** — R1 (the whole fix) reddens 6;
  **R4 and R5 are precise complements** (each reddens exactly one of the two rider probes, which
  is the only way to show two byte-identical copies of a defect both needed fixing); **R6 is the
  row worth reading** — respelling the departure as `retain(..)` satisfies PB-DX52's inherited
  `r1a` and is caught ONLY by this batch's `r3`, which is *"a gate you edit prose to satisfy has
  stopped measuring"* demonstrated by execution; R7 catches a third `sba.rs` stack reader in the
  `for so in &state.stack_objects` form the gate's own first draft could not have seen.
  **TWO ROWS ARE COVERAGE MEASUREMENTS, NOT PASSES, AND SAY SO IN THE TEST ITSELF**: R2 (the
  function-boundary design) and R3 (the backstop) each redden ONE source gate and **no
  behavioural probe anywhere** — `OOS-DX52-2`'s shape said out loud. R2's probe is currently
  **UNBUILDABLE** rather than merely unwritten, blocked behind `OOS-DX54-4`; R3's needs three
  fixtures nothing in the tree builds (`OOS-DX54-5`).
  Filed **OOS-DX54-1..8** (`-6`, `-7` and `-8` by the `/review` fix cycle).
- **Tests (delta 2026-09-05, PB-DX53 + `/review` fix cycle)**: **5,210 / 0 / 5** full-workspace on
  branch `scutemob-231` (+14 over the **5,196** baseline, measured on this branch BEFORE any edit and
  **reproducing PB-DX39's close pin exactly** — the third consecutive batch in which an inherited
  pin reproduces with no correction owed; the task's AC quoted **5,194**, which is a transcription
  off by two from `CLAUDE.md`'s own PB-DX39 line, reported rather than reconciled away because a
  non-reproducing baseline is the signal `OOS-DX51-5` exists for and must not be spent on a typo),
  `--workspace --no-fail-fast` to a file, **67** result-producing targets (66 → 67: one new
  simulator test binary), residual list empty.
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs — never
  `sort` + `comm` (`OOS-DX20b-5`): 14 additions, 0 leavers, 0 removals, 0 renames.** Count delta
  14 == name-set delta 14, and the duplicate-name scan the byte-exact method is structurally blind
  to (`OOS-DX35-8`) is **EMPTY on both runs** (5,196 / 5,196 distinct; 5,210 / 5,210).
  **RE-TAKEN AFTER the `/review` fix cycle, not before it** (dispatch hygiene 8): the cycle added
  `mechanism_gate_classifier_discriminates`, so the pre-cycle figure of 13 is superseded by this
  line rather than left standing beside it.
  **HASH 84 → 85 / PROTOCOL 43 → 44, ONE bump each**, both taken from the failing gates' own output
  and **both predicted in writing before any production line** (`a37f8239`), with both closure type
  counts predicted and confirmed UNCHANGED at **98 / 132** — measured at the merge base by raising
  `MIN_CLOSURE_TYPES` to 9999 and reading the gates' own panic text, not inherited from PB-DX52.
  **The AC predicted PROTOCOL UNMOVED and that prediction is REFUTED with its own ground verified
  TRUE**: `PlayerState` really is in `CLOSURE_MUST_NOT_CONTAIN`, so a `PlayerState` field alone
  moves HASH only — but the fix cannot BE a field alone, and `Condition` is in the wire closure via
  `Effect::Conditional`. Verified BY EXECUTION at stage 0 (planting `Condition` in both gates'
  `CLOSURE_MUST_NOT_CONTAIN` fails both; `TriggerCondition` fails neither), and `rules/protocol.rs`'s
  **v21** history row already said both halves in the tree — *the same batch that created this field
  wrote down this prediction five weeks ago*.
  History rows appended, never edited; both `FROZEN_HISTORY_PREFIX_DIGEST`s re-pinned;
  `history_is_append_only` and `frozen_prefix_is_pinned` green on both. **Sentinels re-pinned by
  symbol across 49 files, then survivor-scanned on BOTH axes** (`OOS-DX36-8`) — a ±3-line window
  instead of a symbol-adjacent match AND a suffix-tolerant value pattern, because `\b` between a
  digit and `u` is not a word boundary: **0 candidates**. Then `OOS-DX18-3`'s OPPOSITE check, which
  a survivor scan is structurally blind to: all **74** added lines carrying `85`/`44` read
  individually — 58 assertion arguments, 16 history/doc/digest/continuation lines, **no prose
  rewritten**.
  Coverage **1,139 → 1,140 / 1,803 = 63.2%** by regeneration, **ONE flip, NAMED before any code**
  (`minas_tirith` `partial` → `Complete`); exactly **one** `Completeness` marker line moves in the
  whole card-def diff, checked by `git diff` over the marker rather than inferred from the total
  (PB-DX26's lesson that a stable COUNT is not a stable SET). **3 card-def edits.**
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `cargo build --workspace` clean (the SR-3 seal
  gate) — all against the FINAL tree. **`npm run build` was NOT run and that is stated rather than
  omitted**: N/A, because `git diff --numstat 5182600e..HEAD -- tools/` is **EMPTY** and
  `node_modules` is absent.
  **Engine lines** (re-taken against the FINAL tree rather than transcribed — PB-DX28's re-take
  MEDIUM): `crates/engine/src` **+150 / −30**; `crates/card-types/src` **+67 / −14**;
  `crates/card-defs` **+107 / −71**; and **`crates/simulator/src`, `crates/view-model` and
  `tools/` are all EXACTLY 0** — every consumer of the raid gate lives in the engine and the card
  defs.
  **Benches: MEASURED, SIX runs on a quiet machine, verdict NO REGRESSION — and the FIRST A/B was
  thrown away rather than published.** Merge-base runs 1-3 were taken while the implementation
  agent was compiling and moved `board_wipe_4p` **120.34 → 126.52 → 139.34 µs on IDENTICAL code**
  (a **16%** same-code spread); discarded before any comparison was computed. Re-run quiet, with
  the same-code band measured FIRST across three base runs: **0.46-1.42%**. Base-vs-HEAD medians
  −1.96% to +0.39%, four of six overlapping outright and both non-overlapping ones (−0.74%,
  −1.04%) SMALLER than the widest same-code band. **The apparent improvement is deliberately NOT
  claimed** — the controls (`priority_cycle_4p`/`6p`, `sba_check`, none of which is on any line
  this batch touches) move the same order, which is a two-compilation layout artefact.
  **And this batch's OWN prediction of a regression is refuted by measurement**, which is the
  interesting half: `size_of::<PlayerState>()` moves **376 → 400 (+6.4%)**, MORE than PB-DX18's
  +4.4% that published a real uniform 2.5-4.5% regression. The candidate explanation, offered as an
  inference with its evidence rather than as a finding: PB-DX18 grew **BOTH** structs
  (`GameState` 3512 → 3536 as well), while this batch leaves `size_of::<GameState>()` **UNMOVED at
  3536** (executed at both revisions; a `PlayerState` lives behind an `OrdMap`) — so the evidence
  points at **`GameState`'s** size as PB-DX18's real driver.
  **Revert matrix: 3 rows, EXECUTED BY THE COORDINATOR rather than accepted from the delegated
  report, 3 discriminating, 0 UNDISCRIMINATED** — R2 and R3 are precise complements of R1's
  blanket, isolating *populated* vs *read by the right consumer* vs *cleared at the right time*.
  **Two rows need their reason stated rather than their count read**: `t7` is GREEN under R1 as a
  stated CONTROL (a correct fix must not break Legion's Landing), and `t6` reddens under R1 on its
  NON-VACUITY FLOOR rather than its subject — without that floor, *"Legion's Landing did not
  transform"* would be satisfied by an engine that counted nothing at all, which is the pre-fix
  engine. **The matrix also corrected its own instrument twice**: a build-failure detector matching
  `^error(\[|:)` also matches cargo's `error: test failed`, so all three real verdicts were first
  reported as void builds (`OOS-DX39-8` inverted — an over-wide build detector turns a verdict into
  a non-verdict); and an earlier R1 patch that never applied printed seven greens that were the
  UNMODIFIED tree. Both caught before anything was published.
  Filed **OOS-DX53-1..3**.
- **Tests (delta 2026-09-05, PB-DX39)**: **5,196 / 0 / 5** full-workspace on branch
  `scutemob-230` (+40 over the **5,156** baseline, measured on this branch BEFORE any edit and
  **reproducing PB-DX52's close pin exactly** — the second consecutive batch in which an inherited
  pin reproduces with no correction owed), `--workspace --no-fail-fast` to a file, **66**
  result-producing targets (65 → 66: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs — never
  `sort` + `comm` (`OOS-DX20b-5`): 40 additions, 0 leavers, 0 removals, 0 renames.**
  **RE-TAKEN AFTER the `/review` fix cycle, not before it** (dispatch hygiene 8): the cycle added
  `r1c` and `r2c`, so the pre-cycle figure of 38 is superseded by this line rather than left
  standing beside it. Count delta 40 == name-set delta 40, and the duplicate-name scan the byte-exact method is
  structurally blind to (`OOS-DX35-8`) is **EMPTY on both runs** (5,156 lines / 5,156 distinct;
  5,196 / 5,196).
  **HASH 84 / PROTOCOL 43 BOTH UNMOVED — ZERO bumps for the whole PB**, gate-executed
  (`hash_schema` 36/36, `protocol_schema` 17/17) and **predicted PER OPTION in writing before any
  production line** (`60975661`), with the counterfactual costed rather than waved away: option
  (b), a source snapshot stored on the `StackObject`, was priced at HASH +1 / PROTOCOL unmoved plus
  a 48-site sentinel re-pin — and **rejected on CR grounds rather than cost**, because a snapshot
  taken at activation answers *"the creature equipped when the ability was ACTIVATED"* while the
  Jitte's 2005-02-01 ruling says **"most recently equipped"**. No sentinel re-pin, no survivor scan,
  no history row and no frozen-prefix re-pin were OWED; `history_is_append_only` and
  `frozen_prefix_is_pinned` executed green on both gates as the evidence.
  Coverage **UNMOVED at 1,139/1,803 = 63.2%** by regeneration, **0 flips** predicted before any
  code (clean 1,139 / todo 517 / empty 147 identical), self-dating churn reverted; **1 card-def
  edit, note-only** (`mardu_ascendancy`'s marker, rewritten to name BOTH blockers), with **no
  `Completeness` marker KIND moved anywhere** — checked by `git diff` over the marker rather than
  inferred from the unchanged total (PB-DX26's lesson that a stable COUNT is not a stable SET), so
  `OOS-CARDS2-3`'s re-deal budget was checked and found **not owed**.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `cargo build --workspace` clean (the SR-3 seal
  gate) — all against the FINAL tree, **and `clippy` FIRED there**, on `doc_lazy_continuation` in a
  probe file whose line 54 opened with `+ ` and so read as a markdown list bullet, making the three
  lines it actually reported that bullet's lazy continuation; the CAUSE line was reworded rather
  than the three symptom lines indented. **`npm run build` was NOT run and that is stated rather
  than omitted**: N/A, because `git diff main..HEAD --numstat -- tools/` is **EMPTY** and
  `node_modules` is absent — and unlike PB-DX52 no acceptance criterion predicted otherwise.
  **Engine lines** (`git diff --numstat main..HEAD`, re-taken against the FINAL tree rather than
  transcribed from a mid-batch figure — PB-DX28's re-take MEDIUM): `crates/engine/src`
  **+609 / −194** across `rules/layers.rs`, `state/mod.rs` and `rules/abilities.rs`;
  `crates/card-defs` **+13 / −2** (the one note-only marker rewrite); and **`crates/card-types`,
  `crates/view-model`, `crates/simulator/src` and `tools/` are all EXACTLY 0** — every consumer of
  a source-relative filter lives in the engine, measured before the design was chosen rather than
  asserted after.
  **Benches: MEASURED, SIX runs, verdict NO REGRESSION — and the apparent 1-1.5% improvement is
  deliberately NOT claimed.** `effect_applies_to` is on the layer walk, so the A/B is owed rather
  than optional. Same-code band measured FIRST across **three** merge-base runs taken before any
  HEAD run was compiled: **0.46-3.80%**. Base-vs-HEAD deltas −1.49% to +0.89%; four of six overlap
  outright. The two non-overlapping ones are `full_turn_6p` (−1.06%) and **`priority_cycle_4p`
  (−1.49%), which is the CONTROL** — it executes no line this batch touched and moves the same
  order, which is a build/layout artefact of two compilations, not an effect. **The mechanism is
  bounded by execution rather than argued**: the one real saving (`snapshot_affected_set` resolving
  the source view ONCE instead of per candidate) is on a mass-filter RESOLUTION, and none of the
  six benches resolves one.
  **Fuzz: the change is FUZZ-NEUTRAL BY MEASUREMENT, and the output is BYTE-IDENTICAL.** Matched
  A/B (`--games 20 --seed 1 --max-turns 200`) between merge base `604b7242` and HEAD, each in its
  own worktree with its own `CARGO_TARGET_DIR`: the fuzzer's entire program output is
  **byte-for-byte identical** (14,703 bytes each) — 20 games completed, HARD **103 / distinct 4**,
  TRANSIENT **362 / distinct 74**, rejections **2,847 / 102,803 = 27.694‰**, every per-seed band
  and every histogram row the same. Stated precisely: no observable divergence in this invocation,
  which is not the same as proving no `public_state_hash` anywhere moved.
  **Revert matrix: 6 rows, EXECUTED BY THE COORDINATOR rather than accepted from the four delegated
  reports, 6 discriminating, 0 UNDISCRIMINATED**, all three engine files verified restored
  byte-exactly — and **RE-RUN, because the first harness omitted a whole test target**
  (`cargo test -p mtg-engine --lib`, where five of this batch's most direct probes live), so every
  published red set was a FLOOR: R1 is **11** not 9, R5 is **10** not 8, and R4 is **3** not 2 with
  its third row **behavioural** — which demolished this batch's own "R4 reddens only source gates"
  finding as an artefact of its own instrument (`OOS-DX39-6`, rewritten in place). **R2 and R3 are precise complements** — R2 (the stack-capture clause) reddens the
  Jitte probes and leaves Mardu green, R3 (the activation-cost clause) does the exact opposite —
  which is the proof BOTH capture clauses are load-bearing, and the reason a batch that built only
  one would have passed every probe it thought to write for its own half. Filed **OOS-DX39-1..10** — and the first draft of this line said `-1..8`, which is dispatch
  hygiene 8's exact case for the third batch running, caught by re-checking this cell against the
  registry AFTER the `/review` fix cycle rather than before it.
- **Tests (delta 2026-09-04, PB-DX52)**: **5,156 / 0 / 5** full-workspace on branch
  `scutemob-229` (+39 over the **5,117** baseline, measured on this branch BEFORE any edit and
  **reproducing PB-DX36's close pin exactly** — the first time in five batches an inherited pin
  reproduces with no correction owed), `--workspace --no-fail-fast` to a file, **65**
  result-producing targets (64 → 65: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs — never
  `sort` + `comm` (`OOS-DX20b-5`): 40 additions, 1 leaver, 0 removals.** The single leaver is
  **disclosed and is not a removal**: `t3_ability_half_is_still_unreachable` became
  `t3_ability_half_is_reachable_via_target_stack_object` — the inversion `OOS-DX25b-1`'s own row
  and `bolt_bend.rs`'s own note both instructed, and it KEEPS the two assertions that are still
  true (the entry is still not a `state.objects` key; naming it as a bare `Target::Object` still
  fails) beside the two that inverted. Honest reading: **39 genuine additions and 1 mandated
  inversion.** Count-vs-name reconciliation run and AGREES (39 == 39); the duplicate-name scan
  the byte-exact method is structurally blind to (`OOS-DX35-8`) is **EMPTY on both runs**.
  **PROTOCOL 42 → 43 / HASH 83 → 84, ONE bump each**, both taken from the failing gates' own
  output and **both predicted in writing, per half, before any production line changed**
  (`8f919967`) — including the prediction that **neither** closure's type count would move,
  confirmed by the gates' own text at **98** and **132**. History rows appended, never edited;
  both `FROZEN_HISTORY_PREFIX_DIGEST`s re-pinned; `history_is_append_only` and
  `frozen_prefix_is_pinned` green on both; `hash_schema` + `protocol_schema` 51/51.
  **The two-step stream observation recurs for the FOURTH version running** (v40, v82, v83, now
  v84): with both variants in the tree and hashed but BEFORE the version bump,
  `declaration_fingerprint_is_pinned` was RED and `stream_fingerprint_is_pinned` **GREEN** —
  `canonical_fixture()` carries no stack object with a `StackObject` target and no non-default
  `TargetRequirement`, so none of this batch's new bytes can reach it.
  **48 HASH + 14 PROTOCOL sentinels re-pinned by symbol across 49 files**, the `u8`/`u32` suffix
  spellings included, then **survivor-scanned on BOTH axes** (`OOS-DX36-8`, the lesson that
  defeated PB-DX36's own scan): axis 1 SHAPE — a ±3-line window instead of a symbol-adjacent
  same-statement regex — AND axis 2 VALUE — every Rust integer suffix, not a bare `\b83\b`.
  **2 candidates, both correct historical prose in `hash.rs`, 0 real survivors.** Then
  `OOS-DX18-3`'s opposite check: every changed line of the re-pin diff read individually — 62
  assertion arguments and 2 frozen-prefix pins moved, **no prose rewritten**.
  Coverage **UNMOVED at 1,139/1,803 = 63.2%** by regeneration, **0 flips** predicted with a
  per-def reason before any regeneration (`caf642f9`) and confirmed (clean 1,139 / todo 517 /
  empty 147 identical), self-dating churn reverted; **no `Completeness` marker moved anywhere**,
  checked by `git diff` over the marker rather than inferred from the unchanged total (PB-DX26's
  lesson that a stable COUNT is not a stable SET), so `OOS-CARDS2-3`'s re-deal budget was checked
  and found **not owed**.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `cargo build --workspace` clean (the SR-3 seal
  gate) — all against the FINAL tree, **and `clippy` FIRED there**, on `manual_contains` in a
  channel probe added after the implementers' own runs. **`npm run build` was NOT run and that is
  stated rather than omitted**: N/A, because `git diff main..HEAD --numstat --
  tools/play-server/frontend` is **EMPTY** and `node_modules` is absent — **and the acceptance
  criterion predicted otherwise, so the refutation is reported rather than skipped**.
  AC 7352 says the frontend *"will"* move *"if the picker learns a new target kind"*; it does
  not, because `TargetPicker.svelte` echoes each candidate's `.value` verbatim and displays
  `.label`, and **never reads `.kind` at all** (grepped across the whole frontend: the only
  `.kind` consumers are `CostPicker`'s cost tags and `stores.js`'s `ApiError`). `tools/` is
  **not** zero: `play-server/src/view.rs` gains the `"stack_object"` wire kind and a THIRD
  `NameIndex` map keyed by stack-entry id — deliberately not folded into the `ObjectId`-keyed
  one, because that exact fold is the collision that file's own `from_view` comment records as a
  shipped bug — and `tools/tui/` gains a render arm plus one Cargo dependency.
  **Benches: MEASURED, SEVEN runs, verdict NO REGRESSION — and the FIRST A/B was thrown away
  rather than published.** Base runs 1-2 were taken while this session's own test suite and
  revert matrix were running; their same-code band came out at **up to 47%**
  (`full_turn_4p` 326.58 vs 221.96 µs on IDENTICAL code) and the contaminated table read
  *"HEAD 30% faster on `sba_check`"* — an effect nothing in this batch can cause, which is the
  tell. Re-run on a quiet machine: same-code band **0.10-1.65%**, base-vs-HEAD deltas
  **−0.18% to −2.22%**. **The apparent improvement is deliberately NOT claimed**, on three
  grounds: HEAD's own three-run spread (**5.2%** on `sba_check`) is wider than every difference
  in the table; the **controls** (`priority_cycle_4p`/`6p`, `sba_check` — nothing here is on the
  priority loop or the SBA loop) move the same order as everything else; and the mechanism bound
  is **measured rather than argued** — `crates/engine/benches/engine_perf.rs` contains **zero**
  occurrences of any symbol this batch touched, and `size_of` executed **at both revisions** is
  identical (`Target` **16 → 16**, `SpellTarget` **32 → 32**, `TargetRequirement` **304 → 304**,
  `StackObject` **504 → 504** — the new variant carries exactly one `ObjectId`, like the two it
  joins).
  **Fuzz: NOT A/B'd, and the reason is stated as a reason rather than dressed as a measurement**
  — no `Completeness` marker moved, so no seeded fixture is re-dealt and `OOS-CARDS2-3`'s usual
  budget does not apply.
  **Revert matrix: 8 rows, EXECUTED BY THE COORDINATOR rather than accepted from the delegated
  reports, 8 discriminating, 0 UNDISCRIMINATED** — and row **R6 is a coverage measurement, not a
  pass**: undoing the CR 702.16b fix reddened only `r7b`, a SOURCE gate, with no behavioural
  probe moving at all. Closed by `t10`, RED under R6 on its own assertion. Filed as
  `OOS-DX52-2`.
- **Tests (delta 2026-09-04, PB-DX36 + `/review` fix cycle)**: **5,117 / 0 / 5** full-workspace on
  branch `scutemob-228` (+20 over the **5,097** baseline, measured on this branch BEFORE any edit and
  **reproducing PB-DX35's close pin exactly** — `OOS-DX51-5`'s non-reproducing-pin failure did not
  recur), `--workspace --no-fail-fast` to a file, **64** result-producing targets (63 → 64: one new
  simulator test binary), residual list empty.
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs — never
  `sort` + `comm` (`OOS-DX20b-5`): 20 additions, 0 leavers, 0 removals, 0 renames.** Count delta
  20 == name-set delta 20, and the duplicate-name scan the byte-exact method is structurally blind
  to (`OOS-DX35-8`) is **EMPTY on both runs**. **RE-TAKEN after the `/review` fix cycle, not before
  it** (dispatch hygiene 8): the cycle added `t8`/`t9`, so the pre-cycle figure of 18 is superseded
  by this line rather than left standing beside it.
  **PROTOCOL 41 → 42 / HASH 82 → 83, ONE bump each**, both taken from the failing gates' own output
  and **both predicted in writing, per half, before any production line changed** (`a9fca688`) —
  including the prediction that **neither** closure's type count would move, confirmed by the
  gates' own text at **98** and **132**. Every wire cell was **PROBED at stage 0**, not inherited
  from the v4 memo: each gate's `CLOSURE_MUST_NOT_CONTAIN` was temporarily extended and its closure
  walk executed, giving `TriggerCondition` OFF-wire, `PendingTrigger` OFF-wire, and
  `EffectAmount` / `TriggerEvent` / `TriggeredAbilityDef` ON-wire. History rows appended, never
  edited; both `FROZEN_HISTORY_PREFIX_DIGEST`s re-pinned; `history_is_append_only` and
  `frozen_prefix_is_pinned` green on both; `hash_schema` 36/36, `protocol_schema` 17/17.
  **The two-step stream observation recurs for the third version running** (v40, v82, now v83):
  with every change in the tree and hashed but BEFORE the version bump,
  `declaration_fingerprint_is_pinned` was RED and `stream_fingerprint_is_pinned` **GREEN** —
  `canonical_fixture()` carries no pending trigger, no stack object with a damage amount and no
  card registry, so none of this batch's new bytes can reach it.
  **THE SENTINEL SWEEP FAILED ONCE AND THE SURVIVOR SCAN REPRODUCED THE FAILURE, WHICH IS THE
  DURABLE HALF.** The first sweep re-pinned 48 HASH + 13 PROTOCOL by symbol and **missed
  `pb_dx2_command_gates.rs`'s `41u32`** — `\b` between `1` and `u` is not a word boundary, which is
  `OOS-DX20b`'s own `79u8` lesson, handled for HASH (`82(u8)?`) five lines earlier in the same
  script and not carried across to the sibling symbol. The survivor scan obeyed PB-DX50's rule to
  the letter — a ±3-line window instead of a symbol-adjacent match, a genuinely different SHAPE —
  and still reported **0 real survivors**, because it used the same **value** pattern `\b41\b`.
  *A survivor scan has TWO axes, the shape of the match and the spelling of the value, and varying
  one while holding the other is half a check* (`OOS-DX36-8`). Re-swept with a suffix-tolerant
  value pattern on both symbols: final **49 HASH + 14 PROTOCOL**, 0 real survivors, the only
  remaining hits being two correct historical-prose lines. Then `OOS-DX18-3`'s opposite check —
  all 61 changed lines of the first sweep read individually, all 61 assertion arguments, no prose
  rewritten.
  Coverage **1,138 → 1,139 / 1,803 = 63.2%** by regeneration, **ONE flip, NAMED before any code**
  (`exalted_angel` `partial` → `Complete`); exactly **one** `Completeness` marker line moves in the
  whole card-def diff, checked by `git diff` over the marker rather than inferred from the count.
  **8 card-def files edited** — and the first draft of this line said **6**, transcribed from the
  stage-0 §0.4 prediction list rather than re-taken from the diff after `tandem_lookout` and
  `niv_mizzet_visionary` were narrowed. PB-DX28's re-take MEDIUM, caught by this batch's own
  `/review`. `CORPUS_COMPLETE` 1138 → **1139**, with `COMMANDER_POOL`
  **re-measured by executing the gate and found UNCHANGED at 90** — measured, not reasoned from
  "an Angel is not Legendary".
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs) — all against the FINAL tree. **`npm run build` was
  NOT run and that is stated rather than omitted**: N/A, because `git diff main..HEAD --numstat --
  tools/play-server/frontend` is **EMPTY** and `node_modules` is absent. `tools/` is **not** zero —
  `play-server/src/main.rs` moves inside `#[cfg(test)]` for the `UI3_SPLIT_COMBAT_SEED`
  re-observation.
  **Benches: MEASURED, SIX runs, verdict NO REGRESSION — and the apparent improvement is
  deliberately NOT claimed.** Matched-set A/B against merge base `e7d7ae31`, each revision in its
  own worktree with its own `CARGO_TARGET_DIR`. Same-code band measured FIRST across **three**
  merge-base runs: 0.31-**3.76%**, the widest being `sba_check`. Every base-vs-HEAD difference in
  the table is smaller than that band. `board_wipe_4p`'s HEAD run 1 read 117.53 µs against a base
  range of 120.17-121.52 and would have published a tidy −2.7%; HEAD run 2 reads **120.27**, inside
  the base range, so run 1 is the outlier and not the effect. **Bounded independently by mechanism
  rather than left to the numbers**: the criterion's premise that *"`DamageDealt` dispatch is on the
  hot path"* is **false of every BENCHED path** — `board_wipe_4p` is a `DestroyAll`, and
  `full_turn_4p`/`6p` walk *through* the CombatDamage step with **no attackers declared**, so
  `assignments` is empty and the extracted loop does nothing.
  **Fuzz: the engine half is FUZZ-NEUTRAL BY MEASUREMENT.** Five seeded fixtures reddened on the
  `CORPUS_COMPLETE` re-deal; an executed ablation in an isolated worktree — the entire engine change
  in the tree, ONLY `exalted_angel`'s marker forced back to `partial` — turns **all five green**, so
  every bit of the movement is `OOS-CARDS2-3` and none of it is this batch's dispatch change.
- **Tests (delta 2026-09-04, PB-DX35)**: **5,097 / 0 / 5** full-workspace on branch
  `scutemob-227` (+39 over the **5,058** baseline, measured on this branch BEFORE any edit and
  reproducing PB-DX51's close pin exactly), `--workspace --no-fail-fast` to a file, **63**
  result-producing targets (61 → 63: two new test binaries), residual list empty.
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs — never
  `sort` + `comm` (`OOS-DX20b-5`): 39 additions, 0 leavers, 0 removals, 0 renames.**
  **And the delta's FIRST run was wrong in a way the byte-exact method is structurally blind to.**
  It reported **32 additions against a count delta of 33**, because Half B named its bot-path
  channel probe `c3_the_bot_path_is_offered_and_answers_the_same_action` — a name
  `crates/simulator/tests/pb_dx45_optional_cost_channel.rs:293` already used. `tests/` compiles one
  binary per file, so both compiled and both ran; a set difference over NAMES collapses the pair.
  Renamed, and **the check that separates them costs one line and is now run on this batch's own
  close-out numbers, and RE-TAKEN after the `/review` fix cycle**: count delta 39 == name-set
  delta 39, duplicate-name scan EMPTY
  (`OOS-DX35-8`).
  **HASH 82 / PROTOCOL 41 BOTH UNMOVED — ZERO bumps for the whole PB**, gate-executed
  (`hash_schema` 36/36, `protocol_schema` 17/17) and **both predicted in writing before any
  production line changed** (`c6646052`), per half, with the reason stated rather than asserted:
  `TriggeredAbilityDef` carries **no `modes` field at all**, so the REGISTRY is already the
  incumbent source of a modal trigger's `ModeSelection` at both existing read sites and reading
  `mode_targets` there adds no type, variant or field; and Half B mints no question variant. The
  `both-if-lowered` counterfactual is **costed rather than waved away** — that struct has no
  `Default` derive and **190 exhaustive struct literals across 44 files** construct it, and it is
  reachable from `Characteristics`, a PROTOCOL closure root. `git diff` over `state/hash.rs` and
  `rules/protocol.rs` is **EMPTY**, so no sentinel re-pin and no history row were owed;
  `history_is_append_only` and `frozen_prefix_is_pinned` executed green on both gates as the
  evidence.
  Coverage **1,137 → 1,138 / 1,803 = 63.1%** by regeneration, **ONE flip, NAMED before any code**
  (`shambling_ghast` `partial` → `Complete`; clean +1 / todo −1 / empty 147 unchanged, and the
  report's bucket is MARKER-driven, so the pair IS that one def). **12 card-def edits, of which 9
  are comment-only**; `CORPUS_COMPLETE` 1137 → **1138**, with `COMMANDER_POOL` **re-measured and
  found UNCHANGED at 90** rather than reasoned about.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `cargo build --workspace` clean (the SR-3 seal
  gate) — all against the FINAL tree, **and `cargo fmt --check` FIRED there**, on a file added
  after the implementers' own runs. **`npm run build` was NOT run and that is stated rather than
  omitted**: N/A, because `git diff main..HEAD --numstat -- tools/play-server/frontend` is **EMPTY**
  and `node_modules` is absent. `tools/` is **not** zero — `play-server/src/main.rs` is
  **+366 / −0, entirely inside `#[cfg(test)]`**, plus 6 doc lines in `view.rs` and 4 in
  `tui/play/app.rs`.
  **Engine lines**: `crates/engine/src` **+722 / −356** (`rules/abilities.rs` +636/−337, but
  `git diff -w` gives **+460 / −161** — the rest is a `cargo fmt` reflow of the shortened
  derivation chain, stated so 600 lines do not read as new logic); `crates/card-types/src`
  **+6 / −2**; `crates/simulator/src` **+51 / −13**; **`crates/view-model` is 0**.
  **Benches: MEASURED, SEVEN runs, verdict NO REGRESSION — and the one outlier was killed by a
  third run on each side rather than averaged away.** Same-code band measured FIRST across two
  merge-base runs: **0.42-2.14%**. Every base-vs-HEAD criterion interval overlaps.
  `board_wipe_4p`'s HEAD run 1 read **135.15 µs** against a base range of 119.33-121.57, and taking
  the mean of two HEAD runs would have published a tidy, meaningless **+5.37%**; a third run each
  side (HEAD **116.73** / base **120.23**) puts both later HEAD runs below every base run, so run 1
  was contended and is discarded with its reason stated. **The resulting apparent 1.4-3%
  improvement is deliberately NOT claimed** — `sba_check` and the priority cycles are controls
  (nothing here is on the SBA loop or the priority-pass path) and move the same order.
  **Fuzz: NOT A/B'd, and the reason is attribution rather than effort** — the flip moves
  `CORPUS_COMPLETE`, which re-deals every seeded game, so a cross-boundary A/B would measure
  `OOS-DX21-6` trajectory reindexing. The in-tree gate config was **re-observed by execution**
  instead, and its served-row partition gained the new `look_at_top_then_place_optional` row.
- **Tests (delta 2026-09-04, PB-DX51)**: **5,058 / 0 / 5** full-workspace on branch
  `scutemob-226` (+14 over the **5,044** baseline, measured on this branch BEFORE any edit),
  `--workspace --no-fail-fast` to a file, **61** result-producing targets (60 → 61: one new
  simulator test binary), residual list empty.
  **The baseline does NOT reproduce PB-DX18's published close pin of 5,041, and that is reported
  rather than reconciled away**: `git diff 2861b3a7..71113bda --stat -- '*.rs'` is EMPTY, so the
  tree is byte-identical in Rust to PB-DX18's final commit and the discrepancy is in the
  MEASUREMENT. Candidate mechanism, stated as an inference with its evidence: PB-DX18's `/review`
  fix cycle `b72b8c80` adds **7** `#[test]` items and the close-out commit `2861b3a7` that
  published the figure came after it — PB-DX28's "re-take the measured table" MEDIUM, for the
  fourth time in this queue and the first time on the pin every subsequent batch inherits
  (`OOS-DX51-5`).
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs — never
  `sort` + `comm` (`OOS-DX20b-5`): 14 additions, 0 leavers, 0 removals, 0 renames.** No doctest
  line-number shift, because nothing this batch edits sits above one.
  **HASH 80 → 81 → 82 / PROTOCOL 41 UNMOVED, ONE bump**, both gate-computed (`hash_schema` 36/36,
  `protocol_schema` 17/17) and **both predicted in writing before any production line changed**
  (`06ba6760`), with the reason stated: `CombatState` is reachable only through
  `GameState::combat` and `CLOSURE_MUST_NOT_CONTAIN` excludes `GameState` (the PB-DX21 precedent,
  `CombatState.attackers_declared`, HASH 72 → 73 / PROTOCOL 35 unmoved). **Closure type count
  UNMOVED at 132 — measured at the merge base by temporarily raising `MIN_CLOSURE_TYPES`, not
  assumed**; the 131 → 132 move belongs to PB-DX18's `PregamePhase`. History row appended, never
  edited; `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned; `history_is_append_only` and
  `frozen_prefix_is_pinned` green. **47 HASH sentinels across 46 files re-pinned by symbol** (2
  spelled across a line break, 2 spelled `, 81,` with no `u8` suffix — both known failure modes
  handled), then survivor-scanned with a differently-shaped line-window matcher (**0 real
  survivors**, 7 candidates all historical prose) AND every changed line of the diff read for an
  OVER-replacement (`OOS-DX18-3`): exactly 47 assertion lines moved, no prose rewritten.
  **A two-step digest observation worth keeping**: with the field added AND hashed but BEFORE the
  version bump, `declaration_fingerprint_is_pinned` was RED and `stream_fingerprint_is_pinned` was
  **GREEN** — `canonical_fixture()` never populates `GameState::combat`, so no `CombatState` field
  can reach the stream digest at all. The stream moved afterwards only because
  `HASH_SCHEMA_VERSION` is its own first byte. Filed as `OOS-DX51-4`.
  Coverage **1,137/1,803 = 63.1%** by regeneration, **0 flips** as predicted (clean 1,137 / todo
  519 / empty 147 identical), self-dating churn reverted; **0 card-def edits of any kind** —
  `git diff main..HEAD --numstat` over `crates/card-defs` and `crates/card-types/src/cards` is
  **empty**, so the shortcut was available and the regeneration was run anyway.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `cargo build --workspace` clean (the SR-3 seal
  gate) — all against the FINAL tree. **`npm run build` was NOT run and that is stated rather than
  omitted**: it is N/A here, because `git diff main..HEAD --numstat -- tools/` is **EMPTY** and
  `node_modules` is absent from this worktree.
  **Engine lines**: `crates/engine/src` + `crates/card-types/src` is **+199 / −23**;
  `crates/simulator/src` is **+9 / −1** (the one offer conjunct and its comment);
  **`crates/view-model` and `tools/` are both 0**.
  **Benches: FIVE runs, and the verdict is NO REGRESSION with the apparent SPEED-UP explicitly not
  claimed.** Same-code band measured FIRST across three merge-base runs (0.5-1.9%). HEAD reads
  1-3.7% faster on five of six. **A third base run was taken specifically to kill the "the base
  runs were contended" explanation, and it killed it** — same quiet machine, after both HEAD runs,
  reproducing base runs 1-2 (`full_turn_4p` 222.80 against 221.68 / 221.79). **The claim is still
  not "3% faster", because `priority_cycle_4p` is the control and it moves too**: that bench
  executes no line this batch touched and shifts 0.9-1.9%, the same order as `full_turn_4p`'s
  2.7-2.8%. A uniform shift across a bench that cannot be affected is a build/layout artefact of
  two separate compilations — PB-DX20b's own tell, reached from the other direction.
  **Fuzz: the movement is ATTRIBUTED by an executed ablation, not excused.** PB-DX32 gate config
  (seeds [1,2,3] × 25 turns) is **byte-identical** before and after — `18 / 2717 = 6.625‰`, waste
  `97 / 105 = 92%`, the same six served rows — and byte-identical AGAIN with the new conjunct
  entirely removed, which is what proves the suppression window is never reached by those
  trajectories. On the wider `--games 20 --seed 1 --max-turns 200` run the base is `2931 / 110271
  = 26.580‰` with **`AlreadyDeclaredBlockers` 9** and HEAD is `3234 / 108802 = 29.724‰` with that
  class **absent** — the closure evidence. A third run carrying the FULL engine change with ONLY
  the offer conjunct ablated reproduces the merge base **byte-identically** (HARD 90 / distinct 13,
  TRANSIENT 680 / distinct 159, avg turns 117.8), so **the engine half is fuzz-neutral by
  measurement** and every bit of the HEAD-vs-base movement — including HARD 90 → 198 — is
  `OOS-DX21-6` trajectory reindexing: identical violation CLASSES at both revisions, DISTINCT
  count 13 → **12**, and raw counts are checkpoint-weighted, which the fuzzer's own output says is
  not the defect-shaped number.
  **Revert matrix: 6 rows, 0 UNDISCRIMINATED, and R1/R2/R4/R5 were RE-EXECUTED independently by
  the coordinator rather than accepted from the delegated report** (all four reproduce exactly).
  Plus **four executed defeats of this batch's own `r1` gate**, of which the first succeeded — see
  the narrative and `OOS-DX51-6`.
- **Tests (delta 2026-09-04, PB-DX18)**: **5,041 / 0 / 5** full-workspace on branch
  `scutemob-225` (+26 over the **5,015** baseline, measured on this branch BEFORE any edit and
  reproducing PB-DX20b's close pin exactly), `--workspace --no-fail-fast` to a file, **60**
  result-producing targets (unmoved), residual list empty.
  **Delta itemised by test NAME by a BYTE-EXACT Python set difference of the two run logs — not
  `sort` + `comm`, which fabricates a removal under a UTF-8 locale (`OOS-DX20b-5`, obeyed rather
  than re-learned): 28 additions, 2 leavers, 0 removals, 0 renames.** Both leavers are disclosed
  and neither is a removal: the two `state::GameState` doctests are named by their LINE NUMBER,
  and both shifted by exactly **+2** — the height of the `pub mod pregame;` / `pub use
  pregame::PregamePhase;` pair added to `state/mod.rs`. Honest reading: **26 genuine additions and
  2 line-number shifts.** The 26 are 7 pregame/cap probes, 5 CR 702.94a miracle probes (in a file
  that was **one byte** and `mod`-declared since SR-9a — `OOS-DX18-2`), 4 targetless-spell probes,
  5 roster gates, 4 `PinnedRng` unit tests and 1 play-server HTTP probe.
  **PROTOCOL 41 UNMOVED / HASH 80 → 81, ONE bump**, both gate-computed (`protocol_schema` 17/17,
  `hash_schema` 36/36) and **both predicted in writing before any production line changed**
  (`82154219`), with the reason stated: `CLOSURE_MUST_NOT_CONTAIN` lists `GameState` and
  `PlayerState`. The prediction survived a mid-batch addition it did not anticipate —
  `AbilityDefinition::Splice` gained a field, and PROTOCOL still did not move, because
  `AbilityDefinition` is reachable only through `CardDefinition`, which the same list excludes,
  and it moved the STREAM digest and not the DECLARATION one because `card_registry` is
  `#[serde(skip)]`. History row appended, never edited; `FROZEN_HISTORY_PREFIX_DIGEST` re-pinned;
  `history_is_append_only` and `frozen_prefix_is_pinned` green. **87 sentinel sites across 47
  files re-pinned by symbol**, then survivor-scanned with a differently-shaped line-window matcher
  — **0 survivors, and that scan was structurally incapable of catching the one thing that went
  wrong** (see `OOS-DX18-3`).
  Coverage **1,137/1,803 = 63.1%** by regeneration, **0 flips** as predicted (clean 1,137 / todo
  519 / empty 147 identical), self-dating churn reverted; **3 card-def edits** — two comment-only
  (`darksteel_colossus`'s note and header) and one authoring `glacial_ray`'s CR 702.47a splice
  targets — with **no `Completeness` marker moved**, so the `CORPUS_COMPLETE` SET is unmoved and
  no seeded fixture was re-dealt for that reason.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `cargo build --workspace` clean (the SR-3 seal
  gate) — all against the FINAL tree. **`npm run build` was NOT run and that is stated rather than
  omitted**: `git diff main..HEAD --numstat -- tools/play-server/frontend` is **empty** and
  `node_modules` is absent from this worktree. `tools/` is **not** zero —
  `tools/play-server/src/main.rs` is **+108 / −0**, entirely inside its `#[cfg(test)]` module.
  **Engine lines**: `crates/engine/src` + `crates/card-types/src` + `crates/card-defs` +
  `crates/simulator/src` + `crates/view-model` is **+1079 / −69**; `crates/view-model` is **0**
  and `crates/simulator/src` is **+27 / −0** (a comment recording the `OOS-DX18-1` trade).
  **Benches: a REAL uniform regression, published as one.** Four matched runs, each revision in
  its own worktree with its own `CARGO_TARGET_DIR`, and **the same-code repeatability band was
  measured BEFORE the verdict was written** (PB-DX20b's lesson): the two merge-base runs differ by
  **3.3%** on `priority_cycle_4p` and **4.5%** on `sba_check`. Against that band, five of six
  benches show non-overlapping intervals — `priority_cycle_4p` ~+2.5%, `priority_cycle_6p` ~+4.0%,
  `full_turn_4p` ~+4.5%, `full_turn_6p` ~+2.5%, `board_wipe_4p` ~+2.7% — and `sba_check`'s +2.3%
  is honestly **marginal**. **The uniformity is the informative part**: nothing this batch adds is
  on the SBA loop or the priority cycle, and the candidate mechanism is bounded by execution
  rather than argued — `size_of::<GameState>()` moves **3512 → 3536** and
  `size_of::<PlayerState>()` **360 → 376** (+4.4%), on a struct copied at every mutation, plus one
  enum and one `Option` per player in every `public_state_hash`.
  **Revert matrix: 13 rows executed, 13 discriminating, 0 UNDISCRIMINATED** — and **R2 defeated
  this batch's own `r1` gate**, which is the row worth reading (`memory/primitives/
  pb-DX18-execution-notes.md` §3).
- **Tests (delta 2026-09-03, PB-DX20b)**: **5,015 / 0 / 5** full-workspace on branch
  `scutemob-222` (+24 over the **4,991** baseline, measured on this branch BEFORE any edit and
  reproducing PB-DX50's close pin exactly), `--workspace --no-fail-fast` to a file, **60**
  result-producing targets (59 → 60: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME by set-diffing the two run logs: 25 additions, 1 leaver, 0
  removals** (23 at the batch's close, +2 in the `/review` fix cycle — `t10` and `t11`; the fix
  cycle's other two findings edited existing tests in place, so it added 0 leavers and 0 renames).** The single leaver is disclosed and is not a removal:
  `pb_dx49_saga_blanking_roster::r4a_pair_a_depends_on_oos_dx20_10` became
  `..._is_dead_since_oos_dx20_10_closed`, because **PB-DX49's Pair A existed only because of the
  defect this batch closed** and that row was written to go red and demand re-adjudication rather
  than silently vacate. Honest reading: **22 genuine additions and 1 re-adjudicated rename.**
  **A measurement error inside this batch's own close-out, recorded because it fails toward the
  one thing the criterion exists to catch**: the first delta used `sort` + `comm` and reported
  24 additions / **2** leavers, the extras being two tests this batch never touched, each present
  once with `... ok` in BOTH logs. `sort` under `en_US.UTF-8` collates by locale while `comm`
  compares byte-wise, so they disagree around `::` and `_` and `comm` invents rows. Redone as a
  byte-exact set difference. **A NAME delta taken with `sort` + `comm` under a UTF-8 locale is
  not a delta, and it fabricates a REMOVAL** (`OOS-DX20b-5`).
  **PROTOCOL 40 → 41 / HASH 79 → 80, ONE bump each**, both taken from the failing gates' own
  output and **both predicted in writing before any production line changed** (`21f68337`),
  including the prediction that neither closure's type count would move — confirmed by the gates'
  own text at **98 / 131**. History rows appended, never edited; both
  `FROZEN_HISTORY_PREFIX_DIGEST`s re-pinned; `history_is_append_only` and `frozen_prefix_is_pinned`
  green on both; `hash_schema` + `protocol_schema` 51/51. **PB-DX50's sentinel lesson recurred
  inside the batch that had it available**: the census (47 HASH + 13 PROTOCOL, multi-line-aware)
  reproduced PB-DX50's corrected figures exactly, and then the first re-pin regex replaced
  **2 of 47** — because the tree spells the sentinel `79u8` and `\b` between `9` and `u` is not a
  boundary. Caught by an independent survivor scan with a differently-shaped regex, not by the
  re-pin. *A re-pin is only as wide as the spelling its regex matched, and "spelling" includes the
  literal's type suffix.*
  Coverage **1,137/1,803 = 63.1%** by regeneration, **0 flips** as predicted (clean 1,137 / todo
  519 / empty 147 identical), self-dating churn reverted; **3 card-def edits**, all
  keyword-declaration changes, **no `Completeness` marker moved** — so the `CORPUS_COMPLETE` SET
  is unmoved as well as its count and `OOS-CARDS2-3`'s re-deal budget was checked and found not
  owed, rather than assumed away.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs) — all against the FINAL tree. **`npm run build` was
  NOT run and that is stated rather than omitted**: `node_modules` is absent from this worktree and
  `git diff main..HEAD --numstat -- tools/play-frontend` is **empty**. `tools/` is **not** zero —
  `tools/play-server/src/main.rs` is **+386 / −0**, entirely inside its `#[cfg(test)]` module.
  **Engine lines**: `crates/engine/src` **+146 / −52** across four files, of which
  `rules/protocol.rs` (+27/−2) and `state/hash.rs` (+39/−1) are almost entirely the two appended
  history rows and their doc paragraphs; `crates/card-types/src` **+18 / −0**;
  **`crates/view-model` and `crates/simulator/src` are both 0** — every consumer of the Enchant
  restriction lives in the engine, measured before the design was chosen rather than asserted
  after.
  **Benches: MEASURED, four runs, and the honest answer is "no regression demonstrated".**
  Matched-set A/B against merge base `e457f931` in an isolated worktree with its own
  `CARGO_TARGET_DIR`. Base → HEAD alone reads *"`sba_check` +1.2%, everything else 2-4% faster"*
  — and "everything else 2-4% faster" is not something this change can cause, which is the tell
  that the comparison is contaminated. **The same code benched twice** moves `priority_cycle_6p`
  −1.5%, `full_turn_6p` +1.2% and `board_wipe_4p` +1.4%; and a second merge-base run puts
  `sba_check` at **15.114-15.204 µs, SLOWER than either HEAD run**, so the two runs of identical
  base code differ by **4.1%** — wider than any base-vs-HEAD difference measured. Not *"within
  the historical band"* (the phrasing PB-DX49's `/review` refuted) but *"the same-code
  repeatability band measured in this session is wider than the effect"*. Bounded independently by
  two mechanism facts: `size_of::<KeywordAbility>()` is **88 bytes at BOTH revisions** (executed
  at each; `EnchantTarget` grew 56 → 80 but is still not the largest variant, so nothing on the
  layer/SBA hot path got bigger), and `crates/engine/benches/engine_perf.rs` contains **zero**
  occurrences of `Aura` or `Enchant`, so the one function that gained an allocation is off every
  benched path by construction.
  **Revert matrix: 16 rows executed — 11 engine + 5 channel — all discriminating, 0
  UNDISCRIMINATED**, and re-executed independently by the coordinator in a fresh isolated worktree
  rather than accepted from the delegated reports. R1 (widen back to `EnchantTarget::Permanent`)
  reproduces on **all four surfaces**; R5's headline reproduces exactly — with an eighth
  `EnchantFilter` field planted, `cargo build --workspace` prints `Finished` with **zero errors**
  and all ten behavioural probes stay green, so `r5` is the only thing in the tree that catches an
  unlowered field. Two structural findings the matrix produced that argument would not: the CR
  303.4a gate is **one-directional** (R3 shows it adds nothing in the accepting direction; R10
  shows it is decisive in the refusing one, so deleting it as "covered upstream" is half right and
  half wrong), and **two reverts were not enough** — R-A and R-B are both over-wide and cannot
  redden the "no printed-legal target refused" half, so without a third UNDER-wide revert two of
  five channel rows would have been honestly UNDISCRIMINATED.
- **Tests (delta 2026-09-03, PB-DX50)**: **4,991 / 0 / 5** full-workspace on branch
  `scutemob-221` (+50 over the **4,941** baseline, measured on this branch BEFORE any edit and
  reproducing PB-DX49's close pin exactly), `--workspace --no-fail-fast` to a file, **59**
  result-producing targets (58 → 59: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME by set-diffing the two run logs: 53 additions, 3 leavers, 0
  removals, 0 renames.** The three leavers are **disclosed individually and none is a removal** —
  they are PB-DX29's mutate trio: `test_dx29_m1_provider_offers_both_on_top_and_under` and
  `test_dx29_m2_params_forwards_the_actions_on_top_choice` are **inversions** (the provider stops
  offering the over/under pair, because the choice is no longer made at cast time), and
  `test_dx29_m3_mutating_under_keeps_the_hosts_characteristics` is **re-homed** onto the
  resolution-time answer, which is the proof AC 7302 required be preserved. Each has a named
  `test_dx50_*` successor in the additions. Honest reading: **50 genuine additions, 2 inversions,
  1 re-home.**
  **PROTOCOL 39 → 40 / HASH 78 → 79, ONE bump each**, both taken from the failing gates' own
  output and **both predicted in writing, per half, before any production line changed**
  (`595e4e28`): Half 1 (target legality) was predicted to move **neither** fingerprint and moved
  neither; Half 2 (CR 702.140c timing) was predicted to move each **once** and did. Type counts
  predicted unchanged at **98 / 131** and confirmed at 98 / 131. History rows appended, never
  edited; both `FROZEN_HISTORY_PREFIX_DIGEST`s re-pinned; `history_is_append_only` and
  `frozen_prefix_is_pinned` green on both. **The sentinel census in this batch's own plan was
  WRONG and its own survivor check reproduced the error**: the plan published 45 HASH + 11
  PROTOCOL from a same-line regex *while explicitly citing PB-DX45's lesson that a re-pin is only
  as wide as the spelling its regex matched*; the truth is **47 + 13**, the extras spelling the
  assertion across a line break. Re-verified with an independent multi-line scan: **0 stale
  survivors**. *A survivor check written with the same regex as the re-pin is not a check.*
  Coverage **1,137/1,803 = 63.1%** by regeneration, **0 flips** as predicted (clean 1,137 / todo
  519 / empty 147 identical), self-dating churn reverted; **0 card-def edits of any kind** —
  `git diff main..HEAD --numstat` over `crates/card-defs` and `crates/card-types/src/cards` is
  **empty**, so the shortcut was available and the regeneration was run anyway.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), **`npm run build` green** — all against the FINAL
  tree. `npm run build` was RUN rather than declared N/A, because `tools/` is **not** zero here:
  `play-server/src/{main,view,api}.rs`, `ActionBar.svelte` and the new `BinaryChoicePicker.svelte`
  all move, and `tui/play/app.rs` and `replay-viewer/src/replay.rs` follow the deleted field.
  **Engine lines**: `crates/engine/src` is **+665 / −149** across halves 2-3 on top of Half 1's
  `casting.rs` / `queries.rs` / `resolution.rs`; `crates/view-model` is **0**.
  **Benches: NOT measured, and therefore nothing is claimed.** Nothing this batch touches is on
  `sba_check` / `priority_cycle` / `full_turn`, and the two changes that could move anything both
  *remove* work (one fewer trigger sweep per answered choice, half as many mutate offers). That is
  a reason to expect no regression, not a measurement — stated as such rather than published as a
  band, which is the claim PB-DX49's `/review` refuted.
- **Tests (delta 2026-09-03, PB-DX49)**: **4,941 / 0 / 5** full-workspace on branch
  `scutemob-220` (+41 over the **4,900** baseline, measured on this branch BEFORE any edit and
  reproducing PB-DX48's close pin exactly), `--workspace --no-fail-fast` to a file, **58**
  result-producing targets (57 → 58: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME by set-diffing the two run logs: 41 additions, 0 removals,
  0 leavers, 0 renames** — 10 in the new
  `crates/engine/tests/primitives/pb_dx49_blanked_saga_sites.rs`, **24** in the new
  `crates/engine/tests/core/pb_dx49_saga_blanking_roster.rs` (20 shipped, +4 in the `/review` fix
  cycle), 4 in the new `crates/simulator/tests/pb_dx49_saga_blanking_channel.rs`, and **3** in
  `tools/tui/src/dashboard/parser.rs`'s new `#[cfg(test)]` module (the `/review`'s LOW 7). **"0 leavers" is literal here** — the
  three `fire_saga_chapter_triggers` call sites in the test tree lost a parameter and were edited
  **in place**, so no test name changed.
  **PROTOCOL 39 / HASH 78 both UNMOVED**, gate-executed and **predicted in writing before any code
  changed** (`57d1dc42`), with the reason stated rather than asserted: the batch adds free functions
  and one engine-internal struct that is not reachable from the `Command`/`GameEvent`/`Effect`/
  `Characteristics` closure, and adds no field to any hashed type — the whole change is a *read* of
  `state.continuous_effects` and `obj.status.face_down`, both already hashed. `history_is_append_only`
  and `frozen_prefix_is_pinned` green on both; no pin edited, no history row appended, because none
  was owed. **The counterfactual is stated because §1g's row is why it matters**: lowering
  `AbilityDefinition::SagaChapter` into `Characteristics` would have moved BOTH fingerprints, which
  is exactly why the continuous-effect-scan design was mandated.
  Coverage **1,137/1,803 = 63.1%** by regeneration, **0 flips** as predicted (clean 1,137 / todo 519
  identical), self-dating churn reverted; **0 card-def edits of any kind** —
  `git diff main..HEAD --numstat` over `crates/card-defs` and `crates/card-types` is **empty**, so
  the shortcut was available and the regeneration was run anyway.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs) — all against the FINAL tree. **`npm run build` was
  NOT run and that is stated rather than omitted**: it is N/A here, because
  `git diff main..HEAD --numstat -- tools/play-server tools/play-frontend` is **empty** and
  `node_modules` is absent from this worktree. `tools/` is **not** zero, and the first draft of this
  line would have implied it was: `tools/tui/src/dashboard/{parser.rs,data.rs,tabs/dashboard.rs}`
  each move a few lines, for `OOS-DX49-6`.
  **Engine lines**: `git diff --numstat` over `crates/engine/src` is **+253 / −180** tracked, **plus
  the new untracked `rules/saga.rs` (178 lines)** — stated separately, because `--numstat` cannot see
  an untracked file and the tracked figure alone understates the change by a whole module.
  `crates/view-model` and `crates/simulator/src` are both **0**: every consumer of CR 714 lives in
  the engine, which was measured before the design was chosen rather than asserted after.
  **Benches: a REAL ~1.7% regression, published as one rather than as "inside the historical band"**
  — and the first draft of this line said the latter, from a branch-only measurement against a
  remembered figure, which is PB-DX28's re-take MEDIUM that PB-DX45 already repeated once. The
  `/review` ran the A/B this batch had not and measured **~+6% `sba_check` / ~+2.4% `full_turn_4p`**.
  The cause was a `Vec` of every phased-in battlefield permanent materialised before the query, which
  **was never necessary**: one immutable reborrow (`let s: &GameState = state;`) lets the walk and
  the query share the borrow. After that fix, matched-set A/B against merge base `be7f29a5` in an
  isolated worktree — `sba_check` 14.685-14.751 → **14.954-14.989 µs (+1.7%, non-overlapping, REAL)**,
  `priority_cycle_4p` +1.7%, `priority_cycle_6p` +1.9%, `full_turn_4p` and `full_turn_6p` both
  **noise** (intervals overlap), `board_wipe_4p` **−5% (faster)**. The residual is inherent to the
  mandated design — `saga_view` takes an `ObjectId` and re-resolves it, one hash probe per
  battlefield permanent per SBA check — and **threading the caller's object through instead would
  shave it and re-create the drift this batch exists to remove.**
  **Revert matrix: 4 rows executed across two files, all discriminating, 0 UNDISCRIMINATED**, plus
  **seven** executed source-gate defeats in the `/review` fix cycle. Engine R-A (the query stops consulting `abilities_are_blanked`)
  reddens **7 of 10**; R-B (drop the face-down conjunct from `is_saga_permanent`) reddens exactly
  `t2`/`t7`, which is what proves `t7` discriminates a different line rather than riding on R-A.
  Channel R-A reddens `c1`/`c3`/`c4`; channel R-B (site 1 alone re-reads the printed def) reddens
  the same three on their `lore = 3` leg, proving that leg load-bearing independently of site 3.
  **`t8` and `c2` are green under both reverts as STATED CONTROLS, not as gaps** — `t8` is the
  CR 113.7a exclusion, which a correct fix must not break, and `c2` has no blanking at all.
  **One row is honestly UNDISCRIMINATED and it is disclosed in the test file's own module doc**:
  sites 3 and 5 chain on the channel path (`turn_actions.rs` only calls
  `fire_saga_chapter_triggers` for a Saga it just placed a lore counter on), so with site 3 fixed no site-5-only
  revert can redden anything in the channel suite; `primitives::…::t5` is what exercises site 5
  alone.
- **Tests (delta 2026-09-02, PB-DX48)**: **4,900 / 0 / 5** full-workspace on branch
  `scutemob-219` (+27 over the **4,873** baseline, measured on this branch BEFORE any edit and
  reproducing PB-DX47's close pin exactly), `--workspace --no-fail-fast` to a file, **57**
  result-producing targets (56 → 57: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME: 27 additions, 0 removals, 0 leavers, 0 renames** — 12 in the new
  `crates/engine/tests/primitives/pb_dx48_ward_dispatch.rs` (11 shipped, +1 in the `/review` fix
  cycle), 11 in the new
  `crates/engine/tests/core/pb_dx48_announcement_site_roster.rs`, 3 in the new
  `crates/simulator/tests/pb_dx48_ward_channel.rs`, and 1 in
  `crates/engine/tests/primitives/pb_eng2_targets_announced.rs`.
  **"0 leavers" must not be read as "nothing was touched"**: the ENG-2 deviation pin was inverted
  **IN PLACE**, so its test name is unchanged while its assertion's CLAIM is not. Disclosed here
  rather than left to the name set to hide.
  **PROTOCOL 39 / HASH 78 both UNMOVED**, gate-executed and **predicted in writing before any code
  changed** (`43fc20ab`), with the reason stated rather than asserted: `PermanentTargeted` is
  already in the wire closure, so emitting the same variant with the same three fields at more
  sites adds no type, variant or field, and HASH hashes declarations rather than event volume.
  `history_is_append_only` and `frozen_prefix_is_pinned` green on both; no pin edited, no history
  row appended, because none was owed.
  Coverage **1,137/1,803 = 63.1%** by regeneration, **0 flips** as predicted (clean 1,137 / todo
  519 / empty 147 all identical), self-dating churn reverted; **0 card-def edits of any kind**.
  `git diff main..HEAD --numstat` over `crates/card-defs`, `crates/card-types`,
  `crates/view-model`, `crates/simulator/src` and `tools/` is **empty**; the engine diff is 4 files,
  **+267 / −61** — a figure that was published as `+235 / −61` and did not reproduce, **twice**:
  a doc-comment commit and then the `/review`'s MEDIUM-1 engine fix both landed after it was
  taken. PB-DX28's "re-take the measured table" MEDIUM, committed again and caught by this
  batch's own `/review`. **`npm run build` was NOT run and that is stated rather than omitted**: it is
  N/A here, because `tools/` is zero and `node_modules` is absent from this worktree.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs) — all against the FINAL tree.
  **The movement budget the ENG-2 handoff and the v4 memo row both demanded DID come due, on the
  fuzz side only, and it is itemised rather than rounded to "no change".** Identical invocation
  (`--games 20 --seed 1 --max-turns 200`) on the merge base in an isolated `git worktree` with its
  own `CARGO_TARGET_DIR` vs this branch — the two trees differ in exactly one thing, so the delta is
  attributable by construction. HARD **185 / 13 distinct UNMOVED** with both sub-checks
  (`player_consistency` 138, `attachment_validity` 47) and both game lists identical; games
  completed, wins and **avg turns per game (122.3)** all identical. Moved: TRANSIENT
  `no_orphaned_tokens` **273 / 62 → 275 / 63**, `SpellCast` 905 → 902, `LandPlayed` 938 → 937,
  casts-with-announced-targets 890/106 → 887/105, `triggered_targets` decision points 70 → 69, and
  rejections 2677 → 2697 **of which +20 of +20 are in ONE game (seed 12)**. Read as ONE divergence
  with everything else downstream of it. Mechanism stated as an inference with its premise:
  `PermanentTargeted` dispatches only Ward and `PermanentBecomesTarget`, and the census measures the
  latter at **0** deck-legal `Complete` members; the specific card is **UNIDENTIFIED** because the
  fuzzer writes no per-game journal in batch mode — said rather than guessed. **Correction while
  re-measuring**: the v4 memo §2.8's HARD **106** / TRANSIENT **226** for this exact invocation does
  **not** reproduce at the merge base (**185** / **273**); that drift is entirely pre-PB-DX48 and is
  recorded so the next batch re-measures rather than trusting the memo.
  **Revert matrix RE-EXECUTED by the coordinator rather than accepted from the delegated reports,
  and one report was wrong.** R-A (no emission) reddens all 12 engine probes + the new
  `pb_eng2` sibling + all 3 channel probes; R-B (single wave) reddens exactly `t1`/`t5`/`t6` and
  `c1`/`c2`, which is the per-site dispatch map confirmed by execution on two files that share no
  fixture; R-C (a second dispatcher, i.e. the rejected design) reddens the same three via the
  wave-bound `debug_assert!`, proving the **exact-count** assertions load-bearing. **No
  UNDISCRIMINATED row.** The roster file's own 10 constant-mutation rows are all RED.
- **Tests (delta 2026-09-02, PB-DX47)**: **4,873 / 0 / 5** full-workspace on branch
  `scutemob-218` (+12 over the **4,861** baseline, measured on this branch BEFORE any edit and
  reproducing PB-DX45's close pin exactly), `--workspace --no-fail-fast` to a file, **56**
  result-producing targets (55 → 56: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME: 13 additions, 1 leaver, 0 removals** — 9 in the new
  `crates/engine/tests/core/pb_dx47_dispatch_path_roster.rs`, 2 in the new
  `crates/simulator/tests/pb_dx47_double_push_probe.rs`, 1 in the new
  `crates/engine/tests/primitives/pb_dx47_modal_trigger_mode_zero.rs`, and 1 the inversion's
  successor in `crates/engine/tests/primitives/pb_dx24_trigger_zone_and_index_spaces.rs`.
  **The leaver is disclosed rather than netted out and it is not a removal**:
  `test_dx24_when_deals_combat_damage_to_player_reads_the_visible_face_of_a_transformed_attacker`
  became `test_dx47_transformed_attacker_queues_exactly_one_trigger_off_the_visible_face` — same
  file, same Q4 property, subject **inverted**, because what it pinned is what this batch deleted.
  **PROTOCOL 39 / HASH 78 both UNMOVED**, gate-executed (`hash_schema` 36/36, `protocol_schema`
  17/17) and **predicted in writing before any code changed**, with the reason stated rather than
  asserted: a suppression adds no type, variant or field to the wire closure and changes no hashed
  declaration. `history_is_append_only` and `frozen_prefix_is_pinned` green; no pin edited and no
  history row appended, because none was owed.
  Coverage **1,137/1,803 = 63.1%** by regeneration, **0 flips** as predicted (clean 1,137 / todo
  519 / empty 147 all identical), self-dating churn reverted; **0 card-def edits of any kind**, so
  the empty-diff shortcut was available and was checked.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs) — all against the FINAL tree. **`npm run build` was
  NOT run and that is stated rather than omitted** (`node_modules` is absent from this worktree):
  it is N/A here, because `git diff main..HEAD --numstat -- tools/` is **empty** and the engine
  diff is `rules/abilities.rs` alone (**+106 / −81**) — `crates/view-model`,
  `crates/simulator/src`, `crates/card-defs` and `crates/card-types` are all zero.
  **10 revert rows executed, 10 RED, 0 UNDISCRIMINATED**, with three green-under-revert rows
  disclosed as such rather than left implicit — including that disabling the `r3` parser's
  comment-stripping reddens `r3b` and leaves `r3` GREEN, so that stripping is defensive today and
  load-bearing only for `r3b`'s own guarantee (`OOS-DX47-7`).
  **Two of the batch's own published claims were refuted by the batch's own gates and corrected
  in place.** (1) `DECLARING_MEMBERS` was typed from
  `grep -l WhenDealsCombatDamageToPlayer crates/card-defs/src/defs/*.rs` — **30 files** — and the
  `all_cards()` walk returns **26 defs**; the four extras name the variant only inside a `// TODO`
  saying why they cannot use it. That is **SR-36's rule verbatim** (*enumerate `all_cards()` for
  rosters, never grep source*) broken inside the batch whose subject is a false comment, and
  `OOS-CARDS2-7`'s shape a second time (`OOS-DX47-2`). (2) `OOS-DX47-3` was published **twice**
  before it was true, and the second draft is the instructive one. Draft 1 (the engine comment):
  *"ZERO corpus defs pair `modes` with this `TriggerCondition`"* — refuted by `r5b` on first run, the
  population is **one** (`glissa_sunslayer`, `partial`, so deck-legal exposure is zero). Draft 2:
  *"a real capability the fix gives up"* — refuted by execution, and this is the shape the whole
  batch exists to punish, **a consequence inferred from a code shape and published without anyone
  running it**. `primitives::pb_dx47_modal_trigger_mode_zero::t1` measures **+1 life (mode 0,
  once)** at HEAD and **+2** with the scan restored — never +10 or +100. **Nothing modal was ever
  offered on either path**: `flush_sorted` hard-codes `modes_chosen = vec![0]` in both arms for
  every trigger kind, `resolution.rs`'s modal replacement sits outside the `is_carddef_etb` branch,
  and `modal_trigger` (CR 603.3c) is a standing `AutoChosen` row in `core::decision_site_walk`. So
  the pre-fix engine resolved **mode 0 twice**, and `OOS-DX47-3` is re-scoped to the structural gap
  alone with its behavioural delta measured at **zero**.
- **Tests (delta 2026-09-02, PB-DX45)**: **4,861 / 0 / 5** full-workspace on branch
  `scutemob-217` (+26 over the **4,835** baseline, measured on this branch BEFORE any edit and
  reproducing PB-DX15a's close pin exactly), `--workspace --no-fail-fast` to a file, **55**
  result-producing targets (54 → 55: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME: 26 additions, 0 renames, 0 leavers, 0 removals** — 12 in the new
  `crates/engine/tests/primitives/pb_dx45_optional_cost.rs`, 6 in the new
  `crates/engine/tests/core/pb_dx45_may_pay_roster.rs`, 3 in the new
  `crates/simulator/tests/pb_dx45_optional_cost_channel.rs`, 5 in `tools/play-server/src/main.rs`'s
  `#[cfg(test)]` module.
  **↻ The `/review` (7 findings — 1 gate, 3 MEDIUM record/rationale, 3 LOW — all seven taken) found
  a SECOND gate this batch had left silent, and it is the one that would have cost a future batch.**
  `test_dp9_mana_ability_gate` asserts no `Complete` def puts an asking channel inside a mana
  ability (CR 605.4a leaves no room to announce there, so the branch silently applies the default);
  its needle list was never taught the sixth channel, and the comment describing it still said
  FIVE — **the same sentence PB-DX28's own `/review` caught one variant short, filed as
  `OOS-DX28-6`, now one channel short again.** Proved by planting a `MayPayThenEffect` inside a
  `WhenTappedForMana` trigger and watching the gate stay GREEN. Two needles added
  (`MayPayThenEffect`, and `LookAtTopThenPlace` over-wide because the second site is a FIELD, not
  a variant — stated rather than accepted silently), both revert-proven RED.
  Also taken: **the execution notes published two fingerprints that exist nowhere at HEAD** — they
  moved a second time when `Cost` was `Box`ed and the "measured" table was never re-taken, which is
  PB-DX28's MEDIUM verbatim, inside a batch whose headline is three figures that did not
  reproduce; **R2's failure message inverted its own consequence** (`can_pay_optional_cost`'s tail
  returns `false`, so an undecidable cost is a silent no-op, not a harmless over-ask — proved by
  executing `Cost::Tap`); and **six "pay when able" claims left standing in production source**,
  including on `try_pay_optional_cost`'s own doc, on the `MayPayThenEffect` DSL variant a card
  author reads, and on `birthing_ritual` — PB-DX27's *a blocker note is a claim* left un-applied to
  this batch's own subject matter.
  **PROTOCOL 38 → 39 / HASH 77 → 78**, ONE bump each, both **predicted in writing before any code
  changed** — including the prediction that neither closure's type count would move, confirmed at
  **98** and **131** — and both taken from the failing gates' own output, never invented. History
  rows appended never edited; both `FROZEN_HISTORY_PREFIX_DIGEST`s re-pinned; `history_is_append_only`
  and `frozen_prefix_is_pinned` green on both sides; `protocol_schema` 17/17, `hash_schema` 36/36.
  **44 scattered sentinels re-pinned by symbol, then 2 more** — a "re-pin by symbol" is only as
  wide as the spelling the regex matched, and two files spell the assertion across a line break.
  **Both fingerprints were re-taken a SECOND time and the version numbers never moved twice**:
  `clippy::large_enum_variant` fired on this batch's own `PayOptionalCost { cost: Cost }`
  (`Cost::Sacrifice(TargetFilter)` is ~296 bytes), so `Cost` is `Box`ed — `Box<T>` serializes and
  hashes transparently as `T`, so the WIRE shape is unchanged while the DECLARATION text is not.
  Coverage **1,136 → 1,137 / 1,803 = 63.0% → 63.1%**, **one flip, predicted and NAMED before
  regeneration** (`vampire_gourmand` `partial` → `Complete`, from the policy re-adjudication).
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `npm run build` green — all against the FINAL tree.
  **14 revert rows executed, 14 RED, 0 UNDISCRIMINATED** — one of them re-applies this batch's own
  shipped SR-38 defect.
  **Three gates fired on this batch's own work**: PB-DX8's `completeness_deviation_scan` (on the
  new card-def comment; answered by ALLOWLIST with the contract widening stated), UI-5's
  `>Back</button>` label gate (on `ConfirmPicker`'s first-draft markup) and UI-4's picker-error
  ratchet (4 → 5). **And one did NOT fire that should have**, which is the batch's headline —
  see the narrative.
- **Tests (delta 2026-08-23, PB-DX15a + `/review` fix cycle)**: **4,835 / 0 / 5** full-workspace on
  branch `scutemob-216` (+38 over the **4,797** baseline, measured on this branch BEFORE any edit
  and reproducing PB-DX44's close pin exactly), `--workspace --no-fail-fast` to a file, **54**
  result-producing targets (53 → 54: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME: 42 additions, 4 names leaving the passing set, 0 removals** —
  and the four are disclosed individually rather than netted out, because two of them are not
  removals at all. Two are the batch's **inversions**
  (`test_400_7_same_zone_move_produces_new_id` → `..._keeps_the_same_id`;
  `test_dp9_choice_inside_for_each_each_player` → `test_dx15a_each_player_search_asks_in_apnap_order`).
  The other two are **doctests whose name IS their line number** —
  `state::GameState (line 81)` → `(line 91)` and `(line 90) - compile fail` → `(line 100)` — both
  shifted by exactly **+10**, the height of the new `ZoneEnd` declaration. Honest reading: **40
  genuine additions, 2 inversions, 2 line-number shifts.**
  **PROTOCOL 38 / HASH 77 both UNMOVED**, gate-executed (`hash_schema` 36/36, `protocol_schema`
  17/17) and **predicted in writing before any code changed**; the stop-condition never fired and
  no pin was edited. `history_is_append_only` and `frozen_prefix_is_pinned` both green.
  **The moved-pin list is EMPTY, and that is reported as a paid-and-unclaimed budget rather than
  dropped**: the plan budgeted golden-script, SR-9b per-step fingerprint and
  `timestamp_counter`-seeded movement, and none came due. The measured reason is the batch's own
  headline — every multi-seat fixture in the tree sets `active_player` to the LOWEST `PlayerId`,
  so APNAP and ascending `PlayerId` are the same list in all of them, and the same-zone class had
  **no behavioural coverage at all**.
  Coverage **1,136/1,803 = 63.0%** by regeneration, **0 flips** as predicted (clean 1,136 / todo
  520 / empty 147 all identical), self-dating churn reverted. **1 card-def edit, comment-only**
  (`nether_traitor.rs`, in the `/review` fix cycle — its in-source note cited an enforcement that
  the rider had made conditional; the first draft of this line said **0** and is corrected here
  rather than left to be caught at collect).
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs).
  **Four gates fired on this batch's own work and all four were right**: SR-25's
  `bare_lookup_ratchet` (the new helper's bare `.zones.get_mut(..)`), PB-DX7's
  `unordered_iteration_ratchet` (the rider's first draft, 11 → 15 `HashSet`s — answered by
  converting to `BTreeSet` and **lowering** the ceiling 11 → 6 rather than raising it), the
  batch's own scry non-vacuity floor (which caught a fixture push-order convention error in its
  first draft), and its own `r5` roster row (which found that `move_object_to_zone` mints
  **three** ids, not one). **One probe passed VACUOUSLY before it passed honestly** — a bare
  `execute_effect` on `Effect::SearchLibrary` measures nothing, because PB-DP9 rolls the whole
  resolution back until the choice is answered.
- **Tests (delta 2026-08-15, PB-DX44 + `/review` fix cycle)**: **4,797 / 0 / 5** full-workspace on
  branch `scutemob-215` (+44 over the **4,753** baseline, measured on this branch BEFORE any edit
  and reproducing PB-DX43's close pin exactly), `--workspace --no-fail-fast` to a file, **53**
  result-producing targets (50 → 53: three new test binaries), residual list empty.
  **Delta itemised by test NAME**, by set-diffing the two run logs: **45 additions, 1 RENAME,
  0 removals** — 10 in the new `crates/engine/tests/core/pb_dx44_uncastable_roster.rs` (r1-r9 +
  `t_census_report`), 8 in the new `crates/engine/tests/rules/pb_dx44_split_half_cast.rs`, 7 in the
  new `crates/simulator/tests/pb_dx44_pitch_channel.rs` (T1-T7), 6 in the new
  `crates/simulator/tests/pb_dx44_spree_mode_costs.rs`, 4 in the new
  `crates/engine/tests/rules/pb_dx44_fuse_targets.rs`, 4 in `tools/play-server/src/main.rs`'s
  `#[cfg(test)]` module, 3 in the new `crates/simulator/tests/pb_dx44_split_half_channel.rs`, and 1
  the rename's successor. **The rename is disclosed rather than netted out**:
  `p1e_fuse_is_suppressed_while_its_right_half_targets_cannot_be_announced` became
  `p1e_fuse_is_offered_and_its_target_count_matches_what_the_cast_validates` — same file, subject
  **inverted**, because the suppression it pinned is what this batch deleted. "+44 with zero
  removals" would have been a true number hiding a real edit.
  **PROTOCOL 37 → 38 / HASH 76 → 77**, both taken from the failing gates' own output and both
  **predicted in writing before any code changed**; the stop-condition (a gate moving in a way the
  half selector does not explain, or not moving at all) never fired. History rows appended, never
  edited; frozen-prefix digests re-pinned; `history_is_append_only` and `frozen_prefix_is_pinned`
  green. ONE wire bump for the whole PB — stages 1 and 2b each gate-verified unmoved.
  Coverage **1,136/1,803 = 63.0%** by regeneration, **0 flips** as predicted, self-dating churn
  reverted. `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs).
- **Tests (delta 2026-08-14, PB-DX43 + `/review` fix cycle)**: **4,753 / 0 / 5** full-workspace on
  branch `scutemob-213` (+32 over the **4,721** baseline, measured on this branch BEFORE any edit
  and reproducing PB-DX29's close pin exactly), `--workspace --no-fail-fast` to a file, **50**
  result-producing targets (49 → 50: one new simulator test binary), residual list empty.
  **Delta itemised by test NAME with ZERO removals**, by set-diffing the two run logs: **16** in
  the new `crates/engine/tests/rules/pb_dx43_intrinsic_land_mana.rs` (P1-P13 + the fix cycle's
  F1-F3), **8** in the new `crates/simulator/tests/pb_dx43_intrinsic_mana_channel.rs` (C1-C6b, the
  real-activation-path probes), **6** in the new
  `crates/engine/tests/core/pb_dx43_land_type_roster.rs` (R1-R5 + the fix cycle's
  fingerprint-vs-declaration gate), and **2** in `crates/card-types`' new
  `state::types::basic_land_types_tests`.
  **PROTOCOL 37 / HASH 76 both UNMOVED**, gate-executed (`hash_schema` 36/36, `protocol_schema`
  17/17) — the derivation adds no type, no variant and no field, and `hash.rs` hashes **base**
  `obj.characteristics` rather than the resolved value, which is the same reason `AddManaAbility`
  grants have never moved a state hash. Predicted in the plan (D7) before any code change and
  gate-confirmed after; **no sentinel re-pin and no history row were owed**, and
  `history_is_append_only` / `frozen_prefix_is_pinned` pass unchanged on both gates.
  Coverage **1,136/1,803 = 63.0%** by regeneration, **0 flips** as predicted (clean 1,136 / todo
  520 / empty 147 all identical), self-dating churn reverted.
  **Engine lines**: `git diff --numstat` gives `rules/layers.rs` **+174 / −2**,
  `card-types/src/state/types.rs` **+75 / −0**, `blood_moon.rs` **+33 / −42**,
  `magus_of_the_moon.rs` **+25 / −38**, `tools/play-server/src/main.rs` **+15 / −1** (the
  `UI3_SPLIT_COMBAT_SEED` re-observation, inside `#[cfg(test)]`). **`crates/view-model` and
  `crates/simulator/src` are both 0** — the derived ability is reachable through every consumer
  with no production line outside the engine, because each one already read layer-resolved
  characteristics; that was measured before the design was chosen, not asserted after.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs). **17 revert rows executed** across three matrices;
  **1 honestly UNDISCRIMINATED** (`p8`, CR 708.2a face-down — no line this batch owns can break it,
  because the face-down blank runs before the layer loop starts), disclosed in the test itself and
  not only in `memory/`. Benches within the historical band (`/review` Issue 11, which noted the
  change adds work to the hot layer walk): `full_turn_4p` **213.7-215.8 µs**, `priority_cycle_4p`
  **23.8-24.0 µs**, `sba_check` **14.5 µs** on one run. **Stated as one run, not as a delta** —
  the re-review measured the same commit at `full_turn_4p` **220.5-223.1 µs** against its own
  pre-fix **218.4-220.4 µs**, i.e. within noise. Both sets sit inside the historical band, so the
  supportable claim is *no regression*; the per-call `SubType` allocation the reviewer flagged is
  genuinely gone (the derivation now compares interned strings) but **the improvement is not
  demonstrated by measurement** and is not claimed.
- **Tests (delta 2026-08-14, PB-DX29 + `/review` fix cycle)**: **4,721 / 0 / 5** full-workspace on
  branch `scutemob-211` (+87 over the **4,634** baseline, which was measured on this branch BEFORE
  any edit and reproduced PB-DX28's close pin exactly), `--workspace --no-fail-fast` to a file,
  **49** result-producing targets (46 → 49: three new test binaries), residual list empty.
  **Delta itemised by test NAME with ZERO removals**, by set-diffing the two run logs: **29** in
  the new `crates/simulator/tests/pb_dx29_cost_kind_surface.rs` (P/C/E groups), **24** in
  `tools/play-server/src/main.rs`'s `#[cfg(test)]` module (14 unit tests of the 400 boundary, 2
  wire-shape pins, 2 full HTTP drives, 2 frontend source gates, 1 inverted deviation pin, 3 from
  the fix cycle), **11** in the new
  `crates/engine/tests/primitives/pb_dx29_loyalty_target_surface.rs`, **8** in the new
  `crates/simulator/tests/pb_dx29_loyalty_channel.rs`, **8** in the new
  `crates/engine/tests/core/pb_dx29_additional_cost_roster.rs` (R1-R7 + R2m), **4** in `view.rs`'s
  new `format_mana_cost_compact_tests` (the function had **none** despite five call sites), and
  **3** in the new `crates/simulator/tests/pb_dx29_mutate_on_top.rs`.
  **PROTOCOL 37 / HASH 76 both UNMOVED**, gate-executed (`hash_schema` 36/36, `protocol_schema`
  17/17) — nothing added is a type in the `Command`/`GameEvent`/`Effect`/`Characteristics`
  closure, and `LegalAction::CastWithMutate` gaining a field is not a wire change because
  `LegalAction` is a simulator type.
  Coverage **1,136/1,803 = 63.0%** by regeneration, **0 flips** as predicted, self-dating churn
  reverted — proven by regeneration rather than by an empty card-defs diff, because three defs
  were edited and the shortcut was unavailable.
  **Engine lines are NOT zero and the brief predicted zero** — `git diff --numstat` over
  `crates/engine/src` + `crates/card-types/src` is **+218 / −12**, of which **138** are the new
  read-only query surface (`rules/queries.rs`' three functions + their re-export), **76** are
  registry *declarations* that SR-5's keyword gate and its ability-definition sibling refused to
  let the batch omit, and **4** are one comment in `engine.rs` disambiguating a renumbered seed
  ID. Zero behaviour-changing engine lines anywhere; `crates/view-model` is **0**.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs), `npm run build` green on the frontend.
  **`/review`: 2 HIGH / 6 MEDIUM / 11 LOW, all 19 taken** — the reviewer had a shell, reproduced
  every figure above independently (its own test-NAME set was byte-identical), and found no second
  divergence in the two mirrors this batch flagged hardest.
- **Tests (delta 2026-08-14, PB-DX28 + fix cycle)**: **4,634 / 0 / 5** full-workspace on branch
  `scutemob-210` (+29 over the **4,605** baseline, which was **re-measured at `c5b9e459` in a
  scratch worktree** after a mid-batch reboot destroyed the original log — it reproduced the
  pre-reboot figure exactly, so the number is measured twice rather than remembered),
  `--workspace --no-fail-fast` to a file, 46 result-producing targets, residual list empty.
  **Delta itemised by test NAME with zero removals**, by set-diffing the two run logs: 12 in the
  new `crates/engine/tests/primitives/pb_dx28_owner_axis.rs`, 11 in the new
  `crates/engine/tests/primitives/pb_dx28_untargeted_choice.rs`, and 6 in the new
  `crates/engine/tests/core/pb_dx28_chosen_object_roster.rs`. **The two names that left the
  PASSING set mid-batch were not removals** — `hash_schema` and `protocol_schema` moved from pass
  to fail and back, deliberately, so the wire moved once and the bump could be read off.
  **PROTOCOL 36 → 37 / HASH 75 → 76**, both taken from the failing gates' own output
  (`hash_schema` 36/36, `protocol_schema` 17/17 after the bump). The PROTOCOL closure moves
  **96 → 98** types — its first count change since v31 — because `ChoiceZone` and `TargetOwner`
  are genuinely new members; `TargetFilter.owner` rides in on a struct reachable since v14, which
  is the half a wire prediction from the engine types alone would have missed.
  Coverage **1,136/1,803 = 63.0%** by regeneration, **0 flips** as predicted, self-dating churn
  reverted. `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs).
- **Tests (delta 2026-08-13, PB-DX27 + fix cycle)**: **4,605 / 0 / 5** full-workspace on branch
  `scutemob-209` (+44 over the **4,561** baseline measured on this branch BEFORE any edit),
  `--workspace --no-fail-fast` to a file, 46 result-producing targets, residual list empty.
  **Delta itemised by test NAME with zero removals**, by set-diffing the two run logs: 9 in the
  new `crates/engine/tests/rules/pb_dx27_blood_moon_type_scope.rs`, 9 in the new
  `crates/engine/tests/primitives/pb_dx27_headline_defs.rs`, 7 in the new
  `crates/engine/tests/core/pb_dx27_stale_blocker_notes.rs`, 9 in the new
  `crates/engine/tests/primitives/pb_dx27_stale_blocker_repairs.rs`, and 10 in the new
  `crates/engine/tests/primitives/pb_dx27_sweep_repairs_b.rs`.
  **PROTOCOL 35 → 36 / HASH 74 → 75**, both taken from the gates' own output
  (`hash_schema` 36/36, `protocol_schema` 17/17) — **the brief predicted "expected wire impact
  NONE" and the gate refuted it**: `ContinuousEffectDef.modification` is a sibling of
  `filter`/`duration`, both already in the `Command`/`GameEvent` closure, so
  `LayerModification` is on the wire. Coverage **1,136/1,803 = 63.0%** by regeneration; flips
  named — UP `chord_of_calling`, `reconnaissance`, `wight_of_the_reliquary`,
  `chandra_flamecaller`; DOWN `qarsi_sadist` and `green_suns_zenith`.
  `clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean,
  `tools/check-defs-fmt.sh` clean (1,803 defs).
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
  **↻ 2026-08-14**: the live queue is now `memory/primitives/seed-rerank-2026-08-14.md` §4 (v4,
  `scutemob-212`); v3's §4 joins v2's as SUPERSEDED, §1-§3 of both still canonical.

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
- **Last Updated**: 2026-09-05 — **PB-DX56 SHIPPED** (`scutemob-235`; v4 queue rank 20, task 2
  of 5 of the SECOND chain — **OOS-FB1-1** *(the stated prerequisite)*, **OOS-DX32-1** and
  **OOS-DX22-8** ALL CLOSED, plus the rider **OOS-DP9-19(b)**).
  **THE FUZZER'S HARD BUCKET GOES 291 → 0 ON THE STANDARD INVOCATION**, so `--stop-on-error` no
  longer halts on an undiagnosed class.
  **BOTH filed figures were re-measured FIRST and NEITHER reproduces, both UP** — for the third
  time each: `player_consistency` 84 → **189** across 4 → **11** of 20 games,
  `attachment_validity` 22 → **102** across 3 → **7**. The row's *"79.2% of the HARD bucket"* is
  now **64.9%**, because the other class grew faster.
  **`player_consistency` IS NOT ONE CLASS — IT IS TWO ARMS THE CR GIVES OPPOSITE DISPOSITIONS,
  AND THE REGISTRY ROW, THE v4 MEMO CELL AND THE DISPATCH CRITERION ALL TREAT THEM AS ONE.**
  **189 of 189 reports are the ACTIVE-PLAYER arm; the priority-holder arm produced ZERO.** The
  active arm asserts what **CR 800.4j** *permits* — *"that turn continues to its completion
  **without an active player**"* — and `TurnState::active_player` is a bare `PlayerId`, not an
  `Option`, with exactly ONE production write site, so *"without an active player"* is
  **inexpressible in this engine's state type** and it necessarily encodes that turn by leaving
  the departed id in the field. Everything CR 800.4j actually requires is discharged elsewhere:
  `grant_priority_to_active_player` routes past a dead active player **citing 800.4j by name**,
  and `validate_player_active` rejects every command from a departed seat. **CR 800.4a's last
  sentence, by contrast, is unconditional**, so the priority arm is a real defect and stays HARD.
  **The strictly stronger property is a TURN-BOUNDARY rule, not an end-state one, and the shape
  matters**: CR 800.4k bounds the condition to the remainder of that turn, and an end-state check
  would fire spuriously because at game end the player who just died may legitimately still be
  `active_player`.
  **TWO holes made that bound lucky rather than true and both are FIXED**: `advance_turn`'s
  EXTRA-TURN branch applied **no liveness filter at all** and nothing ever pruned the queue, so a
  departed player's queued extra turn BEGAN (CR 800.4k-wrong — the one UNBOUNDED route); and
  `enter_step`'s cleanup-SBA-round grant was unconditional, the single live route for the priority
  arm, closing `OOS-DP9-19(b)` as a rider by **finishing the wiring of a helper whose own doc named
  that site as the one unrouted hole**.
  **`OOS-DX22-8`'s answer is neither of the two the row told its successor to check first: the
  check was pointed at the direction of a two-directional relation that HEALS.**
  `move_object_to_zone` and its bottom-of-library sibling retire an object performing exactly TWO
  cross-object fix-ups — CR 702.95e soulbond and the replacement-effect GC — and touch the
  attachment relation in **NEITHER direction**. Direction A (a HOST leaves) IS cleared by
  CR 704.5m / CR 704.5n and survives a checkpoint only because the engine sweeps SBAs at nine
  sites and `rules/{abilities,casting,combat,mana,turn_actions}.rs` contain **zero** —
  `OOS-M11-7`'s shape one field over. **The ENGINE DEFECT is direction B, which had no check
  anywhere and never heals**: when an ATTACHER leaves by any route other than the six that clean
  up, the host keeps the dead `ObjectId` in `attachments` for the rest of the game — a **HASHED**
  field, so it perturbs `public_state_hash` **and** `compute_mandatory_state_hash` (CR 104.4b
  loop detection), is read by the equipped-creature trigger family (**CR 301.5f**, not CR 510.3a),
  is walked by CR 702.26g/h phasing through `expect_object_mut` (a latent debug-build panic), and
  is rendered to the browser.
  **THE TRANSIENT-VS-AT-REST QUESTION IS SETTLED BY AN ARITHMETIC THE CENSUS WROTE DOWN FIRST.**
  Revert row R-E measures direction B at **10,290 raw / 7 distinct across 5 of 20 games**
  (~**1,470** checkpoints per condition) against direction A's 102 / ~13 (~**8**) — two orders of
  magnitude, same run, same stateless per-command checker. With F1 in, it is **0**.
  **CR 400.7f is what makes the fix's one-directionality load-bearing** rather than merely
  conservative: it exists so a leaves-the-battlefield trigger can find an Aura in its owner's
  graveyard *"as a result of being put there as a state-based action for not being attached to a
  permanent. (See rule 704.5m.)"* — a rule whose antecedent is that the Aura got there THROUGH
  704.5m. Pinned wrong-way-round so a later batch cannot "finish the job".
  **THE TOOLING CAUGHT A DEFECT IN ITS OWN CONSUMER ON ITS FIRST RUN AND IT WAS THIS BATCH'S
  OWN.** After the disposition, HARD read **1**, not 0 — and reading the artefact's EVIDENCE
  rather than its count showed the promotion was a false positive: it keyed the departed seat on
  `player=`, which the prepended state context emits **per seat**, so it collapsed
  `PlayerId(4)`'s turn-154 report against `PlayerId(1)`'s turn-133 one and reported
  *"turns_crossed=21"* for two DIFFERENT seats. **That is `OOS-FB1-1`'s entire argument
  instantiated on the batch that closed it** (`OOS-DX56-1`): the count said *"one hard violation,
  diagnosed"*; the evidence said *"your own key is wrong"*.
  **ELEVEN GATES ON THIS BATCH WERE DEFEATED BY EXECUTION AND ALL ELEVEN ARE NOW RED** — 8 by the
  coordinator's bypass pass (two whose probes called a PRIVATE function directly so nothing
  asserted `check_all` dispatched to it; an end-state check with **no call-site gate whose hole is
  INHERITED from PB-DX32**; a promotion with **no test of any kind**) and 3 more by the `/review`
  (a **commented-out call** satisfies a `contains`-based source gate — PB-DX8's `OOS-DX32-6`, not
  carried across; an **argument swap** in the promotion call that compiles and uses every binding;
  and ORing two HARD class names into the transient test, which silently disarms
  `--stop-on-error` — **PB-DX50's `r3` finding verbatim, that a gate on a predicate's DEFINITION
  says nothing about its CONSUMER**). The classification now lives in one free function a unit
  test can drive, and `record_violations` is a delegation with a gate saying so.
  **The `/review` found 15 and all 15 were taken.** Its other keepers: the new `evidence` was
  printed **NOWHERE on any real invocation**, because the disposition made both firing classes
  transient and only the HARD bucket was dumped (`OOS-DX56-8`); the new end-state check reported a
  **CR-legitimate** state, a phased-out attacher, which CR 702.26b and CR 702.26i exempt and which
  measured 0/20 only because phasing is rare (`OOS-DX56-9`); and the module doc's own check count
  was wrong **inside the paragraph telling you to count it** — `check_all` makes **eleven** calls,
  and this batch took `main`'s already-wrong nine and added one instead of counting
  (`OOS-DX56-13`).
  Tests **5,316 / 0 / 5** (+29, 72 targets, byte-exact NAME set difference, 0 leavers, duplicate
  scan EMPTY, re-taken AFTER the fix cycle). **HASH 85 / PROTOCOL 44 both UNMOVED — zero bumps**,
  predicted per half before any production line, counterfactual executed. Coverage **UNMOVED at
  1,140/1,803 = 63.2%**, 0 flips, **0 card-def edits**. Filed **OOS-DX56-1..15**. Full record:
  `memory/primitives/pb-DX56-execution-notes.md`; census:
  `memory/primitives/pb-DX56-mechanism-census.md`; bypass rows:
  `memory/primitives/pb-DX56-bypass-attempts.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-05 — **PB-DX55 SHIPPED** (`scutemob-234`; v4 queue rank 19, task 1
  of 5 of the SECOND user-approved chain — **OOS-SIM6-3**, **OOS-SIM5-3** and **OOS-SIM5-5** ALL
  **FILED** *and* **CLOSED**, plus the rider **OOS-DX51-3** CLOSED. **None of the three had a
  registry row**, for the second batch running: all three lived only inside `OOS-DX32-9`'s
  ranked-defect-list cell, which is the v4 memo's own 61-of-208 blind spot instantiated on three
  seeds at once.)
  **THE WHOLE BOT REFUSAL SURFACE GOES 70 → 9, AND EVERY SURVIVOR IS ONE CLASS.**
  **The memo's §2.6 table does not reproduce, in three directions at once**, measured before any
  edit by its own invocation: the total is **70, not 105**; `OOS-SIM6-3` is **18, not 76** (its
  share falls 72.4% → 25.7%, so it is not even the largest class, and §2.6's own correction of
  the filed figure to *"76 of 105 — larger in share and in count"* is stale in the OTHER
  direction); `OOS-SIM5-5` is **22, not 2** — **grown 11× and now the LARGEST class**, because
  PB-DX35 made the per-mode requirement enforceable on the validation axis and left the query
  that would let a caller satisfy it unchanged; and **"residue zero" is REFUTED**, along with
  §2.6's separate claim that *"cast-side refusals are zero of any kind"* — the nine survivors are
  all `expected 1..=1 target(s) but got 0` and **three are cast-side**. That residue is
  `OOS-SIM5-4`'s parked offer-suppression class, priced by §2.6 at **0 of 105** and worth
  **9 of 9** now that everything else is shut (`OOS-DX55-1`).
  **EACH HALF IS ONE ARITHMETIC, NOT A SECOND COPY, AND THE COMPILER ENFORCES IT WHERE IT CAN.**
  `legal_actions::command_mana_cost` is an **exhaustive `match` over all 45 `Command` variants
  with NO wildcard arm**, so the mana-bearing census is a CEILING the compiler holds rather than a
  document: 9 arms return a cost, **21** charge no mana, 14 charge mana but no `LegalAction` can
  produce them, and `AnswerEffectChoice` is `None` for a stated reason rather than by omission.
  `auto_tap_commands_for` collapses to that call plus `solve_mana_payment_with_pool` — **the same
  two calls `can_afford` makes**, so SR-38 holds by construction instead of by two functions that
  happen to agree.
  **THE ENGINE ALREADY HELD TWO HAND-ROLLED BLOCK PREDICATES INSIDE ONE FUNCTION, AND THEY
  DIFFERED IN THREE GUARDS.** `handle_declare_blockers`' per-pair loop and its CR 702.39a provoke
  requirement's `continue`-shaped mirror are the same ~19 checks written twice; the mirror omitted
  **phased-out**, **`CrossPlayerBlock`** and the **duplicate** check, so a phased-out or
  cross-player-attacked provoked creature was REQUIRED to block a block it could not make — an
  impossible requirement raised as a refusal instead of skipped, which is the opposite of what
  CR 509.1c says. So *"never a second hand-rolled copy"* described HEAD rather than a future risk.
  Both collapse into `check_block_pair`, consumed by the handler, by its own mirror and by the
  offer through `queries::legal_blocks`, and **`combat.rs` is +342 / −473 — a NET REDUCTION of
  131 lines.** The offer needed a shape change to consume it at all: `LegalAction::DeclareBlockers`
  was a flat cross product that DISCARDED each attacker's `AttackTarget`, so no per-attacker
  predicate was expressible in it.
  **AND `handle_activate_ability` HELD A FIFTH INLINE COPY of `casting::per_mode_target_
  requirements`' body** — same `debug_assert_eq!`, same `flat_map`/`get`/`unwrap_or_default` —
  which PB-DX35 left behind when it unified the trigger side. Deleted; the query, the cast path,
  `trigger_modal_plan` and the handler now share one slicer.
  **THREE THINGS FOUND ONLY BY EXECUTION.** (1) Karn's Bastion carries `{T}: Add {C}` beside
  `{4}, {T}: Proliferate`, so a naive auto-tapper taps the permanent to fund its OWN `{T}` cost
  and the engine refuses `PermanentAlreadyTapped`; closed by an exclusion applied at BOTH the
  offer layer and the funding layer, and it appears in no plan. (2) Umezawa's Jitte's modal
  ability is at layer-resolved `ability_index` **0**, not the **1** the dispatch brief and the
  plan both claimed — `enrich_spec_from_def` lowers in declaration order and the modal ability
  precedes Equip. (3) The `pay: true` payment offers were gated on the **pool alone** and
  therefore UNDER-offered, a dual nobody had named; the block comment saying *"the engine's
  payment path reads only the pool (it never auto-taps)"* is what this batch makes false.
  **TWO CR CITES CORRECTED AGAINST THE RULES SERVER BEFORE ANY CODE, ONE OF THEM THE PLAN'S
  OWN.** **CR 700.2a** governs a modal *spell or **ACTIVATED** ability*; **CR 700.2b** governs a
  modal **TRIGGERED** ability and adds *"if no mode is chosen, the ability is removed from the
  stack"*. The plan's first draft cited PB-DX35's rule, and the distinction is load-bearing: an
  activated ability is never on a stack to be removed from, so the consequence is an SR-38 OFFER
  SUPPRESSION, not a removal. And **`combat.rs:1271` cited CR 509.1c** for *"the attacker must be
  attacking the declaring player"*; CR 509.1c is the requirements-maximisation rule, correctly
  cited three lines over for provoke, and the rule that says it is **CR 509.1a**, verbatim.
  **THE BROWSER HALF IS PROVEN IN THE CHANNEL THE SEED WAS WRITTEN ABOUT.** `OOS-SIM6-3` says a
  browser human gets a **422**; a 422 is an HTTP fact, so a real `POST /api/game/action` drive
  answers it or nothing does. With an EMPTY pool, untapped lands and **no `TapForMana` ever
  submitted** (asserted by counting the posted kinds, not merely omitted), the activation is
  accepted and the verdict is the **CR 702.6a attachment**, not the status code. Under revert it
  reproduces the seed's own sentence verbatim: `422 "player does not have enough mana to pay the
  cost"`. **Its first draft failed correctly**, because an activated ability uses the stack and
  acceptance is not resolution.
  Tests **5,287 / 0 / 5** (+44 over a **5,243** baseline reproducing PB-DX42b's close pin
  exactly, **72** targets, byte-exact NAME set difference with a non-end-anchored regex: 43
  additions / 0 leavers / 0 removals / 0 renames, duplicate-name scan EMPTY on both runs; re-taken
  AFTER the fix cycle).
  **HASH 85 / PROTOCOL 44 BOTH UNMOVED — zero bumps**, predicted per half before any production
  line, closure counts MEASURED at **132 / 98**, and **the counterfactual EXECUTED**: every type
  the new query surfaces traffic in already fails both gates' `CLOSURE_MUST_NOT_CONTAIN`, which
  is why nothing moved. Coverage **UNMOVED at 1,140/1,803 = 63.2%**, 0 flips, **0 card-def
  edits**. **Fuzz: the gate config's rejection rate is 1.843‰ → 0**, and the zero is pinned where
  it happened rather than lost to the seed move it forced. **Benches NOT measured**, bounded by
  execution. **`npm run build` NOT run and it is a GAP this time, not an N/A** (`OOS-DX55-7`).
  **Revert matrix 13 rows, coordinator-executed, with a CONTROL row** — R1/R2 precise
  complements, R4's zero settled as structural redundancy by the R4b/R5 pair, **R9 a gate defeat
  that SUCCEEDED** (`OOS-DX55-3`), **and the harness itself wrong first** (`shutil.copy2`
  preserves mtimes, so cargo did not rebuild and every row after the first measured the previous
  row's binary — `OOS-DX55-4`). Filed **OOS-DX55-1..10** (`-9` and `-10` by the `/review` fix cycle — dispatch hygiene 8's
  exact case, caught by re-checking this cell against the registry AFTER the cycle). Full record:
  `memory/primitives/pb-DX55-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-05 — **PB-DX42b SHIPPED** (`scutemob-233`; v4 queue rank 18 —
  **OOS-ADJ-1** ≡ **OOS-DX19-2** FILED *and* CLOSED as ONE defect, plus **OOS-DX19-1**'s residue
  and **OOS-DX19-4** CLOSED BY CONSTRUCTION, and the rank-21 rider **OOS-ADJ-2** taken in both
  halves). **Ranks 1-18 are ALL SHIPPED and no further dispatch is authorised.**
  **A rule the engine could not obey, because the only thing it could ask was "am I inside the
  layer system".** CR 613.1d decides a Layer-6 effect's condition against characteristics resolved
  through Layer 4, because Layer 4 has already run. `characteristics_for_condition` returned
  **printed** `obj.characteristics` for **any** condition evaluated inside a
  `calculate_characteristics` walk, because the only state available to it was an ambient
  `thread_local!` depth counter. **A depth counter suppresses the ENTIRE layer system where an
  `EffectId` set suppresses the one self-referential effect — and that difference IS the seven
  live-wrong pairs.** It is also why *"two distinct conditional effects nest without mutual
  suppression"* is unwritable against a depth counter, which is what made it the discriminating
  probe.
  **THE HEADLINE SEED HAD NO REGISTRY ROW, AND FIVE OF ITS SIBLINGS STILL DO NOT.** The
  adjudication (`scutemob-186`) filed **seven** `OOS-ADJ-*` seeds in its own §6 under the heading
  *"Filed here for the collector to register in the canonical registry … This task does not write
  that file."* **Six of the seven were never registered.** Only `OOS-ADJ-7` has a row, and PB-DX27
  wrote it a fortnight later. So the headline seed of a **rank-18 queue entry** — the one the v4
  memo, the queue banner and this batch's own dispatch criterion all name — lived five weeks in a
  document dispatch hygiene 5 does not treat as ground truth. That is the **61-of-208 registry
  blind spot the v4 re-rank measured**, instantiated on its own highest-ranked unshipped row.
  `OOS-ADJ-1` and `OOS-ADJ-2` now have rows; `-3`, `-4`, `-5`, `-6` still do not.
  **THE SUPPLY CENSUS REPRODUCES EXACTLY AND IS NOW A CEILING AS WELL AS A FLOOR.** Seven
  deck-legal `Complete` pairs — `indomitable_archangel` × { `blinkmoth_nexus`, `inkmoth_nexus`,
  `darksteel_mutation` } under-counting and × { `eaten_by_piranhas`, `kenriths_transformation`,
  `imprisoned_in_the_moon`, `thaumatic_compass` } over-counting — re-derived from serialized
  payloads without consulting the adjudication's list. **The ceiling comes from enumerating the
  five `LayerModification` arms that write `chars.card_types` at the APPLY SITE** (`Copy` /
  `SetTypeLine` / `AddCardTypes` / `RemoveCardTypes` / `SetCardTypes`) rather than the four
  remembered variant names: `SetLandTypes` is correctly absent (PB-DX27's `OOS-ADJ-7` repair) and
  `Copy` has zero corpus supply. The CR 708.2a face-down class is genuinely unbounded and is
  **stated, not tallied** — six batches have taught this queue that a yield cell is a floor, and
  this is the first in a while that is both.
  **THE ROW WORTH READING IS R3, AND IT CAME BACK GREEN.** Deleting the activity sweep's layer
  bound — the adjudication's OWN §3.2(iii) load-bearing precondition, the one it says is *"stated
  here because it is stated nowhere else"* — reddens **NOTHING** in the workspace. That is
  structural rather than a missing test: **a later-layer effect cannot change an earlier layer's
  output**, which is the very fact that makes bounding semantically free, and the `in_flight`
  backstop absorbs the rest. So no assertion on characteristics can separate the two designs, and
  it was settled by a **complementary pair** instead — **the first time §3.2(iii)'s claim has been
  executed rather than argued**: sweep bound PRESENT + backstop REMOVED runs **23/23 green**
  (termination IS by construction), and sweep bound REMOVED + backstop REMOVED **aborts with
  `fatal runtime error: stack overflow` (SIGABRT)**, which is `OOS-SIM2-6`'s original crash. So the
  labelled cycle-breaker is genuinely **UNREACHABLE**, not merely unused — and it still ships
  LABELLED, with a wrong-way-round pin, because the CR is silent on condition-evaluation cycles.
  **A GATE THIS BATCH WOULD HAVE DEFEATED WITH A RENAME, PROVEN BY EXECUTION.**
  `no_condition_evaluator_resolves_characteristics_directly` scans the bodies of
  `pub fn check_condition` and `pub fn check_static_condition`. After the refactor both are
  three-line wrappers and the real evaluators are `check_condition_ctx` /
  `check_static_condition_ctx`, so the gate would have gone **vacuously green**. Planting an
  `expect_characteristics` inside `check_static_condition_ctx` — a literal re-opening of
  `OOS-SIM2-6` — leaves the **pre-batch gate shape completely GREEN** and reddens the re-keyed one.
  Re-keyed onto all four bodies with a per-function non-vacuity floor on body SIZE.
  **A DEFECT THE DELEGATED WORK SHIPPED, REPRODUCED BY EXECUTION BEFORE IT WAS WRITTEN DOWN.** The
  new CR 613.1d `debug_assert` computed its required layer with `.unwrap_or(EffectLayer::Copy)`,
  which collapses *"this condition reads no characteristic at all"* into *"this condition needs
  Layer 1"*. `Copy` is the FIRST layer, so `required < effect.layer` is false for **every**
  Layer-1 effect: an `EffectLayer::Copy` effect carrying `Condition::IsYourTurn` panicked the debug
  build with a message asserting it required characteristics it does not require. Zero corpus
  exposure, and that is not why it matters. `OOS-DX42b-1`; the generalisation is that `None` and
  `Some(MINIMUM)` are different claims and `unwrap_or(MINIMUM)` silently converts the first into
  the second wherever the consumer's test is a strict inequality.
  **TWO CR CITES CORRECTED AGAINST THE RULES SERVER, ONE OF THEM ON THIS BATCH'S OWN HEADLINE
  CARD.** `indomitable_archangel.rs` cited **CR 702.45a** for Metalcraft from the day it was
  authored, and **CR 702.45a is BUSHIDO**; Metalcraft has no CR 702.x entry at all, because
  CR 207.2c names it in its own list of ability words and says ability words have *"no individual
  entries in the Comprehensive Rules"* (`OOS-DX42b-2`). And **`CR 613.8a(a)` is not a rule NUMBER**
  — 613.8a is a single rule with an internal (a)/(b)/(c) list, so a reader who greps for it finds
  nothing; the CLAIM is true, the form is not, and it is inherited from the adjudication into eight
  documents (`OOS-DX42b-3`, filed to ride PB-DX38 rather than swept here).
  **BENCHES: A REAL REGRESSION, FOUND AND REMOVED RATHER THAN PUBLISHED.** The first A/B measured
  `sba_check` **+6.76% with non-overlapping intervals** against a 0.32% same-code band. Cause 1: a
  **heap allocation on every `calculate_characteristics` call** (the bounded layer list built with
  `.filter(..).collect::<Vec<_>>()` where the pre-batch code used a stack array) — one allocation
  per battlefield permanent per SBA check. Cause 2: a fresh eval context per effect inside
  `abilities_are_blanked`'s O(permanents × effects) sweep. **Final: every interval OVERLAPS, max
  +1.51%, no regression demonstrated and nothing claimed.** **Fuzz: the 20-game A/B output differs
  in EXACTLY ONE LINE, the wall clock.**
  Tests **5,243 / 0 / 5** (+12 over a **5,231** baseline reproducing PB-DX54's close pin exactly,
  **69** targets, byte-exact set difference RE-TAKEN AFTER the fix cycle: 15 additions / 3 leavers /
  0 removals, all three leavers being mandated renames — **and the first draft of that cell said 12
  and 2 and was missing the third**, an enumeration error inside the one cell whose purpose is
  enumeration, caught by running the set difference rather than transcribing). **HASH 85 / PROTOCOL 44 BOTH UNMOVED**,
  gate-executed, closure counts MEASURED at **98 / 132**, and the counterfactual for the rejected
  stored-field design verified by execution. Coverage **UNMOVED at 1,140/1,803 = 63.2%**, 0 flips,
  3 comment-only card-def edits with the `Completeness::` marker diff EMPTY. All gates clean
  against the FINAL tree, **where two standing card-def gates FIRED** and were answered rather than
  dodged. **Revert matrix 9 rows, coordinator-executed, all three source files restored
  byte-exactly**; two rows were COVERAGE MEASUREMENTS and both gaps are now closed by probes RED
  under their own rows. Filed **OOS-DX42b-1..7** (`-6` and `-7` by the `/review` fix cycle). Full record:
  `memory/primitives/pb-DX42b-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-05 — **PB-DX54 SHIPPED** (`scutemob-232`; v4 queue rank 17 —
  **OOS-DX25c-6** CLOSED, plus rider **OOS-DX25-4** CLOSED and rider **OOS-DX25b-4** DECLINED
  and re-filed with a measured wire cost).
  **A ruling the engine could not implement, because its own resolution order put the answer out
  of reach.** Misdirection's 2004-10-04 ruling says *"You can choose to make a spell on the stack
  target this spell … **This spell is still on the stack when new targets are selected for the
  spell**"*, and `resolve_top_of_stack_inner` opened with `state.stack_objects.pop_back()` — so
  for the whole of a resolution the resolving object's ENTRY did not exist. Both single-target
  retarget requirements resolve their candidate through `stack_index_for_announced_target`, which
  therefore returned `None`, and Misdirection's own card was rejected as *not a spell* rather than
  for anything to do with self-targeting.
  **THE CR CITE IS WRONG IN THE SEED ROW, THE MEMO ROW AND THE DISPATCH CRITERION — ALL THREE
  INHERITED FROM ONE FILING.** They cite **CR 608.2m**; the rule that puts the departure LAST is
  **CR 608.2n** (*"As the final part of an instant or sorcery spell's resolution…"*), reinforced
  by CR 608.2's own preamble (*"608.2n and 608.2p are followed last"*). CR 608.2m is about an
  object removed by SOMETHING ELSE mid-resolution and cannot warrant the fix — and it IS the right
  cite for exactly one thing here, the departure helper's idempotence, where "already gone" really
  is 608.2m's case. `OOS-DX54-1`; the fourth batch running to inherit a wrong cite from its own
  dispatch.
  **THE DESIGN WAS SETTLED BY MEASUREMENT, AND THE MEASUREMENT IS THE HEADLINE.** The criterion
  demanded both options' blast radius be executed, and it was, before any real line changed: with
  the pop moved to the FUNCTION BOUNDARY the workspace runs **5,207 / 3 / 5** and **all three
  failures are SOURCE gates** — two keyed on a function name the scaffold renamed, one on the
  scaffold's own re-open-coded `.position(` scan. **ZERO behavioural.**
  **AND THAT ZERO IS EXACTLY WHY THE OBVIOUS SHAPE IS WRONG.** Departing at the function boundary
  breaks two SBAs that read `state.stack_objects` from inside `check_and_apply_sbas`: **CR 714.4**'s
  Saga sacrifice (*"…isn't the source of a chapter ability that has triggered but not yet left the
  stack"*) and **CR 309.6**'s dungeon removal. CR 704.3 checks SBAs when a player would receive
  priority — AFTER CR 608.2n — so a resolving FINAL chapter ability that had not departed postpones
  its own Saga's sacrifice by a whole SBA round, and **no behavioural test in the workspace catches
  it**. Shipped instead: departure at the two CR-ordered points inside `resolve_top_of_stack_inner`,
  plus an idempotent backstop in `resolve_top_of_stack` for the four early returns that run no
  trigger/SBA/priority tail — PB-DP8's *"a guard that returns early inherits the obligation of the
  statements it skipped"*, discharged structurally rather than at four `return` statements.
  **THE FIX OBEYS AN INHERITED GATE RATHER THAN RESPELLING PAST IT, AND A REVERT ROW PROVES THAT
  MATTERS.** The departure resolves its entry through the shared `stack_index_for_announced_target`
  (its first clause is `so.id == announced`, and the two id spaces are disjoint by construction, so
  the card clause cannot fire). Revert row **R6** respells it as `retain(|so| so.id != entry_id)`
  — which **satisfies PB-DX52's `r1a`**, that gate staying green — while re-opening exactly the
  drift `OOS-DX25-3`/`OOS-SIM3-5` were. Only this batch's `r3` catches it. *"A gate you edit prose
  to satisfy has stopped measuring"*, demonstrated by execution rather than quoted.
  **THE ROW'S YIELD CELL REPRODUCES — THE FIRST IN FIVE BATCHES THAT IS NOT A FLOOR — AND THE
  CENSUS SUPPLIES THE REASON THE ROW DOES NOT.** 2 deck-legal `Complete` defs: `misdirection` and
  `bolt_bend`. Only the two SINGLE-target requirements consult `state.stack_objects`;
  `TargetSpell`, `TargetSpellWithFilter` and `TargetSpellOrAbility` decide the object branch on
  `obj.zone == ZoneId::Stack` alone and the resolving spell's CARD never leaves that zone — which
  is precisely why PB-DX25c's T7 could route around the defect with a filter, and why
  `TargetSpellOrAbility` ships as a **stated CONTROL** (green before and after) rather than as a
  third subject. `untimely_malfunction` is a `Partial` third declarer; `deflecting_swat` declares
  the unaffected variant and is the still-open `OOS-DX25b-4`.
  **THE INVERSE ORACLE AXIS FOUND A SIBLING GAP NO DOCUMENT NAMES.** 7 printed-only defs against 0
  declared-only; **six** of the seven print *"choose new targets"* for a **COPY** (CR 707.10), not
  a CR 115.7 retarget, and the seventh is a real retarget that declares none of the three scanned
  requirements — the first draft of this line said FIVE and left two of the seven unaccounted, an
  enumeration error inside a census whose whole purpose is enumeration, caught by this batch's own
  `/review` reading the test's printed output rather than the prose. And
  `Effect::CopySpellOnStack`'s own doc says *"choose-new-targets deferred to M10"*. Same
  under-permission as `OOS-DX25b-4`, through a different `Effect`, with no registry row.
  `OOS-DX54-2`. *The two axes do not nest*, for the fifth batch running.
  **A PROBE THAT COULD NOT BE BUILT FOUND A LIVE, PRE-EXISTING DEFECT.** CR 714.4 exempts a Saga
  that is the source of a chapter ability *"that has **triggered** but not yet left the stack"*.
  `enter_step` QUEUES the chapter trigger, then runs SBAs, then flushes — and `sba.rs`'s guard
  scans `state.stack_objects` alone, never `state.pending_triggers`. So on the step-entry that
  crosses the FINAL chapter the Saga is sacrificed one mechanism early and that chapter resolves
  sourceless, doing nothing. Observed in one command's event slice (`CounterAdded {Lore, 3}` →
  `PermanentDestroyed` → `AbilityTriggered` → `AbilityResolved` **with no effect event**), with
  chapters I and II resolving correctly on the same fixture — which is what isolates it to the
  final chapter. **Pre-existing, proven structurally**: `git diff` over `sba.rs`,
  `turn_actions.rs`, `engine.rs`, `replacement.rs` and `saga.rs` is EMPTY, and the sacrifice
  happens at step entry, outside any resolution. Filed as **`OOS-DX54-4`** with **no probe**, on
  PB-DX49's `OOS-DX49-1` precedent, and distinct from the two Saga rows already filed — both of
  those would also break chapters I and II, which work here.
  **TWO REVERT ROWS ARE COVERAGE MEASUREMENTS, NOT PASSES, AND SAY SO IN THE TEST ITSELF.** R2
  (the function-boundary design) and R3 (the backstop) each redden ONE source gate and **no
  behavioural probe anywhere** — `OOS-DX52-2`'s shape said out loud. R2's probe is currently
  **UNBUILDABLE**, blocked behind `OOS-DX54-4`; R3's needs three fixtures nothing in the tree
  builds, filed as **`OOS-DX54-5`** with the generalisation that is worth more than the three
  instances — an early `return` inside `resolve_top_of_stack_inner` is a debt-inheriting guard and
  the workspace cannot tell whether a FIFTH one was added correctly.
  **THREE CORRECTIONS THE COORDINATOR MADE TO DELEGATED OUTPUT, AND ONE OF THEM IS THE
  COORDINATOR'S OWN MISTAKE.** (1) The probe agent shipped an **empty `#[test]`** whose body was a
  comment; its doc was excellent and is preserved verbatim in the module header, but an
  assertion-free test adds **+1 to this batch's own test delta** for a row that tests nothing —
  the one figure every later batch inherits as its baseline. (2) Removing that wrapper, **the
  coordinator's own cut ran back to the wrong section banner and deleted a PASSING test plus two
  helpers**; recovered from the agent transcript and re-verified green, and recorded because a
  silent recovery is how a deleted test becomes a permanently missing one. (3) `t5`'s headline
  assertion message **overclaimed**: its count is taken BEFORE the resolution, when the entry is
  present under both revisions, so the 0 it described happens somewhere no assertion in the file
  can see; reworded to state it is a PRECONDITION.
  **↻ THE `/review` FOUND 2 HIGH / 2 MEDIUM / 3 LOW / 1 NIT AND ALL EIGHT WERE TAKEN — AND BOTH
  HIGHs WERE THIS BATCH'S OWN GATES, DEFEATED BY EXECUTION.** *(1)* `r1`'s needle set was
  **narrower than its own doc sentence**: the doc said *"Every `X.pop_back(` / `X.remove(` /
  `X.pop_front(`…"* and the code iterated two of the three, so planting
  `state.stack_objects.remove(len - 1)` after the peek left **all nine roster gates GREEN** while
  `t1`/`t2`/`t4`/`t5` went RED — the plant reproduced the entire pre-fix defect, R1's exact red
  set, invisibly. It cited `OOS-DX51-6` **by name** as the reason it was *"keyed on the
  MECHANISM"*. **Why it was invisible from inside the file is the durable half**: `r1b`, the
  companion proving the detector fires, exercised only the two spellings the detector already
  handled — *a revert-proof written by the same author from the same mental model tests the
  needle set against itself* (`OOS-DX54-6`). *(2)* `r2` asked *"is there a departure within the
  preceding N bytes"*, so a **fifth tail placed 945 bytes AFTER an existing departure** was
  vouched for by it — precisely the CR 714.4 / CR 309.6 violation the gate exists to forbid, and
  every gate stayed green. Its own second measurement (*"the two departures are 464,693 bytes
  apart"*) ruled out the two EXISTING sites vouching for each other and said nothing about a new
  one. Re-keyed from a DISTANCE to a per-departure **COUNT**. Both defeats re-executed against
  the fixes: RED.
  Also taken: **the consumer audit missed the LKI CAPTURE clause**, and the change really does
  move `public_state_hash` at a command boundary (`[93,250,169,233]` vs `[225,85,133,201]`,
  measured) — not a regression, since CR 608.2h / CR 113.7a make capturing a resolving ability's
  source correct, but §7's fuzz justification rested on the omission and is rewritten
  (`OOS-DX54-8`); **`r5` was file-scoped while the reader set is a call graph**, defeated by a
  reader planted in `rules::saga::saga_view`, which is PB-DX48's `SITE_SRCS` defeat and PB-DX49's
  workspace-walk repair **one batch old and not carried across** (`OOS-DX54-7`); the census
  narrative said FIVE copy-family members and left two of seven unaccounted; `r3`'s stated claim
  was wider than what it measures; `c2`'s bot probe never reaches the redirect and now says so;
  and **the worker's own provenance claim about an untracked `None` file was wrong** — it compared
  a local EDT filesystem mtime against a UTC session start, and in one timezone the file was
  created 17 minutes INTO the session. *A timestamp comparison across two clocks is not a
  comparison.*
  Tests **5,231 / 0 / 5** (+21 over a **5,210** pre-edit baseline reproducing PB-DX53's close pin
  exactly, **68** targets, byte-exact set difference: 21 additions / 0 leavers / 0 removals / 0
  renames, count-vs-name reconciliation AGREES and the duplicate-name scan is EMPTY on both runs).
  **HASH 85 / PROTOCOL 44 BOTH UNMOVED — zero bumps**, gate-executed, closure counts MEASURED at
  **98 / 132**, and the counterfactual for both rejected designs verified by execution. Coverage
  **UNMOVED at 1,140/1,803 = 63.2%**, 0 flips, 0 card-def edits. All gates clean against the FINAL
  tree; `npm run build` N/A and said so. **Revert matrix 7 rows, coordinator-executed, all three
  source files restored byte-exactly** — R4/R5 precise complements, R6 defeating an inherited gate,
  R2/R3 disclosed. Filed **OOS-DX54-1..8** — and the first draft of this line said `-1..5`, which
  is dispatch hygiene 8's exact case for the FIFTH batch running, caught by re-checking this cell
  against the registry AFTER the `/review` fix cycle rather than before it. Full record:
  `memory/primitives/pb-DX54-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-05 — **PB-DX53 SHIPPED** (`scutemob-231`; v4 queue rank 16 —
  **OOS-DX21-1** CLOSED, its row corrected against three of its own claims).
  **One DSL identifier carrying two CR concepts, and the obvious fix repairs one card by breaking
  the other.** `rules/combat.rs` ASSIGNED `attackers_declared_this_turn = attackers.len()`, so on a
  turn with an extra combat phase (CR 500.8) attacking with three creatures in combat 1 and one in
  combat 2 dropped the count to one and `windbrisk_heights` went dead for the rest of the turn.
  **The ruling settles all three of the batch's open questions in one sentence each and needed no
  inference** (2007-10-01, via MCP): *"you'll get to play the card if you declared three
  **different** creatures as attackers **at any point in the turn**. A creature declared as an
  attacker in two different attack phases **counts only once**. A creature that entered attacking …
  **doesn't count** because you never attacked with it."*
  **But `Condition::YouAttackedWithNOrMore` had two readers wanting OPPOSITE semantics.**
  `legions_landing` is CR 508.3d — per DECLARATION — so making the field accumulate repairs
  Windbrisk and REGRESSES Legion's Landing (2 in combat 1 + 2 in combat 2 would transform it; the
  printed trigger never even fires). PB-DX21's review finding M3 says exactly this, and the seed
  row's own prescribed fix shape — *"a per-turn accumulation with per-creature dedup … and the
  migration must leave Legion's Landing reading the per-declaration count"* — is right about the
  two REQUIREMENTS and wrong about their being one field. **So the DSL split**:
  `latest_attacker_declaration_size: u32` (renamed, semantics untouched) beside a new hashed
  `creatures_declared_as_attackers_this_turn: OrdSet<ObjectId>`, read by a new
  `YouAttackedWithNOrMoreCreaturesThisTurn` while the renamed `…ThisDeclaration` keeps the old one.
  **Both old names were LIES and both were renamed** — the field said "this turn" and meant "the
  latest declaration"; the Condition stated neither scope while two cards read it for opposite ones,
  so a card author reaching for the shorter identifier got the per-declaration semantics silently,
  which is the default-choice trap that produced this seed.
  **TWO PROPERTIES HOLD BY CONSTRUCTION RATHER THAN BY CARE, WHICH IS THE POINT OF THE SHAPE.**
  Legion's Landing is byte-identical — same field, same assignment, same arm body, zero behavioural
  lines on its path — so `t6`/`t7` are a PIN on a property that already holds, not the thing
  establishing it. And CR 508.4's exclusion (*"Such creatures are 'attacking' but, for the purposes
  of trigger events and effects, they never 'attacked.'"*) holds because the write site reads the
  DECLARATION command's own attacker list, never `combat.attackers` — which PB-DX51 made the shared
  path for the four entrant sites too, so an entrant is never a parameter to that function.
  **THE SECOND MEMBER WAS INVISIBLE TO THE DECLARED AXIS BECAUSE ITS DEFECT WAS THAT IT WAS
  MISSING.** The row scopes this to `windbrisk_heights` ALONE and the v4 memo cell says "1
  deck-legal `Complete`". The inverse ORACLE axis found `minas_tirith`, printing *"Activate only if
  you attacked with two or more creatures this turn"* and `partial` behind an `ENGINE-BLOCKED` note
  demanding `Condition::AttackedWithNCreatures(2)` — **an identifier that had existed as
  `Condition::YouAttackedWithNOrMore(u32)` since PB-OS6 (2026-07-19)**. The note was FALSE at HEAD
  and outlived the commit that falsified it (`OOS-DX47-6`'s shape). Authored; the batch's single
  coverage flip. *A declared-axis census cannot see a card whose defect is an unauthored ability.*
  **THE BATCH'S OWN ROSTER GATE THEN REPRODUCED `OOS-DX36-8` ONE AXIS OVER, AND ITS MODULE DOC
  ARGUED FOR THE CHOICE.** R1/R3 matched on `format!("{def:#?}")` under a doc defending it —
  correctly — on the EXHAUSTIVENESS axis: a derived `Debug` has no variant list to under-enumerate,
  so PB-DX26's `RollDice` lesson does not apply. True, and irrelevant to the failure it had. A
  `Debug` render also prints PROSE compiled into the def, so `scourge_of_the_throne`'s
  `Completeness::partial("… Effect::AdditionalCombatPhase …")` — a string literal, not a comment —
  was counted as a DECLARER and R3's population read **5** when the truth is **4**. The tree already
  solves this and the plan told the batch to use it: `decision_site_walk::def_contains_variant`,
  whose string arm matches a variant name EXACTLY — **and the mechanism is that exactness, not the
  `PROSE_FIELDS` denylist, which the first draft of this cell credited and which is never even
  consulted on a sentence-shaped note** (a later batch "hardening" one of these censuses by adding
  a `PROSE_FIELDS` key would be doing nothing; `OOS-DX49`'s rule that a reason is the half the next
  batch reuses). **R1 had the same shape and was ONE BLOCKER NOTE
  away from the same false positive** — not a remote risk here, since the card this batch repaired
  carried a note naming a `Condition` variant BY IDENTIFIER, which is what blocker notes do. Both
  re-keyed; `scourge_of_the_throne` pinned in R3's must-be-absent list as the member that
  DISCRIMINATES the two walks. Filed `OOS-DX53-2`. *A census walk has two axes — how exhaustively
  it reaches, and whether what it reaches is code or prose — and defending one says nothing about
  the other.*
  **SR-36 HAS A WORKED EXAMPLE THREE TIMES OVER IN ONE AXIS**: `grep -rl AdditionalCombatPhase`
  returns **8** files and **four** declare nothing — three mention it in a `//` comment and one in
  a compiled completeness note. The plan's own §9.2, itself written as an SR-36 worked example, said
  the population was 7; the implementation agent reported 5; it is **4**.
  **FOUR CITES CORRECTED, INCLUDING THIS TASK'S OWN TITLE.** The v4 row title, the dispatch title
  and AC 7368's framing all cite **CR 508.6**, which is verbatim a BOOLEAN per-player predicate with
  no count and no turn-scope content. The `OOS-DX21-1` row already said so, and PB-DX21's review had
  corrected `legions_landing.rs` for exactly this mis-cite — after which it propagated into the
  queue row title and the dispatch title.
  Tests **5,210 / 0 / 5** (+14 over a **5,196** pre-edit baseline reproducing PB-DX39's close pin
  exactly, **67** targets, byte-exact set difference: 14 additions / 0 leavers / 0 removals / 0
  renames, count-vs-name reconciliation run and duplicate-name scan EMPTY on both runs; re-taken
  AFTER the `/review` fix cycle).
  **HASH 84 → 85 / PROTOCOL 43 → 44, ONE bump each, both predicted in writing before any production
  line** (`a37f8239`), closure type counts predicted and confirmed UNCHANGED at **98 / 132**;
  **the AC's PROTOCOL-UNMOVED prediction REFUTED with its own ground verified true**. Coverage
  **1,139 → 1,140 = 63.2%**, ONE flip named before regeneration. All gates clean against the FINAL
  tree; `npm run build` N/A and said so. **Benches: no regression, six quiet runs, the first A/B
  thrown away as contaminated (16% same-code spread) rather than published, and the apparent
  improvement deliberately not claimed** — and the batch's own prediction of a regression refuted,
  with the evidence pointing at `GameState`'s size rather than `PlayerState`'s as PB-DX18's real
  driver. **Revert matrix 3 rows, coordinator-executed, 3 discriminating, 0 UNDISCRIMINATED**, with
  its own instrument corrected twice.
  **↻ The `/review` found 2 HIGH / 5 MEDIUM / 4 LOW / 2 NIT and ALL THIRTEEN WERE TAKEN — and both
  HIGHs are this batch's own thesis committed inside the gates that state it.** *(1)* The
  single-write-site mechanism gate was defeated **two ways by execution**: an aliased
  `let set = &mut ps.<field>; set.insert(id);` in a NON-allowlisted file (which is `OOS-DX51-6`
  verbatim, cited in that gate's own body for a different lesson and not carried across), and a
  SECOND `.insert(` planted beside the real one inside an allowlisted file, because the allowlist
  match was a PRESENCE check (`OOS-DX48`'s r1 defeat — and not academic, since inserting twice per
  declaration IS the double-count the CR 400.7 dedup exists to prevent). Re-keyed by INVERTING the
  polarity — enumerating what may mutate a container is unbounded and fails open, enumerating the
  8 READ methods is short and fails closed — plus a preceding-path `&mut` axis and an EXACT COUNT
  per allowlisted file; both defeats re-executed and RED, with a new classifier test pinning all
  nine forms on synthetic input (`OOS-DX53-4`). *(2)* **R2 still classified declaredness with
  `format!("{def:#?}")` while R1 and R3 had been re-keyed** — and R2 is the ONLY test whose job is
  to find an undeclared printed member, i.e. the exact method that found `minas_tirith`. Proven by
  planting a printed "attacked with ... this turn" line plus a `Completeness::partial` note naming
  the missing identifier: `is_declared` came back TRUE **from the note** and the `undeclared` list
  came back EMPTY with all four roster tests green. This file's own module doc had named that risk
  **for R1** and left it standing in R2.
  Also taken: **AC 7368's "and its exiled card resolving" was VACUOUS** — the fixture placed
  Windbrisk on the battlefield, so no Hideaway ETB ever fired, the exile zone was measured EMPTY
  and `Effect::PlayExiledCard` resolved on nothing; now driven for real (played from HAND, spanning
  to p1's next turn because the land enters TAPPED and cannot pay its own `{T}` that turn) and RED
  under an executed revert (`OOS-DX53-5`). **R3's doc said the population was 5 while its own
  assertion said 4** and named four absentees. **`minas_tirith`'s replacement comment was itself
  false**: it called the old `ENGINE-BLOCKED` note *"already false when authored"*, but that note
  is present in `b6f748f8` (2026-07-10) and the variant did not exist until PB-OS6's `bc79a72c`
  (2026-07-19) — it was TRUE when written and ROTTED, which is the same defect one direction over.
  **The stated REASON `def_contains_variant` works was wrong in three places** (the mechanism is
  EXACT matching; `PROSE_FIELDS` is never consulted on a sentence-shaped note, so a later batch
  "hardening" the census by adding a key there would do nothing — `OOS-DX49`'s rule that a reason
  is the half the next batch reuses). Plus a fuzz-gate failure message contradicting its own pin,
  **four wrong CR cites** (Melee is **702.121a**, not 702.111 which is Menace; CR **602.5** not
  602.5b for activation legality; CR **602.5d/307.5** not 500.10a for sorcery timing; CR **506.1**
  not 506.5 for the per-phase combat state), c2's control shape disclosed (it reddens under R1 on
  its PRECONDITION, not its subject — *"all rows RED" is a true sentence the wrong assertion can
  produce*), and an r1 assertion described as a non-vacuity check that the compiler already
  guarantees.
  Filed **OOS-DX53-1..5** — and the first draft of this line said `-1..3`, which is dispatch
  hygiene 8's exact case for the fourth batch running, caught by re-checking this cell against the
  registry AFTER the fix cycle rather than before it. Full record:
  `memory/primitives/pb-DX53-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-04 — **PB-DX52 SHIPPED** (`scutemob-229`; v4 queue rank 14 —
  **OOS-DX25b-1** and **OOS-DX25b-5** CLOSED, plus **OOS-DX25c-3** CLOSED as a third).
  **A printed line with no id space to say it in.** CR 115.7a lets Bolt Bend *"change the target
  of target spell **or ability** with a single target"*, and the "or ability" half was dead:
  an activated or triggered ability's stack entry is minted by `state.next_object_id()` and
  pushed into `state.stack_objects` and **never into `state.objects`**, because it owns no card.
  So it was unannounceable twice over — `queries::legal_targets_per_slot` enumerates object
  candidates from `state.objects()` alone, and `validate_object_satisfies_requirement` opens with
  `state.objects.get(&id).ok_or(ObjectNotFound)?`. The visible shadow: **
  `TargetSpellOrAbilityWithSingleTarget` and `TargetSpellWithSingleTarget` were behaviourally
  IDENTICAL on every production path**, and the `is_spell` guard separating them was reachable
  only from a fixture that collapsed the two id spaces.
  **THE CR ARGUES FOR THE DESIGN THIS BATCH REJECTED, AND THE REJECTION IS MEASURED RATHER THAN
  ARGUED.** CR 113.1c is explicit — *"An ability can be an activated or triggered ability on the
  stack. This kind of ability is an object"* — and CR 109.1 lists an ability on the stack first
  among the things an object is. Registering ability entries in `state.objects` was costed at
  stage 0 across 241 full-map walk sites and REJECTED, because such an entry must claim a
  `ZoneId` and the only honest claim is `ZoneId::Stack`; and **`casting.rs`'s `TargetSpell` arm
  decides "is this a spell" by `obj.zone == ZoneId::Stack` ALONE**, so a registered ability would
  immediately have become a legal target for *"counter target spell"* — CR 115.1a-wrong (a spell is not a permanent, and "counter target spell" names a spell), a new
  defect shipped while closing an old one. It also forces zone membership
  (`simulator::invariants::check_zone_integrity`), which moves `public_state_hash` for **every**
  game with an ability on the stack and double-counts the entry in
  `loop_detection::compute_mandatory_state_hash`; and `GameObject` has two non-`Option` fields to
  fabricate and **no `CardType` that fits an ability**. `state.objects` is this engine's CARD-object
  map; CR 113 abilities are modelled by `state.stack_objects`, and registering them in both is a
  DUPLICATE representation, not a truer one.
  **ONE SHAPE CHOICE COLLAPSED MOST OF THE WORK, AND IT WAS ALREADY IN THE TREE.**
  `Target::StackObject(ObjectId)` carries the stack ENTRY's own id — and
  `stack_registry::stack_index_for_announced_target`'s **first clause is already
  `so.id == announced`**, so `Effect::ChangeTargets`, `Effect::CounterSpell` and
  `Effect::CopySpellOnStack` all resolve one through the SAME shared arithmetic a card id goes
  through, with no second lookup to drift. `effects/mod.rs`'s `resolve_effect_target_list_indexed`
  likewise ALREADY accepted a stack-entry id (its `exists_on_stack` clause, written for
  CR 702.21a Ward). **`ResolvedTarget` is deliberately NOT widened**: it has ~55
  `if let ResolvedTarget::Object(..)` sites with no `else` in that file, so a third variant would
  have minted 55 silent-swallow sites the compiler cannot flag, to buy nothing.
  **THE BATCH HAD TO CLOSE A THIRD SEED TO AVOID SHIPPING A DEFECT, AND THE REGISTRY NAMED IT IN
  ADVANCE.** Making an ability announceable makes it a reachable `Effect::ChangeTargets` VICTIM,
  and `retarget::plan_target_change` derived BOTH `source_chars` (CR 702.16b protection) and
  `self_id` (CR 601.2c) from `card_in_stack_zone`, which is `None` for every ability. Shipping the
  id space alone would have **silently disabled the protection check for every ability-shaped
  redirect** — Bolt Bend redirecting a red ability onto a creature with protection from red.
  `OOS-DX25c-3` predicted exactly this and filed it as *"unreachable today, blocked behind
  `OOS-DX25b-1`"*. **A seed's "unreachable today" is a claim with an expiry date, and the batch
  that closes its blocker is the batch that has to honour it.** Closed with
  `stack_registry::source_of` (CR 113.7, exhaustive over all 25 kinds, no wildcard) — the helper
  `OOS-DX25-4`'s fix shape also names, built here **without** taking that rider, because no event
  semantics changed on either counter path.
  **THE REVERT MATRIX IS ALSO A COVERAGE MEASUREMENT, AND THAT IS THIS BATCH'S DURABLE HALF.**
  8 rows, executed by the coordinator rather than accepted from the delegated reports, all 8
  discriminating. Row **R6** — put `card_in_stack_zone` back, i.e. undo the protection fix —
  reddened **exactly one thing**: `r7b`, a SOURCE gate that reads the call site's text. **No
  behavioural probe moved.** A source gate proves a line is spelled a certain way; it cannot prove
  the line does anything, and a later batch that "simplifies" the helper while keeping the name
  satisfies it completely. So the fix described at length as *"a defect this batch would have
  created"* was, at that moment, standing on a text comparison. Closed by
  `t10_protection_from_red_refuses_an_ability_shaped_redirect`, asserted in BOTH directions with a
  non-vacuity floor, RED under R6. Filed as `OOS-DX52-2`: **a row that reddens only a source gate
  is telling you the behaviour has no probe, not that the row is uninteresting.**
  **A GATE'S JUSTIFICATION ROTTED AND THE GATE COULD NOT SEE IT — FOUND BY READING WHY IT HAD NOT
  FIRED.** PB-DX8's `completeness_deviation_scan` fired correctly on `bolt_bend`'s rewritten note
  (answered by ALLOWLIST with the contract widening STATED, rather than by rewording the comment
  to dodge the needle — a gate you edit prose to satisfy has stopped measuring). Checking why it
  had NOT fired on `deflecting_swat` found that its `RECORDED_BASELINE` reason quotes *"Interactive
  choice deferred to M10"*, a sentence **this batch deleted**; the entry kept passing because the
  def still matched the same needles for the same underlying reason. Nothing in the tree checks
  that an allowlist entry's quoted fragment still occurs in the def it names (`OOS-DX52-1`).
  **FOUR OF THIS BATCH'S OWN CR CITES WERE WRONG AND WERE CORRECTED AGAINST THE RULES SERVER.**
  CR 113.3 was cited four times for *"an ability on the stack has its source's text"* — CR 113.3 is
  *"There are four general categories of abilities"* and **no rule says that at all**; CR 113.7a
  says the opposite, that the ability exists INDEPENDENTLY of its source, so naming one by its
  source is a display convention and now says so. CR 113.7a was cited six times for the SOURCE
  definition, which is CR 113.7. A bare CR 113.1 stood where CR 113.1c, CR 110.1 or CR 102.1 was
  meant. And *"an ability ceases to exist"* is **CR 608.2n**, verbatim, now quoted where it is
  load-bearing. The pass happened at all because the delegated channel agent reported it had no
  MCP rules tools and **flagged that rather than proceeding as if it had**.
  **THE CENSUS IS PRINTED BY A TEST AND ITS FIRST DRAFT WAS WRONG IN THE WAY SR-36 PREDICTS.**
  Union **35** by `all_cards()` — DECLARED 34, PRINTED 4, 3 on both axes (all NOW-CORRECT), 1
  PRINTED-only (`siren_stormtamer`, STILL-BLOCKED, and the missing identifier is NAMED: a FILTERED
  sibling of `TargetSpellOrAbility` for *"that targets you or a permanent you control"*), 31
  DECLARED-only. A preliminary file-text grep had put `misdirection` in the PRINTED axis; it is not
  there — that phrase occurs only in a COMMENT comparing it to Bolt Bend. **The v4 memo's
  "1 deck-legal `Complete`" cell is a FLOOR**, and closing the seed is what makes Misdirection's
  spell-only restriction enforceable for the first time.
  Tests **5,156 / 0 / 5** (+39 over a **5,117** pre-edit baseline that reproduces PB-DX36's close
  pin exactly, **65** targets, byte-exact set difference: 40 additions / 1 disclosed leaver / 0
  removals — the leaver is the mandated `t3` inversion — with the count-vs-name reconciliation run
  and the duplicate-name scan EMPTY). **PROTOCOL 42 → 43 / HASH 83 → 84, ONE bump each, both
  predicted in writing per half before any production line** (`8f919967`), both closure type counts
  predicted and confirmed UNCHANGED at **98 / 132**. Coverage **UNMOVED at 63.2%**, 0 flips
  predicted per def before regeneration. All gates clean against the FINAL tree, where `clippy`
  fired once; `npm run build` N/A **and the acceptance criterion's prediction that the frontend
  would move is REFUTED and reported**. **Benches: no regression, seven runs, the first A/B thrown
  away as contaminated rather than published, and the apparent improvement deliberately not
  claimed** — the controls move the same order, and `size_of` is identical at both revisions.
  Filed **OOS-DX52-1..9**. Full record: `memory/primitives/pb-DX52-execution-notes.md`; handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-09-04 — **PB-DX36 SHIPPED** (`scutemob-228`; v4 queue rank 13 —
  **OOS-CARDS2-6** FILED, because it had no registry row at all, and **CLOSED**, both halves).
  **A `bool` that only the hasher reads, and a printed trigger family with no `TriggerCondition`.**
  `TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer { combat_only }` was dispatched
  **inside the `GameEvent::CombatDamageDealt` arm only**, under a `TODO(PB-37)`, and the lowering
  destructured `combat_only` away with `{ .. }` — so the runtime `TriggeredAbilityDef` had no home
  for it and the flag was read in **exactly one place in the workspace, `state/hash.rs:6848`**.
  `true` and `false` were behaviourally identical. `sigil_of_sleep` — `Complete` by derive,
  deck-legal — declares `combat_only: false` and silently dropped the noncombat half of its printed
  trigger. Separately, no general *"whenever this permanent deals damage"* condition and no
  damage-dealt `EffectAmount` existed, so `exalted_angel`'s printed ability was unauthored.
  **THE OBVIOUS FIX — DELETE THE FLAG NOBODY READS — IS WRONG, AND ONLY AN INVERSE ORACLE CENSUS
  SHOWS IT.** Declared users of `combat_only: true` at HEAD: **0**. *Printed* users: **1** —
  `breath_of_fury`, *"When enchanted creature deals **combat** damage to a player…"*, which simply
  does not declare the condition today because its blocker is Aura re-attachment. A census over the
  DECLARED axis alone says "delete it", and deleting it would have made that card permanently
  over-fire. The flag is **animated** instead. *The declared axis and the printed axis do not nest*
  — PB-DX26's and PB-DX43's lesson, saving a third batch.
  **THE RECIPIENT AXIS IS THE HALF NO DOCUMENT NAMES, AND IT IS WHAT REPAIRS TWO MORE DEFS.**
  `curiosity` and `ophidian_eye` both print *"to an **opponent**"* and approximated it as *any
  player*. New `DamageRecipient { Any, Player, Opponent }` closes that; each def's SURVIVING blocker
  is named in-source rather than left implied — the costless *"you may"* (`OOS-DX35-5`).
  **It went on `TriggerEvent`, not `TriggeredAbilityDef`, and the reason is measured**: that struct
  has no `Default` derive and **190 exhaustive struct literals across 44 files**, reproducing
  `OOS-DX35-1`'s figure exactly at HEAD. Seven new unit variants; one ability lowers to exactly
  **one** `trigger_on` through an exhaustive wildcard-free `match`, so a new axis value is a compile
  error rather than a silent drop — the failure mode `combat_only` itself was.
  **DISJOINTNESS IS BY CONSTRUCTION AND VERIFIED AT THE EMIT SITES, NOT ASSUMED.**
  `GameEvent::CombatDamageDealt` has exactly **one** emit site (`rules/combat.rs:2382`) and emits no
  `DamageDealt`; all **five** `DamageDealt` emit sites are CR 120 noncombat. One shared
  `queue_damage_source_triggers` serves both arms with `is_combat` a property of the **EVENT** —
  which is the distinction `combat_only` failed to make. A combat damage event fires any one ability
  **exactly once**, COUNT-asserted, because PB-DX47's double-push shape passes a `>= 1` assertion,
  and RED under an executed revert that duplicates the combat-arm call.
  **THE TASK BRIEF'S CR CITE IS WRONG AND WAS NOT OBEYED.** It cites **CR 603.10a** for *"that
  much"*, and repeats it inside acceptance criterion 7333. CR 603.10a is *look-back-in-time
  **zone-change** triggers* — verbatim, and it says nothing about a damage amount. The **13 cites
  that would otherwise have said CR 603.10a** ship against **CR 603.2c** and **CR 608.2h /
  CR 113.7a** instead. (The first draft of this sentence said *"all 13 cites this batch
  introduced"*, which is false as written — the batch introduces ~110 CR cites in all; the
  `/review` counted them.) The tree's pre-existing
  (and correct) 603.10a LKI cites were not touched. **A brief is a claim like any other**, and
  obeying this one would have put 13 wrong cites in the tree under an AC that read as satisfied.
  **THE MEMBER LIST WAS A FLOOR TWICE OVER, AND THE EXTRA MEMBER IS A *GRANTED* ABILITY.**
  The task brief names **one** self-family def (`exalted_angel`) — queried directly, not assumed;
  `goblin_lackey`, `warren_instigator` and `tandem_lookout` all came from this batch's own stage-0
  inverse oracle scan, and the `all_cards()` roster then corrected **that** to ten still blocked.
  *An inherited member list is a floor; so is the one you derived yourself an hour earlier.*
  `tandem_lookout` grants *"Whenever this creature deals damage to an opponent, draw a card"*
  through Soulbond — structurally invisible to a per-def ability-list walk, because it declares zero
  triggered abilities of its own. **Two blocker notes this batch FALSIFIES were repaired in place**
  (PB-DX27's rule): `niv_mizzet_visionary`'s *"Neither is expressible"* is now half false, because
  `EffectAmount::DamageDealt` **is** its *"that many"*.
  **THE SENTINEL SURVIVOR SCAN OBEYED PB-DX50'S RULE TO THE LETTER AND WAS DEFEATED ANYWAY.** The
  re-pin missed `pb_dx2_command_gates.rs`'s **`41u32`** — `\b` between `1` and `u` is not a word
  boundary, `OOS-DX20b`'s own lesson, handled for HASH five lines earlier in the same script. The
  survivor scan changed the matcher's **SHAPE** (a line window, not symbol-adjacent) and kept the
  **VALUE** pattern `\b41\b`, so it was structurally incapable of seeing the miss and reported 0.
  *A survivor scan has two axes and varying one is half a check* (`OOS-DX36-8`).
  **THE `/review` FOUND 2 HIGH / 4 MEDIUM / 5 LOW-NIT AND ALL ELEVEN WERE TAKEN — AND HIGH 1 IS A
  CORRECTNESS DEFECT THIS BATCH SHIPPED, WITH THE DOC COMMENT ASSERTING THE OPPOSITE.**
  `queue_damage_source_triggers` was called **inside `for assignment in assignments`**. One
  `CombatDamageDealt` carries every assignment of the step in a single `events.push`, and CR 510.2
  makes them simultaneous — so CR 603.2c's *"triggers only once each time its trigger event
  occurs"* was violated by any source with more than one assignment. Measured: a 5/5 blocked by two
  2/2s dispatched the self family **twice**, gaining 2 + 3 in two resolutions; a 6/6 trampler
  carrying `Sigil of Sleep` fired the self family twice while the Aura half correctly fired once.
  The mirror ruling settles the CR question — **Boros Reckoner**, Gatherer 2017-03-14: *"its ability
  triggers once and one target is dealt that much damage."*
  **The census behind the false claim was CORRECT, which is what made it dangerous**: emit-site
  disjointness is true and bounds the ARMS, not the LOOP INSIDE one arm, and nobody checked the
  second. **And every COUNT probe drove a single-assignment fixture** — `t2`, whose own docstring
  says a `>= 1` assertion would pass on PB-DX47's double-push shape, **passes under the defect**,
  re-verified by the coordinator. *A COUNT assertion proves exactly-once only on the fixture shape
  it drives* — PB-DX47's own lesson one axis over, inside the batch that cites it. Fixed by grouping
  each event's assignments by source (first-appearance order, never sorted) and dispatching the self
  family once per source with the SUM; `t8`/`t9` added, and **the revert was re-executed
  independently rather than accepted from the report** — `left: 2, right: 1` on exactly those two,
  every other probe green.
  **HIGH 2: the class gate was bypassable on the two axes it did not key on**, proven twice by
  execution — a second dispatcher **outside `src/rules/`** (in `effects/mod.rs`, which emits
  `DamageDealt` at four of five sites) and a **`use` alias inside** the scanned directory, each
  leaving all 710 core tests green. Both fixes already existed in this same test crate, one batch
  old (PB-DX49's workspace walk and its bare-name re-key). **Fixing it surfaced a third axis nobody
  had named**: the scan window looked only FORWARD from a walk marker, and a `use` alias's bare name
  sits BEFORE it. Now bidirectional; both bypasses re-executed and RED.
  Tests **5,117 / 0 / 5** (+20 over a **5,097** pre-edit baseline that reproduces PB-DX35's close
  pin exactly, **64** targets, byte-exact set difference: 20 additions / 0 leavers / 0 removals /
  0 renames, count-vs-name reconciliation run and duplicate-name scan EMPTY, **re-taken AFTER the
  fix cycle**). **PROTOCOL 41 → 42 /
  HASH 82 → 83, ONE bump each, both predicted in writing per half before any code** (`a9fca688`),
  with every wire cell PROBED at stage 0 and both closure type counts predicted and confirmed
  UNCHANGED at **98 / 132**. Coverage **1,138 → 1,139 = 63.2%**, ONE flip named before regeneration.
  All gates clean against the FINAL tree; `npm run build` N/A and said so. **Benches: no regression,
  six runs, same-code band (3.76%) measured FIRST and wider than every difference in the table, with
  the apparent `board_wipe_4p` improvement killed by a second HEAD run rather than published.**
  **Fuzz: the engine half is fuzz-neutral BY MEASUREMENT** — five seeded fixtures reddened on the
  `CORPUS_COMPLETE` re-deal and an executed ablation (engine change in, marker reverted) turns all
  five green. Filed **OOS-DX36-1..9**. Full record:
  `memory/primitives/pb-DX36-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-04 — **PB-DX35 SHIPPED** (`scutemob-227`; v4 queue rank 12 —
  **OOS-DX4-2** and **OOS-DX4-5** both CLOSED, plus **OOS-DP10-5** CLOSED and **OOS-DX8-3**
  updated). **A mode you may not legally choose, and a flag that decided nothing.**
  `ModeSelection.mode_targets` was honoured on the casting and activated paths and **nowhere** on
  the triggered-ability path, so a modal trigger had to declare its targets FLAT and they were
  required whichever mode was chosen. `retreat_to_kazandu` — `Complete`, deck-legal, named by
  neither the seed nor the memo — could not gain 2 life on an empty board, because the OTHER mode's
  "+1/+1 counter on target creature" had no legal target and CR 603.3d removed the whole trigger.
  Separately, `Effect::LookAtTopThenPlace.optional` was destructured away (`optional: _`), so five
  `Complete` defs recorded a printed "you may" the engine never asked.
  **"SCOPE THE TARGETS TO MODE 0" — ONE OF THE TWO OPTIONS THE BRIEF OFFERS — IS CR-WRONG, AND
  CR 700.2b SAYS SO IN ONE SENTENCE**: *"If one of the modes would be illegal (due to an inability
  to choose legal targets, for example), that mode can't be chosen."* The engine wrote
  `modes_chosen = vec![0]` in **both** arms of its `min_modes` branch, unconditionally — picking
  the very mode the rule forbids. So the automatic choice is now legality-aware, which is the
  minimum correct behaviour rather than scope creep, and it is what makes the criterion's own
  headline probe pass. The controller is still not ASKED; that decision is STATED, the
  `modal_trigger` row stays `AutoChosen` with its now-false `site` string rewritten, and the human
  channel is filed (`OOS-DX35-4`).
  **THE SEED ASKS FOR ONE SHARED ARITHMETIC AND THE TREE HELD THREE HAND-ROLLED COPIES.** The
  memo names no consumer at all; the stage-0 re-derivation found **four** — `flush_sorted`'s
  requirement lookup, its CR 603.3d slot derivation, the answer-path cross-slot re-derivation, and
  the `modes_chosen` assignment itself — all now served by one `trigger_modal_plan`, which slices
  through `casting::per_mode_target_requirements`, the SAME helper `handle_cast_spell` and
  `queries::spell_target_requirements` call. `rules/mana.rs` is deliberately NOT unified and says
  so: it answers *"is this targeted"*, not *"which targets"*.
  **THE MEMO'S SECOND PREDICTED FLIP DID NOT HAPPEN, AND THE SEED ROW PREDICTED ITS OWN FAILURE.**
  `OOS-DX4-2` warns that *"moving the targets into `mode_targets` looks like the CR 601.2c-correct
  repair and would silently DROP the requirement"*. For `hullbreaker_horror` that trap is **still
  armed** — because a `Normal`-kind trigger carries a RUNTIME `ability_index` while both
  `ModeSelection` read sites index the REGISTRY, and its modal ability sits at registry index 1
  behind `Keyword(Flash)`. Census over all 7 corpus modal triggered abilities: **3 are misaligned**
  (`hullbreaker_horror`, `glissa_sunslayer`, `junji_the_midnight_sky`), with **two distinct
  symptoms** — the first two resolve `Effect::Nothing` so the whole modal ability is a no-op, while
  junji's `WhenDies` is one of the three lowering arms that pre-resolve `modes.first()` into
  `effect`, so it executes **mode 0 forever** and the mode choice is a fiction. **Zero deck-legal
  blast radius, measured** (all three are non-`Complete`), which is why it is FILED
  (`OOS-DX35-1`): the only structural fix lowers `modes` into `TriggeredAbilityDef`, costing
  **190 exhaustive struct literals across 44 files plus both bumps**. So `hullbreaker_horror` is
  **re-adjudicated, not re-shaped** — `partial` kept, marker rewritten to name the surviving
  blocker — and the flip count is **ONE**. *A yield cell that names members is a FLOOR on the
  census and a CEILING on the flips*: `OOS-DX4-2`'s member list was short by more than double
  (5 of 7) while its two named flips delivered one.
  **HALF B NEEDED NO NEW WIRE VARIANT, AND THE PRINTED CARDS ARE WHY.** All five defs say *"you may
  put **a** [creature/land] card from among them"* (MCP, verbatim) — CR 608.2's *choose up to one*,
  which is exactly `EffectChoiceQuestion::ChooseObject { count: 1, up_to: true }`, the variant
  PB-DX28 built and the v4 memo's own row-15 cell had already named. A zero-cost `PayOptionalCost`
  was rejected as dishonest (the client would render *"Pay {0}?"*) and a new `Confirm` variant as
  both costlier and NARROWER — it carries "whether" and not "which". The decline is asserted by
  RESOLUTION EFFECT through all three channels, and `candidates` is sorted ASCENDING by `ObjectId`
  because `Zone::top_n` is TOP-first, the reverse — so `first()` equals the pre-batch
  `min_by_key` winner exactly, pinned by a probe whose fixture makes the two orders genuinely
  disagree.
  **`OOS-DP10-5`'s STANDING SWEEP HAD BEEN INHERITED UNRUN BY NINE BATCHES. IT WAS EXECUTED AND IT
  FOUND A LIVE DEFECT ON SEVEN DECK-LEGAL CARDS.** `Effect::CounterUnlessPays` destructures
  `cost: _` and delegates to `Effect::CounterSpell`, so CR 118.12a's *"unless its controller pays"*
  is never offered — `make_disappear`, `spell_pierce`, `stubborn_denial`, `mana_leak`,
  `izzet_charm`, `mana_tithe`, `flusterstorm`, **all `Complete` by derive**. Its in-source
  justification (*"the payer never has an incentive to voluntarily tax themselves"*) is **false on
  its face**: the payer is the OPPONENT whose spell is being countered (`OOS-DX35-3`). The sweep
  also produced one **checked-and-CLEAN** result, recorded because it is what proves each discard
  was read rather than counted.
  **TWO OF THIS BATCH'S OWN PUBLISHED FIGURES WERE REFUTED BY ITS OWN PRINTING TEST.** The
  corpus-wide "you may" population was written into two registry rows as **213 / 90** from a
  throwaway script whose `oracle_text` extractor did not join Rust's `\`-newline continuations and
  silently truncated every multi-line string; the truth is **365 / 165**. And the pinned MDFC is
  named `Turntimber Symbiosis // Turntimber, Serpentine Wood`, not its file stem. **PB-DX8's rule —
  publish the figure, do not transcribe it — caught its own author.**
  **AND TWO SEED IDs COLLIDED, BECAUSE THE BATCH RAN ITS HALVES AS TWO DELEGATED IMPLEMENTATIONS
  AND BOTH ALLOCATED `OOS-DX35-1`.** Found at close-out by grepping the ID rather than trusting
  either report. The index-space defect keeps the number on **12** in-source cites against **1**;
  the other is `-2` and its single cite was repointed in the same commit, on the `OOS-M11-10`
  precedent. *A seed ID is allocated against the registry, and two workers on one task cannot both
  read a registry neither has written to yet.*
  Tests **5,097 / 0 / 5** (+39 over a **5,058** pre-edit baseline, **63** targets, byte-exact set
  difference: 39 additions / 0 leavers / 0 removals / 0 renames, with the count-vs-name
  reconciliation run on this batch's own close-out numbers AND re-taken after the `/review` fix
  cycle, per dispatch hygiene 8). **HASH 82 / PROTOCOL 41 both UNMOVED
  — ZERO bumps for the whole PB, both predicted in writing before any code** (`c6646052`).
  Coverage **1,137 → 1,138 / 1,803 = 63.1%**, ONE flip named before regeneration. All gates clean
  against the FINAL tree (`cargo fmt --check` fired there and was fixed); `npm run build` N/A and
  said so. **Benches: no regression, seven runs, the one outlier killed by a third run on each side
  rather than averaged away, and the apparent speed-up deliberately not claimed.** Filed
  **OOS-DX35-1..10** — **and the first draft of these lines said `-1..9`, which is dispatch
  hygiene 8's exact case for the second batch running**, caught by re-checking this cell against
  the registry AFTER the `/review` fix cycle rather than before it; `-10` exists because the
  `/review` found CR 700.2b mode legality is decided PER SLOT, so a mode with no legal
  COMBINATION is still chosen and then removed. **The `/review` found 9 issues and all 9 were
  taken; four were gate defeats it PROVED by execution** — `r7` twice (a reword that re-asserts
  the lie while still naming `trigger_modal_plan`, and the same needle split across a Rust
  line continuation), `b2` once (a SECOND `LookAtTopThenPlace` node with `optional: false` hid
  behind a first one carrying `true`, because the fold was `any` and not `all`), and `t9` once
  (a branch-selective fifth copy behind `if trigger.kind == CardDefETB` left the whole crate
  green, because both of `t9`'s cases drove `Normal` — *a differential probe proves agreement on
  the branches it drives and nothing about the branches it does not*). All four defeats were
  re-executed against the fixes and are now RED. `r8` is the general answer to the third: a
  MECHANISM gate asserting every triggered-target extraction in `rules/` lives inside
  `trigger_modal_plan` — **and its own first run refuted the population figure its author had
  written one paragraph above it**, because the throwaway script behind that sentence searched
  for `AbilityDefinition::Triggered {` *with the brace* and `rules/mana.rs` puts the brace on
  the next line. Full record: `memory/primitives/pb-DX35-execution-notes.md`; handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-09-04 — **PB-DX51 SHIPPED** (`scutemob-226`; v4 queue rank 11 —
  **OOS-DX21-4**, **OOS-DX21-2** and rider **OOS-DX21-5** ALL CLOSED). **CR 508.8 asks whether
  creatures WERE DECLARED; the engine asked whether any are still in combat NOW.**
  `turn_structure::advance_step` decided "skip declare-blockers and combat-damage" from a
  step-END read of `combat.attackers.is_empty()`. Remove the lone attacker from combat while the
  step is still open and the engine jumped to `EndOfCombat` in a combat where creatures *were*
  declared — taking every other creature's block, every later CR 508.4 entrant and the whole of
  CR 510 with it.
  **THE ROW ASKS FOR A THIRD FIELD AND CR 508.8 ASKS FOR ONE.** `OOS-DX21-4` prescribes *"whether
  the declaration was empty at declaration time, AND whether anything has been put onto the
  battlefield attacking since"*. That is right about the two FACTS and wrong about the two FIELDS:
  CR 508.8 ORs them in a single sentence — *"if no creatures are declared as attackers **or** put
  onto the battlefield attacking"* — so the predicate is one existential and one monotone
  `bool` IS it. `CombatState.had_attackers` is set by ONE new mutator, `CombatState::add_attacker`,
  which is now the only production path into `attackers` and therefore serves BOTH CR rules.
  **An empty declaration needs no special case at all** — it never enters the declaration loop, so
  the marker stays clear and CR 508.8's skip still fires (CR 508.1a's *"if any"*, pinned
  wrong-way-round by `t3`). `!attackers_declared` would have been wrong for exactly that reason,
  and the row says so.
  **THE SEED'S OWN REPRODUCTION RECIPE IS WRONG IN ALL THREE OF ITS NAMED ROUTES, AND THAT MAKES
  THE DEFECT WORSE, NOT BETTER.** *"Kill it / phase it out / stop it being a creature"* removes
  **nothing**: an exhaustive census (`attackers.remove` ∪ `retain` ∪ `=` ∪ `clear`) finds
  `combat.attackers` emptied at exactly three production sites — `remove_from_combat`, reachable
  only from `Effect::RemoveFromCombat` and `apply_regeneration`, plus one raw removal on the
  Ninjutsu bounce path. **The engine implements TWO of CR 506.4's six removal causes**
  (`OOS-DX51-2`); a dead attacker's stale `ObjectId` just stays in the map. The route that DOES
  reproduce is **`reconnaissance`** — `Complete`, deck-legal, *"{0}: Remove target attacking
  creature you control from combat and untap it"*, `Cost::Mana(default)` and
  `timing_restriction: None`, so **free, instant-speed and repeatable during the declare-attackers
  step**. `thaumatic_compass` is the other `Complete` user. **Live on 2 deck-legal `Complete` defs,
  not reachable-in-principle**, and every behavioural probe drives that real route.
  **THE CR 508.4 ENTRANT CENSUS REPRODUCES AT FOUR AND THE BRIEF'S LIST HAD TWO FALSE MEMBERS.**
  The four are `effects/mod.rs`'s two token paths and `resolution.rs`'s Myriad (CR 702.116a) and
  Ninjutsu (CR 702.49a) — PB-DX21's four, every line number drifted. `state/builder.rs` sets
  `combat: None` and cannot put anything onto the battlefield attacking; `replacement.rs:2347` has
  no insert at all (`:2435` is `enters_attacking: false`, a `TokenSpec` field initialiser that
  FEEDS site 1). Reported as refuted rather than silently dropped.
  **THIS BATCH'S OWN `r1` GATE WAS DEFEATED BY EXECUTION, AND BOTH HALVES WERE BLIND AT ONCE.**
  The first draft matched the single literal `.attackers.insert(`. A sixth entry site written as
  `let map = &mut combat.attackers; map.insert(..)` left `r1` **GREEN** — and because it ADDS a
  site rather than replacing one, `r1b`'s exact-5 call-site count stayed **GREEN** too. That is
  *a gate written for one variant measures that variant* (PB-DX26 → PB-DX43 → PB-DX45 → PB-DX47),
  committed inside the gate whose own module doc cites two of those defeats. Re-keyed on the
  MECHANISM — all four ways to obtain a mutable path to the map, on ANY receiver, over-collecting
  deliberately; `r1d` carries the multi-line spelling with a differently-shaped matcher; `r1c`
  re-checks each allowlist entry's reason IN SOURCE and checks the two "different type, same field
  name" entries **by type**. Four defeats executed, all four now RED, and the multi-line one
  reddens `r1d` alone (`OOS-DX51-6`).
  **A SECOND SR-38 HOLE SITS ON THE VERY `if` STATEMENT THE ROW SENDS YOU TO, AND IS FILED NOT
  FIXED.** `legal_actions.rs:954` computes `is_active` and the *attacker* offer consumes it; the
  *blocker* offer has no such exclusion, while the engine refuses the attacking player outright
  (CR 509.1a). Found twice independently inside this batch. Out of scope — the criterion says ONE
  condition, and every offer-layer change carries `OOS-DX21-6`'s blast radius (`OOS-DX51-3`).
  **THE PROBE AUTHOR REFUTED TWO OF THE COORDINATOR'S OWN REVERT PREDICTIONS AND WAS RIGHT.** `t2`
  does not discriminate `R1`/`R2`: the AND-with-`is_empty()` means `had_attackers` is load-bearing
  only when the WHOLE map empties, so a "some attackers survive" fixture cannot separate the two
  predicates. `t2` keeps its place because it is the only probe that proves the DOWNSTREAM
  consequence — a real block registers and damage is marked — which `t1`'s single-attacker fixture
  structurally cannot. `t4` as specified had the same defect and was redesigned to remove the
  entrant too. Both disclosed in the tests' own docs.
  Tests **5,058 / 0 / 5** (+14 over a **5,044** pre-edit baseline, **61** targets, byte-exact set
  difference: 14 additions / 0 leavers / 0 removals / 0 renames). **HASH 81 → 82 / PROTOCOL 41
  UNMOVED, one bump, both predicted in writing before any code** (`06ba6760`), closure type count
  UNMOVED at 132 and measured at the merge base rather than assumed. Coverage unmoved
  **1,137/1,803 = 63.1%**, **0 flips**, churn reverted, **0 card-def edits**. All gates clean
  against the FINAL tree; `npm run build` N/A and said so (`tools/` diff is empty). **Benches: no
  regression, five runs, and the apparent 1-3.7% speed-up deliberately NOT claimed** — the control
  bench moves as much as the affected one. **Fuzz: the engine half is fuzz-neutral BY MEASUREMENT**
  (a third run with only the offer conjunct ablated reproduces the merge base byte-identically),
  and `AlreadyDeclaredBlockers` goes **9 → 0**. Filed **OOS-DX51-1..7** — **and the first draft of
  these lines said `-1..6`, which is dispatch hygiene 8's exact case**, caught by re-checking this
  cell against the registry AFTER the `/review` fix cycle rather than before it. **`-7` exists
  because the `/review` defeated the RE-KEYED `r1` gate twice more** (a wholesale `CombatState`
  write-back, and a second `&mut self` mutator on the type itself — the latter blinding `r1` and
  `r1b` simultaneously for the second time in one batch), **and because the widening written for a
  third finding did not fix it**: `r1d` skipped allowlisted files WHOLESALE, so a multi-line borrow
  planted in a file allowlisted for one `remove(` call stayed green. Six successful bypasses of
  three successive drafts is the argument for making the field private — measured at ~160 sites and
  deliberately not taken here. Full record:
  `memory/primitives/pb-DX51-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-04 — **PB-DX18 SHIPPED** (`scutemob-225`; v4 queue rank 10 —
  **six seeds closed in one batch**: **OOS-DP2-7**, **OOS-DP2-4**, **OOS-DP2-8**, **OOS-DX2-4**,
  **OOS-DX2-1**, **OOS-M11-5**). **Six commands and one announcement the engine accepted without
  checking the precondition the CR attaches to them.**
  `Command::TakeMulligan` and `Command::KeepHand` were gated on `validate_player_exists` and
  nothing else — there was no pregame state anywhere to consult, so a mid-game `TakeMulligan`
  shuffled the sender's whole hand into their library and drew seven. `Command::ChooseMiracle`
  checked `cards_drawn_this_turn == 1` and never WHICH card was drawn, so a tutored miracle card
  could be revealed on any turn whose first draw had happened. `validate_targets_inner`'s
  empty-requirements arm waved arbitrary declared targets onto a spell that requires none — and
  since PB-DX48 that fires **Ward**. Both `ShuffleIntoOwnerLibrary` sites emitted
  `GameEvent::LibraryShuffled` and shuffled nothing.
  **TWO NEW STORED FIELDS, DESIGNED TOGETHER FOR ONE HASH BUMP, AND ONE OF THEM CARRIES TWO
  RULES.** CR 103.5 says both *"a player who is dissatisfied with their initial hand may take a
  mulligan"* (a pregame procedure) **and** *"Once a player chooses not to take a mulligan ... that
  player may not take any further mulligans"* (per player). A bare `game_started: bool` closes
  only the first, so `GameState.pregame: PregamePhase` carries the set of players who have kept
  and one shared `validate_pregame_mulligan_allowed` answers both for both commands.
  `PlayerState.miracle_pending` is CR 702.94a's *"as you draw it"* conjunct, assigned
  UNCONDITIONALLY at the draw site so a non-eligible draw CLEARS it.
  **THE CENSUS WAS SHORT BY A WHOLE MECHANISM AND IT IS SPLICE.** Neither seed row, nor the v4
  memo cell, nor this batch's own site table names it. CR 702.47a *"copy this card's text box onto
  that spell"* means a spliced spell requires the spliced card's targets, and
  `AbilityDefinition::Splice` carried `cost` / `onto_subtype` / `effect` and **no `targets` field
  at all** — so the splice target rode the very arm this batch closes: no type check, no hexproof
  / shroud / protection, no CR 608.2b re-validation, and `glacial_ray`'s spliced *"2 damage to any
  target"* resolved at **nothing**. A batch that only added the CR 601.2c rejection would have
  broken the corpus's one splice card.
  **44 OF 46 REJECTIONS ARE A SHAPE PRODUCTION CANNOT PRODUCE, AND THAT IS WHY 42 GREEN TESTS
  COULD NOT SEE THIS.** Instrumented and measured before anything was repaired: of 46 CR 601.2c
  rejections workspace-wide, **44 are on objects with no `card_id`** — the naked
  `ObjectSpec::card()` gotcha, where a fixture builds a def carrying the right requirement,
  registers it, and never links it. Architecture Invariant 9 makes that unreachable in a real
  game. Exactly **two** have real defs, and both are findings: golden script `layers/081` says
  BESTOW in its metadata, its notes and all eight of its CR cites while issuing a plain
  `cast_spell`, and `stack/146` is the splice case.
  **THE SEED NAMES ONE RE-PERMUTATION CHANNEL AND THERE ARE TWO.** `OOS-DP2-4`'s addendum warns
  that `StdRng` is not algorithm-stable across `rand` majors. True — and `Zone::shuffle` drew its
  indices with `Rng::random_range`, whose *sampling* algorithm is equally unpinned, so pinning
  only the generator leaves the identical defect one layer down. Shipped as an in-tree
  Fisher-Yates over an in-tree SplitMix64, and the pin is **structural**: `rand` is dropped from
  `crates/engine/Cargo.toml` AND `crates/card-types/Cargo.toml`, so the engine cannot construct an
  RNG at all.
  **THE RE-DEAL COST THE MEMO BUDGETS AT "18+" IS ONE PIN, MEASURED.** The `*_SEED` axis is the
  wrong axis: the simulator's opening deal uses `SliceRandom` with its own `StdRng` and is not one
  of the four sites. The single pin that moved is the fuzz decision partition, which gained
  `surveil` — an improvement — **attributed by an EXECUTED A/B** (`e7dee121` green without the
  pin, `c1132e44` red with it) and re-pinned rather than re-tuned.
  **THREE FIXTURE FAMILIES WERE PINS ON THE DEFECTS.** Three `rules::commander` mulligan tests
  drove `KeepHand` → `TakeMulligan` on one player, which CR 103.5 forbids (repaired in place, no
  name changed). Golden script `layers/081`, above. And the Darksteel Colossus test asserted the
  `LibraryShuffled` EVENT and never the library — with a fixture whose library was **EMPTY**,
  which is why the phantom was invisible: with nothing to permute, "shuffled" and "put on top"
  are the same state.
  **A NEW FAILURE MODE OF THE SENTINEL RE-PIN, AND IT IS THE OPPOSITE OF THE KNOWN ONE.** PB-DX50
  and PB-DX20b each recorded a re-pin regex that was too NARROW. This batch's handled both
  spellings and was too **WIDE**: it rewrote the prose *"HASH 80 -> 81"* into *"HASH 81 -> 81"*
  inside the doc paragraph announcing the bump. **A survivor scan is structurally blind to that**
  — it looks for what was MISSED — and this batch's correctly reported 0. Reading every changed
  line of the diff is what caught it (`OOS-DX18-3`).
  **AND THIS BATCH'S OWN GATE WAS DEFEATED BY ITS OWN REVERT ROW.** `r1`'s first draft looked for
  the string `finish_redirect_shuffle` in a `Redirect` arm; a consumer written as
  `finish_redirect_shuffle(false, ..)` contains it, drops the obligation entirely, and left the
  gate GREEN. `OOS-DX47`'s `r3` shape, committed inside the roster file that states the rule,
  found by executing the revert. Re-keyed on the arm's own bound field.
  **The splice offer gate was SHIPPED AND THEN REVERSED, and the suite is what refuted it.**
  Gating the offer for target-carrying splice cards reddened three PB-DX29 pins, because the
  corpus's only splice card targets — so the gate deleted the whole channel to avoid one refusal
  on a path that previously produced a SILENT wrong resolution. The deviation stays open, pinned
  wrong-way-round (`OOS-DX18-1`).
  Tests **5,041 / 0 / 5** (+26 over the 5,015 pre-edit baseline, **60** targets, itemised by NAME
  by a byte-exact Python set difference as **28 additions / 2 leavers / 0 removals** — both
  leavers being doctest line-number shifts of exactly +2, the height of the new `pub mod pregame;`
  pair). **HASH 80 → 81 / PROTOCOL 41 UNMOVED, one bump, both predicted in writing before any
  code.** Coverage unmoved **1,137/1,803 = 63.1%**, **0 flips**, churn reverted; 3 card-def edits
  with **no `Completeness` marker moved**. All gates clean against the FINAL tree; `npm run build`
  N/A and said so (`git diff main..HEAD -- tools/play-server/frontend` is empty and `node_modules`
  is absent). **Benches: a REAL uniform ~2.5-4.5% regression, four runs, same-code band measured
  FIRST** — `size_of::<PlayerState>()` moves 360 → 376 (+4.4%), on a struct copied at every
  mutation. Filed **OOS-DX18-1..6** (`-6` by the `/review` fix cycle, after the first draft of these
  lines said `-1..5` — dispatch hygiene 8, caught by re-checking this cell against the
  registry AFTER the fix cycle rather than before it). Full record:
  `memory/primitives/pb-DX18-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-03 — **PB-DX20b SHIPPED** (`scutemob-222`; v4 queue rank 9 —
  **OOS-DX20-10** ≡ **OOS-DX20-5** CLOSED as ONE defect, cross-cited). **An Aura's printed
  restriction named three card types and the DSL could only say one, so the def said "any
  permanent" instead — and PB-DX20 had just made that human-reachable.**
  CR 702.5a restricts both what an Aura spell may target (CR 303.4a) and what it may stay
  attached to (CR 704.5m). `imprisoned_in_the_moon` (`Complete`, deck-legal) prints *"Enchant
  creature, land, or planeswalker"* and declared `EnchantTarget::Permanent`, which also admits
  artifacts, enchantments and battles; `sba::matches_enchant_target`'s `Permanent` arm is a bare
  `true`, so the SBA would not clean up an illegal attachment either. `EnchantFilter` had
  `has_card_type` (ONE type) and `has_subtypes` (an OR over **sub**types) and no OR over card
  **types**. Shipped: `EnchantFilter::has_card_types`, lowered onto the **already existing**
  `TargetFilter.has_card_types` — no parallel OR mechanism.
  **THE STRUCTURAL HALF IS THE BATCH, AND THE MEMO'S SITE CELL HAD THE RIGHT NUMBER AND THE
  WRONG SHAPE.** The v4 row says *"three sites"*. Re-derived at stage 0, before any code: it is
  **two ARITHMETICS and three CONSUMERS**, and the consumers were already shared —
  `casting::enchant_target_to_requirement` and `sba::enchant_filter_matches` were independent
  hand-written copies of one six-field predicate, while the CR 303.4a gate, the CR 704.5m SBA and
  `queries::spell_target_requirements` each consumed one of them. **A batch that patched "three
  sites" one at a time would have carried the new field in two copies** — the drift the fix
  exists to remove. `casting::enchant_filter_to_target_filter` is now the only place that knows
  what an `EnchantFilter` field means, and `sba.rs`'s predicate is DELETED in favour of calling
  it and handing off to `effects::matches_filter`, the same predicate the cast path already runs.
  The CR 303.4a gate's call is deliberately KEPT: PB-DX20 put it there so cast-time and SBA-time
  agree, and that property now holds by construction rather than by two copies agreeing.
  **THE COMPILER WILL NOT DO ITS JOB HERE, PROVEN TWICE BY EXECUTION.** Adding a field to
  `EnchantFilter` produces **ZERO** compile errors workspace-wide — every construction site,
  engine and tests and all 1,803 card defs, uses `..Default::default()`, and `#[serde(default)]`
  covers deserialization. Re-executed independently by the coordinator in an isolated worktree:
  with an eighth field planted, `cargo build --workspace` printed `Finished` and **all ten
  behavioural probes stayed green**. `r5_every_enchant_filter_field_is_lowered` is the only thing
  in the tree that reddens, and revert row R5b proves its second half separately — planting the
  field *and* updating the pin, leaving the lowering alone, still reddens on the *unlowered*
  assertion, because a pin that only checked the field list would be satisfied by the very edit
  that hides the bug (`OOS-DX20b-2`, and the class is not `EnchantFilter`-specific).
  **THE CENSUS FOUND A THIRD MEMBER NO DOCUMENT NAMES, AND A FOURTH CORRECTION TO ITS OWN AXIS.**
  `breath_of_fury` prints *"Enchant creature you control"* and declared `EnchantTarget::Creature`,
  silently dropping the controller clause — absent from both seed rows and from the memo cell,
  and needing **no new expressiveness at all**, since `EnchantFilter.controller` has existed since
  PB-DX20. Repaired rather than filed. And the population needing a `Filtered` filter is **SEVEN,
  not the six** an OR-or-controller substring axis finds: `awaken_the_ancient` prints *"Enchant
  Mountain"* — no OR, no comma, no controller clause — and still cannot be any bare variant. *A
  substring axis would have pinned six and called it measured.* Both populations now pinned
  separately.
  **THE MEMO'S COVERAGE CELL IS REFUTED, AND THE DEF'S OWN NOTE ALREADY SAID SO.** Row 9 predicts
  *"+1 `partial` unblocked (`kayas_ghostform`)"*. That def's `Completeness::partial` marker reads
  *"NOT blocked: 'Enchant creature or planeswalker'"* and names a different blocker — a trigger
  keyed to the **enchanted permanent's** zone change plus a return from graveyard-or-exile. It
  stays `partial`, with the surviving blocker restated and its now-false sentence rewritten.
  Predicted in §0.4 before regeneration; coverage **unmoved at 1,137/1,803 = 63.1%, 0 flips**.
  **A NEIGHBOURING BATCH'S ROW DIED HERE, AND IT WAS DESIGNED TO.** PB-DX49's Pair A
  (`imprisoned_in_the_moon` × `binding_the_old_gods`) was reachable **only** because of the
  over-wide `Permanent` — an enchantment is a permanent. With the printed filter in place the two
  card-type sets are disjoint, so CR 303.4a refuses the cast and CR 704.5m detaches.
  `r4a_pair_a_depends_on_oos_dx20_10` went red exactly as PB-DX49 wrote it to, and is
  **re-adjudicated, not deleted**: the death is now COMPUTED from the intersection of the two type
  sets, so a widening resurrects the pair loudly. Verified it vacates no behavioural coverage —
  nothing outside that roster file names the card, and PB-DX49's deck-legal coverage rests on
  Pair B, which never sat behind this seed — **and the first draft of that justification was
  itself false, which the `/review` caught**: it said *"nothing outside that roster file names
  the card"*, while at the merge base five files did, one of them the very PB-DX20 roster pin
  this batch inverts. The conclusion survives on the narrower true reason — that reference was a
  roster PIN and no fixture ever drove Pair A — and the correction is recorded in the test's own
  doc, because a stated reason is the half the next batch reuses. The rename is this batch's
  single test leaver.
  **AND A LIVE, PRE-EXISTING SR-38 DEFECT FOUND BY EXECUTION**: `legal_actions.rs:1276` builds the
  `DeclareAttackers` eligible list from **raw printed** `obj.characteristics.card_types`, never
  `calculate_characteristics` — so once Imprisoned resolves and its Layer-4 effect makes the
  enchanted permanent a Land, the offer layer keeps offering it as an attacker and the engine
  refuses. `status.tapped`, `Defender` and `Haste` are read from the same raw struct three lines
  away, so a *granted* Defender is equally invisible. Byte-identical under revert, so pre-existing;
  filed `OOS-DX20b-1` and pinned by CLASS and COUNT rather than asserted away.
  Tests **5,015 / 0 / 5** (+24 over the 4,991 pre-edit baseline, **60** targets, itemised by NAME
  as 25 additions / 1 disclosed leaver / 0 removals). **PROTOCOL 40 → 41 / HASH 79 → 80, one bump
  each, predicted in writing before any code including the unchanged type counts (98 / 131).**
  Coverage unmoved **63.1%**, **0 flips**, 3 card-def edits with no `Completeness` marker moved.
  All gates clean against the FINAL tree; benches measured over four runs and *no regression
  demonstrated*, with the same-code repeatability band shown to be wider than the effect. Filed
  **OOS-DX20b-1..7**. Full record: `memory/primitives/pb-DX20b-execution-notes.md`; handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-09-03 — **PB-DX50 SHIPPED** (`scutemob-221`; v4 queue rank 8 —
  **OOS-DX25-1** and **OOS-DX29-2** both CLOSED). **CR 702.140a says a mutate spell *targets* its
  host, and the engine had never modelled that as a target at all.**
  The choice lived in `AdditionalCost::Mutate` and never entered `spell_targets`, which is the only
  thing `GameEvent::PermanentTargeted` is built from — so **Ward never fired on a mutate cast**, and
  the mutate validator checked zone, creature-ness, non-Human and owner **and nothing else**: no
  hexproof, no shroud, no protection, on 6 deck-legal `Complete` defs that PB-DX29 had just made
  human-reachable. Shipped shape: `casting::mutate_target_requirement()` is ONE synthesized
  `TargetRequirement` (PB-DX20's `enchant_target_to_requirement` idiom) consumed by cast validation,
  by announcement and by the CR 702.140b re-check; the host is APPENDED to `spell_targets`, so
  PB-DX48's `push_target_announcement` dispatches Ward with **no new code**. CR 702.140c's over/under
  choice moves to **resolution time** on PB-DP9's CR 608.2d channel.
  **THE SEED'S OWN PRESCRIPTION WOULD HAVE MADE THE FIX CR-WRONG, AND ONLY READING THE RULE CAUGHT
  IT.** `OOS-DX25-1` says to route the host into `spell_targets` so CR 608.2b re-validation sees it.
  **CR 702.140b is an explicit EXCEPTION to CR 608.2b** — *"if its target is illegal, it ceases to be
  a mutating creature spell and continues resolving as a creature spell"* — so it does **not** fizzle,
  and obeying the seed would have regressed a behaviour the engine already got right. It does not
  regress as shipped for a **structural** reason rather than a checked one: the fizzle gate lives
  inside the `StackObjectKind::Spell` arm and `MutatingCreatureSpell` is a disjoint arm. That is the
  kind of load-bearing accident a later batch deletes by "unifying the two arms", so `t7`/`t7b`/`t7c`
  pin it, each asserting the fallback fires **and** that no `SpellFizzled` is emitted.
  **THE SITE LIST WAS SHORT BY ONE AND THE MISSING SITE IS THE ONE THAT WOULD HAVE BROKEN.** Both
  seeds and the v4 row name two sites. The third is `legal_actions.rs`'s `non_human_own` offer
  enumeration — a fourth hand-rolled copy, reading **raw** characteristics. Tightening the cast path
  while it kept a looser predicate is *a clean offer followed by a guaranteed refusal*: the SR-38
  shape PB-DX29 gated Fuse to avoid, PB-DX44 recreated and PB-DX45 shipped — **this batch would have
  been the fourth**. Fixed; the offer layer reads layer-resolved characteristics for the first time,
  and its host set had to become **per-CARD**, which no document anticipated, because protection is a
  property of the *(source, target)* pair.
  **THIS BATCH'S OWN PLAN WAS REFUTED THREE TIMES BY EXECUTION, AND THE COORDINATOR TWICE.** (i) The
  plan told the implementer to delegate the CR 702.140b re-check to `is_target_legal` — which checks
  **only the cast-time zone**, so "one arithmetic" would have DELETED three checks; *the shared thing
  was weaker than the duplicated thing*. Corrected before shipping. (ii) The plan's sentinel census
  (45/11) was produced by a same-line regex **while citing PB-DX45's lesson that a re-pin is only as
  wide as its regex**; the truth is 47/13, and the first survivor check used the same regex and
  reported zero. (iii) The coordinator's prescription "make the pairing an exhaustive `match` on the
  pair" **does not work** — an N² tuple match cannot be made compile-forced — and the first draft
  followed it and shipped a comment claiming the opposite, refuted by its own revert matrix.
  **A PRE-EXISTING ENGINE-WIDE DEFECT, FOUND BY EXECUTION**: `Command::AnswerEffectChoice` swept
  triggers `resolve_top_of_stack_inner`'s tail had already swept, so **every trigger a replayed
  CR 608.2d resolution produced was queued TWICE** — on every PB-DP9 / ENG-1 / PB-DX28 / PB-DX45
  channel, reachable before this batch, with no fixture to show it. Found because golden script 192
  put two Gemrazer triggers on the stack. `handle_all_passed`, the ordinary CR 608.1 path, calls
  `resolve_top_of_stack` and then **nothing** — verified in source. Filed and closed as
  `OOS-DX50-1`.
  **↻ The `/review` (1 HIGH / 1 MEDIUM / 2 LOW-MEDIUM / 3 LOW / 1 NIT — all eight taken, none
  declined) FOUND A HANG CAUSED BY THE COORDINATOR'S OWN INSTRUCTION.** The `is_copy` guard added to
  the mutate arm — which the coordinator ordered, overruling the copy audit's advice to defer it —
  shipped as an early `return Ok(events);`. The instruction was *"make it agree with
  `resolution.rs:819`"*, and it copied `:819`'s **condition** while dropping its **control flow**:
  `:819` is an `if/else if` chain that FALLS THROUGH to the shared resolution tail. The `return`
  skips `check_triggers_with_timing`, `check_and_apply_sbas`, `flush_pending_triggers` and
  `grant_priority_to_active_player`, leaving `priority_holder: None` with both players passed and the
  spell stranded — every subsequent `PassPriority` returning `NotPriorityHolder { expected: None }`,
  an **unrecoverable game**, proven by execution. **That is PB-DP8's own recorded lesson — *a guard
  that returns early inherits the obligation of the statements it skipped* — committed inside a batch
  that had the sentence available to it.** The batch's own `r4` gate stayed GREEN throughout.
  Also taken: **`r3` was defeated TWICE** (it polices the requirement's DEFINITION and is blind to
  its CONSUMER — and the consumer is where all four historical hand-rolled copies lived); the
  CR 605.4a site census was defeated two ways at once (wrong file set **and** blind to the
  struct-literal spelling of its own needle); a 30-space run in a user-visible browser prompt; and
  **a false comment neither the batch nor the review had seen** — `abilities.rs` still said the
  mutate target *"is never entered into `spell_targets`… this fix only takes effect once that gap
  closes"*, and **PB-DX50 half 1 IS that gap closing**, so the comment outlived the commit that
  falsified it (`OOS-DX47-6`'s shape, inside the batch whose headline is a false comment). Pinned
  behaviourally by `t12` rather than swapped for another sentence.
  **AND THE COORDINATOR'S REGISTRY EDIT DESTROYED A WORD.** The `OOS-DX29-2` closure split that row
  by column, but it has carried **6 cells in a 4-column table since it was filed** (its own
  `Entwine | Fuse | EscalateModes` uses unescaped pipes), so the edit appended to a fragment and
  **overwrote the cell holding `Fuse`**. Repaired, pipes escaped, incident recorded in the row. A
  sweep found **five** such rows; the other four are deliberately NOT repaired and are filed as
  `OOS-DX50-11` with the gate that would have caught all five — *the registry is machine-read, which
  is the finding PB-DX49 closed `OOS-RR4-3` on.*
  Tests **4,991 / 0 / 5** (+50 over the 4,941 pre-edit baseline, **59** targets, itemised by NAME as
  53 additions / 0 removals / 3 disclosed leavers / 0 renames). **PROTOCOL 39 → 40 / HASH 78 → 79,
  one bump each, predicted per half in writing before any code.** Coverage unmoved **63.1%**, **0
  flips**, **0 card-def edits**. All gates clean against the FINAL tree. Filed **OOS-DX50-1..11**.
  Full record: `memory/primitives/pb-DX50-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-03 — **PB-DX49 SHIPPED** (`scutemob-220`; v4 queue rank 7 —
  **OOS-RR4-1** CLOSED and rider **OOS-RR4-3** CLOSED). **The engine half of corner case #36 —
  the audit's last open GAP — closes, and #36 goes to PARTIAL rather than COVERED, because the
  card half is honestly still open.**
  **Every CR 714 decision read the PRINTED card definition, at five sites, and none consulted the
  layer axis.** A permanent whose abilities are blanked kept accruing lore counters (CR 714.3b),
  kept firing chapter triggers (CR 714.2b) and was sacrificed anyway (CR 714.4) — it behaved as if
  nothing had happened to it. Shipped shape: `layers::abilities_are_blanked` is now **the**
  ability-blanking predicate (CR 708.2a face-down plus the continuous-effect scan, with the
  classification delegated to PB-DX43's exhaustive no-wildcard `modification_blanks_abilities`, so
  a fourth channel is a compile error), IG-1 in `queue_carddef_etb_triggers` was refactored to
  consume it so **exactly one such predicate exists in the tree**, and `rules::saga::saga_view`
  answers every CR 714 question once for all five sites. `resolution.rs`'s two chapter-effect
  lookups are deliberately **not** consumers — CR 113.7a makes an ability on the stack independent
  of its source — and now say so at each site, so a later batch cannot "finish the job".
  **THE SEED'S OWN PRESCRIPTION WOULD HAVE MADE THE FIX CR-WRONG, AND ONLY READING THE RULE
  CAUGHT IT.** `OOS-RR4-1` says a fix to the first three sites *"leaves a blanked Saga still taking
  its ETB counter"*, i.e. it treats the surviving counter as part of the defect. **CR 714.3a has no
  "with one or more chapter abilities" clause** — CR 714.3b and CR 714.4 both carry it and 714.3a
  does not (verified verbatim). CR 613.1f removes abilities, **not subtypes**, so a Layer-6-blanked
  permanent is still a Saga and still takes its counter; only CR 708.2a's *"no text, no name, **no
  subtypes**"* makes a face-down permanent not a Saga. Suppressing the counter would have produced
  a **second** wrong outcome — an un-blanking would fire chapter I instead of resuming at chapter
  II, because CR 714.2b needs the ability to exist at the instant counters are put on. So site 4
  asks TWO questions and the query answers them from two fields; `t6` pins the pair at exactly
  **1** lore counter and exactly **0** chapter triggers.
  **THE CENSUS REFUTED THREE PUBLISHED FIGURES AND ONE OF THEM WAS THIS BATCH'S OWN.** The Saga
  population is **3**, not the 4 in §1g, in the registry row, in this batch's plan and in its own
  orientation pass — `song_of_freyalise` declares `abilities: vec![]` and names `SagaChapter` only
  in two `// TODO`s and its `inert` note. **SR-36's failure mode for the fourth consecutive batch in
  this queue** (`OOS-CARDS2-7` → `OOS-DX47-2` → PB-DX48 → here). The blanker population is
  **11 / 8**, not 13 / 8 and not the row's own corrected 9: **every figure in that chain grepped the
  string `RemoveAllAbilities`, which is the wrong question**, because PB-DX43 moved CR 305.7's
  ability loss into `SetLandTypes` and both moons are blankers again through a variant no such grep
  can see. Only deciding by **calling** `modification_blanks_abilities` counts a blanker as a
  blanker. **And the deck-legal 8 agrees with the row by coincidence of TOTALS, not of MEMBERSHIP**
  — the row's 8 was 8-of-13 `RemoveAllAbilities` defs; the true 8 is six of those **plus the two
  moons**. A batch that checked only the total would have recorded the row as confirmed. A **fourth**
  blanker that can reach an enchantment was found and no document names it (`oko_thief_of_crowns`,
  a bare `TargetPermanent` for a printed *"target artifact or creature"*; `known_wrong`, so 0
  deck-legal blast radius — `OOS-DX49-4`).
  **A LIVE DEFECT FOUND BY EXECUTION, ON A DECK-LEGAL `Complete` CARD, AND DELIBERATELY LEFT
  UNPINNED.** While the channel probes were choosing an observable resolution effect,
  `binding_the_old_gods`' chapter I — *"Destroy target nonland permanent an opponent controls"* —
  **destroyed nothing**, with one legal target on the board and the trigger measurably on the stack.
  `fire_saga_chapter_triggers` queues a `Normal` trigger whose `ability_index` indexes
  `def.effective_abilities(..)`, while `flush_sorted`'s requirement lookup reads
  `obj.characteristics.triggered_abilities[ability_index]` — a different index space — and
  `grep -c SagaChapter crates/engine/src/rules/abilities.rs` returns **0**. Empty requirements, no
  CR 603.3d announcement, `DeclaredTarget { index: 0 }` resolving at nothing. Filed as
  **`OOS-DX49-1`** with **no probe**, because a probe asserting today's behaviour would have to be
  inverted by whoever fixes it, and nothing this batch touched is on that path.
  **RIDER `OOS-RR4-3` CLOSED HONESTLY, AND THE LIVE HALF IS THE HALF A TOOL READS.** Each of its
  three findings was re-verified at HEAD *before* any document was touched. (i) **WITHDRAWN** — its
  own 2026-08-14 inversion stands and `corner-cases.md:468` is correct at HEAD; not edited.
  (ii) only **HALF** discharged: CLAUDE.md was fixed by `scutemob-212`, but the audit's own
  **Summary table** still read `32 / 0 / 4 / 0` — and that table, not the row census, is what
  `tools/tui/src/dashboard/parser.rs` machine-reads. (iii) live and fixed. **A fourth error in the
  same §36 that no document named** was also corrected, and it is the one a test would have been
  written from: the *"Entry order matters for retained abilities"* paragraph was wrong **on the CR
  itself** — CR 305.7 says verbatim *"this doesn't remove any abilities that were granted to the
  land by other effects"*, and Blood Moon has no Layer-6 effect at HEAD. Row 36 is **PARTIAL**, not
  COVERED; the card half stays gated on `urzas_saga` authoring (`OOS-RR4-2`), which this batch
  explicitly did not take.
  **Two standing gates fired on this batch's own work and both were answered, not weakened**: the
  ability-definition registry's `SagaChapter` site roster (four files → `saga.rs` + `resolution.rs`,
  which is the refactor's own success signal) and SR-25's `bare_lookup_ratchet` on `sba.rs`, whose
  ceiling was **lowered** 7 → 6 rather than left stale-high — *a stale-high ceiling is slack a
  regression hides in*. **And one claim in CLAUDE.md's own PB-DX48 narrative was refuted**:
  `KeywordAbility::Cloak` **does** exist (`types.rs:1696`); PB-DX48's conclusion and measurement
  both survive, but the stated reason was wrong, and a reason is the half the next batch reuses.
  **↻ The `/review` (2 MEDIUM / 1 LOW-MEDIUM / 4 LOW / 1 NIT — all eight taken, none declined)
  DEFEATED THREE OF THIS BATCH'S OWN CLAIMS BY EXECUTION, and one of them was printed in bold in
  production source.** *(1)* **"There is exactly one ability-blanking predicate in this tree" was
  TRUE AND UNENFORCED.** The reviewer appended a second hand-rolled predicate to `turn_actions.rs` —
  the exact pre-PB-DX43 shape whose 26-def regression this batch's own doc comment narrates — and
  **all 652 core tests stayed GREEN**. That is `OOS-DX49-6`'s own shape, a comment asserting a
  property the code does not enforce, inside the batch that filed it. Closed by `r7`, keyed on the
  mechanism and carrying a **second conjunct** that re-checks each allowlisted site's function body,
  because set equality cannot catch a predicate added *inside* an already-allowlisted function.
  **The finding's own prescribed needle was itself PB-DX47's defect** — the qualified
  `LayerModification::RemoveAllAbilities` is evaded by a `use` import — so `r7` keys on the bare name
  at word boundaries. *(2)* **The bench claim was refuted**; see the bench paragraph above. *(3)*
  **`saga.rs` claimed a seed that did not exist** (*"Stated residual (seeded…)"*, and `OOS-DX49-3` is
  a different residual) — filed as **`OOS-DX49-9`**. Also taken: `r6` walked one crate while
  `saga_view` is `pub` (a consumer planted in the simulator crate kept it green — PB-DX48's
  `SITE_SRCS` defeat one crate up; now a **workspace** walk, 14 roots / 148 files, with executing
  non-vacuity floors); `modification_blanks_abilities` could be widened silently for any zero-corpus
  variant (`SwitchPowerToughness` → `true` left the whole engine green while **`r3` stayed green
  too** — now `r8`, all **33** variants gated against the enum's own declaration); `r5b`'s
  4,000-byte window **was already over-scanning** by 520 and 1,116 bytes into the next arm, not
  merely at risk of it (now brace-matched and fail-closed — and the superseded window was proven to
  PASS on a planted call the new one catches); the `tools/tui` repair shipped untested; and a
  "countered" typo.
  Tests **4,941 / 0 / 5** (+41 over the 4,900 pre-edit baseline, **58** targets, itemised by NAME as
  41 additions / 0 removals / 0 leavers / 0 renames). **PROTOCOL 39 / HASH 78 both gate-executed and
  UNMOVED**, predicted in writing before any code. Coverage unmoved **63.1%**, **0 flips**, **0
  card-def edits**. All gates clean against the FINAL tree. Filed **OOS-DX49-1..9**. Full record:
  `memory/primitives/pb-DX49-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-02 — **PB-DX48 SHIPPED** (`scutemob-219`; v4 queue rank 6 —
  **OOS-ENG2-1** ≡ **OOS-ENG2-2** FILED *and* CLOSED, cross-cited; **OOS-ENG2-3** FILED and
  NARROWED. **None of the three had a registry row** — all were filed into ENG-2's handoff prose,
  which is the 61-of-208 blind spot the v4 re-rank measured).
  **Ward never fired on a triggered ability, and the reason the seeds give is not the reason.**
  **The census is EXACT, and that is the rare part.** Re-verified at HEAD by the inverse method —
  every `push_target_announcement` site minus every `PermanentTargeted` emitter, never by trusting
  either list — **12 = 3 emitters + 5 missing + 4 structurally target-free** (`targets: vec![]`, the
  `OOS-ENG2-3` free-cast sites, checked individually rather than inferred from the comments that
  claim it). After three consecutive batches in which the filed site list was a floor, this one
  reproduces without correction; the exception is reported because the discipline is only credible
  if it is.
  **EMITTING THE EVENT IS NECESSARY AND NOT SUFFICIENT, AND NEITHER SEED SAYS SO.** Both rows
  describe the fix as emitting `GameEvent::PermanentTargeted` at five more sites.
  `check_and_flush_triggers` ran `check_triggers` over a command's events and only THEN called
  `flush_pending_triggers`, so **the events a flush itself produced were fed back to nothing**: a
  Ward trigger caused by a *triggered* ability would have sat in `state.pending_triggers` until the
  next command — after priority had been granted, which CR 603.3b forbids. A batch that took the
  rows at their word ships a diff that looks exactly like a fix and moves **nothing** at the
  headline site.
  **THE DESIGN WAS WRONG TWICE, AND NEITHER CORRECTION CAME FROM ARGUMENT.** *(1)* A hook at
  `flush_sorted`'s tail — `Command::ChooseTriggerTargets` re-scans the very events it dispatched, so
  **Ward fired TWICE** (two `AbilityTriggered`, two ward stack objects, observed on a running
  probe). That is why every probe here asserts a **COUNT**: a `>= 1` assertion passes on the broken
  design. *(2)* The fixpoint in `check_and_flush_triggers` — green on the full suite AND on an
  end-to-end probe, and still short, because **`Command::PassPriority` never calls it** and
  `resolution.rs` sweeps before it flushes. So a targeted ETB trigger placed during a spell's
  *resolution* — the ordinary way one reaches the stack — still dispatched nothing. Measured both
  ways on a purpose-built probe: emission **1**, ward on stack **0**. Caught by enumerating the
  other five callers, not by a test; and the end-to-end probe was satisfied *because it drove the
  interactive `ChooseTriggerTargets` path*, i.e. **the probe was more interactive than the common
  case and that is what made it weaker.** A third uncovered path (`handle_concede` →
  `drop_departed_trigger_flush`, CR 800.4d, whose arm runs no sweep at all) was found by
  enumerating a third time.
  **Shipped shape**: `rules::events::permanent_targeted_events` derives the CR 702.21a payload
  **once** and `push_target_announcement` emits both halves, so all 12 sites dispatch and the
  **three hand-rolled loops are deleted** — a thirteenth site cannot omit Ward by forgetting to copy
  a loop; `abilities::dispatch_becomes_target_waves` is a bounded fixpoint with an **exactly-once
  scan cursor**, called from `flush_pending_triggers` and `handle_concede` and deliberately **not**
  from `resume_trigger_flush`, whose events are already swept. That asymmetry is `OOS-DX48-3`, and
  R-B **demonstrates** it: `c3` is the one channel probe that stays green under a single-wave
  revert, because its trigger suspends.
  **FIVE CORRECTIONS TO THIS BATCH'S OWN CENSUS, ALL FROM WALKING `all_cards()` INSTEAD OF A GREP**
  — SR-36's rule, broken by this batch's own brief **one batch after PB-DX47 filed `OOS-DX47-2` for
  the identical thing**. Ward-declaring population is **4**, not 5 (`vein_ripper` names the variant
  only inside a `// TODO` explaining why it cannot use it). `WhenBecomesTarget` has **1** structural
  declaration, not 6 (the other five are comment mentions). And **two LIVE finds where the brief
  said latent**: `KeywordAbility::Cloak` — *this narrative said the variant **does not exist**, and
  **that is false at HEAD**; it is a unit variant at `card-types/src/state/types.rs:1696`,
  discriminant 157, beside `KeywordAbility::Manifest` at `:1689`. Corrected in place 2026-09-03 by
  PB-DX49 (`scutemob-220`), whose `r5c` proves the discrimination on synthetic input. **PB-DX48's
  CONCLUSION survives and its measurement was right** — zero corpus defs declare either marker, so
  the grep's zero was the true population; what was wrong is the stated REASON, which is the more
  dangerous half, because a reason is what the next batch reuses.* Cloak is reached through
  `Effect::Cloak`, so a grep for the keyword
  measured zero and read like a measurement — `cryptic_coat` is `Complete`, deck-legal, and its ETB
  Cloak puts a face-down permanent on the battlefield that the layer walk gives ward {2} **and no
  Ward triggered ability** (`OOS-DX48-4`, LIVE not latent); and an **INVERSE oracle-text axis**
  found **`brutal_cathar`**, `Complete` and deck-legal, whose back face prints *"Ward—Pay 3 life"*
  with no Ward mechanism authored and an in-source `// DSL gap` note saying so (`OOS-DX48-7`). The
  three deck-legal `Complete` Ward defs the rank rested on reproduce **exactly**.
  **BOTH DELEGATED REVERT MATRICES WERE RE-EXECUTED RATHER THAN ACCEPTED, AND ONE WAS WRONG.** The
  channel suite reported "3/3 RED" — true — while every probe panicked on the **journal** assertion
  its own comment labels *"corroboration, not the verdict"*; `damage_marked == 0`, the resolution
  effect AC 7252 asks for, stayed **TRUE** under the revert, because the drive ran past **CR 514.2's
  Cleanup**, which erases damage either way. Repaired to stop the instant the trigger chain settles
  and to assert that settlement as a precondition; re-executed, all three now fail on the damage
  assertion and `c2` reports `left: 1, right: 0`. **"All rows RED" is a true sentence the wrong
  assertion can produce**, and the check that separates them costs one command: read the panic LINE.
  **AC 7252's "ward cost paid" branch is UNREACHABLE at HEAD and is reported, not narrowed.**
  `Effect::MayPayOrElse` discards its `cost` and `payer` and always applies `or_else`. Blocker read
  off the source: `EffectChoiceQuestion::PayOptionalCost`'s payload cannot distinguish a
  `MayPayOrElse` ask from a `MayPayThenEffect` one, and its default is a hard `pay: true` under a
  comment already calling the alternative *"a different batch"*. **Zero deck-legal `Complete` card
  defs use the variant** — Ward is its only live consumer — so the fix is bounded but needs a wire
  bump this batch's own gates pin as unmoved (`OOS-DX48-2`). What is exercised instead is
  CR 702.21a's own two-sided discrimination: Ward fires once for an **opponent's** ability and not
  at all for its own controller's.
  **↻ The `/review` (2 MEDIUM engine-or-gate, 2 MEDIUM doc, 3 LOW, 2 NIT — all nine taken, eight
  fixed and one declined with its reason) DEFEATED THREE OF THIS BATCH'S OWN GATES BY EXECUTION,
  and found a real dispatch hole the shipped engine still had.** *(1)* `dispatch_becomes_target_waves`
  tested suspension at the TOP of its loop, so a batch's **prefix** — the members placed before it
  suspended — had its `PermanentTargeted` events dropped by everything: callers scan before the
  flush, and `ChooseTriggerTargets` sweeps only the RESUMED events. **The loop's own comment
  asserted the resumed call covered it, and it was false in both halves** — a false comment inside
  the batch whose subject is a false comment. Fixed by the ORDER (queue, then stop); `t9` pins it
  and is RED under the first draft with the `PermanentTargeted` assertion staying GREEN, which is
  the whole reason the ward-trigger COUNT is the verdict. `OOS-DX48-3` also had its precondition
  wrong: **two** triggers, one asking — not three. *(2)* **`r2` fell to FIELD ORDER**: it keyed on
  the token after the brace being `target_id:`, and Rust does not constrain field order, so a real
  second construction written in another order stayed green — the docstring named only the residual
  it *had* thought of and called it "measured". *(3)* **`r1` fell twice**: its site was a
  `BTreeSet<(file, func, marker)>`, so a **duplicated** call inside a marked site collapsed into one
  element — and a duplicated announcement IS the Ward-fires-twice defect this batch rejected, so the
  gate was blind to its own headline; and its file list was six hardcoded `rules/` files while
  `push_target_announcement` is `pub(crate)`, which mattered concretely because `OOS-DX48-6` names
  `effects/mod.rs` as the next dispatch site and the list did not contain it. Both re-keyed on the
  mechanism, `SITE_SRCS` deleted, every defeat re-run and now RED. *(4)* The v4 memo's row-6 strike
  still said the movement budget "did NOT come due"; it was written before the fuzz A/B ran and
  never re-taken — PB-DX45's own MEDIUM, and it matters because the memo is what the next
  dispatcher reads.
  Tests **4,900 / 0 / 5** (+27 over the 4,873 pre-edit baseline, **57** targets, itemised by NAME as
  27 additions / 0 removals / 0 leavers — with the ENG-2 pin's **in-place** inversion disclosed so
  that "0 leavers" is not read as "nothing was touched"). **PROTOCOL 39 / HASH 78 both
  gate-executed and UNMOVED**, predicted in writing before any code. Coverage unmoved **63.1%**,
  **0 flips**, **0 card-def edits**. All gates clean against the FINAL tree. Filed
  **OOS-DX48-1..7**. Full record: `memory/primitives/pb-DX48-execution-notes.md`; handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-09-02 — **PB-DX47 SHIPPED** (`scutemob-218`; v4 queue rank 5 —
  **OOS-DX24-4** CLOSED, with four corrections to its own claims recorded in the row).
  **A probe-first batch whose probe came back with the LARGE answer: the double-push is REAL,
  and it was live on 18 deck-legal `Complete` defs.**
  **The experiment ran first and it is the headline.** The seed was filed MEDIUM confidence, the
  v4 memo explicitly blessed the small outcome ("if a dedup exists, the batch collapses to a
  comment fix"), and the brief demanded the measurement come before any design. It did, and it was
  committed before a line of engine source changed (`bb5a2f8e`).
  `crates/simulator/tests/pb_dx47_double_push_probe.rs` builds through
  `setup::build_initial_state` — the **production** pregame path, deliberately not
  `GameStateBuilder`, because the false comment under test claims the hand-built path is the
  special one, so a hand-built fixture would have proven nothing. Both seats human, no bot RNG;
  subject `drana_liberator_of_malakir`, `Complete` and deck-legal and **legendary**, so CR 903.6
  puts it in the command zone by construction rather than leaving the probe to a shuffle. Result:
  the engine's own `check_triggers` pushed **`{CardDefETB: 1, Normal: 1}`** for one
  `CombatDamageDealt`, and a card printing **ONE** `+1/+1` counter put **TWO** on its lone
  attacker.
  **The justifying comment was false in TWO ways, not one, and the memo only caught one.**
  `abilities.rs` said the CardDef ability is *"not converted to runtime `TriggeredAbilityDef` (that
  only happens in `enrich_spec_from_def` for tests)"*. The memo's correction — that
  `enrich_spec_from_def` IS the production pregame path (`setup.rs:419/433/440`,
  `fuzz_setup.rs:119/130`) — is right and is the second half. The first half is that
  **`build_face_ability_vectors` has a dedicated loop converting exactly this `TriggerCondition`**,
  and PB-DX1 *extended* that loop (`intervening_if` propagation) without anyone reconciling it with
  the comment. **The same sentence was copy-pasted onto the `WhenExertedAsAttacks` arm, which cites
  this one as precedent** — and there its CONCLUSION is correct (that condition has no lowering
  loop) while its stated general PREMISE is false, which is worse than a wrong conclusion because
  it reads as confirmation (`OOS-DX47-6`). Both corrected.
  **The fix is one deletion, and the survivor was chosen on CR grounds rather than incumbency.**
  The card-registry scan is deleted; the layer-resolved runtime lowering is the single dispatcher.
  It is the CR-correct one of the two on three axes, each a place the scan was *also* wrong:
  `collect_triggers_for_event` reads layer-resolved characteristics (**CR 613.1f**, so Humility /
  Dress Down / any `RemoveAllAbilities` suppresses the trigger, where a raw registry scan bypasses
  layers entirely); it sees granted and copied abilities; and it sees **tokens**, which carry no
  `card_id` for a registry scan to find.
  **The scan's own historical justification is DISCHARGED BY EXECUTION.** PB-EF3 A2 / EF-W-MISS-10
  said `CardDefETB` had to stay authoritative so Throat Slitter's declared `targets` survive
  auto-target selection. The lowering forwards `targets` verbatim, `flush_sorted` reads them for a
  `Normal` trigger, and `pbd_damaged_player_filter`'s end-to-end Throat Slitter probe **passes** —
  once its fixture stops building a **NAKED object**
  (`ObjectSpec::creature(..).with_card_id(..)`, never enriched), which is a shape no production
  path can produce and which is the only reason that probe had ever exercised the scan
  (`OOS-DX47-4`, *with its unmeasured half stated*: how many other tests are green against that
  shape is UNKNOWN).
  **The CLASS is swept mechanically, because the defect is "two dispatchers", not "this event".**
  `r3_no_trigger_condition_has_two_dispatchers` intersects the **34** `TriggerCondition`s the
  lowering converts with the **6** the `abilities.rs` queue sites registry-scan — both sets parsed
  from source rather than hand-listed, for `OOS-DX24-4`'s own reason, that **a hand-listed set is a
  claim and this defect survived five months behind exactly such a claim written as a comment**.
  The only intersection member is `WheneverYouSacrifice`, allowlisted with the reason stated (its
  occurrence is a `triggers.retain(..)` POST-FILTER, never a second push) and proven load-bearing
  by an executed revert.
  **PB-DX24's own Q4 probe was a PIN ON this defect and its docstring said so** (`OOS-DX47-5`): it
  filtered by `PendingTriggerKind::CardDefETB` *because* the trigger "is ALSO lowered into the
  runtime Channel-A vector", so an end-to-end assertion "would be satisfied by Channel A alone".
  The durable rule is not that PB-DX24 was careless — isolating the path you are changing is
  correct technique — it is that **the isolation becomes a pin on a defect the moment it is the
  only thing asserting the count.**
  **↻ The `/review` (1 MEDIUM / 1 LOW-MEDIUM / 4 LOW / 1 NIT — all seven taken, none declined)
  DEFEATED this batch's own class gate by execution, and the defeat is this batch's thesis
  committed inside the gate that states it.** `r3`'s first draft keyed on ONE syntactic form —
  `trigger_condition:` immediately followed by `TriggerCondition::X`. The reviewer re-created
  **`OOS-DX24-4` verbatim**, a second `WhenDealsCombatDamageToPlayer` dispatcher written in the
  BINDING form (`let ... { trigger_condition, .. }` then `matches!`), and **all nine gates in the
  file stayed GREEN**; only the behavioural probe reddened. Reproduced here before fixing. And the
  form is not contrived — `collect_graveyard_carddef_triggers` in the same file is a real registry
  scan written that way, filtering two conditions that ARE in the lowered 34, so the header's *"a
  second `OOS-DX24-4` is now a red test"* was false about the very family it could not see.
  *A gate written for one variant measures that variant*, for the fourth time in this queue
  (PB-DX26, PB-DX43, PB-DX45, now here). **The axis is re-keyed on the MECHANISM**: a registry scan
  must walk an ability list, so every `TriggerCondition::X` within 3,000 bytes of an
  `effective_abilities(` / `abilities.iter()` hit is collected, across **five** `rules/` files
  rather than one (6 → **17** conditions). Over-collection can only make `r3` redder, and each of
  the three resulting false positives is named with the mechanism that separates it **plus a
  companion assertion that the mechanism still exists in source** — an allowlist whose reason is
  not checked is a comment, which is what started this batch. Also taken: the registry axis read
  only `abilities.rs` while `mana.rs`/`turn_actions.rs`/`replacement.rs`/`resolution.rs` also queue
  from the registry; `r2`'s ratchet ceiling was **2× its measurement** under a comment claiming *"it
  cannot grow in silence"* (**a ratchet's slack IS its blind spot** — 40 → 22); the superset table
  omitted the one axis where the deleted scan was **wider** (CR 113.7a LKI — narrower is CR-correct
  here, and it now says so); `OOS-DX47-4`'s "population UNMEASURED" was cheap and is now measured
  (**247** test files / **1,619** `.with_card_id(` sites, **149** files never calling
  `enrich_spec_from_def` — an upper bound and a search space, not a work list); and a fixed-width
  byte slice in `r5` that would have **panicked** rather than failed with its own message.
  Census re-derived and PRINTED (never transcribed): **26** corpus defs declare the trigger, **18**
  deck-legal `Complete`. **The v4 memo's conditional "18 if real" reproduces EXACTLY**, which is
  the outcome the re-derivation discipline is FOR; the agreement is kept honest by the inverse
  oracle-text axis, where the two do not agree (**20** `Complete` defs print the trigger without
  declaring it, ratcheted).
  Tests **4,873 / 0 / 5** (+12 over the 4,861 pre-edit baseline, **56** targets, itemised by NAME
  as 13 additions / 1 disclosed inversion / 0 removals). **PROTOCOL 39 / HASH 78 both
  gate-executed and UNMOVED**, predicted in writing before any code with the reason stated.
  Coverage unmoved **63.1%**, 0 flips, 0 card-def edits. All gates clean against the FINAL tree.
  **10 revert rows, 10 RED.** Filed **OOS-DX47-1..7**. Full record:
  `memory/primitives/pb-DX47-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-09-02 — **PB-DX45 SHIPPED** (`scutemob-217`; v4 queue rank 4 —
  **OOS-DX24-9** ≡ **OOS-DX27-5** CLOSED as ONE defect, cross-cited, each row corrected).
  **CR 118.12 makes an optional cost a player decision, and the engine was making it — at TWO
  sites, one of which no document in the chain names.**
  **The headline is a site list, and the reason nobody noticed it.** `effects/mod.rs` has **two**
  callers of `try_pay_optional_cost`: the `Effect::MayPayThenEffect` arm both registry rows name,
  and `Effect::LookAtTopThenPlace`'s `place_cost` (`:6365`), which is the identical CR 118.12
  decision one function over and was live on a deck-legal `Complete` def — `birthing_ritual`, whose
  auto-paid sacrifice also parameterises the mana-value cap on what it may then cheat onto the
  battlefield. Both now suspend into one `EffectChoiceQuestion::PayOptionalCost` on PB-DP9's
  shipped CR 608.2d channel. The scope line is stated rather than inferred: **PB-DX45 repairs every
  caller of `try_pay_optional_cost`, not every printed "you may pay"** — which is what puts the
  second site IN and three never-charged `Complete` defs OUT (`OOS-DX45-3`).
  **THREE PUBLISHED FIGURES DID NOT REPRODUCE, and one of them was offered as a proof.** (1) The
  v4 memo's **11** deck-legal `Complete` defs is **10** — re-derived at HEAD by two independent
  routes, with no member's marker having moved since *before* the memo's census closed. §1d offered
  *"two independent measurements both returned 11"* as the PROOF that the two rows are one defect.
  They are one defect; the evidence was two agreeing wrong numbers (`OOS-DX45-2`). Six batches have
  taught this queue that a member list is a FLOOR — **this is the first recorded OVER-count**, and
  the correction is that a census figure is an estimate in BOTH directions. (2) `OOS-DX27-5` says
  PB-DX27 left *two* defs `partial` "on the same shape"; only `vampire_gourmand`'s marker cites
  this deviation, `ruthless_technomancer`'s cites its **activated** ability's missing variable-X
  sacrifice cost. So the policy re-adjudication is **ONE flip, not two** — a batch taking the row
  at its word would have promoted a def whose real blocker is live. (3) `MOVED_MSG` predicts five
  named sibling gates "will redden alongside" a `CORPUS_COMPLETE` move; **none did**. Exactly one
  seeded pin in the workspace moved (`UI3_SPLIT_COMBAT_SEED` 32 → 13, re-observed by an executed
  sweep). PB-DX26's lesson runs both ways.
  **THE DEFECT THIS BATCH SHIPPED, AND THE OBLIGATION IT ADDED.** `play-server`'s
  `api::validate_decision_params` matched `(question, answer)` with a trailing
  `_ => Err("… a different kind")` — **a wildcard written to mean *wrong question* silently also
  serving as the fallback for *unknown question***. So every legal `PayOptionalCost` answer 400'd
  and the browser was offered a `Confirm` picker whose Confirm **and** Decline buttons both failed:
  a clean offer followed by a guaranteed refusal, the SR-38 shape PB-DX29 gated Fuse to avoid and
  PB-DX44 recreated while fixing it — **the third instance**. Eight consumers had to learn the new
  variant; **seven were compile errors and the eighth was the one that broke.** Fixed structurally
  (dispatch on `question` alone, exhaustive, no wildcard), and `rules/engine.rs`'s obligation list
  gains **obligation (8)**: *a wildcard arm that encodes a JUDGEMENT cannot also be the fallback for
  the UNKNOWN, and seven compile-forced sites are not evidence the eighth is safe — they are the
  reason nobody looks for it.*
  **Reachability proven with a NON-DEFAULT answer through all three channels, asserted by
  RESOLUTION EFFECT rather than by the offer** — `nether_traitor`'s `{B}` declined and accepted
  through `LocalGame`/`HumanChoice` (the human taps a real Swamp; the two probes differ in exactly
  one bool and land the Traitor in the GRAVEYARD with the mana still floating, or on the
  BATTLEFIELD with it spent), through genuine `POST /api/game/action`, and through the bot path
  (`StubProvider` needed no change, asserted rather than assumed). The decline is a state the old
  engine could not produce from any channel, which is exactly why an offer-shaped assertion would
  have been worthless. **One disclosure the `/review` asked for**: the HTTP pair drives
  `birthing_ritual`'s `Cost::Sacrifice` at the SECOND site, not `nether_traitor`'s `{B}` — a
  play-server session installs from a DECK and cannot be asked for a Traitor in a graveyard with a
  creature dying on it. More coverage than the criterion asked for, and not the coverage it named;
  the untested combination is (site 1 × HTTP transport) alone, whose engine path is the very
  `LocalGame::submit` the channel probes drive.
  **`default_effect_choice_answer` returns `pay: true` deliberately** — the exact recovery of the
  pre-batch auto-pay, which is what keeps every bot game, the fuzzer and every pre-existing golden
  script behaviourally identical while only the command trace grows.
  Tests **4,861 / 0 / 5** (+26 over the 4,835 pre-edit baseline, **55** targets, itemised by NAME
  as 26 additions / 0 leavers / 0 removals). **PROTOCOL 39 / HASH 78**, one bump each, predicted in
  writing before any code. Coverage **63.0% → 63.1%**, one named flip. All gates clean against the
  FINAL tree. Filed **OOS-DX45-1..8**. Full record:
  `memory/primitives/pb-DX45-execution-notes.md`; ruling: `memory/decisions.md`; handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-08-23 — **PB-DX15a SHIPPED** (`scutemob-216`; v4 queue rank 3 —
  **OOS-DP9-8** and **OOS-DP9-11** both CLOSED). **Two CR violations that 4,797 tests could not
  see, and a pin that pinned nothing.**
  **The headline is one fact that explains three separate things.** `OOS-DP9-8`'s row said its
  deviation was *"pinned as the engine's actual behaviour"* by
  `test_dp9_choice_inside_for_each_each_player`. It was not, and could not be: that test ran on a
  two-seat fixture with `.active_player(p(1))`, and APNAP (CR 101.4 — active player, then the rest
  in turn order) starting from the **lowest** `PlayerId` over an ascending `turn_order` **is**
  ascending `PlayerId`, because rotating a list to start at its first element is the identity. The
  assertion `vec![p(1), p(2)]` was green under either rule. That single fact explains (i) why the
  seed survived five months and eleven batches behind a test claiming to hold it, (ii) why the v4
  memo's wire cell — *"golden scripts and SR-9b per-step fingerprints move; budget the re-pin"* —
  was **wrong**, since every fixture in the tree makes the same choice and the reorder is invisible
  to all of them, and (iii) why this batch had **no inherited red-before evidence anywhere** and
  every probe had to earn its own revert. Now stated **structurally**:
  `test_dx15a_active_lowest_id_makes_apnap_and_ascending_indistinguishable` asserts the coincidence
  over 2..=6 seats plus the contrasting non-vacuous case.
  **Both filed populations were floors and both were mis-framed.** `OOS-DP9-11`: the five named
  defs all reproduce and are **one of FOUR mechanisms** — measured **17** deck-legal `Complete`,
  the others being every `SearchLibrary`-to-library tutor (8), Hideaway (1) and PartnerWith (3).
  The two census axes **do not nest** (an oracle-text axis sees only the first family; a structural
  `Effect`-payload axis cannot see the keyword families at all) — PB-DX26's and PB-DX43's lesson a
  third time. Two further corrections: **`chaos_warp`, one of the row's own five, reaches the
  `Library{Top}` branch**, not the bottom helper the row is filed against; and **PartnerWith's blast
  radius is the whole library** — every id moved to the bottom in turn, so a 99-card library minted
  99 `ObjectId`s and burned 99 `timestamp_counter` values per ETB, unconditionally.
  `OOS-DP9-8`: the memo's *"repairs the Fleshbag / Grave Pact family (10 defs)"* reads as ten defs
  regaining a choice; **that family makes no per-player choice at all** —
  `sacrifice_permanents_for_player` sorts and takes the first `n` — so what is repaired is the
  ORDER the sacrifices happen. Only **2** deck-legal `Complete` defs exercise the literal
  question-order claim. Filed as `OOS-DX15a-2` so the family is not treated as closed.
  **The same-zone fix is deliberately not the sweep the row asks for.** `Effect::MoveZone` and
  `Effect::PutOnLibrary` resolve their destination **at runtime**, so "is this call same-zone" is
  not a property of any call site; the guard lives inside both `GameState` move helpers, which
  makes a renumbering same-zone move **unrepresentable**. **One existing test was a pin ON the
  defect**: `test_400_7_same_zone_move_produces_new_id` asserted `assert_ne!` because *"the
  zone-change event creates a new object regardless of the source and destination zones being the
  same"* — which **inverts CR 400.7**, whose antecedent is *"moves from one zone to another"*. That
  test is why the seed stayed open: a helper-level fix reddened it, so every earlier reader
  concluded the helper was right.
  **Both riders' prescribed fixes were wrong as written, and both were settled by executing them
  rather than by argument.** `OOS-DX24-1` **DEFERRED**: its "one source-zone conjunct" would break
  Teysa Karlov's doubling of a look-back dies trigger, because such a trigger is built as
  `PendingTrigger::blank(*new_grave_id, ..)` — **its source is a graveyard object too**, so zone
  alone cannot separate the legitimate case from the defect. `trigger_doubling.rs` had **nine**
  tests and **none** touched the `CreatureDeath` arm, so the missing probe was written first,
  confirmed green, and the conjunct then applied verbatim → **`left: 1, right: 2` with all nine
  still green**. `OOS-DX24-7` **TAKEN**: its "rebuild the set per event prefix" (a) makes `sba.rs`
  wrong — in one CR 704.3 fixpoint pass the deaths *are* simultaneous, which is the Gatherer ruling
  the function already quotes — and (b) has the direction backwards, since the set is a
  **suppression** set and the prefix is what to **subtract**; passing it reproduces the very defect
  the row describes. Shipped as `EventBatchTiming` + the complement, with the four call sites
  PB-DX24 recorded as unaudited passing byte-identical behaviour under a comment saying so.
  **Also measured rather than asserted**: CR 701.23i does **not** require simultaneous movement
  (only CR 701.22c does), and `Effect::Scry`'s per-player move sets are pairwise **disjoint by
  construction**, so the ask-then-move loop is observationally simultaneous — asserted directly and
  wrong-way-round instead of restructuring for a difference no observer can make.
  Tests **4,835 / 0 / 5** (+38 over the 4,797 pre-edit baseline, **54** targets), delta itemised by
  NAME as **42 additions / 4 leaving / 0 removals** with all four disclosed (2 inversions, 2
  doctest line-number shifts). **PROTOCOL 38 / HASH 77 both gate-executed and UNMOVED**, predicted
  in writing before any code. Coverage unmoved **1,136/1,803 = 63.0%**, **0 flips**, churn
  reverted, **1 comment-only card-def edit**. `clippy --workspace --all-targets -D warnings`,
  `cargo fmt --check` and `tools/check-defs-fmt.sh` (1,803 defs) all clean, against the FINAL
  tree. Filed **OOS-DX15a-1..7**.
  **↻ The `/review` (1 HIGH / 4 MEDIUM / 5 LOW, all ten taken) inverted BOTH rider verdicts
  above, so read them as first drafts.** The HIGH was a regression the batch introduced with its
  own argument applied to a case it missed: `Effect::DestroyAll` destroys in ONE loop, so a wrath's
  deaths are simultaneous and `resolution.rs` cannot be declared `Sequential` — `nether_traitor`
  (`Complete`, deck-legal) fired its graveyard ability off a creature that died at the same instant
  (**21** corpus board wipes). `resolution.rs` reverted byte-identical to `Simultaneous` and
  **`OOS-DX24-7` is RE-OPENED**: premise intact, granularity refuted — the unit is a simultaneous
  GROUP and one resolution holds both kinds, so closing it needs group boundaries in the event
  stream; `t5` pins the wrath case wrong-way-round. And `OOS-DX24-1`'s deferral reason was
  factually wrong — a wire-neutral discriminator (the triggering EVENT) was already passed to the
  function, and the split is total because the battlefield-sourced `AnyCreatureDies` collector
  filters on `zone == Battlefield` — so **`OOS-DX24-1` is CLOSED** with a two-probe pair that only
  a correct implementation satisfies together. **The fix cycle itself introduced FIVE failures the
  batch did not catch** (it ran targeted tests, not the suite); three were standing gates firing a
  second time, all correctly. A fix cycle is a change like any other: gates run on the final tree.
  Full record: `memory/primitives/pb-DX15a-execution-notes.md`; handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-08-15 — **PB-DX44 SHIPPED** (`scutemob-215`; v4 queue rank 2 —
  **OOS-DX29-9**, **OOS-DX29-12** and **OOS-DX29-14** CLOSED, **OOS-DX29-3** NARROWED).
  **Seven deck-legal `Complete` defs could not be cast as printed, and one could not be cast at
  all.** Four halves, one wire bump.
  **The Spree half is PB-DX29's own headline one enum over.** `casting.rs` charges
  `ModeSelection.mode_costs`; `legal_actions::effective_cast_cost_with_additional` — the function
  `LocalGame::auto_tap_commands_for` asks how much mana to tap — modelled none, so
  `insatiable_avarice` was `InsufficientMana` from **every** channel. **The fix is not the
  arithmetic, it is the argument**: `auto_tap_commands_for` must pass `&cast.modes_chosen`
  *verbatim off the `Command` it is about to apply*, which is the same value for the human path
  and the bot path. `&[]` there is the obvious first draft and leaves the defect alive on both —
  proven, that one substitution reddens three end-to-end probes. The mirror also needed a clause
  the seed never mentions (entwine charges **every** mode, not the chosen ones).
  **The fuse half shipped an SR-38 defect in its first draft and the coordinator caught it.**
  Concatenating CR 702.102d's targets in the shared `card_def_target_requirements` is correct and
  **deleting the offer suppression is not the same as making the offer honest**: `view.rs` passed
  `fuse: false` and `ActionBar`'s stage order is `ValuePrompt → CostPicker → TargetPicker`, so a
  human ticked Fuse and was then asked for **one** target while the engine demanded **two**. A
  clean offer followed by a guaranteed 422 — exactly what PB-DX29 gated the offer to avoid,
  recreated by the batch fixing it. The probe that missed it compared `fuse: true` against
  `fuse: false` on the query: **both assertions true, neither about the channel.**
  **The half selector is `AltCostKind::SplitRightHalf`, not a 16th `CastSpellData` field** —
  that struct has no `Default` and **793** sites list every field, and `AltCostKind` already
  carries `Aftermath`, literally *"cast the other half of a split card"*. **Its risk was never the
  cost arm**: a right half declares a **globally offset** `DeclaredTarget` index, correct only for
  a fused cast, so cast alone it resolves **at nothing** — silent wrong game state, not a refusal.
  `resolution.rs` pads the effect **context** by the left half's declared count, after the
  `is_target_legal` filter and never on `stack_obj.targets` (which CR 608.2b fizzling reads and
  `TargetsAnnounced` publishes). **Population is 3, not the seed's 2**: `connive_concoct`'s right
  half is as uncastable as Burn and Tear, it merely cannot fuse.
  **The pitch half was one line.** `params.rs` hard-coded `alt_cost: None`, so `casting.rs`'s
  pitch payment path — shipped in PB-AC5 — was unreachable by construction. Two affordability
  traps had to be dodged: a pitch cast costs `{0}` and Force of Will's whole point is casting it
  when you *cannot* afford `{3}{U}{U}`, and `Turn // Burn`'s printed cost is the LEFT half's.
  **PROTOCOL 37 → 38 / HASH 76 → 77**, both gate-computed and **both predicted in writing before
  any code changed** — the v4 memo's cell predicted PROTOCOL only and was short by the HASH half.
  **The batch corrected itself four times by execution, which is the durable half.** Its census
  asserted pitch = 5 from a **source grep**; the `all_cards()` walk refuted it (`force_of_despair`
  mentions `AltCostKind::Pitch` in a *comment*) — SR-36's exact failure, inside the census written
  to obey SR-36. `OOS-DX29-13`'s own prescribed fix (assert `card_name_to_id(name) == card_id`)
  **fails on 50 defs in four classes**, so it ships as a pinned floor and the row's prescription is
  corrected. A probe doc claimed Misdirection was "the only pitch member with no life component";
  making life mandatory reddened **four** tests — `force_of_will` is the only one that *pays* it.
  And `OOS-DX44-4`'s first draft said "a **fused** spell's target indices shift", until the
  ordinary cast path showed the identical `filter`-then-positional-`get`: **where a defect is
  noticed is not where it lives**, and the measured candidate population is **7** deck-legal
  `Complete` defs, not 2.
  Tests **4,797 / 0 / 5** (+44 over the 4,753 pre-edit baseline, **53** targets), itemised by NAME:
  **45 additions, 1 rename, 0 removals** — the rename (`p1e`) is the one the criterion mandated and
  is stated rather than netted out. Coverage unmoved **1,136/1,803 = 63.0%**, **0 flips**, churn
  reverted. `clippy --workspace --all-targets -D warnings`, `cargo fmt --check` and
  `tools/check-defs-fmt.sh` (1,803 defs) all clean. Filed **OOS-DX44-1..5**. Full record:
  `memory/primitives/pb-DX44-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-08-14 — **PB-DX43 SHIPPED** (`scutemob-213`; v4 queue rank 1 —
  **OOS-DX27-1** and **OOS-DX27-10** both CLOSED). **A rule the engine had never derived, on cards
  that print no text for it.** CR 305.6 gives any object with the land card type and a basic land
  type the intrinsic `{T}: Add [symbol]`; `Characteristics.mana_abilities` was written from four
  kinds of site and **none read `chars.subtypes`**, so three deck-legal `Complete` format staples
  under-delivered their entire printed text in the shipped browser game — every land under
  `urborg_tomb_of_yawgmoth` produced nothing, likewise `yavimaya_cradle_of_growth` and
  `dryad_of_the_ilysian_grove`. New `rules::layers::derive_intrinsic_land_mana_abilities` runs at
  the **end of the layer-4 iteration**, reading the fully resolved subtype set.
  **The layer placement is the batch, and it forced a second fix the brief did not scope.**
  CR 305.6's intrinsic is a consequence of the type change (CR 613.1d), so a layer-6 removal must
  still be able to strip it (CR 613.1f, pinned by `p9`). That reading makes "delete the moons'
  mana grant" — literally what the criterion asked — **strictly worse than HEAD**: each moon's own
  layer-6 `RemoveAllAbilities` would have wiped the layer-4 derived ability and **Blood Moon would
  have stopped working entirely**. So CR 305.7's ability-LOSS half moved into the `SetLandTypes`
  primitive, conditioned on its payload containing a basic land type (CR 305.7's own precondition;
  `p13` proves a `Gate` payload triggers neither), and each moon dropped **two** statics, not one.
  **That relocation closes a second, unfiled CR violation**: CR 305.7's last sentence forbids
  removing abilities *granted by other effects*, and a blanket layer-6 removal is timestamp-ordered
  against every other layer-6 effect, so it could strip an earlier grant from Cryptolith Rite,
  Chromatic Lantern, The World Tree, Bootleggers' Stash or Wrenn and Realmbreaker (`p7`).
  **The memo's census reproduced exactly and was still a floor short by three.** Its published rule
  scans `LayerModification` payloads; a token confers through a `TokenSpec`, which that rule
  structurally cannot see. An inverse axis over printed text found **`awaken_the_woods`** — a
  **fourth live-wrong `Complete` def**, its "Forest Dryad land" token declaring
  `mana_abilities: vec![]`, fixed for free — **`overlord_of_the_hauntwoods`** (a **third**
  double-grant risk, its Everywhere token hand-authoring all five subtypes *and* all five
  abilities, proven to resolve to 5 and not 10) and `leyline_of_the_guildpact` (`Inert`). The class
  is **8 defs, not 5**. PB-DX26's lesson again: *a roster derived from one declaration construct
  measures that construct* — both axes are now standing roster rows.
  **The basics question was decided by a gate, not by taste**: basics KEEP their printed
  `{T}: Add` and the derivation is idempotent instead, because
  `every_complete_land_registers_each_printed_tap_mana_color` reads the **registry** lowering and
  not the layer walk, and because `Command::TapForMana.ability_index` is a dense index that
  deletion would move on the commonest object in the game (**OOS-DX26-3**). Index neutrality proven
  across all **46** `Complete` defs printing a basic land subtype.
  **Reachability was proven, not assumed** — the `kaito_shizuki` lesson that existence is never
  sufficiency. 8 probes drive the human `LocalGame`/`HumanChoice` channel, the offer layer and the
  mana solver, asserting mana in a pool or a `Command` the solver emitted. **Two fixture defects
  surfaced there and both are filed**: `GameStateBuilder::build()` registers no static continuous
  effects, so a conferring permanent placed straight on the battlefield confers **nothing** and the
  first draft failed *for a reason it did not describe* (`OOS-DX43-6`); and an offer-layer
  assertion about a non-priority player is **structurally vacuous**, since `StubProvider` returns
  an empty list for them — written as a `== 0` expectation it would have passed forever
  (`OOS-DX43-7`).
  **The `/review` found 1 HIGH / 4 MEDIUM / 8 LOW and all 13 were taken — and the HIGH was this
  batch committing its own headline lesson.** *A gate written for one variant measures that
  variant*: `replacement.rs`'s IG-1 ETB-trigger suppressor asked "are this permanent's abilities
  blanked?" by matching one literal variant (`EffectLayer::Ability` + `RemoveAllAbilities`), which
  was correct while that was the only blanking channel. This batch added a **second** channel and
  deleted the moons' Layer-6 static, so the scan stopped seeing them and **26** `Complete` nonbasic
  land defs — the ten Karoos, the six Temples, the five gain-lands — began firing CardDef ETB
  triggers off a land with no abilities, **with all 4,749 tests green**. Fixed structurally, not
  locally: new `rules::layers::modification_blanks_abilities`, exhaustive over all 33
  `LayerModification` variants with **no wildcard arm**, and IG-1's layer filter **deleted** rather
  than widened — keying on the modification is what makes a third channel impossible to add
  silently. **The exhaustiveness earned its keep on its first compile**: the first draft's variant
  list was short by three and the compiler refused it, where a `matches!` would have accepted the
  same short list in silence. The reviewer also **deleted four of the five CR 305.7 clearing lines
  and the entire workspace stayed green** (now `f2`, and the wider `rules` target reddens 7 tests);
  found **three claims this batch invalidated in registry rows filed the day before**, including
  `OOS-RR4-3`'s finding (i), whose correction of `corner-cases.md:468` **has inverted** — the cite
  it rested on no longer exists and the sentence it called wrong is now right; and caught **a false
  claim in the batch's own execution notes** ("matches the plan's stated figure exactly" — the plan
  states no such figure). Also taken: one `clear_all_abilities` with an exhaustive destructure
  replacing two hand-written copies; two **dead** non-vacuity floors deleted (unreachable behind
  exact-set `assert_eq!`s); `TOKEN_SPEC_FIELDS` gated against the struct declaration, applying the
  repair `OOS-DX28-1` recommends and this batch had reused the fragile construct without; P8's
  UNDISCRIMINATED status disclosed in the test itself; benches measured; and the plan deliverable
  the implement phase silently dropped (updating `t6`'s doc) taken rather than reframed as a
  decision.
  Tests **4,753 / 0 / 5** (+32 over the 4,721 pre-edit baseline, itemised by NAME, **0 removals**,
  50 targets); coverage unmoved **1,136/1,803 = 63.0%**, **0 flips**, proven by regeneration;
  **PROTOCOL 37 / HASH 76 both gate-executed and UNMOVED**, as the plan predicted in writing before
  any code change. `crates/view-model` and `crates/simulator/src` are **0 lines** — every consumer
  already read layer-resolved characteristics. One seeded constant moved and the plan predicted it
  (`UI3_SPLIT_COMBAT_SEED` 28 → 32, re-observed by an executed sweep). Filed **OOS-DX43-1..7**.
  Full record: `memory/primitives/pb-DX43-execution-notes.md`; handoff:
  `memory/workstream-state.md`.
- **Prior**: 2026-08-14 — **SEED RE-RANK v4 SHIPPED** (`scutemob-212`, doc-only):
  `memory/primitives/seed-rerank-2026-08-14.md` is the authoritative queue; v3's §4 is banner'd
  SUPERSEDED (its §1-§3 stay canonical). **A census cutoff is a date on a document, and work does
  not respect it.** Census **208** post-v3 seed IDs — 2.6× v3's 80, ~6× the brief's "~35+" — by a
  published set-difference rule (`ALL 488 − V3 79 − LEGACY 196 = 213`, minus 5 that are not seeds:
  a plan-only closed-on-arrival, a conditional that never fired, an explicitly rejected number, a
  deliberately skipped number, and a renumbering). **v3 recorded this exact failure about v2 and
  then reproduced it**: its census closed 2026-08-02, the day the adjudication and the whole
  triage-2 successor run shipped. **61 of 208 — 29% — have no registry row**, and the registry is
  what dispatch hygiene 5 calls ground truth; add the 7 behind standing rows and the blind spot is
  **68**. The unrowed set is one era of work (`scutemob-186..194`) filed into handoff prose under a
  convention nobody wrote down as a rule — `OOS-G1-1`'s note says a seed closed in its own batch
  gets no row, which is fine for the nine such seeds and does not cover the ~50 that are OPEN.
  **Rank 1 is a seed filed "latent" that is live-wrong on three deck-legal `Complete` format
  staples**: `OOS-DX27-1`, no CR 305.6 intrinsic-mana-ability derivation, so `urborg_tomb_of_yawgmoth`,
  `yavimaya_cradle_of_growth` and `dryad_of_the_ilysian_grove` grant a basic land type and no mana
  ability ever follows — the `AddSubtypes` arm is three lines that touch `chars.subtypes` and
  nothing else, and `swamp.rs:11-27` hand-authors `{T}: Add {B}` which CR 305.6 says it should not
  need to. `OOS-DX27-10` closes for free inside it. **PB-DX42b re-decided, not carried**:
  `OOS-DX27-9`'s "the rank premise is false" **does not hold on the axis the rank used** — the
  total layer-querying population moved 1 → 2 but `the_world_tree` is `partial`, so the deck-legal
  `Complete` population moved **1 → 1** and the 7 pairs are unmoved. The seed's durable half (the
  supply census does not carry over to a `Land` filter) lands only when PB-DX9 promotes that def,
  and that coupling is now written down. **Two independent verifications of this task reached
  opposite verdicts on the same gate, and reconciling them is worth more than either**: `OOS-ADJ-2`
  is **partially discharged** — PB-DX42a's gate pins the population by name and **fired on its
  first real event**, and it is blind to **7 of the 11** layer-querying `Condition` variants,
  because axis 1 filters on one literal variant name and axis 2 needs a `TargetFilter` eight of
  them do not carry. *A gate written for one variant measures that variant*, arriving at the gate
  written to close the seed that predicted it. **`OOS-DX24-9` ≡ `OOS-DX27-5`** — the same
  `MayPayThenEffect` defect filed twice by two batches five days apart, neither row citing the
  other, and **two independent passes both re-measured it at 11 deck-legal `Complete` defs**.
  **Four silently-closed seeds** found by reading code (`OOS-SIM4-2`, `OOS-DX20-7`, `OOS-DX26-7`'s
  class half, `OOS-DX7-3`), **none recorded anywhere**. **Five of twenty-one standing wire cells
  were wrong** — two predict a bump and measure none, one "none" is unsafe, two omit a bump — so
  every v4 wire cell now carries a confidence. **Two registry rows are WRONG rather than stale**,
  and `OOS-UI2-5`'s would make PB-DX33 **create** the defect it describes: the TUI has never routed
  a cast, so a human gets a refusal, not a silent default. **The user-directed Blood Moon / Urza's
  Saga flag is DISCHARGED** as `OOS-RR4-1/-2/-3` — and the flag was refuted in four particulars,
  including that the "missing" gains-an-ability primitive **exists with four corpus users** and
  that the Saga site list is **five** behavioural sites, not two. Corner case #36 is the audit's
  **only** remaining GAP (35 COVERED / 1 GAP, measured). Doc-only: `git diff --numstat` over
  `crates/` and `tools/` is **empty**; tests **4,721**, coverage **63.0%**, PROTOCOL **37** /
  HASH **76** all untouched **by construction**. Full memo:
  `memory/primitives/seed-rerank-2026-08-14.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-08-14 — **PB-DX29 SHIPPED** (`scutemob-211`; v3 queue rank 13 —
  **OOS-M11-10(loyalty)** and **OOS-UI2-4** both CLOSED, and the **OOS-M11-10 ID collision
  RESOLVED**: that note deferred renumbering to "whichever task next touches `params.rs`", this is
  that task, and the closed equip seed is now **OOS-M11-10E**). **A choice you cannot express is a
  choice you do not have.** Both halves were framed as pure routing — "the `Command` fields already
  exist" — and **both framings were short by the load-bearing link.**
  The loyalty half needed **two engine functions**: the seed says CR 602.2b targets are "already
  reachable through `queries.rs::ability_target_requirements`' sibling path", which is true of the
  *machinery* and **false of the index space** — that function indexes
  `Characteristics::activated_abilities` while a loyalty `ability_index` is minted against the
  **registry** def's filtered `AbilityDefinition::LoyaltyAbility` list. Index 0 means different
  abilities to the two. The cost half needed **`effective_cast_cost_with_additional` extended**, and
  no document named it: it read **Squad and nothing else**, and it is what
  `LocalGame::auto_tap_commands_for` asks how much mana to tap — so shipping the seven pickers alone
  would have tapped the base cost, accepted the human's announcement and let the engine refuse with
  `InsufficientMana`. **The batch would have created the exact SR-38 defect it was dispatched to
  remove.**
  **Enforcement-site lists were short in both halves**: five loyalty sites, not two, including
  `view.rs::target_query_source` (which renders a picker with **zero candidates** on its own) and
  `targeting.rs`, the **bot** path outside `tools/play-server` entirely, whose omission would have
  re-created SIM-5's zero-target defect on a new action.
  **Populations re-derived**: **4 of SEVEN** `Complete` planeswalkers, not 4 of 6; **15**
  `AdditionalCost` variants and **Kicker is not one of them** — UI-2's README and `api.rs` doc both
  listed a variant that does not exist; and "13 of 15 kinds invisible" is arithmetically right and
  **materially misleading**, because four have **no deck-legal member at all** and three more are
  unreachable by construction. **A fifth `Complete` planeswalker was live-wrong and no cite had
  ever named it**: `chandra_flamecaller`'s `LoyaltyCost::MinusX` was **−0 for 0 damage** in every
  client, because `params.rs` hard-coded `x_value: None`.
  **A new live defect, and it is `r3b`'s Squad shape INVERTED**: `nocturnal_hunger` is `Complete`,
  deck-legal, carries `AbilityDefinition::Gift` and **no `KeywordAbility::Gift`**, and `casting.rs`
  gates on the marker first — printed gift, unpayable, nothing red. UI-2 wrote `r3b` *because the
  corpus had failed it on Squad*, and the same defect had already recurred one variant over. **A
  gate written for one variant measures that variant** — the batch's thesis, which then arrived
  **three more times inside its own work**: UI-2's R4 promised to fail "the day one is authored" and
  PB-DX29's widened R3 went red on its **first run** on `brokkos_apex_of_forever`'s hybrid mutate
  cost (fixed the *formatter*, CR 107.4e/107.4f/107.3, not the gate); R5 justifies `ActionBar`'s
  stage order while walking Sacrifice and Squad only, so it reports a clean board while its own
  condition is live on **five** defs (new R6 prints them and asserts the half that matters — **0**
  defs pair a cost with an `{X}`); and **the batch's own Fuse cost arm called the seven-component
  helper under a comment saying `casting.rs` mirrors three more fields "for Fuse … mirrored
  deliberately"**. It did not. Proven by execution: predicted mana value 3, engine charged 4, cast
  refused from a pool holding exactly the prediction.
  **A picker was GATED rather than shipped**: `casting.rs` never concatenates the fuse right half's
  targets, so a fused cast of either deck-legal fuse def is a guaranteed `InvalidTarget` —
  pre-existing, and unreachable until this batch's picker made it reachable (`OOS-DX29-12`).
  **Mutate is the mandatory-kind proof**: measured, it is the only mandatory kind both reachable and
  buildable, `LegalAction::CastWithMutate` gains `on_top`, and the provider emits one action per
  `(target, on_top)` pair — the PayEcho/ChooseDredge idiom, no params field and no wire change. No
  client could ever mutate **under** before, and CR 702.140e makes the topmost component supply the
  merged permanent's name, cost, colours, types and P/T. Its CR 702.140c **timing** is filed, not
  moved.
  Three machine gates caught this batch's own work and every one was right (SR-5, its
  ability-definition sibling, and `pb_dx27_stale_blocker_notes` firing on the batch's own
  `dawns_truce` note). Two part-A defects were found by the batch's own test author and taken: the
  new queries used `expect_object` — the impossible-absence lookup — while their rustdoc promised
  "never panics" (**what is impossible for an engine-internal caller is ordinary input for a UI
  one**), and joining the allowlist widened a declared residual from nine arms to ten while the doc
  still said nine. Coverage unmoved at **1,136/1,803 = 63.0%**, **0 flips**, proven by regeneration;
  **PROTOCOL 37 / HASH 76 both gate-executed and unmoved**; engine lines **NOT zero** and the brief
  predicted zero — **+177 / −11**, of which 101 are the new read-only query surface and 76 are
  registry *declarations* two gates refused to let the batch omit. Refusal-channel A/B **105 → 105
  with an empty diff**, reported as proof of **bot-path neutrality** rather than of nothing
  happening. **`/review` 2 HIGH / 6 MEDIUM / 11 LOW, all 19 taken** — the two HIGHs were a Splice
  offer with no affordability bound that 422'd after a clean offer, and a renumbering that
  orphaned 30 in-source cites under a note asserting it had not. Filed **OOS-DX29-1..17**. Full record:
  `memory/primitives/pb-DX29-execution-notes.md`; handoff: `memory/workstream-state.md`.
- **Prior**: 2026-08-14 — **PB-DX28 SHIPPED** (`scutemob-210`; v3 queue rank 12 —
  **OOS-DX4-6** and **OOS-DX4-1** both CLOSED). **A spell targets only where it says "target"**
  (CR 115.10), and 18 `Complete` deck-legal defs said no such thing while declaring a real
  `TargetRequirement` anyway. Wrong in two directions: hexproof / shroud / protection wrongly
  restricted the choice, and CR 608.2b fizzled the effect when the chosen permanent left in
  response — which on the ten Karoo bounce lands is **an exploit in the controller's favour**
  (respond by moving the chosen land, keep both). New `EffectTarget::ChosenObject` resolves on
  the existing CR 608.2d suspend-and-replay channel as `EffectChoiceQuestion::ChooseObject`.
  **Candidate derivation deliberately does NOT route through `validate_targets_inner` /
  `validate_object_satisfies_requirement` / `legal_targets_per_slot`** — all three apply full
  CR 115 targeting legality, which IS the defect.
  **Both seeds' member lists were floors, and the batch proved its own census was one too.**
  `OOS-DX4-6` said "two `Complete` defs"; the census found **18**, and then the batch's own
  inverse gate found a **19th** (`Connive // Concoct`) *after* the roster had been pinned at 17.
  It was migrated, not deferred: closing a class while a known deck-legal member keeps the
  defective shape closes it on a false premise. That reversal paid for itself — it reddened R3,
  whose walk enumerated only `Triggered`/`Spell`/`Activated` and therefore **could not see** a
  split card's `Fuse` half, a hole that was unreachable for as long as no member used the
  missing variant. `OOS-DX4-1` named four members; **two survive** (`staff_of_compleation`,
  `nether_traitor`) and the refutations matter more than the members: **the six mutate defs are
  clean** (CR 702.140a ownership is enforced open-coded in `casting.rs`, outside `TargetFilter`),
  and **`fecundity` is not a member though `nether_traitor`'s own note said it was** — its gap is
  `ControllerOf(TriggeringCreature)`, a *controller* gap, exactly as its own marker note already
  said.
  **The plan's enforcement-site list was short, again**: `rules::abilities`' auto-target picker
  is **two** functions — the predicate and the enumerator — and fixing only the first would have
  left the offer layer wrong while validation was right. Also repaired
  `sword_of_war_and_peace`, whose comment claimed `ctx.damaged_player` resolution while the code
  read a declared target, so in a 4-player game the Sword could damage the **wrong seat**.
  **The `/review` found 3 MEDIUM / 4 LOW, all 7 taken, and defeated two of the batch's own
  gates by execution**: a `ChosenObject` moved to an unsupported effect arm kept all five roster
  rows green (silent resolve-to-empty in release, since the `debug_assert` is compiled out —
  closed by a new R5); and R4's `slots > words` census **cancels**, defeated by one planted
  sentence in the SAME ability, so its "the class cannot silently regrow" claim is WITHDRAWN.
  The third MEDIUM is the batch's own record: the execution notes' "verbatim" gate output quoted
  two fingerprints **that have never existed in any source file in this repository** — PB-DX8's
  "publish the figure, do not transcribe it" rule broken in the evidence for the very criterion
  that depends on it. Tests **4,634** (+29, itemised by name, 0 removals); coverage unmoved at
  **1,136/1,803 = 63.0%**, 0 flips; **PROTOCOL 36 → 37 / HASH 75 → 76**, both from the gates'
  own output, with the PROTOCOL closure moving **96 → 98** types — its first count change since
  v31. Filed **OOS-DX28-1..10**. Full handoff: `memory/workstream-state.md`; census, revert
  matrices and the review table: `memory/primitives/pb-DX28-execution-notes.md`.
- **Prior**: 2026-08-13 — **PB-DX27 SHIPPED** (`scutemob-209`; v3 queue rank 11 —
  **OOS-CARDS2-8**, **OOS-CARDS2-10**, **OOS-CARDS2-11**, **OOS-RR3-2** and the rider
  **OOS-ADJ-7** all FILED *and* CLOSED). **A blocker note is a claim, and nothing had ever
  re-checked one.** `OOS-DX3-1`'s closure called the corpus-wide re-check "a cheap standing
  sweep" and closed without filing it; this batch is that sweep, plus a gate so it cannot
  silently reopen. **None of the five seeds had a registry row** — the third batch running
  to find its own seeds unrowed (dispatch hygiene 5 held: grep first, then file, then close).
  **The population the seed was RANKED on does not reproduce**: the memo's 67 machine-checkable
  notes yields **49** by its own literal method at HEAD, 46 ground-truth-restricted, 109 by an
  inverse method, and no variant reaches 67. The brief called it "a FLOOR and a snapshot"; it
  is a snapshot and **not** a floor, because every reproduction is *smaller*. Of the 46
  adjudicated: 10 REFUTED and repaired, 3 REFUTED-PARTIAL, 30 CONFIRMED with the still-missing
  identifier named, 9 STALE-WORDING. The dominant shape is the one the CARDS-2 sweep predicted
  — **an inline `// TODO` and a `Completeness` note in the same file disagreeing, note
  correct** — and `marisi_breaker_of_the_coil`'s note literally says **STALE** while its TODO
  denies a variant six corpus defs already use.
  **Two REFUTED repairs were declined, and the distinction was load-bearing**: `kaito_shizuki`'s
  −7 (`Effect::CreateEmblem` exists, but `collect_emblem_triggers_for_event` has 6 call sites
  and **none is a combat-damage site**, so authoring it ships a 7-loyalty no-op) and
  `blackblade_reforged`'s land-count static (`resolve_cda_amount` resolves the controller from
  the **equipped creature**, CR 108.5/611.2c-wrong). **Existence is necessary and never
  sufficient.**
  **The rider found more than it was filed for.** `OOS-ADJ-7`'s population is **3** `Complete`
  defs, not 2 — `dryad_arbor` was missed — and the same ruling's third sentence ("they gain
  `{T}: Add {R}`") was implemented **nowhere**, so a Blood-Mooned land lost every ability and
  gained nothing. New `LayerModification::SetLandTypes`, the exact analogue of the shipped
  `SetCreatureTypes`. **The brief's "expected wire impact NONE" was refuted by the gate** —
  `ContinuousEffectDef.modification` is a sibling of `filter`/`duration`, both already on the
  wire — so **PROTOCOL 35 → 36 / HASH 74 → 75**, taken from the gates' own output.
  **`OOS-ADJ-2` came true on its own gate's first real event**: authoring The World Tree's
  six-lands static grew the layer-querying population **1 → 2**, exactly what that seed
  predicted would "silently join the deviation". It was not silent — the PB-DX42a gate fired
  and forced exit (b), so **PB-DX42b's rank premise (a measured population of exactly 1) is
  now false** (`OOS-DX27-9`).
  **The `/review` found 1 HIGH / 5 MEDIUM / 6 LOW and all 12 were taken — and the HIGH was
  this batch committing its own subject matter, twice.** `chord_of_calling` and
  `green_suns_zenith` were promoted to deck-legal `Complete` with their printed **"then
  shuffle" unauthored**: `Effect::SearchLibrary` has no post-search shuffle, and
  `eldritch_evolution.rs:12-14` — **the very file both defs cite as precedent** — says so
  in-source. Checking the *other* clause then found the second instance:
  `self_shuffle_on_resolution` does not shuffle (deterministic top-of-library,
  `resolution.rs:2023-2025`), `nexus_of_fate` is `partial` for that reason, and
  `green_suns_zenith` claiming `Complete` was **the same outlier shape this batch demoted
  `qarsi_sadist` for**. Demoted back; coverage +4 → **+3**. The reviewer's diagnosis of *why*
  it shipped is the durable half: **the three headline defs had zero behavioural coverage**,
  now closed by 9 probes whose revert row R2 reproduces the exact HIGH. Also taken: a second
  recall bound the gate never stated (**74** defs name a live identifier inside a gap phrased
  outside the needle set — invisible to both ratchets, now ratcheted and revert-proven); a
  calibration table publishing figures that **did not reproduce against the shipped code**
  (deleted, and every population is now PRINTED by `t_derivation_report` — the same correction
  PB-DX8 made and this file's own doc claimed to have learned); and **"`ALL_LAND_TYPES` had
  zero users" asserted as *the proof* in three places, which is false** (`correlated_card_types`
  reads it) — right conclusion, wrong proof, in a batch whose thesis is that a note is a claim.
  **The corpus was reconciled TWICE**: the demotion moved `CORPUS_COMPLETE` again and re-dealt
  every seeded fixture a day after the implement phase had re-observed all nine. **One marker
  flip anywhere in 1,803 defs invalidates every seeded pin** — budget for two passes, not one.
  Tests **4,605** (+44, itemised by name, 0 removals); coverage **63.0%**; PROTOCOL **36** /
  HASH **75**. Filed **OOS-DX27-1..10**. Full handoff: `memory/workstream-state.md`;
  measurements, disposition table and revert matrices:
  `memory/primitives/pb-DX27-execution-notes.md`.
- **Prior**: 2026-08-12 — **PB-DX8 SHIPPED** (`scutemob-208`; v3 queue rank 10 —
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
- 36 corner cases: **35 COVERED, 1 PARTIAL, 0 GAP, 0 DEFERRED** (re-measured 2026-09-03 by
  `scutemob-220`/PB-DX49, which moved #36 GAP → PARTIAL: its **engine** half is covered and its
  **card** half is still gated on `urzas_saga` authoring (`OOS-RR4-2`). Prior measurement
  2026-08-14 by `scutemob-212`
  from `docs/mtg-engine-corner-case-audit.md` itself — the previous "32 COVERED, 4 GAP" was stale
  by three closures, and the audit's own **Summary table** — the one
  `tools/tui/src/dashboard/parser.rs` machine-reads — was still saying `32 / 0 / 4 / 0` until
  PB-DX49 fixed it, so `OOS-RR4-3`'s finding (ii) was only half discharged and the surviving half
  was the half a tool depended on. `OOS-RR4-1` and `OOS-RR4-3` are CLOSED; `OOS-RR4-2` (the card
  half) stays open and ranked in `memory/primitives/seed-rerank-2026-08-14.md` §4)

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
| **Course Correction (2026-09)** | `docs/course-correction-2026-09.md` | **DRAFT under section review** — audit findings, the context diet, pod-first roadmap (P0–P3), agents/skills tuning; task lists CC-1..CC-14 are filed only after each section is signed off |
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
| Interaction deconstructions | `docs/interactions/` (`blood-moon-urzas-saga.html`) | Shareable, self-contained HTML explainers of engine-resolved interactions; two-layer (table + engine room) |
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

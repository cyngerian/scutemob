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
  `scutemob-182`; PB-DX6 shipped `cb0755bf`, **next PB-DX19**; the playtest-successor run 174–181
  completed 2026-08-02 in between).
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
  action, and with a vigilant attacker the engine will accept re-declaration without limit).
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
  `memory/workstream-state.md`). PROTOCOL **33** / HASH **70**. **Next dispatch: PB-DX19**
  (OOS-SIM2-6 + OOS-SIM2-5 — an unbounded `calculate_characteristics` recursion that hard-aborts
  the process from a deck-legal `Complete` card, one-line fix; brief in
  `memory/primitives/seed-rerank-2026-08-02.md` §4). **PB-DX7 is no longer next** — it survives at
  rank 9; eight new entries outrank it. Older queue history (the PB-OS,
  PB-RS and PB-DP chains) is rotated to the 2026-08 archive.
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
- **Last Updated**: 2026-08-02 — **PB-DX19 SHIPPED** (`scutemob-184`), the v3 queue's first dispatch:
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
  probe is the only one that fails by assertion, not panic. **35 arithmetic edits across two
  files**, counted from the diff rather than asserted — the count first published was wrong twice. **The fix's cost is real and is
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
  wrong in the other direction, several being *correct* as layer-resolved. The deviation's scope is
  now the layer walk alone. Seeds **OOS-DX19-1..4** filed. PROTOCOL **33** / HASH **70**
  gate-executed and unmoved. Tests **4,278 / 0 / 5** (+15). Coverage **unmoved** — proven by
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

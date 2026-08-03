# Primitive batch WIP — PB-DX22 (prior batches are stale history; see their own plan files)

**Batch**: PB-DX22 — make the fuzzer a real instrument
**Seeds**: `OOS-UI2-1` (the fuzzer has never cast a spell at ordinary depth) + `OOS-SIM3-1`
(earliest measured cast turn 143) + `OOS-SIM1-4` (`commander_ids` never registered, so
CR 903.8 / 903.9a / 903.10a have never been fuzzed)
**Brief (authoritative)**: `memory/primitives/seed-rerank-2026-08-02.md` §4, PB-DX22 entry
(rank 4, EVIDENCE INTEGRITY) + §2.4. Cross-read `docs/mtg-engine-feedback-engineering.md` §2.1
row 1.
**Task**: `scutemob-196` · **Branch**: `feat/pb-dx22-make-the-fuzzer-a-real-instrument-oos-ui2-1-oos-sim3`
**Phase**: implement (Stages 0-4; Stage 5 is the coordinator's)

---

## THE MANDATORY PRE-PLAN MEASUREMENT — RAN, AND IT IS DECISIVE

The brief made one measurement mandatory *before* acceptance evidence could be written:
SIM-1 added a command-zone cast loop (`legal_actions.rs:675-693`) and a commander is **not**
in the library, so the no-shuffle defect does not gate it — a bot should be able to cast its
commander around game turn 12-24, a hundred-odd turns before SIM-3's measured 143. **Either
SIM-3 measured a pre-SIM-1 build, or something suppresses that offer for bots.**

**Method**: a scratch `crates/simulator/examples/dx22_measure.rs` (deleted before commit)
replicating `mtg-fuzzer::run_single_game`'s state build byte-for-byte as it stands at HEAD
(`9aa4f220`) — no shuffle, no `player_commander` — driven through `LocalGame` with
`record_journal: true`, scanning the journal for `SpellCast`,
`CommanderCastFromCommandZone`, `CommanderReturnedToCommandZone` and
`CommanderZoneRedirect`. 5 games, base seed 1, `--max-turns 200`, 4 players, `RandomBot`.
Raw output: `memory/primitives/pb-dx22-measurement-head.txt`.

| seed | end | commands | `commander_ids` populated | first `SpellCast` turn | total casts | first commander cast | cmdr returns / redirects | rejections |
|---|---|---|---|---|---|---|---|---|
| 1 | GameOver t176 | 9,802 | **0/4** | **154** | 25 | **none** | 0 / 0 | 33 |
| 2 | GameOver t192 | 12,631 | **0/4** | **143** | 21 | **none** | 0 / 0 | 107 |
| 3 | Halted MaxTurns 201 | 12,362 | **0/4** | **151** | 35 | **none** | 0 / 0 | 149 |
| 4 | GameOver t167 | 9,397 | **0/4** | **153** | 7 | **none** | 0 / 0 | 98 |
| 5 | Halted MaxTurns 201 | 12,601 | **0/4** | **151** | 33 | **none** | 0 / 0 | 38 |

**THE ANSWER: the offer is SUPPRESSED, not merely late — and `OOS-SIM1-4` is the cause of it.**
Zero `CommanderCastFromCommandZone` in ~56,800 commands across five games running to turn
167-201. The mechanism is in the provider's own documented filter: `legal_actions.rs:675-693`
says in as many words that "the zone is NOT the filter; `commander_ids` is" (CR 903.8, and
CR 408.1 is the reason — emblems live in the command zone too). `fuzzer.rs:322-327` places the
commander *object* in `ZoneId::Command(pid)` but never calls `builder.player_commander`, so
`commander_ids` is empty in **every seat of every game** and the loop's own CR 903.8 filter
rejects the commander before the offer is ever built.

**Three consequences the plan must carry, none of which were settled before this ran:**

1. **The brief's disjunction resolves to its second branch.** SIM-3 did **not** measure a
   pre-SIM-1 build — SIM-1's loop is present and correct at HEAD; it is starved of its input.
   `OOS-SIM1-4` and the missing commander cast are **one defect, not two**, and registering
   `commander_ids` is what un-suppresses the offer. That reduces PB-DX22's sizing: no provider
   change is needed.
2. **`OOS-SIM3-1`'s turn-143 number reproduces exactly** (seed 2, first `SpellCast` at turn
   143), and the whole five-seed band is **143-154**, which sits inside §2.4's arithmetically
   predicted 136-156. So `OOS-UI2-1`'s word "never" and `OOS-SIM3-1`'s "turn 143" are the same
   fact read at two `--max-turns` values, exactly as the brief argued — and this batch must
   record that as a **threshold**, not a closure.
3. **The horizon is a hard floor for commander mechanics too.** CR 903.9a zone return and
   CR 903.10a commander damage cannot be exercised by a commander that is never cast, so they
   are gated on fix (1), not on the shuffle.

Recorded **before** any acceptance evidence was written, per the brief.

---

## Plan file

`memory/primitives/pb-plan-DX22.md` (written by `primitive-impl-planner`).

---

## Stage checklist (plan §5)

### - [x] Stage 0 — baseline, no source edits

**Full-workspace baseline**, captured to a file (never `| tail`), summed with awk over 41
`^test result` lines: **4,345 passed / 0 failed / 5 ignored**. Exit 0, zero `failures:` /
`FAILED` / `error[` lines. Raw: `/tmp/pb-dx22-baseline.txt`. Matches the plan's expected 4,345.

**Wire sentinels GATE-EXECUTED, not predicted**:

* `cargo test -p mtg-engine --test core hash_schema` → exit 0, all green;
  `crates/engine/src/state/hash.rs:743` `pub const HASH_SCHEMA_VERSION: u8 = 72;` → **HASH 72**.
* `cargo test -p mtg-engine --test core protocol_schema` → `17 passed; 0 failed`;
  `crates/engine/src/rules/protocol.rs:360` `pub const PROTOCOL_VERSION: u32 = 35;` →
  **PROTOCOL 35**.

**Baseline fuzz run** (`--profile fuzz`, the assertions profile):
`cargo run --profile fuzz --bin mtg-fuzzer -- --games 20 --seed 1 --max-turns 200 --threads 1
--verbose` → `/tmp/pb-dx22-fuzz-before.txt`.

| metric | value |
|---|---|
| games completed | 20 |
| wall time | 40.7 s (build 25.5 s separately) |
| wins / draws / errors | 9 / 0 / **11** |
| error distribution | 11 × `MaxTurnsReached(200)`, 0 of any other kind |
| total violations | **1,519** |
| avg turns per game | **191.7** |
| violations by `check` (over the 721 lines the binary prints — it prints only the first 5 offending games, so this is a *sample*, not the 1,519) | `no_orphaned_tokens` 569, `player_consistency` 152 |

Per-game (seed / turns / commands / violations / end): 1/176/9802/526/P1 · 2/192/12631/50/P1 ·
3/201/12362/14/MaxTurns · 4/167/9397/116/P1 · 5/201/12601/15/MaxTurns · 6/172/9338/0/P1 ·
7/173/10182/36/P2 · 8/201/12035/8/MaxTurns · 9/201/11594/8/MaxTurns · 10/201/11575/36/MaxTurns ·
11/201/12013/445/MaxTurns · 12/201/12083/14/MaxTurns · 13/182/10044/0/P1 · 14/193/10943/3/P1 ·
15/182/9876/35/P1 · 16/201/12079/31/MaxTurns · 17/201/11475/0/MaxTurns · 18/185/10739/5/P3 ·
19/201/13228/151/MaxTurns · 20/201/11279/26/MaxTurns.
Seeds 1-5 reproduce the §0 pre-plan table exactly (turns 176/192/201/167/201, commands
9802/12631/12362/9397/12601) — the instrument is the same one the measurement used.

**Tree hygiene**: `git status --short` showed exactly `?? crates/simulator/examples/` and
`?? memory/primitives/pb-plan-DX22.md`; `git ls-files crash-reports` **empty**;
`git check-ignore -v crash-reports` → `.gitignore:52`, so the run's crash artefacts are
untracked by construction.

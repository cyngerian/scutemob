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

### - [x] Stage 1 — extract the build path, behaviour-NEUTRAL

**New** `crates/simulator/src/fuzz_setup.rs` (147 lines): `FuzzGameSetup`, `FuzzSetupError`,
`place_registered_deck` (`:64`), `build_fuzz_state` (`:102`). A verbatim lift of the old
`fuzzer.rs:289-359` — **no shuffle, no `player_commander`, no
`register_commander_zone_replacements`** at this stage. The `if let Some(def)` silent-skip is
preserved verbatim and documented as a divergence from `setup.rs` (`OOS-DX22-4`), not fixed.

* `crates/simulator/src/lib.rs:19` `pub mod fuzz_setup;` + `:35` re-exports
  `build_fuzz_state, place_registered_deck, FuzzGameSetup, FuzzSetupError`.
* `crates/simulator/src/bin/fuzzer.rs:281-306` — `run_single_game`'s 70-line state build
  collapses to one `build_fuzz_state(..)` call. The `GameDriverError::EngineError(format!(
  "Failed to build state: {:?}", e))` string is kept **byte-identical**. Imports narrowed to
  `{all_cards, CardDefinition, CardRegistry, PlayerId}` + `{build_fuzz_state, ..., FuzzSetupError}`.
* `crates/simulator/examples/dx22_measure.rs` **deleted**, and the now-empty `examples/` dir
  with it. `git ls-files crates/simulator/examples` → empty (it was never tracked).

**NEUTRALITY EVIDENCE (mandatory, plan §5)**: the Stage-0 fuzz command re-run verbatim into
`/tmp/pb-dx22-fuzz-after-stage1.txt`, then `diff` against `/tmp/pb-dx22-fuzz-before.txt` with
only cargo's own build chatter filtered out. **766 lines each side; exactly ONE differing
line**, and it is wall time:

```
9c9
< Games completed: 20  Time: 40.7s  (0 games/sec)
---
> Games completed: 20  Time: 41.3s  (0 games/sec)
```

Every per-game line (seed / turns / commands / violations / winner) and all 721 printed
violation lines are byte-identical. The extraction changed nothing.

**Probe P1** — `test_dx22_build_fuzz_state_produces_the_fuzzers_table`
(`crates/simulator/tests/pb_dx22_fuzz_instrument.rs`). Asserts, per seat: `decks` in ascending
`PlayerId` order, `main_deck.len() == 99`, exactly 1 command-zone object and it IS the
decklist's commander, 99 library objects, and **0 hand objects** (the CR 103.5 pin for §B2 /
`OOS-DX22-1`).

**P1 revert-proof, EXECUTED**: `place_registered_deck`'s `for card_id in &deck.main_deck` →
`deck.main_deck.iter().take(98)`. Rebuild **succeeded** (`Compiling mtg-simulator v0.1.0` in
the output — checked, per the `-D warnings`/stale-binary gotcha), then:

```
thread 'test_dx22_build_fuzz_state_produces_the_fuzzers_table' panicked at
crates/simulator/tests/pb_dx22_fuzz_instrument.rs:78:9:
assertion `left == right` failed: seat PlayerId(1)'s library must hold all 99 main-deck cards
  left: 98
 right: 99
test result: FAILED. 0 passed; 1 failed
```

Restored; green again.

**Divergence from the plan worth recording**: the plan's probe table has P2's
`library_card_ids` helper living in this file from Stage 1. Written that way it is *dead code
at Stage 1*, and `-D warnings` turned that into a **build failure** — `error: function
library_card_ids is never used ... -D dead-code implied by -D warnings`. Rather than paper
over it with an `#[allow(dead_code)]` that would have to be remembered and removed, the helper
is deferred to Stage 2 where it has a caller. (Same mechanism as the gotchas-infra revert
lesson, arriving from the other direction.)

**Stage-1 gates, all executed**: `cargo build --workspace` OK · `cargo test -p mtg-simulator`
**170 passed / 0 failed** across 12 targets · `cargo clippy --workspace --all-targets -D
warnings` exit 0 · `cargo fmt --check` exit 0 · `tools/check-defs-fmt.sh` → `1803 defs
checked / clean` (SR-35).

### - [x] Stage 2 — CR 103.3 / CR 903.6 shuffle

`fuzz_setup.rs::build_fuzz_state` now keeps **two** lists: `decks` is the pre-shuffle decklist
(returned on `FuzzGameSetup`, so the probe has something to compare against) and `dealt` is the
shuffled order actually placed. The shuffle is drawn **inside the existing per-seat deck loop,
immediately after that seat's `random_deck` call**, in ascending `PlayerId` order — the
`deck₁, shuffle₁, deck₂, shuffle₂, …` interleaving `setup.rs` uses. The reason for that choice
(free here; load-bearing there) is written at the function, not left to the plan.

**Probes P2 / P3 / P4** added, each proven red by an EXECUTED revert whose rebuild succeeded:

| probe | revert executed | rebuild | failure observed |
|---|---|---|---|
| **P2** `test_dx22_libraries_are_shuffled_cr_103_3` | `deck.main_deck.shuffle(&mut rng)` → `deck.main_deck.truncate(99)` (a no-op) | `Compiling mtg-simulator` present | `pb_dx22_fuzz_instrument.rs:146` `assertion left != right failed: CR 103.3: seat PlayerId(1)'s library must not be the decklist in its construction order` — 3 passed / 1 failed |
| **P3** `test_dx22_shuffle_is_seed_deterministic` | `shuffle(&mut rng)` → `shuffle(&mut rand::rng())` | present | `:177` `assertion left == right failed: seed 1 must reproduce seat PlayerId(1)'s library order exactly` — 3 passed / 1 failed |
| **P4** `test_dx22_different_seed_different_order` | `StdRng::seed_from_u64(seed)` → `seed_from_u64(0)` | present | `:209` `assertion left != right failed: seeds 1 and 2 must not deal seat PlayerId(1) the same library` — 3 passed / 1 failed |

**PLAN DIVERGENCE (P4's revert).** The plan's probe table gives P4's revert as "as P3". That is
**wrong, and it was measured, not reasoned**: under P3's revert (`rand::rng()`) P4 stayed
GREEN, and under P2's revert (no shuffle at all) P4 *also* stayed green — because seeds 1 and 2
draw different **decklists**, so the two libraries differ in construction order too. The only
revert that actually reddens P4 is one that makes the build ignore its `seed`, which is what
was executed. Recorded rather than accommodated: had the plan's revert been run and reported,
P4 would have been shipped with no discrimination proof at all.

**SECOND `-D warnings` TRAP, hit and recorded.** The first draft of P2's revert deleted the
shuffle line outright. That rebuild **FAILED** — `error: unused import: rand::seq::SliceRandom`
and `error: variable does not need to be mutable` — which under the gotchas-infra rule means
`cargo test` would have run the stale binary and reported a pass. The revert was rewritten as a
no-op that still consumes `mut deck` (`truncate(99)`) plus an `#[allow(unused_imports)]`, and
only then was the red result trusted. This is the documented hazard occurring in the wild
twice in one batch.

**Stage-2 fuzz run** (`/tmp/pb-dx22-fuzz-after-stage2.txt`, same command as Stage 0). This is
the point of the batch and it moves enormously:

| metric | Stage 0 (no shuffle) | Stage 2 (shuffled) |
|---|---|---|
| games completed | 20 | 20 |
| wall time | 40.7 s | **14.3 s** |
| wins / draws / errors | 9 / 0 / **11** | **20 / 0 / 0** |
| error distribution | 11 × `MaxTurnsReached(200)` | **none** |
| total violations | 1,519 | **504** |
| avg turns per game | 191.7 | **112.7** |
| total turns / total commands | 3,833 / 225,276 | 2,254 / 104,252 |
| commands per turn | 58.8 | **46.3** |
| violation `check`s seen | `no_orphaned_tokens`, `player_consistency` | same two — **no new check class** |

Notable: not one game now hits the turn cap, and `HaltReason::InfiniteLoop` did **not** appear
either — commands/turn went *down*, 58.8 → 46.3, so risk 2 of plan §8 (`max_commands` binding
before `max_turns`) did **not** materialise at these settings. That number is the input
`OOS-DX22-2` asks for. No crash, no abort, no new violation class surfaced at this stage
(plan §8 risk 1 did not fire here).

**Stage-2 gates**: `cargo build --workspace` OK · `cargo test -p mtg-simulator` **173 passed /
0 failed** (+3 = P2/P3/P4) · `clippy --workspace --all-targets -D warnings` exit 0 ·
`cargo fmt --check` exit 0 · `tools/check-defs-fmt.sh` clean.

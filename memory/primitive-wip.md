# Primitive batch WIP — PB-DX22 (prior batches are stale history; see their own plan files)

**Batch**: PB-DX22 — make the fuzzer a real instrument
**Seeds**: `OOS-UI2-1` (the fuzzer has never cast a spell at ordinary depth) + `OOS-SIM3-1`
(earliest measured cast turn 143) + `OOS-SIM1-4` (`commander_ids` never registered, so
CR 903.8 / 903.9a / 903.10a have never been fuzzed)
**Brief (authoritative)**: `memory/primitives/seed-rerank-2026-08-02.md` §4, PB-DX22 entry
(rank 4, EVIDENCE INTEGRITY) + §2.4. Cross-read `docs/mtg-engine-feedback-engineering.md` §2.1
row 1.
**Task**: `scutemob-196` · **Branch**: `feat/pb-dx22-make-the-fuzzer-a-real-instrument-oos-ui2-1-oos-sim3`
**Phase**: COMPLETE — stages 0-5 + stage 4b + two review cycles, close-out written

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
| violation `check`s seen | `no_orphaned_tokens`, `player_consistency` | same two — **no new check class** *(⚠️ read off the binary's 5-game detail sample, not a tally — see the fix cycle's HIGH 2; the by-`check` histogram did not exist yet)* |

Notable: not one game now hits the turn cap, and `HaltReason::InfiniteLoop` did **not** appear
either — commands/turn went *down*, 58.8 → 46.3, so risk 2 of plan §8 (`max_commands` binding
before `max_turns`) did **not** materialise at these settings. That number is the input
`OOS-DX22-2` asks for. No crash, no abort, no new violation class surfaced at this stage
(plan §8 risk 1 did not fire here).

**Stage-2 gates**: `cargo build --workspace` OK · `cargo test -p mtg-simulator` **173 passed /
0 failed** (+3 = P2/P3/P4) · `clippy --workspace --all-targets -D warnings` exit 0 ·
`cargo fmt --check` exit 0 · `tools/check-defs-fmt.sh` clean.

### - [x] Stage 3 — CR 903.6 / 903.8 / 903.9 commander registration

**P11 CONFIRMED RED ON THE PRE-FIX TREE, BY EXECUTION** — this is the plan's strongest
discrimination proof and it was taken first, before the fix:

```
thread 'test_dx22_command_zone_placement_and_registration_are_one_operation' panicked at
crates/simulator/tests/pb_dx22_fuzz_instrument.rs:288:5:
CR 903.6: these files place an object in a command zone without ever calling
`player_commander`, so `commander_ids` stays empty and every commander rule is silently
inert there: ["src/fuzz_setup.rs", "tests/local_game.rs"]
test result: FAILED. 4 passed; 1 failed
```

The gate walks `crates/simulator/{src,tests}` from `CARGO_MANIFEST_DIR`; **5 files** contain
`in_zone(ZoneId::Command(` (`src/setup.rs`, `src/fuzz_setup.rs`, `src/legal_actions.rs`,
`tests/commander_cast.rs`, `tests/local_game.rs`), which clears the ≥4 non-vacuity floor, and
exactly the 2 named were offenders.

**Fix**:

* `fuzz_setup.rs::place_registered_deck` — `builder.player_commander(pid, deck.commander.clone())`
  placed adjacent to the `in_zone(ZoneId::Command(pid))` object, inside the same `if let
  Some(def)` so the two cannot separate. The `setup.rs:381-393` rationale is restated at the
  function.
* `fuzz_setup.rs::build_fuzz_state` — `register_commander_zone_replacements(&mut state)` after
  `builder.build()` (CR 903.9b, plan §B1).
* `crates/simulator/tests/local_game.rs::build_state` rewired onto `place_registered_deck`; its
  `:56-59` doc corrected from "Mirrors `mtg-fuzzer::run_single_game`'s builder logic" to the
  account of why the mirror existed and what it inherited. Import list gains
  `place_registered_deck`.

**Probes P5 / P6 / P7 / P8 added. Reverts EXECUTED, rebuilds confirmed:**

*Revert A — delete `builder.player_commander(..)`* (rebuild OK, `Compiling mtg-simulator`
present). Result **5 passed / 4 failed**, and each of the four fails on its own subject rather
than a shared symptom:

| probe | message |
|---|---|
| **P5** `..._commander_ids_are_registered_by_both_build_paths` | `:355` `left: []` / `right: [CardId("samut-voice-of-dissent")]` |
| **P6** `..._cr_903_9b_replacements_are_registered` | `:411` `left: 0` / `right: 8` |
| **P7** `..._cr_903_9a_zone_return_sba_is_reachable_from_the_fuzz_build` | `:477` `CR 903.9a: the SBA must offer P1 the choice ...; pending = []` |
| **P8** `..._cr_903_8_tax_applies_on_the_fuzz_build` | `:525` `left: generic 3` / `right: generic 5` — i.e. the tax silently vanished |

*Revert B — delete only `register_commander_zone_replacements(&mut state)`* (rebuild OK; the
revert was written as `let _revert_proof = register_commander_zone_replacements; let _ = &mut
state;` precisely so `-D warnings` could not turn it into a build failure). Result **8 passed /
1 failed**: only **P6** reddens (`left: 0` / `right: 8`).

**That isolation CONFIRMS plan §B1's claim by experiment**: under revert B, **P7 stays GREEN**.
CR 903.9a's graveyard/exile return is a state-based action keyed on `commander_ids` and does
not depend on the CR 903.9b replacements at all — exactly as §B1 argued, now measured rather
than asserted. What omitting the call breaks is CR 903.9b *silently*, which is the vacuity
§B1 warns about.

**Plan §D verification, EXECUTED not assumed**: `cargo test -p play-server` → **78 passed / 0
failed**, no fixture-drift message. No play-server seed pin moved, as the plan's chain predicted
(`session::new_game` builds through `setup::build_initial_state`, which this batch does not
touch). `git diff main..HEAD --numstat -- crates/engine/ crates/card-defs/ crates/card-types/
crates/view-model/ tools/` is **EMPTY**.

**`crates/simulator/tests/local_game.rs` re-roll**: plan §4 predicted "expected: all pass
unchanged" and that is what happened — **23 passed / 0 failed**, no pin re-derived, no
numerical adjustment made.

**Stage-3 fuzz run** (`/tmp/pb-dx22-fuzz-after-stage3.txt`, same command):

| metric | Stage 0 | Stage 2 | Stage 3 |
|---|---|---|---|
| games completed | 20 | 20 | 20 |
| wall time | 40.7 s | 14.3 s | **12.5 s** |
| wins / draws / errors | 9 / 0 / 11 | 20 / 0 / 0 | **20 / 0 / 0** |
| total violations | 1,519 | 504 | **426** |
| avg turns | 191.7 | 112.7 | **103.4** |
| commands per turn | 58.8 | 46.3 | **45.7** |
| `check`s seen | orphaned_tokens, player_consistency | same | orphaned_tokens, player_consistency, **`attachment_validity` (NEW)** |

#### FINDING — a pre-existing engine defect became reachable (plan §8 risk 1)

`[attachment_validity] Object ObjectId(532) attached to ObjectId(677) which doesn't exist
(turn 88)`, ×3, **game seed 5**. Zero occurrences of this check in the Stage-0 and Stage-2
runs; first appearance at Stage 3.

* **Repro (deterministic, re-run and confirmed identical ObjectIds and turn)**:
  `cargo run --profile fuzz --bin mtg-fuzzer -- --replay 5 --players 4 --max-turns 200`
  **Fix-cycle correction: this is 3 of 11 violations in 1 of 3 affected games.** The full
  by-`check` histogram gives `attachment_validity` **11 violations across seeds 5, 9 and 15**
  (5: `532 → 677` turn 88 ×3; 9: `1158 → 928` turn 145 ×4; 15: `573 → 678` turn 94 ×4). The
  original "×3, seed 5" was read off the detail loop, which prints the first five offending
  games only. Each game reports the violation at exactly ONE turn and then runs to a winner —
  the shape of the `OOS-M11-7` transient SBA-lag class, i.e. a false-positive candidate.
* **Check**: `crates/simulator/src/invariants.rs:386 check_attachment_validity` — a
  battlefield object whose `attached_to` names an `ObjectId` that no longer resolves. CR 400.7
  (a zone change makes a NEW object) / ~~CR 704.5n~~ **CR 704.5m** (an Aura attached to an
  illegal or absent object is put into its owner's graveyard as an SBA). **The cite was wrong
  as filed and is corrected here and in the audit row** — 704.5n is the Equipment/Fortification
  rule and prescribes the opposite disposition ("it becomes unattached … It remains on the
  battlefield"). Verified against the CR text via MCP during the fix cycle.
* **Why this is not a regression this batch caused**: `git diff main..HEAD --numstat --
  crates/engine/` is **empty** — 0 engine lines. Per plan §8 that is the strongest available
  argument, and it is stronger than attempting to reproduce on the merge base, which *cannot
  shuffle* and therefore cannot reach turn-88 board states of this kind.
* **NOT FIXED, by instruction and by `memory/conventions.md`'s default-to-defer.** Captured
  here for Stage 5 seed filing.

**Stage-3 gates**: `cargo build --workspace` OK · `cargo test -p mtg-simulator` **178 passed /
0 failed** (+5 = P5/P6/P7/P8/P11) · `cargo test -p play-server` 78/0 · `clippy --workspace
--all-targets -D warnings` exit 0 · `cargo fmt --check` exit 0 · `check-defs-fmt.sh` clean.

### - [x] Stage 4 — the ordinary-depth probes

**P9** `test_dx22_a_spell_is_cast_at_an_ordinary_depth` — seeds `[1,2,3,4]`, 4 seats,
`RandomBot` seeded exactly as `run_single_game` seeds its bots, `LocalGameLimits { max_turns:
30, max_commands: 30*400, max_consecutive_passes: 500, record_journal: true }`, `human_seats`
empty. Asserts a `CommandRecord` with `Command::CastSpell(..)` at `turn <= 30` for **every**
seed, and PRINTS the observed turn. Wall time for all four games: **0.90 s**, so the plan's
"reduce to seeds `[1,2]` if it exceeds 60 s" escape hatch was not needed.

```
P9 seed 1: first CastSpell on game turn 17
P9 seed 2: first CastSpell on game turn 9
P9 seed 3: first CastSpell on game turn 25
P9 seed 4: first CastSpell on game turn 23
```

**THE BINDING RULE FIRED, AND THE GATE WAS NOT MOVED.** The plan says: record the observed
turn on every seed, and *if any exceeds 15, do NOT raise the gate to fit the data — investigate
and report*. **Three of the four exceed 15** (17, 25, 23). The gate stays at **30**, exactly as
the plan specified it; nothing was tuned. The investigation is below.

**P10** is a measurement, not a test (a commander needs 3-6 lands, which a 30-turn debug probe
cannot reliably reach, and a statistical assertion in the suite is the flake class this project
bans). Measured with a scratch `crates/simulator/examples/dx22_p10.rs` on the POST-FIX build
path — 20 games, base seed 1, `--max-turns 200`, `--profile fuzz`, journal on — then
**deleted**; `git ls-files crates/simulator/examples` is empty and `crates/simulator/examples/`
no longer exists.

| event | before (pre-plan measurement) | after (20 games) |
|---|---|---|
| `CommanderCastFromCommandZone` | **0** in ~56,800 commands / 5 games | **36**, in **16 of 20** games (first cast typically game turn 38-107; seed 8 as early as turn 3) |
| `CommanderReturnedToCommandZone` (CR 903.9a) | 0 | **13** |
| non-empty `commander_damage_received` (CR 903.10a) | 0 games | **16 of 20** games |
| `CommanderZoneRedirect` (CR 903.9b) | 0 | **0** — see the finding below |
| `SpellCast` | 25/21/35/7/33 per game at turns 143-154 | **670** across 20 games |

**`OOS-SIM1-4`'s closure condition (c) is SATISFIED — the count is 36, not 0, so no STOP.**
CR 903.8, CR 903.9a and CR 903.10a are all now exercised by the fuzzer for the first time.

#### FINDING — the plan's "≤15" expectation for P9 is refuted by measurement

Over 20 seeds the first-cast game turn is **min 3 / median 12 / max 29**, sorted:
`[3, 5, 5, 6, 8, 9, 9, 10, 10, 11, 12, 17, 17, 18, 18, 18, 23, 25, 26, 29]`.

Against a gate of 30 that is a margin of **one turn** at 20 seeds. P9 itself uses seeds
`[1,2,3,4]` (max 25) so it is not currently at risk, but a successor that widens the seed set
without reading this will get a flake, and the correct response will still be to investigate,
not to raise the gate.

**Mechanism, measured not guessed.** The same run records the first `Command::PlayLand` turn:
it is **1-7 on every one of the 20 seeds**, so land availability is *not* the limiter (seed 14
plays its first land on turn 1 and still does not cast until turn 29). The limiter is §B2's own
declined item, **`OOS-DX22-1` — no opening hand (CR 103.5)**: every seat starts with **zero**
cards and draws one per *personal* turn, so by game turn *T* in a four-player game a seat has
drawn only about *T*/4 cards. The plan's §B2 reason 2 says "Seven extra opening cards move that
by ≤1-2 personal draws" — which is true, and is precisely the point: **1-2 personal draws is
4-8 GAME turns at four seats**, and P9's threshold is stated in game turns. The plan converted
personal draws to game turns when arguing the pre-fix floor (draw ~35-40 ⇒ turn ≈136-156) and
did not convert them when predicting the post-fix band. `RandomBot`'s uniform choice adds the
rest of the spread.

This does not weaken any closure: the pre-fix floor was 143-154 and the post-fix band is 3-29,
so `OOS-UI2-1` and `OOS-SIM3-1` are closed by an order of magnitude either way. It sharpens
`OOS-DX22-1` from "would not change anything measurable" to "is the measured reason the band is
3-29 rather than 3-12", which is the number the successor needs.

#### FINDING — CR 903.9b is registered but still never exercised

`CommanderZoneRedirect` fired **0 times in 20 games**, even though P6 proves the eight
replacement effects exist on every built state. Nothing in those games bounced a commander to
its owner's hand or shuffled one into a library. So the CR 903.9b half of the commander rules
is now *reachable* but still *unreached* by the fuzzer at this sample size — which is exactly
the vacuity §B1 warned about, one step removed: before this batch a zero would have meant "the
mechanism does not exist"; now a zero means "no game happened to trigger it". The distinction
is only visible because P6 exists. Worth a seed at Stage 5.

#### FINDING — P9's revert-proof does NOT work the way the plan says

The plan's row for P9 gives the revert as "delete the shuffle → no cast before turn ~136,
every seed reddens". **Executed on the Stage-4 tree, that is false.** With the shuffle removed
but the commander still registered:

```
P9 seed 1: first CastSpell on game turn 26
P9 seed 2: first CastSpell on game turn 25
P9 seed 3: first CastSpell on game turn 25
panicked ...: CR 103.3: seed 4 cast no spell at all within 30 turns (1094 commands recorded)
test result: FAILED. 0 passed; 1 failed
```

Only **one of four** seeds reddens. The reason is a genuine confound the plan did not
anticipate: a **registered commander is cast from the command zone**, which is not in the
library and therefore is not gated by library order at all — and `Command::CastSpell` is the
same command either way. Stage 3's fix partially *masks* Stage 2's fix from this probe.

The discrimination that matters was then executed: reverting **both** fixes, i.e. the
merge-base behaviour, fails on the very first seed —

```
CR 103.3: seed 1 cast no spell at all within 30 turns (1073 commands recorded)
test result: FAILED. 0 passed; 1 failed
```

— rebuild confirmed (`Compiling mtg-simulator`) on both runs. So P9 does discriminate against
the tree it exists to discriminate against; it is simply not a single-variable probe for the
shuffle, and the plan's row overstates it. P9's doc comment now records this measurement in
place of the plan's claim (`memory/conventions.md`'s aspirationally-wrong-comment rule); the
gate itself was **not** changed.

**Stage-4 gates**: `cargo build --workspace` OK · `cargo test -p mtg-simulator` **179 passed /
0 failed** (+1 = P9) · `clippy --workspace --all-targets -D warnings` exit 0 · `cargo fmt
--check` exit 0 · `tools/check-defs-fmt.sh` → `1803 defs checked / clean`.

**FULL-WORKSPACE RE-MEASURE** (`--workspace --no-fail-fast` to `/tmp/pb-dx22-final.txt`, never
tail-piped, summed with awk over 42 `^test result` lines): **4,355 passed / 0 failed / 5
ignored**. That is **+10** over the Stage-0 baseline of 4,345, and 10 is exactly the probe count
(P1-P9 + P11). Residual failure list **empty** — zero `failures:` / `FAILED` / `error[` lines.

**Wire sentinels re-executed** (not predicted): `hash_schema` and `protocol_schema` green;
`HASH_SCHEMA_VERSION = 72`, `PROTOCOL_VERSION = 35` — **unmoved**, as they must be, since the
branch touches 0 engine lines.

**Footprint, verified by diff rather than claimed**:

* `git diff main..HEAD --numstat -- crates/engine/ crates/card-defs/ crates/card-types/
  crates/view-model/ tools/` → **EMPTY**.
* `git diff main..HEAD -- crates/simulator/src/setup.rs` → **EMPTY** (0 lines, not even doc
  comments — the plan's §5 `setup.rs` doc correction is Stage 5's).
* Card coverage unmoved by construction: 0 lines in `crates/card-defs`.
* `git ls-files crates/simulator/examples` → empty; the directory does not exist.
* Changed files, whole branch: `crates/simulator/src/{bin/fuzzer.rs, fuzz_setup.rs, lib.rs}`,
  `crates/simulator/tests/{local_game.rs, pb_dx22_fuzz_instrument.rs}`, plus `memory/`.

---

### - [x] Stage 4b (unplanned) — the second half of `tests/local_game.rs`'s Commander game

Commit `eb60cc80`, found after Stage 4 and outside the plan. Stage 3 rewired
`tests/local_game.rs::build_state` onto `place_registered_deck`, which fixed its *placement +
registration* half — but that file's own post-`build()` step never called
`register_commander_zone_replacements`, so CR 903.9b was inert in every game it drove: the
same half-built Commander game one link down from the defect `OOS-SIM1-4` names. Fixed, with
probe `test_dx22_cr_903_9b_replacements_exist_in_the_fixed_deck_build` proven red by an
executed revert (**left 0 / right 8**). That is the +1 that takes the branch's probe count to
11 and its test total to 4,356. The third site, `tests/commander_cast.rs`, has the same shape
and was deliberately **left alone** — it is a focused CR 903.8 tax fixture that never bounces
a commander, so widening P11 into "every registrar must also install the 903.9b redirects"
would have fired on legitimate code. Recorded as `OOS-DX22-10`.

---

## Stage 5 — corrections + seed filing DONE; bookkeeping still the coordinator's

**DONE (`6e7988cd` corrections, `8c23e1f0` seeds).** Nine correction sites and the seed
filing shipped. Still NOT done and deliberately left: the `CLAUDE.md` Current State delta,
`memory/workstream-state.md`'s handoff section, and **`memory/primitives/seed-rerank-2026-08-02.md`
— untouched by instruction** (the coordinator strikes the §4 row and settles §2.4's open
measurement at collect; the settled answer lives in the plan, this file, and
`docs/mtg-engine-feedback-engineering.md` §2.1's status block instead).

**Corrections made (aspirationally-wrong-comment rule, `memory/conventions.md`)** — each
past-tenses the record rather than deleting it, and each carries the measured replacement:
`crates/simulator/src/legal_actions.rs` (SIM-1's structural-unreachability argument →
`OOS-SIM1-4` CLOSED, seeds DID move, play-server 78/0 and `tests/local_game.rs` 23/23 did
not), `src/local_game.rs` (SIM-5's premise closed, its parity claim retired **not**
re-validated), `src/invariants.rs` (the SIM-3 A/B table dated as an unshuffled measurement +
the post-shuffle re-measure: 426 violations, **0** `stack_consistency`), `src/setup.rs`
(**doc lines only, verified: 0 non-`//` lines in the whole-branch diff** — the "not rewired"
claim is still true, so it now names the three things that still differ), `src/bin/fuzzer.rs`
(PB-DX22 appended as a third seed-portability boundary event), `tests/local_game.rs` (UI-2's
25,964-observation block), `docs/mtg-engine-simulator.md`, `docs/mtg-engine-feedback-engineering.md`
(row 1 SHIPPED + its open measurement recorded SETTLED), `memory/workstream-state.md:2903`
(the `--seed 504` repro annotated dead across this merge).

**Seeds filed** in `docs/audits/decision-point-audit.md` §8.1: closures appended in-row to
**`OOS-SIM1-4`** (CLOSED), **`OOS-UI2-1`** (CLOSED, "never" corrected to a `--max-turns 80`
threshold) and **`OOS-SIM3-1`** (CLOSED, turn 143 retained as the calibration constant), plus
a §8.1 banner and eleven new rows **`OOS-DX22-1..11`** — four more than the plan's seven, the
extras being `-8` (the `attachment_validity` find), `-9` (CR 903.9b registered but
unexercised), `-10` (`commander_cast.rs`'s half-built shape, judged acceptable, and why P11
was not widened) and `-11` (the method seed: P4's and P9's stated revert-proofs were both
wrong and were re-derived by execution).

**Stage-5 gates**: `cargo build --workspace` OK · `cargo test -p mtg-simulator` **180 passed /
0 failed** · `clippy --workspace --all-targets -D warnings` exit 0 · `cargo fmt --check` exit 0
· `tools/check-defs-fmt.sh` 1803 clean · full workspace `--no-fail-fast` to a file over 42
targets: **4,356 passed / 0 failed / 5 ignored**, residual list empty. (4,355 at Stage 4, +1
for the `eb60cc80` probe added after that measurement.) The forbidden-path diff
(`crates/engine/`, `crates/card-defs/`, `crates/card-types/`, `crates/view-model/`, `tools/`)
is still **EMPTY** across the whole branch.

### Original Stage-5 note (kept as filed)

The A/B measurement write-up, the 10 comment/doc corrections in plan §5, seed filing
(`OOS-DX22-1..7` plus the three new ones below), and the `CLAUDE.md` /
`memory/workstream-state.md` bookkeeping are Stage 5 and were deliberately not started.

**New seed candidates this run surfaced, with their evidence already captured above:**

* **`attachment_validity` on fuzz seed 5, turn 88** — a pre-existing engine defect made
  reachable. Repro `mtg-fuzzer --replay 5 --players 4 --max-turns 200 --profile fuzz`;
  0 engine lines in the branch diff.
* **P9's band is 3-29, not ≤15**, and the cause is `OOS-DX22-1` (no opening hand) measured
  rather than assumed — this sharpens that seed's justification and gives it the number it
  needs.
* **CR 903.9b is registered but unexercised** — `CommanderZoneRedirect` = 0 in 20 post-fix
  games, while P6 proves the 8 replacements exist.

**Numbers Stage 5 needs and already has**: commands/turn **58.8 → 46.3 → 45.7** across the
three stages (the input `OOS-DX22-2` asks for; `HaltReason::InfiniteLoop` never appeared and
the turn cap stopped being reached at all, so `max_commands` did **not** start binding first).

### Probe → revert-proof ledger (all EXECUTED, all rebuilds confirmed)

| probe | revert executed | observed failure |
|---|---|---|
| P1 | `place_registered_deck` library loop → `.iter().take(98)` | `left: 98 / right: 99` |
| P2 | `deck.main_deck.shuffle(&mut rng)` → `truncate(99)` no-op | `left != right` — library IS the construction order |
| P3 | `shuffle(&mut rng)` → `shuffle(&mut rand::rng())` | seed 1 did not reproduce seat 1's order |
| P4 | `seed_from_u64(seed)` → `seed_from_u64(0)` | seeds 1 and 2 dealt the same library (**plan's stated revert does not work — see Stage 2**) |
| P5 | delete `builder.player_commander(..)` | `left: [] / right: [CardId("samut-voice-of-dissent")]` |
| P6 | delete `register_commander_zone_replacements(..)` only | `left: 0 / right: 8` — **and P7 stays green, confirming §B1** |
| P7 | delete `builder.player_commander(..)` | `pending = []` |
| P8 | delete `builder.player_commander(..)` | effective cost `generic 3` where `generic 5` was required |
| P9 | delete shuffle **and** registration (merge-base) | seed 1 cast no spell in 30 turns / 1,073 commands (**plan's stated revert reddens only 1 of 4 — see the finding above**) |
| P11 | **none needed — RED on the pre-fix tree**, on exactly `["src/fuzz_setup.rs", "tests/local_game.rs"]` | recorded verbatim in Stage 3 |

---

## Fix cycle — all 13 review findings TAKEN, 0 deferred, 0 filed-instead-of-fixed

**Phase**: fix · **Review**: `memory/primitives/pb-review-DX22.md` (2 HIGH, 4 MEDIUM, 7 LOW).

**Headline**: the review's HIGH 1 asked for the deleted instrument back. It came back as part
of the fuzzer itself — `mtg-fuzzer` now prints a **violation-by-`check` histogram over every
game** and a **commander-mechanics and first-cast census over every game**, so both HIGHs are
settled by the same change, and PB-DX22's "after" side now has the same standing as its
committed "before" side. Raw run: `memory/primitives/pb-dx22-measurement-after-fixcycle.txt`.

### The re-measurement, against the six published numbers

`cargo run --profile fuzz --bin mtg-fuzzer -- --games 20 --seed 1 --max-turns 200 --threads 1
--verbose`, i.e. exactly the command the batch published under.

| published by the deleted instrument | re-derived by the shipped instrument | verdict |
|---|---|---|
| `CommanderCastFromCommandZone` **36**, in **16 of 20** games | 36, in 16 of 20 | **exact** |
| CR 903.9a returns **13** | 13 | **exact** |
| non-empty `commander_damage_received` in **16 of 20** | 16 of 20 | **exact** |
| `SpellCast` **670** | 670 | **exact** |
| first-cast band **3-29** | min 3 / median 12 / max 29 | **exact** |
| first `PlayLand` turn **1-7** | min 1 / median 2 / max 7 | **exact** |
| (also) total violations **426**, avg turns **103.4**, 20 wins / 0 errors | 426 / 103.4 / 20 / 0 | **exact** |
| `CommanderZoneRedirect` **0** | 0 | **exact** |

**No discrepancy in any published number.** The deleted instrument was accurate; it was
unreproducible, which is a different defect and is the one that is now closed.

**Three things the re-measurement adds that the published set did not contain**, each of which
changes how an existing claim should be read:

1. **The by-`check` tally is 301 `no_orphaned_tokens` (15 games) + 114 `player_consistency`
   (5 games) + 11 `attachment_validity` (3 games) + **0** `stack_consistency` = 426.** The
   bolded universal negative SURVIVES. Its 94-line sample did **not**: it projected
   `player_consistency` at ~1% of the run (it is **27%**) and `attachment_validity` at 3
   violations in 1 game (it is **11 in 3**). Restated at `invariants.rs` as a table, with the
   sampling failure named. Filed as the generic class in `OOS-DX22-13`.
2. **`attachment_validity` fired in seeds 5, 9 AND 15, not seed 5 alone** — and in each game at
   exactly one turn, after which the game runs to a winner. That is the shape of the
   `OOS-M11-7` transient SBA-lag class, so it is a live candidate for the same false-positive
   family SIM-3 withdrew from `stack_consistency`. Recorded in `OOS-DX22-8` as the successor's
   first check.
3. **`max_commander_damage` = 31**, past CR 903.10a's threshold of 21. The commander-damage
   *loss condition itself* is now reachable under automated exercise, not merely the counter.

### Finding-by-finding

| # | sev | disposition |
|---|---|---|
| 1 | HIGH | **FIXED, both parts.** (a) `bin/fuzzer.rs::print_mechanics_summary` + `print_violation_histogram`, fed by a new constant-size `MechanicsTally` folded from events in `local_game.rs` (no journal, so `GameDriver`'s `record_journal: false` is untouched) and returned by a new `GameDriver::run_game_with_mechanics`. **Not a new field on `GameResult`**: `tools/play-server/src/main.rs:3326` constructs one and this batch may not touch `tools/`. (b) new probe **P12** `test_dx22_cr_903_10a_commander_damage_is_recorded_on_the_fuzz_build` — real combat on the fuzz-built state (`handle_declare_attackers` → `apply_combat_damage` → `check_and_apply_sbas`), plus **P13** gating the census itself against vacuity. |
| 2 | HIGH | **FIXED by obtaining the real tally**, not by re-wording to the sampled scope. See the table above; `invariants.rs`'s block restated, `OOS-DX22-3` restated, `OOS-DX22-13` filed for the generic class. |
| 3 | MED | **FIXED.** `bin/fuzzer.rs`'s boundary-event doc is now a 3-row table with the instrument AND denominator per side (5 games before / 20 after); `docs/mtg-engine-simulator.md` likewise; `docs/mtg-engine-feedback-engineering.md` (a third site the review did not name) carries the refinement pointer. |
| 4 | MED | **FIXED, and it settled the open question.** P9 excludes command-zone casts twice over — by the four command-zone `ObjectId`s read off the STARTED state, and by rejecting any record carrying a `CommanderCastFromCommandZone` event (which also covers a post-CR-903.9a recast under a new id, which the id set cannot see). **The turns did not move: 17/9/25/23 before and after.** Seeds 1-4's first casts always were library casts — the thing the batch could only call "probably". |
| 5 | MED | **FIXED.** Both needles split with `concat!` so the gate no longer matches itself; census is now exactly **4** and is printed. Floor re-derived and stated **by name** (`src/fuzz_setup.rs`, `src/legal_actions.rs`, `src/setup.rs`, `tests/commander_cast.rs`) rather than by count, so a file that stops placing reddens while a new placer does not. |
| 6 | MED | **FIXED, and both additions made.** CR text fetched via MCP: 704.5m is the Aura rule (graveyard), 704.5n is Equipment/Fortification (unattach, stays). Row corrected in both `docs/audits/decision-point-audit.md` and this file, both dispositions kept since the orphan's card type is unknown, and the two successor checks added — (a) transient-vs-persistent with the one-turn-per-game evidence, (b) the commander-zone-change mechanism (13 CR 903.9a returns = CR 400.7 events this batch uniquely enabled). |
| 7 | LOW | **FIXED.** P5's doc now says half (b) is a reconstruction with no discrimination over half (a), and names `build_state`'s two real gates (P11 by source walk; the stage-4b probe by output). |
| 8 | LOW | **FIXED.** P2 loops seeds `[1, 8]`; **seed 8 seat `PlayerId(3)` draws `rograkh-son-of-rohgahh`** (empty colour identity), so the CR 903.5c colourless padding arm is exercised for the first time by any probe. Found by enumerating seeds 1..=120 (hits 8, 50, 73, 119 — all the same card, the only `Complete` colourless legendary creature in the pool). The test asserts the arm was taken, so losing it reddens. |
| 9 | LOW | **FIXED, not filed.** `place_registered_deck` builds a local `HashMap<&CardId, &CardDefinition>` once per seat (the `setup.rs::find_def` index), replacing an O(defs) scan per card. Built with `entry().or_insert()`, not `collect()`, so a duplicated `card_id` keeps the FIRST def exactly as `Iterator::find` did — behaviour-identical. |
| 10 | LOW | **FIXED.** `debug_assert_eq!(deck.main_deck.len(), 99, ...)` at `build_fuzz_state`'s per-seat loop, mirroring `setup.rs`. |
| 11 | LOW | **FIXED.** The "both branches advance the stream identically" claim is replaced with the honest one ("nothing here depends on cross-branch alignment"; `shuffle` is rejection-sampled and data-dependent), and the fallback branch now says it would place no commander at all if `teysa-karlov` left the pool. |
| 12 | LOW | **FIXED.** `FuzzGameSetup::decks` → **`decklists`**, with the reason at the field. |
| 13 | LOW | **SATISFIED, recorded rather than re-run.** The coordinator ran `tools/authoring-report.py` at collect: the report body came back **byte-identical except the git-sha stamp line**; coverage **1,133 clean / 1,803 = 62.8%**, unmoved. Consistent with the by-construction argument (`git diff main..HEAD -- crates/card-defs` is empty), and it is now the executed check the plan §7 asked for rather than a substitution. |

### Revert-proof ledger for the fix cycle (all EXECUTED, all rebuilds confirmed)

| revert executed | rebuild | observed |
|---|---|---|
| delete `builder.player_commander(..)` (replaced by a `let _revert_proof = GameStateBuilder::player_commander;` so `-D warnings` could not turn it into a build failure) | `Compiling mtg-simulator` present | **6 failed / 6 passed** — P5, P6, P7, P8 as before, **plus P12** (`left: None / right: Some(3)`, `commander_damage_received = {}`) and **P13** (`commander_casts_from_command_zone: 0, seats_dealt_commander_damage: 0`) |
| `deck.main_deck.shuffle(&mut rng)` → `truncate(99)` no-op (+ `#[allow(unused_imports)]` on the `SliceRandom` import — the same `-D warnings` trap, hit a THIRD time in this batch and worked around the same way) | present | **P9 reddens on seed 1**, and, re-run with the seed list narrowed to each of `[2]`, `[3]`, `[4]` in turn, **on all four seeds individually**. That is the plan's original prediction, delivered for the first time — the pre-fix probe reddened only 1 of 4 |
| P2's seed list `[1, 8]` → `[1, 1]` | present | P2 reddens on the colourless-commander non-vacuity assertion |
| P11's walk `vec![src, tests]` → `vec![src]` | present | P11 reddens by NAME on `tests/commander_cast.rs` — the by-name floor discriminates where a bare count would not have |
| `MechanicsTally::record` made a no-op | present | P13 reddens: `spell_casts: 0, first_spell_cast_turn: None, lands_played: 0, …` |

### Fix-cycle gates, all EXECUTED

* `cargo build --workspace` OK · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 ·
  `cargo fmt --check` exit 0 · `tools/check-defs-fmt.sh` → `1803 defs checked / clean` (SR-35).
* Full workspace `--no-fail-fast` to `/tmp/pb-dx22-fixcycle-final.txt`, summed with awk over
  **42** `^test result` lines: **4,358 passed / 0 failed / 5 ignored**. That is **+2** over the
  batch's 4,356, and 2 is exactly the new probe count (P12, P13). Residual list **empty** — zero
  `failures:` / `FAILED` / `error[` / `error:` lines.
* Wire sentinels **re-executed**, not predicted: `hash_schema` 21/0, `protocol_schema` 17/0;
  `HASH_SCHEMA_VERSION = 72`, `PROTOCOL_VERSION = 35` — **unmoved**.
* `git diff main..HEAD --numstat -- crates/engine/ crates/card-defs/ crates/card-types/
  crates/view-model/ tools/` → **EMPTY** (and the working tree likewise).
* `git diff main..HEAD -- crates/simulator/src/setup.rs` → 27 changed lines, of which **0** are
  non-comment lines.

### New seeds filed by the fix cycle

* **`OOS-DX22-12`** — the published "first cast turn 3-29" band mixes command-zone commander
  casts with library casts; the shuffle-gated band is **5-29 / median 17**. Everywhere the
  batch reasons from library depth, that is the number that belongs. No conclusion changes.
* **`OOS-DX22-13`** — **method**: the 5-game detail cap is a property of the binary, so every
  historical "check X never fired in a fuzz run" claim (including SIM-3's 90.3% share and
  `OOS-SIM3-4`'s 929-of-938 noise floor) was read off a sample unless it says otherwise. Not a
  retraction; a filing that they were never re-derived from a complete tally, which
  `print_violation_histogram` now makes a one-command job.

Rows `OOS-DX22-3`, `OOS-DX22-8` and `OOS-DX22-11` were **corrected in place** rather than
re-filed, each keeping the original text and naming what changed.

---

## Close-out (2026-08-03)

* **Review cycle 1** — `primitive-impl-reviewer`, findings in `memory/primitives/pb-review-DX22.md`:
  2 HIGH / 4 MEDIUM / 7 LOW, **all 13 taken** (`b79a943a`). The two HIGHs were the batch's own
  defect class: the headline "after" numbers came from a deleted scratch instrument, and a
  universal negative over 426 violations was evidenced by the 94 the binary prints. Both repaired
  by making `mtg-fuzzer` report an all-games violation histogram and mechanics census, then
  re-measuring — every published number matched to the digit.
* **Review cycle 2** — the `/review` skill's Opus reviewer, **with shell access** (cycle 1 had
  none, so it could verify no git-diff or measurement claim). It re-executed the full suite, both
  version gates, the play-server pins, clippy/fmt/defs-fmt, the whole 20-game fuzz A/B, **and
  built the merge base in a throwaway worktree to reproduce the before side**. All five acceptance
  clauses PASS (clause 5 partial by design at the time). 4 findings, **all 4 taken** (`59ad18a1`):
  a comment-satisfiable P11 source gate (the only code change — proven by an executed revert that
  truly deletes the call), a 5-game-vs-20-game denominator mismatch in `invariants.rs`, a stale
  "23 passed" in four shipped places (stage 4b made it 24), and an audit banner saying eleven rows
  where thirteen were filed.
* **Coverage** — `tools/authoring-report.py` regenerated; body byte-identical except the git-sha
  stamp line. **1,133 clean / 1,803 = 62.8%, unmoved.** The regeneration churn was reverted, since
  the batch changed 0 card defs.
* **Final gates**: `--workspace --no-fail-fast` to a file → **4,358 / 0 / 5 over 42 targets**,
  residual list empty. `hash_schema` 21/0 and `protocol_schema` 17/0 EXECUTED; declared
  **HASH 72 / PROTOCOL 35**, unmoved. `clippy --workspace --all-targets -D warnings`, `fmt
  --check` and `tools/check-defs-fmt.sh` (1803 defs) all clean. Forbidden-path branch diff
  (`crates/engine/`, `crates/card-defs/`, `crates/card-types/`, `crates/view-model/`, `tools/`)
  **EMPTY**; `crates/simulator/src/setup.rs` doc-comment lines only.
* **Close-out written**: lean CLAUDE.md bullet (2026-08-02 `memory/decisions.md` schema) + a new
  `Tests (delta 2026-08-03, PB-DX22)` pin; full handoff at the head of
  `memory/workstream-state.md`; `OOS-DX22-1..13` filed and `OOS-UI2-1` / `OOS-SIM3-1` /
  `OOS-SIM1-4` closed in `docs/audits/decision-point-audit.md` §8.1.
  `memory/primitives/seed-rerank-2026-08-02.md` untouched by design.

# Feedback Engineering for Alpha

<!-- last_updated: 2026-08-03 -->

> **Status: STRATEGY + DISPATCH PROPOSAL.** **Row 1 (≡ PB-DX22) is SHIPPED** as of 2026-08-03
> (`scutemob-196`, merge `95f53b78` — see §2.1); rows 2-8 are unimplemented. This file is written in
> the shape of `memory/playtest-triage-2026-08-02b.md`: verified claims with `file:line`, a ranked
> successor table a coordinator can dispatch from verbatim, and dispatch notes.
> **Created**: 2026-08-03 (`scutemob-192`, FEEDBACK-1).
> **Authority**: this doc *proposes*. The coordinator sequences. The PB-DX correctness queue
> (`memory/primitives/seed-rerank-2026-08-02.md` §4, v3) is the standing engine track and nothing
> below displaces it — three of the rows below **already live in it** and are reconciled, not
> duplicated.

---

## 0. Headline

Two human playtests produced **10** findings (`test-data/bot testing notes.md`, 2026-08-01 → F1-F10)
and **13** (`test-data/bot testing notes 2.md`, 55 lines, 2026-08-02 → G1-G13). Of the **17
functional** findings among them (F1-F10 + G1-G7; G8-G13 are UX), the layer distribution is:

| layer | count | findings |
|---|---|---|
| `crates/simulator` | 9 | F3, F4, F5, F6, F7, F8, **G4**, G5, G6 |
| card defs | 3 | F1, F2, F10 |
| **engine** | **2** | G3, G7 |
| `tools/play-server` (Rust) | 2 | F9, G2 |
| frontend (Svelte) | 1 | G1 |
| **total** | **17** | |

(F8, F9 and G4 straddle the simulator's offer layer and the play-server's view layer; each is
counted once, at the layer the fix landed in. G4's fix was `legal_actions.rs` + `view.rs`, so it
sits with the simulator; F9's was the same shape but its 422 came from `casting.rs` refusing a
play-server-forwarded command, so it sits with the play-server. Move either and the headline is
unchanged: **engine 2 of 17.**)

**The test suite is concentrated where the defects are not.** Source-level `#[test]`/`#[tokio::test]`
attribute counts by area (method: `grep -rho '^\s*#\[\(tokio::\)\?test\]' --include=*.rs <dir> | wc -l`
— an attribute census, **not** the 4,345 runtime count, which includes proptest cases and
script-driven tests):

```
4039  crates/engine          169  crates/simulator        27  tools/play-server
  15  crates/view-model       15  tools/replay-viewer      14  crates/card-types
   2  crates/card-db           2  tools/tui                 0  crates/card-defs
```

Plus **271 golden scripts** (208 approved / 63 retired / 0 pending — counted, see §1.4), every one
of which drives the engine and **none** of which touches the provider, the play-server or the
browser (verified by grep, §1.4).

Three facts explain why automation missed nearly all of it, and each is verified below:

1. **The fuzzer has never cast a spell.** `crates/simulator/src/bin/fuzzer.rs:330-339` pushes
   `deck.main_deck` into `ZoneId::Library` in construction order with no shuffle anywhere in the
   file; `random_deck` appends basics **last** (`crates/simulator/src/deck.rs:143-148`);
   `Zone::insert` is a `push_back` (`crates/card-types/src/state/zone.rs:107`) and `Zone::top()` is
   `v.last()` (`:159`), so the basics are drawn first. **Re-verified at HEAD** — SIM-4, SIM-5 and
   SIM-6 never touched `bin/fuzzer.rs` or `deck.rs`. `git log --oneline -- crates/simulator/src/bin/fuzzer.rs
   crates/simulator/src/deck.rs` returns **seven** commits, none of them SIM-4/5/6: `f3ab45d4`
   (scutemob-168), `e658c9d8` (scutemob-168 / PB-DX4, **+61 lines to `deck.rs`** — the
   colourless-commander padding arm, `deck.rs:108-142`), `f2a9647b` (scutemob-147), `878d593a`
   (SR-32), `96a3bd23` (SR-12), `b6274fcb` (SR-10), `34079131` (the feature commit). **The last
   change to either file is 2026-08-01**, before both playtests and before all three SIM batches.
2. **The sensors that would have caught most of the rest were built *after* the playtests.**
   SIM-5's `RejectedCommand` channel (`crates/simulator/src/local_game.rs:165-178`) shipped
   2026-08-02, on the same day, in response to G5.
3. **Where a gate existed, it recorded the defect and stayed green.** See §0.1.

### 0.1 The one thing to read before proposing any gate

`crates/engine/tests/core/decision_site_walk.rs` was added **2026-07-27** (`76b4f1cd`, PB-DP10).
Its `discard_cards` row carried, as data, a verbatim statement of the defect a human found six days
later:

> `why_not_flagged_is_wrong: "CR 701.9b: the affected player chooses which card, by default; the
> engine picks the lowest ObjectId"`

That row was classified `DecisionClass::AutoChosen`, the suite was green the entire time, and G3 —
*"Fell Spector entered, and the bot chose me — card was automatically discarded"* — is exactly that
sentence, observed by a person. ENG-1 flipped the row to `Served { by: "ENG-1" }` at `87594d08`
(`decision_site_walk.rs:319-327`).

The gate is not broken. It says so itself, at `crates/engine/tests/core/decision_gate.rs:12-17`:

> *"It converts **silent** growth into **recorded, reviewed** growth: a `Complete` def newly
> carrying a still-auto-chosen decision fails `T4` until its author either demotes it or adds a
> `BASELINE` entry with a written reason. **This gate cannot stop the growth; it makes it
> recorded**"*

`MAX_AUTO_CHOSEN_COMPLETE_UNION` (`decision_gate.rs:495`) is an **exact-equality** ratchet, currently
**80**, walked 97 (2026-07-27) → 91 (PB-DX4) → 80 (ENG-1). **80 deck-legal `Complete` defs still have
a decision the engine takes for the player.** Every one of them is a G3 waiting for a human.

> **The generalisation this doc is built on: a recording gate converts an unknown defect into a
> known debt, and a known debt is not a caught defect. Nothing in §2 changes that for the 80 —
> only doing the work does. What §2 can do is stop the classes that *are* mechanically
> detectable from reaching a human at all, so a human's hour is spent on the classes that are not.**

---

## 1. Inventory — every existing feedback channel

Line cites are snapshots (OOS-DP6-8 class); re-verify by symbol. Every can-catch / cannot-catch
statement below was derived by reading the code, not by reading a doc about the code.

| # | Channel | Runs in CI? | Catches | Structurally cannot catch |
|---|---|---|---|---|
| 1 | `mtg-fuzzer` binary | **no** | engine aborts, `GameDriverError` halts, the 9 live invariants | anything requiring a spell to be cast (§1.1); anything above `LocalGame` |
| 2 | `crates/simulator/src/invariants.rs` (`check_all`) | via `cargo test --all` only on `LocalGame`-based tests | 9 structural/bookkeeping properties of `GameState` | every rules-correctness question; rejections; waste; decisions (§1.2) |
| 3 | SIM-5 rejection channel (`RejectedCommand`) | no assertion anywhere | **bot** command refusals, retained ≤256, counted uncapped | **human** refusals — not recorded at all (§1.3) |
| 4 | `GET /api/game/report` (`BugReportView`) | one probe | manual export of seed + config + journal + rejections | it computes no metrics; it is not redacted (§1.6) |
| 5 | golden-script corpus + replay harness | yes | 271 scripted engine scenarios, cross-validated (SR-9b) | the provider, the play-server, the browser (§1.4) |
| 6 | `local_game_playthrough.rs` | yes | SR-38 on the **human** path, 5 seeds × 25 turns | anything outside those 5 seeds (§1.5) |
| 7 | play-server `#[test]`s (78) | yes | 51 HTTP probes; 7 frontend **source-text** gates; 3 Rust source gates | that any component **renders** (§1.7) |
| 8 | Architecture-Invariant-7 redaction gates | yes | 5 named leak channels, 6+ gates | a channel nobody enumerated (`OOS-UI6-5`) |
| 9 | R7 frontend harness | — | **does not exist** (§1.8) | — |
| 10 | bug-report artefact | — | seed + config + journal | no free-text; no coded replay path (§1.9) |
| 11 | `docs/mtg-engine-runtime-integrity.md` | — | **PROPOSAL, Layer 1 & 2 unbuilt, Layer 3 partial** (§1.10) | — |
| 12 | CI (`.github/workflows/ci.yml`) | — | fmt, defs-fmt, clippy, build, `cargo test --all` | zero fuzz, zero browser, zero Node (§1.11) |
| 13 | `decision_gate.rs` / `decision_site_walk.rs` | yes | **records** every auto-chosen decision site on a `Complete` def | it cannot stop growth, and it says so (§0.1) |
| 14 | SR-37 printed-field fidelity | yes | wrong mana cost / P/T / type line vs a Scryfall fixture | oracle *text* → DSL semantics (that gap is PB-DX8) |

### 1.1 `mtg-fuzzer` — what it actually is

**CLI** (`crates/simulator/src/bin/fuzzer.rs:58-106`): `--games` (1000), `--players` (4),
`--max-turns` (200), `--seed` (random), `--threads` (num_cpus), `--bot` (`random`|`heuristic`,
default `random`), `--stop-on-error`, `--replay <SEED>`, `--verbose`.
**Note**: the module doc claims a 2-6 range for `--players` (`:8`) and nothing validates it.

`GameDriver` is a pass-through: `run_game` (`crates/simulator/src/driver.rs:62-126`) builds a
`LocalGame` and calls one `advance()`. `check_invariants` is hard-coded `true` (`driver.rs:52`) and
not CLI-exposed. `record_journal` is hard-coded `false` (`driver.rs:77`), which — per
`local_game.rs:867` — also zeroes the retained rejection list.

**Can catch**: a hard abort; `EngineError`/`MaxTurnsReached`/`NoLegalActions`/`InfiniteLoop`; the
nine live invariants. Under `--profile fuzz` (`Cargo.toml:51-54`, `inherits = "release"` +
`debug-assertions` + `overflow-checks`) it also trips the SR-4/SR-14 `state::diagnostics`
tripwires that plain `--release` compiles out — this is how PB-DX19's SIGABRT was found.

**Cannot catch, and this is the load-bearing limitation:**

* **Anything downstream of casting a spell.** Chain re-verified at HEAD, five links, above (§0).
  `OOS-SIM3-1` dates rather than contradicts `OOS-UI2-1`: with 34 basics + ≤5 non-basic lands on
  top and **no opening hand dealt**, the first non-land is personal draw ~35-40 ≈ game turn 136-156,
  and SIM-3 measured the earliest cast at **turn 143**.
  *(The no-opening-hand half is re-verified here directly rather than taken from the queue memo,
  whose `engine.rs:3485-3500` cite has rotted onto an unrelated function: `fuzzer.rs:309-342`
  populates only `ZoneId::Command` and `ZoneId::Library`, and `first_turn_of_game`
  (`crates/engine/src/state/builder.rs:205-208`) only sets a flag — it draws nothing. The
  playthrough test says the same in prose at `local_game_playthrough.rs:344`: "the fuzzer's games
  start with empty hands".)*
  **"Never" is `--max-turns 80`; "from ~turn 143" is the default 200 cap.** Every historical
  "fuzz parity" claim must be read against the `--max-turns` it ran at.
  **✅ FIXED by PB-DX22 (`scutemob-196`, row 1 below).** `fuzz_setup::build_fuzz_state` shuffles
  each library from the game's own seeded RNG (CR 103.3 / 903.6). Measured over 20 seeds, the
  first `SpellCast` moved from game turn **143-154** to a **3-29** band (min 3 / median 12 /
  max 29), and a 20-game run casts **670** spells. The `--max-turns` caveat above stays as the
  rule for reading pre-merge claims; it no longer describes the instrument.
  *(PB-DX22's fix cycle refinement: that 3-29 band counts command-zone commander casts, which
  library order does not gate. The band for the first cast the SHUFFLE is responsible for is
  **5-29, median 17** — `OOS-DX22-12`. Both are printed by `mtg-fuzzer`'s own summary now; raw
  run at `memory/primitives/pb-dx22-measurement-after-fixcycle.txt`.)*
* **Commander rules.** `fuzzer.rs:322-327` puts the commander card in `ZoneId::Command` but never
  calls `builder.player_commander` — the only production registrar is `setup.rs:399`, whose own
  comment at `:384` says *"`ZoneId::Command` only puts a card there; `player_commander` is what
  records it"* — so
  `commander_ids` is empty and CR 903.8 tax / 903.9a zone return / 903.10a commander damage have
  **never been fuzzed** (`OOS-SIM1-4`).
  **✅ FIXED by PB-DX22 (`scutemob-196`)**, which puts the placement and the registration in one
  function (`place_registered_deck`) and adds `register_commander_zone_replacements` (CR 903.9b).
  Same 20-game run: `CommanderCastFromCommandZone` **0 → 36** across 16 of 20 games, **13**
  CR 903.9a returns, non-empty `commander_damage_received` in **16 of 20**. CR 903.9b is
  registered but still unexercised at this sample size (`CommanderZoneRedirect` = 0,
  `OOS-DX22-9`) — which is now a statement about the games, not about the mechanism.
* **Reproduction.** See §1.1a — this is worse than the docs say.
* **Determinism.** `driver.rs:12-19`, current code doc: *"the fuzzer is not run-to-run deterministic
  for very long games, and that reproduces on pristine pre-refactor code."* PB-DX19 fixed the
  **stack overflow** (`layers.rs:24,144`, the re-entrancy guard); it did not touch determinism.

### 1.1a The crash artefact does not reproduce anything — verified, and it is the biggest single gap

The fuzzer's own module doc (`fuzzer.rs:29-37`) advises: *"capture the crash JSON, not just the
seed, for anything that must outlive the build."* **That advice is unsatisfiable as written.**

```rust
// crates/simulator/src/bin/fuzzer.rs:265-274
let report = CrashReport {
    seed: result.seed,
    player_count: cli.players as usize,
    violation: v.clone(),
    command_history: Vec::new(), // Would need to capture during game
    turn_number: v.turn_number,
    total_commands: result.total_commands,
};
```

Three separate failures, each independently fatal to a "crash JSON → seed → replay" pipeline:

1. **`command_history` is unconditionally empty.** The JSON carries no more reproduction power than
   the bare seed it wraps.
2. **A hard crash writes nothing at all.** The crash-report block (`fuzzer.rs:260-278`) runs only
   *after* `.collect()` on `:197` succeeds. A panic or SIGABRT in any parallel game prevents the
   whole block from executing. There is no `catch_unwind` anywhere in the fuzzer, the simulator or
   the engine — and per `crates/engine/tests/primitives/pb_dx19_characteristics_recursion.rs:24,169`
   the recursion abort is *"6/SIGABRT, not an unwindable panic — `catch_unwind` cannot contain it."*
3. **`--replay <SEED>` is seed replay, not command replay** (`fuzzer.rs:133-145`) — it re-derives the
   game from `StdRng::seed_from_u64(seed)`, so it reproduces only within the same build and only if
   the run was deterministic for that game length, which `driver.rs:12-19` says it is not.

`crash-reports/crash_<seed>.json` is written relative to the process CWD (`fuzzer.rs:261,276`).

### 1.2 `crates/simulator/src/invariants.rs` — nine checks that can fire

`check_all` (`:26-43`) dispatches to ten functions. `check_mana_non_negative` (`:100-103`) is a
**deliberate no-op** — `ManaPool`'s fields are `u32`. The nine that can fire:

| # | fn | asserts | cannot catch |
|---|---|---|---|
| 1 | `check_zone_integrity` `:46-81` | every object in exactly one zone | whether the move was rules-legal |
| 2 | `check_id_uniqueness` `:84-97` | no duplicate `ObjectId` | — |
| 4 | `check_stack_consistency` `:171-323` | Stack zone == cards claimed by non-copy card-owning stack objects, 1:1, same order | wrong targets, wrong cost, wrong controller |
| 5 | `check_player_consistency` `:326-349` | active/priority player is not lost or conceded | a live player being skipped |
| 6 | `check_turn_order` `:352-363` | active player present in `turn_order` | duplicates or wrong ordering |
| 7 | `check_object_zone_agreement` `:366-383` | `object.zone` matches containing zone | both wrong in agreement |
| 8 | `check_attachment_validity` `:386-401` | `attached_to` target exists | an attachment that is real but illegal |
| 9 | `check_game_progression` `:403-420` | `turn_number` never decreases | phase/step regression |
| 10 | `check_no_orphaned_tokens` `:422-440` | tokens only on Battlefield/Stack | a non-token that should have been cleaned up |

**None of the nine verifies rules correctness.** They are structural consistency checks on the
resulting `GameState`. The two checks that would have been semantic —
**#10 legal-action soundness (SR-38's own property)** and **#11 SBA idempotency** — are documented
in `docs/mtg-engine-simulator.md:249-255` and **exist in no function** (`OOS-SIM3-2`).

**Test coverage of the checker itself is one check deep.** `invariants.rs` gained a `#[cfg(test)]
mod tests` at `:461-799` in SIM-3 — ten probes, `t1`..`t10` — and **all ten target
`check_stack_consistency`** (`t10` only proves it is wired into `check_all`). Checks 1, 2, 5, 6, 7,
8, 9 and 10 have **no test at all**. That is the shape SIM-3's own finding was about: F6 was a
sensor that was itself the defect, and eight of the nine sensors are still unprobed.

**Call sites** (whole workspace): `local_game.rs:806` (`apply_sequence`) and `:832`
(`apply_command`), both gated on `self.check_invariants`, plus `invariants.rs:783` in its own test.
**`LocalGame` is the only production caller.** The replay harness does not run it; the golden-script
corpus does not run it; direct-`Command` engine tests do not run it.

**De-noising, verified**: `stack_card_of` (`:127-169`) is an exhaustive `match` with no wildcard over
all **27** `StackObjectKind` variants (`crates/card-types/src/state/stack.rs:573`; 2 owning arms +
25 in the `None` arm). SIM-3 measured 9,719 → 938 violations on `--games 5 --seed 1 --max-turns 200`.
**But** `OOS-SIM3-4` records that **929 of the remaining 938** are `no_orphaned_tokens` reports that
`OOS-M11-7` says are expected, and `--stop-on-error` halts on one — *"the fuzzer still cannot be
used as a clean smoke test for simulator changes"*. And `OOS-SIM3-3`: every "N violations" figure
this project has quoted is **checkpoint-weighted** (929 reports from 183 distinct tokens), so
A/B deltas can be entirely a difference in how long a condition persisted.

### 1.3 The SIM-5 rejection channel — bot path only

```rust
// crates/simulator/src/local_game.rs:165-178
pub struct RejectedCommand { pub player: PlayerId, pub turn: u32,
                             pub command: Command, pub error: String }
```

`MAX_RETAINED_REJECTIONS = 256` (`:245`); retention additionally gated on
`LocalGameLimits::record_journal` (`:867`); `rejection_count` is `saturating_add`ed unconditionally
(`:866`) so the cap is visible rather than silent. Accessors `rejections()` / `rejection_count()`
are public (`:306-314`).

**The single recording call site is `local_game.rs:564`, inside `advance()`'s bot arm.** The human
path, `submit()` (`:599-643`) → `apply_sequence` (`:801`), returns `LocalGameError::Rejected`
straight to the caller and **never calls `record_rejection`**. This is asserted as an invariant by
the play-server's own probe (`tools/play-server/src/main.rs:3202-3205`):

> *"only BOT seats reach the rejection recorder — a human submission returns its error to the client
> instead"*

**Consequence, and it matters for every proposal below**: a 422 produced by a human's browser click
— the exact symptom of F9, G4, `OOS-CARDS2-4` and `OOS-SIM6-3` — leaves **no trace anywhere**. It is
a one-shot HTTP response. The channel that made SIM-6's measurement possible is blind to the seat
the playtester occupies.

**Not reachable from the fuzzer either**: `GameResult` and `CrashReport`
(`crates/simulator/src/report.rs`) carry no rejection field, and `driver.rs:77` sets
`record_journal: false`.

### 1.4 Golden-script corpus + replay harness

**Counted, not quoted**: `find test-data/generated-scripts -name '*.json' | wc -l` → **271**;
`grep -rho '"review_status"...' | sort | uniq -c` → **208 approved / 63 retired**. The gate asserts
the **partition**, not the values — `crates/engine/tests/scripts/run_all_scripts.rs:240-272`
(`the_corpus_is_fully_accounted_for`), plus `no_script_is_awaiting_triage` (`:168-192`), which is
how "0 pending" is enforced without ever being printed.

`check_assertions` lives in the test binary (`crates/engine/tests/scripts/script_replay.rs:435-692`),
not in `replay_harness.rs`, and supports 19 assertion path shapes. **An unknown path is a hard
failure**, and the SR-9c comment at `:674-683` says why: *"an unrecognized path used to return `None`
— 'no mismatch' — so a script could assert anything it liked … 244 assertions across the corpus were
doing exactly that."*

SR-9b (`crates/engine/tests/scripts/harness_equivalence.rs`) cross-validates the JSON-script regime
against the direct-`Command` regime on a per-step `Fingerprint { public: [u8;32], private:
Vec<(PlayerId,[u8;32])> }` (`:212-250`) — public **and** per-player private hashes, deliberately a
superset of `public_state_hash` (`:100-109`). It covers 13 of ~79 dispatch arms, ratcheted at
`:50-69`.

SR-9a: **9 test targets**, verified by `find crates/engine/tests -mindepth 2 -name main.rs` →
`core`, `rules`, `casting`, `combat`, `scripts`, `primitives`, `mechanics_a_d`, `mechanics_e_l`,
`mechanics_m_z`; the gate is `crates/engine/tests/no_stray_test_binaries.rs:63-79` (*"There were 297
of them; linking dominated test-build wall time"*, `:4` — the gate file is itself the one permitted
top-level `tests/*.rs`, so it does not live in a group directory).

**Cannot catch — verified by grep, zero hits each:**
* `grep -rln "generated-scripts\|replay_harness\|GameScript" tools/play-server` → nothing.
* `grep -rn "StubProvider\|LegalAction" crates/engine/src/testing crates/engine/tests/scripts` → nothing.
* `grep -rln "svelte" crates/engine` → one **file**
  (`crates/engine/tests/core/ui2_additional_cost_roster.rs`, four hits at `:28`, `:93`, `:244`,
  `:261`), every one of them prose in a doc comment.

**The corpus cannot see the provider, the play-server or the browser. 15 of the 17 functional
playtest findings live in exactly those three places.**

### 1.5 `local_game_playthrough.rs` — the only place SR-38 is asserted at runtime today

A scripted-human policy drives seat 1 through a full game through `LocalGame` alone.
`SEEDS: [u64; 5] = [1, 7, 42, 1234, 9001]` (`:50`), `MAX_TURNS = 25` (`:55`), 64 MiB hand-built stack
(`:58`), `DeckSource::RandomPerSeat` over the full def pool through the real `validate_deck` (the
module doc says *"the actual 1,804-def pool"* at `:28`; the corpus is 1,803 files today — the
comment is one stale).
It asserts no `Rejected` *"from a policy that only ever submits an action the game just offered it"*
(`:13-15`) — that is SR-38 — plus zero violations with the `OOS-M11-7` transient-token class
separated out, plus that the game reaches `GameOver` or the cap.

**This is the whole of the runtime SR-38 surface: 5 seeds, 25 turns, one seat, one policy.**
`OOS-SIM3-2` says it plainly: *"The fuzzer runs millions of bot actions past `process_command` and
checks none of them."*

### 1.6 `GET /api/game/report` — and the two unrelated things called "report"

There are two. They have no code relationship.

* `crates/simulator/src/report.rs` — **53 lines: two plain structs, one IO method
  (`CrashReport::write_to_file`, `:23-28`), and zero derivation logic.**
  `CrashReport { seed, player_count, violation, command_history, turn_number, total_commands }` and
  `GameResult { seed, winner, turn_count, total_commands, violations, error }`. **It computes no
  metrics.** The wasted-tap and `ManaPoolsEmptied` counters that SIM-5's A/B table reports exist
  only in a **test-only** struct, `crates/simulator/tests/sim5_bot_cast_discipline.rs:45-48`
  (`wasted_tap_runs`, `wasted_taps`) — they are not shipped, not on any DTO, and not reachable from
  the fuzzer's output.
* `tools/play-server/src/view.rs::BugReportView` — the HTTP route's DTO (§1.9).

**`GET /api/game/report` is the one payload in the crate that is not seat-redacted**, deliberately
and with a written argument (`view.rs:748-764`):

> *"a redacted repro is not a repro… That is **safe only because of what M11-local is**: one human,
> three bots, one process, no networking… **When M10a puts a real opponent on the other end of a
> socket this endpoint must be re-scoped** — either redacted, or restricted to a single-player game,
> or authenticated."*

**Any automated consumer of this route inherits that obligation.** It is safe to build one now; it
is not safe to carry it into M10a unchanged, and the proposal in §2 R4 says so at the row.

### 1.7 Play-server tests — 78, and what each kind proves

`grep -nE '^\s*#\[test\]\s*$' tools/play-server/src/main.rs | wc -l` → **27**;
`grep -nE '^\s*#\[tokio::test(\(flavor = "multi_thread"\))?\]\s*$' … | wc -l` → **51**. Total **78**.
(A naive `grep -c` returns 28/55 because the needles also appear as string literals inside
`test_no_socket_symbol_appears_in_the_test_region`'s own detector array at `main.rs:7117-7120`.)

* **51 HTTP probes.** They bind **no port**: `build_router(state, &PathBuf::from("nonexistent_dist"))`
  driven through `tower::ServiceExt::oneshot` (`main.rs:180,287,299-342`), a hard rule stated at
  `tools/play-server/README.md:1268-1271`. **Can catch**: the whole wire, end to end, including a
  non-default answer distinguishing the human's choice from the engine's default.
  **Cannot catch**: anything above the wire. `main.rs:3796
  test_ui1_search_pick_is_answered_over_http` proved library search worked; G1 made it dead in the
  browser for weeks.
* **7 frontend gates** — `test_frontend_never_structured_clones_reactive_state` (`:7302`),
  `…picker_failures_reach_the_error_strip` (`:7454`), `…search_picker_looks_wider_than_it_picks`
  (`:7566`), `…card_elements_carry_no_native_title` (`:7796`),
  `test_concede_lives_in_the_header_behind_a_confirmation` (`:8003`),
  `…tap_for_mana_is_grouped_and_still_reachable` (`:8118`),
  `…land_stacking_key_is_not_just_the_name` (`:8173`). These are **text walks over `.svelte`/`.js`
  source**. Two of them also walk the `$viewer` shared library
  (`../replay-viewer/frontend/src/lib`, `main.rs:7304-7319`) — widened there by a `/review` finding,
  *"currently zero hits, so this arm is coverage, not a repair."*
  **Cannot catch**: that any component renders, that a template is read correctly, or that an answer
  is right. Each says so in its own doc comment.
* **3 Rust source gates** (`test_production_code_never_builds_an_omniscient_view` `:6937`,
  `test_no_socket_symbol_appears_in_the_test_region` `:7085`,
  `test_ui6_view_rs_reads_game_state_in_exactly_the_three_known_places` `:4993`), 1 meta-test, 16
  in-process unit tests.

**Redaction gates — the README table has FIVE channels, not three** (`README.md:1147-1157`): names;
reconstruction keys; free-form strings; look entitlements; whole-zone looks — served by at least six
distinct test functions, one of which (the count-pinned needle-set gate) covers two channels.
UI-6 re-pinned that gate's raw-read count 2 → 3 and made it a needle **set** with five zero-pins,
because *"the new read spells `.zone(`, not `.objects()`"* and the single-needle gate would have
stayed green (`README.md:1216-1226`). `OOS-UI6-5` records the residual: an accessor nobody
enumerated is still invisible.

### 1.8 The R7 frontend harness — the design, incorporated as-is

**There is no JavaScript test harness.** Both `package.json`s
(`tools/play-server/frontend/`, `tools/replay-viewer/frontend/`) have `scripts: {dev, build,
preview}` and `devDependencies: {@sveltejs/vite-plugin-svelte ^6.2.1, svelte ^5.45.2, vite ^7.3.1}`.
No `test` script. Zero hits for `vitest`, `jest`, `playwright`, `@testing-library`. `node_modules/`
is not present in this worktree.

The design below is **taken verbatim from the UI-4 handoff** (`memory/workstream-state.md:2265-2309`)
plus UI-5's proof-of-concept (`:1629-1645`). It is **not redesigned here.**

* **Tier 1 — component tests** (vitest + jsdom + `@testing-library/svelte`): 3 devDeps, an `npm test`
  script, a `vitest.config.js` reusing the existing `svelte.config.js`, one spec per picker (8)
  ≈ 400-600 lines.
  **The one rule that makes or breaks it: a fixture MUST wrap the template in `$state()` before
  passing it as a prop.** A spec handing a picker a plain object *would have passed green against
  the G1 bug*. Write that rule into the harness's own module doc.
* **Tier 2 — real-browser scenarios** (`playwright-core`, ~30 lines of setup): drive the game to the
  target decision **over HTTP**, then do only the last few clicks in the browser. *"Tier 1 without
  Tier 2 could have missed this exact bug"* — `DataCloneError` is real-browser structured-clone
  behaviour and a jsdom polyfill may not reproduce it.
* **The real cost is fixtures, and it is bigger than the harness.** UI-4 scanned **~2,400 seeds**
  through `POST /api/game` to reach scry/surveil/Squad, because `session.rs:168` starts from
  `DeckSource::RandomPerSeat` and there is no decklist channel. Two routes: (a) commit the known-good
  tuples; (b) let `POST /api/game` accept a fixed decklist. Recommend (a) now, (b) when someone
  touches `session.rs` — see §2 R6, which is (b).
* **Known-good tuples, handed over so nobody re-scans** — UI-4: seed **116** → Three Visits
  (`PickOne`, 33 candidates incl. Dryad Arbor); seed **28** → Preordain *and* Consider (scry 2 and
  surveil 1 in one game); seed **29** → Harrow (sacrifice-a-land); seed **1364** → Galadhrim Brigade
  (Squad, `max_count: 1`; **1 seed in the first 600**). SIM-6: seed **79** → Yahenni; **62** → Altar
  of Dementia; **219** → Rummaging Goblin (discard); **282** → Vampiric Rites; **63/70/73/106** →
  High Market / Spawning Pit / Scavenger Grounds / Viscera Seer. UI-6: seed **29** (seat 2 Aven
  Mindcensor, from a 300-seed sweep). **Path correction**: the chromium binary is at
  `~/.cache/ms-playwright/chromium-1228/chrome-linux64/chrome` (the UI-4 handoff's `chrome-linux`
  path no longer exists and cost a launch failure); `/usr/bin/chromium` also worked for UI-5.
* **UI-5 already built tier 1 and threw it away** (`OOS-UI5-4`): a throwaway Vite entry mounted
  `ZoneBattlefield` twice on one page against a 6-Forest fixture, **10/10 in ~15 minutes**. Recipe:
  a directory beside `tools/replay-viewer/frontend/src` with `index.html` + `main.js`
  (`mount(Harness, …)`) + `Harness.svelte` + a `vite.config.js` whose `root` is that directory,
  built with `npx vite build --config <dir>/vite.config.js` from the frontend package (so
  `node_modules` resolves), served by `python3 -m http.server`.
  **Do not start from scratch. Start here.**
* **CI note, flagged not fixed**: the workflow is a single Ubuntu **cargo** job. Tier 1 needs
  `npm ci && npm test` and a Node toolchain. Today's Rust source gates need neither — that is why
  they are Rust gates and not a JS lint.

### 1.9 The bug-report artefact

```rust
// tools/play-server/src/view.rs:780-816
pub struct BugReportView {
    pub seed: u64, pub config: ReportConfigView,
    pub protocol_version: u32, pub protocol_fingerprint: String,
    pub hash_schema_version: u8, pub state_hash: String,
    pub turn: u32, pub command_count: u32,
    pub violations: Vec<String>, pub journal: Vec<JournalEntryView>,
    pub rejections: Vec<RejectionView>, pub rejection_count: u32,
}
// :832-843
pub struct ReportConfigView { players, human_seat, bot, mulligan_count,
                              max_turns, max_commands, max_consecutive_passes }
```

**The free-text gap is CONFIRMED.** There is no `description`, `notes` or `comment` field, and the
frontend export path (`PlayApp.svelte:167-190`, button at `:575-580`) is a pure click → `Blob` →
`<a download>` with no text input; the file is named
`scutemob-report-seed${seed}-mull${mulligan_count}.json`. `violations: Vec<String>` is
**engine-produced** text, and `view.rs:735` says so explicitly. Nothing is written server-side.

The deviation is deliberate and reasoned (`docs/mtg-engine-runtime-integrity.md:182-189`):
*"a free-text box is UI with no consumer while there is no submission endpoint."* **That reasoning
is sound and is exactly what §3 changes**: the alpha loop gives it a consumer.

**No coded replay path exists.** `view.rs:766-778` documents the *procedure* (*"`seed` plus `config`
rebuild the exact table… Replaying `journal`'s commands in order from that state reaches
`state_hash`"*), and `README.md:1109-1114` repeats it, but `grep -rn "BugReportView"` outside
`tools/play-server/src` returns nothing. `tools/replay-viewer` takes a `GameScript`, a different
schema entirely (`tools/replay-viewer/src/main.rs:8-9,22,32-34`). **There is no `--replay-report
<file>` and no converter.**

### 1.10 Reconciliation with `docs/mtg-engine-runtime-integrity.md`

The doc's own `Status:` line (`:10`) is **"PROPOSAL — not yet scheduled"**, `last_updated:
2026-08-02`. It is registered in `.claude/docs.yaml` as an evergreen reference (`triggers: []`,
`frequency: session`). Layer by layer:

| Layer | Doc requirement | Reality |
|---|---|---|
| **1** — runtime invariant checker | `pub fn validate_invariants(state: &GameState) -> Result<(), Vec<InvariantViolation>>` **in the engine crate** (`:46,52`), integrated into the replay harness after every `process_command`, <1ms budget, unit tests against deliberately corrupt states | **NOT BUILT AS SPECIFIED.** `grep -rn "validate_invariants" crates/ tools/` → **zero hits.** What exists is `mtg_simulator::invariants::check_all` — **the wrong crate**, called from `LocalGame` alone, so the replay harness and the 271-script corpus never run it. Of its nine live checks, **one** has unit tests (§1.2). Roadmap M9.7's five deliverables are all unchecked (`docs/mtg-engine-roadmap.md:739-753`) |
| **2** — state recovery / auto-rollback | M10 deliverable; ring buffer, rewind, pause | **NOT BUILT**, doc says so (`:229-231`) |
| **3** — bug reporter | automatic capture on integrity error + manual capture; replayable JSON; privacy stripping | **PARTIAL.** Manual export shipped (§1.9). Automatic capture is Layer 2's and does not exist. **The privacy requirement is explicitly unmet** (`:196-201`) — *"`BugReportView` carries every seat's raw events, deliberately… This must be re-scoped at M10a."* |

**The gap the doc does not name, and the reason §2 R2 exists**: Layer 3's manual export is
play-server-only. The **fuzzer** — the channel that generates crashes without a human present — has
its own artefact (`CrashReport`), and that artefact reproduces nothing (§1.1a). The roadmap's M15
alpha deliverable *"Crash reporting: basic telemetry for unhandled errors"*
(`docs/mtg-engine-roadmap.md:1139`) currently has no mechanism behind it on the automated side.

### 1.11 CI

`.github/workflows/ci.yml` is the **only** file in `.github/`. One job (`check:`, `:28`),
`ubuntu-latest`, `timeout-minutes: 45` (`:31`), triggers `push`/`pull_request` on `main` plus
`workflow_dispatch` (`:3-8`), concurrency-cancelled. Steps in order: checkout → free disk space →
read `channel` out of `rust-toolchain.toml` via `grep -oP` (`:59-68`) → install that exact toolchain
(`:70-74`) → verify `rustc --version` matches the pin, `exit 1` on mismatch (`:80-88`) →
`Swatinem/rust-cache` **pinned by commit SHA `c19371144…` (v2.9.1)**, not the floating `@v2` tag
(`:98`, SR-32) → `cargo fmt --all -- --check` → `tools/check-defs-fmt.sh` → `cargo clippy
--all-targets -- -D warnings` → `cargo build --workspace` → `cargo test --all`.

`grep -niE "npm|playwright|node |browser test|benchmark|criterion" .github/workflows/ci.yml` →
**zero matches**. There is no `schedule:` trigger and no second workflow.

**What that means:** `cargo test --all` *does* run the 51 HTTP probes and the 7 frontend source
gates, because `tools/play-server` is a workspace member (`Cargo.toml:15`). It runs **no fuzz game**,
**no browser**, and **no Node**. The fuzzer is a `[[bin]]` inside `crates/simulator`, never invoked.
`tools/play-server/frontend/dist/` is neither committed nor present (root `.gitignore` has `dist/`),
so any browser step would need `npm ci && npm run build` first.

**No wall-clock measurement of the full test suite exists anywhere** — README, CLAUDE.md, `docs/`,
`memory/` were searched. The only timings on record are *link* times in
`docs/sr-9a-test-consolidation.md:101-123` (cold build 39.8s → 24.0s, warm rebuild 34.2s → 11.1s),
measured at 3,162 tests and now stale. **§2 R7 therefore prescribes measuring before budgeting, and
no row below quotes a runtime it did not measure.**

---

## 2. Ranked proposals

Successor-table shape, dispatch-ready. **Three rows already exist in the v3 queue and are
reconciled, not duplicated** — for those, the proposal is a *re-ranking argument*, and the coordinator
should dispatch the existing batch number.

| # | Task | Findings it would have caught | Track | Scope | Wire |
|---|---|---|---|---|---|
| 1 | **≡ PB-DX22 — ✅ SHIPPED** (`scutemob-196`, merge `95f53b78`; was v3 rank 4) — make the fuzzer a real instrument: shuffle the library, register the commander | **none directly, and that is the honest answer** — no F/G finding is catchable by a shuffled fuzzer *alone*. It is what makes row 3 able to catch **F3, F4, F5, G5** at all, and it is the only row that puts CR 903.8/903.9a/903.10a (**F7**'s subsystem) under any automated exercise | simulator | `bin/fuzzer.rs` + `deck.rs`; re-rolls every recorded seed **once** | none |
| 2 | **FUZZ-CRASH** *(new)* — make the crash artefact reproduce | PB-DX19's SIGABRT (found by a bespoke instrument, not by the fuzzer's own artefact) | simulator | ~120-200 lines: fill `command_history`, per-game abort boundary, `--replay-report` | none |
| 3 | **≡ PB-DX32** *(v3 rank 19, already queued — argue for promotion)* — invariant #10 (legal-action soundness = SR-38 at runtime) + dedupe the checkpoint weighting + classify the transient-token floor | **F4, F9, G5**; `OOS-SIM5-3` (25 blocker refusals), `OOS-SIM6-3` (62 refusals), `OOS-CARDS2-4` *(CLOSED by PB-DX20, `scutemob-198`, 2026-08-04)* | simulator | `invariants.rs` + `GameResult` fields + dedupe by `(check, description)` | none |
| 4 | **HTTP-FUZZ** *(new)* — a randomized walker over the 6 play-server routes driving the **human** seat | **G2**, F7, F9, **G4**, G6-as-coverage, `OOS-CARDS2-4` *(CLOSED by PB-DX20, `scutemob-198`, 2026-08-04)*, `OOS-G10-1`, `OOS-SIM6-3`'s human half | play-server (test/bin) | ~300-500 lines over `build_router` + `oneshot`, no port | none |
| 5 | **R7-HARNESS** *(≡ `OOS-UI5-4`, designed twice, built zero times)* — tier 1 vitest/jsdom + tier 2 playwright-core | **G1 — and nothing else catches G1** | frontend | tier 1 ~400-600 lines + 3 devDeps; tier 2 ~30 lines setup | none |
| 6 | **DECK-CHANNEL** *(new)* — steered decks: a salt on `random_deck` + a decklist channel on `POST /api/game` | PB-DX19's crash sooner; **and it deletes the ~2,400-seed fixture cost that is R7's largest line item** | simulator + play-server | ~150-250 lines; **re-rolls seeds — batch with row 1** | none (DTO only) |
| 7 | **CI-POLICY** *(new, policy not code)* — what runs in CI vs a scheduled job | **none — a policy row catches nothing itself.** It decides how often rows 3, 4 and 5 run, which is the difference between catching **F4/F9/G2/G4/G5** on the commit that introduces them and catching them at the next playtest | infra | see §2.7; first deliverable is a **measurement**, not a config change | none |
| 8 | **REPORT-LOOP** *(new)* — free-text on the bug report + a coded replay path | G7's class (a report a human writes prose into), and every future report | play-server + tooling | ~100-150 lines | none (DTO only) |

**Dispatching from this table.** The table supplies title, findings, track, scope and wire; each
row's `§2.N` supplies the argument and the constraints a brief must carry. What it deliberately does
**not** supply is per-row acceptance criteria, because this project's criteria are house-standard
and identical across rows — write them at dispatch, from this list:

1. every new gate **proven red by executing a revert**, not by inspection (the standing rule since
   PB-DX5; UI-5's G11 gate was wrong on its first run and its own non-vacuity arm caught it);
2. **PROTOCOL and HASH computed from the failing gate's own output, never predicted** — all eight
   rows below predict *no* wire change, so the expected result is "unmoved", and an unmoved
   prediction still has to be gate-executed;
3. `git diff main..HEAD --numstat -- crates/engine/` **empty** for rows 2, 4, 5, 6, 7, 8 (row 1 and
   row 3 are `crates/simulator`, also 0 engine lines);
4. any behavioural claim **measured A/B on named seeds**, both directions, with the instrument named
   (SIM-5's `sim5_bot_cast_discipline.rs` is the precedent and rows 1 and 3 should reuse it);
5. full-workspace `--workspace --no-fail-fast` **captured to a file, never tail-piped** (the
   2026-08-02 lesson: a tail pipe hid a compile failure and faked a green run);
6. a seed filed for everything found and not fixed, with its measurement.

**Ranking rationale, stated so it can be argued with**: 1 gates 3/6/7. 2 is cheap and is the only
row that makes an *unattended* run produce something a person can act on. 3 has the highest
finding-count per line and its sensor now exists. 4 is the only row that covers the seat the
playtester occupies. 5 is the only row that covers the layer G1 lived in. 6 is a force multiplier on
4, 5 and 1. 7 is free. 8 is small and closes the loop's last link.

### 2.1 Row 1 — ≡ PB-DX22. ✅ SHIPPED (`scutemob-196`, merge `95f53b78`)

> **Status: SHIPPED.** Dispatched as this doc recommended (the existing batch, not a new task).
> All three seeds are CLOSED — `OOS-SIM1-4` outright, `OOS-UI2-1` and `OOS-SIM3-1` as closures
> **with their numbers kept as thresholds**, because those numbers are what date every pre-merge
> fuzz claim. Dispositions and the full A/B are in `docs/audits/decision-point-audit.md` §8.1
> (rows `OOS-SIM1-4`, `OOS-UI2-1`, `OOS-SIM3-1`, plus new `OOS-DX22-1..11`).
>
> **The open measurement below is SETTLED, and it resolved to the second branch.** An
> instrumented 5-game run at HEAD *before* any edit found **zero**
> `CommanderCastFromCommandZone` in ~56,800 commands over games running to turn 167-201 — so
> SIM-3 did **not** measure a pre-SIM-1 build. SIM-1's loop is present and correct; it was
> *starved of its input*, because the offer is gated on `commander_ids` (CR 903.8) and the
> fuzzer never populated it. **`OOS-SIM1-4` and the missing commander cast were one defect,
> not two**, and no provider change was needed. Post-fix, the same command yields **36**
> commander casts across 16 of 20 games. The other half of the memo's question also resolved:
> `OOS-SIM3-1`'s turn-143 figure **reproduced exactly** (seed 2), so the two seeds were one
> fact read at two `--max-turns` values, exactly as argued.
>
> **What shipping it cost, measured rather than feared**: the fuzz seeds re-rolled once, and
> nothing else did — `cargo test -p play-server` 78/0 and `crates/simulator/tests/local_game.rs`
> 24/24 (23 pre-existing, all unchanged, plus this batch's own CR 903.9b probe).
> **What it found**: `attachment_validity` violations on fuzz seed 5
> turn 88 (`OOS-DX22-8`) — a pre-existing engine defect, 0 engine lines in the branch diff, and
> the first defect the repaired instrument surfaced.
>
> **Rows 3, 6 and 7 are unblocked.** Row 2 (FUZZ-CRASH) is unaffected and is now the cheapest
> remaining row; `OOS-DX22-7` feeds it directly.

`OOS-UI2-1` + `OOS-SIM3-1` + `OOS-SIM1-4`, ranked **4** in
`memory/primitives/seed-rerank-2026-08-02.md` §4, class *"EVIDENCE INTEGRITY — every historical
fuzz-parity claim depends on it"*.

**Everything else in this document is downstream of it.** A rejection-rate invariant (row 3) over a
game where no spell is ever cast measures nothing. A steered deck (row 6) that seeds a corner-case
card into a library that is drawn basics-first delivers that card around turn 143.

Two constraints carried from the queue memo verbatim: **PB-DX19 must precede PB-DX22** (shipped —
`451e3517`), and **PB-DX22 re-rolls every recorded seed once**, as does any card-def batch via
`OOS-CARDS2-3` — so batch the re-deal. **Both held.** DX19 preceded it; the re-roll happened once
and was confined to the fuzz path, so a card-def batch that re-rolls again pays its own cost and
not this one's.

The memo also left **one open measurement PB-DX22's plan had to settle first**: SIM-1 added a
command-zone cast loop (`legal_actions.rs:675-693`) and a commander is not in the library, so a
bot should have been able to cast its commander around game turn 12-24, a hundred turns before
SIM-3's measured 143. One instrumented `mtg-fuzzer --games 5 --seed 1` was to settle whether
SIM-3 measured a pre-SIM-1 build or something suppressed that offer. **It was run, and the answer
is "suppressed" — see the status block above.**

**Cannot catch even after it lands**: everything above `LocalGame`. Rows 4 and 5 exist for that.

### 2.2 Row 2 — FUZZ-CRASH: the artefact must reproduce

**New. Not in the queue.** §1.1a is the whole argument. Three concrete pieces:

1. **Fill `command_history`.** `LocalGame` already records the journal when
   `LocalGameLimits::record_journal` is on; `driver.rs:77` sets it `false`. Either flip it for the
   game that violated and re-run, or keep a bounded ring. *Weigh the memory cost against
   `--games 1000` before choosing* — the SIM-5 `/review` cycle already gated rejection retention on
   this flag for exactly that reason (`memory/workstream-state.md:2076-2078`).
2. **A per-game abort boundary.** A panic can be contained with `catch_unwind` around
   `run_single_game`; **a SIGABRT cannot** (`pb_dx19_characteristics_recursion.rs:24,169`). The
   honest design is therefore **write-before, delete-after**: emit `crash-reports/inflight_<seed>.json`
   containing the seed and config *before* the game starts and remove it on clean completion, so an
   abort leaves its own tombstone. That is a few lines and it is the only mechanism that survives
   `abort()`. State this in the brief so a worker does not spend the batch trying to catch the
   uncatchable.
3. **`--replay-report <file>`** — the missing consumer. Row 8 shares it.

**Cannot catch**: anything. It is not a detector; it is what turns a detection into a fix.

### 2.3 Row 3 — ≡ PB-DX32, and the case for promoting it from rank 19

The seed is `OOS-SIM3-2` and it says exactly what row 3 is:

> *"#10 **legal-action soundness** ('actions from the provider don't get rejected by
> `process_command()`') and #11 SBA idempotency exist in no function… #10 is the notable absence: it
> is **SR-38's property** — the one the play-server's 422s and this milestone's F4/F7/F9 findings are
> all instances of — and the only thing currently asserting it is `local_game_playthrough.rs`, on
> five seeds, on the human path. **The fuzzer runs millions of bot actions past `process_command`
> and checks none of them.** … #10 is close to free (`GameDriver` already distinguishes a rejected
> command from an applied one)."*

**Why it should move up**: PB-DX32 was ranked on 2026-08-02 **before SIM-5 shipped the rejection
channel that afternoon**. The expensive half of #10 — capturing the refusal and its reason — now
exists (`RejectedCommand`, §1.3) and is already exported over HTTP. What remains is putting it on
`GameResult` and asserting a threshold. The measurement that proves the value is already recorded:
SIM-5's channel classified **166 refusals** on its first run, and SIM-6 then closed **53** of them
and identified **62** more as one seed (`OOS-SIM6-3`). That is what an SR-38 invariant produces —
a ranked defect list, from a run nobody had to watch.

**Three components, and the second and third are not optional:**

* **(a) rejection-rate ≈ 0.** `GameResult` gains `rejection_count` and a bounded `rejections`
  sample; the fuzzer fails (or reports) above a threshold. **A threshold, not zero** — `OOS-SIM5-3`'s
  blocker refusals and `OOS-SIM5-5`'s modal slices are known-open, so a hard zero would be red on
  arrival. Pin the current number and ratchet it down, the `MAX_AUTO_CHOSEN_COMPLETE_UNION` pattern.
* **(b) waste thresholds.** SIM-5 already built the instrument — `wasted_tap_runs`, `wasted_taps`,
  `ManaPoolsEmptied` in `crates/simulator/tests/sim5_bot_cast_discipline.rs:45-48` — and it is
  **test-only**. Promoting it into `invariants.rs`/`GameResult` is mostly a move.
  **Verified non-duplicative**: none of the nine live checks inspects rejections, taps, or pool
  emptying (§1.2). The 1:1 `ManaPoolsEmptied`-to-wasted-run correspondence has been reproduced three
  times (triage 18/18 live; SIM-5 10/10, 15/15, 5/5), so it is a genuine oracle and not a proxy.
  **Honest limit**: SIM-5's seed-7 residual traced to greedy-solver slack (`OOS-SIM2-1`) on a cast
  that *succeeded*, so the threshold must be a threshold, not zero, and the brief must say so.
* **(c) the noise floor, or (a) and (b) are unusable.** `OOS-SIM3-4`: **929 of the 938** remaining
  violations are `no_orphaned_tokens` reports that `OOS-M11-7` says are expected, and
  `--stop-on-error` halts on one. `OOS-SIM3-3`: counts are checkpoint-weighted (929 reports, 183
  distinct tokens), so A/B deltas can be pure persistence differences.
  `local_game_playthrough.rs` already solved this (`transient_token_violations` — report the class,
  assert the strictly stronger end-state property). Give `check_no_orphaned_tokens` the same
  treatment and dedupe by `(check, description)`.

**Decision-point coverage — the third leg, and it must NOT be built from scratch.** §0.1: the gate
exists. Two things it cannot do, and only one of them is worth a proposal here:

* Its **self-named blind spot** (`decision_gate.rs:19-28`) is a choice **dropped at authoring time**
  — "you may X, if you do Y" authored as a bare `Effect::Sequence` with no gating `Effect` is
  invisible to a walk over variant names (its own cited example: Smuggler's Copter). That is
  **PB-DX8**, the oracle-text-vs-DSL cross-check, already at v3 rank 10 and already described as
  *"the worst blind spot, now measured"*. **Ride it there, do not re-file it.**
* What is genuinely new and belongs in row 3: a **runtime counterpart** — count the decision points a
  fuzz run actually *reaches* against the `ROWS` roster in `decision_site_walk.rs`. The static gate
  proves a row is *recorded*; nothing proves any row is ever *exercised*. A `Served` row that no
  fuzz game ever reaches is an untested feature wearing a green check. This is cheap (a counter keyed
  by row id) and it is the only decision-point measurement that is not already built.

**Cannot catch**: G3 and its 79 siblings. **A rejection-rate invariant is blind to them by
construction** — the engine picks a default, the command is accepted, nothing is refused. That is
precisely why G3 needed a human, and it is the sharpest boundary in this document.

### 2.4 Row 4 — HTTP-FUZZ: the browser path, automated

**New.** A randomized walker over the six routes (verified current, `main.rs:180-189`):

| Method | Path | Handler | Body |
|---|---|---|---|
| GET | `/api/game` | `api::get_game` | — |
| POST | `/api/game` | `api::post_game` | `NewGameRequest {players?, bot?, seed?}` (`deny_unknown_fields`, `view.rs:857-866`) |
| POST | `/api/game/action` | `api::post_action` | `ActionRequest {seq, action_index, params?}` |
| POST | `/api/game/mulligan` | `api::post_mulligan` | `MulliganRequest {take, cards_to_bottom?}` |
| GET | `/api/game/report` | `api::get_report` | — |
| GET | `/api/healthz` | `api::get_healthz` | — |

The walker seats itself as the human, reads `SeatView`, picks a random offered action, fills its
params from the **descriptors the server itself sends** (`AnswerShapeView`, `costs`,
`TargetRequirement` — the same data `ActionBar.svelte` reads), submits, and asserts:

* **every offered action is accepted** — SR-38 at the seat where SIM-6 measured the defect actually
  living. This is the same property as row 3(a), *on the other seat*, and the two are not
  substitutes: SIM-6's A/B found **zero** of 166 bot refusals were the sacrifice/discard channel
  (`memory/workstream-state.md:1751`), because bots never reached it. **G4 was invisible to bots by
  construction and visible on a human's first click.**
* **invariants across a mulligan** — the G2 property. All four command zones are public (CR 903.6)
  and appear in `SeatView`, so *"no seat's commander changes across a mulligan"* is assertable from
  the response body alone, with no engine access. SIM-4 shipped exactly this as two probes —
  `test_sim4_mulligan_preserves_every_seats_commander` (`main.rs:8583`) and
  `test_sim4_session_resolves_decks_once_and_a_mulligan_preserves_them` (`:8666`) — and the walker
  generalises them from two fixed flows to every reachable one.
* **no 5xx, no unexpected 422, no `seq` desync.**

**Why the 51 existing probes are not this**: each drives one hand-picked flow to one assertion.
Nothing walks the *whole* offer space, so an action kind nobody wrote a probe for is untested — and
that is the shape of F7 (commander casts), G6 (~70 defs with an unofferable alt cost) and
`OOS-CARDS2-4` (13 `Complete` Auras that 422 on first contact) — the last of these **CLOSED by
PB-DX20** (`scutemob-198`, 2026-08-04): the offer layer and the cast path now derive an Aura's
target requirement from the same function, and a real HTTP round trip (T6) proves at least one of
the 13 (Rancor) castable end to end.

**Cannot catch**: **G1** — it starts below the browser, which is the exact sentence the G1 triage
wrote about the probe that proved library search worked (*"an end-to-end HTTP probe that starts below
the browser proves the channel and says nothing about the only part a human touches"*). **G3** —
nothing is refused. **G7** — nothing is wrong at the wire; the data simply was not there.

**Sequencing**: `OOS-SIM6-3` (HIGH) caps its yield. Auto-tap covers `CastSpell` and nothing else
(`local_game.rs:738`), so a human activating a mana-cost ability 422s today — the walker would find a
large, already-known refusal class and little else until that closes. **Dispatch `OOS-SIM6-3` first,
or accept that the walker's first run mostly re-reports it.**

**Carry the §1.6 obligation into the brief**: if the walker consumes `GET /api/game/report`, it
inherits the non-redacted-endpoint deviation, which `view.rs:748-764` says must be re-scoped at M10a.

### 2.5 Row 5 — R7-HARNESS

Design in §1.8, taken as-is from UI-4 and UI-5. Nothing to add except the dispatch framing:

**It is the only channel that catches G1, and G1 was the most expensive finding either playtest
produced** — five CR flows the project believed shipped (library search 701.23, scry 701.22a,
surveil 701.25a, sacrifice 118.8, Squad 702.157a) had never worked in a browser, and the playtester
conceded a game because the only live control on screen was Concede.

**It has now been designed twice and built zero times.** UI-4 wrote the two-tier design; UI-5 built
tier 1 in ~15 minutes, proved it 10/10, and threw it away because its brief did not ask for it
(`OOS-UI5-4`: *"the right call for a batch that was not asked to build it and the wrong outcome to
repeat a third time"*). Every UI batch since UI-4 has paid for its absence in source-text gates that
cannot prove a component renders.

**Cannot catch**: anything below the component. It is not a substitute for the 51 HTTP probes; UI-4's
gates prove a *pattern* is absent, tier 1 proves a component *behaves*, tier 2 proves a *browser*
agrees, and all three are needed because jsdom may not reproduce real structured-clone semantics.

**CI**: tier 1 needs a Node toolchain and `npm ci && npm test` (§2.7). Tier 2 needs a browser and
belongs in the scheduled job.

### 2.6 Row 6 — DECK-CHANNEL: steered decks, and the fixture cost

Two halves. **The second is the one with the immediate payoff.**

**(a) Salt the fuzz deck.** `random_deck` (`crates/simulator/src/deck.rs:30-157`) picks a commander
uniformly from `Complete` legendary creatures (`:53`), then draws ≤60 non-lands and ≤5 non-basic
lands as **singletons** from a colour-identity-filtered pool, and pads to 99 with basics. Proposal: a
caller-supplied "must include" set, drawn from a roster query — layer-condition cards, cost
modifiers, DFCs, the `AltCastAbility` family — so a fuzz run exercises corner mechanics at a rate
the singleton draw cannot deliver.

> **Correction to the dispatch brief, recorded rather than repeated.** The brief cites *"the
> Archangel lesson: random singleton draws made the crash card ~9%-per-deck rare."* **That figure is
> unverified and nothing in the repo pins it.** From the actual algorithm the per-deck inclusion
> probability for a specific eligible non-land is `min(1, 60/N)` where `N` is the
> colour-identity-filtered `Complete` non-land pool — and `indomitable_archangel` is eligible only
> when the randomly chosen commander's identity ⊇ {White}. With the broadest pool (`N ≈ 841`, the
> whole `Complete` non-land corpus) that is **≈ 7.1%**; a mono- or two-colour White commander gives
> `N` in the low hundreds and **20-40%**. A single blended figure needs the commander
> colour-breadth distribution, which requires running `compute_color_identity` over the corpus.
> **Cite it as "≈7-40% depending on commander colour breadth, exact figure unmeasured" and note
> that `OOS-CARDS2-3` already records that no test pins the pool size.** The *lesson* — a rare draw
> hid a HIGH for 4.5 months — is unaffected; only the number is.

**(b) A decklist channel on `POST /api/game`.** `NewGameRequest` is `{players?, bot?, seed?}` with
`deny_unknown_fields` (`view.rs:857-866`), so a decklist in the body is a hard 400 today, and
`config_for` (`session.rs:156-174`) ignores the body for deck contents entirely.
`DeckSource::Fixed` already exists (`setup.rs:52-59`) and is **already used internally**: SIM-4's
`session::new_game` builds from the `RandomPerSeat` recipe, reads back the dealt decks with
`setup::dealt_decks` and pins `cfg.decks = Fixed(dealt)` (`session.rs:237-243`) — so the plumbing
is done and only the public channel is missing.

**Why (b) is the payoff**: it deletes the largest line item in row 5 and in every future browser
verification. UI-4 scanned **~2,400 seeds** to reach three decisions; SIM-6 scanned **0..400**; UI-6
swept **300**. A scenario that names its cards costs zero seeds. The R7 handoff already recommends
exactly this as *"the real fix"* (`memory/workstream-state.md:2290-2292`).

**Constraint**: (a) re-rolls every recorded seed, so **batch it with row 1** — the queue memo's own
instruction (`seed-rerank-2026-08-02.md` §4 banner, constraint 2). (b) re-rolls nothing: it is an
additive optional field on a request DTO.

### 2.7 Row 7 — CI-POLICY

**Measure first.** No wall-clock figure for `cargo test --all` exists anywhere (§1.11). Every budget
below is a shape, not a number, and the first task of this row is to produce the number.

| What | Where | Why |
|---|---|---|
| fmt, defs-fmt, clippy, build, `cargo test --all` | **CI, unchanged** | it already runs the 51 HTTP probes, the 7 frontend source gates, the golden corpus, `decision_gate`, SR-37 and `local_game_playthrough` |
| row 4's HTTP walker, bounded (fixed seed list, low turn cap) | **CI** | it is an ordinary `#[tokio::test]` over `oneshot` with no port and no browser — the same class as the 51 probes. Budget it against the measured suite time and cap it |
| row 5 tier 1 (vitest/jsdom) | **CI, behind a new Node step** | `npm ci && npm test`; the job has no Node today. Non-negotiable if the harness is to mean anything |
| row 5 tier 2 (playwright) | **scheduled job** | needs a browser **and** `npm run build` — `dist/` is gitignored and absent |
| the fuzzer, long runs | **scheduled job, and NOT before row 3(c)** | `OOS-SIM3-4`: 929 of 938 violations are known-transient and `--stop-on-error` halts on one. *"The fuzzer still cannot be used as a clean smoke test."* Putting it in CI today buys a red build on a known non-defect |
| the fuzzer, short smoke (`--games N --max-turns M`, `--profile fuzz`) | **CI, after rows 1 + 3(c)** | `--profile fuzz` is what trips the SR-4/SR-14 tripwires; a short run is the cheapest possible engine-abort detector. Requires row 1, or it smoke-tests a land-only game |

**The 45-minute timeout is the binding constraint and it is shared.** A scheduled job has its own
budget and is the right home for anything unbounded. Recommendation: add **one** `schedule:` workflow
(nightly), not a matrix — the repo has exactly one workflow today and the SR-32 SHA-pinning
convention should be followed for any new action.

### 2.8 Row 8 — REPORT-LOOP

Two small things that turn an artefact into a loop:

* **A free-text field.** `BugReportView` gains `description: Option<String>`; `PlayApp.svelte`'s
  export flow gains a text box. The runtime-integrity doc's stated reason for its absence — *"a
  free-text box is UI with no consumer while there is no submission endpoint"*
  (`docs/mtg-engine-runtime-integrity.md:184-185`) — is **correct today and is what §3 changes**:
  the consumer is the triage step, and the two triages this doc is built on both had to work from a
  separate `.md` file the tester wrote by hand. **The value is precise**: the tester's prose is what
  distinguishes *"I clicked confirm and nothing happened"* (G1, the most important finding either
  playtest produced, and **not filed as a bug** — it was a sub-bullet) from a seed and a hash.
* **A coded replay path.** `--replay-report <file>`, shared with row 2. The procedure is documented
  (`view.rs:766-778`) and has no implementation; a converter to the `GameScript` schema the replay
  viewer already reads (`tools/replay-viewer/src/main.rs:8-9`) would also make a bug report loadable
  in the stepper, which is what `docs/mtg-engine-simulator.md:270` promises for crash reports
  (*"Serialized as JSON — loadable in the replay viewer for debugging"*) and which is not true today.

**Also fix while there** (doc-only, found during verification): `tools/play-server/README.md:1098-1101`
lists the `GET /api/game/report` fields and **omits `rejections` and `rejection_count`**, which SIM-5
added and which are tested at `main.rs:3137-3215`. Code and tests are correct; the README sentence is
stale.

---

## 3. The alpha loop

The point of §2 is not more automation. It is to make a human's playtest hour buy something no
machine can buy. Below, every defect class is owned by exactly one stage, and the ownership is
argued from what the stage can and cannot see.

```
                          ┌─────────────────────────────────────────────┐
   every commit  ───────► │ CI: fmt · defs-fmt · clippy · build          │
                          │     cargo test --all                        │
                          │     = 271 golden scripts + decision_gate     │
                          │       + SR-37 + 51 HTTP probes + 7 source    │
                          │       gates + playthrough(5 seeds×25 turns)  │
                          │     + [R4 bounded walker] + [R5 tier 1]      │
                          └──────────────┬──────────────────────────────┘
                                         │ green
                          ┌──────────────▼──────────────────────────────┐
      nightly    ───────► │ scheduled: mtg-fuzzer --profile fuzz         │
                          │            long run, steered decks           │
                          │            + R5 tier 2 (real browser)        │
                          └──────────────┬──────────────────────────────┘
                                         │ violation / abort
                          ┌──────────────▼──────────────────────────────┐
                          │ crash artefact  ── R2 ──►  seed + config     │
                          │                            + command journal │
                          │                            + inflight tomb   │
                          └──────────────┬──────────────────────────────┘
                                         │ --replay-report
                          ┌──────────────▼──────────────────────────────┐
                          │ deterministic repro → engine test / seed row │
                          └─────────────────────────────────────────────┘

                          ┌─────────────────────────────────────────────┐
   human playtest ──────► │ notes (prose) + bug-report JSON with R8      │
                          │ free-text, exported from the live game       │
                          └──────────────┬──────────────────────────────┘
                                         │
                          ┌──────────────▼──────────────────────────────┐
                          │ TRIAGE: chain-verify every claim to file:line│
                          │  → F/G record + ranked successor table       │
                          └──────────────┬──────────────────────────────┘
                                         │
                          ┌──────────────▼──────────────────────────────┐
                          │ dispatch, one concern per batch, own branch  │
                          └─────────────────────────────────────────────┘
```

### 3.1 Ownership by defect class

| Defect class | Owner | Why that stage and not another |
|---|---|---|
| Illegal `GameState` (zone/id/stack/attachment/turn) | **fuzzer invariants** (§1.2) | the only channel that runs `check_all` on unattended games |
| Engine abort / unbounded recursion / arithmetic wrap | **fuzzer under `--profile fuzz`** | plain `--release` compiles the SR-4/SR-14 tripwires out (`Cargo.toml:32-50`); this is how PB-DX19 was found |
| **SR-38 on the bot path** (offered-then-refused) | **fuzz invariant #10** (row 3) | millions of bot actions; nothing checks them today |
| **SR-38 on the human path** | **HTTP walker** (row 4) | SIM-6 measured **0 of 166** bot refusals in the channel a human hits on first click. The bot seat is not a proxy for the human seat |
| Bot resource waste (taps, pools) | **fuzz waste thresholds** (row 3b) | 1:1 `ManaPoolsEmptied` oracle, reproduced 3× |
| Wire/protocol regressions | **PROTOCOL/HASH gates + golden corpus** | already machine-enforced; gate-executed every batch |
| Printed-field errors (cost, P/T, type line) | **SR-37** | already closed as a class — F1/F2 cannot recur |
| Oracle-text → DSL semantic drift (dropped "may"/"choose") | **PB-DX8** (v3 rank 10) | the `decision_gate` blind spot; a variant-name walk cannot see a clause that was never authored |
| Component behaviour (a picker that does not fire) | **R7 tier 1** (row 5) | source-text gates prove a pattern absent, not a component alive |
| Real-browser semantics (structured clone, `title` chrome, z-index) | **R7 tier 2** (row 5) | jsdom may not reproduce them; G1 is the proof |
| Hidden-information leaks | **the 5-channel redaction gate set** (§1.7) | with `OOS-UI6-5`'s caveat: a new channel is invisible until enumerated |
| **Agency — "I was never asked"** | **HUMAN** | 80 `Complete` defs still auto-choose (§0.1). No rejection, no violation, no failed assertion. Only a person notices they were not asked |
| **Legibility — "I can't tell what happened"** | **HUMAN** | G7: the event log carried no targets. Every layer was internally correct; the information simply was not there |
| **Feel, pacing, layout, clutter** | **HUMAN** | 6 of playtest 2's 13 findings (G8-G13) |
| Causal *hypotheses* in playtest notes | **TRIAGE, not the tester** | both of the tester's hypotheses in playtest 2 were wrong, and both blamed an innocent mechanism. *"A playtester reports symptoms accurately and mechanisms speculatively"* |

### 3.2 What this buys, stated as a target

Playtest 2 produced 13 findings: 7 functional, 6 UX. **After rows 1-6, the functional half is the
part that should shrink** — G2 (row 4), G4 (row 4), G5 (row 3), G6-as-coverage (row 4) and G1
(row 5) are each owned by a stage upstream of the human. **G3 and G7 are not**, and no proposal in
§2 claims them: an unasked decision produces no error, and missing information in a log is not an
inconsistency. Those two are the shape of what human playtests should be spending their time on,
and there are **80** more of the G3 kind on the books right now.

### 3.3 Sequencing constraints, stated for the coordinator

1. **The PB-DX queue is not displaced.** Rows 1 and 3 **are** queue entries (PB-DX22 rank 4,
   PB-DX32 rank 19). Rows 2, 4, 5, 6, 8 are new and none of them touches `crates/engine`.
2. **Row 1 gates rows 3, 6 and the CI fuzz smoke.** Without the shuffle, all three measure a
   land-only game.
3. **`OOS-SIM6-3` (HIGH) gates row 4's yield** and half of row 3's: auto-tap covers `CastSpell` and
   nothing else (`local_game.rs:738`), so **62 of the 113** remaining bot refusals and every human
   mana-cost activation are one seed. Close it first or expect the walker's first run to re-report it.
4. **Row 3(c) gates any CI fuzz step.** `OOS-SIM3-4`, verbatim: *"the fuzzer still cannot be used as
   a clean smoke test for simulator changes."*
5. **Rows 1 and 6(a) both re-roll every recorded seed. Batch them**, and land `OOS-CARDS2-3`'s
   pool-size gate first so the re-deal announces itself (queue memo §4 banner, constraint 2).
6. **Rows 4 and 5 are frontend/play-server and run parallel to the engine track.** Row 3 and row 1
   both touch `crates/simulator` and **must not run in parallel with each other** — the 2026-08-02
   collect's lesson (parallel workers sharing a crate produce semantic conflicts that survive a clean
   textual merge).
7. **Row 4 must not consume `GET /api/game/report` without carrying its M10a re-scope obligation**
   (§1.6).

---

## 4. Corrections — from / to

The `scutemob-186` pattern: every claim this task found stale, with what corrected it. **Nothing in
this table was fixed at source** — this branch may touch one file. Each is a candidate seed or a
one-line doc repair for whoever next edits the owning file.

| # | From (as stated) | To (verified) | Evidence |
|---|---|---|---|
| C1 | *"the R7 design in the UI-4 handoff"* (dispatch brief) | UI-4 designed it; **UI-5 built tier 1, proved it 10/10 in ~15 min, and threw it away**. The recipe is recorded and is where a worker should start | `memory/workstream-state.md:1629-1645`, `OOS-UI5-4` |
| C2 | *"GET /api/game/report journal metrics"* (dispatch brief) | The route ships a **journal**, not metrics. `crates/simulator/src/report.rs` is 53 lines of two plain structs and **computes nothing**. Wasted-tap / `ManaPoolsEmptied` counters are **test-only**, in `sim5_bot_cast_discipline.rs:45-48` | read in full |
| C3 | *"the Archangel lesson: ~9%-per-deck rare"* (dispatch brief) | **Unverified; no test pins it.** From the algorithm: `min(1, 60/N)`, ≈**7.1%** at the broadest pool, **20-40%** for a narrow White commander. Cite a range, not 9% | `deck.rs:30-157`; `OOS-CARDS2-3` |
| C4 | *"three gates for three channels, tabled in the play-server README"* (CLAUDE.md, M11 close) | The README table has **five channels** served by **six+** gates, one of which covers two channels. What is true and narrower: **UI-6 re-pinned one gate's raw-read count 2 → 3** and made it a needle **set** with five zero-pins | `tools/play-server/README.md:1147-1157`, `main.rs:4993` |
| C5 | *"rust-cache@v2"* (CLAUDE.md CI bullet) | SHA-pinned `Swatinem/rust-cache@c19371144…` = **v2.9.1**, per SR-32's anti-drift policy — the same treatment as the toolchain action | `.github/workflows/ci.yml:93-98` |
| C6 | Runtime-integrity Layer 1 = *"extract the existing 83 proptest invariants into `validate_invariants` in the engine crate"* | **Zero hits for `validate_invariants` in the workspace.** What exists is `mtg_simulator::invariants::check_all` — **wrong crate**, called only from `LocalGame`, so the replay harness and the 271-script corpus never run it. M9.7's five deliverables are all unchecked | `grep -rn "validate_invariants" crates/ tools/`; `docs/mtg-engine-roadmap.md:739-753` |
| C7 | *"capture the crash JSON, not just the seed, for anything that must outlive the build"* (fuzzer module doc `:29-37`) | **Unsatisfiable as written.** `command_history: Vec::new(), // Would need to capture during game`, and a hard abort writes no artefact at all | `crates/simulator/src/bin/fuzzer.rs:265-278`, `:197`, `:260` |
| C8 | *"the fuzzer never shuffles" may have been fixed by SIM-4/5/6* (open question at dispatch) | **CONFIRMED still true at HEAD.** None of the three touched `bin/fuzzer.rs` or `deck.rs`; `git log --oneline` on those paths shows only `34079131`, `b6274fcb`, `96a3bd23`, `878d593a` | verified by `git log` + `git show --stat` |
| C9 | *"PB-DX19 fixed the fuzzer's stack overflow and its determinism"* (a natural but wrong reading of the DX19 close-out) | PB-DX19 fixed **only** the stack overflow (`layers.rs:24,144`, re-entrancy guard). `driver.rs:12-19` still says, in current code, *"the fuzzer is not run-to-run deterministic for very long games"* | `driver.rs:12-19` |
| C10 | *"the rejection channel records refusals"* (SIM-5 handoff, read without its scope) | **Bot path only.** The single call site is `local_game.rs:564` in `advance()`; `submit()` returns the error to the client and records nothing. A browser 422 leaves no trace anywhere. Asserted as an invariant by the project's own probe | `main.rs:3202-3205` |
| C11 | *"the fuzzer's invariants are tested"* (implied by SIM-3's ten probes) | All **ten** target `check_stack_consistency`. Checks 1, 2, 5, 6, 7, 8, 9 and 10 have **no test** | `invariants.rs:461-799` |
| C12 | *"`docs/mtg-engine-simulator.md` lists twelve invariant checks"* | **Nine can fire**; #3 is an explicit no-op and #10/#11 exist in no function. **Already corrected in-doc by SIM-3** — recorded here because the corrected list is load-bearing for §2 row 3 | `docs/mtg-engine-simulator.md:220-255` |
| C13 | `tools/play-server/README.md:1098-1101` lists the report's fields | **Omits `rejections` and `rejection_count`**, both real, both tested. Doc-only; code and tests are correct | `view.rs:813,815`; `main.rs:3137-3215` |
| C14 | The G3 census line numbers in `memory/playtest-triage-2026-08-02b.md:268-279` (`:4222`, `:3157`, `:3274`, …) | **All stale** — the file moved and ENG-1 removed one site. Re-derived by symbol: **19 body hits in 12 distinct effects**, including four the census missed (`Discover`, `CreateTokenCopy`'s CR 508.4 attack-target inheritance, `CopySpellOnStack`, `ChangeTargets`), and one probable false positive (`:4916`, a dice-range dispatch, not a player choice). Also: several carry `deferred to M10` without the `+` | `grep -n "deterministic fallback\|M10+\|first matching\|deferred to M" crates/engine/src/effects/mod.rs` |
| C15 | `effects/mod.rs:15`'s policy doc: *"Effects requiring player choice (SearchLibrary, modal Choose) use a deterministic fallback in M7… Interactive choice requires the Command loop added in M9"* | **Stale in scope**: it names two effects; the pattern is now in a dozen-plus. And the Command loop landed and has served `SearchLibrary`, `Scry`, `Surveil`, trigger targets and (ENG-1) `DiscardCards` | `decision_site_walk.rs` `ROWS`; `effects/mod.rs:1275,1322,1326` |
| C16 | *"the fuzzer accepts `--players` 2-6"* (`fuzzer.rs:8` module doc) | Nothing validates it. `#[arg(long, default_value = "4")] players: u32` has no range check | `fuzzer.rs:58-100` |
| C17 | *"`engine.rs:3485-3500` deals no opening hand"* (`seed-rerank-2026-08-02.md` §2.4) | **The claim is true; the cite has rotted** onto an unrelated `IncompleteCardsInGame` check. The verifiable form: `fuzzer.rs:309-342` populates only Command and Library, and `first_turn_of_game` only sets a flag | `builder.rs:205-208`; `local_game_playthrough.rs:344` |
| C18 | `local_game_playthrough.rs:28` — *"the actual 1,804-def pool"* | 1,803 def files today (`find crates/card-defs/src/defs -name '*.rs'` minus `mod.rs`; `docs/authoring-status.md:22` agrees). One stale, harmless, but it is a number a future reader would quote | counted |

**Two claims from the dispatch brief were checked and are CONFIRMED exactly**, recorded so nobody
re-checks them: the fuzzer's 90.3% false-positive rate before SIM-3 (9,719 → 938, and
`stack_card_of` is genuinely an exhaustive match over all 27 `StackObjectKind` variants), and the
bug-report artefact's missing free-text field.

---

## 5. Seeds this task would file (none filed here — this branch may touch one file)

`OOS-FB1-1` — the fuzzer's `CrashReport.command_history` is unconditionally empty and a hard abort
writes no artefact at all, so the module doc's own reproduction advice cannot be followed (§1.1a;
row 2). ·
`OOS-FB1-2` — eight of the nine live invariant checks have no test (§1.2; the F6 shape). ·
`OOS-FB1-3` — a human's rejected command is recorded nowhere; the SR-38 sensor is blind to the seat
the playtester occupies (§1.3; row 4). ·
`OOS-FB1-4` — `validate_invariants` does not exist and `check_all` is not reachable from the replay
harness or the 271-script corpus, so M9.7's pre-alpha Layer 1 is unbuilt in the crate it was
specified for (§1.10). ·
`OOS-FB1-5` — nothing measures whether a `Served` decision row in `decision_site_walk.rs` is ever
*exercised* at runtime; a served-but-unreached row is an untested feature wearing a green check
(§2.3). ·
`OOS-FB1-6` — no wall-clock measurement of the full test suite exists anywhere, so every CI budgeting
decision is currently unmeasured (§1.11; row 7). ·
`OOS-FB1-7` — `tools/play-server/README.md:1098-1101` omits `rejections`/`rejection_count` from the
report's field list (C13). ·
`OOS-FB1-8` — `fuzzer.rs`'s `--players` has no range validation despite its doc claiming 2-6 (C16). ·
`OOS-FB1-9` — the G3 census in `playtest-triage-2026-08-02b.md` is stale in its line numbers and
short by four sites (C14).

---

## 6. Method and limits

**Method.** Five parallel read-only code-verification passes, each required to cite `file:line` it
had actually read rather than repeat the dispatch brief's numbers — which caught **18** stale claims
(§4), **three of them in the dispatch brief itself** (C1, C2, C3) and two in CLAUDE.md (C4, C5). Counts in §0 and §1 come from commands quoted at the
point of use, not from documents. The evidence base read in full:
`memory/playtest-triage-2026-08-02.md`, `memory/playtest-triage-2026-08-02b.md`, the UI-4/UI-5/UI-6/
SIM-4/SIM-5/SIM-6/ENG-1/ENG-2 handoffs in `memory/workstream-state.md`,
`memory/primitives/seed-rerank-2026-08-02.md` §2.4/§4, `docs/mtg-engine-runtime-integrity.md`,
`docs/mtg-engine-simulator.md`, `docs/mtg-engine-roadmap.md` M9.7/M15, `.github/workflows/ci.yml`,
and both raw playtest notes files.

**What was not done.** Nothing was built, nothing was run: `cargo` was not invoked (this worktree has
no `target/`), no fuzz run was executed, no browser was opened, and no file outside this one was
modified. Consequently:

* **Every runtime figure in this document is quoted from a prior batch's recorded measurement**, not
  re-measured here. The three that matter most — SIM-3's 9,719 → 938, SIM-5's 166 refusals,
  SIM-6's 166 → 113 — are attributed at the point of use.
* **No timing budget is proposed for CI** because no timing exists to budget against (§1.11, row 7,
  `OOS-FB1-6`). A row that named a number would be inventing one.
* **The "~7-40%" range in §2.6 is derived from the algorithm, not measured.** The exact figure needs
  `compute_color_identity` run over the corpus, which needs execution.
* **Scope estimates in the §2 table are shapes, not measurements.** They are calibrated against
  comparable shipped batches (UI-4's 3 lines + 2 gates; SIM-6's ~150; UI-5's ~400-500) and should be
  re-derived at dispatch, per the queue memo's standing instruction that a brief written now for a
  batch dispatched later is a stale premise waiting to happen.

### The `/review` cycle — 10 findings, all 10 taken, and three were wrong cites in this file

The reviewer re-checked ~30 of this document's `file:line` cites against the code and re-ran four of
its counting commands. **Three cites did not resolve, and the worst of them was inside a correction
row**, which is the shape this document is about:

1. **C8 quoted a `git log` output the command does not produce.** It claimed four commits on
   `bin/fuzzer.rs` + `deck.rs`; the command returns **seven**, and one of the three omitted
   (`e658c9d8`, PB-DX4) added **61 lines to `deck.rs`**. The conclusion — that SIM-4/5/6 never
   touched either file — survives and was independently re-confirmed in code, but *"verified by
   `git log`"* attached to a `git log` output nobody could reproduce is precisely the folklore this
   task exists to remove. Now lists all seven with the last-change date.
2. **`setup.rs:276`** was cited as the commander registrar; it is **`:399`** (`:276` is inside
   `dealt_decks`). The claim "the only production registrar" is correct — re-verified, the only
   other non-test hits are the builder definition and one test.
3. **The two SIM-4 mulligan probes** were cited at `main.rs:7149`/`:7232`, which are an assert
   message and a doc comment. They are at **`:8583`** and **`:8666`**, and are now named as well as
   numbered — a name survives a file edit and a line number does not, which is the OOS-DP6-8 lesson
   this document's own header invokes.

Also taken: §0's layer table dropped **G4** and summed to 16 against its own stated 17 (fixed, with
the straddle rule written down); the notes file is 55 lines, not 56, in a section whose stated method
is "counted, not quoted"; `report.rs` has one IO method as well as its two structs; the SR-9a "297"
quote is at `:4` and the gate file is not where SR-9a's own rule sends a reader looking; the
`svelte` grep is one *file* with four hits; §0.1 rendered an altered-case string as a verbatim
blockquote (restored); and rows 1 and 7 of §2 carried no F/G column (now stated as *"none, and that
is the honest answer"* with what each row unblocks), plus the dispatch note above on acceptance
criteria.

**Nothing the reviewer found changed a conclusion.** All ten were evidence defects — a citation that
did not resolve, a count that did not close, a quote that was not verbatim. That is the right
outcome for a document whose entire claim is that its predecessors' evidence defects are why a human
had to find these bugs.

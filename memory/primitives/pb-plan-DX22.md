# Primitive Batch Plan: PB-DX22 — Make the Fuzzer a Real Instrument

**Generated**: 2026-08-03
**Task**: `scutemob-196` · **Branch**: `feat/pb-dx22-make-the-fuzzer-a-real-instrument-oos-ui2-1-oos-sim3`
**Primitive**: not a DSL primitive — an **evidence-integrity** change to the simulator's fuzz
build path. Two behavioural edits (seeded library shuffle; commander registration) plus the
extraction that makes them testable, plus the probes that gate them.
**CR Rules**: 103.3, 103.5, 903.6, 903.8, 903.9a, 903.9b, 903.10a
**Seeds**: `OOS-UI2-1`, `OOS-SIM3-1`, `OOS-SIM1-4`
**Dependencies**: **PB-DX19 (SHIPPED, `451e3517`)** — mandatory predecessor; without it the
shuffle turns `indomitable_archangel`'s unbounded `calculate_characteristics` recursion from a
rare turn-191 abort into a routine one.
**Deferred items from prior PBs carried here**: none claimed. Two *adjacent* items are
explicitly declined below with reasons (opening hand — §B2; `max_commands` re-tuning — §B2 point 4
and `OOS-DX22-2`).
**Expected footprint**: `crates/simulator` + docs/memory only. **0 engine lines, 0 card-def
lines, 0 `tools/` lines, 0 wire change.** PROTOCOL **35** / HASH **72**, gate-EXECUTED and
unmoved.

---

## 0. What is settled before this plan starts (do NOT re-litigate)

`memory/primitive-wip.md` records the brief's mandatory pre-plan measurement, run at HEAD
(`9aa4f220`) against the fuzzer's own unmodified build path: 5 games / base seed 1 /
`--max-turns 200` / 4 players / `RandomBot`.

| seed | end | commands | `commander_ids` | first `SpellCast` turn | casts | first commander cast | cmdr returns / redirects |
|---|---|---|---|---|---|---|---|
| 1 | GameOver t176 | 9,802 | **0/4** | **154** | 25 | **none** | 0 / 0 |
| 2 | GameOver t192 | 12,631 | **0/4** | **143** | 21 | **none** | 0 / 0 |
| 3 | Halted MaxTurns 201 | 12,362 | **0/4** | **151** | 35 | **none** | 0 / 0 |
| 4 | GameOver t167 | 9,397 | **0/4** | **153** | 7 | **none** | 0 / 0 |
| 5 | Halted MaxTurns 201 | 12,601 | **0/4** | **151** | 33 | **none** | 0 / 0 |

**The answer**: the commander offer is *suppressed*, not late. `OOS-SIM1-4` is the cause — the
SIM-1 command-zone loop (`crates/simulator/src/legal_actions.rs:724-768`) filters on
`commander_ids` (CR 903.8; CR 408.1 is why the zone cannot be the filter), and
`crates/simulator/src/bin/fuzzer.rs:322-327` never calls `builder.player_commander`. **No
provider change is needed.** The brief's disjunction resolves to its second branch: SIM-3 did
*not* measure a pre-SIM-1 build.

Raw output: `memory/primitives/pb-dx22-measurement-head.txt`. Scratch instrument:
`crates/simulator/examples/dx22_measure.rs` — **untracked; delete it in Stage 1 and never commit
it** (`cargo test` compiles `examples/`, so leaving it in the tree also makes it a silent build
dependency of the baseline).

**Pre-existing-TODO sweep (the roster-recall gate, run for this batch):**

* `Grep "TODO.*(shuffle|commander_ids|player_commander|fuzzer)" -i` over the whole repo →
  **0 source TODOs naming this primitive.** The only hits are card-def TODOs about the
  *keyword action* "shuffle" (`ponder.rs:20`, `endurance.rs:27`) — a different subject.
  Recorded as a positive assertion: **TODO sweep: 0 cards, 0 source sites.**
* The structural analogue of the sweep — *every place in the workspace that puts an object in a
  command zone without registering it* — WAS run and IS load-bearing:
  `Grep "in_zone(ZoneId::Command("` → 29 files. Inside `crates/simulator` the offenders are
  exactly the two the brief names (`src/bin/fuzzer.rs:324`, `tests/local_game.rs:78`) plus the
  scratch example. One out-of-scope hit is recorded as `OOS-DX22-6`
  (`crates/view-model/src/tests.rs:163`). Everything else registers correctly. This sweep is
  machine-frozen by probe **P11** below.

---

## 1. CR rule text (from MCP, verbatim)

* **103.3** — "After the starting player has been determined and any additional steps
  performed, each player shuffles their deck so that the cards are in a random order. … The
  players' decks become their libraries."
* **103.5** — "Each player draws a number of cards equal to their starting hand size, which is
  normally seven. …"
* **903.6** — "At the start of the game, each player puts their commander from their deck face
  up into the command zone. Then each player shuffles the remaining cards of their deck so that
  the cards are in a random order. Those cards become the player's library."
* **903.8** — "A player may cast a commander they own from the command zone. A commander cast
  from the command zone costs an additional {2} for each previous time the player casting it has
  cast it from the command zone that game. This additional cost is informally known as the
  'commander tax.'"
* **903.9a** — "If a commander is in a graveyard or in exile and that object was put into that
  zone since the last time state-based actions were checked, its owner may put it into the
  command zone. This is a state-based action. See rule 704."
* **903.9b** — "If a commander would be put into its owner's hand or library from anywhere, its
  owner may put it into the command zone instead. This replacement effect may apply more than
  once to the same event. This is an exception to rule 614.5."
* **903.10a** — "A player who's been dealt 21 or more combat damage by the same commander over
  the course of the game loses the game. (This is a state-based action. See rule 704.)"

### How the engine implements each, and what that means for this batch

| CR | Engine site | Keyed on | Reachable from the fuzzer today? |
|---|---|---|---|
| 903.6 shuffle | *caller's* job; `setup.rs:403` does it, `fuzzer.rs` does not | — | **No** — the whole defect |
| 903.8 tax | `rules/casting.rs`; mirrored by `legal_actions.rs:1811-1828` `effective_cast_cost` | `PlayerState::commander_ids` | **No** |
| 903.9a return | SBA — `rules/commander.rs:358 check_commander_zone_return_sba`, called from `rules/sba.rs:241` | `PlayerState::commander_ids` | **No** |
| 903.9b redirect | **replacement effects**, registered by `state/builder.rs:1220 register_commander_zone_replacements`, which the builder does **not** derive | `commander_ids` at registration time | **No** |
| 903.10a damage | SBA — `rules/sba.rs:295-310`, reads `PlayerState::commander_damage_received` | populated by combat damage from a registered commander | **No** |
| 103.5 opening hand | *caller's* job; `start_game` deals none (`rules/engine.rs:3504-3514` runs `place_opening_hand_permanents` + `reset_turn_state` only) | — | **No, and deliberately left so — §B2** |

---

## 2. The two fixes

### Fix A — shuffle every library from the game's own seeded RNG (CR 103.3 / 903.6)

Reference implementation is `crates/simulator/src/setup.rs:331` (`let mut rng =
StdRng::seed_from_u64(cfg.seed);`) + `:403` (`deck.main_deck.shuffle(&mut rng);`), inside the
single `for &pid in &player_ids` loop at `:349`.

**Where the fuzzer's shuffle draw sits in its stream — DECIDED AND STATED, as the brief
requires.** The fuzzer keeps its single `StdRng::seed_from_u64(seed)` (`fuzzer.rs:289`) and the
shuffle is drawn **inside the existing per-seat deck loop, immediately after that seat's
`random_deck` call, in ascending `PlayerId` order** — i.e. the draws interleave
`deck₁, shuffle₁, deck₂, shuffle₂, …`, byte-for-byte the `setup.rs` pattern.

Why interleaved rather than "all decks, then all shuffles":

1. `build_initial_state`'s own doc (`setup.rs:311-318`) states the interleaving is
   **load-bearing** *there* — seat 2's decklist depends on seat 1's shuffle — and that splitting
   the loop re-rolls every table (measured: seven tests, `scutemob-187`).
2. For the fuzzer nothing is preserved either way: `fuzzer.rs:29-37` already declares recorded
   fuzz seeds non-portable across engine changes, and this batch is another such boundary. So
   the choice is free, and the free choice should be *the same shape as the reference*, so the
   two build paths stay comparable and a future reader does not have to work out why they
   differ.
3. Practical: it needs no second loop and no second `Vec` of decks.

The fallback branch (`fuzzer.rs:300-304`, 99 Plains when `random_deck` returns `None`) shuffles
too. `SliceRandom::shuffle` consumes RNG on a 99-element slice regardless of element
distinctness, so both branches advance the stream identically per seat; only the deck-construction
draws differ, which is a pre-existing asymmetry this batch does not touch.

### Fix B — register the commander (CR 903.6 / 903.8 / 903.9a / 903.10a)

Two calls, and **both** are required at each site — placing the object in `ZoneId::Command(pid)`
records nothing. `setup.rs:381-399` states the rule and pairs them;
`crates/simulator/tests/commander_cast.rs:9-20` states it again as the one rule every fixture in
that file obeys. This batch makes the pairing structural (§B3).

Sites: `crates/simulator/src/bin/fuzzer.rs:322-327` (via the extracted helper) **and**
`crates/simulator/tests/local_game.rs:74-81`.

---

## 3. The four questions the brief requires answered

### B1 — Does the fuzzer path need `register_commander_zone_replacements`? **YES.**

**Decision**: call `mtg_engine::register_commander_zone_replacements(&mut state)` on the built
state, immediately after `builder.build()`, exactly as `setup.rs:433` does.

**Code path**: `crates/engine/src/state/builder.rs:1220`. It is a free function taking `&mut
GameState`; it reads `PlayerState::commander_ids` and pushes **two** `ReplacementEffect`s per
registered commander — `WouldChangeZone { to: ZoneType::Hand, filter:
ObjectFilter::HasCardId(cid) } → RedirectToZone(Command)` and the same for
`ZoneType::Library`. `GameStateBuilder` does **not** derive them (its doc says so at
`builder.rs:1215-1219`), so if nobody calls the function they do not exist.

**What breaks if it is omitted** — and note that this is *not* the same as "903.9 stops
working", which is the mistake to avoid:

* **CR 903.9a is unaffected.** Graveyard/exile return is a state-based action
  (`rules/commander.rs:358`, called from `rules/sba.rs:241`) keyed on `commander_ids`, not on
  any replacement. It would work from Fix B alone.
* **CR 903.9b silently does not exist.** A commander bounced to its owner's hand, or shuffled
  into their library, stays there forever. No `CommanderZoneRedirect` event is ever emitted, so
  any probe or measurement counting that event is **vacuous rather than failing** — the exact
  shape of blindness this batch exists to remove.
* Consequential state corruption is real, not cosmetic: a commander shuffled into a library is a
  card that CR 903.9b says the owner may retrieve; leaving it there changes library contents and
  therefore every subsequent draw.
* The precedent is unanimous: `setup.rs:428-433` and `crates/engine/src/testing/replay_harness.rs`'s
  script path both pair `player_commander` with this call, and `setup.rs`'s comment says in as
  many words that **both** are required.

Probe **P6** gates it. Revert that must redden it: delete the
`register_commander_zone_replacements` call.

### B2 — Does the fuzzer need an opening hand (CR 103.5)? **NO — out of scope, filed as `OOS-DX22-1`.**

**Decision: OUT OF SCOPE.** Reasons, in the order of their weight:

1. **It reverses a deliberate, documented decision without its own review.**
   `crates/simulator/src/setup.rs:14-17`: *"`crates/simulator/src/bin/fuzzer.rs` is deliberately
   **not** rewired onto this module: its games start every player with an empty hand … Giving it
   real opening hands would silently change what every existing seed reproduces."* Reversing that
   is a design change, and `memory/conventions.md`'s standing "implement-phase default-to-defer"
   rule says a scope extension is a micro-PB with its own plan/review cycle, not a rider.
2. **It closes none of the three seeds.** The horizon collapses on the **shuffle alone**. Today's
   floor is arithmetic: ~34 basics + ≤5 nonbasic lands sit on top (`deck.rs:90-148` appends
   basics last; `zone.rs:159-161`'s `top()` is `v.last()`), so the first non-land is personal
   draw ~35-40 ⇒ game turn ≈136-156 in a 4-player game, and the measured band is 143-154. After
   the shuffle a normal deck is ~60 non-lands of 99, so the first non-land arrives within a
   handful of personal draws. Seven extra opening cards move that by ≤1-2 personal draws. The
   outcome the batch is buying is already bought.
3. **The "batch the re-roll, pay it once" argument does not apply here, and §D is why.** That
   argument assumes there are expensive pins to amortise. There are not: the play-server's eight
   seeded pins build through `setup.rs`, which this batch does not touch, so they do not move at
   all (§D). The re-roll this batch actually pays is confined to a handful of structural tests in
   `crates/simulator/tests/local_game.rs`. There is nothing to amortise.
4. **It would drag a second, independent re-tuning into the batch.** `memory/gotchas-infra.md`
   ("Simulator / play-client Gotchas"): *"`GameDriver`'s `max_turns * 200` command budget is the
   FUZZER's ratio, and the fuzzer's games start with empty hands. A real four-player table dealt
   from the full pool runs ~260 commands/turn, so at that ratio the `InfiniteLoop` valve fires
   before the turn cap."* Dealing seven cards therefore also requires changing
   `driver.rs:51`'s `max_commands`, which changes which `HaltReason` a fuzz run reports and
   invalidates every historical halt-distribution claim by a *second* mechanism. One re-baselining
   per batch.

**File `OOS-DX22-1`** with: the CR cite (103.5), the two call sites that would change
(`fuzz_setup::build_fuzz_state`, and `driver.rs:51`'s ratio), the measured commands/turn from
this batch's A/B (which the successor needs), and the note that it is a natural rider on
feedback-engineering row 6 (**DECK-CHANNEL**), which also re-rolls seeds.

### B3 — Extract the build path into the library? **YES.**

**Decision: extract.** `run_single_game`'s state build lives in `src/bin/fuzzer.rs`, which Cargo
compiles as its own crate; **no integration test can `use` it**, so today the only way to "test"
it is to write a second copy — which is precisely how
`crates/simulator/tests/local_game.rs:56-59` came to exist ("Mirrors
`mtg-fuzzer::run_single_game`'s builder logic") and to carry the identical defect. Registering
the commander in two places without unifying them re-creates the drift on the next change. The
brief's constraint — *do not ship a probe that gates a copy of the code rather than the code* —
is only satisfiable by extraction.

**Home**: a NEW module `crates/simulator/src/fuzz_setup.rs`. Deliberately **not** added to
`setup.rs`: `setup.rs` is the `LocalGame`/play-server pregame path and every play-server seed pin
is a function of it, so keeping the fuzzer's build in a separate file makes "this batch cannot
move a play-server pin" a property a reviewer can check from the diff's **file list** rather than
from its contents.

**Exact public surface** (add `pub mod fuzz_setup;` to `crates/simulator/src/lib.rs` and
re-export `build_fuzz_state`, `place_registered_deck`, `FuzzGameSetup`, `FuzzSetupError`):

```rust
/// The un-started `GameState` `mtg-fuzzer` plays, plus the decklists it was built from.
pub struct FuzzGameSetup {
    pub state: mtg_engine::GameState,
    /// **Pre-shuffle** decks, ascending `PlayerId`, exactly as `random_deck` produced them —
    /// `deck.rs`'s structural order (≤60 non-lands, ≤5 non-basic lands, basics LAST).
    /// The shuffle probe compares the built libraries against this.
    pub decks: Vec<(mtg_engine::PlayerId, crate::deck::DeckConfig)>,
}

#[derive(Debug)]
pub enum FuzzSetupError {
    Builder(mtg_engine::GameStateError),
}

/// CR 103.3 / 903.6 — build the fuzzer's un-started `GameState` for `player_count` seats.
///
/// Deterministic in `seed` alone: one `StdRng::seed_from_u64(seed)`, drawn in ascending
/// `PlayerId` order, per seat `random_deck` then `shuffle` (the `setup.rs` interleaving).
pub fn build_fuzz_state(
    seed: u64,
    player_count: u32,
    cards: &[mtg_engine::CardDefinition],
    registry: &std::sync::Arc<mtg_engine::CardRegistry>,
) -> Result<FuzzGameSetup, FuzzSetupError>;

/// CR 903.6 — place one seat's commander **and register it**, then its library in the given
/// order (the caller shuffles). The two commander steps are ONE operation here so no caller
/// can do half of it; see `setup.rs:381-399` for why half is not a Commander game.
pub fn place_registered_deck(
    builder: mtg_engine::GameStateBuilder,
    pid: mtg_engine::PlayerId,
    deck: &crate::deck::DeckConfig,
    cards: &[mtg_engine::CardDefinition],
    card_defs: &std::collections::HashMap<String, mtg_engine::CardDefinition>,
) -> mtg_engine::GameStateBuilder;
```

`crates/simulator/tests/local_game.rs::build_state` is rewired onto `place_registered_deck`
(it cannot use `build_fuzz_state` — it deliberately uses a fixed 99-Plains deck, not
`random_deck`, for the reason its module doc gives at `:5-14`). That puts the
place-and-register pairing in **exactly one** function in the crate, which is the real anti-drift
fix; probe **P11** then machine-checks that no third copy appears.

**How the probes gate the real path**: `mtg-fuzzer`'s `run_single_game` calls
`build_fuzz_state` and does nothing else to the state, so a probe on `build_fuzz_state` is a
probe on the binary. Stage 1 proves that by re-running the Stage-0 fuzz command and diffing the
per-game output line-for-line (extraction must be behaviour-neutral before any behaviour changes).

### B4 — Honest disposition of each of the three seeds

Registry of record: `docs/audits/decision-point-audit.md` §8.1 (rows around L1060-L1079; locate
by ID, not by line). Update the rows **there**, and mirror one line each into
`memory/primitives/seed-rerank-2026-08-02.md` §2.4 and the §4 PB-DX22 row.

| Seed | Disposition | Evidence that disposition requires |
|---|---|---|
| **`OOS-SIM1-4`** | **CLOSED** | (a) probe **P5** — `commander_ids` populated by *both* build paths, proven red by reverting `player_commander`; (b) probe **P6** — CR 903.9b replacements present; (c) measurement: `CommanderCastFromCommandZone` count over the post-fix A/B run **> 0**, against a measured **0 in ~56,800 commands / 5 games** before. If (c) is 0, the seed is **NOT** closed — STOP and report. |
| **`OOS-UI2-1`** | **CLOSED as a defect + CORRECTION RECORDED** | Its word *"never"* is a horizon artefact, and the row must say so rather than being deleted: *"never" was measured at `--max-turns 80`; the true pre-fix floor is game turn ≈136-156 (arithmetic) / 143-154 (measured).* Closure evidence: probes **P2/P3/P4** (shuffled, seed-deterministic) + probe **P9** (a cast at ordinary depth) + the measured before/after first-cast turns. |
| **`OOS-SIM3-1`** | **CLOSED as a defect + RECORDED AS A THRESHOLD** | Its number (turn 143) is a *correct historical measurement of the unshuffled instrument*, reproduced exactly at HEAD (seed 2, §0 table). Its row keeps a permanent rider: **"any fuzz evidence produced before merge `<sha>` at `--max-turns` below ~140 is a claim about a land-only game."** Do not delete the number; it is the calibration constant that dates every earlier claim. |

**The consequence nobody has written down yet, and this batch must**: closing `OOS-UI2-1` does
**not** retroactively validate the parity claims that leaned on it — it **retires them as
evidence**. At least six shipped artefacts argue "no fuzz seed moves / the change is unreachable
in fuzz" *from this premise*:

* `crates/simulator/src/legal_actions.rs:711-723` (SIM-1's structural-unreachability argument)
* `crates/simulator/src/local_game.rs:552-556` (SIM-5's "seeds cannot reach the changed branch")
* `crates/simulator/src/invariants.rs:212-221` (SIM-3's "what the clean side of that A/B is not
  evidence of")
* `crates/simulator/tests/local_game.rs:2058-2065` (UI-2's 360-game byte-identical A/B)
* `memory/primitives/pb-plan-ENG1.md:915`, `memory/primitives/pb-plan-ENG2.md:1016`
* `docs/mtg-engine-simulator.md:393-396`

Each was *valid when made*. The batch must annotate each (see §5 "comment corrections") with:
premise closed as of PB-DX22, argument not re-validated here. That is the honest reading and it
is the single most valuable output of an EVIDENCE-INTEGRITY batch.

---

## 4. §D — the seed re-derivation, worked out rather than assumed

### The analytical answer: the play-server pins do NOT move.

Chain, verified in code:

1. `tools/play-server/src/session.rs:238` — `new_game` calls
   `mtg_simulator::setup::build_initial_state(&cfg)`, then `setup::dealt_decks`, then
   `LocalGame::start`. Every browser game, every seeded HTTP probe.
2. `setup::build_initial_state` **already** shuffles (`setup.rs:403`), **already** registers
   (`setup.rs:399`), and **already** calls `register_commander_zone_replacements`
   (`setup.rs:433`). There is nothing here for PB-DX22 to change.
3. PB-DX22's behavioural diff is confined to `crates/simulator/src/bin/fuzzer.rs`, the new
   `crates/simulator/src/fuzz_setup.rs`, and `crates/simulator/tests/local_game.rs`. It touches
   **no** line of `setup.rs`, `deck.rs`, `legal_actions.rs`, `mana_solver.rs`, `random_bot.rs`,
   `heuristic_bot.rs`, `local_game.rs` or `params.rs` other than doc comments.

**Therefore the re-derivation reduces to a VERIFICATION, executed and not assumed.** Run the
play-server test target and confirm every seeded fixture still passes and no fixture-drift
message fires. **If any play-server pin moves, STOP** — it means the diff reached something the
plan forbids, and the correct response is to find that, not to re-pin.

The complete pin inventory, from `Grep "_SEED|const SEED|seed:" tools/play-server/src` (this is
the enumerated list, not the brief's remembered one — it is **longer**):

| Pin | Site | Moves? |
|---|---|---|
| `SEED = 0` | `main.rs:268` | no |
| `COMBAT_SEED = 6` | `main.rs:1573` | no |
| `TARGET_SEED = 13` | `main.rs:1608` | no |
| `UI3_SPLIT_COMBAT_SEED = 21` | `main.rs:2329` | no |
| `DISTINCTIVE_SEED = 987_654_321_987` | `main.rs:2990` | no |
| `UI1_SEED = 184` | `main.rs:3494` | no |
| `UI6_SEED = UI1_SEED` | `main.rs:3940` | no |
| `UI6_RESTRICTED_SEED = 29` | `main.rs:4065` | no |
| `SIM1_SEED = UI1_SEED` | `main.rs:5105` | no |
| `UI2_SEED = UI1_SEED` | `main.rs:6027` | no |
| `ENG1_SEED = 7` | `main.rs:9496` | no |
| `REBUILD_FAILURE_SEED = 0xDEAD_BEEF_F00D` | `main.rs:258` | no |
| inline `seed: 7` | `main.rs:3327` | no |

Same reasoning for the other consumers, each verified by reading its state construction:

| Consumer | Build path | Moves? |
|---|---|---|
| `crates/simulator/tests/local_game_playthrough.rs` | `setup::build_initial_state` (`:381`) | **no** |
| `crates/simulator/tests/setup.rs` | `setup::build_initial_state` | **no** |
| `crates/simulator/tests/sim5_bot_cast_discipline.rs` | `setup::*` | **no** |
| `crates/simulator/tests/sim2_mana_intelligence.rs` | hand-built / `setup` | **no** (verify) |
| `crates/simulator/tests/local_game_human_actions.rs` | hand-built `GameStateBuilder` (`:71`, `:635`), no command-zone objects | **no** |
| `crates/simulator/tests/commander_cast.rs` | hand-built, already registers | **no** |
| `tools/tui/src/play/app.rs:155` | `mtg_simulator::build_initial_state` | **no** |
| `crates/view-model` | pure render fixtures, no game driven | **no** |
| **`crates/simulator/tests/local_game.rs`** | its own `build_state` — **edited by this batch** | **YES** |

### The batch's real seed churn, and how to handle it

Only `crates/simulator/tests/local_game.rs`. Registering the commander adds a command-zone
`CastSpell` to `StubProvider`'s offer list whenever it is affordable, and `RandomBot` chooses by
**index** into that list (`legal_actions.rs:706-718` says so explicitly), so every game in that
file re-rolls from the first turn a commander is affordable.

Its tests are structural, not outcome-pinned — `test_local_game_bot_only_matches_game_driver_for_fixed_seeds`
compares the *two paths against each other* on the same states, and the rest assert decision
shapes, journal/command-count agreement and halt reasons. **Expected: all pass unchanged.** If any
fails, the rule is **re-derive, never adjust**: read what the new deal produces, and re-pin from
the observed run with the reason recorded at the pin (the `UI1_SEED` convention).

### Other recorded seeds this merge invalidates (not "pins" — repro procedures)

* `memory/workstream-state.md:2903` — *"Reproduce: `mtg-fuzzer --games 1 --seed 504 --max-turns 200`"*
  (the `OOS-SIM2-6` crash repro). **Dead** after this merge. Annotate in place with the merge sha
  and a pointer to `OOS-DX22-7`; do not silently leave it.
* `crates/simulator/src/invariants.rs:200-221` — the SIM-3 A/B table pinned to
  `--games 5 --seed 1 --max-turns 200` (8,781 → 0). The *numbers* stay as history; the block's
  own caveat at `:212-221` must be updated with the post-shuffle re-measurement (see
  `OOS-DX22-3`).
* `docs/mtg-engine-simulator.md:621` — `--games 100 --seed 42` is a smoke-test recipe, not a pin.
  No change needed beyond §5's correction to `:393-396`.
* `crash-reports/` is gitignored (`.gitignore:52`), so there is no committed crash artefact to
  re-derive. Confirm with `git ls-files crash-reports` returning empty.

---

## 5. Staged implementation

Every stage: independently committable, independently testable. After each stage run
`cargo build --workspace`, `cargo test -p mtg-simulator`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`,
`tools/check-defs-fmt.sh` (SR-35). Commit prefix `W6-prim:`.

### Stage 0 — baseline, no source edits

1. **Full workspace baseline, captured to a file, never tail-piped** (the 2026-08-02 lesson):
   ```
   ~/.cargo/bin/cargo test --workspace --no-fail-fast > /tmp/pb-dx22-baseline.txt 2>&1
   awk '/^test result/ {p+=$4; f+=$6; i+=$8} END {print p, f, i}' /tmp/pb-dx22-baseline.txt
   ```
   Record passed / failed / ignored. Expected ≈ 4,345 / 0 / 5.
2. **Gate-EXECUTE the wire sentinels** (never predict):
   `cargo test -p mtg-engine --test core hash_schema` and
   `cargo test -p mtg-engine --test core protocol_schema`. Record **HASH 72 / PROTOCOL 35**.
3. **Baseline fuzz run** under the assertions profile (`Cargo.toml:51-54`):
   ```
   cargo run --profile fuzz --bin mtg-fuzzer -- --games 20 --seed 1 --max-turns 200 \
       --threads 1 --verbose > /tmp/pb-dx22-fuzz-before.txt 2>&1
   ```
   Record: games completed, avg turns, total violations broken down by `check`, error/halt
   distribution, wall time.
4. Confirm `git status` shows `crates/simulator/examples/dx22_measure.rs` as **untracked** and
   `git ls-files crash-reports` is empty.

### Stage 1 — extract the build path, behaviour-NEUTRAL

* New `crates/simulator/src/fuzz_setup.rs` with `FuzzGameSetup`, `FuzzSetupError`,
  `place_registered_deck` and `build_fuzz_state` — **at this stage `place_registered_deck` does
  NOT call `player_commander`, `build_fuzz_state` does NOT shuffle and does NOT call
  `register_commander_zone_replacements`.** It is a verbatim lift of `fuzzer.rs:289-359`.
* `crates/simulator/src/lib.rs`: `pub mod fuzz_setup;` + re-exports.
* `crates/simulator/src/bin/fuzzer.rs`: `run_single_game` shrinks to `build_fuzz_state(seed,
  player_count, cards, registry)`, mapping `FuzzSetupError::Builder(e)` to the existing
  `GameDriverError::EngineError(format!("Failed to build state: {:?}", e))` string — **keep that
  string byte-identical**, crash reports and the `driver.rs:91-97` comment depend on its shape.
* Preserve the existing `if let Some(def)` silent-skip semantics verbatim (a missing def drops
  the card). Do not "fix" it here; file `OOS-DX22-4`.
* **Delete `crates/simulator/examples/dx22_measure.rs`.**
* **Neutrality evidence (mandatory)**: re-run Stage 0 step 3 into
  `/tmp/pb-dx22-fuzz-after-stage1.txt` and `diff` the two. It must be identical except wall
  time. If it is not, the extraction changed something — find it before proceeding.
* Probe **P1** (see §6).

### Stage 2 — CR 103.3 / 903.6 shuffle

* In `build_fuzz_state`'s per-seat loop: capture the pre-shuffle `DeckConfig` into
  `FuzzGameSetup::decks`, then `deck.main_deck.shuffle(&mut rng);` (`use rand::seq::SliceRandom;`).
  Cite CR 103.3 / 903.6 at the call, and state the interleaving decision from §2 Fix A in the
  doc comment.
* Probes **P2, P3, P4**.
* Re-run the fuzz command into `/tmp/pb-dx22-fuzz-after-stage2.txt`; record the first-cast turn
  band and the halt/error distribution. **Expect this to change a lot** — that is the point.

### Stage 3 — CR 903.6 / 903.8 / 903.9 commander registration

* `place_registered_deck` gains `builder.player_commander(pid, deck.commander.clone())`,
  adjacent to the `ObjectSpec::card(...).in_zone(ZoneId::Command(pid))` placement, with the
  `setup.rs:381-393` rationale restated (the two are one operation).
* `build_fuzz_state` gains, after `builder.build()`:
  `mtg_engine::register_commander_zone_replacements(&mut state);` — CR 903.9b, §B1.
* `crates/simulator/tests/local_game.rs::build_state` (`:60-96`) is rewired onto
  `place_registered_deck`; its doc at `:56-59` is corrected.
* Probes **P5, P6, P7, P8, P11**.
* Re-run the fuzz command into `/tmp/pb-dx22-fuzz-after-stage3.txt`.

### Stage 4 — the ordinary-depth probes

* Probes **P9, P10** (see §6 for why P10 is a measurement, not an assertion).

### Stage 5 — measurement, corrections, seeds, bookkeeping

* **The A/B, both directions, named seeds, instrument named** (the house rule, and SIM-5's
  `sim5_bot_cast_discipline.rs` is the precedent):
  `--games 20 --seed 1 --max-turns 200 --threads 1 --profile fuzz`, before vs after. Report at
  minimum: games completed; avg turns; **first `SpellCast` turn per game**; total casts;
  **`CommanderCastFromCommandZone` count** (before = 0, measured); `CommanderZoneRedirect` +
  `CommanderReturnedToCommandZone` counts; `commander_damage_received` non-empty in how many
  games; violations by `check`; halt/error distribution; commands/turn.
* **Comment corrections — the aspirationally-wrong-comment rule (`memory/conventions.md`).**
  Every one of these currently asserts something this batch makes false. Correct each in place;
  none may be left standing:

  | File:line | What it now says | Required correction |
  |---|---|---|
  | `crates/simulator/src/legal_actions.rs:711-723` | "`commander_ids` is empty in every fuzzer game and this loop cannot fire there at all… structural unreachability, filed as `OOS-SIM1-4`" | past tense; `OOS-SIM1-4` CLOSED by PB-DX22; recorded fuzz seeds DID move here |
  | `crates/simulator/src/local_game.rs:552-556` | "Per `OOS-UI2-1` the fuzzer has never cast a spell at all, so the fuzzer's seeds cannot reach the changed branch" | premise closed; SIM-5's parity claim is **not** re-validated by this batch |
  | `crates/simulator/src/invariants.rs:212-221` | "the fuzzer never shuffles… the run proves the check is quiet on ordinary casts" | post-shuffle re-measurement; a `stack_consistency` violation is now a real finding (`OOS-SIM3-5`) |
  | `crates/simulator/src/setup.rs:14-17` | "`bin/fuzzer.rs` is deliberately not rewired onto this module" | still true, but say **what now differs**: opening hand, `validate_deck`, `DeckSource`. **Doc-comment lines only — 0 code lines in this file.** |
  | `crates/simulator/src/bin/fuzzer.rs:29-37` | "Repro seeds are not portable across engine changes" | append PB-DX22's merge sha as a new boundary event |
  | `crates/simulator/tests/local_game.rs:56-59`, `:2058-2065` | "Mirrors `run_single_game`'s builder logic"; the UI-2 25,964-observation block | now shares `place_registered_deck`; UI-2's 360-game byte-identical A/B is history, not evidence |
  | `docs/mtg-engine-simulator.md:393-396` | "its games are not Commander games at all… which is also *why* no recorded fuzz seed moves" | corrected; the reason no seed moved is closed |
  | `docs/mtg-engine-feedback-engineering.md:144,160,518,551-570` | row 1 as queued | row 1 SHIPPED, merge sha, and the measured answer to its open measurement |
  | `memory/primitives/seed-rerank-2026-08-02.md` §2.4, §4 PB-DX22 row | "one open measurement this task could not settle" | settled; record the answer and the disposition table from §B4 |
  | `memory/workstream-state.md:2903` | `--seed 504` repro | annotate dead-across-this-merge |

* **Seeds to file** (registry: `docs/audits/decision-point-audit.md` §8.1):
  * `OOS-DX22-1` — no opening hand (CR 103.5); §B2's four reasons + the commands/turn measurement.
  * `OOS-DX22-2` — `driver.rs:51`'s `max_commands = max_turns * 200` was calibrated on land-only
    games; report the post-shuffle commands/turn and whether `HaltReason::InfiniteLoop` now
    precedes the turn cap.
  * `OOS-DX22-3` — `invariants.rs`'s A/B table and `stack_consistency`'s clean side were measured
    on land-only games; the first shuffled run is that check's first real test.
  * `OOS-DX22-4` — `fuzz_setup` silently drops a deck card whose def is missing, producing a
    short library with no error; `setup.rs` refuses with `MissingCardDefinition`. Divergence
    recorded, not fixed.
  * `OOS-DX22-5` — the fuzzer never runs `validate_deck`; a fuzz deck is admitted by
    `check_all_defs_complete` alone, so CR 903.5a/903.4 are unchecked there (`deck.rs:117-119`
    already notes the consequence).
  * `OOS-DX22-6` — `crates/view-model/src/tests.rs:163` places a command-zone object with no
    `player_commander`; harmless in a render fixture, same shape as the closed defect.
  * `OOS-DX22-7` — recorded crash seeds are not portable across this merge; feeds
    feedback-engineering row 2 (**FUZZ-CRASH**).
  * plus anything the A/B run surfaces (§8).
* Update `memory/primitive-wip.md`, `memory/workstream-state.md` (handoff), `CLAUDE.md` Current
  State delta (a NEW short bullet — never grow an existing line).

---

## 6. Probes — the deliverable

**File**: `crates/simulator/tests/pb_dx22_fuzz_instrument.rs` (new).
**SR-9a does not apply here** — that gate (`crates/engine/tests/no_stray_test_binaries.rs`) is
scoped to `CARGO_MANIFEST_DIR` = `crates/engine`. `crates/simulator/tests/` is already seven flat
files; adding one is correct.

Escape hatches are available: `crates/simulator/Cargo.toml:36` dev-depends on `mtg-engine` with
`features = ["test-util"]`, so `state.players_mut()`, `state.replacement_effects()`,
`mtg_engine::state::test_util::move_object_to_zone` and
`mtg_engine::rules::commander::check_commander_zone_return_sba` are all reachable
(`commander_cast.rs:104` already uses `players_mut`).

**Every probe must be proven to discriminate by EXECUTING its revert, not by inspection** — and
per `memory/gotchas-infra.md`, *a revert-and-rerun proves nothing unless the rebuild succeeded*:
check for a compile line, and add `#[allow(unused)]` if the disabled version warns, because
`-D warnings` turns a warning into a build failure and `cargo test` then runs the **stale** binary
and reports a pass.

| # | Test name | Asserts | Revert that must redden it |
|---|---|---|---|
| **P1** | `test_dx22_build_fuzz_state_produces_the_fuzzers_table` | 4 seats; each has exactly 1 object in `ZoneId::Command(pid)` and 99 in `ZoneId::Library(pid)`; `decks[i].1.main_deck.len() == 99`; hand is empty (CR 103.5 not dealt — pins the §B2 decision) | change any of those counts in `build_fuzz_state` |
| **P2** | `test_dx22_libraries_are_shuffled_cr_103_3` | for each seat: `library_card_ids != pre_shuffle_main_deck` **as an ordered sequence**, and **equal as a multiset** (sorted compare). Non-vacuity floor: both are exactly 99 long. Read order from `state.zones().get(&ZoneId::Library(pid)).object_ids()` (the `Zone::Ordered` vector), not from `objects_in_zone` | delete `deck.main_deck.shuffle(&mut rng)` → sequences become identical |
| **P3** | `test_dx22_shuffle_is_seed_deterministic` | `build_fuzz_state(1,..)` twice → identical per-seat library `CardId` sequences **and** identical `public_state_hash` | seed the shuffle from anything but the game RNG (e.g. `rand::rng()`) |
| **P4** | `test_dx22_different_seed_different_order` | `build_fuzz_state(1,..)` vs `build_fuzz_state(2,..)`: seat 1's library sequence differs. (Multiset may also differ — different deck; that is fine and must not be asserted equal) | as P3 |
| **P5** | `test_dx22_commander_ids_are_registered_by_both_build_paths` | (a) `build_fuzz_state`: every seat's `commander_ids` == `[decks[i].1.commander]`; (b) `local_game.rs::build_state`'s deck: every seat's `commander_ids` == the fixed deck's commander. **Both halves in one test or it proves half the fix.** | delete `builder.player_commander(..)` in `place_registered_deck` |
| **P6** | `test_dx22_cr_903_9b_replacements_are_registered` | `state.replacement_effects()` contains, per seat, exactly one `WouldChangeZone { to: Hand, filter: HasCardId(cmdr) } → RedirectToZone(Command)` and one with `to: Library`; total == `2 * player_count` | delete the `register_commander_zone_replacements` call |
| **P7** | `test_dx22_cr_903_9a_zone_return_sba_is_reachable_from_the_fuzz_build` | build → `mtg_engine::start_game` → `test_util::move_object_to_zone(cmdr → Graveyard(p1))` → `rules::commander::check_commander_zone_return_sba(&mut state)` → `state.pending_commander_zone_choices()` names `(p1, new_id)` | delete `player_commander` → SBA sees no commander → the vector stays empty |
| **P8** | `test_dx22_cr_903_8_tax_applies_on_the_fuzz_build` | on the fuzz-built state: `effective_cast_cost(&state, p1, cmdr_obj)` == printed cost with tax 0; then `players_mut()…commander_tax.insert(cid, 1)` ⇒ == `apply_commander_tax(&printed, 1)` (printed + {2}). `effective_cast_cost` gates on `commander_ids` at `legal_actions.rs:1823` | delete `player_commander` → the taxed assertion returns the printed cost |
| **P9** | `test_dx22_a_spell_is_cast_at_an_ordinary_depth` | seeds `[1,2,3,4]`, 4 players, `RandomBot`, `LocalGameLimits { max_turns: 30, max_commands: 30*400, max_consecutive_passes: 500, record_journal: true }`, `human_seats` empty. For **each** seed, some `CommandRecord` with `Command::CastSpell(..)` and `record.turn <= 30`. Test must also **print** the observed first-cast turn per seed | delete the shuffle → no cast before turn ~136, every seed reddens |
| **P10** | *(measurement, not a test)* | `CommanderCastFromCommandZone` count over the Stage-5 A/B run must be **> 0** (before: 0 in ~56,800 commands). NOT asserted in a unit test: a commander needs 3-6 lands, which is not reliably reached inside a 30-turn debug-profile probe, and a statistical assertion in the suite is exactly the kind of flake this project bans | n/a — if the count is 0, `OOS-SIM1-4` is **not** closed; STOP and report |
| **P11** | `test_dx22_command_zone_placement_and_registration_are_one_operation` | source gate over `crates/simulator/**/*.rs` (src + tests, via `CARGO_MANIFEST_DIR`): **every file containing `in_zone(ZoneId::Command(` must also contain `player_commander`**. Non-vacuity floor: assert the matched-file set is non-empty and ≥ 4 | **already red on the pre-fix tree** — `tests/local_game.rs` violates it today. That is the strongest available discrimination proof; record it |

**Why the P9 threshold is a floor and not a tuned number.** The pre-fix floor is *arithmetic*:
~34 basics on top ⇒ first non-land at personal draw ~35-40 ⇒ game turn ≈136-156 in a 4-player
game, and the measured band was 143-154 (§0). A gate at turn 30 sits **>4× below** that, so it
cannot be satisfied by the old behaviour under any seed. It is also expected to sit well **above**
the post-fix reality (~60 non-lands of 99 ⇒ a non-land within a few personal draws; a castable
1-2 drop needs 1-2 lands). **Binding rule for the runner: record the observed first-cast turn on
every seed. If any exceeds 15, do NOT raise the gate to fit the data** — investigate (the likely
suspects are `OOS-UI2-2`-shaped upkeep tap-waste and `OOS-SIM6-3`'s activation-cost mana gap),
file what you find, and report. Raising a gate to fit a run is the failure mode this queue exists
to stop.

**Runtime caution**: `cargo test` builds debug. If P9's four games exceed ~60 s wall time, reduce
to seeds `[1,2]` — **keep the turn-30 threshold**, never the other way round — and say so at the
test.

---

## 7. Verification checklist

- [ ] Stage-0 full-workspace baseline captured to a **file** (never `| tail`) and summed with awk
- [ ] HASH **72** and PROTOCOL **35** gate-EXECUTED at Stage 0 and again at Stage 5 — unmoved
- [ ] `git diff main..HEAD --numstat -- crates/engine/ crates/card-defs/ crates/card-types/ crates/view-model/ tools/` is **EMPTY**
- [ ] `git diff main..HEAD -- crates/simulator/src/setup.rs` changes **only** `//!`/`///` lines
- [ ] `crates/simulator/examples/dx22_measure.rs` deleted and never committed (`git ls-files crates/simulator/examples` empty)
- [ ] Stage-1 extraction proven behaviour-neutral by a byte-identical fuzz-output diff
- [ ] P1-P9 + P11 all present; **each proven red by an executed revert whose rebuild succeeded**
- [ ] P11's pre-fix redness recorded (it fails on the merge base — say so)
- [ ] Card coverage unmoved — prove by an empty `git diff -- crates/card-defs` **and** a
      `tools/authoring-report.py` regeneration with a byte-identical body (1,133/1,803 = 62.8%)
- [ ] A/B measurement reported both directions on named seeds with the instrument named
- [ ] `CommanderCastFromCommandZone` count **> 0** after (0 before)
- [ ] Play-server seed pins **verified unmoved by execution**: `cargo test -p mtg-play-server`
      green, no fixture-drift message
- [ ] All 10 comment/doc correction sites in §5 corrected
- [ ] Seeds `OOS-DX22-1..7` (+ any new) filed in `docs/audits/decision-point-audit.md` §8.1
- [ ] `OOS-SIM1-4` CLOSED; `OOS-UI2-1` CLOSED+corrected; `OOS-SIM3-1` CLOSED+threshold recorded
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` **and** `tools/check-defs-fmt.sh` (SR-35)
- [ ] Final `cargo test --workspace --no-fail-fast` to a file; residual list empty

---

## 8. Risks and edge cases

1. **A previously unreachable pre-existing crash surfaces.** This is a *foreseeable outcome of
   making the instrument work*, and "the batch caused it" is the wrong reading. Protocol:
   (a) establish the diff changed **0 engine lines** (`git diff main..HEAD --numstat --
   crates/engine/` empty) — that is the strongest available argument that the defect is
   pre-existing, and it is stronger than trying to reproduce on the merge base, which *cannot*
   shuffle; (b) capture seed, `--max-turns`, profile, and a `gdb` backtrace if it is a SIGABRT
   (`catch_unwind` does not contain an abort — `pb_dx19_characteristics_recursion.rs:24,169`);
   (c) name the card if a depth probe can (PB-DX19's method); (d) **file, do not fix** —
   conventions' default-to-defer; (e) if it makes the mandatory A/B impossible (0 of N games
   complete), reduce `--max-turns`, record the limitation explicitly, and **STOP and report**
   rather than shipping weakened acceptance evidence.
2. **`max_commands` binds before `max_turns`.** `driver.rs:51`'s `max_turns * 200` was
   calibrated on land-only games (`memory/gotchas-infra.md`); a real table runs ~260
   commands/turn. Post-shuffle runs may report `HaltReason::InfiniteLoop` where they used to
   report `MaxTurns`. **Do not re-tune it in this batch** — measure it, report the distribution,
   file `OOS-DX22-2`. P9 sets its own generous `max_commands` (30*400) precisely so that a P9
   failure means "no cast", never "budget exhausted".
3. **`stack_consistency` and the other eight invariants run against real casts for the first
   time.** SIM-3's clean A/B was measured on land-only games and its own doc says so
   (`invariants.rs:212-221`); `OOS-SIM3-5` names two live engine defects that legitimately trip
   the check and could not have been caught. Expect new violations. They are findings, not
   regressions; classify by `check` and file.
4. **Noise floor.** `OOS-SIM3-4`: 929 of 938 residual violations are `no_orphaned_tokens`
   reports that `OOS-M11-7` says are expected, and `--stop-on-error` halts on one. Run the A/B
   **without** `--stop-on-error` and report counts by `check`, or the measurement is unreadable.
5. **`crates/simulator/tests/local_game.rs` re-rolls.** Expected to pass unchanged (its tests are
   structural). If one fails: re-derive from the observed run and record the reason at the pin —
   never numerically adjust.
6. **Deck legality is unchecked on the fuzz path.** `random_deck` is `Complete`-filtered but
   never passes `validate_deck` (`deck.rs:117-119` says the fuzzer "silently PLAYED those illegal
   decks"). Registering commanders makes CR 903.4 colour identity newly *relevant* to the fuzz
   games without making it *checked*. Out of scope; `OOS-DX22-5`.
7. **Silent short libraries.** `place_registered_deck` inherits `if let Some(def)`; a missing def
   drops a card and no error is raised. Preserved deliberately (behaviour-neutral extraction);
   `OOS-DX22-4`.
8. **A colourless-commander deck's padding is colourless nonlands, not basics**
   (`deck.rs:128-142`), so P2's "basics last" intuition does not hold for those seats. P2 is
   written as sequence-inequality + multiset-equality precisely so it is structure-independent —
   do not "improve" it into a basics-position assertion.
9. **A commander deck with a partner** cannot be represented by `DeckConfig`
   (`setup.rs:233-237`, `OOS-SIM4-3`). `random_deck` never builds one, so `commander_ids` is
   always a 1-element vector on this path; P5 may assert exactly one.
10. **Do not add an action to `StubProvider`.** `memory/gotchas-infra.md`: `RandomBot` picks an
    index, so appending anything re-rolls every draw downstream. This batch needs no provider
    change (§0), and adding one would be a second, unrelated seed re-roll.

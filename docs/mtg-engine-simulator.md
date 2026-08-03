# MTG Engine — Game Simulator & Fuzzer

<!-- last_updated: 2026-08-03 -->

> Design document for automated game simulation, fuzz testing, and interactive TUI play.

## Context

The engine has 974 tests and ~80 approved game scripts, but all testing is scripted — no full games have been played end-to-end with real decision-making. Before playing with friends, we need brute-force testing via thousands of automated games plus interactive play against bots to surface bugs that scripted tests miss.

**Sequencing decision**: Build the simulator shell (bot framework, fuzzer, TUI) NOW in new crates with zero engine modifications. Use a stub `LegalActionProvider` for basic move enumeration. Wire in the full `legal_actions.rs` engine module LATER, after ability coverage is complete. This avoids collisions with ongoing ability implementation work.

## Architecture Overview

```
crates/simulator/                          <-- Phases 1-2 (NOW): bot + fuzzer
tools/tui/src/play/                        <-- Phase 3 (NOW): interactive TUI
crates/engine/src/rules/legal_actions.rs   <-- Phase 4 (LATER): real legal actions
```

---

## Phase 1: Simulator Crate — Bot Framework + Game Driver (NOW)

**Goal**: New `crates/simulator/` crate with bot trait, game driver, mana solver, and stub legal action provider. Zero engine modifications.

### The Stub Strategy

Define a `LegalActionProvider` trait. Ship a `StubProvider` that does basic checks without deep engine knowledge. Later, swap in a `FullProvider` backed by `legal_actions.rs`.

```rust
// crates/simulator/src/legal_actions.rs
pub trait LegalActionProvider: Send {
    fn legal_actions(&self, state: &GameState, player: PlayerId) -> Vec<LegalAction>;
}

/// Basic legal action enumeration — enough to play games,
/// but misses edge cases that the full engine implementation will catch.
pub struct StubProvider;

impl LegalActionProvider for StubProvider {
    fn legal_actions(&self, state: &GameState, player: PlayerId) -> Vec<LegalAction> {
        // 1. Always: PassPriority, Concede
        // 2. Play lands: check hand for lands, land_plays_remaining > 0, main phase, stack empty
        // 3. Cast from hand: check mana pool vs cost, basic timing (instant anytime, sorcery main+empty stack)
        // 4. Tap for mana: iterate battlefield for untapped permanents with mana abilities
        // 5. Declare attackers: untapped creatures without summoning sickness (if DeclareAttackers step)
        // 6. Declare blockers: untapped creatures (if DeclareBlockers step)
        // 7. Mulligan: TakeMulligan / KeepHand if in mulligan phase
        // 8. Commander zone choices: if pending
    }
}
```

The stub is ~150-200 lines. It won't handle flashback, escape, foretell, cycling, abilities, etc. — those get added when the full `legal_actions.rs` replaces it.

### LegalAction Enum

Lives in the simulator crate for now. Moves to engine in Phase 4.

```rust
pub enum LegalAction {
    PassPriority,
    Concede,
    PlayLand { card: ObjectId },
    CastSpell { card: ObjectId, from_zone: ZoneId },
    TapForMana { source: ObjectId, ability_index: usize },
    ActivateAbility { source: ObjectId, ability_index: usize },
    DeclareAttackers { eligible: Vec<ObjectId>, targets: Vec<AttackTarget> },
    DeclareBlockers { eligible: Vec<ObjectId>, attackable: Vec<ObjectId> },
    TakeMulligan,
    KeepHand,
    ReturnCommanderToCommandZone { object_id: ObjectId },
    LeaveCommanderInZone { object_id: ObjectId },
}
```

### Bot Trait

```rust
pub trait Bot: Send {
    fn choose_action(&mut self, state: &GameState, player: PlayerId, legal: &[LegalAction]) -> Command;
    fn choose_targets(&mut self, state: &GameState, valid: &[ObjectId], count: usize) -> Vec<ObjectId>;
    fn choose_attackers(&mut self, state: &GameState, eligible: &[ObjectId], targets: &[AttackTarget]) -> Vec<(ObjectId, AttackTarget)>;
    fn choose_blockers(&mut self, state: &GameState, eligible: &[ObjectId], attackers: &[ObjectId]) -> Vec<(ObjectId, ObjectId)>;
    fn choose_mulligan_bottom(&mut self, hand: &[ObjectId], count: usize) -> Vec<ObjectId>;
    fn name(&self) -> &str;
}
```

### Two Bot Tiers (NOW)

| Bot | Strategy | Purpose |
|-----|----------|---------|
| **RandomBot** | Uniform random from legal actions | Fuzzing — maximizes state space coverage |
| **HeuristicBot** | Weighted scoring | More realistic games, finds interaction bugs |

**RandomBot**: Seeded `StdRng`, uniform selection. Bias toward attacking (80/20) to ensure games progress.

**HeuristicBot** scoring:
- Play a land: +100 (always first)
- Cast a spell: +50 base, +10 per mana value, +20 if removal
- Attack with creature: +30 if opponent tapped out, +10 otherwise
- Pass priority: +1 (last resort)
- Hold up mana for instant: +25 if interaction in hand

### Mana Solver

```rust
/// Greedy mana payment: for each colored pip, tap a source that produces
/// that color. For generic, tap any remaining source.
pub fn solve_mana_payment(state: &GameState, player: PlayerId, cost: &ManaCost) -> Option<Vec<Command>>
```

### Game Driver

```rust
pub struct GameDriver<P: LegalActionProvider> {
    provider: P,
    bots: HashMap<PlayerId, Box<dyn Bot>>,
    max_turns: u32,
    event_log: Vec<(Command, Vec<GameEvent>)>,
    rng: StdRng,
}

impl<P: LegalActionProvider> GameDriver<P> {
    pub fn run_game(&mut self, state: GameState) -> GameResult {
        // 1. start_game(state) → events
        // 2. Loop:
        //    a. Check game over / max turns
        //    b. Determine acting player (priority_holder or pending choice)
        //    c. provider.legal_actions(state, player)
        //    d. bot.choose_action(state, player, legal) → Command
        //    e. process_command(state, command) → (new_state, events)
        //    f. Log command + events
        //    g. Check invariants
    }
}

pub struct GameResult {
    pub seed: u64,
    pub winner: Option<PlayerId>,
    pub turn_count: u32,
    pub total_commands: usize,
    pub event_log: Vec<(Command, Vec<GameEvent>)>,
    pub violations: Vec<InvariantViolation>,
    pub error: Option<GameDriverError>,
}
```

### Deck Builder

Build decks from the existing 66+ CardDefinitions:

```rust
pub fn random_deck(rng: &mut StdRng, registry: &CardRegistry) -> DeckConfig {
    // Pick a legendary creature as commander
    // Fill with cards matching color identity
    // Pad with basic lands
}
```

### Crate Structure

```
crates/simulator/
  Cargo.toml              # depends on mtg-engine, rand, rayon, clap, indicatif, serde_json
  src/
    lib.rs
    legal_actions.rs      # LegalAction enum, LegalActionProvider trait, StubProvider
    bot.rs                # Bot trait
    random_bot.rs         # RandomBot
    heuristic_bot.rs      # HeuristicBot
    driver.rs             # GameDriver<P> game loop
    mana_solver.rs        # Greedy mana payment
    invariants.rs         # InvariantChecker + all checks
    deck.rs               # Deck construction helpers
    report.rs             # CrashReport serialization
    bin/
      fuzzer.rs           # Fuzzer CLI binary
```

### Files

| File | Action | Touches engine? |
|------|--------|-----------------|
| `crates/simulator/Cargo.toml` | NEW | No |
| `crates/simulator/src/*.rs` (11 files) | NEW | No |
| `Cargo.toml` (workspace) | MODIFY — add member | No |

### Testing

- Unit test: StubProvider returns expected actions for known states
- Unit test: RandomBot picks from legal actions without panicking
- Unit test: mana solver finds payment for simple costs
- Integration: RandomBot plays a 4-player game to completion (no panics)
- Integration: HeuristicBot plays a game (game ends in < 200 turns)

---

## Phase 2: Fuzzer CLI (NOW)

**Goal**: Binary that runs thousands of games in parallel, checks invariants, reports crashes.

### CLI

```
mtg-fuzzer [OPTIONS]

  --games <N>         Number of games (default: 1000)
  --players <N>       Players per game, 2-6 (default: 4)
  --max-turns <N>     Turn limit (default: 200)
  --seed <SEED>       Base RNG seed (default: random)
  --threads <N>       Parallel threads (default: num_cpus)
  --bot <TYPE>        random | heuristic (default: random)
  --stop-on-error     Stop after first violation
  --replay <SEED>     Replay a specific game by seed
  --verbose           Print each game result
```

### Invariant Checks (run after every state transition)

> **Nine of these twelve can fire.** Re-derived from `invariants::check_all` by SIM-3
> (`scutemob-177`, 2026-08-02) rather than trusted: #3 is an explicit no-op in the
> source, and **#10 and #11 do not exist** — no function implements them and no
> violation in the codebase carries their name. They are marked below rather than
> deleted, because they are still worth having; filed as **OOS-SIM3-2**. A list of
> checks is not a list of checks that run.

1. **Zone integrity**: Every object in exactly one zone
2. **ID uniqueness**: No duplicate ObjectIds
3. **Mana non-negative**: All mana pool values >= 0 — **no-op**: `ManaPool`'s fields are
   `u32` and cannot go negative, so `check_mana_non_negative` has an empty body
4. **Stack consistency**: `ZoneId::Stack` holds exactly the cards named by the stack
   objects that put a card there — one apiece, in the same order.
   **This line used to read "stack_objects matches objects in Stack zone", and that is
   wrong.** A `StackObject::id` and the Stack-zone `ObjectId` of the card being cast are
   two different id namespaces (`casting.rs::handle_cast_spell` mints the zone object
   under CR 400.7, then takes the *next* id for the stack entry), so they never match in
   a healthy game and the check that compared them fired on every spell and every
   ability. `invariants::stack_card_of` now classifies each `StackObjectKind` — only
   `Spell` and `MutatingCreatureSpell` own a Stack-zone card — and
   `invariants::check_stack_consistency` asserts the four properties documented on it.
   See its doc comment for the measured before/after.
5. **Player consistency**: Active player and priority holder are alive
6. **Turn order**: All players in turn_order, active player present
7. **Object-zone agreement**: Object's zone field matches containing zone
8. **Attachment validity**: attached_to references existing battlefield objects
9. **Game progression**: Turn number never decreases
10. **Legal action soundness**: Actions from provider don't get rejected by `process_command()`
    — **NOT IMPLEMENTED** (OOS-SIM3-2). Nothing in `invariants.rs` checks this. It is the
    SR-38 property, and it is currently enforced only by the assertions in
    `local_game_playthrough.rs` (a policy rejection fails that test), not by the fuzzer.
11. **SBA idempotency**: After SBAs, running again produces no events — **NOT IMPLEMENTED**
    (OOS-SIM3-2).
12. **No orphaned tokens**: No tokens in non-battlefield zones after SBAs

### Crash Reports

```rust
pub struct CrashReport {
    pub seed: u64,
    pub violation: InvariantViolation,
    pub command_history: Vec<Command>,   // full replay
    pub state_before: GameState,
    pub turn_number: u32,
}
```

Serialized as JSON — loadable in the replay viewer for debugging.

### Parallel Execution

`rayon::par_iter` over game seeds. Each game is independent.

### Progress Display

`indicatif` progress bar showing: games completed, violations found, current games/sec.

```
[████████████████░░░░░░░░░░░░░░] 534/1000 games  2 violations  47 games/sec
```

### Files

All in `crates/simulator/` — already listed in Phase 1 crate structure. The `bin/fuzzer.rs` is the CLI entry point; `invariants.rs` and `report.rs` handle checking and output.

---

## Phase 3b: The web play client (M11-local, ADDED 2026-08-01)

> Not in the original 2026-02-28 design, which had exactly one interactive host (the
> TUI). M11-local added a second — a **browser** client — and, in the course of doing
> so, moved the "1 human + 3 bots" machinery out of the TUI and into this crate where
> both hosts share it. `tools/tui` is unchanged in what it does; it is no longer the
> only thing that does it.

**What lives in `crates/simulator` now, and why it is here rather than in a host:**

| Module | What it is | Who uses it |
|---|---|---|
| `local_game.rs` | `LocalGame` — the steppable driver. `advance()` runs bot seats autonomously and stops when a human seat must act; `submit(seq, choice)` answers. Idempotent while a decision is outstanding. | play server, `GameDriver`, tests |
| `setup.rs` | `build_initial_state` / `redeal` — deterministic seeded pregame assembly, decks admitted through the real `validate_deck` (Architecture Invariant 9) | play server, TUI |
| `params.rs` | `ActionParams` + `action_to_command_with_params` — the **single** `LegalAction` → `Command` mapping table in this crate | play server, `RandomBot` |
| `legal_actions.rs` | `StubProvider`, unchanged in role | everything |

**`GameDriver::run_game` is now a thin wrapper over `LocalGame` with no human seats**,
rather than a second copy of the same loop. That is the structural point: the fuzzer and
the interactive clients cannot drift, because there is one loop.

**Two action classes are offered to human seats only** and are appended by
`local_game::human_only_actions`, *never* by `StubProvider`:

* `LegalAction::Concede` (CR 104.3a) — a bot must not auto-concede;
* `LegalAction::OrderBlockers` (CR 509.2) — optional (the engine falls back to
  `OrdMap` order), so a bot that never issues it plays a legal game.

Keeping both out of the provider is what lets a change like this leave **every recorded
fuzz seed reproducing the same game**: `RandomBot` picks an index into the provider's
list, so appending to that list would re-roll every subsequent draw.

**`HeuristicBot` carries a per-turn preference cap** (`RepeatKey`) on repeated
activations and on re-declaring a combat. Both were found by the M11-local S8 scripted
playthrough halting on `max_commands`: a free repeatable ability (`lightning_greaves`'
Equip `{0}`, which resolves as a no-op) and re-declaring the same combat (neither the
provider nor `combat.rs::handle_declare_attackers` gates "already declared", CR 508.1 —
seed `OOS-M11-9`). CR 104.4b loop detection catches neither, because both are *optional*
actions. `RandomBot` is unaffected: it picks uniformly and passes often enough to
advance.

**SIM-1 (2026-08-02) made the `OOS-M11-9` loop reachable from a second client, and
mitigated it the same way.** Once the provider offers a command-zone cast, commanders
actually reach the battlefield — and commanders are disproportionately vigilant. The
scripted human policy in `local_game_playthrough.rs` had no repeat cap, so seed 1 halted
`InfiniteLoop` at turn 17 with exactly 20,000 commands, **19,351 of them
`DeclareAttackers` in that single turn** (seed 1's human commander is `Samut, Voice of
Dissent`, which has Vigilance). That policy now carries the same per-combat cap the bot
does, reset on the combat-entry edge rather than the turn number — `MR-M11-09` found that
exact regression in `HeuristicBot`, where a turn-keyed tally silently disabled attacks in
every CR 506.5 extra combat. **The mitigation stayed client-side both times, deliberately:
putting it in `StubProvider` would change the provider's action list and re-roll every
recorded fuzz seed.** The engine-side fix — an "already declared this combat" guard in
`combat.rs::handle_declare_attackers` — remains `OOS-M11-9` and is still open.

**The play server itself** is `tools/play-server` (axum, port 3040, 6 routes, Svelte 5
frontend). It is the only crate in the M11-local stack with async or IO; nothing below
`api.rs` references tokio. See `tools/play-server/README.md`.

**Known simulator-side gaps, all recorded rather than fixed here**: `StubProvider`
enumerates no Adventure, alt-cost, or Convoke/Improvise/Delve casts (plan §8 R4).

**`mana_solver` is pool-aware and layer-resolved as of SIM-2** (2026-08-02, `scutemob-176`,
playtest triage F3/F4). It previously (a) ignored the mana pool entirely, with
`LocalGame::auto_tap_commands_for` compensating by an all-or-nothing pool check that solved
for the **whole printed cost** whenever the pool did not fully cover it; (b) counted one
mana per SOURCE tapped rather than per mana produced, so Sol Ring was one mana — which
over-tapped in one direction and, worse, made `can_afford` refuse to offer a `{2}` spell a
Sol Ring pays for; and (c) read `obj.characteristics.mana_abilities` raw, so a **face-down**
permanent's stripped mana abilities (CR 707.2) were planned and the engine refused the tap.
`solve_mana_payment_with_pool` now solves the residual, production is credited in mana, and
sources are gathered through `calculate_characteristics` — the same function the provider's
own `TapForMana` loop and `handle_tap_for_mana` use, so an offer and a payment plan cannot
disagree. What remains of **`OOS-M11-2`** is cost *modifiers* (no Thalia-style increase, no
reduction) and CR 106.12 restricted mana; its pool, commander-tax (SIM-1) and layer halves
are all closed. Residual gaps have their own seeds: `OOS-SIM2-1` (the solve is greedy, so an
under-offer is still possible on a board where source assignment interacts), `OOS-SIM2-2`
(abilities with their own mana component are never planned), `OOS-SIM2-4` (SR-36 scaled
production and CR 106.6a replacements are under-counted).

**`HeuristicBot` no longer taps out on an empty upkeep** (SIM-2, playtest triage F5):
`TapForMana` scores **0**, below `PassPriority`'s 1. Every action that can consume mana
already outscored the old 5, so the demotion only removes the case where a tap was the sole
alternative to passing — and `LocalGame` auto-taps a bot's casts, so nothing the bot can pay
for depended on pre-floating mana. A bot still cannot pay an activated ability's mana cost
(`OOS-SIM2-3`), which was equally true before.

**`StubProvider` enumerates command-zone casts as of SIM-1 (2026-08-02, playtest triage
F7).** It previously enumerated casts **from hand only**, so a human clicking their
commander in the browser was correctly told the server had offered nothing — the engine
has supported CR 903.8 since M6, and only the provider was blind. The new loop mirrors
three engine gates rather than inventing policy: the zone must be `Command(player)`, the
object's `CardId` must be in `commander_ids` (CR 408.1 means the zone also holds emblems
and CR 903.9a/b returns — **the zone is not the filter**), and CR 101.2's non-hand cast
restriction must not apply. That last one was newly reachable and is worth its own note:
`casting.rs` rejects *any* non-hand cast while an opponent controls a **Drannith
Magistrate**, and `is_cast_restricted_by_stax` deliberately does not mirror per-card zone
restrictions — harmless only while every offer was a hand cast. Affordability charges the
CR 903.8 tax through the shared `effective_cast_cost`, which all three printed-cost sites
(the offer gate, the human auto-tap, the bot auto-tap) now consume, so they cannot
disagree about what has to be paid. Still open: a hybrid/Phyrexian-pipped commander is
gated by `can_afford` rather than by a payment plan, because `LegalAction::CastSpell` has
no plan channel (`OOS-SIM1-1`); and the TUI keeps a fourth printed-cost auto-tap and its
human path still enumerates the hand only (`OOS-SIM1-2`).

SIM-1 also recorded a third open item here — that `mtg-fuzzer` built command-zone objects
without `builder.player_commander`, so its games were not Commander games at all, the new
offer was unreachable there, and that unreachability was *why* no recorded fuzz seed moved
(`OOS-SIM1-4`). **That is closed by PB-DX22 (`scutemob-196`)**: `fuzz_setup::place_registered_deck`
places and registers as one operation, and `build_fuzz_state` also installs the CR 903.9b
replacements. The offer fires in fuzzer games now, so **the reason no seed moved is gone
and the seeds did move** — the one-time cost SIM-1 predicted, paid.

The **after** side is `mtg-fuzzer --games 20 --seed 1 --max-turns 200 --threads 1
--profile fuzz`, whose summary now prints these numbers itself (`print_mechanics_summary`,
added by the PB-DX22 fix cycle): `CommanderCastFromCommandZone` **36 across 16 of 20
games**, **13** CR 903.9a returns, non-empty `commander_damage_received` in **16 of 20**
games with a largest single total of **31** — past CR 903.10a's 21 threshold, so the
loss condition itself is now reachable under automated exercise. Raw run committed at
`memory/primitives/pb-dx22-measurement-after-fixcycle.txt`.

The **before** side is a different instrument with a different denominator, and saying so
is the point (review Finding 3): it is a scratch harness over **5** games (~56,800
commands) recorded at `memory/primitives/pb-dx22-measurement-head.txt`, in which every one
of those counts was **0**. A "0 → 36" written under one command name hides that; the fuzzer
could not print either number until the fix cycle, and cannot print the pre-fix ones at all,
because the build path they measured no longer exists.

The re-roll did **not** reach the play server (78/0 green; `session.rs` builds through
`setup.rs`) or `crates/simulator/tests/local_game.rs` (24 passed — the 23 that predate
this batch all unchanged, plus its own new CR 903.9b probe).

---

## Phase 3: Interactive TUI (NOW)

**Goal**: `mtg-tui play` subcommand — human plays as one player against 1-5 bots (up to 6 total).

### Layout: Focused Player + Sidebar

```
+-----------------------------------------------+
| Turn 5 | P1's Turn | Main 1 | Priority: You   |  <- Phase bar (always visible)
+-----------------------------------------------+
|              STACK (if non-empty)              |  <- Stack (always visible when populated)
|  [1] Counterspell targeting Wrath of God       |
+------------------------------------+----------+
|                                    | Players  |
|  BATTLEFIELD                       |----------|
|  [Sol Ring] [Forest] [Forest]      | >P1  40  |  <- You (highlighted)
|  [Llanowar Elves 1/1] [Bear 2/2]  |  P2  38  |
|                                    |  3 perms |
|  HAND                              |----------|
|  [1] Cultivate  [2] Counterspell   |  P3  40  |
|  [3] Forest     [4] Wrath of God   |  0 perms |
|                                    |----------|
|  Mana: GGcc  Life: 40  Lands: 1   |  P4  35  |
+------------------------------------+  5 perms |
| ACTIONS                            |----------|
| [p]ass  [1-4]cast  [a]ttack       |  P5  22  |
| [t]ap mana  [Tab]switch player     |  1 perm  |
+------------------------------------+----------+
| EVENT LOG                                     |  <- Scrollable
| > You cast Llanowar Elves                     |
| > P2 passes priority                          |
+-----------------------------------------------+
```

### Navigation Principles

| Key | Action |
|-----|--------|
| **1-9** | Select card from hand or battlefield by position |
| **Tab / Shift+Tab** | Cycle focused player (view any player's full board) |
| **p** | Pass priority |
| **c** | Cast selected card (opens mana payment if needed) |
| **l** | Play selected land |
| **a** | Enter attacker declaration mode |
| **b** | Enter blocker declaration mode |
| **t** | Tap a permanent for mana |
| **Space** | Expand card detail popup (oracle text, types, abilities) |
| **Enter** | Confirm current selection |
| **Esc** | Cancel / close popup / exit sub-mode |
| **Arrow keys** | Navigate within zones (battlefield cards, hand cards) |
| **q** | Quit game |

### Input Modes (modal)

1. **Normal**: View board, select cards, invoke actions
2. **Mana payment**: Select sources to tap, running total vs cost
3. **Attacker declaration**: Toggle creatures to attack, select targets per attacker
4. **Blocker declaration**: Assign blockers to attackers
5. **Target selection**: Pick targets for a spell/ability
6. **Card detail**: Popup overlay showing full card text

### Game Loop

```rust
// Alternates between bot and human turns
loop {
    terminal.draw(|f| render(f, &app))?;

    if app.game_over() { break; }

    let acting_player = app.acting_player();
    if acting_player == app.human_player {
        // Wait for keyboard input → translate to Command
        match poll_event()? {
            Key(key) => handle_key(&mut app, key),
            _ => {}
        }
    } else {
        // Bot turn: compute immediately, optional delay for readability
        let legal = app.provider.legal_actions(&app.state, acting_player);
        let cmd = app.bots[&acting_player].choose_action(&app.state, acting_player, &legal);
        app.execute_command(cmd)?;
        sleep(Duration::from_millis(app.bot_delay)); // configurable, e.g., 200ms
    }
}
```

### Reuse from Existing TUI

| Component | Source | How to reuse |
|-----------|--------|--------------|
| Event loop pattern | `tools/tui/src/dashboard/mod.rs` | Same poll + draw loop |
| Theme colors/symbols | `tools/tui/src/theme.rs` | Import directly |
| Progress bar widget | `tools/tui/src/widgets/progress_bar.rs` | Life total bars |
| Status badge widget | `tools/tui/src/widgets/status_badge.rs` | Phase/status indicators |
| Subcommand dispatch | `tools/tui/src/main.rs` (clap) | Add `Play` variant |

### Files

| File | Action | Touches engine? |
|------|--------|-----------------|
| `tools/tui/src/main.rs` | MODIFY — add `Play` subcommand | No |
| `tools/tui/src/play/mod.rs` | NEW — entry point, main loop | No |
| `tools/tui/src/play/app.rs` | NEW — app state, game state, input mode | No |
| `tools/tui/src/play/render.rs` | NEW — main render dispatch | No |
| `tools/tui/src/play/input.rs` | NEW — keyboard handling, mode transitions | No |
| `tools/tui/src/play/panels/phase_bar.rs` | NEW | No |
| `tools/tui/src/play/panels/stack_view.rs` | NEW | No |
| `tools/tui/src/play/panels/battlefield.rs` | NEW | No |
| `tools/tui/src/play/panels/hand_view.rs` | NEW | No |
| `tools/tui/src/play/panels/sidebar.rs` | NEW | No |
| `tools/tui/src/play/panels/action_menu.rs` | NEW | No |
| `tools/tui/src/play/panels/card_detail.rs` | NEW | No |
| `tools/tui/src/play/panels/event_log.rs` | NEW | No |
| `tools/tui/src/play/panels/combat_view.rs` | NEW | No |
| `tools/tui/Cargo.toml` | MODIFY — add mtg-engine + mtg-simulator deps | No |

---

## Phase 4: Full Legal Actions in Engine (LATER — after abilities complete)

**Goal**: Replace `StubProvider` with a comprehensive `legal_actions()` in the engine crate.

### What changes

1. Create `crates/engine/src/rules/legal_actions.rs` — full implementation (~600-900 lines)
2. Move `LegalAction` enum from simulator to engine (re-export in simulator for compat)
3. Add `pub mod legal_actions` to `crates/engine/src/rules/mod.rs`
4. Create `FullProvider` in simulator that delegates to `engine::legal_actions()`
5. All existing simulator/TUI code works unchanged — just swap the provider

### Implementation covers everything the stub misses

- Flashback, escape, foretell, unearth casting from graveyard/exile
- Activated abilities on permanents
- Cycling, crew, companion
- Dredge/miracle choice responses
- Split second blocking
- Protection-based targeting restrictions
- Alternative cost enumeration (evoke, bestow, convoke, etc.)
- Full mana affordability analysis

### Testing

- Round-trip: every `LegalAction` → `Command` → `process_command` must succeed
- The fuzzer becomes the primary validation tool — run thousands of games with FullProvider
- Any action the stub allowed but Full rejects = stub was too permissive (log, don't crash)
- Any action Full allows but stub missed = the stub was too conservative (expected)

---

## Phase 5: StrategyBot — Informed by Articles (LATER)

**Goal**: A configurable bot whose weights come from MTG strategy knowledge.

### Design

```rust
pub struct StrategyBot {
    weights: StrategyConfig,  // loaded from TOML
    rng: StdRng,
}

// strategy.toml
[scoring]
play_land = 100
cast_removal = 80
cast_creature = 60
attack_tapped_opponent = 40
hold_up_counterspell_mana = 35
pass_priority = 1

[threat_assessment]
target_highest_board = true
target_lowest_life = false

[resource_management]
max_creatures_before_wipe_fear = 3
hold_mana_for_instant = true
```

Strategy articles → translate concepts to scoring weights → save as TOML profiles → swap profiles to test different strategies.

---

## Phase 6: Deck Pipeline (LATER — after full ability coverage)

**Goal**: Submit deck list of card names, auto-generate CardDefinitions for missing cards.

### Flow

1. Parse deck list (plain text: `1 Sol Ring`, `1 Command Tower`, etc.)
2. Check each card against `all_cards()` registry
3. Missing cards: query `cards.sqlite` for oracle text, types, mana cost
4. Auto-generate CardDefinition with correct stats + keyword extraction
5. Validate Commander legality (color identity, singleton, 99+commander)
6. Output: complete CardRegistry

---

## Dependency Graph

```
Phase 1 (Simulator Crate) ──> Phase 2 (Fuzzer CLI)
         │
         └──> Phase 3 (Interactive TUI)

--- ability work continues independently on master ---

Phase 4 (Full Legal Actions) ──> swap StubProvider for FullProvider
Phase 5 (StrategyBot) ── independent
Phase 6 (Deck Pipeline) ── independent
```

**Collision risk**: Phases 1-3 create only NEW files in NEW crates. Zero engine modifications. No merge conflicts with ability work.

## Verification Plan

| Phase | How to verify |
|-------|---------------|
| Phase 1 | `cargo test -p mtg-simulator` — RandomBot completes a 4-player game |
| Phase 2 | `cargo run --bin mtg-fuzzer -- --games 100 --seed 42` — runs to completion |
| Phase 3 | `cargo run --bin mtg-tui -- play --players 4` — play a few turns manually |
| Phase 4 | Fuzzer with FullProvider: `--games 1000` — 0 violations |
| Phase 5 | StrategyBot beats RandomBot in win-rate over 100 games |
| Phase 6 | Submit a real Commander deck list, generate all definitions, run a game |

# MTG Engine — Game Simulator & Fuzzer

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

1. **Zone integrity**: Every object in exactly one zone
2. **ID uniqueness**: No duplicate ObjectIds
3. **Mana non-negative**: All mana pool values >= 0
4. **Stack consistency**: stack_objects matches objects in Stack zone
5. **Player consistency**: Active player and priority holder are alive
6. **Turn order**: All players in turn_order, active player present
7. **Object-zone agreement**: Object's zone field matches containing zone
8. **Attachment validity**: attached_to references existing battlefield objects
9. **Game progression**: Turn number never decreases
10. **Legal action soundness**: Actions from provider don't get rejected by `process_command()`
11. **SBA idempotency**: After SBAs, running again produces no events
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

# MTG Commander Rules Engine: Architecture & Testing Strategy

## Purpose of This Document

This document defines the architectural decisions, system design, and testing strategy for an MTG rules engine targeting Commander format play. It serves as the primary technical reference for Claude Code during development. Every decision here is scoped to support the Development Roadmap (see `mtg-engine-roadmap.md`) and should be read alongside it.

The engine's goal: a correct, networked, Commander-capable MTG rules engine built in Rust with a Tauri-based desktop client, capable of enforcing the comprehensive rules with full stack visualization and player interaction.

---

## 1. System Architecture Overview

The system is composed of four major subsystems:

```
┌─────────────────────────────────────────────────────┐
│                   Tauri Desktop App                  │
│  ┌───────────────────────┐  ┌─────────────────────┐ │
│  │     Web UI (Svelte)   │  │   Tauri IPC Bridge   │ │
│  │  - Battlefield render │  │  - Command dispatch  │ │
│  │  - Stack visualization│  │  - State sync        │ │
│  │  - Card browser       │  │  - Asset management  │ │
│  │  - Targeting UI       │  │                      │ │
│  └───────────────────────┘  └──────────┬──────────┘ │
└────────────────────────────────────────┼────────────┘
                                         │
┌────────────────────────────────────────┼────────────┐
│              Rust Engine Core          │            │
│  ┌──────────────┐  ┌─────────────┐  ┌─┴──────────┐ │
│  │  Game State   │  │ Rules Engine│  │  Net Layer  │ │
│  │  (immutable)  │  │  - Turn FSM │  │  - Commands │ │
│  │  - Zones      │  │  - Stack    │  │  - Events   │ │
│  │  - Objects    │  │  - Layers   │  │  - Sync     │ │
│  │  - Effects    │  │  - SBAs     │  │  - Lobby    │ │
│  └──────────────┘  │  - Priority │  └────────────┘ │
│                     └─────────────┘                  │
│  ┌──────────────┐  ┌─────────────────────────────┐  │
│  │   Card DB     │  │  Card Definition Runtime    │  │
│  │  (SQLite)     │  │  - Ability resolution       │  │
│  │  - Oracle     │  │  - Keyword mechanics        │  │
│  │  - Rulings    │  │  - Replacement effects      │  │
│  │  - Metadata   │  │  - Triggered ability queue   │  │
│  └──────────────┘  └─────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### Why This Separation Matters

The engine core is a pure Rust library crate with **zero UI dependencies**. This enables:

- **Headless testing**: The entire engine can be tested without any UI, Tauri, or network stack. Tests instantiate the engine directly and issue commands programmatically.
- **Deterministic replay**: Game sessions are sequences of commands. Any game can be replayed deterministically for debugging, testing, or spectating.
- **Network flexibility**: The same engine core runs on a host machine (authoritative) and can run on clients for prediction/validation. The network layer wraps the engine; it doesn't penetrate it.
- **Future portability**: The engine could later be compiled to WASM for a web client, or wrapped in a C FFI for other frontends, without modification.

---

## 2. Game State Model

### 2.1 Immutable State with Structural Sharing

Game state is the single most important data structure in the engine. Every rules decision is a function of game state. The state model must support:

- **Snapshots**: Cheap copies of entire game state for undo, replay, and "what if" analysis.
- **Diffing**: The UI needs to know what changed between states to animate transitions.
- **Determinism**: Given the same state and the same command, the engine must always produce the same result.

**Decision: Immutable state using `im-rs` persistent data structures.**

Rust's `im` crate provides hash maps, vectors, and sets with structural sharing — cloning a large map is O(1) because unchanged subtrees are shared. This gives us cheap snapshots without the complexity of a manual undo log.

```
GameState
├── turn: TurnState (phase, step, active_player, priority_holder)
├── players: im::HashMap<PlayerId, PlayerState>
│   ├── life_total: i32
│   ├── mana_pool: ManaPool
│   ├── commander_tax: HashMap<CardId, u32>
│   ├── commander_damage_received: HashMap<PlayerId, u32>
│   ├── land_plays_remaining: u32
│   ├── has_drawn_for_turn: bool
│   └── ...
├── zones: im::HashMap<ZoneId, Zone>
│   ├── library (per player, ordered)
│   ├── hand (per player)
│   ├── battlefield (shared)
│   ├── graveyard (per player, ordered)
│   ├── stack (shared, ordered)
│   ├── exile (shared)
│   └── command (per player)
├── objects: im::HashMap<ObjectId, GameObject>
│   ├── characteristics (name, mana_cost, types, subtypes, supertypes, power, toughness, etc.)
│   ├── controller: PlayerId
│   ├── owner: PlayerId
│   ├── zone: ZoneId
│   ├── status (tapped, flipped, face_down, phased_out)
│   ├── counters: HashMap<CounterType, u32>
│   ├── attachments: Vec<ObjectId>
│   ├── attached_to: Option<ObjectId>
│   └── abilities: Vec<AbilityInstance>
├── continuous_effects: im::Vector<ContinuousEffect>
│   ├── source: ObjectId
│   ├── timestamp: Timestamp
│   ├── layer: Layer
│   ├── duration: Duration
│   ├── modification: Modification
│   └── affected: AffectedFilter
├── delayed_triggers: im::Vector<DelayedTrigger>
├── replacement_effects: im::Vector<ReplacementEffect>
├── pending_triggers: im::Vector<TriggeredAbility>
├── stack_objects: im::Vector<StackObject>
├── combat: Option<CombatState>
├── turn_number: u32
├── timestamp_counter: u64
└── history: Vec<GameEvent>  // append-only log for triggers that look back
```

### 2.2 Object Identity

MTG rule 400.7 states that an object that changes zones becomes a new object with no memory of its previous existence (with specific exceptions). This means `ObjectId` is not a card identifier — it's an instance identifier. When a creature dies and is reanimated, it gets a new `ObjectId`. The underlying card data is referenced separately via a `CardId` that maps into the SQLite database.

This distinction is critical for:
- Auras and equipment that "fall off" when their target changes zones
- "When this creature dies" triggers that must track the object that died, not the new object in the graveyard
- Commander identity tracking (the physical card persists across zone changes, but the game object doesn't)

### 2.3 Why Immutability Matters for Testing

Every test can construct a precise game state, apply an action, and assert on the resulting state without worrying about mutation side effects. Tests are isolated by construction. No setup/teardown, no shared mutable fixtures.

```rust
// Conceptual test structure
let state = GameStateBuilder::new()
    .with_player(player_a, |p| p.life(40).hand(vec![card!("Lightning Bolt")]))
    .with_player(player_b, |p| p.life(40))
    .build();

let command = Command::CastSpell {
    player: player_a,
    card: find_card(&state, "Lightning Bolt"),
    targets: vec![Target::Player(player_b)],
};

let new_state = engine.process(state, command).unwrap();
assert_eq!(new_state.player(player_b).life_total, 37);
```

---

## 3. Rules Engine Architecture

### 3.1 Turn Structure as a Finite State Machine

The turn structure (CR 500-514) is modeled as a state machine with well-defined transitions:

```
Beginning Phase
├── Untap Step (no priority)
├── Upkeep Step (priority)
└── Draw Step (priority)

Main Phase 1 (priority)

Combat Phase
├── Beginning of Combat Step (priority)
├── Declare Attackers Step (priority)
├── Declare Blockers Step (priority)
├── Combat Damage Step (priority)
│   └── (First Strike Damage Step if applicable)
└── End of Combat Step (priority)

Main Phase 2 (priority)

Ending Phase
├── End Step (priority)
└── Cleanup Step (normally no priority)
```

Each step follows the same pattern when priority exists:
1. Turn-based actions for that step occur
2. Triggered abilities are put on the stack
3. Active player receives priority
4. Priority passing loop (see 3.2)
5. When all players pass in succession with an empty stack, move to next step

The FSM is explicit — an enum of states with a transition function. This makes it trivial to test: assert that from state X with input Y, the FSM moves to state Z.

**Commander-specific additions:**
- The command zone is a zone that persists across games
- Commander tax tracking per-commander per-player
- The "commander dies/exiled" replacement effect choice
- Turn order in multiplayer (left of active player)

### 3.2 Priority System

Priority (CR 117) governs when players may take actions. In Commander (multiplayer), this is more complex than 1v1:

1. Active player receives priority
2. That player may take an action or pass
3. If they take an action, SBAs are checked, triggers go on stack, active player gets priority again
4. If they pass, the next player in turn order receives priority
5. If all players pass in succession without any actions, the top stack item resolves (or the step/phase ends if stack is empty)

**Implementation: Priority is a sub-state machine within each step.**

```rust
enum PriorityState {
    // A player has priority and must decide
    PlayerHasPriority { player: PlayerId },
    // Waiting for SBA check after an action
    CheckingStateBasedActions,
    // Putting triggered abilities on the stack (active player chooses order)
    OrderingTriggers { player: PlayerId, triggers: Vec<TriggeredAbility> },
    // All players passed — resolve top of stack or advance
    AllPassed,
}
```

### 3.3 The Stack

The stack (CR 405) is an ordered zone where spells and abilities wait to resolve. Key properties:

- LIFO resolution order
- Each object on the stack has: source, controller, targets (if any), modes (if any), and any choices made on casting/activation
- Players can respond to each stack addition before it resolves
- The stack is shared across all players

**For the UI**: Each stack object references its source card (for display) and its targets (for drawing targeting arrows). The stack visualization is a direct rendering of the `stack_objects` vector.

### 3.4 State-Based Actions (SBAs)

SBAs (CR 704) are checked every time a player would receive priority. They are a fixed-point computation — you keep applying them until none trigger:

```rust
fn check_state_based_actions(state: &mut GameState) -> Vec<GameEvent> {
    let mut all_events = vec![];
    loop {
        let events = check_sbas_once(state);
        if events.is_empty() {
            break;
        }
        apply_events(state, &events);
        all_events.extend(events);
    }
    all_events
}
```

SBAs include: creature with 0 or less toughness dies, player at 0 or less life loses, legendary rule, planeswalker uniqueness rule, unattached auras/equipment, 21+ commander damage causes loss, and many more.

**Testing priority**: SBAs are heavily tested because they interact with everything. Each SBA is a unit test, and combinations of SBAs (e.g., a creature dying from SBA triggers another SBA) are integration tests.

### 3.5 The Layer System (CR 613)

This is the most complex subsystem. Continuous effects modify game objects, and they apply in a strict order:

| Layer | What It Modifies |
|-------|-----------------|
| 1     | Copy effects |
| 2     | Control-changing effects |
| 3     | Text-changing effects |
| 4     | Type-changing effects |
| 5     | Color-changing effects |
| 6     | Ability-adding/removing effects |
| 7a    | P/T from characteristic-defining abilities |
| 7b    | P/T setting effects |
| 7c    | P/T modifications (counters, static effects) |
| 7d    | P/T switching |

Within a layer, effects apply in **timestamp order** unless there's a **dependency** (CR 613.8). Dependencies override timestamp when one effect could change what another affects or what it does.

**Implementation strategy:**

```rust
fn calculate_characteristics(state: &GameState, object_id: ObjectId) -> Characteristics {
    let base = get_copiable_values(state, object_id); // Layer 1
    let effects = gather_applicable_effects(state, object_id);
    let sorted = sort_by_layer_and_timestamp(effects);
    let resolved = resolve_dependencies(sorted); // CR 613.8

    let mut chars = base;
    for effect in resolved {
        chars = apply_effect(chars, effect);
    }
    chars
}
```

The dependency resolver is the hardest part. Two effects have a dependency if applying one in a different order would change the other's result. This requires detecting circular dependencies (CR 613.8k — apply in timestamp order if circular) and is the primary source of rules engine bugs in existing implementations.

**Testing strategy for the layer system**: A dedicated test suite of ~100+ known corner cases, sourced from:
- CR examples (embedded in the rules text itself)
- Judge blog posts and L3+ rulings
- Known "gotcha" interactions (Humility + Opalescence, Blood Moon + Urborg, Yixlid Jailer + Anger, etc.)
- Existing engine behavior (Forge, XMage) as reference oracles

### 3.6 Replacement Effects

Replacement effects (CR 614) modify events as they happen rather than triggering after. They interact with the layer system and with each other in specific ways:

- If multiple replacement effects could apply to the same event, the affected player (or controller of the affected object) chooses the order
- A replacement effect can only apply to a given event once (CR 614.5) — this prevents infinite loops
- Self-replacement effects apply before other replacement effects

These are critical for Commander because the "commander goes to command zone instead of graveyard/exile" is a replacement effect.

### 3.7 Card Definition Runtime

Cards are not hard-coded. Each card's behavior is defined by a structured representation loaded at runtime:

```rust
struct CardDefinition {
    card_id: CardId,
    name: String,
    mana_cost: Option<ManaCost>,
    types: TypeLine,
    oracle_text: String,  // for display
    abilities: Vec<AbilityDefinition>,
}

enum AbilityDefinition {
    Activated {
        cost: Cost,
        effect: Effect,
        timing_restriction: Option<TimingRestriction>,
    },
    Triggered {
        trigger_condition: TriggerCondition,
        effect: Effect,
        intervening_if: Option<Condition>,
    },
    Static {
        continuous_effect: ContinuousEffectDef,
    },
    Keyword(KeywordAbility),
    Spell {
        effect: Effect,
        targets: Vec<TargetRequirement>,
        modes: Option<ModeSelection>,
    },
}
```

**The Effect type** is a recursive enum of primitives:
- `DealDamage`, `GainLife`, `LoseLife`
- `DrawCards`, `DiscardCards`, `MillCards`
- `CreateToken`, `DestroyPermanent`, `ExileObject`
- `AddMana`, `AddCounter`, `RemoveCounter`
- `MoveZone`, `TapPermanent`, `UntapPermanent`
- `SearchLibrary`, `Shuffle`
- `ApplyContinuousEffect`
- `Conditional { condition, if_true, if_false }`
- `ForEach { over, effect }`
- `Choose { choices }`
- `Sequence(Vec<Effect>)`

This is the engine's internal DSL. The card definition pipeline (see Section 5) translates oracle text into this representation.

---

## 4. Networking Architecture

### 4.1 Command/Event Model

All player actions are serialized as **commands**. All state changes are serialized as **events**. This is the foundation of both networking and replay.

```
Player Input → Command → Engine validates → Events → State transition
                                                   → Broadcast to all clients
```

**Commands** (player intent):
```rust
enum Command {
    CastSpell { card: ObjectId, targets: Vec<Target>, modes: Vec<usize> },
    ActivateAbility { source: ObjectId, ability_index: usize, targets: Vec<Target> },
    DeclareAttackers { attackers: Vec<(ObjectId, AttackTarget)> },
    DeclareBlockers { blockers: Vec<(ObjectId, Vec<ObjectId>)> },
    AssignDamage { assignments: Vec<DamageAssignment> },
    PassPriority,
    Concede,
    ChooseOption { choice: usize },  // for modal spells, replacement effects, etc.
    OrderTriggers { order: Vec<usize> },
    MulliganDecision { keep: bool },
    // Commander-specific
    MoveCommanderToCommandZone { commander: ObjectId },
}
```

**Events** (what happened):
```rust
enum GameEvent {
    ObjectMovedZone { object: ObjectId, from: ZoneId, to: ZoneId },
    LifeTotalChanged { player: PlayerId, old: i32, new: i32 },
    DamageDealt { source: ObjectId, target: Target, amount: u32 },
    CounterAdded { object: ObjectId, counter_type: CounterType, amount: u32 },
    PlayerLost { player: PlayerId, reason: LossReason },
    GameOver { winner: Option<PlayerId> },
    PriorityChanged { player: PlayerId },
    PhaseChanged { phase: Phase },
    StepChanged { step: Step },
    TurnChanged { active_player: PlayerId },
    // ... many more
}
```

### 4.2 Network Model

> **Active M10 plan**: Centralized WebSocket server. One server instance (runnable on a
> ~$5-10/mo VPS) runs the engine authoritatively, filters hidden information per player,
> and broadcasts events. Simpler than P2P for a trusted playgroup; no bad-internet bottleneck;
> trivial reconnection. See `docs/mtg-engine-roadmap.md` M10 for deliverables.
>
> **Deferred upgrade path**: P2P distributed verification + Mental Poker is fully designed
> in `docs/mtg-engine-network-security.md` for future trustless play. The engine doesn't
> need to change for either model — only the network layer differs.

#### Active: Centralized Server Model

One server instance runs the canonical engine. All players connect via WebSocket, send commands to the server, and receive filtered events back. The server is the single source of truth and handles hidden information by sending private events only to the relevant player.

```
Host Machine                     Client Machines
┌──────────────┐                ┌──────────────┐
│  Engine Core │ ←── commands ──│  UI + Input   │
│  (canonical) │ ─── events ──→ │  State Mirror │
│  Validation  │                │  Prediction   │
└──────────────┘                └──────────────┘
```

### 4.3 Hidden Information & Partial State

The centralized server knows all game state. It filters events before broadcasting — private events (card draws, scry peeks, hand reveals) are sent only to the relevant player; all others receive a redacted version. The server exposes `GameEvent::private_to() -> Option<PlayerId>` to determine routing.

Each client receives a **view** of the game state — a projection that excludes information they shouldn't have:

```rust
fn project_state_for_player(state: &GameState, viewer: PlayerId) -> ClientGameState {
    // Full info: battlefield, stack, graveyards, exile, command zone
    // Partial info: opponent hand sizes (but not contents)
    // Hidden: opponent hands, all libraries (order), face-down cards they don't control
    // Special: top of library if an effect lets them look
}
```

### 4.4 Network Protocol

Communication uses WebSocket for real-time bidirectional messaging. The protocol is message-based with these categories:

- **Lobby**: Create game, join game, set game parameters (format, house rules), player ready
- **Game commands**: Player actions (see Command enum above)
- **Game events**: State changes broadcast from host
- **State sync**: Full state snapshot on connect/reconnect
- **Heartbeat/ping**: Connection health

Messages are serialized with `serde` — likely MessagePack for compactness, with JSON as a debug option.

### 4.5 Reconnection & Resilience

**Centralized server**: If a client disconnects, the game pauses and other players are notified. On reconnect, the client receives a full public state dump plus their own private state (hand contents, known library cards). If the server goes down, the game is interrupted — host on a reliable VPS to minimise this.

---

## 5. Card Data Architecture

### 5.1 Data Sources

| Source | Format | Purpose | Storage |
|--------|--------|---------|---------|
| Scryfall Bulk Data | JSON | Oracle text, types, costs, legality, rulings | SQLite (canonical card DB) |
| Comprehensive Rules | Text | Rules spec for engine logic | RAG index (development-time) |
| Scryfall Rulings | JSON | Per-card rulings and edge cases | SQLite + RAG index |
| Existing engines (Forge, XMage) | Source code | Reference implementations for edge cases | RAG index (development-time) |
| Card images | PNG/JPG | UI rendering | Local file cache (downloaded per-user) |

### 5.2 SQLite Schema (Simplified)

```sql
CREATE TABLE cards (
    id TEXT PRIMARY KEY,              -- scryfall ID
    oracle_id TEXT NOT NULL,          -- groups printings
    name TEXT NOT NULL,
    mana_cost TEXT,
    cmc REAL NOT NULL,
    type_line TEXT NOT NULL,
    oracle_text TEXT,
    power TEXT,
    toughness TEXT,
    loyalty TEXT,
    colors TEXT,                      -- JSON array
    color_identity TEXT,              -- JSON array (critical for Commander)
    keywords TEXT,                    -- JSON array
    legalities TEXT,                  -- JSON object
    set_code TEXT NOT NULL,
    collector_number TEXT NOT NULL
);

CREATE TABLE rulings (
    id INTEGER PRIMARY KEY,
    oracle_id TEXT NOT NULL,
    published_at TEXT NOT NULL,
    comment TEXT NOT NULL,
    FOREIGN KEY (oracle_id) REFERENCES cards(oracle_id)
);

CREATE TABLE card_faces (
    id INTEGER PRIMARY KEY,
    card_id TEXT NOT NULL,
    face_index INTEGER NOT NULL,
    name TEXT NOT NULL,
    mana_cost TEXT,
    type_line TEXT NOT NULL,
    oracle_text TEXT,
    power TEXT,
    toughness TEXT,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);

-- Engine-specific: parsed card definitions
CREATE TABLE card_definitions (
    oracle_id TEXT PRIMARY KEY,
    definition_json TEXT NOT NULL,    -- serialized AbilityDefinition[]
    definition_version INTEGER NOT NULL,
    validated BOOLEAN DEFAULT FALSE,
    validation_notes TEXT,
    FOREIGN KEY (oracle_id) REFERENCES cards(oracle_id)
);
```

### 5.3 Card Definition Pipeline

The pipeline for converting oracle text into engine-consumable definitions:

```
Oracle Text + Rulings (from SQLite)
         │
         ▼
Claude Code + CR RAG (development-time)
         │
         ▼
Structured CardDefinition (Rust struct / JSON)
         │
         ▼
Automated Test Harness (runs known interactions)
         │
    ┌────┴────┐
    ▼         ▼
  PASS      FAIL → Human review → Correction → Feed back as examples
    │
    ▼
card_definitions table (validated = true)
```

Cards are prioritized by Commander format staples first, then by frequency of play (EDHREC data), then by set release order.

### 5.4 Card Asset Management

Card images are NOT bundled with the app. On first run (or when a new set releases), the app downloads card images from Scryfall's API and caches them locally:

```
~/.mtg-engine/
├── card_db.sqlite          (bundled, updated with app releases)
├── card_definitions.json   (bundled, updated with app releases)
└── assets/
    └── images/
        ├── large/          (for battlefield display)
        │   ├── {scryfall_id}.jpg
        │   └── ...
        └── small/          (for hand, lists)
            ├── {scryfall_id}.jpg
            └── ...
```

The app provides a "download set" UI where users select which sets to cache locally. A background download manager handles fetching, with progress indication.

---

## 6. Testing Strategy

Testing is not a phase — it's the development methodology. The engine is built test-first because MTG rules are a specification. The CR tells you what should happen; the test encodes that; the implementation makes the test pass.

### 6.1 Test Pyramid

```
                    ▲
                   ╱ ╲
                  ╱   ╲        Golden Tests
                 ╱ ~20 ╲       Full game replays
                ╱───────╲
               ╱         ╲     Integration Tests
              ╱  ~200-500  ╲   Multi-card interactions, complex scenarios
             ╱─────────────╲
            ╱               ╲   Unit Tests
           ╱   ~1000-2000    ╲  Individual rules, SBAs, layer calculations
          ╱───────────────────╲
         ╱                     ╲ Property Tests
        ╱      ~50 properties   ╲ Invariant checking via fuzzing
       ╱─────────────────────────╲
```

### 6.2 Unit Tests

Each rule section in the CR maps to a test module:

- `tests/rules/turn_structure.rs` — Phase/step transitions, turn-based actions
- `tests/rules/priority.rs` — Priority passing, multiplayer priority order
- `tests/rules/stack.rs` — Stack ordering, resolution, countering
- `tests/rules/sba.rs` — Each state-based action individually
- `tests/rules/layers.rs` — Layer system, timestamp ordering, dependencies
- `tests/rules/combat.rs` — Attackers, blockers, damage assignment, first strike, trample
- `tests/rules/replacement.rs` — Replacement effects, self-replacement, loop prevention
- `tests/rules/triggered.rs` — Trigger conditions, intervening-if, APNAP order
- `tests/rules/zones.rs` — Zone changes, object identity (400.7), last-known information
- `tests/rules/commander.rs` — Command zone, tax, commander damage, color identity

**Source material**: The CR itself contains examples for many rules. These are the first tests written. Example:

> CR 613.10 example: "Humility and Opalescence are on the battlefield. Humility makes all creatures 1/1 with no abilities. Opalescence makes each enchantment a creature with P/T equal to its mana value."

This becomes a test case asserting specific characteristics of both permanents after the layer system resolves.

### 6.3 Integration Tests

Integration tests verify multi-card interactions and complex game sequences. Each test sets up a specific board state, executes a sequence of commands, and asserts the final state.

Categories:
- **Known interactions**: Curated list of famous corner cases (see Appendix A for initial list)
- **Keyword interactions**: First strike + deathtouch, trample + deathtouch, flying + reach, protection + board wipes, etc.
- **Mechanic combos**: Cascading triggers, replacement effect chains, copy effects on the stack
- **Commander-specific**: Commander damage from copies, commander tax after zone change choices, partner commander interactions

### 6.4 Golden Tests (Full Game Replays)

A golden test records an entire game as a sequence of commands plus expected state snapshots at key points. The test replays the commands through the engine and asserts state matches at each snapshot.

Sources for golden test data:
- Hand-authored simple games (e.g., a 5-turn game with basic creatures and combat)
- Transcriptions from MTGO/Arena replays (manual initially, potentially automated later)
- Community-submitted game logs once the engine is functional

Format:
```json
{
  "name": "basic_combat_game",
  "players": [...],
  "decks": [...],
  "random_seed": 12345,
  "commands": [
    { "turn": 1, "command": { "type": "CastSpell", ... } },
    ...
  ],
  "snapshots": [
    { "after_command": 5, "assertions": { "player_b_life": 37, ... } },
    ...
  ]
}
```

### 6.5 Property-Based Tests (Fuzzing)

Using Rust's `proptest` or `quickcheck` crates, define invariants that must hold across any game state:

- **Life total conservation**: Total life lost equals total damage dealt (accounting for life gain, Commander damage, infect, etc.)
- **Object zone integrity**: Every object is in exactly one zone
- **Stack integrity**: Stack is always ordered; no orphaned stack objects
- **Turn structure validity**: The FSM never enters an invalid state
- **No panics**: Random sequences of valid commands never crash the engine
- **SBA convergence**: SBA checking always terminates (no infinite loops)
- **Layer calculation termination**: The layer system always produces a result
- **Trigger queue completeness**: No triggers are lost between SBA checks

These tests generate random game states and command sequences to find edge cases that hand-written tests miss.

### 6.6 Network Tests

- **Latency simulation**: Inject artificial delays; verify game state remains consistent
- **Packet loss/reorder**: Simulate unreliable connections; verify command ordering is maintained
- **Reconnection**: Disconnect a client, advance the game, reconnect; verify state sync
- **State divergence detection**: If a client's predicted state diverges from the host, verify correction
- **Concurrent commands**: Multiple players issuing commands simultaneously; verify serialization

### 6.7 Performance Benchmarks

Tracked as part of CI to catch regressions:

- **Layer calculation time**: Time to recalculate all characteristics with N continuous effects. Target: <1ms for 50 effects.
- **SBA check time**: Time for a full SBA fixed-point check. Target: <0.5ms for a complex board.
- **Full turn processing**: Time to process a full turn with no player interaction (for simulation). Target: <10ms.
- **State snapshot size**: Memory footprint of a game state. Target: <1MB for a complex Commander game.
- **Network message size**: Serialized size of events and state syncs. Target: <10KB for typical events, <100KB for full state sync.

### 6.8 Test Infrastructure

- **`GameStateBuilder`**: Fluent API for constructing arbitrary game states for tests. This is the most important test utility — make it expressive and well-documented.
- **`card!()` macro**: Shorthand for referencing cards by name in tests without full DB lookup.
- **`assert_board!()` macro**: Assert multiple board state conditions in a readable format.
- **Test card database**: A subset of real cards plus synthetic "test cards" designed to exercise specific rules (e.g., a creature with every keyword, an instant with every targeting mode).
- **CI pipeline**: All tests run on every commit. Performance benchmarks run nightly with regression alerts.

---

## 7. Commander-Specific Design Considerations

Commander is not an afterthought — it's the target format. These design choices are woven into the core architecture:

### 7.1 Multiplayer Priority

The priority system is designed for N players from the start. The `PriorityState` machine tracks the "pass count" — how many consecutive passes have occurred — and resets on any action. In 4-player Commander, all 4 players must pass in succession for the stack to resolve or a step to advance.

### 7.2 Commander Zones and Tax

The command zone is a first-class zone in the zone system. Commander tax is tracked per-commander per-player in `PlayerState`. The replacement effect for "commander dies/goes to exile → may move to command zone" is implemented as a standard replacement effect that presents a choice to the controller.

### 7.3 Commander Damage

Commander combat damage is tracked as a matrix: `HashMap<(PlayerId_dealer, PlayerId_receiver), u32>`. The SBA for "21+ commander damage from a single commander" checks this matrix. Note: only combat damage counts, and damage from copies of commanders does NOT count (the copy is not a commander).

### 7.4 Color Identity

Deck validation enforces color identity. The engine loads color identity from the card database and validates decks before game start. Hybrid mana, color indicators, and characteristic-defining abilities (e.g., Transguild Courier) all contribute to color identity per CR 903.4.

### 7.5 Politics & Multiplayer Interactions

Commander involves political elements: deals, alliances, threat assessment. The engine doesn't model these — they happen in voice chat or text. But the engine must support the mechanical consequences: "I won't attack you this turn" is a social contract, not a game rule. The engine should never prevent legal actions.

---

## 8. Development Tools & Environment

### 8.1 Project Structure

```
mtg-engine/
├── Cargo.toml                    (workspace root)
├── CLAUDE.md                     (Claude Code context document)
├── crates/
│   ├── engine/                   (core rules engine — no dependencies on UI or network)
│   │   ├── src/
│   │   │   ├── state/            (GameState, zones, objects)
│   │   │   ├── rules/            (turn structure, priority, stack, SBAs, layers, combat)
│   │   │   ├── cards/            (card definition types, keyword implementations)
│   │   │   ├── effects/          (effect resolution, replacement effects, triggers)
│   │   │   └── lib.rs
│   │   └── tests/
│   │       ├── rules/            (unit tests organized by CR section)
│   │       ├── interactions/     (integration tests for card combos)
│   │       ├── golden/           (full game replays)
│   │       └── properties/       (property-based/fuzz tests)
│   ├── network/                  (networking layer — wraps engine)
│   │   ├── src/
│   │   │   ├── protocol/         (message types, serialization)
│   │   │   ├── host/             (authoritative game host)
│   │   │   ├── client/           (client connection, state mirror)
│   │   │   └── lobby/            (game creation, player management)
│   │   └── tests/
│   ├── card-db/                  (SQLite card database management)
│   │   ├── src/
│   │   └── migrations/
│   └── card-pipeline/            (development tool: oracle text → card definitions)
│       └── src/
├── tauri-app/                    (Tauri application shell)
│   ├── src-tauri/                (Rust backend — glues engine + network + Tauri IPC)
│   └── src/                      (Svelte frontend)
│       ├── components/
│       │   ├── Battlefield.svelte
│       │   ├── Stack.svelte
│       │   ├── Hand.svelte
│       │   ├── CardDetail.svelte
│       │   └── TargetingOverlay.svelte
│       └── ...
├── test-data/
│   ├── golden-games/             (golden test game files)
│   ├── corner-cases.json         (known interaction test cases)
│   └── test-cards/               (synthetic cards for testing)
└── tools/
    ├── scryfall-import/          (script to import Scryfall bulk data)
    └── replay-viewer/            (tool to visualize game replays)
```

### 8.2 CLAUDE.md

The `CLAUDE.md` file is critical for Claude Code development. It should contain:

- Current development phase and milestone (referencing the roadmap)
- Architectural invariants that must not be violated
- Key design decisions and their rationale (back-reference this document)
- Rules reference conventions (how to cite CR sections in code comments)
- Test expectations (every PR must include tests; every rules implementation cites the CR section it implements)
- Known gotchas and things to watch for

### 8.3 MCP Server for Development

An MCP server providing Claude Code with:

- **CR search**: Semantic search over the comprehensive rules, returning full rule text with cross-references
- **Card lookup**: Query the Scryfall SQLite DB for card details, oracle text, and rulings
- **Rulings search**: Semantic search over all card rulings for a concept or interaction
- **Engine reference search**: Search Forge/XMage source code for implementations of specific mechanics

---

## Appendix A: Initial Corner Case Test List

These are known high-value test cases to implement early:

1. Humility + Opalescence (layer system interaction)
2. Blood Moon + Urborg, Tomb of Yawgmoth (dependency in layer 4)
3. Deathtouch + Trample (damage assignment)
4. First strike + Double strike ordering
5. Protection from X (all four effects: can't be damaged, enchanted/equipped/fortified, blocked, or targeted)
6. Hexproof vs. "each opponent" effects (targeting vs. non-targeting)
7. Copying a spell on the stack (characteristics of the copy)
8. Morph/manifest face-down characteristics
9. Panharmonicon-style trigger doubling
10. Replacement effect ordering (player's choice with multiple applicable)
11. "Dies" triggers with simultaneous destruction
12. APNAP (Active Player, Non-Active Player) ordering for triggers in multiplayer
13. Commander replacement effect when destroyed with Rest in Peace in play
14. Cascade into a split card (CMC rules)
15. Mutate stack ordering and characteristics
16. Companion deck restriction validation
17. Clone copying another Clone
18. Aura attached to an illegal permanent after type change
19. Phasing and auras/equipment persistence
20. Time Stamp ordering with Opalescence-type effects entering at different times

---

## Appendix B: Key Comprehensive Rules Sections

Reference list for the most critical CR sections that the engine must implement:

- **109**: Objects
- **110-112**: Permanents, Spells, Abilities
- **117**: Timing and Priority
- **400-408**: Zones
- **405**: Stack
- **500-514**: Turn Structure
- **601**: Casting Spells
- **602**: Activating Abilities
- **603**: Handling Triggered Abilities
- **608**: Resolving Spells and Abilities
- **613**: Interaction of Continuous Effects (Layer System)
- **614-616**: Replacement Effects, Prevention Effects, Interaction of R/P Effects
- **700-704**: General Rules, State-Based Actions
- **706**: Copying Objects
- **903**: Commander format rules

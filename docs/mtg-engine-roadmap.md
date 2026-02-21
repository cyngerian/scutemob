# MTG Commander Rules Engine: Development Roadmap

## Purpose of This Document

This document defines the development roadmap as a sequence of milestones, each with concrete deliverables, acceptance criteria, and dependencies. It is the project management backbone — all task tracking, sprint planning, and progress assessment flows from this document.

This roadmap is designed in lockstep with the Architecture & Testing Strategy (`mtg-engine-architecture.md`). Each milestone builds on the architecture defined there, and the testing requirements at each phase are integral to the milestone's completion criteria.

The Game Script Generation & Validation Strategy (`mtg-engine-game-scripts.md`) defines an engine-independent testing methodology that runs parallel to engine development. Script generation tasks are integrated into the milestones below.

---

## Guiding Principles

1. **Test-first, always.** No milestone is complete without its test suite passing. Tests are written before or alongside implementation, never after.
2. **Engine before UI.** The rules engine is the product. The UI is a consumer of the engine. Engine milestones do not depend on UI milestones.
3. **Correctness before coverage.** It's better to have 50 cards implemented correctly than 500 cards implemented with bugs. The layer system and priority must be right before card volume matters.
4. **Commander is the target.** Every design decision considers multiplayer from the start. 1v1 is a degenerate case of multiplayer, not the other way around.
5. **Playable increments.** Each major milestone produces something testable — either programmatically or by a human player.
6. **Independent validation.** Game scripts generated from rules reasoning (not engine behavior) serve as an external correctness oracle. The engine is tested against scripts it never influenced.

---

## Milestone Overview

```
M0: Project Scaffold & Data Foundation          (~1-2 weeks)
M1: Game State & Object Model                   (~2-3 weeks)
M2: Turn Structure & Priority (Multiplayer)     (~2-3 weeks)
M3: Stack, Spells & Abilities                   (~3-4 weeks)
M4: State-Based Actions                         (~1-2 weeks)
M5: The Layer System                            (~3-4 weeks)
M6: Combat                                      (~2-3 weeks)
M7: Card Definition Framework & First Cards     (~3-4 weeks)
M8: Replacement & Prevention Effects            (~2-3 weeks)
M9: Commander Rules Integration                 (~2-3 weeks)
───────────────────────────────────────────────────────────
    ENGINE CORE COMPLETE — Playable via tests
───────────────────────────────────────────────────────────
M10: Networking Layer (Distributed Verification)  (~3-4 weeks)
M10.5: Mental Poker Integration (NEW)             (~2-3 weeks)
M11: Tauri App Shell & Basic UI                  (~3-4 weeks)
M12: Card Definition Pipeline (Bulk Generation)  (~3-4 weeks)
M13: Full UI — Battlefield, Stack, Targeting     (~4-6 weeks)
M14: Card Asset Management & Polish              (~2-3 weeks)
M15: Alpha — End-to-End Commander Games          (~2-3 weeks)
───────────────────────────────────────────────────────────
    ALPHA RELEASE — Playable networked Commander
───────────────────────────────────────────────────────────
M16+: Post-Alpha (ongoing)
```

Estimated total to Alpha: **~9-12 months** of active development. Time estimates assume Claude Code is the primary development tool with significant velocity gains over manual coding.

---

## Milestone Details

---

### M0: Project Scaffold & Data Foundation

**Goal**: Establish the project structure, tooling, and data pipeline so that all subsequent milestones can build on a solid foundation.

**Deliverables**:
- [x] Cargo workspace with crate structure per architecture doc (`engine`, `network`, `card-db`, `card-pipeline`)
- [x] Tauri app scaffold with Svelte frontend (minimal — just a window that loads)
- [x] `CLAUDE.md` with initial architecture context, coding conventions, and CR citation format
- [x] Scryfall bulk data importer: script that downloads latest bulk data and populates SQLite DB
- [x] SQLite schema for cards, rulings, card_faces, card_definitions (per architecture doc Section 5.2)
- [x] MCP server configuration for Claude Code: CR search + card/rulings lookup
- [x] CI pipeline: `cargo test`, `cargo clippy`, `cargo fmt` on every commit
- [x] `im-rs` dependency added; basic proof-of-concept showing structural sharing works for game state
- [ ] Game script JSON schema defined as Rust types in `engine` crate (see `mtg-engine-game-scripts.md` Hook 1)
- [ ] `test-data/generated-scripts/` directory structure with subdirectories per subsystem

**Acceptance Criteria**:
- [x] `cargo build` succeeds for all crates
- [x] `cargo test` runs (even if there are few tests)
- [x] SQLite DB contains all Standard-legal cards with oracle text and rulings
- [x] MCP server responds to CR and card queries
- [x] Tauri app launches and shows a blank window
- [ ] `GameScript` Rust type compiles and can round-trip serialize/deserialize a sample JSON script

**Dependencies**: None (this is the root)

**Architecture doc references**: Section 5 (Card Data Architecture), Section 8 (Development Tools)

---

### M1: Game State & Object Model

**Goal**: Implement the core data structures that represent an MTG game. No rules logic yet — just the data model.

**Deliverables**:
- [x] `GameState` struct with all fields per architecture doc Section 2.1
- [x] `GameObject` with full characteristics representation
- [x] Zone system: all zone types, zone membership tracking
- [x] `ObjectId` generation and tracking; object identity rules per CR 400.7
- [x] `PlayerId` and `PlayerState` with Commander-relevant fields (life 40, commander tax, commander damage matrix)
- [x] `GameStateBuilder` test utility with fluent API
- [x] Snapshot/clone tests proving structural sharing performance

**Tests** (minimum):
- [x] Construct a game state with 4 players, 100-card decks, permanents on battlefield
- [x] Clone game state; verify clone is independent (modify one, other unchanged)
- [x] Object identity: create object, move to graveyard, verify new ObjectId
- [x] Zone integrity: every object in exactly one zone
- [x] Performance: clone a complex state in <1ms

**Acceptance Criteria**:
- [x] All tests pass
- [x] `GameStateBuilder` can construct any game state needed by future milestones
- [x] State clone benchmark meets <1ms target

**Dependencies**: M0

**Architecture doc references**: Section 2 (Game State Model)

---

### M2: Turn Structure & Priority (Multiplayer)

**Goal**: Implement the turn state machine and multiplayer priority system. After this milestone, the engine can "run" a game of players doing nothing but passing priority through every phase.

**Deliverables**:
- [x] Turn FSM: all phases and steps as enum, transition function
- [x] Priority state machine: `PlayerHasPriority`, `CheckingStateBasedActions`, `OrderingTriggers`, `AllPassed`
- [x] Multiplayer priority passing: APNAP order, pass counter, reset on action
- [x] Turn-based actions for each step (untap all, draw card, empty mana pools, etc.)
- [x] Extra turn tracking (for future use)
- [x] Extra combat tracking (for future use)
- [x] `Command::PassPriority` processing

**Tests** (minimum):
- [x] Full turn cycle: verify each phase/step is visited in order
- [x] Priority passes through all 4 players before stack resolves
- [x] Active player receives priority first after each step transition
- [x] Turn-based actions fire at correct steps (untap during untap, draw during draw)
- [x] Cleanup step: hand size check, "until end of turn" effects expire
- [x] Extra turn insertion: verify turn order modification
- [x] Multiplayer: player elimination doesn't break turn order

**Acceptance Criteria**:
- [x] A 4-player game can run 10 full turn cycles with all players passing priority, visiting every phase/step correctly
- [x] Turn structure matches CR 500-514 exactly

**Dependencies**: M1

**Architecture doc references**: Section 3.1 (Turn Structure), Section 3.2 (Priority System)

---

### M3: Stack, Spells & Abilities

**Goal**: Implement the stack zone, spell casting, ability activation, and resolution. After this milestone, players can cast spells and activate abilities (with simplified card logic). Also implement Tier 1 deterministic state hashing (prerequisite for distributed verification in M10).

**Deliverables**:
- [x] **Tier 1: Deterministic State Hashing** (from `mtg-engine-network-security.md`):
  - [x] `public_state_hash()` on `GameState` — deterministic hash of all public state
  - [x] `private_state_hash(player)` on `GameState` — hash of a player's hidden info
  - [x] Dual-instance hash comparison property test (process same commands on two independent engine instances, assert hashes match after every command)
  - [x] Run property test in CI on every commit
- [x] Stack as ordered zone with `StackObject` type (`state/stack.rs`: `StackObject`, `StackObjectKind`)
- [x] Mana ability system: `ManaAbility` struct, `TapForMana` command, mana pool add/empty (CR 605)
- [x] Land playing: `PlayLand` command, special action (not on stack), land plays remaining (CR 305.1)
- [x] Spell casting: `CastSpell` command, sorcery/instant speed, Flash, spell enters Stack zone, `StackObject` pushed (CR 601)
- [x] `keywords: OrdSet<KeywordAbility>` on `Characteristics` for keyword-based speed (Flash)
- [ ] Ability activation process per CR 602 (`Command::ActivateAbility`)
- [ ] Triggered ability handling per CR 603: trigger event detection, APNAP ordering for simultaneous triggers, "intervening if" clauses
- [ ] Resolution per CR 608: resolve top of stack, carry out effects
- [ ] Countering: a countered spell moves to graveyard (or exile, depending on effect)
- [ ] Mana payment cost validation on cast (currently deferred — M3-D)
- [ ] Target legality validation on cast and on resolution (fizzle rule)

**Tests** (minimum):
- [x] Cast a sorcery during main phase with empty stack — legal
- [x] Cast a sorcery during opponent's turn — illegal
- [x] Cast an instant in response to a spell — legal, stack has 2 items
- [x] Flash spell castable at instant speed outside main phase
- [x] Priority resets to active player after casting (CR 601.2i)
- [x] Stack is LIFO: second spell cast is on top
- [x] Mana ability tap: tap land, verify mana pool increases, player retains priority
- [x] Play land: land moves to battlefield, land plays decremented, players_passed resets
- [ ] Resolve stack in LIFO order
- [ ] Spell fizzles: all targets become illegal before resolution
- [ ] Spell partially fizzles: some targets illegal, remaining resolve
- [ ] Triggered ability: permanent enters battlefield, trigger goes on stack
- [ ] Multiple simultaneous triggers: APNAP ordering in 4-player game
- [ ] Intervening-if: trigger checks condition on trigger and on resolution

**Progress** (in-milestone tracking):
- [x] Tier 1 state hashing (blake3, `public_state_hash`, `private_state_hash`, 19 hash tests)
- [x] M3-A: `StackObject`/`StackObjectKind`, `ManaAbility`, `TapForMana`, `PlayLand` (19 tests)
- [x] M3-B: `CastSpell`, casting windows, Flash, `keywords` field, `SpellCast` event (12 tests)
- [ ] M3-C: Stack resolution — all-pass → resolve top, LIFO, move to graveyard, countering
- [ ] M3-D: Target legality — fizzle rule, partial fizzle, cost payment validation
- [ ] M3-E: `ActivateAbility`, triggered abilities, APNAP ordering, intervening-if

**Acceptance Criteria**:
- Players can cast spells from hand, pay mana, and have them resolve
- Abilities (activated and triggered) function on the stack
- Priority flows correctly between each stack addition and resolution
- All stack-related CR examples from sections 405, 601-608 pass as tests

**Dependencies**: M2

**Architecture doc references**: Section 3.3 (The Stack), Section 3.7 (Card Definition Runtime)

---

### M4: State-Based Actions

**Goal**: Implement the full SBA check as a fixed-point computation. This milestone is intentionally small but critical — SBAs interact with everything.

**Deliverables**:
- [ ] SBA check loop: check all SBAs, apply any that trigger, repeat until none trigger
- [ ] All SBAs from CR 704.5, including:
  - [ ] Player at 0 or less life loses (704.5a)
  - [ ] Player who attempted to draw from empty library loses (704.5b)
  - [ ] Creature with 0 or less toughness is put into graveyard (704.5f)
  - [ ] Creature with lethal damage is destroyed (704.5g)
  - [ ] Creature with deathtouch damage is destroyed (704.5h)
  - [ ] Planeswalker with 0 loyalty is put into graveyard (704.5i)
  - [ ] Legendary rule (704.5j)
  - [ ] Token in a non-battlefield zone ceases to exist (704.5d)
  - [ ] Aura attached to illegal object goes to graveyard (704.5m)
  - [ ] Equipment/fortification attached illegally becomes unattached (704.5n)
  - [ ] +1/+1 and -1/-1 counter annihilation (704.5q)
  - [ ] Commander damage >= 21 causes loss (704.5u — Commander specific)
- [ ] SBA integration with priority: SBAs checked every time any player would receive priority
- [ ] Triggers generated by SBAs are collected and placed on stack after all SBAs finish

**Tests** (minimum):
- [ ] Each SBA individually in isolation
- [ ] SBA chain: creature dies from SBA, death trigger produces token, no further SBAs
- [ ] SBA convergence: always terminates (property test)
- [ ] Legendary rule with 2+ copies: owner chooses which to keep
- [ ] Counter annihilation: 3 +1/+1 and 2 -1/-1 → 1 +1/+1 remains
- [ ] Multiple players at 0 life simultaneously: all lose simultaneously
- [ ] SBA triggers go on stack in APNAP order after all SBAs finish

**Game Script Tasks**:
- [ ] Generate 5-10 baseline game scripts covering simple scenarios: vanilla combat, basic spell resolution, simple priority passing. Use Claude Code with MCP tools, following the generation process in `mtg-engine-game-scripts.md`.
- [ ] Scripts stored in `test-data/generated-scripts/baseline/`
- [ ] All generated scripts human-reviewed and marked `approved` before use

**Acceptance Criteria**:
- All SBAs from CR 704.5 implemented and individually tested
- Fixed-point loop terminates for all tested states (property test)
- SBA check integrates correctly with priority system
- Baseline game scripts generated and reviewed (not yet replayable — replay harness comes in M7)

**Dependencies**: M3 (SBAs reference the stack for trigger placement)

**Architecture doc references**: Section 3.4 (State-Based Actions)

---

### M5: The Layer System

**Goal**: Implement the continuous effect layer system (CR 613). This is the hardest milestone. Budget time accordingly and expect iteration.

**Deliverables**:
- [ ] `ContinuousEffect` type with layer, sublayer, timestamp, duration, affected filter, modification
- [ ] Layer application function: given all active continuous effects, calculate characteristics of any object
- [ ] Timestamp system: effects get timestamps when they start; newer = later
- [ ] Dependency detection per CR 613.8: effect A depends on effect B if B could change what A applies to or what A does
- [ ] Dependency resolution: apply dependents after their dependencies; circular dependencies fall back to timestamp
- [ ] Duration tracking: "until end of turn", "as long as", "for as long as" — effects are removed when duration expires
- [ ] Characteristic-defining abilities (CDAs) calculated in the appropriate layer
- [ ] Copy effects (Layer 1): full copiable values handling per CR 707
- [ ] Control-changing effects (Layer 2)
- [ ] Type-changing effects (Layer 4): including interaction with Blood Moon style effects
- [ ] P/T modifications (Layer 7a-d): CDAs, setting, +/-, switching

**Tests** (minimum — this milestone has the most tests):
- [ ] Basic layer ordering: type change applies before P/T change
- [ ] Timestamp ordering within layer: later timestamp wins
- [ ] **Humility + Opalescence**: verify both cards' characteristics after full layer resolution
- [ ] **Blood Moon + Urborg**: dependency in layer 4 — Blood Moon depends on Urborg (or vice versa depending on timestamp)
- [ ] Copy effect on a permanent with continuous effects
- [ ] Control change via continuous effect; verify controller changes propagate
- [ ] "Until end of turn" effects removed during cleanup
- [ ] CDA in layer 7a: Tarmogoyf power/toughness calculation
- [ ] P/T switching (layer 7d) after other P/T effects
- [ ] Removal of source: continuous effect from a permanent that leaves the battlefield
- [ ] Multiple dependencies forming a chain (A depends on B depends on C)
- [ ] Circular dependency: falls back to timestamp order
- [ ] At least 20 additional corner cases from Appendix A of architecture doc

**Game Script Tasks**:
- [ ] Generate scripts for each layer system corner case in `mtg-engine-corner-cases.md` (cases 1-7, 30). These scripts document the expected behavior with full CR citations, independent of the engine.
- [ ] Generate 5-10 additional scripts for layer interactions not covered by the corner cases doc (e.g., basic continuous effects from enchantments, +1/+1 counters interacting with setting effects)
- [ ] Scripts stored in `test-data/generated-scripts/layers/`
- [ ] Cross-validate each script (second Claude Code pass verifying CR citations)
- [ ] All scripts human-reviewed

**Acceptance Criteria**:
- All 20+ corner case tests pass
- Layer system produces correct characteristics for every object in any test state
- Performance benchmark: <1ms for 50 continuous effects
- Layer system game scripts generated, cross-validated, and reviewed

**Dependencies**: M1 (state model), M3 (effects reference stack for spell-based continuous effects)

**Architecture doc references**: Section 3.5 (The Layer System)

**Risk note**: This is the highest-risk milestone. The dependency system is subtle and the test cases may reveal architectural issues requiring refactoring of the effect representation. Budget extra time and plan for iteration.

---

### M6: Combat

**Goal**: Implement the complete combat system: attacker declaration, blocker declaration, damage assignment, and all combat-related mechanics.

**Deliverables**:
- [ ] `CombatState` tracking: attackers, blockers, damage assignment orders
- [ ] Attacker declaration: legal attack targets (player or planeswalker, expanded in Commander to "any opponent or opponent's planeswalker"), restrictions and requirements
- [ ] Blocker declaration: legal blocks, blocking restrictions/requirements, damage assignment order
- [ ] Combat damage assignment: lethal damage rule, player choice for ordering
- [ ] First strike / double strike: extra combat damage step
- [ ] Trample: excess damage to defending player/planeswalker
- [ ] Deathtouch + trample interaction: 1 damage is lethal
- [ ] Damage prevention in combat (foreshadowing M8 but basic framework here)
- [ ] "Whenever this creature attacks/blocks" triggers
- [ ] "Whenever this creature deals combat damage to a player" triggers
- [ ] Commander damage tracking: combat damage from a commander increments the matrix
- [ ] Multiplayer combat: active player attacks one or more opponents; each opponent declares blockers for creatures attacking them

**Tests** (minimum):
- [ ] Basic combat: 2/2 attacks, unblocked, defending player takes 2
- [ ] Blocked combat: 2/2 attacks, blocked by 3/3, 2/2 dies
- [ ] Mutual destruction: 3/3 attacks, blocked by 3/3, both die
- [ ] First strike: first striker kills blocker before blocker deals damage
- [ ] Double strike: deals damage in both steps
- [ ] Trample: 5/5 with trample blocked by 2/2, 3 damage to player
- [ ] Deathtouch + trample: 1 to blocker (lethal), rest to player
- [ ] Multiple blockers: damage assignment order, distribute lethal
- [ ] Combat triggers fire at correct timing
- [ ] Commander combat damage tracked in matrix
- [ ] Multiplayer: player A attacks player B and player C simultaneously

**Game Script Tasks**:
- [ ] Generate scripts for combat corner cases from `mtg-engine-corner-cases.md` (cases 8, 9, 20, 21, 22)
- [ ] Generate keyword interaction matrix scripts: create a script for every meaningful pair of combat keywords (first strike × deathtouch, trample × protection, flying × reach, menace × single blocker, etc.)
- [ ] Generate multiplayer combat scripts: player A attacks both player B and player C, each declares blockers independently
- [ ] Scripts stored in `test-data/generated-scripts/combat/`
- [ ] All scripts cross-validated and human-reviewed

**Acceptance Criteria**:
- Full combat phase executes correctly for multiplayer
- All combat keyword interactions tested
- Commander damage tracking accurate
- Combat game scripts generated, cross-validated, and reviewed

**Dependencies**: M4 (SBAs check lethal damage), M5 (continuous effects modify P/T and abilities)

**Architecture doc references**: Section 3.1 (Combat Phase in turn structure)

---

### M7: Card Definition Framework & First Cards

**Goal**: Build the card definition system, implement the first set of real cards, and build the game script replay harness so generated scripts become executable tests.

**Deliverables**:
- [ ] `CardDefinition` struct and `AbilityDefinition` enum per architecture doc Section 3.7
- [ ] `Effect` recursive enum with all primitives (DealDamage, GainLife, DrawCards, CreateToken, etc.)
- [ ] Card definition loader: read from `card_definitions` table, instantiate abilities
- [ ] Keyword ability implementations (first batch):
  - [ ] Flying / Reach
  - [ ] First strike / Double strike
  - [ ] Trample
  - [ ] Deathtouch
  - [ ] Lifelink
  - [ ] Haste
  - [ ] Vigilance
  - [ ] Hexproof / Shroud
  - [ ] Indestructible
  - [ ] Flash
  - [ ] Menace
- [ ] Target type system: legal targets for "target creature", "target player", "target permanent", etc.
- [ ] Mode selection for modal spells
- [ ] **First 50 real card definitions**: hand-authored, focusing on Commander staples (Sol Ring, Command Tower, Lightning Greaves, Swords to Plowshares, Counterspell, Cultivate, etc.)
- [ ] Test harness that loads a card definition and verifies its behavior in isolation
- [ ] **Game script replay harness** (see `mtg-engine-game-scripts.md` Hook 2): loads a `GameScript` JSON, constructs initial state via `GameStateBuilder`, feeds actions as `Command`s to the engine, asserts state at every `assert_state` checkpoint
- [ ] **Script auto-discovery test** (see `mtg-engine-game-scripts.md` Hook 3): `cargo test` automatically finds and runs all approved scripts in `test-data/generated-scripts/`
- [ ] Run all previously generated scripts (from M4-M6) through the replay harness; fix mismatches

**Game Script Tasks**:
- [ ] Generate scripts for the first 50 cards' individual behaviors (cast Sol Ring, tap for mana; cast Swords to Plowshares, exile creature and gain life; etc.)
- [ ] Generate scripts for keyword ability scenarios using real cards
- [ ] Scripts stored in `test-data/generated-scripts/` organized by subsystem
- [ ] All scripts cross-validated and reviewed

**Tests** (minimum):
- [ ] Each keyword ability in a combat or game scenario
- [ ] Sol Ring: tap for 2 colorless mana
- [ ] Swords to Plowshares: exile target creature, controller gains life equal to power
- [ ] Counterspell: counter target spell
- [ ] Lightning Bolt: 3 damage to any target
- [ ] Cultivate: search library for two basic lands, one to battlefield tapped, one to hand
- [ ] Modal spell: choose one or more modes, each resolves
- [ ] Card definition load/save round-trip
- [ ] Replay harness processes a simple script end-to-end
- [ ] Replay harness detects a deliberate state mismatch (negative test)
- [ ] Script auto-discovery finds and runs all approved scripts

**Acceptance Criteria**:
- 50 real cards implemented and individually tested
- All keyword abilities functional
- Card definition system is extensible (adding a new card doesn't require engine changes)
- Replay harness runs all approved scripts from M4-M6; all pass
- Any script failures investigated and resolved (engine fix or script correction)

**Dependencies**: M3-M6 (the card framework exercises all prior systems)

**Architecture doc references**: Section 3.7 (Card Definition Runtime)

---

### M8: Replacement & Prevention Effects

**Goal**: Implement replacement effects and prevention effects, which modify events as they happen rather than triggering afterward.

**Deliverables**:
- [ ] Replacement effect framework: intercept an event, apply modification, continue with modified event
- [ ] Self-replacement effects: apply before other replacement effects (CR 614.15)
- [ ] Player choice when multiple replacement effects apply to the same event
- [ ] Loop prevention: a replacement effect can modify a given event at most once (CR 614.5)
- [ ] Prevention effects: prevent N damage, prevent all damage, etc.
- [ ] Prevention/replacement interaction per CR 616
- [ ] "If ~ would die" replacement effects (critical for Commander zone-change choice)
- [ ] "If a player would draw" replacement effects (e.g., Notion Thief)
- [ ] "Enters the battlefield" replacement effects (e.g., "enters tapped")

**Tests** (minimum):
- [ ] Simple replacement: "If you would gain life, draw that many cards instead"
- [ ] Multiple replacement effects: player chooses order of application
- [ ] Self-replacement: applies first regardless of player choice
- [ ] Loop prevention: same effect can't apply twice to same event
- [ ] Prevention shield: "prevent the next 3 damage" then take 5 damage → 2 gets through
- [ ] Replacement + prevention interaction: which applies first (player's choice per CR 616)
- [ ] Commander zone-change replacement: commander would die → choose command zone or graveyard

**Game Script Tasks**:
- [ ] Generate scripts for replacement effect corner cases from `mtg-engine-corner-cases.md` (cases 16-19, 28, 33)
- [ ] Generate scripts for prevention effect scenarios (damage prevention shields, protection preventing damage)
- [ ] Generate scripts for replacement + prevention interaction ordering
- [ ] Scripts stored in `test-data/generated-scripts/replacement/`
- [ ] All scripts cross-validated and reviewed

**Acceptance Criteria**:
- Replacement effects integrate cleanly with existing event system
- Commander zone-change choice works correctly
- No infinite loops possible in replacement effect chains
- Replacement effect game scripts pass through replay harness

**Dependencies**: M4 (SBAs generate events that can be replaced), M5 (continuous effects can create replacement effects)

**Architecture doc references**: Section 3.6 (Replacement Effects)

---

### M9: Commander Rules Integration

**Goal**: Implement all Commander-specific rules as a cohesive layer on top of the core engine.

**Deliverables**:
- [ ] Commander format enforcement:
  - [ ] 100-card singleton deck validation
  - [ ] Color identity validation
  - [ ] Banned list checking (loaded from card DB)
- [ ] Command zone mechanics:
  - [ ] Casting commander from command zone
  - [ ] Commander tax: additional {2} for each previous cast from command zone
  - [ ] Commander tax tracks separately per commander (for partners)
- [ ] Commander replacement effects:
  - [ ] "If your commander would go to graveyard/exile from anywhere, you may put it in the command zone instead"
  - [ ] Tax increments on cast, not on zone change
- [ ] Commander damage:
  - [ ] SBA: player who has received 21+ combat damage from a single commander loses
  - [ ] Tracking across zone changes (the commander is the same card even with new ObjectId)
- [ ] Partner mechanics: two commanders, shared color identity, separate tax
- [ ] Companion (if in scope): deck restriction validation, companion casting from sideboard-equivalent
- [ ] Mulligan: Commander-specific free mulligan, then London mulligan
- [ ] Starting life: 40

**Tests** (minimum):
- [ ] Deck validation: reject 99-card deck, reject off-color-identity cards, reject banned cards
- [ ] Cast commander: first cast costs printed cost, second costs +2, third costs +4
- [ ] Partner commanders: each tracked separately for tax
- [ ] Commander dies: player chooses command zone or graveyard
- [ ] Commander exiled: player chooses command zone or exile
- [ ] Commander damage: 21 combat damage from one commander → SBA loss
- [ ] Commander damage: 10 from commander A + 11 from commander B → no loss (tracked separately)
- [ ] Commander damage from a copy of a commander: does NOT count
- [ ] Free mulligan then London mulligan sequence
- [ ] 4-player game start: all commander-specific setup correct

**Game Script Tasks**:
- [ ] Generate scripts for Commander corner cases from `mtg-engine-corner-cases.md` (cases 26, 27, 28)
- [ ] Generate scripts for full Commander game setup: mulligan sequence, first few turns with commander casting and tax
- [ ] Generate scripts for commander damage tracking across multiple combats and zone changes
- [ ] Generate scripts for partner commander interactions
- [ ] Scripts stored in `test-data/generated-scripts/commander/`
- [ ] All scripts cross-validated and reviewed

**Acceptance Criteria**:
- A full 4-player Commander game can be played programmatically (via test commands) from game start through win/loss conditions
- All Commander-specific rules from CR 903 tested
- All Commander game scripts pass through replay harness
- This milestone marks **Engine Core Complete**

**Dependencies**: M1-M8 (all core systems)

**Architecture doc references**: Section 7 (Commander-Specific Design)

---

### ═══════════ ENGINE CORE COMPLETE ═══════════

At this point, the engine can run a complete Commander game programmatically. All rules are implemented and tested. No UI, no network — but any game scenario can be constructed and played via test code.

**Checkpoint validation**:
- [ ] Property tests pass: 50+ invariants validated via fuzzing
- [ ] All golden tests pass (at least 5 hand-authored full game replays)
- [ ] All approved game scripts pass through replay harness (~100+ scripts)
- [ ] All corner case tests from `mtg-engine-corner-cases.md` pass
- [ ] Performance benchmarks meet targets

---

### M10: Networking Layer (Distributed Verification)

**Goal**: Implement the distributed verification network model where all peers run the engine independently and a lightweight coordinator manages protocol sequencing. See `mtg-engine-network-security.md` for full design.

> **Architecture change**: This milestone implements Tier 2 (distributed verification)
> from the network security strategy, replacing the original authoritative host model.
> A trusted host fallback mode is also supported. Tier 1 (deterministic state hashing)
> is a prerequisite — implemented during M3.

**Deliverables**:
- [ ] Peer-to-peer mesh networking (WebSocket mesh or WebRTC)
- [ ] Coordinator role: protocol sequencing, command broadcasting (lightweight — no game state)
- [ ] All peers run engine independently on received commands
- [ ] Hash comparison after every command (using `public_state_hash()` from Tier 1)
- [ ] Dispute detection: hash mismatch → majority vote resolution
- [ ] Coordinator election and migration on disconnect (lowest peer ID)
- [ ] Message protocol: Command/Event serialization with MessagePack (serde)
- [ ] Lobby system: create game, join game, set parameters, select security mode
- [ ] Reconnection protocol: public state sync from majority peers
- [ ] Trusted host fallback mode: legacy authoritative model for debugging/simplicity
- [ ] Basic latency tolerance: commands are timestamped and ordered

**Tests** (minimum):
- [ ] Coordinator starts game, 4 peers connect, game begins
- [ ] Command round-trip: acting peer sends command, all peers process, hashes compared
- [ ] Hash consensus: all 4 peers agree after every command
- [ ] Dispute detection: one peer with tampered state is flagged
- [ ] Coordinator migration: coordinator disconnects, next peer takes over seamlessly
- [ ] Reconnect: peer disconnects, rejoins, state synced from majority
- [ ] Invalid command rejection: all peers reject illegal command independently
- [ ] Latency simulation: 200ms delay, game plays correctly
- [ ] Trusted host fallback: same game works in legacy mode

**Acceptance Criteria**:
- 4-player Commander game playable over localhost via programmatic clients
- All peers independently verify every game state transition
- Hash mismatch detection works and flags the divergent peer
- Coordinator migration is seamless
- Trusted host fallback mode functional

**Dependencies**: M9 (engine core complete), Tier 1 state hashing (implemented during M3)

**Architecture doc references**: Section 4 (Networking Architecture), `mtg-engine-network-security.md` Tier 2

---

### M10.5: Mental Poker Integration

**Goal**: Implement cryptographic card dealing so no player can see hidden information (library order, other players' hands) without the distributed trust of all peers.

> See `mtg-engine-network-security.md` Tier 3 for full protocol design.

**Deliverables**:
- [ ] Commutative encryption (ElGamal over Curve25519 via `curve25519-dalek`)
- [ ] Joint shuffle protocol with zero-knowledge shuffle proofs
- [ ] Draw protocol: partial decryption from each peer reveals card only to drawing player
- [ ] Search protocol: full library reveal to searching player + re-shuffle afterward
- [ ] Commit-reveal scheme for public random events (coin flips, random selection)
- [ ] Hand commitment system: prove card legitimacy when playing from hand
- [ ] MTG-specific operations: scry, reveal, look at hand, morph/manifest
- [ ] Integration with engine's draw, search, and shuffle commands (engine stays crypto-free)
- [ ] Performance testing: draw latency <100ms, search latency <1s

**Tests** (minimum):
- [ ] Joint shuffle: 4 players shuffle a 100-card deck, no player knows the order
- [ ] Draw protocol: drawing player sees card, opponents do not
- [ ] Search protocol: searching player sees library, library re-shuffled afterward
- [ ] Commit-reveal: public random result is unbiased
- [ ] Hand commitment: player proves they held a card when casting it
- [ ] Performance: draw <100ms, search <1s with 4 peers
- [ ] Integration: full game with Mental Poker produces same outcomes as seeded RNG game

**Acceptance Criteria**:
- Hidden information is cryptographically protected during networked play
- Performance is acceptable for turn-based gameplay
- Engine crate has zero cryptographic dependencies (all crypto in network layer)

**Dependencies**: M10 (distributed verification networking)

**Architecture doc references**: `mtg-engine-network-security.md` Tier 3

---

### M11: Tauri App Shell & Basic UI

**Goal**: Build the Tauri application with a functional but minimal UI. Players can see the game and interact, even if it's not polished.

**Deliverables**:
- [ ] Tauri IPC bridge: Rust commands exposed to Svelte frontend
- [ ] Game state rendering: display all zones, players, life totals
- [ ] Card display: render cards from cached images, fallback to text-only
- [ ] Hand display: player's cards in hand, clickable to select
- [ ] Battlefield display: grid/freeform layout of permanents, tapped state visible
- [ ] Stack display: ordered list of stack objects with source card info
- [ ] Phase/step indicator: current turn phase displayed
- [ ] Priority indicator: whose turn to act
- [ ] Basic input: click to cast spell, click to pass priority, click to select targets
- [ ] Life total display and commander damage tracker per opponent

**Tests**: UI tests are manual at this stage. Checklist:
- [ ] Launch app, connect to local host, see game state
- [ ] Cast a spell from hand by clicking it
- [ ] Pass priority
- [ ] See stack update when opponents cast spells
- [ ] See battlefield update when permanents enter/leave

**Acceptance Criteria**:
- A human player can play a simplified Commander game through the UI (against programmatic opponents or other humans on localhost)
- All game information is visible and actionable

**Dependencies**: M10 (networking for multi-window testing), M0 (Tauri scaffold)

---

### M12: Card Definition Pipeline (Bulk Generation)

**Goal**: Scale from 50 hand-authored cards to 500+ using Claude-assisted generation.

**Deliverables**:
- [ ] Pipeline tool (`card-pipeline` crate): takes oracle text + rulings, outputs structured `CardDefinition`
- [ ] Batch processing: run the pipeline over a set of cards, output JSON definitions
- [ ] Validation harness: each generated definition is tested against known rulings
- [ ] Failure tracking: cards that fail validation are flagged for human review
- [ ] Priority queue: cards ordered by EDHREC popularity (Commander staples first)
- [ ] First 500 cards generated and validated
- [ ] Coverage report: percentage of Commander-legal cards with definitions
- [ ] Generate game scripts for newly defined cards' key interactions

**Tests**:
- [ ] Pipeline generates correct definitions for the original 50 hand-authored cards (baseline)
- [ ] Newly generated cards pass individual behavior tests
- [ ] Known interactions between newly generated cards pass integration tests
- [ ] New game scripts for card interactions pass through replay harness

**Acceptance Criteria**:
- 500+ cards with validated definitions
- Pipeline has a documented process for adding more cards
- Failure rate <10% on first-pass generation

**Dependencies**: M7 (card definition framework), M9 (engine can execute cards)

**Architecture doc references**: Section 5.3 (Card Definition Pipeline)

---

### M13: Full UI — Battlefield, Stack, Targeting

**Goal**: Polish the UI into a rich, interactive experience suitable for Commander gameplay.

**Deliverables**:
- [ ] Battlefield layout: zones for each player, permanents grouped by type, visual tapping
- [ ] Targeting UI: click source → click target, with arrow/line overlay showing connections
- [ ] Stack visualization: expandable stack with card art, targets shown, source shown
- [ ] Combat UI: attack declaration (drag to opponent), blocker declaration (drag to attacker), damage assignment
- [ ] Triggered ability ordering: when multiple triggers, player drags to reorder
- [ ] Modal/choice UI: popup for mode selection, replacement effect choices, etc.
- [ ] Card zoom: hover or click for full card view with oracle text and rulings
- [ ] Game log: scrollable log of all game events in natural language
- [ ] Turn history: step back through turn states (using immutable snapshots)
- [ ] Responsive layout: works on various screen sizes

**Tests**: Manual testing with real gameplay sessions. Automated screenshot regression tests if feasible.

**Acceptance Criteria**:
- A full 4-player Commander game is playable and visually clear
- All player decisions are accessible through the UI
- No game information is hidden that should be visible

**Dependencies**: M11 (basic UI), M10 (networking)

---

### M14: Card Asset Management & Polish

**Goal**: Implement the card image download system and polish the overall experience.

**Deliverables**:
- [ ] Set browser: list available sets, download card images per set
- [ ] Background download manager: non-blocking downloads with progress
- [ ] Image caching: `~/.mtg-engine/assets/images/` per architecture doc Section 5.4
- [ ] Fallback rendering: text-only card display when image not cached
- [ ] Deck builder: basic deck construction UI with card search, color identity filtering, legality validation
- [ ] Deck import/export: support common formats (MTGO .txt, Arena format, Moxfield URL)
- [ ] Settings: display preferences, network configuration, asset management
- [ ] Error handling: graceful failures for network issues, missing assets, invalid decks

**Acceptance Criteria**:
- User can download card images for any set
- User can build and save a Commander deck
- App handles missing images gracefully

**Dependencies**: M13 (UI framework), M0 (card DB)

---

### M15: Alpha — End-to-End Commander Games

**Goal**: Integration testing and bug fixing to produce a playable alpha release.

**Deliverables**:
- [ ] End-to-end test: 4 human players play a full Commander game over network
- [ ] Bug triage and fixes from playtesting
- [ ] Missing card definitions identified and added for commonly-played staples
- [ ] Performance optimization: profile and fix any hot spots
- [ ] Crash reporting: basic telemetry for unhandled errors
- [ ] README and player-facing documentation
- [ ] Build pipeline: produce installable binaries for Windows, macOS, Linux
- [ ] Final game script validation: run full script suite, all approved scripts pass

**Acceptance Criteria**:
- 4 players can complete a Commander game without crashes
- All common staple cards (top 200 EDHREC) have working definitions
- Installable builds for all three platforms
- 200+ approved game scripts passing

**Dependencies**: M10-M14

---

### ═══════════ ALPHA RELEASE ═══════════

---

### M16+: Post-Alpha (Future Roadmap)

These are not scheduled but represent the next directions after alpha:

**Card coverage expansion**:
- Continue generating card definitions toward full Commander-legal coverage
- Community contribution pipeline: players can submit and validate card definitions
- Community-submitted game scripts for edge cases and new interactions

**Gameplay features**:
- Spectator mode
- Replay recording and playback
- Game save/load (serialize full state + command log)
- House rules configuration (custom banned lists, starting life, etc.)
- Planechase / Archenemy variant support

**Networking improvements**:
- Host migration: if host drops, another player takes over
- Direct IP connect (no matchmaking server needed)
- Latency optimization: client-side prediction for non-hidden actions

**UI improvements**:
- Card art animations
- Sound effects
- Keyboard shortcuts
- Accessibility (screen reader support, colorblind modes)
- Theming and customization

**Engine improvements**:
- AI opponent (rules-based, not LLM — for testing and solo play)
- Performance: WASM compilation for potential web client
- Additional formats: Standard, Modern, Legacy (engine already supports them; just need card pool filtering)

**Testing improvements**:
- Game script review interface: visual HTML renderer for human review workflow
- Automated regression seed generation from bug reports
- Forge/XMage cross-reference for three-way validation

---

## Risk Register

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Layer system complexity exceeds estimates | M5 delayed 2-4 weeks | High | Start layer tests early; accept incremental correctness; reference Forge/XMage |
| Card definition pipeline produces too many errors | M12 requires extensive manual correction | Medium | Invest in rulings RAG quality; build feedback loop early |
| Networking introduces non-determinism | State divergence bugs in M10+ | Medium | Tier 1 state hashing from M3 onward catches non-determinism early; `im::OrdMap` ensures deterministic iteration |
| Mental Poker latency unacceptable | Library search operations too slow for gameplay | Low | Protocol designed for <1s search latency; acceptable for turn-based game; fallback to trusted host mode |
| Scryfall API changes or terms change | Card data pipeline breaks | Low | Vendor-lock only on data format, not API; cache aggressively |
| Performance bottleneck in layer recalculation | Unplayable with complex board states | Medium | Benchmark from M5; incremental recalculation if needed |
| Scope creep from Commander complexity | Milestones slip | High | Strict MVP: basic Commander first, variants and edge cases in post-alpha |
| Game script schema changes break existing scripts | Accumulated scripts need rework | Medium | Version the schema; write migration tooling; prefer additive changes |
| Engine and scripts disagree on correct behavior | Ambiguous which is wrong | Medium | Always trace back to CR text; when in doubt, check Forge/XMage; human judge review |

---

## Appendix: Milestone Dependency Graph

```
M0 ──→ M1 ──→ M2 ──→ M3 ──→ M4 ──→ M5 ──→ M6 ──→ M7 ──→ M8 ──→ M9
 │                    │Tier1│      │      │      │              │
 │                    │hash │      │      │      │              │
 │                    └──┬──┘      ▼      ▼      ▼              ▼
 │                       │       scripts scripts scripts    scripts
 │                       │       (base)  (layer) (combat)   (cmdr)
 │                       │                        (replay+
 │                       │                         cards)
 │                       │                           │
 └────────────────────────────────────────────────→ M11        M10 ←── Tier1
                                                     │          │
                                                     ▼          ▼
                                        M7 ──→ M12  M13     M10.5
                                                │    │  ←──────┘
                                                ▼    ▼
                                                M14 ←┘
                                                  │
                                                  ▼
                                                 M15
```

Engine milestones (M0-M9) are strictly sequential — each builds on the prior. Tier 1 state hashing is implemented during M3 and is a prerequisite for M10 distributed verification. Game script generation runs as a parallel workstream starting at M4, producing scripts that become executable tests when the replay harness arrives in M7. M10.5 (Mental Poker) depends on M10 and adds cryptographic hidden information protection. UI and networking (M10-M14) can partially overlap once the engine core is complete. M12 (card pipeline) can run in parallel with UI work since it's primarily a data generation effort.

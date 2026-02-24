# MTG Commander Rules Engine: Development Roadmap

## Purpose of This Document

This document defines the development roadmap as a sequence of milestones, each with concrete deliverables, acceptance criteria, and dependencies. It is the project management backbone — all task tracking, sprint planning, and progress assessment flows from this document.

This roadmap is designed in lockstep with the Architecture & Testing Strategy (`mtg-engine-architecture.md`). Each milestone builds on the architecture defined there, and the testing requirements at each phase are integral to the milestone's completion criteria.

The Game Script Generation & Validation Strategy (`mtg-engine-game-scripts.md`) defines an engine-independent testing methodology. **Hybrid approach**: the `GameScript` schema is defined as Rust types now (M5) so it evolves under the compiler, but actual script generation is deferred to M7 when the replay harness is built. Generating scripts before they can run risks format drift and wasted effort. M7 generates all baseline + subsystem scripts together and runs them immediately through the harness.

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
M9.5: Game State Stepper (Dev Replay Viewer)      (~2-3 weeks)
M10: Networking Layer (Centralized Server)         (~2-3 weeks)
M10.5: P2P Distributed Verification (DEFERRED)    (unscheduled)
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
- [x] Game script JSON schema defined as Rust types in `engine` crate (see `mtg-engine-game-scripts.md` Hook 1) — done in M5
- [x] `test-data/generated-scripts/` directory structure with subdirectories per subsystem — done in M5

**Acceptance Criteria**:
- [x] `cargo build` succeeds for all crates
- [x] `cargo test` runs (even if there are few tests)
- [x] SQLite DB contains all Standard-legal cards with oracle text and rulings
- [x] MCP server responds to CR and card queries
- [x] Tauri app launches and shows a blank window
- [x] `GameScript` Rust type compiles and can round-trip serialize/deserialize a sample JSON script — done in M5

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
- [x] Ability activation process per CR 602 (`Command::ActivateAbility`)
- [x] Triggered ability handling per CR 603: trigger event detection, APNAP ordering for simultaneous triggers, "intervening if" clauses
- [x] Resolution per CR 608: resolve top of stack, carry out effects
- [x] Countering: a countered spell moves to graveyard (or exile, depending on effect)
- [x] Mana payment cost validation on cast (M3-D)
- [x] Target legality validation on cast and on resolution (fizzle rule)

**Tests** (minimum):
- [x] Cast a sorcery during main phase with empty stack — legal
- [x] Cast a sorcery during opponent's turn — illegal
- [x] Cast an instant in response to a spell — legal, stack has 2 items
- [x] Flash spell castable at instant speed outside main phase
- [x] Priority resets to active player after casting (CR 601.2i)
- [x] Stack is LIFO: second spell cast is on top
- [x] Mana ability tap: tap land, verify mana pool increases, player retains priority
- [x] Play land: land moves to battlefield, land plays decremented, players_passed resets
- [x] Resolve stack in LIFO order
- [x] Spell fizzles: all targets become illegal before resolution
- [x] Spell partially fizzles: some targets illegal, remaining resolve
- [x] Triggered ability: permanent enters battlefield, trigger goes on stack
- [x] Multiple simultaneous triggers: APNAP ordering in 4-player game
- [x] Intervening-if: trigger checks condition on trigger and on resolution

**Progress** (in-milestone tracking):
- [x] Tier 1 state hashing (blake3, `public_state_hash`, `private_state_hash`, 19 hash tests)
- [x] M3-A: `StackObject`/`StackObjectKind`, `ManaAbility`, `TapForMana`, `PlayLand` (19 tests)
- [x] M3-B: `CastSpell`, casting windows, Flash, `keywords` field, `SpellCast` event (12 tests)
- [x] M3-C: Stack resolution — all-pass → resolve top, LIFO, move to graveyard, countering (10 tests)
- [x] M3-D: Target legality — fizzle rule, partial fizzle, cost payment validation (13 tests)
- [x] M3-E: `ActivateAbility`, triggered abilities, APNAP ordering, intervening-if (15 tests)

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
- [x] SBA check loop: check all SBAs, apply any that trigger, repeat until none trigger
- [x] All SBAs from CR 704.5, including:
  - [x] Player at 0 or less life loses (704.5a)
  - [x] Player who attempted to draw from empty library loses (704.5b — handled in M2 turn_actions)
  - [x] Creature with 0 or less toughness is put into graveyard (704.5f)
  - [x] Creature with lethal damage is destroyed (704.5g)
  - [x] Creature with deathtouch damage is destroyed (704.5h)
  - [x] Planeswalker with 0 loyalty is put into graveyard (704.5i)
  - [x] Legendary rule (704.5j) — auto-keeps newest; player choice deferred to M7
  - [x] Token in a non-battlefield zone ceases to exist (704.5d)
  - [x] Aura attached to illegal object goes to graveyard (704.5m)
  - [x] Equipment/fortification attached illegally becomes unattached (704.5n)
  - [x] +1/+1 and -1/-1 counter annihilation (704.5q)
  - [x] Commander damage >= 21 causes loss (704.5u — Commander specific)
- [x] SBA integration with priority: SBAs checked every time any player would receive priority
- [x] Triggers generated by SBAs are collected and placed on stack after all SBAs finish

**Tests** (minimum):
- [x] Each SBA individually in isolation
- [ ] SBA chain: creature dies from SBA, death trigger produces token, no further SBAs (deferred to M7 — triggers have no effects yet)
- [x] SBA convergence: always terminates (property test via `test_sba_no_infinite_loop_on_repeated_sba`)
- [x] Legendary rule with 2+ copies: one removed (auto-choice; real player choice M7)
- [x] Counter annihilation: 3 +1/+1 and 2 -1/-1 → 1 +1/+1 remains
- [x] Multiple players at 0 life simultaneously: all lose simultaneously
- [x] SBA triggers go on stack in APNAP order after all SBAs finish (framework in place; M7 adds effects)

**Game Script Tasks**: *(deferred to M7 — see note at top of roadmap)*

**Acceptance Criteria**:
- All SBAs from CR 704.5 implemented and individually tested
- Fixed-point loop terminates for all tested states (property test)
- SBA check integrates correctly with priority system

**Dependencies**: M3 (SBAs reference the stack for trigger placement)

**Architecture doc references**: Section 3.4 (State-Based Actions)

---

### M5: The Layer System

**Goal**: Implement the continuous effect layer system (CR 613). This is the hardest milestone. Budget time accordingly and expect iteration.

**Deliverables**:
- [x] `ContinuousEffect` type with layer, sublayer, timestamp, duration, affected filter, modification
  - `crates/engine/src/state/continuous_effect.rs`: `EffectId`, `EffectLayer` (10 variants), `EffectDuration` (3), `EffectFilter` (10), `LayerModification` (21), `ContinuousEffect`
- [x] Layer application function: given all active continuous effects, calculate characteristics of any object
  - `crates/engine/src/rules/layers.rs`: `calculate_characteristics(state, object_id) -> Option<Characteristics>`
- [x] Timestamp system: effects get timestamps when they start; newer = later
- [x] Dependency detection per CR 613.8: effect A depends on effect B if B could change what A applies to or what A does
  - `depends_on()`: `SetTypeLine` depends on `AddSubtypes`/`AddCardTypes` (Blood Moon + Urborg)
- [x] Dependency resolution: apply dependents after their dependencies; circular dependencies fall back to timestamp
  - `toposort_with_timestamp_fallback()`: Kahn's algorithm with timestamp fallback for cycles
- [x] Duration tracking: "until end of turn", "as long as", "for as long as" — effects are removed when duration expires
  - `expire_end_of_turn_effects()`: called from `cleanup_actions` during Cleanup step
  - `WhileSourceOnBattlefield`: lazily evaluated in `is_effect_active`
- [x] Characteristic-defining abilities (CDAs) calculated in the appropriate layer
  - `is_cda: bool` flag; CDAs sort before non-CDAs within each layer
- [x] Copy effects (Layer 1): placeholder in `apply_layer_modification` — full CR 707 deferred to M7
- [x] Control-changing effects (Layer 2): `SetController` variant defined; controller lives on `GameObject` not `Characteristics` (handled separately from `calculate_characteristics`)
- [x] Type-changing effects (Layer 4): including interaction with Blood Moon style effects
- [x] P/T modifications (Layer 7a-d): CDAs, setting, +/-, switching; counter P/T also applied at Layer 7c

**Tests** (minimum — this milestone has the most tests):
- [x] Basic layer ordering: type change applies before P/T change
- [x] Timestamp ordering within layer: later timestamp wins
- [x] **Humility + Opalescence**: verify both cards' characteristics after full layer resolution
- [x] **Blood Moon + Urborg**: dependency in layer 4 — Blood Moon depends on Urborg (or vice versa depending on timestamp)
- [ ] Copy effect on a permanent with continuous effects — deferred to M7 with full Layer 1 implementation
- [ ] Control change via continuous effect; verify controller changes propagate — deferred (controller on GameObject)
- [x] "Until end of turn" effects removed during cleanup
- [x] CDA in layer 7a: Tarmogoyf power/toughness calculation
- [x] P/T switching (layer 7d) after other P/T effects
- [x] Removal of source: continuous effect from a permanent that leaves the battlefield
- [x] Multiple dependencies forming a chain (A depends on B depends on C)
- [x] Circular dependency: falls back to timestamp order
- [x] 28 tests total in `crates/engine/tests/layers.rs`; 261 total engine tests

**Game Script Tasks**: *(deferred to M7 — see note at top of roadmap)*

**Acceptance Criteria**:
- [x] All 20+ corner case tests pass (28 tests, all passing)
- [x] Layer system produces correct characteristics for every object in any test state
- [ ] Performance benchmark: <1ms for 50 continuous effects — not yet benchmarked

**Dependencies**: M1 (state model), M3 (effects reference stack for spell-based continuous effects)

**Architecture doc references**: Section 3.5 (The Layer System)

**Risk note**: This is the highest-risk milestone. The dependency system is subtle and the test cases may reveal architectural issues requiring refactoring of the effect representation. Budget extra time and plan for iteration.

---

### M6: Combat

**Goal**: Implement the complete combat system: attacker declaration, blocker declaration, damage assignment, and all combat-related mechanics.

**Deliverables**:
- [x] `CombatState` tracking: attackers, blockers, damage assignment orders (`state/combat.rs`)
- [x] Attacker declaration: legal attack targets (player or planeswalker), restrictions and requirements (`rules/combat.rs::handle_declare_attackers`)
- [x] Blocker declaration: legal blocks, damage assignment order (`handle_declare_blockers`, `handle_order_blockers`)
- [x] Combat damage assignment: lethal damage rule, player choice for ordering (`apply_combat_damage`)
- [x] First strike / double strike: extra combat damage step (`should_have_first_strike_step`, `deals_damage_in_step`)
- [x] Trample: excess damage to defending player/planeswalker
- [x] Deathtouch + trample interaction: 1 damage is lethal
- [ ] Damage prevention in combat — deferred to M8 (no prevention effects framework yet)
- [x] "Whenever this creature attacks/blocks" triggers (`TriggerEvent::SelfAttacks`, `SelfBlocks`)
- [ ] "Whenever this creature deals combat damage to a player" triggers — deferred to M7 (needs card effect framework for the ability body)
- [x] Commander damage tracking: combat damage from a commander increments the matrix (CR 903.10a)
- [x] Multiplayer combat: active player attacks one or more opponents; each opponent declares blockers

**Tests** (minimum):
- [x] Basic combat: 2/2 attacks, unblocked, defending player takes 2 (`test_510_unblocked_attacker_deals_damage_to_player`)
- [x] Blocked combat: 5/5 blocked by 1/1, no player damage (`test_509_blocked_attacker_no_player_damage`)
- [x] Mutual destruction: 3/3 attacks, blocked by 3/3, both die (`test_510_mutual_combat_damage_both_die`)
- [x] First strike: first striker kills blocker before blocker deals damage (`test_702_7_first_strike_kills_blocker_before_regular_damage`)
- [x] Double strike: deals damage in both steps (`test_702_4_double_strike_deals_in_both_steps`)
- [x] Trample: 5/5 with trample blocked by 2/2, 3 damage to player (`test_702_19_trample_excess_to_player`)
- [x] Deathtouch + trample: 1 to blocker (lethal), rest to player (`test_702_deathtouch_with_trample`)
- [x] Multiple blockers: damage assignment order, distribute lethal (`test_509_2_multiple_blockers_damage_order`)
- [x] Combat triggers fire at correct timing (`test_603_self_attacks_trigger_fires`)
- [x] Commander combat damage tracked in matrix (`test_903_10a_commander_damage_tracked`)
- [x] Multiplayer: player A attacks player B and player C simultaneously (`test_506_multiplayer_simultaneous_attacks`)

**Game Script Tasks**: *(deferred to M7 — see note at top of roadmap)*

**Acceptance Criteria**:
- [x] Full combat phase executes correctly for multiplayer
- [x] All combat keyword interactions tested
- [x] Commander damage tracking accurate

**Dependencies**: M4 (SBAs check lethal damage), M5 (continuous effects modify P/T and abilities)

**Architecture doc references**: Section 3.1 (Combat Phase in turn structure)

---

### M7: Card Definition Framework & First Cards

**Goal**: Build the card definition system, implement the first set of real cards, and build the game script replay harness so generated scripts become executable tests.

**Deliverables**:
- [x] `CardDefinition` struct and `AbilityDefinition` enum per architecture doc Section 3.7
- [x] `Effect` recursive enum with all primitives (DealDamage, GainLife, DrawCards, CreateToken, etc.)
- [x] Card definition loader: `CardRegistry::new(all_cards())` with `lookup()` by `CardId`
- [x] Keyword ability implementations (first batch):
  - [x] Flying / Reach
  - [x] First strike / Double strike
  - [x] Trample
  - [x] Deathtouch
  - [x] Lifelink
  - [x] Haste
  - [x] Vigilance
  - [x] Hexproof / Shroud
  - [x] Indestructible
  - [x] Flash
  - [x] Menace
- [x] Target type system: legal targets for "target creature", "target player", "target permanent", etc.
- [x] Mode selection for modal spells
- [x] **First 50 real card definitions**: hand-authored, focusing on Commander staples (Sol Ring, Command Tower, Lightning Greaves, Swords to Plowshares, Counterspell, Cultivate, etc.)
- [x] Test harness that loads a card definition and verifies its behavior in isolation (`tests/effects.rs`)
- [x] **Game script replay harness** (see `mtg-engine-game-scripts.md` Hook 2): `tests/script_replay.rs` — `replay_script()` feeds `Command`s to engine, asserts state at every checkpoint
- [x] **Script auto-discovery test** (see `mtg-engine-game-scripts.md` Hook 3): `tests/run_all_scripts.rs` — discovers and runs all `approved` scripts in `test-data/generated-scripts/`

**Game Script Tasks** *(all script generation happens here — schema was defined in M5)*:
- [x] Generate 3 baseline scripts: priority passing, play land, tap land for mana (`test-data/generated-scripts/baseline/`)
- [ ] Generate scripts for layer system corner cases from `mtg-engine-corner-cases.md` — deferred to M8 (engine needs replacement effects for some cases)
- [ ] Generate scripts for combat corner cases — deferred to M8
- [x] Generate scripts for first cards' individual behaviors: Lightning Bolt, Counterspell, Sol Ring, Swords to Plowshares (`test-data/generated-scripts/stack/`)
- [x] All scripts human-reviewed and marked `approved`; all 7 pass through replay harness

**Tests** (minimum):
- [x] Each keyword ability in a combat or game scenario (`tests/keywords.rs`)
- [x] Sol Ring: resolves as permanent onto battlefield (script 003)
- [x] Swords to Plowshares: exile target creature, controller gains life equal to power (script 004)
- [x] Counterspell: counter target spell (script 002)
- [x] Lightning Bolt: 3 damage to any target (script 001)
- [ ] Cultivate: search library for two basic lands — deferred (SearchLibrary not wired through harness yet)
- [ ] Modal spell: choose one or more modes — deferred (Choices effect not fully wired)
- [x] Card definition load/save round-trip (`tests/script_schema.rs`)
- [x] Replay harness processes a simple script end-to-end (`test_harness_end_to_end_priority_passes`)
- [x] Script auto-discovery finds and runs all approved scripts (`run_all_approved_scripts`)

**Acceptance Criteria**:
- 50 real cards implemented and individually tested
- All keyword abilities functional
- Card definition system is extensible (adding a new card doesn't require engine changes)
- Replay harness runs all approved scripts generated in M7; all pass
- Any script failures investigated and resolved (engine fix or script correction)

**Dependencies**: M3-M6 (the card framework exercises all prior systems)

**Architecture doc references**: Section 3.7 (Card Definition Runtime)

---

### M8: Replacement & Prevention Effects

**Goal**: Implement replacement effects and prevention effects, which modify events as they happen rather than triggering afterward.

**Deliverables**:
- [x] Replacement effect framework: intercept an event, apply modification, continue with modified event
- [x] Self-replacement effects: apply before other replacement effects (CR 614.15)
- [x] Player choice when multiple replacement effects apply to the same event
- [x] Loop prevention: a replacement effect can modify a given event at most once (CR 614.5)
- [x] Prevention effects: prevent N damage, prevent all damage, etc.
- [x] Prevention/replacement interaction per CR 616
- [x] "If ~ would die" replacement effects (critical for Commander zone-change choice)
- [x] "If a player would draw" replacement effects (e.g., Notion Thief)
- [x] "Enters the battlefield" replacement effects (e.g., "enters tapped")

**Tests** (minimum):
- [x] Simple replacement: "If you would gain life, draw that many cards instead"
- [x] Multiple replacement effects: player chooses order of application
- [x] Self-replacement: applies first regardless of player choice
- [x] Loop prevention: same effect can't apply twice to same event
- [x] Prevention shield: "prevent the next 3 damage" then take 5 damage → 2 gets through
- [x] Replacement + prevention interaction: which applies first (player's choice per CR 616)
- [x] Commander zone-change replacement: commander would die → choose command zone or graveyard

**Game Script Tasks**:
- [x] Generate scripts for replacement effect corner cases from `mtg-engine-corner-cases.md` (cases 16-19, 28, 33) and add to `test-data/generated-scripts/replacement/`
- [x] Generate scripts for prevention effects and replacement + prevention interaction ordering
- [x] Cross-validate and human-review; run through replay harness

**Acceptance Criteria** (all met — M8 COMPLETE, 395 tests passing):
- [x] Replacement effects integrate cleanly with existing event system
- [x] Commander zone-change choice works correctly
- [x] No infinite loops possible in replacement effect chains
- [x] Replacement effect game scripts pass through replay harness

**Dependencies**: M4 (SBAs generate events that can be replaced), M5 (continuous effects can create replacement effects)

**Architecture doc references**: Section 3.6 (Replacement Effects)

---

### M9: Commander Rules Integration

**Goal**: Implement all Commander-specific rules as a cohesive layer on top of the core engine.

**Deliverables**:
- [x] Commander format enforcement:
  - [x] 100-card singleton deck validation
  - [x] Color identity validation (mana cost + oracle text symbols, CR 903.4)
  - [x] Banned list checking (loaded from card DB)
- [x] Command zone mechanics:
  - [x] Casting commander from command zone
  - [x] Commander tax: additional {2} for each previous cast from command zone
  - [x] Commander tax tracks separately per commander (for partners)
- [x] Commander replacement effects:
  - [x] "If your commander would go to graveyard/exile from anywhere, you may put it in the command zone instead" (SBA for graveyard/exile per CR 903.9a; replacement for hand/library per CR 903.9b; player choice via `CommanderZoneReturnChoiceRequired` event)
  - [x] Tax increments on cast, not on zone change
- [x] Commander damage:
  - [x] SBA: player who has received 21+ combat damage from a single commander loses
  - [x] Tracking across zone changes (the commander is the same card even with new ObjectId)
- [x] Partner mechanics: two commanders, shared color identity, separate tax
- [x] Companion (if in scope): deck restriction validation, companion casting from sideboard-equivalent
- [x] Mulligan: Commander-specific free mulligan, then London mulligan
- [x] Starting life: 40

**Additional deliverable — rewind prerequisites**:
- [x] `GameEvent::reveals_hidden_info() -> bool` method: returns `true` for any event that reveals or commits to hidden information (library draws, scry peeks, face-down reveals). Used by the network layer in M10 to identify safe rewind checkpoints. No engine logic changes — purely a classification method on the existing enum.

**Tests** (minimum):
- [x] Deck validation: reject 99-card deck, reject off-color-identity cards, reject banned cards
- [x] Cast commander: first cast costs printed cost, second costs +2, third costs +4
- [x] Partner commanders: each tracked separately for tax
- [x] Commander dies: player chooses command zone or graveyard
- [x] Commander exiled: player chooses command zone or exile
- [x] Commander damage: 21 combat damage from one commander → SBA loss
- [x] Commander damage: 10 from commander A + 11 from commander B → no loss (tracked separately)
- [x] Commander damage from a copy of a commander: does NOT count
- [x] Free mulligan then London mulligan sequence
- [x] 4-player game start: all commander-specific setup correct
- [x] 6-player game tests:
  - [x] Priority rotation with 6 players (all 6 must pass for stack resolution)
  - [x] Combat with 5 defending players declaring blockers independently
  - [x] APNAP trigger ordering with 6 players
  - [x] Turn advancement skipping eliminated players in 6-player game
  - [x] Concession mid-game with 6 players (priority and turn order adjust correctly)
  - [x] `GameStateBuilder::six_player()` convenience method

**Game Script Tasks**:
- [x] Generate scripts for Commander corner cases from `mtg-engine-corner-cases.md` (cases 26, 27, 28) and add to `test-data/generated-scripts/commander/`
- [x] Generate scripts for full Commander game setup: mulligan sequence, first few turns with commander casting and tax; partner commander interactions
- [x] Cross-validate and human-review; run through replay harness

**Acceptance Criteria**:
- A full 4-player Commander game can be played programmatically (via test commands) from game start through win/loss conditions
- 6-player test coverage: priority rotation, combat with 5 defenders, APNAP ordering, turn advancement
- All Commander-specific rules from CR 903 tested
- All Commander game scripts pass through replay harness
- This milestone marks **Engine Core Complete**

**Dependencies**: M1-M8 (all core systems)

**Architecture doc references**: Section 7 (Commander-Specific Design)

---

### M9.4: Engine Correctness & Core Mechanics

**Goal**: Close all correctness gaps discovered during the Engine Core Complete audit. Fix existing card definition simplifications, close partial test coverage, and implement core mechanics needed for Commander play. This milestone gates the Engine Core Complete checkpoint.

**Audit reference**: `docs/mtg-engine-corner-case-audit.md` — living correctness ledger tracking all 35 corner cases and 12 card definition gaps.

**Phase 1 — Card Definition Correctness** (fix what's broken in the existing 54 cards):
- [x] Equipment static ability granting: Lightning Greaves (shroud + haste), Swiftfoot Boots (hexproof + haste) via continuous effects
- [x] Goad mechanic: replace `DrawCards(0)` placeholder in Alela with real `Effect::Goad`
- [x] Scry: implement `Effect::Scry` (Read the Bones)
- [x] Modal color choice: Dimir Guildgate `{T}: Add {U} or {B}` (not colorless)
- [x] "Can't be blocked" evasion: Rogue's Passage activated ability
- [x] "No maximum hand size": Thought Vessel, Reliquary Tower continuous effect
- [x] Optional search: Path to Exile controller-may-search (not unconditional)
- [x] Rhystic Study: opponent payment choice interaction
- [x] Rest in Peace ETB: exile all cards from all graveyards on entry
- [x] Leyline of the Void: opening hand rule (begin game on battlefield)
- [x] Darksteel Colossus: shuffle into library (not just redirect to zone)
- [x] Alela trigger scoping: opponent-turn-only + creature-type filter

**Phase 2 — Partial Test Coverage** (close the 9 PARTIAL corner cases):
- [x] CC#6: Humility + Magus of the Moon non-dependency (different layers confirmed)
- [x] CC#7: Opalescence + Parallax Wave zone-change interaction
- [x] CC#9: Indestructible + deathtouch combined (survives SBA 704.5h)
- [x] CC#10: Legendary rule simultaneous ETBs fire before removal
- [x] CC#20: First strike + double strike combined blocking scenario
- [x] CC#22: Hexproof does NOT block non-targeted global effects
- [x] CC#24: Token die-trigger fires before SBA removes from graveyard
- [x] CC#31: Aura falls off when animation effect ends (type-change-induced)
- [x] CC#33: Sylvan Library "cards drawn this turn" tracking

**Phase 3 — Core Mechanics Expansion** (new engine features):
- [x] Protection keyword (DEBT: Damage, Enchanting, Blocking, Targeting) — CR 702.16
- [x] Layer 1 copy effects (copiable values, Clone chain) — CR 707.2, 707.3
- [x] Storm keyword + spell copying on stack — CR 702.40, 707.10
- [x] Cascade keyword + split card mana value — CR 702.84, 708.4
- [x] Trigger doubling (Panharmonicon-style modifier) — CR 603.2
- [x] Infinite loop detection (mandatory loop = draw) — CR 726, 104.4b

**Phase 4 — Gap Tests** (test-only items for existing engine capabilities):
- [x] CC#4: Yixlid Jailer + Anger (layer 6 removal of graveyard static ability)
- [x] CC#23: Flicker + object identity (kill spell fizzles, no dies-trigger)

**Tests** (minimum):
- All 9 PARTIAL cases have dedicated named tests
- All Phase 3 mechanics have unit tests citing CR sections
- All 12 card definition fixes verified by updated card-specific tests
- Corner case audit re-run shows 0 GAPs, 0 PARTIALs for implemented mechanics

**Acceptance Criteria**:
- All 54 existing card definitions behave correctly (no simplifications or no-op placeholders)
- Protection, copy effects, storm, cascade, trigger-doubling, infinite loop detection implemented
- All 35 corner cases are COVERED (except 3 deferred: phasing, morph, mutate)
- Corner case audit doc updated with final status
- All tests pass, clippy clean, fmt clean

**Dependencies**: M9 (all core systems)

---

### ═══════════ ENGINE CORE COMPLETE ═══════════

At this point, the engine can run a complete Commander game programmatically. All rules are implemented and tested. No UI, no network — but any game scenario can be constructed and played via test code.

**Checkpoint validation**:
- [ ] M9.4 acceptance criteria met (card def correctness, core mechanics, corner case coverage)
- [x] Property tests pass: 50+ invariants validated via fuzzing
- [x] All golden tests pass (at least 5 hand-authored full game replays)
- [x] All approved game scripts pass through replay harness (~100+ scripts)
- [ ] All corner case tests from `mtg-engine-corner-case-audit.md` pass (0 GAP, 0 PARTIAL for implemented mechanics)
- [x] Performance benchmarks meet targets
- [x] 6-player game tests pass (priority, combat, APNAP, turn order, concession)
- [x] Performance benchmark: 4-player vs 6-player Commander (priority cycle time, SBA check, full turn processing)

---

### M9.5: Game State Stepper (Developer Replay Viewer)

**Goal**: Build a visual developer tool that loads game scripts and lets the developer step through them action-by-action, watching the full game state at every point. This validates engine correctness with human eyes before networking adds complexity, and produces reusable Svelte components for the main Tauri app at M11.

See `docs/mtg-engine-replay-viewer.md` for full architecture design.

**Architecture**: Rust HTTP server (axum) + Svelte 5 frontend, served as a local web app. Lives in `tools/replay-viewer/` as a standalone workspace member. The engine is linked as a path dependency — no engine changes required.

**Deliverables**:

*Phase 1 — Backend + Minimal UI:*
- [ ] Rust HTTP server (axum) loads a game script, replays all commands, stores `Vec<StepSnapshot>` in memory (cheap via im-rs structural sharing)
- [ ] API endpoints: `GET /api/scripts` (list available), `GET /api/session` (metadata + step count), `GET /api/step/:n` (command, events, view-model state), `POST /api/load` (switch script)
- [ ] View model serialization: `GameState` → UI-friendly JSON (zones, players, turn info, combat)
- [ ] Svelte app with `StepControls` (prev/next/first/last) + basic `StateView` (formatted state dump)
- [ ] Keyboard navigation: arrow keys (prev/next command), Shift+arrow (prev/next phase), Home/End

*Phase 2 — Rich Visualization:*
- [ ] `PlayerPanel`: life, mana pool, poison counters, commander damage
- [ ] `ZoneBattlefield`: permanent grid with tapped/counter/damage indicators
- [ ] `ZoneStack`: ordered stack items with controller and targets
- [ ] `ZoneHand`, `ZoneGraveyard`, `ZoneExile`: card lists
- [ ] `PhaseIndicator`: visual turn/phase/step bar highlighting current position
- [ ] `EventTimeline`: scrollable event list; per-command default view, expandable to per-event drill-down

*Phase 3 — Polish & Script Browser:*
- [ ] `ScriptPicker`: browse `test-data/generated-scripts/` tree, select script to load
- [ ] `CombatView`: attacker → blocker arrows, damage assignment visualization
- [ ] `CardDisplay`: oracle text, types, keywords, P/T, counters
- [ ] State diff highlighting: visual indicator of what changed between consecutive steps
- [ ] Assertion result display: pass/fail badges on steps with `assert_state` actions

**Shared component strategy**: All Svelte components in `frontend/src/lib/` accept data via props, not internal fetch. At M11, the Tauri app imports the same components — only the data source changes (`fetch('/api/...')` → Tauri `invoke()`).

**Tests**:
- [ ] Backend: axum endpoints return correct JSON for a known script (unit test)
- [ ] Replay: stepping through Lightning Bolt script shows p2 life 40 → 37 at correct step
- [ ] Replay: combat script shows attackers/blockers/damage correctly
- [ ] Frontend: Svelte components render without errors (Vite build succeeds)
- [ ] Integration: `cargo run -p replay-viewer -- --script <path>` serves working app at localhost

**Acceptance Criteria**:
- [ ] Can load any approved game script and step through it visually
- [ ] Per-command stepping with event expansion works
- [ ] All zones rendered with correct contents at each step
- [ ] Keyboard navigation functional
- [ ] At least one complex script (combat or stack interaction) validated visually
- [ ] Svelte components importable from Tauri app (verified with test import)

**Dependencies**: M9 (engine core complete — all subsystems available to visualize)

**Architecture doc references**: `docs/mtg-engine-replay-viewer.md`

---

### M10: Networking Layer (Centralized Server)

**Goal**: Implement a lightweight centralized WebSocket game server. One server instance (runnable on a ~$5-10/mo VPS) hosts games for a trusted playgroup. The server runs the engine authoritatively, filters hidden information per player, and broadcasts events. P2P distributed verification is preserved in `docs/mtg-engine-network-security.md` as a documented future upgrade path.

> **Architecture decision (2026-02-23)**: P2P mesh + Mental Poker deferred. A single player
> with bad internet stalls the whole table in P2P; Mental Poker adds significant complexity
> for no benefit in a trusted playgroup. Centralized server is simpler, cheaper, solves
> the timing/reconnection problems cleanly. See `memory/decisions.md`.

**Crate**: `crates/server/` — standalone binary, depends on `crates/engine` as a library. No engine changes required.

**Deliverables**:
- [ ] WebSocket server (tokio + axum) accepting player connections
- [ ] Room manager: create game (returns room code), join game by code, 2-6 player slots
- [ ] One engine instance per active game room, running authoritatively on the server
- [ ] Command ingestion: accept `Command` from the acting player only; reject out-of-turn commands
- [ ] Extend `GameEvent` with `private_to() -> Option<PlayerId>` (engine crate, small addition alongside existing `reveals_hidden_info()`) to identify which player a private event belongs to
- [ ] Hidden information filtering: broadcast public events to all clients; private events (draw, scry peek, hand reveal) sent only to the player returned by `private_to()`; all others receive a redacted version
- [ ] Message protocol: `ClientMessage` (command) and `ServerMessage` (event, state sync, error) — serde JSON for simplicity, MessagePack optional upgrade
- [ ] Reconnection: reconnecting client receives full public state dump + their own private state (hand, known library cards)
- [ ] **State history ring buffer**: server retains last N `GameState` snapshots (O(1) via im-rs structural sharing); keyed by turn + step + priority sequence number
- [ ] **Safe checkpoint identification**: use `GameEvent::reveals_hidden_info()` (from M9) to mark snapshots before hidden-info events as safe rewind targets
- [ ] **Rewind**: any player proposes rewind to a named checkpoint; requires unanimous acceptance; server restores snapshot and rebroadcasts state
- [ ] **Pause**: any player sends Pause; server freezes until all players send Resume
- [ ] Graceful disconnect: game pauses on disconnect, others notified; rejoin window before forfeit

**Tests** (minimum):
- [ ] Server starts, 4 clients connect to a room, game begins
- [ ] Command round-trip: acting client sends command, server processes, all clients receive correct events
- [ ] Hidden info: draw event sent only to drawing player; all others see `PlayerDrewCard { count: 1 }` only
- [ ] Out-of-turn command rejected by server
- [ ] Reconnect: client disconnects, rejoins, receives correct public + private state sync
- [ ] Pause: client sends Pause, server freezes, no further Commands processed
- [ ] Resume: all clients send Resume, game continues
- [ ] Rewind accepted: all clients accept, state restored and rebroadcast
- [ ] Rewind rejected: one client dissents, game continues from current state
- [ ] 6-player game completes a full turn cycle without errors

**Acceptance Criteria**:
- 4-6 player Commander game playable over LAN/internet via WebSocket clients
- Hidden information never leaked to wrong client
- Reconnection restores correct state
- Runs as a single statically-linked binary with no external dependencies

**Dependencies**: M9 (engine core complete)

---

### M10.5: P2P Distributed Verification (Deferred — Future Upgrade)

**Goal**: Upgrade from centralized server to peer-to-peer distributed verification for untrusted play — no trusted server required, all peers run the engine independently, hidden information protected via Mental Poker.

> Full design preserved in `docs/mtg-engine-network-security.md` (Tiers 2 and 3).
> This milestone is intentionally unscheduled. Revisit after M11 if there is demand
> for trustless play beyond the trusted playgroup model.

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
- [ ] **Rewind UI**: show a timeline of recent safe checkpoints (turn/step labels); any player can propose a rewind by clicking a checkpoint; all players see a consent prompt; unanimous accept → state rolls back; any decline → proposal dismissed
- [ ] **"Pause for rules discussion" button**: one click sends the Pause command to all peers; all players see "Game paused — discussing rules" overlay with a Resume button; game is frozen until all players resume
- [ ] **Manual state adjustment mode** (active only while paused): players can collaboratively edit the current game state — adjust life totals, move permanents between zones, add/remove counters. Changes are applied locally and broadcast as a proposed new state hash. All peers must accept before the adjusted state becomes official and automation resumes.
- [ ] Clear visual indicator when a proposed rewind would cross a hidden-information boundary (e.g., "Note: a card was drawn after this checkpoint — rewinding requires good faith")

**Tests**: UI tests are manual at this stage. Checklist:
- [ ] Launch app, connect to local host, see game state
- [ ] Cast a spell from hand by clicking it
- [ ] Pass priority
- [ ] See stack update when opponents cast spells
- [ ] See battlefield update when permanents enter/leave
- [ ] Pause button freezes the game for all connected players
- [ ] Rewind proposal appears on all peers' screens; accepting rolls state back; declining dismisses
- [ ] Manual life total adjustment while paused is reflected on all peers after unanimous accept

**Acceptance Criteria**:
- A human player can play a simplified Commander game through the UI (against programmatic opponents or other humans on localhost)
- All game information is visible and actionable

**Dependencies**: M10 (networking for multi-window testing), M0 (Tauri scaffold)

---

### M12: Card Definition Pipeline (Bulk Generation)

**Goal**: Scale from 50 hand-authored cards to the full Commander card pool using a scripted
conversion pipeline. Every card that can appear in a game must have a `CardDefinition` before
that game can start — no mid-game discovery, no graceful degradation during play.

**Core constraint**: The deck builder enforces that all cards in all decks have a `CardDefinition`
before a game begins. This is non-negotiable: the rewind/replay/pause system relies on a
complete and accurate state history from turn 1. A card whose abilities silently never fired
produces a corrupted history that cannot be rewound to correctly.

**Pipeline architecture** (three stages):

1. **Scripted converter** (handles ~70-80% of cards, zero LLM calls):
   - Structured Scryfall fields map 1:1: mana cost, P/T, types, subtypes, color identity
   - `keywords` array from Scryfall maps directly to `KeywordAbility` variants (Flying, Trample, etc.)
   - Growing pattern library covers common oracle text templates:
     - `"{T}: Add {X}."` → `ActivatedAbility { cost: Tap, effect: AddMana }`
     - `"When ~ enters the battlefield, [effect]."` → `Triggered { WhenEntersBattlefield }`
     - `"Whenever ~ deals combat damage to a player, [effect]."` → triggered ability
     - `"At the beginning of your upkeep, [effect]."` → `AtBeginningOfUpkeep`
     - etc.
   - Each new pattern added to the library handles all past and future cards with matching text

2. **LLM-assisted fallback** (handles unmatched tail):
   - Cards that don't match any pattern are sent to an LLM with oracle text + rulings
   - LLM generates a candidate `CardDefinition`
   - Candidate is validated against a generated game script
   - On success: **extract the pattern and add it to the library** so the next similar card
     is handled by stage 1 automatically
   - On failure: flag for human review (expected to be rare)

3. **Human review** (edge cases only):
   - Cards whose LLM-generated definition fails validation
   - Cards with unique mechanics that appear on 1-2 cards ever (e.g., Chaos Orb)
   - Output is either a corrected definition or a new pattern added to stage 1

**Deliverables**:
- [ ] Scripted converter in `card-pipeline` crate: structured Scryfall fields → `CardDefinition` JSON
- [ ] Pattern library with initial 50+ patterns covering common ability templates
- [ ] LLM fallback integration: sends unmatched cards to Claude, extracts new patterns from successes
- [ ] Validation harness: each generated definition runs against a game script
- [ ] Deck builder enforcement: reject game start if any card lacks a `CardDefinition`
- [ ] Unimplemented card UX: deck builder shows which cards aren't supported with clear messaging
- [ ] Priority queue: cards ordered by EDHREC popularity (Commander staples first)
- [ ] DB storage: pipeline writes definitions to `card_definitions` SQLite table; engine loads at startup
- [ ] Coverage report: percentage of Commander-legal cards with validated definitions
- [ ] New set workflow: documented process for processing a new set before it becomes playable

**Tests**:
- [ ] Scripted converter reproduces the original 50 hand-authored definitions exactly (regression)
- [ ] Pattern library handles the 20 most common oracle text templates correctly
- [ ] Deck builder rejects a game containing a card with no `CardDefinition`
- [ ] Deck builder allows a game where all cards have definitions
- [ ] Generated definitions for 500+ cards pass individual game script validation
- [ ] New game scripts for card interactions pass through replay harness

**Acceptance Criteria**:
- 500+ cards with validated definitions stored in the DB
- Deck builder enforces the no-undefined-cards constraint
- Pattern library handles ≥70% of cards without LLM involvement
- Documented new-set workflow: pipeline runs offline before players can use new cards
- Failure rate for LLM fallback stage <10% (definitions that fail game script validation)

**Dependencies**: M7 (card definition framework), M9 (engine can execute cards), M11 (deck builder UI)

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
| Card definition pipeline pattern library has low coverage | LLM fallback invoked too frequently; slow bulk generation | Medium | Prioritize the 50 most common oracle text templates first — these cover the majority of Commander staples; coverage grows monotonically as patterns are added |
| Unimplemented card blocks a popular deck | Players can't start games until pipeline catches up | Medium | Process cards in EDHREC popularity order; most-played Commander cards are implemented first; clear UX tells players which cards are unsupported |
| Networking introduces non-determinism | State divergence bugs in M10+ | Medium | Tier 1 state hashing from M3 onward catches non-determinism early; `im::OrdMap` ensures deterministic iteration |
| Mental Poker latency unacceptable | Library search operations too slow for gameplay | Low | Protocol designed for <1s search latency; acceptable for turn-based game; fallback to trusted host mode |
| Scryfall API changes or terms change | Card data pipeline breaks | Low | Vendor-lock only on data format, not API; cache aggressively |
| Performance bottleneck in layer recalculation | Unplayable with complex board states | Medium | Benchmark from M5; incremental recalculation if needed |
| Scope creep from Commander complexity | Milestones slip | High | Strict MVP: basic Commander first, variants and edge cases in post-alpha |
| Game script schema changes break existing scripts | Accumulated scripts need rework | Medium | Version the schema; write migration tooling; prefer additive changes |
| Engine and scripts disagree on correct behavior | Ambiguous which is wrong | Medium | Always trace back to CR text; when in doubt, check Forge/XMage; human judge review |
| Manual mode state editor produces an invalid GameState | Engine rejects re-entry; game stuck | Low | Validate the proposed state against known invariants before broadcasting; surface specific errors to players so they can correct |
| Rewind proposed across hidden-info boundary | Player who drew a card knows its identity after rewind | Low | Accepted risk for trusted-player use case; engine surfaces a clear warning but does not block; honour system |

---

## Appendix: Milestone Dependency Graph

```
M0 ──→ M1 ──→ M2 ──→ M3 ──→ M4 ──→ M5 ──→ M6 ──→ M7 ──→ M8 ──→ M9
 │                    │Tier1│      │      │      │              │
 │                    │hash │      │      │      │              ├──→ M9.5
 │                    └──┬──┘      ▼      ▼      ▼              │    (stepper)
 │                       │       scripts scripts scripts    scripts  │
 │                       │       (base)  (layer) (combat)   (cmdr)   │
 │                       │                        (replay+           │
 │                       │                         cards)            │
 │                       │                           │               │
 └────────────────────────────────────────────────→ M11 ←── components
                                                     │          │
                                                     ▼       M10 ←── Tier1
                                        M7 ──→ M12  M13       │
                                                │    │  ←── M10.5
                                                ▼    ▼
                                                M14 ←┘
                                                  │
                                                  ▼
                                                 M15
```

Engine milestones (M0-M9) are strictly sequential — each builds on the prior. Tier 1 state hashing is implemented during M3 and is a prerequisite for M10 distributed verification. The `GameScript` schema is defined in M5 so it evolves under the compiler; all script generation happens in M7 and M8-M9 when the replay harness exists to run them immediately. M9.5 (Game State Stepper) validates the complete engine visually and produces Svelte components reused in M11 Tauri app. M10.5 (Mental Poker) depends on M10 and adds cryptographic hidden information protection. UI and networking (M10-M14) can partially overlap once the engine core is complete. M12 (card pipeline) can run in parallel with UI work since it's primarily a data generation effort.

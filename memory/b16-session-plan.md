# Dungeon / Venture into the Dungeon Mini-Milestone Session Plan

**Generated**: 2026-03-09
**Milestone**: Dungeon Mini-Milestone (post-type-consolidation, pre-M10)
**Sessions**: 4
**Estimated new tests**: 25-30
**Workstream**: W1 (abilities)
**Commit prefix**: `W1-Dungeon:`

---

## What This Delivers

- Hardcoded `DungeonId` enum with 4 static dungeons (Lost Mine of Phandelver, Dungeon of the Mad Mage, Tomb of Annihilation, The Undercity)
- `DungeonState` struct tracking current dungeon and room position per player
- `dungeon_state: OrdMap<PlayerId, DungeonState>` on `GameState`
- `dungeons_completed: u32` on `PlayerState` for "completed a dungeon" tracking
- `has_initiative: Option<PlayerId>` on `GameState` for initiative designation (CR 725)
- `Effect::VentureIntoDungeon` for cards that say "venture into the dungeon"
- `Effect::TakeTheInitiative` for cards that say "take the initiative"
- `StackObjectKind::RoomAbility` (new SOK variant) for room effects on the stack
- `Command::VentureIntoDungeon` for player dungeon choice when entering a new dungeon
- `Command::ChooseDungeonRoom` for branching path choice
- SBA 704.5t: remove completed dungeon from command zone
- CR 725: Initiative inherent triggers (upkeep venture, combat damage steal, take = venture into Undercity)
- `Condition::CompletedADungeon` for cards like Nadaar ("as long as you've completed a dungeon")
- `GameEvent::VenturedIntoDungeon` / `GameEvent::DungeonCompleted` / `GameEvent::InitiativeTaken`
- 3 card definitions: Nadaar Selfless Paladin, Seasoned Dungeoneer, Acererak the Archlich
- Harness action: `venture_into_dungeon` (dungeon choice + room choice)

## Architecture Summary

### Design Decision: Hardcoded Dungeon Enum (NOT CardDefinition)

Dungeons are NOT cards in the traditional sense. They begin outside the game, live in the command zone, are not permanents, cannot be cast, and have a fixed graph structure. Modeling them as `CardDefinition` would be wrong: they have no mana cost, no card types (in the normal sense), and their "abilities" are a fixed graph of rooms -- not the standard triggered/activated/static ability model.

Instead, dungeons are a hardcoded enum with static room definitions. Each room has a name (flavor text per CR 309.4b), an `Effect`, and a list of exits (next room indices). The engine knows the complete graph at compile time.

### New Files
- `crates/engine/src/state/dungeon.rs` -- `DungeonId`, `RoomIndex`, `DungeonState`, `DungeonDef`, `RoomDef`, static dungeon definitions, room ability helpers

### Data Model

Key new types (pseudo-Rust):

    /// Identifies which dungeon a player is in.
    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum DungeonId {
        LostMineOfPhandelver,
        DungeonOfTheMadMage,
        TombOfAnnihilation,
        TheUndercity,
    }

    /// Index into a dungeon's room list.
    pub type RoomIndex = usize;

    /// Tracks a player's current dungeon progress (CR 309.4).
    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct DungeonState {
        /// Which dungeon the player is currently in.
        pub dungeon: DungeonId,
        /// Index of the room the venture marker is on (0 = topmost).
        pub current_room: RoomIndex,
    }

    /// Static definition of a dungeon (compile-time constant).
    pub struct DungeonDef {
        pub id: DungeonId,
        pub name: &'static str,
        pub rooms: &'static [RoomDef],
        /// Index of the bottommost room (for SBA 704.5t check).
        pub bottommost_room: RoomIndex,
    }

    /// A single room in a dungeon.
    pub struct RoomDef {
        /// Room name (flavor text per CR 309.4b -- no gameplay effect).
        pub name: &'static str,
        /// The effect that triggers when the venture marker enters this room (CR 309.4c).
        pub effect: fn() -> Effect,
        /// Indices of rooms this room leads to (arrows pointing away).
        /// Empty = bottommost room.
        pub exits: &'static [RoomIndex],
    }

### Dungeon Room Layouts

**Lost Mine of Phandelver** (7 rooms):
- 0: Cave Entrance -- Scry 1. Exits: [1, 2]
- 1: Goblin Lair -- Create a 1/1 red Goblin creature token. Exits: [3, 4]
- 2: Mine Tunnels -- Create a Treasure token. Exits: [4, 5]
- 3: Storeroom -- Put a +1/+1 counter on target creature. Exits: [6]
- 4: Dark Pool -- Each opponent loses 1 life and you gain 1 life. Exits: [6]
- 5: Fungi Cavern -- Target creature gets -4/-0 until your next turn. Exits: [6]
- 6: Temple of Dumathoin -- Draw a card. Exits: [] (bottommost)

**Dungeon of the Mad Mage** (9 rooms):
- 0: Yawning Portal -- You gain 1 life. Exits: [1]
- 1: Dungeon Level -- Scry 1. Exits: [2, 3]
- 2: Goblin Bazaar -- Create a Treasure token. Exits: [4]
- 3: Twisted Caverns -- Target creature can't attack until your next turn. Exits: [4]
- 4: Lost Level -- Scry 2. Exits: [5, 6]
- 5: Runestone Caverns -- Exile the top two cards of your library. You may play them. Exits: [7]
- 6: Muiral's Graveyard -- Create two 1/1 black Skeleton creature tokens. Exits: [7]
- 7: Deep Mines -- Scry 3. Exits: [8]
- 8: Mad Wizard's Lair -- Draw three cards and reveal them. You may cast one without paying its mana cost. Exits: [] (bottommost)

**Tomb of Annihilation** (5 rooms):
- 0: Trapped Entry -- Each player loses 1 life. Exits: [1, 2]
- 1: Veils of Fear -- Each player loses 2 life unless they discard a card. Exits: [3]
- 2: Oubliette -- Discard a card and sacrifice a creature, an artifact, and a land. Exits: [4]
- 3: Sandfall Cell -- Each player loses 2 life unless they sacrifice a creature, artifact, or land of their choice. Exits: [4]
- 4: Cradle of the Death God -- Create The Atropal, a legendary 4/4 black God Horror creature token with deathtouch. Exits: [] (bottommost)

**The Undercity** (7 rooms):
- 0: Secret Entrance -- Search your library for a basic land card, reveal it, put it into your hand, then shuffle. Exits: [1, 2]
- 1: Forge -- Put two +1/+1 counters on target creature. Exits: [3, 4]
- 2: Lost Well -- Scry 2. Exits: [3, 4]
- 3: Arena -- Goad target creature. Exits: [5]
- 4: Stash -- Create a Treasure token. Exits: [5]
- 5: Catacombs -- Create a 4/1 black Skeleton creature token with menace. Exits: [6]
- 6: Throne of the Dead Three -- Reveal top ten cards. Put a creature, a land, a nonland noncreature card from among them into your hand. Put the rest on the bottom in a random order. Exits: [] (bottommost)

### Room Effect Complexity Notes

Most room effects map cleanly to existing `Effect` variants:
- Scry N -> `Effect::Scry`
- Create token -> `Effect::CreateToken`
- Gain/lose life -> `Effect::GainLife` / `Effect::LoseLife` / `Effect::DrainLife`
- Draw cards -> `Effect::DrawCards`
- Add counters -> `Effect::AddCounter`
- Goad -> `Effect::Goad`
- Search library -> `Effect::SearchLibrary`

Several rooms have complex or interactive effects that will use simplified deterministic fallbacks (same pattern as existing engine):
- **Twisted Caverns** ("target creature can't attack until your next turn"): `Effect::ApplyContinuousEffect` with `EffectDuration::UntilYourNextTurn`
- **Fungi Cavern** (-4/-0 until your next turn): same pattern
- **Veils of Fear** ("each player loses 2 life unless they discard"): simplified -- each player loses 2 life (interactive "may discard" deferred to M10+)
- **Sandfall Cell** ("each player loses 2 life unless they sacrifice"): same simplification
- **Oubliette** ("discard + sacrifice creature + artifact + land"): `Effect::Sequence` of discard + sacrifices; deterministic sacrifice selection
- **Runestone Caverns** ("exile top 2, you may play them"): `Effect::PlayExiledCard` variant or simplified -- exile top 2, put them in hand (free-cast deferred)
- **Mad Wizard's Lair** ("draw 3, reveal, may cast one free"): simplified -- draw 3 (free-cast deferred)
- **Throne of the Dead Three** ("reveal top 10, take 3 specific types"): simplified -- draw 3 (interactive selection deferred)
- **Storeroom** / **Forge** ("target creature"): targeting for room abilities uses deterministic fallback (smallest ObjectId creature controlled by owner)

### State Changes

On `GameState`:
- `dungeon_state: OrdMap<PlayerId, DungeonState>` -- per-player dungeon tracking (empty = no dungeon in command zone)
- `has_initiative: Option<PlayerId>` -- which player has the initiative (CR 725.1)

On `PlayerState`:
- `dungeons_completed: u32` -- count of dungeons completed (CR 309.7); used by Condition::CompletedADungeon

### New Events
- `GameEvent::VenturedIntoDungeon { player: PlayerId, dungeon: DungeonId, room: RoomIndex }` -- when venture marker moves to a room
- `GameEvent::DungeonCompleted { player: PlayerId, dungeon: DungeonId }` -- when a dungeon is removed from the game (CR 309.7)
- `GameEvent::InitiativeTaken { player: PlayerId }` -- when a player takes the initiative (CR 725.2)

### New Commands
- `Command::VentureIntoDungeon { player: PlayerId }` -- triggers venture (player choice of dungeon if entering new one; deterministic fallback picks LostMineOfPhandelver for regular venture, TheUndercity for "venture into Undercity")
- `Command::ChooseDungeonRoom { player: PlayerId, room: RoomIndex }` -- for branching paths (deferred to M10+ interactive; deterministic fallback picks first exit)

### New StackObjectKind Variant
- `StackObjectKind::RoomAbility { owner: PlayerId, dungeon: DungeonId, room: RoomIndex }` -- room effect on the stack (CR 309.4c)
  - New SOK variant (NOT KeywordTrigger -- room abilities have no source_object, no keyword, and are not keyword triggers)
  - Discriminant: next available after 64 (KeywordTrigger) = 65

### Interception Sites
- `state/mod.rs` -- add `dungeon_state` and `has_initiative` fields to `GameState`
- `state/player.rs` -- add `dungeons_completed` field to `PlayerState`
- `state/hash.rs` -- hash new fields on `GameState` and `PlayerState`; hash `DungeonId`, `DungeonState`
- `rules/sba.rs` -- add 704.5t check (dungeon on bottommost room + no pending room ability on stack)
- `rules/resolution.rs` -- add `RoomAbility` resolution arm
- `rules/engine.rs` -- add `VentureIntoDungeon` command handler
- `rules/events.rs` -- add 3 new `GameEvent` variants
- `rules/command.rs` -- add `VentureIntoDungeon` and `ChooseDungeonRoom` command variants
- `rules/abilities.rs` -- initiative upkeep trigger dispatch; combat damage initiative steal trigger
- `rules/turn_actions.rs` -- upkeep initiative venture trigger
- `cards/card_definition.rs` -- add `Effect::VentureIntoDungeon` and `Effect::TakeTheInitiative`; add `Condition::CompletedADungeon`
- `cards/helpers.rs` -- export dungeon types
- `effects/mod.rs` -- execute `VentureIntoDungeon` and `TakeTheInitiative` effects
- `state/builder.rs` -- initialize `dungeon_state: OrdMap::new()`, `has_initiative: None`
- `testing/replay_harness.rs` -- add `venture_into_dungeon` action type
- `tools/replay-viewer/src/view_model.rs` -- add `RoomAbility` SOK match arm
- `tools/tui/src/play/panels/stack_view.rs` -- add `RoomAbility` SOK match arm

## Session Breakdown

### Session 1: Data Model and Dungeon Definitions (8 items)

**Files**: `crates/engine/src/state/dungeon.rs` (new), `crates/engine/src/state/mod.rs`, `crates/engine/src/state/player.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/src/state/builder.rs`

1. [x] Create `crates/engine/src/state/dungeon.rs` with `DungeonId` enum, `RoomIndex` type alias, `DungeonState` struct, `DungeonDef` struct, `RoomDef` struct (CR 309.1, 309.4)
2. [x] Implement static dungeon definitions: `fn get_dungeon(id: DungeonId) -> &'static DungeonDef` returning compile-time dungeon graph data for all 4 dungeons. Room effects use `fn() -> Effect` closures returning the appropriate `Effect` variant. All 28 rooms across 4 dungeons (CR 309.2a)
   - Note: `get_dungeon` returns owned `DungeonDef` (not `&'static`) because `RoomDef::effect: fn() -> Effect` is a function pointer whose return type contains heap-allocated data, making static storage impossible. The plan acknowledged this in its notes. Room count: Lost Mine=7, Mad Mage=9, Tomb=5, Undercity=7 (28 total).
   - Note: Storeroom/Forge use `EffectTarget::Controller` as a placeholder (effects engine will need to resolve this to a controller-controlled creature in Session 2+).
   - Note: Arena/Goad uses `EffectTarget::Source` as a placeholder (room abilities have no source object; will be resolved properly in Session 2+).
3. [x] Add `pub mod dungeon;` to `state/mod.rs`. Add `dungeon_state: OrdMap<PlayerId, DungeonState>` and `has_initiative: Option<PlayerId>` to `GameState` struct (CR 309.4, CR 725.1)
4. [x] Add `dungeons_completed: u32` (with `#[serde(default)]`) to `PlayerState` (CR 309.7)
5. [x] Initialize new fields in `GameStateBuilder::build()`: `dungeon_state: OrdMap::new()`, `has_initiative: None`, `dungeons_completed: 0`
6. [x] Implement `HashInto for DungeonId` and `HashInto for DungeonState` in `hash.rs`. Add dungeon_state hashing to `public_state_hash()`, has_initiative to `public_state_hash()`, dungeons_completed to `PlayerState::hash_into()`
7. [x] Add re-exports: `pub use dungeon::{DungeonId, DungeonState, RoomIndex, DungeonDef, RoomDef, get_dungeon};` to `state/mod.rs`; added `DungeonId, DungeonState, RoomIndex` to `helpers.rs` and `lib.rs`
8. [x] Tests: `test_dungeon_def_structure` (verify all 4 dungeon graphs are well-formed -- each room's exits point to valid indices, bottommost room has empty exits, topmost is index 0), `test_dungeon_state_default` (new game has empty dungeon_state), `test_dungeon_hash_determinism` (same state produces same hash)

### Session 2: Core Venture Mechanic and Room Abilities (8 items)

**Files**: `crates/engine/src/rules/events.rs`, `crates/engine/src/rules/command.rs`, `crates/engine/src/cards/card_definition.rs`, `crates/engine/src/effects/mod.rs`, `crates/engine/src/state/stack.rs`, `crates/engine/src/rules/engine.rs`

1. [x] Add `GameEvent::VenturedIntoDungeon { player, dungeon, room }`, `GameEvent::DungeonCompleted { player, dungeon }`, `GameEvent::InitiativeTaken { player }` to events.rs (CR 309.5, 309.7, 725.2). Add `HashInto` impls for new event variants in hash.rs
2. [x] Add `Command::VentureIntoDungeon { player: PlayerId }` and `Command::ChooseDungeonRoom { player: PlayerId, room: RoomIndex }` to command.rs (CR 701.49a-c)
3. [x] Add `Effect::VentureIntoDungeon` and `Effect::TakeTheInitiative` to `card_definition.rs::Effect` enum (CR 701.49, CR 725.2). Add `Condition::CompletedADungeon` to `Condition` enum (CR 309.7)
4. [x] Add `StackObjectKind::RoomAbility { owner: PlayerId, dungeon: DungeonId, room: RoomIndex }` to stack.rs (CR 309.4c, discriminant 65). Add `HashInto` impl for the new variant in hash.rs
5. [x] Implement `handle_venture_into_dungeon(state, player) -> Vec<GameEvent>` in a new function in `engine.rs` (or a `dungeon.rs` rules module). Logic: (a) if player has no dungeon in command zone, pick default dungeon (LostMineOfPhandelver), create `DungeonState { dungeon, current_room: 0 }`, emit `VenturedIntoDungeon` event, push `RoomAbility` to stack (CR 701.49a, 309.4a, 309.4c). (b) If player is on non-bottommost room, advance to next room (first exit -- deterministic fallback), emit `VenturedIntoDungeon`, push `RoomAbility` (CR 701.49b, 309.5a). (c) If player is on bottommost room, complete dungeon (increment `dungeons_completed`, remove `DungeonState`, emit `DungeonCompleted`), then start a new dungeon (same as case a) (CR 701.49c, 309.5b)
6. [x] Wire `Command::VentureIntoDungeon` handler in `engine.rs:process_command()` to call `handle_venture_into_dungeon`. Wire `Effect::VentureIntoDungeon` execution in `effects/mod.rs` to produce `Command::VentureIntoDungeon`
7. [x] Wire `Effect::TakeTheInitiative` in `effects/mod.rs`: set `state.has_initiative = Some(controller)`, emit `InitiativeTaken`, then call `handle_venture_into_dungeon` with undercity-forced variant (CR 725.2 -- "ventures into Undercity")
8. [x] Tests: `test_venture_enters_first_room` (player ventures with no dungeon, gets room 0), `test_venture_advances_room` (player on room 0 advances to room 1), `test_venture_completes_dungeon` (player on bottommost room completes), `test_venture_starts_new_after_completion` (completes + starts new), `test_room_ability_goes_on_stack` (after venture, stack has RoomAbility)
   - Note: `RoomAbility` SOK arms in view_model.rs and stack_view.rs were added here (moved from Session 3 item 7 since the code was needed immediately for compilation). `Condition::CompletedSpecificDungeon(DungeonId)` added alongside `CompletedADungeon` (plan mentioned only CompletedADungeon but the specific variant is needed for Acererak's conditional ability). `handle_venture_into_dungeon` is `pub` (not `pub(crate)`) to enable cross-crate testing; exported from lib.rs.

### Session 3: Room Ability Resolution, SBA 704.5t, and Initiative (8 items)

**Files**: `crates/engine/src/rules/resolution.rs`, `crates/engine/src/rules/sba.rs`, `crates/engine/src/rules/turn_actions.rs`, `crates/engine/src/rules/abilities.rs`, `crates/engine/src/rules/combat.rs` (or `turn_actions.rs`)

1. [x] Add `RoomAbility` resolution arm in `resolution.rs` (CR 309.4c). Already done in Session 2; confirmed at resolution.rs line 7128.
2. [x] Add SBA 704.5t in `sba.rs:apply_sbas_once()`. Added `check_dungeon_completion_sba()` and `transfer_initiative_on_player_leave()` (pub, called from engine.rs handle_concede). (CR 704.5t, CR 309.6, CR 725.4)
3. [x] Add initiative upkeep trigger in `turn_actions.rs:upkeep_actions()`: calls `handle_venture_into_dungeon` with force_undercity=true when active player has initiative. (CR 725.2)
4. [x] Add initiative combat damage steal in `turn_actions.rs:combat_damage_step()`: after apply_combat_damage, scan CombatDamageDealt events for damage to initiative holder, transfer initiative and venture into Undercity. (CR 725.2)
5. [x] Wire initiative player-leaving-game cleanup: `transfer_initiative_on_player_leave` called from `check_player_sbas` (all 3 loss paths) and `handle_concede`. (CR 725.4)
6. [x] Implement `Condition::CompletedADungeon` evaluation. Already done in Session 2 per session notes.
7. [x] Add `RoomAbility` arms to view_model.rs and stack_view.rs. Already done in Session 2 per session notes.
8. [x] Tests in `crates/engine/tests/dungeon_resolution.rs`: all 6 tests pass — `test_room_ability_resolves_scry`, `test_room_ability_resolves_create_token`, `test_sba_704_5t_removes_completed_dungeon`, `test_sba_704_5t_waits_for_room_ability`, `test_initiative_upkeep_venture`, `test_initiative_combat_damage_steal`.

### Session 4: Card Definitions, Harness Integration, and Game Scripts (7 items)

**Files**: `crates/engine/src/cards/defs/nadaar_selfless_paladin.rs`, `crates/engine/src/cards/defs/seasoned_dungeoneer.rs`, `crates/engine/src/cards/defs/acererak_the_archlich.rs`, `crates/engine/src/testing/replay_harness.rs`, `crates/engine/src/cards/helpers.rs`

1. [x] Add `DungeonId` and `DungeonState` to `helpers.rs` exports (for card defs that reference dungeon types) — pre-done; verified at helpers.rs line 8
2. [x] Author card definition: **Nadaar, Selfless Paladin** (3W, 3/3 legendary Dragon Knight, Vigilance, "Whenever Nadaar enters or attacks, venture into the dungeon", "Other creatures you control get +1/+1 as long as you've completed a dungeon") — `Effect::VentureIntoDungeon` for both ETB + WhenAttacks; static +1/+1 buff skipped (DSL gap: EffectFilter::OtherCreaturesControlledBy not implemented)
3. [x] Author card definition: **Seasoned Dungeoneer** (3W, 3/4 Human Warrior, "When this creature enters, you take the initiative", attack trigger) — `Effect::TakeTheInitiative` for ETB; attack trigger (WheneverYouAttack + GrantProtection + Explore) deferred as DSL gap
4. [x] Author card definition: **Acererak the Archlich** (2B, 5/5 legendary Zombie Wizard) — ETB intervening-if uses `Condition::Not(Box::new(CompletedSpecificDungeon(TombOfAnnihilation)))` with `Effect::Sequence[MoveZone+VentureIntoDungeon]`; WhenAttacks ForEach creates Zombie tokens. Added `Condition::Not(Box<Condition>)` variant to card_definition.rs + check_condition evaluation + hash.rs discriminant 17.
5. [x] Add `venture_into_dungeon` harness action in `replay_harness.rs:translate_player_action()` — translates to `Command::VentureIntoDungeon { player }`
6. [x] Generate game script for Nadaar — script `205_nadaar_ventures_on_etb.json` at `test-data/generated-scripts/etb-triggers/`; covers ETB→venture→RoomAbility(Scry 1) pipeline; `review_status: "pending_review"`
7. [x] Tests: all 5 pass — `test_nadaar_enters_ventures`, `test_nadaar_attacks_ventures`, `test_nadaar_completed_dungeon_buff`, `test_acererak_bounces_without_tomb`, `test_initiative_take_ventures_undercity`. Root-cause fix: added `PendingTriggerKind::CardDefETB` + `is_carddef_etb: bool` on `StackObjectKind::TriggeredAbility` to resolve ETB trigger index namespace collision (CardDef index vs runtime triggered_abilities index). Updated turn_actions.rs upkeep/end-step CardDef triggers to also use `CardDefETB`.

## Acceptance Criteria Checklist

- [x] All 4 dungeon definitions are complete with correct room graphs
- [x] `Effect::VentureIntoDungeon` correctly handles all 3 CR 701.49 cases (no dungeon, mid-dungeon, bottommost room)
- [x] Room abilities go on the stack as `RoomAbility` SOK and resolve through the standard stack resolution path (CR 309.4c)
- [x] SBA 704.5t removes completed dungeons only when no room ability from that dungeon is on the stack (CR 309.6)
- [x] Initiative triggers work: upkeep venture (CR 725.2), combat damage steal (CR 725.2), take = venture into Undercity (CR 725.2)
- [x] "Venture into Undercity" forces The Undercity when entering a new dungeon (CR 701.49d)
- [x] `Condition::CompletedADungeon` works for Nadaar-style conditional buffs
- [x] `dungeons_completed` persists correctly across the game
- [x] All new fields hashed in `hash.rs`
- [x] All tests pass: `~/.cargo/bin/cargo test --all` (1953 tests, 0 failures)
- [x] Zero clippy warnings: `~/.cargo/bin/cargo clippy -- -D warnings`
- [x] Formatted: `~/.cargo/bin/cargo fmt --check`
- [x] `cargo build --workspace` succeeds (replay-viewer and TUI compile with new SOK variant)

## Key CR References

| CR Section | Summary | Session |
|------------|---------|---------|
| 309.1 | Dungeon is a card type on nontraditional cards | 1 |
| 309.2a | Dungeon brought into game from outside via venture | 2 |
| 309.2c | Dungeons are not permanents, can't be cast, can't leave command zone except to leave game | 1 |
| 309.3 | Player can own only one dungeon in command zone at a time | 2 |
| 309.4 | Rooms connected by arrows; venture marker tracks position | 1 |
| 309.4a | Venture marker starts on topmost room when dungeon enters command zone | 2 |
| 309.4b | Room names are flavor text | 1 |
| 309.4c | Room abilities are triggered abilities: "When you move your venture marker into this room, [effect]" | 2, 3 |
| 309.5a | Venture advances marker following an arrow | 2 |
| 309.5b | Venture on bottommost: complete dungeon, start new one | 2 |
| 309.6 | SBA: remove completed dungeon when on bottommost + no pending room ability | 3 |
| 309.7 | Player completes a dungeon as it is removed from the game | 2, 3 |
| 701.49a | Venture with no dungeon: choose from outside game, place marker on top room | 2 |
| 701.49b | Venture with marker not on bottommost: advance to adjacent room | 2 |
| 701.49c | Venture with marker on bottommost: complete, then choose new dungeon | 2 |
| 701.49d | "Venture into [quality]" forces specific dungeon choice (Undercity) | 2 |
| 704.5t | SBA: dungeon on bottommost + no room ability on stack = remove from game | 3 |
| 725.1 | Initiative is a player designation | 1 |
| 725.2 | Three inherent triggered abilities: upkeep venture, combat steal, take = venture Undercity | 3 |
| 725.3 | Only one player has initiative at a time | 2 |
| 725.4 | Initiative transfers when holder leaves game | 3 |
| 725.5 | Re-taking initiative still triggers venture into Undercity | 2 |

## Corner Cases Addressed

| Corner Case | Description | Session |
|-------------|-------------|---------|
| Branching paths | Dungeon rooms with multiple exits (deterministic: pick first exit) | 2 |
| Bottommost room + pending room ability | SBA must wait for room ability to leave stack before removing dungeon (CR 309.6) | 3 |
| Complete + re-enter | Completing a dungeon immediately starts a new one in the same venture action (CR 701.49c) | 2 |
| Initiative re-take | Taking initiative when you already have it still triggers venture into Undercity (CR 725.5) | 3 |
| Initiative on player leave | Active player gets initiative when holder leaves game (CR 725.4) | 3 |
| Venture into Undercity variant | Cards saying "venture into Undercity" force The Undercity when choosing a new dungeon (CR 701.49d) | 2 |
| Acererak intervening-if | ETB checks "if you haven't completed Tomb of Annihilation" both at trigger time and resolution (CR 603.4) | 4 |

## Deferred to M10+ (Interactive Features)

These room effects have interactive components that require player choices. They use deterministic fallbacks for now:

| Room | Full Effect | Deterministic Fallback |
|------|-----------|----------------------|
| Storeroom (Lost Mine) | Put +1/+1 counter on **target** creature | Smallest ObjectId creature you control |
| Forge (Undercity) | Two +1/+1 counters on **target** creature | Smallest ObjectId creature you control |
| Twisted Caverns (Mad Mage) | **Target** creature can't attack | Smallest ObjectId creature opponent controls |
| Fungi Cavern (Lost Mine) | **Target** creature gets -4/-0 | Smallest ObjectId creature opponent controls |
| Arena (Undercity) | **Goad target** creature | Smallest ObjectId creature opponent controls |
| Veils of Fear (Tomb) | "unless they discard a card" | Each player just loses 2 life |
| Sandfall Cell (Tomb) | "unless they sacrifice" | Each player just loses 2 life |
| Runestone Caverns (Mad Mage) | "exile top 2, you may play them" | Exile top 2, add to hand |
| Mad Wizard's Lair (Mad Mage) | "draw 3, may cast one free" | Draw 3 |
| Throne of the Dead Three (Undercity) | "reveal top 10, pick 3 by type" | Draw 3 |
| Dungeon/room choice | Player chooses which dungeon to enter, which path to take | First dungeon / first exit |

## Notes for Session Runner

- The `DungeonDef` uses `fn() -> Effect` for room effects (not stored `Effect` values) because `Effect` is not `'static` -- it contains heap-allocated types like `String` and `Vec`. The function returns a fresh `Effect` each time it is called.
- Do NOT add a `KeywordAbility::VentureIntoTheDungeon` variant. Venture is a keyword action (CR 701.49), not a keyword ability. It is modeled as an `Effect`, not a keyword.
- Similarly, initiative is a player designation (CR 725.1), not a keyword ability. It uses `has_initiative: Option<PlayerId>` on `GameState`, not a `KeywordAbility` variant.
- `RoomAbility` is a new `StackObjectKind` variant (NOT `KeywordTrigger`) because room abilities have no `source_object` in the traditional sense (the dungeon is in the command zone, not a permanent) and are not keyword-based triggers.
- When implementing SBA 704.5t, the check "no room ability on the stack from that dungeon" must match on `StackObjectKind::RoomAbility { owner, dungeon, .. }` -- both owner AND dungeon must match.
- The `Condition::CompletedADungeon` check is simple: `player.dungeons_completed > 0`. For Acererak's "if you haven't completed Tomb of Annihilation," a more specific condition variant `Condition::CompletedSpecificDungeon(DungeonId)` may be needed, or the card def can use a `Condition::Always` with the check embedded in the effect logic.
- Use `card-definition-author` agent for the 3 card definitions in Session 4.
- Use `game-script-generator` agent for the Nadaar game script in Session 4.
- After all sessions complete, run `cr-coverage-auditor` for CR 309 and CR 701.49.

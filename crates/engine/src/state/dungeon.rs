//! Dungeon data model and static dungeon definitions.
//!
//! Dungeons are non-traditional cards (CR 309.1) that exist outside the game
//! and are brought in via the "venture into the dungeon" keyword action (CR 701.49).
//! They are NOT permanents and cannot be cast (CR 309.2c).
//!
//! This module contains:
//! - `DungeonId` enum: identifies one of the 4 dungeons
//! - `RoomIndex` type alias: index into a dungeon's room list
//! - `DungeonState` struct: tracks a player's current position in a dungeon (CR 309.4)
//! - `DungeonDef` / `RoomDef` structs: static room graph definitions
//! - `get_dungeon()`: returns a static dungeon definition by id
//!
//! See CR 309 for dungeon rules and CR 701.49 for the venture keyword action.
use crate::cards::card_definition::{Effect, EffectAmount, PlayerTarget, TargetFilter, ZoneTarget};
use crate::state::game_object::ManaAbility;
use crate::state::types::{CardType, Color, KeywordAbility, SubType};
use serde::{Deserialize, Serialize};
/// CR 309.1: Identifies which dungeon a player is exploring.
///
/// All 4 dungeons introduced in Adventures in the Forgotten Realms (2021).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DungeonId {
    /// 7-room dungeon with branching paths. Good early-game option.
    LostMineOfPhandelver,
    /// 9-room dungeon. Longest path with powerful late-game payoffs.
    DungeonOfTheMadMage,
    /// 5-room dungeon. Painful for everyone; used by Acererak the Archlich.
    TombOfAnnihilation,
    /// 7-room dungeon. Entered via "take the initiative" (CR 725.2).
    TheUndercity,
}
/// CR 309.4: Index into a dungeon's room list.
///
/// Room 0 is always the topmost room (entry point). The bottommost room
/// has an empty `exits` list.
pub type RoomIndex = usize;
/// CR 309.4: Tracks a player's current position in a dungeon.
///
/// When a player ventures into the dungeon (CR 701.49), this struct is
/// created or updated in `GameState::dungeon_state` for that player.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DungeonState {
    /// Which dungeon the player is currently in (in their command zone).
    pub dungeon: DungeonId,
    /// Index of the room the venture marker is currently on (0 = topmost entry room).
    pub current_room: RoomIndex,
}
/// Static definition of a dungeon's room graph.
///
/// CR 309.4b: Room names are flavor text — they have no gameplay effect.
/// CR 309.4c: Room abilities are triggered abilities that fire when the
/// venture marker moves into that room.
pub struct DungeonDef {
    /// Which dungeon this definition describes.
    pub id: DungeonId,
    /// Human-readable dungeon name (e.g., "Lost Mine of Phandelver").
    pub name: &'static str,
    /// All rooms in this dungeon, indexed by `RoomIndex`.
    /// Room 0 is always the entry (topmost) room.
    pub rooms: Vec<RoomDef>,
    /// Index of the bottommost room (empty exits). Used by SBA 704.5t.
    pub bottommost_room: RoomIndex,
}
/// CR 309.4: A single room in a dungeon.
///
/// Each room has a name (flavor only), an effect triggered on entry,
/// and a list of exits (indices of adjacent rooms to venture into next).
pub struct RoomDef {
    /// CR 309.4b: Room name — flavor text only, no gameplay effect.
    pub name: &'static str,
    /// CR 309.4c: Effect that triggers when the venture marker enters this room.
    ///
    /// Stored as a function pointer (not a value) because `Effect` contains heap types
    /// (`String`, `Vec`) that are not `'static`. Call `(room.effect)()` to
    /// produce a fresh `Effect` instance.
    pub effect: fn() -> Effect,
    /// Indices of rooms this room leads to (outgoing arrows).
    ///
    /// Empty means this is the bottommost room (CR 309.4: "no arrow pointing away
    /// from a room" = bottommost).
    /// Exactly one exit = linear progression.
    /// Two exits = branching path (CR 309.5a: player chooses; deterministic fallback
    /// picks the first exit).
    pub exits: &'static [RoomIndex],
}
// ── Token helpers for room effects ───────────────────────────────────────────
fn goblin_token_spec() -> crate::cards::card_definition::TokenSpec {
    crate::cards::card_definition::TokenSpec {
        name: "Goblin".to_string(),
        power: 1,
        toughness: 1,
        colors: [Color::Red].into_iter().collect(),
        supertypes: im::OrdSet::new(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
        keywords: im::OrdSet::new(),
        count: 1,
        tapped: false,
        enters_attacking: false,
        mana_color: None,
        mana_abilities: vec![],
        activated_abilities: vec![],
    }
}
fn treasure_token_spec_1() -> crate::cards::card_definition::TokenSpec {
    crate::cards::card_definition::TokenSpec {
        name: "Treasure".to_string(),
        power: 0,
        toughness: 0,
        colors: im::OrdSet::new(),
        supertypes: im::OrdSet::new(),
        card_types: [CardType::Artifact].into_iter().collect(),
        subtypes: [SubType("Treasure".to_string())].into_iter().collect(),
        keywords: im::OrdSet::new(),
        count: 1,
        tapped: false,
        enters_attacking: false,
        mana_color: None,
        mana_abilities: vec![ManaAbility::treasure()],
        activated_abilities: vec![],
    }
}
fn skeleton_11_token_spec() -> crate::cards::card_definition::TokenSpec {
    crate::cards::card_definition::TokenSpec {
        name: "Skeleton".to_string(),
        power: 1,
        toughness: 1,
        colors: [Color::Black].into_iter().collect(),
        supertypes: im::OrdSet::new(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Skeleton".to_string())].into_iter().collect(),
        keywords: im::OrdSet::new(),
        count: 1,
        tapped: false,
        enters_attacking: false,
        mana_color: None,
        mana_abilities: vec![],
        activated_abilities: vec![],
    }
}
fn skeleton_41_menace_token_spec() -> crate::cards::card_definition::TokenSpec {
    crate::cards::card_definition::TokenSpec {
        name: "Skeleton".to_string(),
        power: 4,
        toughness: 1,
        colors: [Color::Black].into_iter().collect(),
        supertypes: im::OrdSet::new(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Skeleton".to_string())].into_iter().collect(),
        keywords: [KeywordAbility::Menace].into_iter().collect(),
        count: 1,
        tapped: false,
        enters_attacking: false,
        mana_color: None,
        mana_abilities: vec![],
        activated_abilities: vec![],
    }
}
fn atropal_token_spec() -> crate::cards::card_definition::TokenSpec {
    use crate::state::types::SuperType;
    crate::cards::card_definition::TokenSpec {
        name: "The Atropal".to_string(),
        power: 4,
        toughness: 4,
        colors: [Color::Black].into_iter().collect(),
        supertypes: [SuperType::Legendary].into_iter().collect(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("God".to_string()), SubType("Horror".to_string())]
            .into_iter()
            .collect(),
        keywords: [KeywordAbility::Deathtouch].into_iter().collect(),
        count: 1,
        tapped: false,
        enters_attacking: false,
        mana_color: None,
        mana_abilities: vec![],
        activated_abilities: vec![],
    }
}
// ── Static dungeon definitions ────────────────────────────────────────────────
/// CR 309.2a, 309.4: Return the static definition for a dungeon.
///
/// Returns an owned `DungeonDef` (not a static reference) because `RoomDef`
/// contains `fn() -> Effect` function pointers whose return types contain
/// heap-allocated data (`String`, `Vec`) and thus cannot be `'static`.
///
/// All 4 dungeons are defined here with their complete room graphs per CR 309.
pub fn get_dungeon(id: DungeonId) -> DungeonDef {
    match id {
        DungeonId::LostMineOfPhandelver => lost_mine_of_phandelver(),
        DungeonId::DungeonOfTheMadMage => dungeon_of_the_mad_mage(),
        DungeonId::TombOfAnnihilation => tomb_of_annihilation(),
        DungeonId::TheUndercity => the_undercity(),
    }
}
/// CR 309.2a: Lost Mine of Phandelver — 7 rooms, two branching paths.
///
/// Room layout:
/// 0 Cave Entrance (entry) → [1, 2]
/// 1 Goblin Lair → [3, 4]
/// 2 Mine Tunnels → [4, 5]
/// 3 Storeroom → [6]
/// 4 Dark Pool → [6]
/// 5 Fungi Cavern → [6]
/// 6 Temple of Dumathoin (bottommost) → []
fn lost_mine_of_phandelver() -> DungeonDef {
    DungeonDef {
        id: DungeonId::LostMineOfPhandelver,
        name: "Lost Mine of Phandelver",
        rooms: vec![
            // 0: Cave Entrance — Scry 1
            RoomDef {
                name: "Cave Entrance",
                effect: || Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                exits: &[1, 2],
            },
            // 1: Goblin Lair — Create a 1/1 red Goblin creature token
            RoomDef {
                name: "Goblin Lair",
                effect: || Effect::CreateToken {
                    spec: goblin_token_spec(),
                },
                exits: &[3, 4],
            },
            // 2: Mine Tunnels — Create a Treasure token
            RoomDef {
                name: "Mine Tunnels",
                effect: || Effect::CreateToken {
                    spec: treasure_token_spec_1(),
                },
                exits: &[4, 5],
            },
            // 3: Storeroom — "Put a +1/+1 counter on target creature."
            // TODO(M10+): requires interactive targeting — deterministic fallback gains 1 life
            // as a placeholder (AddCounter needs an Object target, not a Player target).
            RoomDef {
                name: "Storeroom",
                effect: || Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                exits: &[6],
            },
            // 4: Dark Pool — Each opponent loses 1 life and you gain 1 life
            RoomDef {
                name: "Dark Pool",
                effect: || {
                    Effect::Sequence(vec![
                        Effect::LoseLife {
                            player: PlayerTarget::EachOpponent,
                            amount: EffectAmount::Fixed(1),
                        },
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(1),
                        },
                    ])
                },
                exits: &[6],
            },
            // 5: Fungi Cavern — "Target creature gets -4/-0 until your next turn."
            // TODO(M10+): requires interactive targeting + continuous effect duration.
            // Placeholder: controller gains 1 life (arbitrary — no meaningful approximation).
            RoomDef {
                name: "Fungi Cavern",
                effect: || Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                exits: &[6],
            },
            // 6: Temple of Dumathoin — Draw a card (bottommost)
            RoomDef {
                name: "Temple of Dumathoin",
                effect: || Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                exits: &[],
            },
        ],
        bottommost_room: 6,
    }
}
/// CR 309.2a: Dungeon of the Mad Mage — 9 rooms, linear with branching paths.
///
/// Room layout:
/// 0 Yawning Portal → [1]
/// 1 Dungeon Level → [2, 3]
/// 2 Goblin Bazaar → [4]
/// 3 Twisted Caverns → [4]
/// 4 Lost Level → [5, 6]
/// 5 Runestone Caverns → [7]
/// 6 Muiral's Graveyard → [7]
/// 7 Deep Mines → [8]
/// 8 Mad Wizard's Lair (bottommost) → []
fn dungeon_of_the_mad_mage() -> DungeonDef {
    DungeonDef {
        id: DungeonId::DungeonOfTheMadMage,
        name: "Dungeon of the Mad Mage",
        rooms: vec![
            // 0: Yawning Portal — You gain 1 life
            RoomDef {
                name: "Yawning Portal",
                effect: || Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                exits: &[1],
            },
            // 1: Dungeon Level — Scry 1
            RoomDef {
                name: "Dungeon Level",
                effect: || Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                exits: &[2, 3],
            },
            // 2: Goblin Bazaar — Create a Treasure token
            RoomDef {
                name: "Goblin Bazaar",
                effect: || Effect::CreateToken {
                    spec: treasure_token_spec_1(),
                },
                exits: &[4],
            },
            // 3: Twisted Caverns — "Target creature can't attack until your next turn."
            // TODO(M10+): requires interactive targeting + continuous effect duration.
            // Placeholder: controller gains 1 life (arbitrary — no meaningful approximation).
            RoomDef {
                name: "Twisted Caverns",
                effect: || Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                exits: &[4],
            },
            // 4: Lost Level — Scry 2
            RoomDef {
                name: "Lost Level",
                effect: || Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                exits: &[5, 6],
            },
            // 5: Runestone Caverns — Exile top 2 cards; you may play them
            // Deterministic fallback: draw 2 cards (interactive exile+play deferred to M10+)
            RoomDef {
                name: "Runestone Caverns",
                effect: || Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                exits: &[7],
            },
            // 6: Muiral's Graveyard — Create two 1/1 black Skeleton creature tokens
            RoomDef {
                name: "Muiral's Graveyard",
                effect: || {
                    let mut spec = skeleton_11_token_spec();
                    spec.count = 2;
                    Effect::CreateToken { spec }
                },
                exits: &[7],
            },
            // 7: Deep Mines — Scry 3
            RoomDef {
                name: "Deep Mines",
                effect: || Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                exits: &[8],
            },
            // 8: Mad Wizard's Lair — Draw 3 cards (bottommost)
            // Deterministic fallback: draw 3 (interactive free-cast deferred to M10+)
            RoomDef {
                name: "Mad Wizard's Lair",
                effect: || Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                exits: &[],
            },
        ],
        bottommost_room: 8,
    }
}
/// CR 309.2a: Tomb of Annihilation — 5 rooms, branching paths.
///
/// Room layout:
/// 0 Trapped Entry → [1, 2]
/// 1 Veils of Fear → [3]
/// 2 Oubliette → [4]
/// 3 Sandfall Cell → [4]
/// 4 Cradle of the Death God (bottommost) → []
fn tomb_of_annihilation() -> DungeonDef {
    DungeonDef {
        id: DungeonId::TombOfAnnihilation,
        name: "Tomb of Annihilation",
        rooms: vec![
            // 0: Trapped Entry — Each player loses 1 life
            RoomDef {
                name: "Trapped Entry",
                effect: || Effect::LoseLife {
                    player: PlayerTarget::EachPlayer,
                    amount: EffectAmount::Fixed(1),
                },
                exits: &[1, 2],
            },
            // 1: Veils of Fear — Each player loses 2 life unless they discard a card
            // Deterministic fallback: each player loses 2 life (interactive discard deferred to M10+)
            RoomDef {
                name: "Veils of Fear",
                effect: || Effect::LoseLife {
                    player: PlayerTarget::EachPlayer,
                    amount: EffectAmount::Fixed(2),
                },
                exits: &[3],
            },
            // 2: Oubliette — Discard a card and sacrifice a creature, artifact, and land
            // Deterministic fallback: controller discards 1 card (full sacrifice sequence deferred to M10+)
            RoomDef {
                name: "Oubliette",
                effect: || Effect::DiscardCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                exits: &[3],
            },
            // 3: Sandfall Cell — Each player loses 2 life unless they sacrifice creature/artifact/land
            // Deterministic fallback: each player loses 2 life (interactive sacrifice choice deferred to M10+)
            RoomDef {
                name: "Sandfall Cell",
                effect: || Effect::LoseLife {
                    player: PlayerTarget::EachPlayer,
                    amount: EffectAmount::Fixed(2),
                },
                exits: &[4],
            },
            // 4: Cradle of the Death God — Create The Atropal, a legendary 4/4 black God Horror
            //    creature token with deathtouch (bottommost)
            RoomDef {
                name: "Cradle of the Death God",
                effect: || Effect::CreateToken {
                    spec: atropal_token_spec(),
                },
                exits: &[],
            },
        ],
        bottommost_room: 4,
    }
}
/// CR 309.2a, 725.2: The Undercity — 7 rooms, entered via "take the initiative".
///
/// Room layout:
/// 0 Secret Entrance → [1, 2]
/// 1 Forge → [3, 4]
/// 2 Lost Well → [3, 4]
/// 3 Arena → [5]
/// 4 Stash → [5]
/// 5 Catacombs → [6]
/// 6 Throne of the Dead Three (bottommost) → []
fn the_undercity() -> DungeonDef {
    DungeonDef {
        id: DungeonId::TheUndercity,
        name: "The Undercity",
        rooms: vec![
            // 0: Secret Entrance — Search your library for a basic land, reveal it, put it in hand
            RoomDef {
                name: "Secret Entrance",
                effect: || Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        basic: true,
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    reveal: true,
                    destination: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                exits: &[1, 2],
            },
            // 1: Forge — "Put two +1/+1 counters on target creature you control."
            // TODO(M10+): requires interactive targeting — placeholder gains 2 life.
            RoomDef {
                name: "Forge",
                effect: || Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
                exits: &[3, 4],
            },
            // 2: Lost Well — Scry 2
            RoomDef {
                name: "Lost Well",
                effect: || Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                exits: &[3, 4],
            },
            // 3: Arena — "Goad target creature an opponent controls."
            // TODO(M10+): requires interactive targeting — placeholder gains 1 life.
            // (EffectTarget::Source points at the dungeon, not a creature on the battlefield.)
            RoomDef {
                name: "Arena",
                effect: || Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                exits: &[5],
            },
            // 4: Stash — Create a Treasure token
            RoomDef {
                name: "Stash",
                effect: || Effect::CreateToken {
                    spec: treasure_token_spec_1(),
                },
                exits: &[5],
            },
            // 5: Catacombs — Create a 4/1 black Skeleton creature token with menace
            RoomDef {
                name: "Catacombs",
                effect: || Effect::CreateToken {
                    spec: skeleton_41_menace_token_spec(),
                },
                exits: &[6],
            },
            // 6: Throne of the Dead Three — Reveal top 10 cards, take a creature, a land,
            //    a nonland noncreature card into hand; rest go to bottom in random order (bottommost)
            // Deterministic fallback: draw 3 cards (interactive top-10 selection deferred to M10+)
            RoomDef {
                name: "Throne of the Dead Three",
                effect: || Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                exits: &[],
            },
        ],
        bottommost_room: 6,
    }
}

//! Test utility: fluent builders for constructing game states.
//!
//! The builder panics on invalid configuration — it's a test utility, not a
//! production API. Tests should always use `GameStateBuilder` rather than
//! constructing `GameState` structs directly, ensuring all invariants hold.

use im::{OrdMap, OrdSet, Vector};

use super::game_object::{
    ActivatedAbility, Characteristics, GameObject, ManaAbility, ManaCost, ObjectId, ObjectStatus,
    TriggeredAbilityDef,
};
use super::player::{CardId, ManaPool, PlayerId, PlayerState};
use super::turn::{Step, TurnState};
use super::types::{CardType, Color, CounterType, KeywordAbility, SubType, SuperType};
use super::zone::{Zone, ZoneId};
use super::GameState;

/// Builder for constructing `GameState` values in tests.
pub struct GameStateBuilder {
    players: Vec<PlayerConfig>,
    objects: Vec<ObjectSpec>,
    turn_number: u32,
    step: Option<Step>,
    active_player: Option<PlayerId>,
    is_first_turn_of_game: bool,
}

struct PlayerConfig {
    id: PlayerId,
    life_total: i32,
    poison_counters: u32,
    commander_ids: Vec<CardId>,
    mana_pool: ManaPool,
    land_plays_remaining: u32,
    max_hand_size: usize,
}

impl GameStateBuilder {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            objects: Vec::new(),
            turn_number: 1,
            step: None,
            active_player: None,
            is_first_turn_of_game: false,
        }
    }

    /// Create a builder pre-configured with 4 players (IDs 1-4) at 40 life.
    pub fn four_player() -> Self {
        Self::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .add_player(PlayerId(3))
            .add_player(PlayerId(4))
    }

    /// Add a player with Commander starting life (40).
    pub fn add_player(mut self, id: PlayerId) -> Self {
        self.players.push(PlayerConfig {
            id,
            life_total: 40,
            poison_counters: 0,
            commander_ids: Vec::new(),
            mana_pool: ManaPool::default(),
            land_plays_remaining: 1,
            max_hand_size: 7,
        });
        self
    }

    /// Configure a player using a `PlayerBuilder`.
    pub fn add_player_with(
        mut self,
        id: PlayerId,
        f: impl FnOnce(PlayerBuilder) -> PlayerBuilder,
    ) -> Self {
        let builder = f(PlayerBuilder::new(id));
        self.players.push(builder.config);
        self
    }

    /// Set a player's life total.
    pub fn player_life(mut self, id: PlayerId, life: i32) -> Self {
        for p in &mut self.players {
            if p.id == id {
                p.life_total = life;
            }
        }
        self
    }

    /// Set a player's poison counters.
    pub fn player_poison(mut self, id: PlayerId, counters: u32) -> Self {
        for p in &mut self.players {
            if p.id == id {
                p.poison_counters = counters;
            }
        }
        self
    }

    /// Set a player's mana pool.
    pub fn player_mana(mut self, id: PlayerId, pool: ManaPool) -> Self {
        for p in &mut self.players {
            if p.id == id {
                p.mana_pool = pool.clone();
            }
        }
        self
    }

    /// Register a commander for a player.
    pub fn player_commander(mut self, id: PlayerId, card_id: CardId) -> Self {
        for p in &mut self.players {
            if p.id == id {
                p.commander_ids.push(card_id.clone());
            }
        }
        self
    }

    /// Add a game object to the state.
    pub fn object(mut self, spec: ObjectSpec) -> Self {
        self.objects.push(spec);
        self
    }

    /// Set the starting turn number.
    pub fn turn_number(mut self, n: u32) -> Self {
        self.turn_number = n;
        self
    }

    /// Set the starting step (and derive phase from it).
    pub fn at_step(mut self, step: Step) -> Self {
        self.step = Some(step);
        self
    }

    /// Set the active player (defaults to first player).
    pub fn active_player(mut self, player: PlayerId) -> Self {
        self.active_player = Some(player);
        self
    }

    /// Mark this as the first turn of the game (first player skips draw).
    pub fn first_turn_of_game(mut self) -> Self {
        self.is_first_turn_of_game = true;
        self
    }

    /// Build the `GameState`. Panics if configuration is invalid (no players).
    pub fn build(self) -> GameState {
        assert!(!self.players.is_empty(), "must have at least one player");

        let player_ids: Vec<PlayerId> = self.players.iter().map(|p| p.id).collect();
        let active_player = self.active_player.unwrap_or(player_ids[0]);

        // Build players
        let mut players = OrdMap::new();
        for config in &self.players {
            let player_state = PlayerState {
                id: config.id,
                life_total: config.life_total,
                mana_pool: config.mana_pool.clone(),
                commander_tax: OrdMap::new(),
                commander_damage_received: OrdMap::new(),
                poison_counters: config.poison_counters,
                land_plays_remaining: config.land_plays_remaining,
                has_drawn_for_turn: false,
                has_lost: false,
                has_conceded: false,
                commander_ids: config.commander_ids.iter().cloned().collect(),
                max_hand_size: config.max_hand_size,
            };
            players.insert(config.id, player_state);
        }

        // Build zones — shared zones + per-player zones
        let mut zones = OrdMap::new();
        zones.insert(ZoneId::Battlefield, Zone::new_unordered());
        zones.insert(ZoneId::Stack, Zone::new_ordered());
        zones.insert(ZoneId::Exile, Zone::new_unordered());
        for config in &self.players {
            zones.insert(ZoneId::Library(config.id), Zone::new_ordered());
            zones.insert(ZoneId::Hand(config.id), Zone::new_unordered());
            zones.insert(ZoneId::Graveyard(config.id), Zone::new_ordered());
            zones.insert(ZoneId::Command(config.id), Zone::new_unordered());
        }

        let step = self.step.unwrap_or(Step::PreCombatMain);
        let turn = TurnState {
            phase: step.phase(),
            step,
            active_player,
            priority_holder: if step.has_priority() {
                Some(active_player)
            } else {
                None
            },
            players_passed: OrdSet::new(),
            turn_number: self.turn_number,
            turn_order: player_ids.iter().copied().collect(),
            extra_turns: Vector::new(),
            extra_combats: 0,
            in_extra_combat: false,
            is_first_turn_of_game: self.is_first_turn_of_game,
            last_regular_active: active_player,
        };

        let mut state = GameState {
            turn,
            players,
            zones,
            objects: OrdMap::new(),
            continuous_effects: Vector::new(),
            delayed_triggers: Vector::new(),
            replacement_effects: Vector::new(),
            pending_triggers: Vector::new(),
            stack_objects: Vector::new(),
            combat: None,
            timestamp_counter: 0,
            history: Vector::new(),
        };

        // Add objects
        for spec in self.objects {
            let owner = spec.owner;
            let controller = spec.controller.unwrap_or(owner);
            let zone = spec.zone;

            let characteristics = Characteristics {
                name: spec.name,
                mana_cost: spec.mana_cost,
                colors: spec.colors.into_iter().collect(),
                color_indicator: None,
                supertypes: spec.supertypes.into_iter().collect(),
                card_types: spec.card_types.into_iter().collect(),
                subtypes: spec.subtypes.into_iter().collect(),
                rules_text: String::new(),
                abilities: Vector::new(),
                keywords: spec.keywords.into_iter().collect(),
                mana_abilities: spec.mana_abilities.into_iter().collect(),
                activated_abilities: spec.activated_abilities.into_iter().collect(),
                triggered_abilities: spec.triggered_abilities.into_iter().collect(),
                power: spec.power,
                toughness: spec.toughness,
                loyalty: spec.loyalty,
                defense: None,
            };

            let mut counters = OrdMap::new();
            for (ct, count) in spec.counters {
                counters.insert(ct, count);
            }

            let object = GameObject {
                id: ObjectId(0), // Assigned by add_object
                card_id: spec.card_id,
                characteristics,
                controller,
                owner,
                zone, // Assigned by add_object
                status: ObjectStatus {
                    tapped: spec.tapped,
                    flipped: false,
                    face_down: false,
                    phased_out: false,
                },
                counters,
                attachments: Vector::new(),
                attached_to: None,
                damage_marked: 0,
                is_token: spec.is_token,
                timestamp: 0, // Assigned by add_object
            };

            state
                .add_object(object, zone)
                .expect("failed to add object in builder");
        }

        state
    }
}

impl Default for GameStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring a player.
pub struct PlayerBuilder {
    config: PlayerConfig,
}

impl PlayerBuilder {
    fn new(id: PlayerId) -> Self {
        Self {
            config: PlayerConfig {
                id,
                life_total: 40,
                poison_counters: 0,
                commander_ids: Vec::new(),
                mana_pool: ManaPool::default(),
                land_plays_remaining: 1,
                max_hand_size: 7,
            },
        }
    }

    pub fn life(mut self, life: i32) -> Self {
        self.config.life_total = life;
        self
    }

    pub fn poison(mut self, counters: u32) -> Self {
        self.config.poison_counters = counters;
        self
    }

    pub fn mana(mut self, pool: ManaPool) -> Self {
        self.config.mana_pool = pool;
        self
    }

    pub fn commander(mut self, card_id: CardId) -> Self {
        self.config.commander_ids.push(card_id);
        self
    }

    pub fn land_plays(mut self, n: u32) -> Self {
        self.config.land_plays_remaining = n;
        self
    }

    pub fn max_hand_size(mut self, n: usize) -> Self {
        self.config.max_hand_size = n;
        self
    }
}

/// Specification for a game object to be created by the builder.
pub struct ObjectSpec {
    pub name: String,
    pub card_id: Option<CardId>,
    pub owner: PlayerId,
    pub controller: Option<PlayerId>,
    pub zone: ZoneId,
    pub card_types: Vec<CardType>,
    pub subtypes: Vec<SubType>,
    pub supertypes: Vec<SuperType>,
    pub colors: Vec<Color>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub loyalty: Option<i32>,
    pub mana_cost: Option<ManaCost>,
    pub tapped: bool,
    pub counters: Vec<(CounterType, u32)>,
    pub is_token: bool,
    pub mana_abilities: Vec<ManaAbility>,
    pub keywords: Vec<KeywordAbility>,
    pub activated_abilities: Vec<ActivatedAbility>,
    pub triggered_abilities: Vec<TriggeredAbilityDef>,
}

impl ObjectSpec {
    /// Create a creature specification (defaults to battlefield).
    pub fn creature(owner: PlayerId, name: &str, power: i32, toughness: i32) -> Self {
        Self {
            name: name.to_string(),
            card_id: None,
            owner,
            controller: None,
            zone: ZoneId::Battlefield,
            card_types: vec![CardType::Creature],
            subtypes: Vec::new(),
            supertypes: Vec::new(),
            colors: Vec::new(),
            power: Some(power),
            toughness: Some(toughness),
            loyalty: None,
            mana_cost: None,
            tapped: false,
            counters: Vec::new(),
            is_token: false,
            mana_abilities: Vec::new(),
            keywords: Vec::new(),
            activated_abilities: Vec::new(),
            triggered_abilities: Vec::new(),
        }
    }

    /// Create an artifact specification (defaults to battlefield).
    pub fn artifact(owner: PlayerId, name: &str) -> Self {
        Self {
            name: name.to_string(),
            card_id: None,
            owner,
            controller: None,
            zone: ZoneId::Battlefield,
            card_types: vec![CardType::Artifact],
            subtypes: Vec::new(),
            supertypes: Vec::new(),
            colors: Vec::new(),
            power: None,
            toughness: None,
            loyalty: None,
            mana_cost: None,
            tapped: false,
            counters: Vec::new(),
            is_token: false,
            mana_abilities: Vec::new(),
            keywords: Vec::new(),
            activated_abilities: Vec::new(),
            triggered_abilities: Vec::new(),
        }
    }

    /// Create a land specification (defaults to battlefield).
    pub fn land(owner: PlayerId, name: &str) -> Self {
        Self {
            name: name.to_string(),
            card_id: None,
            owner,
            controller: None,
            zone: ZoneId::Battlefield,
            card_types: vec![CardType::Land],
            subtypes: Vec::new(),
            supertypes: Vec::new(),
            colors: Vec::new(),
            power: None,
            toughness: None,
            loyalty: None,
            mana_cost: None,
            tapped: false,
            counters: Vec::new(),
            is_token: false,
            mana_abilities: Vec::new(),
            keywords: Vec::new(),
            activated_abilities: Vec::new(),
            triggered_abilities: Vec::new(),
        }
    }

    /// Create an enchantment specification (defaults to battlefield).
    pub fn enchantment(owner: PlayerId, name: &str) -> Self {
        Self {
            name: name.to_string(),
            card_id: None,
            owner,
            controller: None,
            zone: ZoneId::Battlefield,
            card_types: vec![CardType::Enchantment],
            subtypes: Vec::new(),
            supertypes: Vec::new(),
            colors: Vec::new(),
            power: None,
            toughness: None,
            loyalty: None,
            mana_cost: None,
            tapped: false,
            counters: Vec::new(),
            is_token: false,
            mana_abilities: Vec::new(),
            keywords: Vec::new(),
            activated_abilities: Vec::new(),
            triggered_abilities: Vec::new(),
        }
    }

    /// Create a planeswalker specification (defaults to battlefield).
    pub fn planeswalker(owner: PlayerId, name: &str, loyalty: i32) -> Self {
        Self {
            name: name.to_string(),
            card_id: None,
            owner,
            controller: None,
            zone: ZoneId::Battlefield,
            card_types: vec![CardType::Planeswalker],
            subtypes: Vec::new(),
            supertypes: Vec::new(),
            colors: Vec::new(),
            power: None,
            toughness: None,
            loyalty: Some(loyalty),
            mana_cost: None,
            tapped: false,
            counters: Vec::new(),
            is_token: false,
            mana_abilities: Vec::new(),
            keywords: Vec::new(),
            activated_abilities: Vec::new(),
            triggered_abilities: Vec::new(),
        }
    }

    /// Create a generic card spec (defaults to owner's hand).
    pub fn card(owner: PlayerId, name: &str) -> Self {
        Self {
            name: name.to_string(),
            card_id: None,
            owner,
            controller: None,
            zone: ZoneId::Hand(owner),
            card_types: Vec::new(),
            subtypes: Vec::new(),
            supertypes: Vec::new(),
            colors: Vec::new(),
            power: None,
            toughness: None,
            loyalty: None,
            mana_cost: None,
            tapped: false,
            counters: Vec::new(),
            is_token: false,
            mana_abilities: Vec::new(),
            keywords: Vec::new(),
            activated_abilities: Vec::new(),
            triggered_abilities: Vec::new(),
        }
    }

    // --- Fluent setters ---

    pub fn in_zone(mut self, zone: ZoneId) -> Self {
        self.zone = zone;
        self
    }

    pub fn controlled_by(mut self, controller: PlayerId) -> Self {
        self.controller = Some(controller);
        self
    }

    pub fn tapped(mut self) -> Self {
        self.tapped = true;
        self
    }

    pub fn with_counter(mut self, counter_type: CounterType, count: u32) -> Self {
        self.counters.push((counter_type, count));
        self
    }

    pub fn with_card_id(mut self, card_id: CardId) -> Self {
        self.card_id = Some(card_id);
        self
    }

    pub fn with_types(mut self, types: Vec<CardType>) -> Self {
        self.card_types = types;
        self
    }

    pub fn with_subtypes(mut self, subtypes: Vec<SubType>) -> Self {
        self.subtypes = subtypes;
        self
    }

    pub fn with_supertypes(mut self, supertypes: Vec<SuperType>) -> Self {
        self.supertypes = supertypes;
        self
    }

    pub fn with_colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self
    }

    pub fn with_mana_cost(mut self, cost: ManaCost) -> Self {
        self.mana_cost = Some(cost);
        self
    }

    pub fn token(mut self) -> Self {
        self.is_token = true;
        self
    }

    pub fn with_loyalty(mut self, loyalty: i32) -> Self {
        self.loyalty = Some(loyalty);
        self
    }

    /// Add a mana ability to this object (CR 605).
    pub fn with_mana_ability(mut self, ability: ManaAbility) -> Self {
        self.mana_abilities.push(ability);
        self
    }

    /// Add a keyword ability to this object (CR 702).
    pub fn with_keyword(mut self, keyword: KeywordAbility) -> Self {
        self.keywords.push(keyword);
        self
    }

    /// Add a non-mana activated ability to this object (CR 602).
    pub fn with_activated_ability(mut self, ability: ActivatedAbility) -> Self {
        self.activated_abilities.push(ability);
        self
    }

    /// Add a triggered ability to this object (CR 603).
    pub fn with_triggered_ability(mut self, ability: TriggeredAbilityDef) -> Self {
        self.triggered_abilities.push(ability);
        self
    }
}

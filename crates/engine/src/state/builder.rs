//! Test utility: fluent builders for constructing game states.
//!
//! The builder panics on invalid configuration — it's a test utility, not a
//! production API. Tests should always use `GameStateBuilder` rather than
//! constructing `GameState` structs directly, ensuring all invariants hold.
use super::continuous_effect::ContinuousEffect;
use super::error::GameStateError;
use super::game_object::{
    ActivatedAbility, Characteristics, Designations, GameObject, InterveningIf, ManaAbility,
    ManaCost, ObjectId, ObjectStatus, TriggerEvent, TriggeredAbilityDef,
};
use super::player::{CardId, ManaPool, PlayerId, PlayerState};
use super::replacement_effect::{
    ObjectFilter, ReplacementEffect, ReplacementId, ReplacementModification, ReplacementTrigger,
};
use super::turn::{Step, TurnState};
use super::types::{CardType, Color, CounterType, KeywordAbility, SubType, SuperType};
use super::zone::{Zone, ZoneId};
use super::GameState;
use crate::cards::card_definition::{
    ContinuousEffectDef, Cost, Effect, EffectAmount, EffectTarget, ForEachTarget, PlayerTarget,
};
use crate::cards::CardRegistry;
use crate::state::continuous_effect::{
    EffectDuration as CEDuration, EffectFilter as CEFilter, EffectLayer, LayerModification,
};
use im::{OrdMap, OrdSet, Vector};
use std::sync::Arc;
/// Builder for constructing `GameState` values in tests.
pub struct GameStateBuilder {
    players: Vec<PlayerConfig>,
    objects: Vec<ObjectSpec>,
    continuous_effects: Vec<ContinuousEffect>,
    replacement_effects: Vec<ReplacementEffect>,
    prevention_counters: OrdMap<ReplacementId, u32>,
    turn_number: u32,
    step: Option<Step>,
    active_player: Option<PlayerId>,
    is_first_turn_of_game: bool,
    card_registry: Arc<CardRegistry>,
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
            continuous_effects: Vec::new(),
            replacement_effects: Vec::new(),
            prevention_counters: im::OrdMap::new(),
            turn_number: 1,
            step: None,
            active_player: None,
            is_first_turn_of_game: false,
            card_registry: CardRegistry::empty(),
        }
    }
    /// Set the card registry for effect execution in tests that use real card definitions.
    pub fn with_registry(mut self, registry: Arc<CardRegistry>) -> Self {
        self.card_registry = registry;
        self
    }
    /// Create a builder pre-configured with 4 players (IDs 1-4) at 40 life.
    pub fn four_player() -> Self {
        Self::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .add_player(PlayerId(3))
            .add_player(PlayerId(4))
    }
    /// Create a builder pre-configured with 6 players (IDs 1-6) at 40 life.
    ///
    /// Used by 6-player tests to validate that the engine's multiplayer systems
    /// (priority rotation, APNAP ordering, combat with multiple defending players,
    /// turn advancement skipping eliminated players) work correctly at N=6.
    pub fn six_player() -> Self {
        Self::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .add_player(PlayerId(3))
            .add_player(PlayerId(4))
            .add_player(PlayerId(5))
            .add_player(PlayerId(6))
    }
    /// Add a player with Commander starting life (40).
    pub fn add_player(mut self, id: PlayerId) -> Self {
        // MR-M1-17: catch duplicate PlayerId in debug/test builds.
        debug_assert!(
            !self.players.iter().any(|p| p.id == id),
            "add_player: duplicate PlayerId {:?}",
            id
        );
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
        // MR-M1-16: catch typo'd PlayerId in debug/test builds.
        let mut found = false;
        for p in &mut self.players {
            if p.id == id {
                p.life_total = life;
                found = true;
            }
        }
        debug_assert!(found, "player_life: PlayerId {:?} not found in builder", id);
        self
    }
    /// Set a player's poison counters.
    pub fn player_poison(mut self, id: PlayerId, counters: u32) -> Self {
        let mut found = false;
        for p in &mut self.players {
            if p.id == id {
                p.poison_counters = counters;
                found = true;
            }
        }
        debug_assert!(
            found,
            "player_poison: PlayerId {:?} not found in builder",
            id
        );
        self
    }
    /// Set a player's mana pool.
    pub fn player_mana(mut self, id: PlayerId, pool: ManaPool) -> Self {
        let mut found = false;
        for p in &mut self.players {
            if p.id == id {
                p.mana_pool = pool.clone();
                found = true;
            }
        }
        debug_assert!(found, "player_mana: PlayerId {:?} not found in builder", id);
        self
    }
    /// Register a commander for a player.
    pub fn player_commander(mut self, id: PlayerId, card_id: CardId) -> Self {
        let mut found = false;
        for p in &mut self.players {
            if p.id == id {
                p.commander_ids.push(card_id.clone());
                found = true;
            }
        }
        debug_assert!(
            found,
            "player_commander: PlayerId {:?} not found in builder",
            id
        );
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
    /// Add a continuous effect to the game state (for layer system tests in M5+).
    ///
    /// Effects added via this method are placed directly into `state.continuous_effects`
    /// with their `id`, `source`, and `timestamp` taken as-is from the provided effect.
    pub fn add_continuous_effect(mut self, effect: ContinuousEffect) -> Self {
        self.continuous_effects.push(effect);
        self
    }
    /// Add a replacement effect to the game state (for M8+ replacement/prevention tests).
    ///
    /// Effects added via this method are placed directly into `state.replacement_effects`
    /// with their `id` taken as-is from the provided effect. The `next_replacement_id`
    /// counter is advanced past any pre-set IDs.
    pub fn with_replacement_effect(mut self, effect: ReplacementEffect) -> Self {
        self.replacement_effects.push(effect);
        self
    }
    /// Register a prevention shield counter for a `PreventDamage(n)` replacement effect.
    ///
    /// Used in tests to set up a shield with a specific remaining capacity.
    /// The corresponding `ReplacementEffect` must also be added via `with_replacement_effect`.
    pub fn with_prevention_counter(mut self, id: ReplacementId, n: u32) -> Self {
        self.prevention_counters.insert(id, n);
        self
    }
    /// Build the `GameState`. Returns `Err` if configuration is invalid (e.g. no players).
    pub fn build(self) -> Result<GameState, GameStateError> {
        if self.players.is_empty() {
            return Err(GameStateError::InvalidCommand(
                "must have at least one player".to_string(),
            ));
        }
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
                companion: None,
                companion_used: false,
                mulligan_count: 0,
                no_max_hand_size: false,
                cards_drawn_this_turn: 0,
                spells_cast_this_turn: 0,
                has_citys_blessing: false,
                life_lost_this_turn: 0,
                damage_received_this_turn: 0,
                protection_qualities: vec![],
                dungeons_completed: 0,
                dungeons_completed_set: im::OrdSet::new(),
                ring_level: 0,
                ring_bearer_id: None,
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
            additional_phases: Vector::new(),
            in_extra_combat: false,
            is_first_turn_of_game: self.is_first_turn_of_game,
            last_regular_active: active_player,
            cleanup_sba_rounds: 0,
        };
        let mut state = GameState {
            turn,
            players,
            zones,
            objects: OrdMap::new(),
            continuous_effects: Vector::new(),
            delayed_triggers: Vector::new(),
            replacement_effects: Vector::new(),
            next_replacement_id: 0,
            pending_zone_changes: Vector::new(),
            pending_commander_zone_choices: Vector::new(),
            prevention_counters: OrdMap::new(),
            pending_triggers: Vector::new(),
            trigger_doublers: Vector::new(),
            etb_suppressors: Vector::new(),
            restrictions: Vector::new(),
            stack_objects: Vector::new(),
            combat: None,
            timestamp_counter: 0,
            loop_detection_hashes: OrdMap::new(),
            history: Vector::new(),
            permanents_put_into_graveyard_this_turn: 0,
            pending_echo_payments: Vector::new(),
            pending_cumulative_upkeep_payments: Vector::new(),
            pending_recover_payments: Vector::new(),
            forecast_used_this_turn: im::OrdSet::new(),
            // CR 730.1: Game starts with neither day nor night.
            day_night: None,
            // CR 730.2: No previous turn spells cast at game start.
            previous_turn_spells_cast: 0,
            // CR 309.4: No player has a dungeon in their command zone at game start.
            dungeon_state: OrdMap::new(),
            // CR 725.1: No player has the initiative at game start.
            has_initiative: None,
            // CR 724.1: No player is the monarch at game start.
            monarch: None,
            card_registry: self.card_registry,
        };
        // Add continuous effects
        for effect in self.continuous_effects {
            state.continuous_effects.push_back(effect);
        }
        // Add replacement effects and advance the ID counter past any pre-set IDs
        for effect in self.replacement_effects {
            if effect.id.0 >= state.next_replacement_id {
                state.next_replacement_id = effect.id.0 + 1;
            }
            state.replacement_effects.push_back(effect);
        }
        // Copy any pre-set prevention shield counters (from with_prevention_counter).
        state.prevention_counters = self.prevention_counters;
        // Add objects
        for spec in self.objects {
            let owner = spec.owner;
            let controller = spec.controller.unwrap_or(owner);
            let zone = spec.zone;
            // CR 702.21a: Translate Ward keyword into a TriggeredAbilityDef.
            // Ward generates a triggered ability at object-construction time: "Whenever
            // this permanent becomes the target of a spell or ability an opponent controls,
            // counter that spell or ability unless that player pays {N}."
            // MayPayOrElse currently always applies or_else (deterministic non-interactive);
            // interactive payment is deferred to M10+.
            let mut triggered_abilities: Vec<TriggeredAbilityDef> =
                spec.triggered_abilities.into_iter().collect();
            let spec_keywords: im::OrdSet<KeywordAbility> = spec.keywords.iter().cloned().collect();
            for kw in spec.keywords.iter() {
                if let KeywordAbility::Ward(cost_n) = kw {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfBecomesTargetByOpponent,
                        intervening_if: None,
                        description: format!(
                            "Ward {{{cost_n}}} (CR 702.21a): counter unless opponent pays"
                        ),
                        effect: Some(Effect::MayPayOrElse {
                            cost: Cost::Mana(ManaCost {
                                generic: *cost_n,
                                ..Default::default()
                            }),
                            // CR 702.21a: "counter unless that player pays" — "that player"
                            // is the controller of the targeting spell/ability (the opponent),
                            // NOT the ward creature's controller. DeclaredTarget { index: 0 }
                            // is the targeting spell/ability on the stack (set by
                            // flush_pending_triggers via targeting_stack_id).
                            payer: PlayerTarget::ControllerOf(Box::new(
                                EffectTarget::DeclaredTarget { index: 0 },
                            )),
                            or_else: Box::new(Effect::CounterSpell {
                                target: EffectTarget::DeclaredTarget { index: 0 },
                            }),
                        }),
                    });
                }
                // CR 702.108a: Prowess — "Whenever you cast a noncreature spell, this
                // creature gets +1/+1 until end of turn."
                // Each keyword instance generates one TriggeredAbilityDef.
                if matches!(kw, KeywordAbility::Prowess) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::ControllerCastsNoncreatureSpell,
                        intervening_if: None,
                        description: "Prowess (CR 702.108a): Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.".to_string(),
                        effect: Some(Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::PtModify,
                                modification: LayerModification::ModifyBoth(1),
                                filter: CEFilter::Source,
                                duration: CEDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        }),
                    });
                }
                // CR 702.83a: Exalted — "Whenever a creature you control attacks alone,
                // that creature gets +1/+1 until end of turn."
                // Each keyword instance generates one TriggeredAbilityDef.
                // The +1/+1 targets the lone attacker (DeclaredTarget { index: 0 }),
                // not the permanent with exalted (which may not even be a creature).
                if matches!(kw, KeywordAbility::Exalted) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::ControllerCreatureAttacksAlone,
                        intervening_if: None,
                        description: "Exalted (CR 702.83a): Whenever a creature you control attacks alone, that creature gets +1/+1 until end of turn.".to_string(),
                        effect: Some(Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::PtModify,
                                modification: LayerModification::ModifyBoth(1),
                                filter: CEFilter::DeclaredTarget { index: 0 },
                                duration: CEDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        }),
                    });
                }
                // CR 702.86a: Annihilator N — "Whenever this creature attacks, defending
                // player sacrifices N permanents."
                // Each keyword instance generates one TriggeredAbilityDef (CR 702.86b).
                // The effect targets the defending player (DeclaredTarget { index: 0 }),
                // which is resolved at flush time via PendingTrigger.defending_player_id.
                if let KeywordAbility::Annihilator(n) = kw {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfAttacks,
                        intervening_if: None,
                        description: format!(
                            "Annihilator {n} (CR 702.86a): Whenever this creature attacks, \
                             defending player sacrifices {n} permanents."
                        ),
                        effect: Some(Effect::SacrificePermanents {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            count: EffectAmount::Fixed(*n as i32),
                        }),
                    });
                }
                // CR 702.91a: Battle Cry — "Whenever this creature attacks, each
                // other attacking creature gets +1/+0 until end of turn."
                // Each keyword instance generates one TriggeredAbilityDef (CR 702.91b).
                // The ForEach iterates over all other attacking creatures at resolution
                // time and applies a +1/+0 ModifyPower continuous effect to each.
                if matches!(kw, KeywordAbility::BattleCry) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfAttacks,
                        intervening_if: None,
                        description: "Battle Cry (CR 702.91a): Whenever this creature attacks, \
                                      each other attacking creature gets +1/+0 until end of turn."
                            .to_string(),
                        effect: Some(Effect::ForEach {
                            over: ForEachTarget::EachOtherAttackingCreature,
                            effect: Box::new(Effect::ApplyContinuousEffect {
                                effect_def: Box::new(ContinuousEffectDef {
                                    layer: EffectLayer::PtModify,
                                    modification: LayerModification::ModifyPower(1),
                                    filter: CEFilter::DeclaredTarget { index: 0 },
                                    duration: CEDuration::UntilEndOfTurn,
                                    condition: None,
                                }),
                            }),
                        }),
                    });
                }
                // CR 702.105a: Dethrone -- "Whenever this creature attacks the player
                // with the most life or tied for most life, put a +1/+1 counter on
                // this creature."
                // Each keyword instance generates one TriggeredAbilityDef (CR 702.105b).
                // The trigger uses SelfAttacksPlayerWithMostLife, a dedicated event
                // that is only dispatched in abilities.rs when the defending player
                // has the most life (or is tied). This avoids unconditional firing
                // on all attacks.
                if matches!(kw, KeywordAbility::Dethrone) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfAttacksPlayerWithMostLife,
                        intervening_if: None,
                        description: "Dethrone (CR 702.105a): Whenever this creature attacks \
                                      the player with the most life or tied for most life, \
                                      put a +1/+1 counter on this creature."
                            .to_string(),
                        effect: Some(Effect::AddCounter {
                            target: EffectTarget::Source,
                            counter: CounterType::PlusOnePlusOne,
                            count: 1,
                        }),
                    });
                }
                // CR 702.149a: Training -- "Whenever this creature and at least one
                // other creature with power greater than this creature's power attack,
                // put a +1/+1 counter on this creature."
                // Each keyword instance generates one TriggeredAbilityDef (CR 702.149b).
                // The trigger uses SelfAttacksWithGreaterPowerAlly, a dedicated event
                // that is only dispatched in abilities.rs when a co-attacker with
                // strictly greater power exists. This avoids unconditional firing on
                // all attacks.
                if matches!(kw, KeywordAbility::Training) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfAttacksWithGreaterPowerAlly,
                        intervening_if: None,
                        description: "Training (CR 702.149a): Whenever this creature and at \
                                      least one other creature with greater power attack, put \
                                      a +1/+1 counter on this creature."
                            .to_string(),
                        effect: Some(Effect::AddCounter {
                            target: EffectTarget::Source,
                            counter: CounterType::PlusOnePlusOne,
                            count: 1,
                        }),
                    });
                }
                // CR 702.121a: Melee -- "Whenever this creature attacks, it gets
                // +1/+1 until end of turn for each opponent you attacked with a
                // creature this combat."
                // Each keyword instance generates one TriggeredAbilityDef (CR 702.121b).
                // The effect is None because resolution is handled by the custom
                // KeywordTrigger (Melee) StackObjectKind -- the bonus is computed at resolution
                // time from combat state (ruling 2016-08-23).
                if matches!(kw, KeywordAbility::Melee) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfAttacks,
                        intervening_if: None,
                        description: "Melee (CR 702.121a): Whenever this creature attacks, it \
                                      gets +1/+1 until end of turn for each opponent you \
                                      attacked with a creature this combat."
                            .to_string(),
                        effect: None, // Custom resolution via KeywordTrigger (Melee)
                    });
                }
                // CR 702.154a: Enlist -- "As this creature attacks, you may tap up to
                // one untapped creature [...]. When you do, this creature gets +X/+0
                // until end of turn, where X is the tapped creature's power."
                // The static ability (optional cost) is handled in combat.rs via the
                // enlist_choices field on DeclareAttackers. The triggered ability is
                // handled by creating an EnlistTrigger StackObjectKind at trigger-
                // collection time in abilities.rs.
                // builder.rs generates a placeholder TriggeredAbilityDef so
                // ability_index is valid. The effect is None because resolution is
                // custom (EnlistTrigger reads the enlisted creature's power).
                // CR 702.154d: Each Enlist instance generates one placeholder.
                if matches!(kw, KeywordAbility::Enlist) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfAttacks,
                        intervening_if: None,
                        description: "Enlist (CR 702.154a): As this creature attacks, you may \
                                      tap an untapped non-attacking creature you control. When \
                                      you do, this creature gets +X/+0 until end of turn, where \
                                      X is the tapped creature's power."
                            .to_string(),
                        effect: None, // Custom resolution via EnlistTrigger
                    });
                }
                // CR 702.70a: Poisonous N -- trigger dispatch is handled manually in
                // abilities.rs CombatDamageDealt handler (same as Ingest / Renown).
                // No TriggeredAbilityDef is registered here to avoid double-triggering:
                // the manual dispatch block already creates the PoisonousTrigger StackObjectKind
                // with the correct N value and target player carried as fields.
                // CR 702.79a: Persist — "When this permanent is put into a graveyard from
                // the battlefield, if it had no -1/-1 counters on it, return it to the
                // battlefield under its owner's control with a -1/-1 counter on it."
                // Each keyword instance generates one TriggeredAbilityDef.
                // The intervening-if is checked at trigger time against pre_death_counters
                // carried by the CreatureDied event (last known information, CR 603.10a).
                if matches!(kw, KeywordAbility::Persist) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfDies,
                        intervening_if: Some(InterveningIf::SourceHadNoCounterOfType(
                            CounterType::MinusOneMinusOne,
                        )),
                        description: "Persist (CR 702.79a): When this permanent dies, \
                                      if it had no -1/-1 counters on it, return it to the \
                                      battlefield under its owner's control with a -1/-1 \
                                      counter on it."
                            .to_string(),
                        effect: Some(Effect::Sequence(vec![
                            Effect::MoveZone {
                                target: EffectTarget::Source,
                                to: crate::cards::card_definition::ZoneTarget::Battlefield {
                                    tapped: false,
                                },
                                controller_override: None,
                            },
                            Effect::AddCounter {
                                target: EffectTarget::Source,
                                counter: CounterType::MinusOneMinusOne,
                                count: 1,
                            },
                        ])),
                    });
                }
                // CR 702.93a: Undying -- "When this permanent is put into a graveyard from
                // the battlefield, if it had no +1/+1 counters on it, return it to the
                // battlefield under its owner's control with a +1/+1 counter on it."
                // Each keyword instance generates one TriggeredAbilityDef.
                // The intervening-if is checked at trigger time against pre_death_counters
                // carried by the CreatureDied event (last known information, CR 603.10a).
                if matches!(kw, KeywordAbility::Undying) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfDies,
                        intervening_if: Some(InterveningIf::SourceHadNoCounterOfType(
                            CounterType::PlusOnePlusOne,
                        )),
                        description: "Undying (CR 702.93a): When this permanent dies, \
                                      if it had no +1/+1 counters on it, return it to the \
                                      battlefield under its owner's control with a +1/+1 \
                                      counter on it."
                            .to_string(),
                        effect: Some(Effect::Sequence(vec![
                            Effect::MoveZone {
                                target: EffectTarget::Source,
                                to: crate::cards::card_definition::ZoneTarget::Battlefield {
                                    tapped: false,
                                },
                                controller_override: None,
                            },
                            Effect::AddCounter {
                                target: EffectTarget::Source,
                                counter: CounterType::PlusOnePlusOne,
                                count: 1,
                            },
                        ])),
                    });
                }
                // CR 702.135a: Afterlife N -- "When this permanent is put into a
                // graveyard from the battlefield, create N 1/1 white and black Spirit
                // creature tokens with flying."
                // Each keyword instance generates one TriggeredAbilityDef (CR 702.135b).
                // No intervening-if: the trigger always fires on death regardless of counters.
                if let KeywordAbility::Afterlife(n) = kw {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfDies,
                        intervening_if: None,
                        description: format!(
                            "Afterlife {n} (CR 702.135a): When this permanent dies, \
                             create {n} 1/1 white and black Spirit creature token(s) \
                             with flying."
                        ),
                        effect: Some(Effect::CreateToken {
                            spec: crate::cards::card_definition::TokenSpec {
                                name: "Spirit".to_string(),
                                power: 1,
                                toughness: 1,
                                colors: [Color::White, Color::Black].into_iter().collect(),
                                supertypes: im::OrdSet::new(),
                                card_types: [CardType::Creature].into_iter().collect(),
                                subtypes: [SubType("Spirit".to_string())].into_iter().collect(),
                                keywords: [KeywordAbility::Flying].into_iter().collect(),
                                count: *n,
                                tapped: false,
                                enters_attacking: false,
                                mana_color: None,
                                mana_abilities: vec![],
                                activated_abilities: vec![],
                            },
                        }),
                    });
                }
                // CR 702.101a: Extort — "Whenever you cast a spell, you may pay
                // {W/B}. If you do, each opponent loses 1 life and you gain life
                // equal to the total life lost this way."
                // Each keyword instance generates one TriggeredAbilityDef (CR 702.101b).
                // The "may pay" optional cost is deferred to interactive mode (M10+);
                // deterministic fallback always resolves the drain effect.
                if matches!(kw, KeywordAbility::Extort) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::ControllerCastsSpell,
                        intervening_if: None,
                        description: "Extort (CR 702.101a): Whenever you cast a spell, \
                                      you may pay {W/B}. If you do, each opponent loses 1 \
                                      life and you gain that much life."
                            .to_string(),
                        effect: Some(Effect::DrainLife {
                            amount: EffectAmount::Fixed(1),
                        }),
                    });
                }
                // CR 702.92a: Living Weapon -- "When this Equipment enters, create a
                // 0/0 black Phyrexian Germ creature token, then attach this Equipment
                // to it."
                // ETB trigger on the Equipment itself. Uses CreateTokenAndAttachSource
                // to atomically create + attach before SBAs.
                if matches!(kw, KeywordAbility::LivingWeapon) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfEntersBattlefield,
                        intervening_if: None,
                        description: "Living Weapon (CR 702.92a): When this Equipment enters, \
                                      create a 0/0 black Phyrexian Germ creature token, then \
                                      attach this Equipment to it."
                            .to_string(),
                        effect: Some(Effect::CreateTokenAndAttachSource {
                            spec: crate::cards::card_definition::TokenSpec {
                                name: "Phyrexian Germ".to_string(),
                                power: 0,
                                toughness: 0,
                                colors: [Color::Black].into_iter().collect(),
                                supertypes: im::OrdSet::new(),
                                card_types: [CardType::Creature].into_iter().collect(),
                                subtypes: [
                                    SubType("Phyrexian".to_string()),
                                    SubType("Germ".to_string()),
                                ]
                                .into_iter()
                                .collect(),
                                keywords: im::OrdSet::new(),
                                count: 1,
                                tapped: false,
                                enters_attacking: false,
                                mana_color: None,
                                mana_abilities: vec![],
                                activated_abilities: vec![],
                            },
                        }),
                    });
                }
                // CR 702.43a: Modular N -- "When this permanent is put into a graveyard from
                // the battlefield, you may put a +1/+1 counter on target artifact creature
                // for each +1/+1 counter on this permanent."
                // Each Modular instance generates one TriggeredAbilityDef (CR 702.43b).
                // The effect is NOT encoded here because the counter count is dynamic
                // (based on pre_death_counters, not the static N). Resolution is handled
                // by StackObjectKind::KeywordTrigger (Modular).
                if let KeywordAbility::Modular(_n) = kw {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfDies,
                        intervening_if: None,
                        description: "Modular (CR 702.43a): When this permanent is put into a \
                                      graveyard from the battlefield, you may put a +1/+1 counter \
                                      on target artifact creature for each +1/+1 counter on this \
                                      permanent."
                            .to_string(),
                        effect: None, // Handled by KeywordTrigger (Modular) resolution
                    });
                }
                // CR 702.116a: Myriad -- "Whenever this creature attacks, for each opponent
                // other than defending player, you may create a token that's a copy of this
                // creature that's tapped and attacking that player or a planeswalker they
                // control. If one or more tokens are created this way, exile the tokens at
                // end of combat."
                // Each keyword instance generates one TriggeredAbilityDef (CR 702.116b).
                // Token creation logic is handled by StackObjectKind::KeywordTrigger (Myriad) at
                // resolution time in resolution.rs. End-of-combat exile is handled by
                // end_combat() in turn_actions.rs.
                if matches!(kw, KeywordAbility::Myriad) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfAttacks,
                        intervening_if: None,
                        description: "Myriad (CR 702.116a): Whenever this creature attacks, \
                                      for each opponent other than defending player, create a \
                                      token copy tapped and attacking that player. Exile tokens \
                                      at end of combat."
                            .to_string(),
                        effect: None, // Handled by KeywordTrigger (Myriad) resolution
                    });
                }
                // CR 702.45a: Bushido N -- "Whenever this creature blocks or becomes
                // blocked, it gets +N/+N until end of turn."
                // Two TriggeredAbilityDefs per Bushido instance: one for SelfBlocks,
                // one for SelfBecomesBlocked. Each triggers separately (CR 702.45b).
                if let KeywordAbility::Bushido(n) = kw {
                    let bushido_effect = Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(*n as i32),
                            filter: CEFilter::Source,
                            duration: CEDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    };
                    // Trigger 1: "Whenever this creature blocks"
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfBlocks,
                        intervening_if: None,
                        description: format!(
                            "Bushido {n} (CR 702.45a): Whenever this creature blocks, \
                             it gets +{n}/+{n} until end of turn."
                        ),
                        effect: Some(bushido_effect.clone()),
                    });
                    // Trigger 2: "Whenever this creature becomes blocked"
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfBecomesBlocked,
                        intervening_if: None,
                        description: format!(
                            "Bushido {n} (CR 702.45a): Whenever this creature becomes blocked, \
                             it gets +{n}/+{n} until end of turn."
                        ),
                        effect: Some(bushido_effect),
                    });
                }
                // CR 702.23a: Rampage N -- "Whenever this creature becomes blocked,
                // it gets +N/+N until end of turn for each creature blocking it
                // beyond the first."
                // Each Rampage(n) keyword instance generates one TriggeredAbilityDef
                // (CR 702.23c: multiple instances trigger separately).
                // The effect is None because resolution is handled by the custom
                // KeywordTrigger (Rampage) StackObjectKind -- the bonus is computed at resolution
                // time from combat state (CR 702.23b).
                if let KeywordAbility::Rampage(n) = kw {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfBecomesBlocked,
                        intervening_if: None,
                        description: format!(
                            "Rampage {n} (CR 702.23a): Whenever this creature becomes blocked, \
                             it gets +{n}/+{n} until end of turn for each creature blocking \
                             it beyond the first."
                        ),
                        effect: None, // Custom resolution via KeywordTrigger (Rampage)
                    });
                }
                // CR 702.39a: Provoke -- "Whenever this creature attacks, you may
                // have target creature defending player controls untap and block this
                // creature this combat if able."
                // Each keyword instance generates one TriggeredAbilityDef (CR 702.39b).
                // The actual untap + forced-block logic is in StackObjectKind::KeywordTrigger (Provoke)
                // resolution. The description starts with "Provoke" so abilities.rs can
                // identify and tag it as a provoke trigger at collection time.
                if matches!(kw, KeywordAbility::Provoke) {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfAttacks,
                        intervening_if: None,
                        description: "Provoke (CR 702.39a): Whenever this creature attacks, \
                                      you may have target creature defending player controls \
                                      untap and block this creature this combat if able."
                            .to_string(),
                        effect: None, // Handled by KeywordTrigger (Provoke) resolution
                    });
                }
                // CR 702.130a: Afflict N -- "Whenever this creature becomes blocked,
                // defending player loses N life."
                // Each keyword instance generates one TriggeredAbilityDef (CR 702.130b).
                // The effect targets the defending player (DeclaredTarget { index: 0 }),
                // which is resolved at flush time via PendingTrigger.defending_player_id.
                if let KeywordAbility::Afflict(n) = kw {
                    triggered_abilities.push(TriggeredAbilityDef {
                        etb_filter: None,
                        death_filter: None,
                        combat_damage_filter: None,
                        targets: vec![],
                        trigger_on: TriggerEvent::SelfBecomesBlocked,
                        intervening_if: None,
                        description: format!(
                            "Afflict {n} (CR 702.130a): Whenever this creature becomes blocked, \
                             defending player loses {n} life."
                        ),
                        effect: Some(Effect::LoseLife {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(*n as i32),
                        }),
                    });
                }
            }
            let characteristics = Characteristics {
                name: spec.name,
                mana_cost: spec.mana_cost,
                colors: spec.colors.into_iter().collect(),
                color_indicator: None,
                supertypes: spec.supertypes.into_iter().collect(),
                card_types: spec.card_types.into_iter().collect(),
                subtypes: spec.subtypes.into_iter().collect(),
                rules_text: spec.rules_text,
                abilities: Vector::new(),
                keywords: spec_keywords,
                mana_abilities: spec.mana_abilities.into_iter().collect(),
                activated_abilities: spec.activated_abilities.into_iter().collect(),
                triggered_abilities,
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
                damage_marked: spec.damage_marked,
                deathtouch_damage: spec.deathtouch_damage,
                is_token: spec.is_token,
                is_emblem: false,
                timestamp: 0, // Assigned by add_object
                // Summoning sickness is set based on zone when the object is added.
                // test-placed permanents on the battlefield are treated as having
                // been there since the beginning of their controller's turn.
                has_summoning_sickness: false,
                goaded_by: im::Vector::new(),
                kicker_times_paid: 0,
                cast_alt_cost: None,
                foretold_turn: 0,
                was_unearthed: false,
                myriad_exile_at_eoc: false,
                decayed_sacrifice_at_eoc: false,
                ring_block_sacrifice_at_eoc: false,
                exiled_by_hideaway: None,
                encore_sacrifice_at_end_step: false,
                encore_must_attack: None,
                encore_activated_by: None,
                is_plotted: false,
                plotted_turn: 0,
                is_prototyped: false,
                was_bargained: false,
                evidence_collected: false,
                phased_out_indirectly: false,
                phased_out_controller: None,
                creatures_devoured: 0,
                champion_exiled_card: None,
                // CR 702.95b: objects start unpaired.
                paired_with: None,
                // CR 702.104b: tribute_was_paid starts false.
                tribute_was_paid: false,
                // CR 107.3m: test-placed objects are not cast spells; x_value is 0.
                x_value: 0,
                // CR 702.157a: test-placed objects are not cast spells; squad_count is 0.
                squad_count: 0,
                // CR 702.175a: test-placed objects are not cast spells; offspring_paid is false.
                offspring_paid: false,
                // CR 702.174a: test-placed objects are not cast spells; gift fields are false/None.
                gift_was_given: false,
                gift_opponent: None,
                // CR 702.99b: test-placed objects have no encoded cipher cards.
                encoded_cards: im::Vector::new(),
                // CR 702.55b: test-placed objects have no haunting relationship.
                haunting_target: None,
                // CR 729.2: test-placed objects are not part of a merged permanent.
                merged_components: im::Vector::new(),
                // CR 712.8d: test-placed objects start with front face up.
                is_transformed: false,
                // CR 701.27f: test-placed objects have not transformed.
                last_transform_timestamp: 0,
                // CR 702.146 ruling: test-placed objects were not cast via disturb.
                was_cast_disturbed: false,
                // CR 702.167c: test-placed objects have no craft materials.
                craft_exiled_cards: im::Vector::new(),
                // CR 708.2: test-placed objects are not morph/manifest/cloak face-down.
                chosen_creature_type: None,
                face_down_as: None,
                // CR 606.3: loyalty abilities not yet activated.
                loyalty_ability_activated_this_turn: false,
                class_level: 0,
                designations: Designations::default(),
                adventure_exiled_by: None,
                meld_component: None,
            };
            state.add_object(object, zone)?;
        }
        Ok(state)
    }
}
/// Register replacement effects for commander zone changes (CR 903.9b).
///
/// CR 903.9a (graveyard/exile): These paths are now handled by a state-based
/// action (`check_commander_zone_return_sba` in `sba.rs`), NOT replacement
/// effects. The M8 replacement registrations for graveyard and exile have been
/// removed as part of the M9 SBA model update.
///
/// CR 903.9b (hand/library): These two paths ARE replacement effects because
/// CR 903.9b says "instead." For each player's commander (identified by `CardId`),
/// registers two replacement effects: one redirecting hand-bound moves to the
/// command zone, one redirecting library-bound moves to the command zone.
/// These are `Indefinite` duration effects with `is_self_replacement: false`.
pub fn register_commander_zone_replacements(state: &mut GameState) {
    use super::continuous_effect::EffectDuration;
    use super::zone::ZoneType;
    let commanders: Vec<(PlayerId, CardId)> = state
        .players
        .iter()
        .flat_map(|(&pid, ps)| ps.commander_ids.iter().map(move |cid| (pid, cid.clone())))
        .collect();
    for (owner, card_id) in commanders {
        // CR 903.9b: commander would go to hand → may go to command zone instead.
        let id_hand = state.next_replacement_id();
        state.replacement_effects.push_back(ReplacementEffect {
            id: id_hand,
            source: None,
            controller: owner,
            duration: EffectDuration::Indefinite,
            is_self_replacement: false,
            trigger: ReplacementTrigger::WouldChangeZone {
                from: None,
                to: ZoneType::Hand,
                filter: ObjectFilter::HasCardId(card_id.clone()),
            },
            modification: ReplacementModification::RedirectToZone(ZoneType::Command),
        });
        // CR 903.9b: commander would go to library → may go to command zone instead.
        let id_library = state.next_replacement_id();
        state.replacement_effects.push_back(ReplacementEffect {
            id: id_library,
            source: None,
            controller: owner,
            duration: EffectDuration::Indefinite,
            is_self_replacement: false,
            trigger: ReplacementTrigger::WouldChangeZone {
                from: None,
                to: ZoneType::Library,
                filter: ObjectFilter::HasCardId(card_id),
            },
            modification: ReplacementModification::RedirectToZone(ZoneType::Command),
        });
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
    /// Oracle / rules text for display.
    pub rules_text: String,
    /// Pre-marked damage (for constructing test states, M4+).
    pub damage_marked: u32,
    /// Pre-set deathtouch damage flag (for constructing test states, M4+).
    pub deathtouch_damage: bool,
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
            rules_text: String::new(),
            damage_marked: 0,
            deathtouch_damage: false,
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
            rules_text: String::new(),
            damage_marked: 0,
            deathtouch_damage: false,
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
            rules_text: String::new(),
            damage_marked: 0,
            deathtouch_damage: false,
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
            rules_text: String::new(),
            damage_marked: 0,
            deathtouch_damage: false,
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
            rules_text: String::new(),
            damage_marked: 0,
            deathtouch_damage: false,
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
            rules_text: String::new(),
            damage_marked: 0,
            deathtouch_damage: false,
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
    /// Set marked damage on this object (for constructing test states, M4+).
    pub fn with_damage(mut self, amount: u32) -> Self {
        self.damage_marked = amount;
        self
    }
    /// Mark that this object has been dealt deathtouch damage (for constructing test states, M4+).
    pub fn with_deathtouch_damage(mut self) -> Self {
        self.deathtouch_damage = true;
        self
    }
}

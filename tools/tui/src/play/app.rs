//! App state for the interactive play mode.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, start_game, CardDefinition, CardRegistry,
    Command, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, ZoneId,
};
use mtg_simulator::{
    random_deck, Bot, HeuristicBot, LegalAction, LegalActionProvider, RandomBot, StubProvider,
};
use rand::prelude::*;

/// Input mode — determines what keys do.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum InputMode {
    Normal,
    AttackerDeclaration,
    BlockerDeclaration,
    CardDetail(ObjectId),
}

/// An entry in the scrollable event log.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct LogEntry {
    pub text: String,
    pub turn: u32,
}

pub struct PlayApp {
    pub state: GameState,
    pub human_player: PlayerId,
    pub provider: StubProvider,
    pub bots: HashMap<PlayerId, Box<dyn Bot>>,
    pub should_quit: bool,
    pub mode: InputMode,
    pub event_log: Vec<LogEntry>,
    pub log_scroll: usize,
    pub selected_hand_idx: usize,
    pub selected_bf_idx: usize,
    pub focused_player: PlayerId,
    pub bot_delay_ms: u64,
    pub status_message: Option<String>,
    pub consecutive_passes: u32,
    pub _player_count: u32,
    _registry: Arc<CardRegistry>,
}

/// Maximum consecutive passes before declaring a stuck game.
const MAX_CONSECUTIVE_PASSES: u32 = 500;

impl PlayApp {
    pub fn new(player_count: u32, bot_type: &str) -> anyhow::Result<Self> {
        let cards = all_cards();
        let registry = CardRegistry::new(cards.clone());
        let human_player = PlayerId(1);
        let mut rng = StdRng::from_entropy();

        // Build initial state with populated libraries
        let mut builder = GameStateBuilder::new().with_registry(registry.clone());
        let player_ids: Vec<PlayerId> =
            (1..=player_count).map(|i| PlayerId(i as u64)).collect();

        for &pid in &player_ids {
            builder = builder.add_player(pid);
        }

        // Build name→def lookup for enriching card specs
        let card_defs: HashMap<String, CardDefinition> =
            cards.iter().map(|c| (c.name.clone(), c.clone())).collect();

        // Give each player a random deck
        for &pid in &player_ids {
            if let Some(deck) = random_deck(&mut rng, &cards) {
                // Commander in command zone
                if let Some(def) = cards.iter().find(|c| c.card_id == deck.commander) {
                    let spec = ObjectSpec::card(pid, &def.name)
                        .in_zone(ZoneId::Command(pid))
                        .with_card_id(deck.commander.clone());
                    let spec = enrich_spec_from_def(spec, &card_defs);
                    builder = builder.object(spec);
                }

                // Main deck cards in library
                for card_id in &deck.main_deck {
                    if let Some(def) = cards.iter().find(|c| c.card_id == *card_id) {
                        let spec = ObjectSpec::card(pid, &def.name)
                            .in_zone(ZoneId::Library(pid))
                            .with_card_id(card_id.clone());
                        let spec = enrich_spec_from_def(spec, &card_defs);
                        builder = builder.object(spec);
                    }
                }
            }
        }

        builder = builder.first_turn_of_game();
        let state = builder.build()?;

        // Create bots for non-human players
        let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
        for i in 2..=player_count {
            let pid = PlayerId(i as u64);
            let seed = rng.gen();
            let name = format!("Bot-{}", i);
            let bot: Box<dyn Bot> = match bot_type {
                "heuristic" => Box::new(HeuristicBot::new(seed, name)),
                _ => Box::new(RandomBot::new(seed, name)),
            };
            bots.insert(pid, bot);
        }

        Ok(Self {
            state,
            human_player,
            provider: StubProvider,
            bots,
            should_quit: false,
            mode: InputMode::Normal,
            event_log: Vec::new(),
            log_scroll: 0,
            selected_hand_idx: 0,
            selected_bf_idx: 0,
            focused_player: human_player,
            bot_delay_ms: 200,
            status_message: None,
            consecutive_passes: 0,
            _player_count: player_count,
            _registry: registry,
        })
    }

    pub fn start_game(&mut self) -> anyhow::Result<()> {
        let (new_state, events) = start_game(self.state.clone())?;
        self.state = new_state;
        self.log_events(&events);
        Ok(())
    }

    pub fn game_over(&self) -> bool {
        self.state.active_players().len() <= 1
    }

    pub fn is_bot_turn(&self) -> bool {
        let acting = self.acting_player();
        acting != self.human_player
    }

    pub fn acting_player(&self) -> PlayerId {
        // Check pending commander zone choices
        if let Some((pid, _)) = self.state.pending_commander_zone_choices.iter().next() {
            return *pid;
        }
        // Priority holder
        if let Some(pid) = self.state.turn.priority_holder {
            return pid;
        }
        // Default to active player
        self.state.turn.active_player
    }

    pub fn execute_bot_turn(&mut self) -> anyhow::Result<()> {
        // Check for stuck game (too many consecutive passes)
        if self.consecutive_passes >= MAX_CONSECUTIVE_PASSES {
            self.status_message =
                Some("Game stuck — bots passing in a loop. Press 'q' to quit.".into());
            return Ok(());
        }

        let acting = self.acting_player();
        let legal = self.provider.legal_actions(&self.state, acting);

        let cmd = if let Some(bot) = self.bots.get_mut(&acting) {
            if legal.is_empty() {
                Command::PassPriority { player: acting }
            } else {
                bot.choose_action(&self.state, acting, &legal)
            }
        } else {
            Command::PassPriority { player: acting }
        };

        // Track consecutive passes for loop detection
        if matches!(cmd, Command::PassPriority { .. }) {
            self.consecutive_passes += 1;
        } else {
            self.consecutive_passes = 0;
        }

        // Auto-tap mana before casting spells
        if let Command::CastSpell { player, card, .. } = &cmd {
            if let Ok(obj) = self.state.object(*card) {
                if let Some(ref cost) = obj.characteristics.mana_cost {
                    if let Some(tap_cmds) =
                        mtg_simulator::mana_solver::solve_mana_payment(&self.state, *player, cost)
                    {
                        for tap_cmd in tap_cmds {
                            self.execute_command(tap_cmd)?;
                        }
                    }
                }
            }
        }

        self.execute_command(cmd)
    }

    pub fn execute_command(&mut self, cmd: Command) -> anyhow::Result<()> {
        match process_command(self.state.clone(), cmd.clone()) {
            Ok((new_state, events)) => {
                self.log_events(&events);
                self.state = new_state;
                self.status_message = None;

                // Fix stale focused_player if they were eliminated
                let active_players = self.state.active_players();
                if !active_players.contains(&self.focused_player) && !active_players.is_empty() {
                    self.focused_player = self.human_player;
                    // If human is also eliminated, pick the first alive player
                    if !active_players.contains(&self.focused_player) {
                        self.focused_player = active_players[0];
                    }
                }

                Ok(())
            }
            Err(e) => {
                self.status_message = Some(format!("Invalid: {:?}", e));
                // Don't propagate — just show the error
                Ok(())
            }
        }
    }

    pub fn legal_actions(&self) -> Vec<LegalAction> {
        self.provider.legal_actions(&self.state, self.human_player)
    }

    pub fn hand_objects(&self) -> Vec<(ObjectId, String)> {
        let hand = ZoneId::Hand(self.focused_player);
        self.state
            .objects_in_zone(&hand)
            .iter()
            .map(|obj| (obj.id, obj.characteristics.name.clone()))
            .collect()
    }

    pub fn battlefield_objects(&self, player: PlayerId) -> Vec<(ObjectId, String, bool)> {
        self.state
            .objects_in_zone(&ZoneId::Battlefield)
            .iter()
            .filter(|obj| obj.controller == player)
            .map(|obj| (obj.id, obj.characteristics.name.clone(), obj.status.tapped))
            .collect()
    }

    fn log_events(&mut self, events: &[GameEvent]) {
        let turn = self.state.turn.turn_number;
        for event in events {
            let text = format_event(event);
            if !text.is_empty() {
                self.event_log.push(LogEntry { text, turn });
            }
        }
    }
}

/// Format a game event for the log.
fn format_event(event: &GameEvent) -> String {
    match event {
        GameEvent::TurnStarted {
            player,
            turn_number,
        } => {
            format!("Turn {} — P{}'s turn", turn_number, player.0)
        }
        GameEvent::StepChanged { step, phase } => {
            format!("{:?} ({:?})", step, phase)
        }
        GameEvent::PriorityPassed { player } => {
            format!("P{} passes", player.0)
        }
        GameEvent::CardDrawn { player, .. } => {
            format!("P{} draws a card", player.0)
        }
        GameEvent::SpellCast { player, .. } => {
            format!("P{} casts a spell", player.0)
        }
        GameEvent::SpellResolved { player, .. } => {
            format!("P{}'s spell resolves", player.0)
        }
        GameEvent::PermanentEnteredBattlefield { player, .. } => {
            format!("P{}: permanent enters the battlefield", player.0)
        }
        GameEvent::CreatureDied { controller, .. } => {
            format!("P{}'s creature dies", controller.0)
        }
        GameEvent::DamageDealt { amount, .. } => {
            format!("{} damage dealt", amount)
        }
        GameEvent::PlayerLost { player, reason } => {
            format!("P{} loses ({:?})", player.0, reason)
        }
        GameEvent::GameOver { winner } => {
            if let Some(w) = winner {
                format!("Game Over — P{} wins!", w.0)
            } else {
                "Game Over — Draw!".to_string()
            }
        }
        GameEvent::LandPlayed { player, .. } => {
            format!("P{} plays a land", player.0)
        }
        GameEvent::AllPlayersPassed => "All players passed — advancing".to_string(),
        _ => String::new(), // Skip verbose events
    }
}

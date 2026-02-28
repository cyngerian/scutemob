//! App state for the interactive play mode.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, start_game, AttackTarget, CardDefinition,
    CardRegistry, CardType, Command, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec,
    PlayerId, ZoneId,
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
    /// Choose which opponent to attack (shows popup with player list).
    AttackTargetSelection {
        /// Eligible creatures that will attack.
        eligible: Vec<ObjectId>,
        /// Valid targets (opponents).
        targets: Vec<AttackTarget>,
        /// Currently highlighted target index.
        selected: usize,
    },
    AttackerDeclaration,
    BlockerDeclaration,
    CardDetail(ObjectId),
}

/// Which zone has keyboard focus (determines Space key target and visual cue).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusZone {
    Hand,
    Battlefield,
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
    pub focus_zone: FocusZone,
    pub focused_player: PlayerId,
    pub bot_delay_ms: u64,
    pub status_message: Option<String>,
    pub auto_pass: bool,
    pub consecutive_passes: u32,
    pub _player_count: u32,
    pub log_path: PathBuf,
    _registry: Arc<CardRegistry>,
    log_file: BufWriter<File>,
}

/// (ObjectId, name, tapped, power, toughness) — used by battlefield_nonlands.
pub type NonlandEntry = (ObjectId, String, bool, Option<i32>, Option<i32>);

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
        let player_ids: Vec<PlayerId> = (1..=player_count).map(|i| PlayerId(i as u64)).collect();

        for &pid in &player_ids {
            builder = builder.add_player(pid);
        }

        // Build name→def lookup for enriching card specs
        let card_defs: HashMap<String, CardDefinition> =
            cards.iter().map(|c| (c.name.clone(), c.clone())).collect();

        // Give each player a random deck: 7 cards in hand, rest in library
        for &pid in &player_ids {
            if let Some(mut deck) = random_deck(&mut rng, &cards) {
                // Commander in command zone
                if let Some(def) = cards.iter().find(|c| c.card_id == deck.commander) {
                    let spec = ObjectSpec::card(pid, &def.name)
                        .in_zone(ZoneId::Command(pid))
                        .with_card_id(deck.commander.clone());
                    let spec = enrich_spec_from_def(spec, &card_defs);
                    builder = builder.object(spec);
                }

                // Shuffle the deck before splitting hand/library
                deck.main_deck.shuffle(&mut rng);

                // First 7 cards go to hand (opening hand)
                let (hand_cards, library_cards) = if deck.main_deck.len() >= 7 {
                    deck.main_deck.split_at(7)
                } else {
                    (deck.main_deck.as_slice(), &[] as &[_])
                };

                for card_id in hand_cards {
                    if let Some(def) = cards.iter().find(|c| c.card_id == *card_id) {
                        let spec = ObjectSpec::card(pid, &def.name)
                            .in_zone(ZoneId::Hand(pid))
                            .with_card_id(card_id.clone());
                        let spec = enrich_spec_from_def(spec, &card_defs);
                        builder = builder.object(spec);
                    }
                }

                // Remaining cards in library
                for card_id in library_cards {
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

        // Create game log file
        let logs_dir = PathBuf::from("logs");
        fs::create_dir_all(&logs_dir)?;
        let secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let log_path = logs_dir.join(format!("game_{}.log", secs));
        let log_file = BufWriter::new(File::create(&log_path)?);

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
            focus_zone: FocusZone::Hand,
            focused_player: human_player,
            bot_delay_ms: 200,
            status_message: None,
            auto_pass: false,
            consecutive_passes: 0,
            _player_count: player_count,
            log_path,
            _registry: registry,
            log_file,
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
                self.state = new_state;
                self.log_events(&events);
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

    /// Should auto-pass stop and give control back to the human?
    /// Stops at the human's own main phases (where they can play lands/spells).
    pub fn should_stop_auto_pass(&self) -> bool {
        use mtg_engine::Step;
        let is_active = self.state.turn.active_player == self.human_player;
        let is_main = matches!(
            self.state.turn.step,
            Step::PreCombatMain | Step::PostCombatMain
        );
        let stack_empty = self.state.stack_objects.is_empty();
        is_active && is_main && stack_empty
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

    /// Lands on the battlefield for a player — compact display row.
    pub fn battlefield_lands(&self, player: PlayerId) -> Vec<(ObjectId, String, bool)> {
        self.state
            .objects_in_zone(&ZoneId::Battlefield)
            .iter()
            .filter(|obj| {
                obj.controller == player && obj.characteristics.card_types.contains(&CardType::Land)
            })
            .map(|obj| (obj.id, obj.characteristics.name.clone(), obj.status.tapped))
            .collect()
    }

    /// Non-land permanents on the battlefield for a player — vertical list with P/T.
    pub fn battlefield_nonlands(&self, player: PlayerId) -> Vec<NonlandEntry> {
        self.state
            .objects_in_zone(&ZoneId::Battlefield)
            .iter()
            .filter(|obj| {
                obj.controller == player
                    && !obj.characteristics.card_types.contains(&CardType::Land)
            })
            .map(|obj| {
                (
                    obj.id,
                    obj.characteristics.name.clone(),
                    obj.status.tapped,
                    obj.characteristics.power,
                    obj.characteristics.toughness,
                )
            })
            .collect()
    }

    fn log_events(&mut self, events: &[GameEvent]) {
        let turn = self.state.turn.turn_number;
        for event in events {
            let text = format_event(event, &self.state);
            if !text.is_empty() {
                let _ = writeln!(self.log_file, "[T{}] {}", turn, text);
                self.event_log.push(LogEntry { text, turn });
            }
        }
        let _ = self.log_file.flush();
    }

    /// Flush the log file to disk.
    pub fn flush_log(&mut self) {
        let _ = self.log_file.flush();
    }
}

/// Resolve an ObjectId to a card name from the game state, with fallback.
fn resolve_name(state: &GameState, id: ObjectId) -> String {
    state
        .object(id)
        .map(|obj| obj.characteristics.name.clone())
        .unwrap_or_else(|_| "???".to_string())
}

/// Format a game event for the log.
fn format_event(event: &GameEvent, state: &GameState) -> String {
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
        GameEvent::SpellCast {
            player,
            source_object_id,
            ..
        } => {
            let name = resolve_name(state, *source_object_id);
            format!("P{} casts {}", player.0, name)
        }
        GameEvent::SpellResolved {
            player,
            source_object_id,
            ..
        } => {
            let name = resolve_name(state, *source_object_id);
            format!("P{}'s {} resolves", player.0, name)
        }
        GameEvent::PermanentEnteredBattlefield {
            player, object_id, ..
        } => {
            let name = resolve_name(state, *object_id);
            format!("P{}: {} enters the battlefield", player.0, name)
        }
        GameEvent::CreatureDied {
            controller,
            object_id,
            ..
        } => {
            // object_id is the old battlefield ID (retired) — try new_grave_id too
            let name = state
                .object(*object_id)
                .map(|obj| obj.characteristics.name.clone())
                .unwrap_or_else(|_| "a creature".to_string());
            format!("P{}'s {} dies", controller.0, name)
        }
        GameEvent::LandPlayed {
            player,
            new_land_id,
        } => {
            let name = resolve_name(state, *new_land_id);
            format!("P{} plays {}", player.0, name)
        }
        GameEvent::AttackersDeclared {
            attacking_player,
            attackers,
        } => {
            let names: Vec<String> = attackers
                .iter()
                .map(|(id, target)| {
                    let name = resolve_name(state, *id);
                    let tgt = match target {
                        mtg_engine::AttackTarget::Player(pid) => format!("P{}", pid.0),
                        mtg_engine::AttackTarget::Planeswalker(pw) => {
                            resolve_name(state, *pw)
                        }
                    };
                    format!("{} -> {}", name, tgt)
                })
                .collect();
            format!("P{} attacks: {}", attacking_player.0, names.join(", "))
        }
        GameEvent::CombatDamageDealt { assignments } => {
            let parts: Vec<String> = assignments
                .iter()
                .map(|a| {
                    let src = resolve_name(state, a.source);
                    let tgt = match &a.target {
                        mtg_engine::CombatDamageTarget::Player(pid) => format!("P{}", pid.0),
                        mtg_engine::CombatDamageTarget::Creature(cid) => {
                            resolve_name(state, *cid)
                        }
                        mtg_engine::CombatDamageTarget::Planeswalker(pw) => {
                            resolve_name(state, *pw)
                        }
                    };
                    format!("{} deals {} to {}", src, a.amount, tgt)
                })
                .collect();
            format!("Combat damage: {}", parts.join(", "))
        }
        GameEvent::DamageDealt { amount, .. } => {
            format!("{} damage dealt", amount)
        }
        GameEvent::LifeGained { player, amount } => {
            format!("P{} gains {} life", player.0, amount)
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
        GameEvent::AllPlayersPassed => "All players passed — advancing".to_string(),
        GameEvent::PermanentTapped { object_id, .. } => {
            let name = resolve_name(state, *object_id);
            format!("{} tapped", name)
        }
        GameEvent::ManaAdded { player, color, amount } => {
            format!("P{} adds {} {:?} mana", player.0, amount, color)
        }
        _ => String::new(), // Skip verbose events
    }
}

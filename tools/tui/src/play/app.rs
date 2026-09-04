//! App state for the interactive play mode.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use mtg_engine::{
    all_cards, process_command, start_game, AttackTarget, CardRegistry, CardType, Command,
    GameEvent, GameState, ObjectId, PlayerId, ZoneId,
};
// `GameStateBuilder`/`ObjectSpec` are only used by this module's `#[cfg(test)]` fixtures
// (M11-local Session 2 moved the only non-test use — pregame setup — to
// `mtg_simulator::build_initial_state`); importing them unconditionally would be an
// unused-import warning in a non-test build (`[workspace.lints.rust] warnings = "deny"`).
#[cfg(test)]
use mtg_engine::{GameStateBuilder, ObjectSpec};
use mtg_simulator::{
    Bot, BotKind, DeckSource, HeuristicBot, LegalAction, LegalActionProvider, LocalGameConfig,
    LocalGameLimits, RandomBot, StubProvider,
};
use rand::prelude::*;

/// Which public zone to browse in the overlay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrowsableZone {
    Graveyard,
    Exile,
}

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
    CardDetail {
        object_id: ObjectId,
        /// If Some, Esc returns here instead of Normal.
        return_to: Option<Box<InputMode>>,
    },
    /// Scrollable zone browser overlay (graveyard or exile).
    ZoneBrowser {
        zone: BrowsableZone,
        player: PlayerId,
        cards: Vec<(ObjectId, String)>,
        selected: usize,
        scroll_offset: usize,
    },
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
    pub hand_scroll_offset: usize,
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
        // `_registry` is stored for the struct field below only -- nothing else in this
        // impl reads it (see the field's doc history). `setup::build_initial_state`
        // builds its own internal registry to admit decks through `validate_deck`; this
        // one is kept only to preserve the pre-existing field, per the M11-local Session
        // 2 call-site-swap scope (`memory/m11-session-plan.md` §4 Session 2 item 5).
        let registry = CardRegistry::new(all_cards());
        let human_player = PlayerId(1);
        let mut os_rng = StdRng::from_os_rng();

        let bot_kind = if bot_type == "heuristic" {
            BotKind::Heuristic
        } else {
            BotKind::Random
        };
        let cfg = LocalGameConfig {
            player_count,
            human_seats: [human_player].into_iter().collect(),
            bot_kind,
            // The TUI still seeds from the OS at the entry point -- each launch gets a
            // genuinely random game, exactly as before Session 2.
            seed: os_rng.random(),
            // A seeded *recipe*, which is correct here only because the TUI never
            // mulligans. `setup::redeal` rebuilds from a perturbed seed, and with a
            // recipe that re-rolls every seat's decklist and commander — the G2
            // defect `scutemob-187` fixed in the play server (CR 103.5: a mulligan
            // permutes a fixed multiset). If this surface ever grows a mulligan,
            // follow `play-server`'s `session::new_game`: build once, then hold
            // `DeckSource::Fixed(setup::dealt_decks(&state, &cfg))`.
            decks: DeckSource::RandomPerSeat,
            // `PlayApp` never runs a `LocalGame` (it drives its own loop via
            // `execute_bot_turn`/`execute_command`), so these limits are inert here --
            // `LocalGameConfig` just always carries them.
            limits: LocalGameLimits {
                max_turns: u32::MAX,
                max_commands: u32::MAX,
                max_consecutive_passes: MAX_CONSECUTIVE_PASSES,
                record_journal: false,
            },
        };
        // CR 103.5/903.6 pregame setup, and Architecture Invariant 9 deck admission
        // (`validate_deck`) for free -- unlike the setup logic this replaces, a seat
        // whose `random_deck` could find no legendary commander is now a hard
        // `SetupError` surfaced through `anyhow`, not a silently-skipped seat with no
        // deck (a broken game masquerading as a graceful degradation).
        let (state, _names) = mtg_simulator::build_initial_state(&cfg)?;

        // Create bots for non-human players
        let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
        for i in 2..=player_count {
            let pid = PlayerId(i as u64);
            let seed = os_rng.random();
            let name = format!("Bot-{}", i);
            // Constructed from `cfg.bot_kind`, not from the `bot_type` string a second
            // time: two independent readings of the same input are two things that can
            // drift. `bot_type` is parsed exactly once, above.
            let bot: Box<dyn Bot> = match cfg.bot_kind {
                BotKind::Heuristic => Box::new(HeuristicBot::new(seed, name)),
                BotKind::Random => Box::new(RandomBot::new(seed, name)),
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
            hand_scroll_offset: 0,
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
        // PB-DP7 / DP-3 (CR 514.1): an outstanding cleanup discard MUST be
        // resolved first -- the engine's admission gate rejects
        // ReturnCommanderToCommandZone (and everything else except
        // DiscardToHandSize/Concede) while it is blocking, so offering the
        // commander-zone choice first would produce a rejected command.
        // `execute_bot_turn` / `execute_command` handle that rejected-command
        // case gracefully (the error is swallowed into `status_message`, not
        // a panic), but note that is NOT the same as avoiding a spin: this
        // ordering only prevents `execute_bot_turn` from repeatedly offering
        // the WRONG action while blocked. It does nothing for the human seat's
        // auto-pass loop in `play/mod.rs`, which never calls `acting_player`
        // at all and drove its own livelock -- see the second fix cycle's
        // Issue 2 fix on `should_stop_auto_pass` below, which is the actual
        // guard against that spin.
        // Fix-cycle Finding 4 (MEDIUM): read the liveness-filtered predicate,
        // not the raw `pending_cleanup_discard()` field -- a dead active
        // player's stale entry must not pin `acting_player` on a player who
        // can never answer it.
        if let Some(decision) = self.state.blocking_decision() {
            return decision.player();
        }
        // Check pending commander zone choices
        if let Some((pid, _)) = self.state.pending_commander_zone_choices().iter().next() {
            return *pid;
        }
        // Priority holder
        if let Some(pid) = self.state.turn().priority_holder {
            return pid;
        }
        // Default to active player
        self.state.turn().active_player
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
        if let Command::CastSpell(cast) = &cmd {
            if let Ok(obj) = self.state.object(cast.card) {
                if let Some(ref cost) = obj.characteristics.mana_cost {
                    if let Some(tap_cmds) = mtg_simulator::mana_solver::solve_mana_payment(
                        &self.state,
                        cast.player,
                        cost,
                    ) {
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
    ///
    /// Fix-cycle Issue 2 (closing /review, MEDIUM): also stops whenever a
    /// `BlockingDecision` is outstanding (PB-DP7 / DP-3 and onward --
    /// `rules::engine::BlockingDecision`). Without this, a human seat blocked
    /// on (e.g.) a cleanup discard never reaches a main phase with an empty
    /// stack, so `is_main` stays permanently `false` and the auto-pass loop in
    /// `play/mod.rs` livelocks: it keeps issuing `PassPriority`, the engine
    /// rejects every one of them with `BlockedByPendingDecision`, and the
    /// human can never reach the `d` key (which answers the block) without
    /// first manually toggling auto-pass off with `z`. A blocking decision
    /// has no priority window at all (CR 514.3 for the cleanup-discard case),
    /// so stopping here is strictly more conservative than requiring a main
    /// phase, not a narrower carve-out of it.
    pub fn should_stop_auto_pass(&self) -> bool {
        use mtg_engine::Step;
        if self.state.blocking_decision().is_some() {
            return true;
        }
        let is_active = self.state.turn().active_player == self.human_player;
        let is_main = matches!(
            self.state.turn().step,
            Step::PreCombatMain | Step::PostCombatMain
        );
        let stack_empty = self.state.stack_objects().is_empty();
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

    pub fn hand_count(&self, player: PlayerId) -> usize {
        self.state.objects_in_zone(&ZoneId::Hand(player)).len()
    }

    pub fn library_count(&self, player: PlayerId) -> usize {
        self.state.objects_in_zone(&ZoneId::Library(player)).len()
    }

    pub fn graveyard_count(&self, player: PlayerId) -> usize {
        self.state.objects_in_zone(&ZoneId::Graveyard(player)).len()
    }

    pub fn exile_count(&self, player: PlayerId) -> usize {
        self.state
            .objects_in_zone(&ZoneId::Exile)
            .iter()
            .filter(|obj| obj.owner == player)
            .count()
    }

    pub fn graveyard_objects(&self, player: PlayerId) -> Vec<(ObjectId, String)> {
        self.state
            .objects_in_zone(&ZoneId::Graveyard(player))
            .iter()
            .map(|obj| (obj.id, obj.characteristics.name.clone()))
            .collect()
    }

    pub fn exile_objects(&self, player: PlayerId) -> Vec<(ObjectId, String)> {
        self.state
            .objects_in_zone(&ZoneId::Exile)
            .iter()
            .filter(|obj| obj.owner == player)
            .map(|obj| (obj.id, obj.characteristics.name.clone()))
            .collect()
    }

    fn log_events(&mut self, events: &[GameEvent]) {
        let turn = self.state.turn().turn_number;
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
                        mtg_engine::AttackTarget::Planeswalker(pw) => resolve_name(state, *pw),
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
                        mtg_engine::CombatDamageTarget::Creature(cid) => resolve_name(state, *cid),
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
        GameEvent::ManaAdded {
            player,
            color,
            amount,
            ..
        } => {
            format!("P{} adds {} {:?} mana", player.0, amount, color)
        }
        GameEvent::DiscardedToHandSize {
            player, object_id, ..
        } => {
            let name = resolve_name(state, *object_id);
            format!("P{} discards {} (cleanup)", player.0, name)
        }
        GameEvent::CardDiscarded {
            player,
            new_id,
            object_id,
        } => {
            // Try new_id first (graveyard copy), fallback to old object_id
            let name = state
                .object(*new_id)
                .or_else(|_| state.object(*object_id))
                .map(|obj| obj.characteristics.name.clone())
                .unwrap_or_else(|_| "???".to_string());
            format!("P{} discards {}", player.0, name)
        }
        GameEvent::CleanupDiscardChoiceRequired { player, count, .. } => {
            format!(
                "P{} must discard {} card(s) to hand size (CR 514.1) — press 'd'",
                player.0, count
            )
        }
        GameEvent::TriggerTargetChoiceRequired { player, slots, .. } => {
            format!(
                "P{} must announce {} trigger target slot(s) (CR 603.3d) — press 'n'",
                player.0,
                slots.len()
            )
        }
        // CR 608.2d (PB-DP9 / DP-7/8/9). Deliberately does NOT print the ids or
        // the count: every one of them names a card in a HIDDEN zone, and this
        // formatter feeds a log panel that a spectator build could show to
        // another seat. `GameEvent::private_to()` returns `Some(player)` for
        // this event for the same reason.
        GameEvent::EffectChoiceRequired {
            player, question, ..
        } => {
            let kind = match question {
                mtg_engine::EffectChoiceQuestion::SearchLibrary { .. } => "library search",
                mtg_engine::EffectChoiceQuestion::Scry { .. } => "scry",
                mtg_engine::EffectChoiceQuestion::Surveil { .. } => "surveil",
                mtg_engine::EffectChoiceQuestion::Discard { .. } => "discard",
                // PB-DX28 (CR 115.10): through PB-DX35, this one named only
                // PUBLIC objects (battlefield permanents / graveyard cards).
                // `Effect::LookAtTopThenPlace`'s `optional` placement
                // (`OOS-DX4-5`) now asks the SAME variant with LIBRARY ids too,
                // so that is no longer true in general -- but the formatter
                // stays uniform and prints the class label only regardless, the
                // same reason `PayOptionalCost` below never printed one.
                mtg_engine::EffectChoiceQuestion::ChooseObject { .. } => "choose object",
                // PB-DX45 (CR 118.12): the optional-cost offer. Like ChooseObject
                // it names no hidden information -- it names no ids at all -- and
                // the formatter stays uniform, printing the class label only.
                mtg_engine::EffectChoiceQuestion::PayOptionalCost { .. } => "optional cost",
                // PB-DX50 (CR 702.140c): the mutate over/under offer. It names
                // exactly one id (`host`), and that id is a BATTLEFIELD permanent
                // (CR 400.1, public) which is already this spell's announced
                // target -- but the formatter stays uniform and prints the class
                // label only, for the same reason the two arms above do.
                mtg_engine::EffectChoiceQuestion::MutateOnTop { .. } => "mutate over/under",
            };
            format!("P{} must answer a {kind} (CR 608.2d) — press 'r'", player.0)
        }
        // ENG-2 (§6.2, CR 601.2c / 602.2b / 603.3d): the TUI is omniscient by
        // design and has no redaction layer, so every target names its object
        // directly through `resolve_name` rather than going through a `card_or`
        // entitlement gate.
        GameEvent::TargetsAnnounced {
            controller,
            source_object_id,
            targets,
            ..
        } => {
            let subject = resolve_name(state, *source_object_id);
            let rendered_targets: Vec<String> = targets
                .iter()
                .map(|t| match t.target {
                    mtg_engine::Target::Player(pid) => format!("P{}", pid.0),
                    mtg_engine::Target::Object(id) => resolve_name(state, id),
                })
                .collect();
            format!(
                "P{}'s {} targets {}",
                controller.0,
                subject,
                rendered_targets.join(", ")
            )
        }
        _ => String::new(), // Skip verbose events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtg_engine::Step;

    /// Build a minimal 2-player state where P1 (the "human" seat in these
    /// tests) reaches the blocked Cleanup pause (PB-DP7 / DP-3, CR 514.1) with
    /// an oversized hand, using only the public engine API `tools/tui`
    /// already depends on -- mirrors
    /// `crates/engine/tests/primitives/pb_dp7_cleanup_discard.rs`'s
    /// `build_oversized_hand`/`advance_to_cleanup_block` fixture, minus the
    /// 4-player/Madness machinery this test doesn't need.
    fn blocked_cleanup_state_for_p1() -> GameState {
        let mut builder = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .active_player(PlayerId(1))
            .at_step(Step::End);
        for i in 0..9u32 {
            builder = builder.object(
                ObjectSpec::card(PlayerId(1), &format!("Filler {i}"))
                    .in_zone(ZoneId::Hand(PlayerId(1)))
                    .with_types(vec![CardType::Instant]),
            );
        }
        let state = builder.build().expect("oversized-hand state should build");
        let (state, _events) = process_command(
            state,
            Command::PassPriority {
                player: PlayerId(1),
            },
        )
        .expect("P1's pass out of End should succeed");
        let (state, _events) = process_command(
            state,
            Command::PassPriority {
                player: PlayerId(2),
            },
        )
        .expect("P2's pass out of End should succeed");
        assert!(
            state.blocking_decision().is_some(),
            "fixture must actually reach the blocked cleanup pause -- if this \
             fails, the fixture itself is broken, not the code under test"
        );
        state
    }

    /// A minimal `PlayApp` wrapping a given `state`, for exercising
    /// state-inspecting methods (`should_stop_auto_pass`, `acting_player`)
    /// without the full `PlayApp::new()` random-deck machinery. Constructed
    /// via a struct literal (this `mod tests` is a descendant of the module
    /// that defines `PlayApp`, so its private fields -- `_registry`/
    /// `log_file` -- are visible here). `log_file` still needs a real,
    /// openable `File` (it is a `BufWriter<File>`, not an `impl Write`), so
    /// this opens one discardable temp file per call rather than one per
    /// real game.
    fn minimal_app(state: GameState, human_player: PlayerId) -> PlayApp {
        let cards = all_cards();
        let registry = CardRegistry::new(cards);
        let log_path = std::env::temp_dir().join(format!(
            "mtg-tui-test-{}-{:?}.log",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let log_file = BufWriter::new(File::create(&log_path).expect("temp log file"));
        PlayApp {
            state,
            human_player,
            provider: StubProvider,
            bots: HashMap::new(),
            should_quit: false,
            mode: InputMode::Normal,
            event_log: Vec::new(),
            log_scroll: 0,
            selected_hand_idx: 0,
            hand_scroll_offset: 0,
            selected_bf_idx: 0,
            focus_zone: FocusZone::Hand,
            focused_player: human_player,
            bot_delay_ms: 0,
            status_message: None,
            auto_pass: true,
            consecutive_passes: 0,
            _player_count: 2,
            log_path,
            _registry: registry,
            log_file,
        }
    }

    /// Issue 2 (closing /review, MEDIUM): demonstrate, then close, the
    /// TUI auto-pass livelock on a cleanup-discard block.
    ///
    /// `tools/tui/src/play/mod.rs`'s auto-pass loop calls
    /// `should_stop_auto_pass()` and, while it returns `false`, issues
    /// `PassPriority` for the human seat every ~50ms forever. Pre-fix,
    /// `should_stop_auto_pass()` only checked `is_active && is_main &&
    /// stack_empty` -- during `Step::Cleanup` `is_main` is always `false`, so
    /// a human blocked on a cleanup discard could never stop auto-pass, and
    /// the only two keys the auto-pass poll handles (`q`/`Ctrl-C`/`z`) do not
    /// include `d` (the key that actually answers the block) -- a real
    /// livelock, not merely a wasted `PassPriority`.
    ///
    /// OBSERVED (pre-fix, this exact assertion run against the code before
    /// this fix cycle's edit to `should_stop_auto_pass`): the assertion
    /// FAILED --
    /// `assertion failed: app.should_stop_auto_pass()` -- confirming
    /// `should_stop_auto_pass()` returned `false` for a `PlayApp` whose state
    /// is blocked on a cleanup discard for the human seat, i.e. the auto-pass
    /// loop would spin `PassPriority` (rejected every time with
    /// `BlockedByPendingDecision`, swallowed into `status_message` by
    /// `execute_command`) with no way for the human to reach the `d` key
    /// without first manually toggling auto-pass off with `z`.
    #[test]
    fn test_dp7_should_stop_auto_pass_true_while_blocked() {
        let state = blocked_cleanup_state_for_p1();
        let app = minimal_app(state, PlayerId(1));

        assert!(
            app.should_stop_auto_pass(),
            "should_stop_auto_pass() must return true while the human seat is \
             blocked on a pending decision (PB-DP7 / DP-3), or the auto-pass \
             loop in play/mod.rs livelocks issuing a PassPriority the engine \
             will always reject"
        );
    }

    /// The pre-PB-DP7 behaviour this method exists for is unaffected: outside
    /// a blocking decision, auto-pass still stops only at the human's own
    /// main phase with an empty stack, exactly as before.
    #[test]
    fn test_dp7_should_stop_auto_pass_unaffected_when_not_blocked() {
        let state = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .active_player(PlayerId(1))
            .at_step(Step::PreCombatMain)
            .build()
            .expect("simple state should build");
        assert!(state.blocking_decision().is_none());
        let app = minimal_app(state, PlayerId(1));
        assert!(
            app.should_stop_auto_pass(),
            "must still stop at the human's own main phase with an empty stack"
        );

        let state_untap = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .active_player(PlayerId(1))
            .at_step(Step::Untap)
            .build()
            .expect("simple state should build");
        let app_untap = minimal_app(state_untap, PlayerId(1));
        assert!(
            !app_untap.should_stop_auto_pass(),
            "must NOT stop outside a main phase when nothing is blocking"
        );
    }
}

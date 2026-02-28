//! Keyboard input handling for play mode.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mtg_engine::Command;
use mtg_simulator::LegalAction;

use super::app::{InputMode, PlayApp};

pub fn handle_key(app: &mut PlayApp, key: KeyEvent) -> anyhow::Result<()> {
    // Global keys
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return Ok(());
        }
        _ => {}
    }

    match &app.mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::CardDetail(_) => handle_card_detail_mode(app, key),
        InputMode::AttackerDeclaration => handle_attacker_mode(app, key),
        InputMode::BlockerDeclaration => handle_blocker_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut PlayApp, key: KeyEvent) -> anyhow::Result<()> {
    let legal = app.legal_actions();

    match key.code {
        // Pass priority
        KeyCode::Char('p') => {
            let cmd = Command::PassPriority {
                player: app.human_player,
            };
            app.execute_command(cmd)?;
        }

        // Play selected land — only if legal
        KeyCode::Char('l') => {
            let hand = app.hand_objects();
            if let Some((obj_id, name)) = hand.get(app.selected_hand_idx) {
                let is_legal = legal
                    .iter()
                    .any(|a| matches!(a, LegalAction::PlayLand { card } if *card == *obj_id));
                if is_legal {
                    let cmd = Command::PlayLand {
                        player: app.human_player,
                        card: *obj_id,
                    };
                    app.execute_command(cmd)?;
                } else {
                    let has_any_land = legal
                        .iter()
                        .any(|a| matches!(a, LegalAction::PlayLand { .. }));
                    if has_any_land {
                        app.status_message =
                            Some(format!("'{}' is not a land — select a land first", name));
                    } else {
                        app.status_message = Some(
                            "Can't play lands now (need Main phase, your turn, stack empty)"
                                .into(),
                        );
                    }
                }
            }
        }

        // Cast selected spell — only if legal, auto-tap mana
        KeyCode::Char('c') => {
            let hand = app.hand_objects();
            if let Some((obj_id, name)) = hand.get(app.selected_hand_idx) {
                let is_legal = legal.iter().any(
                    |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == *obj_id),
                );
                if is_legal {
                    // Auto-tap mana before casting
                    if let Ok(obj) = app.state.object(*obj_id) {
                        if let Some(ref cost) = obj.characteristics.mana_cost {
                            if let Some(tap_cmds) =
                                mtg_simulator::mana_solver::solve_mana_payment(
                                    &app.state,
                                    app.human_player,
                                    cost,
                                )
                            {
                                for tap_cmd in tap_cmds {
                                    app.execute_command(tap_cmd)?;
                                }
                            }
                        }
                    }

                    let cmd = Command::CastSpell {
                        player: app.human_player,
                        card: *obj_id,
                        targets: Vec::new(),
                        convoke_creatures: Vec::new(),
                        improvise_artifacts: Vec::new(),
                        delve_cards: Vec::new(),
                        kicker_times: 0,
                        cast_with_evoke: false,
                        cast_with_bestow: false,
                        cast_with_miracle: false,
                        cast_with_escape: false,
                        escape_exile_cards: Vec::new(),
                        cast_with_foretell: false,
                        cast_with_buyback: false,
                    };
                    app.execute_command(cmd)?;
                } else {
                    let has_any_cast = legal
                        .iter()
                        .any(|a| matches!(a, LegalAction::CastSpell { .. }));
                    if !has_any_cast {
                        app.status_message = Some("No spells you can cast right now".into());
                    } else {
                        app.status_message = Some(format!(
                            "Can't cast '{}' — not enough mana or wrong timing",
                            name
                        ));
                    }
                }
            }
        }

        // Tap for mana
        KeyCode::Char('t') => {
            let bf = app.battlefield_objects(app.human_player);
            if let Some((obj_id, name, _)) = bf.get(app.selected_bf_idx) {
                let tap_action = legal.iter().find(
                    |a| matches!(a, LegalAction::TapForMana { source, .. } if *source == *obj_id),
                );
                if let Some(LegalAction::TapForMana { ability_index, .. }) = tap_action {
                    let cmd = Command::TapForMana {
                        player: app.human_player,
                        source: *obj_id,
                        ability_index: *ability_index,
                    };
                    app.execute_command(cmd)?;
                } else {
                    app.status_message = Some(format!(
                        "'{}' can't tap for mana (tapped or no mana ability)",
                        name
                    ));
                }
            }
        }

        // Attack mode — only if attacks are available
        KeyCode::Char('a') => {
            let has_attack = legal
                .iter()
                .any(|a| matches!(a, LegalAction::DeclareAttackers { .. }));
            if has_attack {
                app.mode = InputMode::AttackerDeclaration;
                app.status_message =
                    Some("Declare attackers: [Enter] attack with all, [Esc] cancel".into());
            } else {
                app.status_message = Some(
                    "Can't attack now (need Declare Attackers step, your turn)".into(),
                );
            }
        }

        // Activate ability
        KeyCode::Char('e') => {
            let bf = app.battlefield_objects(app.human_player);
            if let Some((obj_id, name, _)) = bf.get(app.selected_bf_idx) {
                let ability_action = legal.iter().find(
                    |a| matches!(a, LegalAction::ActivateAbility { source, .. } if *source == *obj_id),
                );
                if let Some(LegalAction::ActivateAbility { ability_index, .. }) = ability_action {
                    let cmd = Command::ActivateAbility {
                        player: app.human_player,
                        source: *obj_id,
                        ability_index: *ability_index,
                        targets: Vec::new(),
                    };
                    app.execute_command(cmd)?;
                } else {
                    app.status_message =
                        Some(format!("'{}' has no activatable ability right now", name));
                }
            }
        }

        // Navigate hand
        KeyCode::Left => {
            if app.selected_hand_idx > 0 {
                app.selected_hand_idx -= 1;
            }
        }
        KeyCode::Right => {
            let hand_size = app.hand_objects().len();
            if hand_size > 0 && app.selected_hand_idx < hand_size - 1 {
                app.selected_hand_idx += 1;
            }
        }

        // Navigate battlefield
        KeyCode::Up => {
            if app.selected_bf_idx > 0 {
                app.selected_bf_idx -= 1;
            }
        }
        KeyCode::Down => {
            let bf_size = app.battlefield_objects(app.human_player).len();
            if bf_size > 0 && app.selected_bf_idx < bf_size - 1 {
                app.selected_bf_idx += 1;
            }
        }

        // Tab to cycle focused player
        KeyCode::Tab => {
            let players = app.state.active_players();
            if let Some(pos) = players.iter().position(|p| *p == app.focused_player) {
                let next = (pos + 1) % players.len();
                app.focused_player = players[next];
            }
        }
        KeyCode::BackTab => {
            let players = app.state.active_players();
            if let Some(pos) = players.iter().position(|p| *p == app.focused_player) {
                let next = if pos == 0 { players.len() - 1 } else { pos - 1 };
                app.focused_player = players[next];
            }
        }

        // Card detail popup
        KeyCode::Char(' ') => {
            let hand = app.hand_objects();
            if let Some((obj_id, _)) = hand.get(app.selected_hand_idx) {
                app.mode = InputMode::CardDetail(*obj_id);
            }
        }

        // Scroll event log
        KeyCode::Char('j') => {
            let max = app.event_log.len().saturating_sub(1);
            if app.log_scroll < max {
                app.log_scroll += 1;
            }
        }
        KeyCode::Char('k') => {
            if app.log_scroll > 0 {
                app.log_scroll -= 1;
            }
        }

        // Number keys: select hand card by position
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let idx = (c as usize) - ('1' as usize);
            let hand_size = app.hand_objects().len();
            if idx < hand_size {
                app.selected_hand_idx = idx;
            }
        }

        _ => {}
    }
    Ok(())
}

fn handle_card_detail_mode(app: &mut PlayApp, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Char(' ') => {
            app.mode = InputMode::Normal;
        }
        _ => {}
    }
    Ok(())
}

fn handle_attacker_mode(app: &mut PlayApp, key: KeyEvent) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Normal;
            app.status_message = None;
        }
        KeyCode::Enter => {
            // Declare all eligible creatures as attackers against a random opponent
            let legal = app.legal_actions();
            if let Some(LegalAction::DeclareAttackers { eligible, targets }) = legal
                .iter()
                .find(|a| matches!(a, LegalAction::DeclareAttackers { .. }))
            {
                if let Some(target) = targets.first() {
                    let attackers: Vec<_> =
                        eligible.iter().map(|&id| (id, target.clone())).collect();
                    let cmd = Command::DeclareAttackers {
                        player: app.human_player,
                        attackers,
                    };
                    app.execute_command(cmd)?;
                }
            }
            app.mode = InputMode::Normal;
            app.status_message = None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_blocker_mode(app: &mut PlayApp, key: KeyEvent) -> anyhow::Result<()> {
    if key.code == KeyCode::Esc {
        app.mode = InputMode::Normal;
        app.status_message = None;
    }
    Ok(())
}

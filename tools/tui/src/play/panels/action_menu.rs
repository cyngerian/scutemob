//! Action menu — shows available actions based on current game state.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::{InputMode, PlayApp};
use mtg_simulator::LegalAction;

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    let content = if let Some(ref msg) = app.status_message {
        Line::from(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(Color::Yellow),
        ))
    } else if app.game_over() {
        let winner = app.state.active_players();
        let text = if winner.len() == 1 && winner[0] == app.human_player {
            "You win! Press [q] to quit."
        } else if winner.len() == 1 {
            "You lost. Press [q] to quit."
        } else {
            "Game over. Press [q] to quit."
        };
        Line::from(Span::styled(
            format!(" {} ", text),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
    } else if app.is_bot_turn() {
        Line::from(Span::styled(
            " Waiting for bots... [q] quit",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        match app.mode {
            InputMode::Normal => build_normal_actions(app),
            InputMode::AttackerDeclaration => Line::from(Span::styled(
                " Declare attackers: [Enter] confirm all, [Esc] cancel",
                Style::default().fg(Color::Yellow),
            )),
            InputMode::BlockerDeclaration => Line::from(Span::styled(
                " Declare blockers: [Enter] confirm, [Esc] cancel",
                Style::default().fg(Color::Yellow),
            )),
            InputMode::CardDetail(_) => Line::from(Span::styled(
                " [Esc] or [Space] to close",
                Style::default().fg(Color::Yellow),
            )),
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

/// Build action hints based on what's actually legal right now.
fn build_normal_actions(app: &PlayApp) -> Line<'static> {
    let legal = app.legal_actions();

    let has_land = legal.iter().any(|a| matches!(a, LegalAction::PlayLand { .. }));
    let has_cast = legal.iter().any(|a| matches!(a, LegalAction::CastSpell { .. }));
    let has_tap = legal.iter().any(|a| matches!(a, LegalAction::TapForMana { .. }));
    let has_attack = legal
        .iter()
        .any(|a| matches!(a, LegalAction::DeclareAttackers { .. }));
    let has_ability = legal
        .iter()
        .any(|a| matches!(a, LegalAction::ActivateAbility { .. }));

    let mut spans: Vec<Span<'static>> = vec![Span::raw(" ")];

    // Always show pass and quit
    spans.push(Span::styled("[p]", Style::default().fg(Color::Cyan)));
    spans.push(Span::raw("ass "));

    if has_land {
        spans.push(Span::styled("[l]", Style::default().fg(Color::Green)));
        spans.push(Span::raw("and "));
    }

    if has_cast {
        spans.push(Span::styled("[c]", Style::default().fg(Color::Green)));
        spans.push(Span::raw("ast "));
    }

    if has_tap {
        spans.push(Span::styled("[t]", Style::default().fg(Color::Cyan)));
        spans.push(Span::raw("ap "));
    }

    if has_attack {
        spans.push(Span::styled("[a]", Style::default().fg(Color::Red)));
        spans.push(Span::raw("ttack "));
    }

    if has_ability {
        spans.push(Span::styled("[e]", Style::default().fg(Color::Cyan)));
        spans.push(Span::raw("ffect "));
    }

    spans.push(Span::styled("[Space]", Style::default().fg(Color::DarkGray)));
    spans.push(Span::raw("detail "));
    spans.push(Span::styled("[Tab]", Style::default().fg(Color::DarkGray)));
    spans.push(Span::raw("view "));
    spans.push(Span::styled("[q]", Style::default().fg(Color::DarkGray)));
    spans.push(Span::raw("uit"));

    Line::from(spans)
}

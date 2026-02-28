//! Action menu — shows available actions and status messages.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::{InputMode, PlayApp};

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
    } else {
        match app.mode {
            InputMode::Normal => Line::from(vec![
                Span::styled(" [p]", Style::default().fg(Color::Cyan)),
                Span::raw("ass "),
                Span::styled("[c]", Style::default().fg(Color::Cyan)),
                Span::raw("ast "),
                Span::styled("[l]", Style::default().fg(Color::Cyan)),
                Span::raw("and "),
                Span::styled("[t]", Style::default().fg(Color::Cyan)),
                Span::raw("ap "),
                Span::styled("[a]", Style::default().fg(Color::Cyan)),
                Span::raw("ttack "),
                Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
                Span::raw("view "),
                Span::styled("[Space]", Style::default().fg(Color::Cyan)),
                Span::raw("detail "),
                Span::styled("[q]", Style::default().fg(Color::Cyan)),
                Span::raw("uit"),
            ]),
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

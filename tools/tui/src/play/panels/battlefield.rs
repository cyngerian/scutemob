//! Battlefield panel — shows permanents for the focused player.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    let objects = app.battlefield_objects(app.focused_player);

    let player_label = if app.focused_player == app.human_player {
        "Your Battlefield".to_string()
    } else {
        format!("P{}'s Battlefield", app.focused_player.0)
    };

    let items: Vec<Line> = if objects.is_empty() {
        vec![Line::from(Span::styled(
            " (empty)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        objects
            .iter()
            .enumerate()
            .map(|(i, (_id, name, tapped))| {
                let selected = i == app.selected_bf_idx && app.focused_player == app.human_player;
                let tap_indicator = if *tapped { " (T)" } else { "" };
                let prefix = if selected { ">" } else { " " };
                let style = if selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if *tapped {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(
                    format!("{} [{}]{}", prefix, name, tap_indicator),
                    style,
                ))
            })
            .collect()
    };

    let block = Block::default()
        .title(format!(" {} ", player_label))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
}

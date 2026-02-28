//! Zone browser overlay — scrollable list of graveyard or exile contents.

use ratatui::prelude::*;
use ratatui::widgets::*;

use mtg_engine::{ObjectId, PlayerId};

use crate::play::app::{BrowsableZone, PlayApp};
use crate::play::panels::card_detail::{card_color, centered_rect};

pub fn render(
    f: &mut Frame,
    app: &PlayApp,
    zone: &BrowsableZone,
    player: PlayerId,
    cards: &[(ObjectId, String)],
    selected: usize,
    scroll_offset: usize,
) {
    let area = centered_rect(50, 60, f.area());
    f.render_widget(Clear, area);

    let visible_height = area.height.saturating_sub(4) as usize; // borders + footer

    let zone_name = match zone {
        BrowsableZone::Graveyard => "Graveyard",
        BrowsableZone::Exile => "Exile",
    };

    let player_label = if player == app.human_player {
        "Your".to_string()
    } else {
        format!("P{}'s", player.0)
    };

    let title = format!(
        " {} {} ({} cards) [Esc] close ",
        player_label,
        zone_name,
        cards.len()
    );

    let items: Vec<Line> = if cards.is_empty() {
        vec![Line::from(Span::styled(
            " (empty)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        cards
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
            .map(|(i, (obj_id, name))| {
                let is_selected = i == selected;
                let color = app
                    .state
                    .object(*obj_id)
                    .map(|obj| card_color(&obj.characteristics))
                    .unwrap_or(Color::Gray);

                let prefix = if is_selected { "> " } else { "  " };

                if is_selected {
                    Line::from(Span::styled(
                        format!("{}{}", prefix, name),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(Span::styled(
                        format!("{}{}", prefix, name),
                        Style::default().fg(color),
                    ))
                }
            })
            .collect()
    };

    // Footer line
    let footer = Line::from(vec![
        Span::styled(" [Space]", Style::default().fg(Color::Cyan)),
        Span::raw("inspect "),
        Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
        Span::raw("close "),
        Span::styled("[↑↓]", Style::default().fg(Color::DarkGray)),
        Span::raw("navigate"),
    ]);

    let mut all_lines = items;
    // Pad to fill visible area so footer stays at bottom
    while all_lines.len() < visible_height {
        all_lines.push(Line::from(""));
    }
    all_lines.push(footer);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(all_lines).block(block);
    f.render_widget(paragraph, area);
}

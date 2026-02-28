//! Status bar — single line showing life, mana pool, and land count.

use ratatui::prelude::*;
use ratatui::widgets::*;

use mtg_engine::{CardType, ZoneId};

use crate::play::app::PlayApp;

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    let player = app.human_player;
    let Ok(p) = app.state.player(player) else {
        return;
    };

    let mut spans = Vec::new();

    // Life total
    spans.push(Span::styled(
        format!("Life: {}", p.life_total),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::raw(" | Mana: "));

    // Colored mana — only non-zero values
    let pool = &p.mana_pool;
    let mana_parts: Vec<(char, u32, Color)> = vec![
        ('W', pool.white, Color::White),
        ('U', pool.blue, Color::Blue),
        ('B', pool.black, Color::Rgb(100, 100, 100)),
        ('R', pool.red, Color::Red),
        ('G', pool.green, Color::Green),
        ('C', pool.colorless, Color::Gray),
    ];

    let mut any_mana = false;
    for (letter, amount, color) in &mana_parts {
        if *amount > 0 {
            if any_mana {
                spans.push(Span::raw(" "));
            }
            spans.push(Span::styled(
                format!("{}:{}", letter, amount),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ));
            any_mana = true;
        }
    }
    if !any_mana {
        spans.push(Span::styled(
            "(empty)",
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Land count from battlefield
    let land_count = app
        .state
        .objects_in_zone(&ZoneId::Battlefield)
        .iter()
        .filter(|obj| {
            obj.controller == player && obj.characteristics.card_types.contains(&CardType::Land)
        })
        .count();

    spans.push(Span::raw(format!(
        " | Lands: {} ({} play left)",
        land_count, p.land_plays_remaining
    )));

    let line = Line::from(spans);
    let bar = Paragraph::new(line).style(Style::default().fg(Color::Yellow));
    f.render_widget(bar, area);
}

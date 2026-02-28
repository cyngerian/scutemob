//! Phase bar — always-visible top bar showing turn, phase, and priority info.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    let turn = app.state.turn.turn_number;
    let active = app.state.turn.active_player;
    let step = &app.state.turn.step;
    let priority = app.state.turn.priority_holder;

    let priority_text = if let Some(p) = priority {
        if p == app.human_player {
            "You".to_string()
        } else {
            format!("P{}", p.0)
        }
    } else {
        "—".to_string()
    };

    let active_text = if active == app.human_player {
        "Your Turn".to_string()
    } else {
        format!("P{}'s Turn", active.0)
    };

    let text = format!(
        " Turn {} | {} | {:?} | Priority: {} ",
        turn, active_text, step, priority_text
    );

    let bar = Paragraph::new(text).style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(bar, area);
}

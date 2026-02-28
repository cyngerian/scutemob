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

    let step_name = step_display_name(step);

    let text = format!(
        " Turn {} | {} | {} | Priority: {} ",
        turn, active_text, step_name, priority_text
    );

    let bar = Paragraph::new(text).style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(bar, area);
}

fn step_display_name(step: &mtg_engine::Step) -> &'static str {
    use mtg_engine::Step;
    match step {
        Step::Untap => "Untap",
        Step::Upkeep => "Upkeep",
        Step::Draw => "Draw",
        Step::PreCombatMain => "Main 1",
        Step::BeginningOfCombat => "Begin Combat",
        Step::DeclareAttackers => "Declare Attackers",
        Step::DeclareBlockers => "Declare Blockers",
        Step::FirstStrikeDamage => "First Strike Damage",
        Step::CombatDamage => "Combat Damage",
        Step::EndOfCombat => "End Combat",
        Step::PostCombatMain => "Main 2",
        Step::End => "End Step",
        Step::Cleanup => "Cleanup",
    }
}

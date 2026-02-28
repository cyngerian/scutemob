//! Main render dispatch for play mode.

use ratatui::prelude::*;

use super::app::{InputMode, PlayApp};
use super::panels;

pub fn render(f: &mut Frame, app: &PlayApp) {
    let size = f.area();

    // Top-level vertical split
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Phase bar
            Constraint::Length(3), // Stack (if populated)
            Constraint::Min(10),   // Main area
            Constraint::Length(3), // Action menu / status
            Constraint::Length(6), // Event log
        ])
        .split(size);

    // Phase bar
    panels::phase_bar::render(f, app, chunks[0]);

    // Stack
    panels::stack_view::render(f, app, chunks[1]);

    // Main area: battlefield + hand on left, player sidebar on right
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(30),    // Battlefield + hand
            Constraint::Length(16), // Player sidebar
        ])
        .split(chunks[2]);

    // Left side: battlefield above, hand below
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Battlefield
            Constraint::Percentage(40), // Hand
        ])
        .split(main_chunks[0]);

    panels::battlefield::render(f, app, left_chunks[0]);
    panels::hand_view::render(f, app, left_chunks[1]);

    // Right side: player sidebar
    panels::sidebar::render(f, app, main_chunks[1]);

    // Action menu / status
    panels::action_menu::render(f, app, chunks[3]);

    // Event log
    panels::event_log::render(f, app, chunks[4]);

    // Card detail overlay
    if let InputMode::CardDetail(obj_id) = &app.mode {
        panels::card_detail::render(f, app, *obj_id);
    }
}

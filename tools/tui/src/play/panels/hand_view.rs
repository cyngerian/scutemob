//! Hand view — shows cards in the focused player's hand.

use ratatui::prelude::*;
use ratatui::widgets::*;

use mtg_engine::{CardType, Characteristics, Color as MtgColor, ZoneId};

use crate::play::app::{FocusZone, PlayApp};
use crate::play::panels::card_detail::colored_mana_spans;

/// Single-letter type indicator for a card.
fn type_indicator(c: &Characteristics) -> &'static str {
    if c.card_types.contains(&CardType::Creature) {
        "C"
    } else if c.card_types.contains(&CardType::Instant) {
        "I"
    } else if c.card_types.contains(&CardType::Sorcery) {
        "S"
    } else if c.card_types.contains(&CardType::Enchantment) {
        "E"
    } else if c.card_types.contains(&CardType::Artifact) {
        "A"
    } else if c.card_types.contains(&CardType::Planeswalker) {
        "W"
    } else if c.card_types.contains(&CardType::Land) {
        "L"
    } else {
        "?"
    }
}

/// Map a card's MTG color identity to a terminal color.
/// Multicolor = Gold, colorless = Gray, mono = the MTG color.
fn card_color(c: &Characteristics) -> Color {
    let colors = &c.colors;
    if colors.len() > 1 {
        Color::Yellow // gold for multicolor
    } else if colors.contains(&MtgColor::White) {
        Color::White
    } else if colors.contains(&MtgColor::Blue) {
        Color::Rgb(100, 150, 255)
    } else if colors.contains(&MtgColor::Black) {
        Color::Rgb(180, 140, 200) // light purple so it's visible on dark bg
    } else if colors.contains(&MtgColor::Red) {
        Color::Rgb(255, 100, 80)
    } else if colors.contains(&MtgColor::Green) {
        Color::Rgb(80, 220, 80)
    } else if c.card_types.contains(&CardType::Land) {
        Color::Rgb(180, 140, 90) // earthy brown for lands
    } else {
        Color::Gray // colorless artifacts etc.
    }
}

pub fn render(f: &mut Frame, app: &PlayApp, area: Rect) {
    let hand_zone = ZoneId::Hand(app.focused_player);
    let hand_objs = app.state.objects_in_zone(&hand_zone);
    let is_human = app.focused_player == app.human_player;

    let title = if is_human {
        " Your Hand ".to_string()
    } else {
        format!(" P{}'s Hand ({} cards) ", app.focused_player.0, hand_objs.len())
    };

    let items: Vec<Line> = if !is_human {
        vec![Line::from(Span::styled(
            format!(" {} card(s) face down", hand_objs.len()),
            Style::default().fg(Color::DarkGray),
        ))]
    } else if hand_objs.is_empty() {
        vec![Line::from(Span::styled(
            " (empty hand)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        hand_objs
            .iter()
            .enumerate()
            .map(|(i, obj)| {
                let selected = i == app.selected_hand_idx;
                let prefix = if selected { ">" } else { " " };
                let c = &obj.characteristics;
                let ti = type_indicator(c);

                // Build: "> [1] Card Name  C {2}{G}"
                // where each mana symbol is individually colored
                let base_color = card_color(c);
                let dim = Color::DarkGray;

                let mut spans: Vec<Span<'static>> = Vec::new();
                if selected {
                    spans.push(Span::styled(
                        format!("{} [{}] ", prefix, i + 1),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(
                        c.name.clone(),
                        Style::default().fg(base_color).add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(
                        format!("  {} ", ti),
                        Style::default().fg(dim).add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(
                        format!("{} [{}] ", prefix, i + 1),
                        Style::default().fg(dim),
                    ));
                    spans.push(Span::styled(
                        c.name.clone(),
                        Style::default().fg(base_color),
                    ));
                    spans.push(Span::styled(format!("  {} ", ti), Style::default().fg(dim)));
                }
                // Append colored mana cost spans
                if let Some(ref cost) = c.mana_cost {
                    spans.extend(colored_mana_spans(cost));
                }
                Line::from(spans)
            })
            .collect()
    };

    let focused = app.focus_zone == FocusZone::Hand && is_human;
    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let paragraph = Paragraph::new(items).block(block);
    f.render_widget(paragraph, area);
}

//! Card detail popup — shows full oracle text and stats.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;
use mtg_engine::{ManaCost, ObjectId};

pub fn render(f: &mut Frame, app: &PlayApp, obj_id: ObjectId) {
    let area = centered_rect(60, 50, f.area());

    // Clear the area
    f.render_widget(Clear, area);

    let (title, lines) = if let Ok(obj) = app.state.object(obj_id) {
        let c = &obj.characteristics;
        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            &c.name,
            Style::default().add_modifier(Modifier::BOLD),
        )));

        if let Some(ref cost) = c.mana_cost {
            lines.push(Line::from(format!("Cost: {}", format_mana_cost(cost))));
        }

        let types: Vec<String> = c.card_types.iter().map(|t| format!("{:?}", t)).collect();
        lines.push(Line::from(format!("Types: {}", types.join(" "))));

        if let (Some(p), Some(t)) = (c.power, c.toughness) {
            lines.push(Line::from(format!("P/T: {}/{}", p, t)));
        }

        if !c.keywords.is_empty() {
            let kws: Vec<String> = c.keywords.iter().map(|k| format!("{:?}", k)).collect();
            lines.push(Line::from(format!("Keywords: {}", kws.join(", "))));
        }

        if !c.rules_text.is_empty() {
            lines.push(Line::from(""));
            for line in c.rules_text.lines() {
                lines.push(Line::from(line.to_string()));
            }
        }

        (c.name.clone(), lines)
    } else {
        ("Unknown".to_string(), vec![Line::from("Object not found")])
    };

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Format a ManaCost in compact MTG notation: {W}{W}{2}, {U}, etc.
/// Colored mana first (WUBRG order), then colorless {C}, then generic {N}.
/// Returns "(none)" for a zero-cost spell.
pub fn format_mana_cost(cost: &ManaCost) -> String {
    let mut parts = Vec::new();
    for _ in 0..cost.white {
        parts.push("{W}".to_string());
    }
    for _ in 0..cost.blue {
        parts.push("{U}".to_string());
    }
    for _ in 0..cost.black {
        parts.push("{B}".to_string());
    }
    for _ in 0..cost.red {
        parts.push("{R}".to_string());
    }
    for _ in 0..cost.green {
        parts.push("{G}".to_string());
    }
    for _ in 0..cost.colorless {
        parts.push("{C}".to_string());
    }
    if cost.generic > 0 {
        parts.push(format!("{{{}}}", cost.generic));
    }
    if parts.is_empty() {
        "{0}".to_string()
    } else {
        parts.join("")
    }
}

/// Terminal color for a single mana symbol.
fn mana_symbol_color(sym: char) -> Color {
    match sym {
        'W' => Color::White,
        'U' => Color::Rgb(100, 150, 255),
        'B' => Color::Rgb(180, 140, 200),
        'R' => Color::Rgb(255, 100, 80),
        'G' => Color::Rgb(80, 220, 80),
        'C' => Color::Gray,
        _ => Color::Rgb(200, 200, 200), // generic numbers
    }
}

/// Build colored Spans for a mana cost — each symbol gets its MTG color.
/// E.g., {2}{G}{G} renders as gray "2", green "G", green "G".
pub fn colored_mana_spans(cost: &ManaCost) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut push_sym = |sym: char, count: u32| {
        for _ in 0..count {
            spans.push(Span::styled(
                format!("{{{}}}", sym),
                Style::default().fg(mana_symbol_color(sym)),
            ));
        }
    };
    push_sym('W', cost.white);
    push_sym('U', cost.blue);
    push_sym('B', cost.black);
    push_sym('R', cost.red);
    push_sym('G', cost.green);
    push_sym('C', cost.colorless);
    if cost.generic > 0 {
        spans.push(Span::styled(
            format!("{{{}}}", cost.generic),
            Style::default().fg(mana_symbol_color('0')),
        ));
    }
    if spans.is_empty() {
        spans.push(Span::styled(
            "{0}".to_string(),
            Style::default().fg(Color::Gray),
        ));
    }
    spans
}

/// Helper: center a rect with given percentage width/height.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

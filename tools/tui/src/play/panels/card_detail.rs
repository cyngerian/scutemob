//! Card detail popup — shows full oracle text and stats.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;
use mtg_engine::{CardType, Characteristics, Color as MtgColor, ManaCost, ObjectId};

pub fn render(f: &mut Frame, app: &PlayApp, obj_id: ObjectId) {
    let area = centered_rect(60, 50, f.area());

    // Clear the area
    f.render_widget(Clear, area);

    let (title, lines) = if let Ok(obj) = app.state.object(obj_id) {
        let c = &obj.characteristics;
        let color = card_color(c);
        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            &c.name,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));

        if let Some(ref cost) = c.mana_cost {
            let mut cost_line = vec![Span::styled("Cost: ", Style::default().fg(Color::DarkGray))];
            cost_line.extend(colored_mana_spans(cost));
            lines.push(Line::from(cost_line));
        }

        // Build full type line: "Legendary Creature — Human Soldier"
        let mut type_parts = Vec::new();
        for st in c.supertypes.iter() {
            type_parts.push(format!("{:?}", st));
        }
        for ct in c.card_types.iter() {
            type_parts.push(format!("{:?}", ct));
        }
        let mut type_line = type_parts.join(" ");
        if !c.subtypes.is_empty() {
            let subs: Vec<String> = c.subtypes.iter().map(|s| format!("{:?}", s)).collect();
            type_line.push_str(&format!(" — {}", subs.join(" ")));
        }
        lines.push(Line::from(type_line));

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

/// Map a card's MTG color identity to a terminal color.
/// Multicolor = Gold, mono = the MTG color, land = color by name, colorless = Gray.
/// Falls back to mana cost colors if `colors` is empty (CR 202.2).
pub fn card_color(c: &Characteristics) -> Color {
    // Use explicit colors if populated
    let colors = &c.colors;
    if colors.len() > 1 {
        return Color::Yellow; // gold for multicolor
    }
    if colors.contains(&MtgColor::White) {
        return Color::White;
    }
    if colors.contains(&MtgColor::Blue) {
        return Color::Rgb(100, 150, 255);
    }
    if colors.contains(&MtgColor::Black) {
        return Color::Rgb(180, 140, 200); // light purple for visibility on dark bg
    }
    if colors.contains(&MtgColor::Red) {
        return Color::Rgb(255, 100, 80);
    }
    if colors.contains(&MtgColor::Green) {
        return Color::Rgb(80, 220, 80);
    }

    // Fallback: derive color from mana cost (CR 202.2)
    if let Some(ref cost) = c.mana_cost {
        let mut count = 0u8;
        let mut last = Color::Gray;
        if cost.white > 0 {
            count += 1;
            last = Color::White;
        }
        if cost.blue > 0 {
            count += 1;
            last = Color::Rgb(100, 150, 255);
        }
        if cost.black > 0 {
            count += 1;
            last = Color::Rgb(180, 140, 200);
        }
        if cost.red > 0 {
            count += 1;
            last = Color::Rgb(255, 100, 80);
        }
        if cost.green > 0 {
            count += 1;
            last = Color::Rgb(80, 220, 80);
        }
        if count > 1 {
            return Color::Yellow; // multicolor
        }
        if count == 1 {
            return last;
        }
    }

    // Lands: color by name
    if c.card_types.contains(&CardType::Land) {
        return land_color_by_name(&c.name);
    }

    Color::Gray // colorless artifacts etc.
}

/// Assign basic lands their MTG color; non-basic lands get earthy brown.
fn land_color_by_name(name: &str) -> Color {
    let lower = name.to_lowercase();
    if lower.contains("forest") {
        Color::Rgb(80, 220, 80) // green
    } else if lower.contains("island") {
        Color::Rgb(100, 150, 255) // blue
    } else if lower.contains("mountain") {
        Color::Rgb(255, 100, 80) // red
    } else if lower.contains("swamp") {
        Color::Rgb(180, 140, 200) // purple/black
    } else if lower.contains("plains") {
        Color::White
    } else {
        Color::Rgb(180, 160, 120) // non-basic: lighter earthy tone
    }
}

/// Format a ManaCost in compact MTG notation: {W}{W}{2}, {U}, etc.
/// Colored mana first (WUBRG order), then colorless {C}, then generic {N}.
/// Returns "(none)" for a zero-cost spell.
#[allow(dead_code)]
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
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

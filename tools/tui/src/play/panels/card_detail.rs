//! Card detail popup — shows full oracle text and stats.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::play::app::PlayApp;
use mtg_engine::ObjectId;

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
            lines.push(Line::from(format!(
                "Cost: W:{} U:{} B:{} R:{} G:{} C:{} G:{}",
                cost.white,
                cost.blue,
                cost.black,
                cost.red,
                cost.green,
                cost.colorless,
                cost.generic
            )));
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

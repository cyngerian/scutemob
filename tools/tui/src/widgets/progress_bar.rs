use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Render a colored horizontal progress bar as a `Line`.
/// `ratio` is in [0.0, 1.0]. `width` is the total terminal column width to fill.
/// The label is appended after the bar.
pub fn progress_bar(ratio: f64, width: u16, label: &str, color: Color) -> Line<'static> {
    let label = label.to_owned();
    let label_width = (label.len() + 1) as u16; // +1 for space
    let bar_width = width.saturating_sub(label_width);
    let filled = ((ratio.clamp(0.0, 1.0) * bar_width as f64) as u16).min(bar_width);
    let empty = bar_width - filled;

    Line::from(vec![
        Span::styled("█".repeat(filled as usize), Style::default().fg(color)),
        Span::styled("░".repeat(empty as usize), Style::default().fg(Color::DarkGray)),
        Span::raw(" "),
        Span::raw(label),
    ])
}

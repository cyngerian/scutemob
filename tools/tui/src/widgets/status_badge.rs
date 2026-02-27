use ratatui::{style::Style, text::Span};

use crate::theme;

/// Return a styled `Span` showing symbol + status for a cell.
#[allow(dead_code)]
pub fn status_span(status: &str) -> Span<'static> {
    let color = theme::status_color(status);
    let symbol = theme::status_symbol(status);
    let text = format!("{} {}", symbol, status);
    Span::styled(text, Style::default().fg(color))
}

/// Return a compact symbol-only `Span` (for table cells with limited width).
#[allow(dead_code)]
pub fn status_symbol_span(status: &str) -> Span<'static> {
    let color = theme::status_color(status);
    let symbol = theme::status_symbol(status);
    Span::styled(symbol.to_owned(), Style::default().fg(color))
}

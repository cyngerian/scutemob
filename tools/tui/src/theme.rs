use ratatui::style::Color;

// MTG color identity palette
pub const WHITE: Color = Color::White;
pub const BLUE: Color = Color::Cyan;
pub const BLACK: Color = Color::DarkGray;
pub const RED: Color = Color::Red;
pub const GREEN: Color = Color::Green;
pub const GOLD: Color = Color::Yellow;
pub const ARTIFACT: Color = Color::Gray;
pub const BACKGROUND: Color = Color::Black;

/// Color for a status string (ability coverage, corner case status).
pub fn status_color(status: &str) -> Color {
    match status {
        "validated" | "COVERED" => GREEN,
        "complete" => BLUE,
        "partial" | "PARTIAL" => GOLD,
        "none" | "GAP" => RED,
        "deferred" | "DEFERRED" => ARTIFACT,
        "n/a" => BLACK,
        _ => WHITE,
    }
}

/// Symbol for a status string.
pub fn status_symbol(status: &str) -> &'static str {
    match status {
        "validated" | "COVERED" => "✓",
        "complete" => "●",
        "partial" | "PARTIAL" => "◑",
        "none" | "GAP" => "○",
        "deferred" | "DEFERRED" => "◌",
        "n/a" => "—",
        _ => "?",
    }
}

/// Color for a review severity.
pub fn severity_color(severity: &str) -> Color {
    match severity {
        "HIGH" => RED,
        "MEDIUM" => GOLD,
        "LOW" => ARTIFACT,
        "INFO" => BLUE,
        "CRITICAL" => Color::Magenta,
        _ => WHITE,
    }
}

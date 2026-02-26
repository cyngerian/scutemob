use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    // Outer: content area + status bar
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    // Content: file list (left) + rendered doc (right)
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(outer[0]);

    render_file_list(f, panels[0], app);
    render_content(f, panels[1], app);
    render_status(f, outer[1], app);
}

// ─── file list ───────────────────────────────────────────────────────────────

fn render_file_list(f: &mut Frame, area: Rect, app: &mut App) {
    let visible = app.visible_files();
    let mut items: Vec<ListItem> = vec![];
    let mut last_group = String::new();

    for file in &visible {
        // Group header when group changes
        if file.group != last_group {
            last_group = file.group.clone();
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", file.group),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ])));
        }

        // Strip the "group/" prefix from display name
        let short = file
            .display
            .splitn(2, '/')
            .nth(1)
            .unwrap_or(&file.display);

        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(short.to_owned(), Style::default().fg(Color::White)),
        ])));
    }

    // Title shows search state
    let title = if app.search_mode {
        format!(" Files — /{} ", app.search)
    } else if !app.search.is_empty() {
        format!(" Files [/{}] ", app.search)
    } else {
        " Files ".to_string()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(if app.search_mode {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

// ─── content panel ───────────────────────────────────────────────────────────

fn render_content(f: &mut Frame, area: Rect, app: &App) {
    let title = app
        .selected_file()
        .map(|f| format!(" {} ", f.display))
        .unwrap_or_else(|| " (no file selected) ".to_string());

    match &app.content {
        None => {
            f.render_widget(
                Paragraph::new("No file selected or file could not be read.")
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .style(Style::default().fg(Color::DarkGray)),
                area,
            );
        }
        Some(content) => {
            let text = tui_markdown::from_str(content);
            f.render_widget(
                Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .scroll((app.scroll, 0)),
                area,
            );
        }
    }
}

// ─── status bar ──────────────────────────────────────────────────────────────

fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let help = if app.search_mode {
        " Type to filter  Enter/Esc:done  Backspace:delete  Ctrl+C:cancel "
    } else {
        " j/k:nav files  J/K:scroll content  PgDn/PgUp:scroll  /:search  q:quit "
    };

    let scroll_info = format!(" line {} ", app.scroll + 1);

    let width = area.width as usize;
    let padded = format!(
        "{:<width$}{}",
        help,
        scroll_info,
        width = width.saturating_sub(scroll_info.len())
    );

    f.render_widget(
        Paragraph::new(padded)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        area,
    );
}

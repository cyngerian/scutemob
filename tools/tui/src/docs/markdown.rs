//! Custom markdown-to-ratatui renderer.
//!
//! Replaces `tui-markdown` to support tables, code blocks, and proper list formatting.
//! Uses `pulldown-cmark` for parsing.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, CodeBlockKind, HeadingLevel};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Convert a markdown string into ratatui Lines for rendering in a Paragraph widget.
pub fn render_markdown(input: &str) -> Vec<Line<'static>> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(input, opts);
    let events: Vec<Event> = parser.collect();

    let mut renderer = MdRenderer::new();
    renderer.process(&events);
    renderer.lines
}

struct MdRenderer {
    lines: Vec<Line<'static>>,
    /// Spans accumulated for the current line
    current_spans: Vec<Span<'static>>,
    /// Style stack for nested formatting
    style_stack: Vec<Style>,
    /// Current indentation prefix (for lists, blockquotes)
    indent: String,
    /// List nesting: each entry is Some(counter) for ordered, None for unordered
    list_stack: Vec<Option<u64>>,
    /// Are we inside a code block?
    in_code_block: bool,
    /// Are we inside a table?
    in_table: bool,
    /// Table rows accumulated (each row is a vec of cell strings)
    table_rows: Vec<Vec<String>>,
    /// Current table row being built
    current_row: Vec<String>,
    /// Current table cell text
    current_cell: String,
    /// Is this the table header row?
    in_table_head: bool,
    /// Are we inside a heading?
    in_heading: bool,
    heading_level: u8,
}

impl MdRenderer {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            current_spans: Vec::new(),
            style_stack: vec![Style::default()],
            indent: String::new(),
            list_stack: Vec::new(),
            in_code_block: false,
            in_table: false,
            table_rows: Vec::new(),
            current_row: Vec::new(),
            current_cell: String::new(),
            in_table_head: false,
            in_heading: false,
            heading_level: 0,
        }
    }

    fn current_style(&self) -> Style {
        self.style_stack.last().copied().unwrap_or_default()
    }

    fn push_style(&mut self, style: Style) {
        self.style_stack.push(style);
    }

    fn pop_style(&mut self) {
        if self.style_stack.len() > 1 {
            self.style_stack.pop();
        }
    }

    fn flush_line(&mut self) {
        if !self.current_spans.is_empty() {
            let spans = std::mem::take(&mut self.current_spans);
            self.lines.push(Line::from(spans));
        }
    }

    fn process(&mut self, events: &[Event]) {
        for event in events {
            match event {
                Event::Start(tag) => self.handle_start(tag),
                Event::End(tag) => self.handle_end(tag),
                Event::Text(text) => self.handle_text(text),
                Event::Code(code) => self.handle_inline_code(code),
                Event::SoftBreak => self.handle_soft_break(),
                Event::HardBreak => self.handle_hard_break(),
                Event::Rule => self.handle_rule(),
                Event::TaskListMarker(checked) => self.handle_task_list(*checked),
                _ => {}
            }
        }
        self.flush_line();
    }

    fn handle_start(&mut self, tag: &Tag) {
        match tag {
            Tag::Heading { level, .. } => {
                self.flush_line();
                self.in_heading = true;
                self.heading_level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                let prefix = "#".repeat(self.heading_level as usize);
                let style = heading_style(self.heading_level);
                self.current_spans.push(Span::styled(
                    format!("{} ", prefix),
                    style,
                ));
                self.push_style(style);
            }
            Tag::Paragraph => {
                self.flush_line();
            }
            Tag::BlockQuote(_) => {
                self.flush_line();
                self.indent.push_str("  │ ");
            }
            Tag::CodeBlock(kind) => {
                self.flush_line();
                self.in_code_block = true;
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        if lang.is_empty() { "" } else { lang.as_ref() }
                    }
                    CodeBlockKind::Indented => "",
                };
                if !lang.is_empty() {
                    self.lines.push(Line::from(Span::styled(
                        format!("  ┌─ {} ", lang),
                        Style::default().fg(Color::DarkGray),
                    )));
                } else {
                    self.lines.push(Line::from(Span::styled(
                        "  ┌───",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
            Tag::List(start) => {
                self.flush_line();
                self.list_stack.push(*start);
            }
            Tag::Item => {
                self.flush_line();
                let depth = self.list_stack.len().saturating_sub(1);
                let indent = "  ".repeat(depth);
                if let Some(entry) = self.list_stack.last_mut() {
                    match entry {
                        Some(n) => {
                            self.current_spans.push(Span::styled(
                                format!("{}{}. ", indent, n),
                                Style::default().fg(Color::Cyan),
                            ));
                            *n += 1;
                        }
                        None => {
                            self.current_spans.push(Span::styled(
                                format!("{}• ", indent),
                                Style::default().fg(Color::Cyan),
                            ));
                        }
                    }
                }
            }
            Tag::Emphasis => {
                let style = self.current_style().add_modifier(Modifier::ITALIC);
                self.push_style(style);
            }
            Tag::Strong => {
                let style = self.current_style().add_modifier(Modifier::BOLD);
                self.push_style(style);
            }
            Tag::Strikethrough => {
                let style = self.current_style().fg(Color::DarkGray);
                self.push_style(style);
            }
            Tag::Link { dest_url, .. } => {
                let style = self.current_style().fg(Color::Blue);
                self.push_style(style);
                // Store URL to show after link text
                let _ = dest_url; // we'll handle in End
            }
            Tag::Table(_) => {
                self.flush_line();
                self.in_table = true;
                self.table_rows.clear();
            }
            Tag::TableHead => {
                self.in_table_head = true;
                self.current_row = Vec::new();
            }
            Tag::TableRow => {
                self.current_row = Vec::new();
            }
            Tag::TableCell => {
                self.current_cell = String::new();
            }
            _ => {}
        }
    }

    fn handle_end(&mut self, tag: &TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                self.pop_style();
                self.in_heading = false;
                self.flush_line();
                self.lines.push(Line::from(""));
            }
            TagEnd::Paragraph => {
                self.flush_line();
                self.lines.push(Line::from(""));
            }
            TagEnd::BlockQuote(_) => {
                // Remove the last "  │ " from indent
                let new_len = self.indent.len().saturating_sub(4);
                self.indent.truncate(new_len);
                self.flush_line();
            }
            TagEnd::CodeBlock => {
                self.in_code_block = false;
                self.lines.push(Line::from(Span::styled(
                    "  └───",
                    Style::default().fg(Color::DarkGray),
                )));
                self.lines.push(Line::from(""));
            }
            TagEnd::List(_) => {
                self.list_stack.pop();
                if self.list_stack.is_empty() {
                    self.flush_line();
                    self.lines.push(Line::from(""));
                }
            }
            TagEnd::Item => {
                self.flush_line();
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => {
                self.pop_style();
            }
            TagEnd::Link => {
                self.pop_style();
            }
            TagEnd::Table => {
                self.render_table();
                self.in_table = false;
                self.table_rows.clear();
                self.lines.push(Line::from(""));
            }
            TagEnd::TableHead => {
                self.in_table_head = false;
                self.table_rows.push(std::mem::take(&mut self.current_row));
            }
            TagEnd::TableRow => {
                self.table_rows.push(std::mem::take(&mut self.current_row));
            }
            TagEnd::TableCell => {
                self.current_row.push(std::mem::take(&mut self.current_cell));
            }
            _ => {}
        }
    }

    fn handle_text(&mut self, text: &pulldown_cmark::CowStr) {
        if self.in_table {
            self.current_cell.push_str(text);
            return;
        }

        if self.in_code_block {
            // Render each line of code block with prefix
            for line in text.lines() {
                self.lines.push(Line::from(vec![
                    Span::styled("  │ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::Green),
                    ),
                ]));
            }
            return;
        }

        let style = self.current_style();
        if !self.indent.is_empty() {
            // Prepend indent to first span if this is the start of a line
            if self.current_spans.is_empty() {
                self.current_spans.push(Span::styled(
                    self.indent.clone(),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }
        self.current_spans.push(Span::styled(text.to_string(), style));
    }

    fn handle_inline_code(&mut self, code: &pulldown_cmark::CowStr) {
        if self.in_table {
            self.current_cell.push_str(code);
            return;
        }
        self.current_spans.push(Span::styled(
            format!("`{}`", code),
            Style::default().fg(Color::Yellow),
        ));
    }

    fn handle_soft_break(&mut self) {
        if self.in_table { return; }
        // Soft break = space (word wrapping handles the rest)
        self.current_spans.push(Span::raw(" "));
    }

    fn handle_hard_break(&mut self) {
        if self.in_table { return; }
        self.flush_line();
    }

    fn handle_rule(&mut self) {
        self.flush_line();
        self.lines.push(Line::from(Span::styled(
            "────────────────────────────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )));
        self.lines.push(Line::from(""));
    }

    fn handle_task_list(&mut self, checked: bool) {
        let marker = if checked { "☑ " } else { "☐ " };
        let color = if checked { Color::Green } else { Color::Gray };
        // Replace the bullet that was already added by Item start
        if let Some(last) = self.current_spans.last_mut() {
            *last = Span::styled(
                format!("{}{}", last.content.trim_end_matches("• ").trim_end_matches(". "), marker),
                Style::default().fg(color),
            );
        }
    }

    /// Render accumulated table rows with box-drawing borders.
    fn render_table(&mut self) {
        if self.table_rows.is_empty() { return; }

        let num_cols = self.table_rows.iter().map(|r| r.len()).max().unwrap_or(0);
        if num_cols == 0 { return; }

        // Calculate column widths (min 3, max 40)
        let mut col_widths: Vec<usize> = vec![3; num_cols];
        for row in &self.table_rows {
            for (i, cell) in row.iter().enumerate() {
                if i < num_cols {
                    col_widths[i] = col_widths[i].max(cell.trim().len()).min(40);
                }
            }
        }

        let border_style = Style::default().fg(Color::DarkGray);
        let header_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
        let cell_style = Style::default().fg(Color::White);

        // Top border
        let top = format!(
            "┌{}┐",
            col_widths.iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┬")
        );
        self.lines.push(Line::from(Span::styled(top, border_style)));

        for (row_idx, row) in self.table_rows.iter().enumerate() {
            // Row content
            let mut spans: Vec<Span<'static>> = vec![Span::styled("│", border_style)];
            let style = if row_idx == 0 { header_style } else { cell_style };
            for (i, width) in col_widths.iter().enumerate() {
                let cell = row.get(i).map(|s| s.trim()).unwrap_or("");
                let truncated = if cell.len() > *width {
                    format!("{:.w$}", cell, w = width)
                } else {
                    format!("{:<w$}", cell, w = width)
                };
                spans.push(Span::styled(format!(" {} ", truncated), style));
                spans.push(Span::styled("│", border_style));
            }
            self.lines.push(Line::from(spans));

            // Separator after header
            if row_idx == 0 {
                let sep = format!(
                    "├{}┤",
                    col_widths.iter()
                        .map(|w| "─".repeat(w + 2))
                        .collect::<Vec<_>>()
                        .join("┼")
                );
                self.lines.push(Line::from(Span::styled(sep, border_style)));
            }
        }

        // Bottom border
        let bottom = format!(
            "└{}┘",
            col_widths.iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┴")
        );
        self.lines.push(Line::from(Span::styled(bottom, border_style)));
    }
}

fn heading_style(level: u8) -> Style {
    match level {
        1 => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        2 => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        3 => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    }
}

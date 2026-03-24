use std::path::PathBuf;
use std::sync::mpsc;

use ratatui::widgets::TableState;

use super::data::DashboardData;
use super::parser;

pub const TAB_COUNT: usize = 4;
pub const TAB_NAMES: [&str; TAB_COUNT] = [
    "1:Dashboard",
    "2:Pipeline",
    "3:Cards",
    "4:Milestones",
];

pub enum LiveTestCount {
    Loading,
    Done(u32),
}

pub struct App {
    pub current_tab: usize,
    pub data: DashboardData,
    pub should_quit: bool,

    // Tab 2: Pipeline
    pub pipeline_scroll: u16,

    // Tab 3: Cards
    pub cards_table_state: TableState,
    /// "all", "todo", "ok", "partial", "stripped"
    pub cards_filter: String,
    /// Scroll offset for the detail pane (DSL view)
    pub cards_detail_scroll: u16,

    // Tab 4: Milestones
    pub milestone_table_state: TableState,

    pub root: PathBuf,

    pub live_test_count: LiveTestCount,
    pub test_count_rx: Option<mpsc::Receiver<u32>>,
}

impl App {
    pub fn new(root: PathBuf) -> Self {
        let data = parser::parse_all(&root);
        let mut milestone_table_state = TableState::default();
        milestone_table_state.select(Some(0));

        let mut app = Self {
            current_tab: 0,
            data,
            should_quit: false,
            pipeline_scroll: 0,
            cards_table_state: TableState::default(),
            cards_filter: "all".to_string(),
            cards_detail_scroll: 0,
            milestone_table_state,
            live_test_count: LiveTestCount::Loading,
            test_count_rx: None,
            root,
        };

        let (tx, rx) = mpsc::channel::<u32>();
        let root_clone = app.root.clone();
        std::thread::spawn(move || {
            let result = std::process::Command::new("cargo")
                .args(["test", "--all"])
                .current_dir(&root_clone)
                .env("CARGO_TERM_COLOR", "never")
                .output();
            if let Ok(out) = result {
                let text = String::from_utf8_lossy(&out.stdout);
                let count = parse_live_test_count(&text);
                if count > 0 {
                    let _ = tx.send(count);
                }
            }
        });
        app.test_count_rx = Some(rx);

        app
    }

    pub fn reload(&mut self) {
        self.data = parser::parse_all(&self.root);
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % TAB_COUNT;
    }

    pub fn prev_tab(&mut self) {
        self.current_tab = (self.current_tab + TAB_COUNT - 1) % TAB_COUNT;
    }

    pub fn jump_to_tab(&mut self, idx: usize) {
        if idx < TAB_COUNT {
            self.current_tab = idx;
        }
    }

    // ─── pipeline tab scroll ──────────────────────────────────────────────

    pub fn pipeline_scroll_down(&mut self) {
        self.pipeline_scroll = self.pipeline_scroll.saturating_add(1);
    }

    pub fn pipeline_scroll_up(&mut self) {
        self.pipeline_scroll = self.pipeline_scroll.saturating_sub(1);
    }

    // ─── cards tab scroll ─────────────────────────────────────────────────

    pub fn cards_items_len(&self) -> usize {
        if self.cards_filter == "all" {
            self.data.live_cards.len()
        } else {
            self.data
                .live_cards
                .iter()
                .filter(|e| e.status == self.cards_filter)
                .count()
        }
    }

    pub fn cards_scroll_down(&mut self) {
        let len = self.cards_items_len();
        if len == 0 {
            return;
        }
        let sel = self.cards_table_state.selected().unwrap_or(0);
        self.cards_table_state.select(Some((sel + 1).min(len - 1)));
        self.cards_detail_scroll = 0;
    }

    pub fn cards_scroll_up(&mut self) {
        let sel = self.cards_table_state.selected().unwrap_or(0);
        self.cards_table_state.select(Some(sel.saturating_sub(1)));
        self.cards_detail_scroll = 0;
    }

    pub fn set_cards_filter(&mut self, filter: &str) {
        if self.cards_filter != filter {
            self.cards_filter = filter.to_string();
            self.cards_table_state.select(Some(0));
            self.cards_detail_scroll = 0;
        }
    }

    pub fn cards_detail_scroll_down(&mut self) {
        self.cards_detail_scroll = self.cards_detail_scroll.saturating_add(1);
    }

    pub fn cards_detail_scroll_up(&mut self) {
        self.cards_detail_scroll = self.cards_detail_scroll.saturating_sub(1);
    }

    // ─── milestones tab scroll ────────────────────────────────────────────

    pub fn milestones_len(&self) -> usize {
        self.data.milestones.len()
    }

    pub fn milestone_scroll_down(&mut self) {
        let len = self.milestones_len();
        if len == 0 {
            return;
        }
        let sel = self.milestone_table_state.selected().unwrap_or(0);
        self.milestone_table_state
            .select(Some((sel + 1).min(len - 1)));
    }

    pub fn milestone_scroll_up(&mut self) {
        let sel = self.milestone_table_state.selected().unwrap_or(0);
        self.milestone_table_state
            .select(Some(sel.saturating_sub(1)));
    }
}

fn parse_live_test_count(text: &str) -> u32 {
    text.lines()
        .filter(|l| l.contains("passed;"))
        .filter_map(|l| {
            l.split("passed;")
                .next()
                .and_then(|s| s.split_whitespace().last())
                .and_then(|s| s.parse::<u32>().ok())
        })
        .sum()
}

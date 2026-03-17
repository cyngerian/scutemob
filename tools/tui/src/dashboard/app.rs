use std::path::PathBuf;
use std::sync::mpsc;

use ratatui::widgets::{ListState, TableState};

use super::data::DashboardData;
use super::parser;

pub const TAB_COUNT: usize = 8;
pub const TAB_NAMES: [&str; TAB_COUNT] = [
    "1:Overview",
    "2:Milestones",
    "3:Abilities",
    "4:Corner Cases",
    "5:Reviews",
    "6:Scripts",
    "7:Cards",
    "8:Progress",
];

pub enum LiveTestCount {
    Loading,
    Done(u32),
}

pub struct App {
    pub current_tab: usize,
    pub data: DashboardData,
    pub should_quit: bool,

    // Tab 2: Milestones
    pub milestone_table_state: TableState,

    // Tab 3: Abilities
    pub ability_list_state: ListState,

    // Tab 4: Corner Cases
    pub corner_case_table_state: TableState,
    pub show_gaps_only: bool,

    // Tab 6: Scripts
    pub scripts_table_state: TableState,
    pub scripts_show_pending_only: bool,

    // Tab 7: Cards
    pub cards_table_state: TableState,

    // Tab 8: Progress
    pub progress_focus: u8, // 0=batches, 1=review backlog, 2=workstreams, 3=path to alpha
    pub progress_scroll: u16,
    pub progress_backlog_scroll: u16,
    pub progress_workstream_scroll: u16,
    pub progress_alpha_scroll: u16,
    /// "all", "ready", "blocked", "deferred"
    pub cards_filter: String,
    /// Scroll offset for the detail pane (DSL view)
    pub cards_detail_scroll: u16,

    pub root: PathBuf,

    pub live_test_count: LiveTestCount,
    pub test_count_rx: Option<mpsc::Receiver<u32>>,
}

impl App {
    pub fn new(root: PathBuf) -> Self {
        let data = parser::parse_all(&root);
        let mut milestone_table_state = TableState::default();
        milestone_table_state.select(Some(0));
        let mut corner_case_table_state = TableState::default();
        corner_case_table_state.select(Some(0));

        let mut app = Self {
            current_tab: 0,
            data,
            should_quit: false,
            milestone_table_state,
            ability_list_state: ListState::default(),
            corner_case_table_state,
            show_gaps_only: false,
            scripts_table_state: TableState::default(),
            scripts_show_pending_only: false,
            cards_table_state: TableState::default(),
            cards_filter: "all".to_string(),
            cards_detail_scroll: 0,
            progress_focus: 0,
            progress_scroll: 0,
            progress_backlog_scroll: 0,
            progress_workstream_scroll: 0,
            progress_alpha_scroll: 0,
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

    // ─── milestones tab scroll ───────────────────────────────────────────

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

    // ─── abilities tab scroll ────────────────────────────────────────────

    pub fn ability_items_len(&self) -> usize {
        self.data
            .abilities
            .sections
            .iter()
            .map(|s| s.rows.len() + 1) // +1 for section header
            .sum()
    }

    pub fn ability_scroll_down(&mut self) {
        let len = self.ability_items_len();
        if len == 0 {
            return;
        }
        let sel = self.ability_list_state.selected().unwrap_or(0);
        self.ability_list_state.select(Some((sel + 1).min(len - 1)));
    }

    pub fn ability_scroll_up(&mut self) {
        let sel = self.ability_list_state.selected().unwrap_or(0);
        self.ability_list_state.select(Some(sel.saturating_sub(1)));
    }

    // ─── corner cases tab scroll ─────────────────────────────────────────

    pub fn corner_case_items(&self) -> usize {
        if self.show_gaps_only {
            self.data
                .corner_cases
                .cases
                .iter()
                .filter(|c| c.status == "GAP")
                .count()
        } else {
            self.data.corner_cases.cases.len()
        }
    }

    pub fn corner_case_scroll_down(&mut self) {
        let len = self.corner_case_items();
        if len == 0 {
            return;
        }
        let sel = self.corner_case_table_state.selected().unwrap_or(0);
        self.corner_case_table_state
            .select(Some((sel + 1).min(len - 1)));
    }

    pub fn corner_case_scroll_up(&mut self) {
        let sel = self.corner_case_table_state.selected().unwrap_or(0);
        self.corner_case_table_state
            .select(Some(sel.saturating_sub(1)));
    }

    pub fn toggle_gaps_only(&mut self) {
        self.show_gaps_only = !self.show_gaps_only;
        self.corner_case_table_state.select(Some(0));
    }

    // ─── scripts tab scroll ──────────────────────────────────────────────

    pub fn scripts_items_len(&self) -> usize {
        if self.scripts_show_pending_only {
            self.data
                .scripts
                .entries
                .iter()
                .filter(|e| e.status == "pending_review")
                .count()
        } else {
            self.data.scripts.entries.len()
        }
    }

    pub fn scripts_scroll_down(&mut self) {
        let len = self.scripts_items_len();
        if len == 0 {
            return;
        }
        let sel = self.scripts_table_state.selected().unwrap_or(0);
        self.scripts_table_state
            .select(Some((sel + 1).min(len - 1)));
    }

    pub fn scripts_scroll_up(&mut self) {
        let sel = self.scripts_table_state.selected().unwrap_or(0);
        self.scripts_table_state.select(Some(sel.saturating_sub(1)));
    }

    pub fn toggle_scripts_pending_only(&mut self) {
        self.scripts_show_pending_only = !self.scripts_show_pending_only;
        self.scripts_table_state.select(Some(0));
    }

    // ─── cards tab scroll ───────────────────────────────────────────────

    pub fn cards_items_len(&self) -> usize {
        if self.cards_filter == "all" {
            self.data.cards.entries.len()
        } else {
            self.data
                .cards
                .entries
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

    // ─── progress tab scroll & focus ─────────────────────────────────────

    pub fn progress_focus_left(&mut self) {
        self.progress_focus = self.progress_focus.saturating_sub(1);
    }

    pub fn progress_focus_right(&mut self) {
        self.progress_focus = (self.progress_focus + 1).min(3);
    }

    pub fn progress_scroll_down(&mut self) {
        match self.progress_focus {
            0 => self.progress_scroll = self.progress_scroll.saturating_add(1),
            1 => self.progress_backlog_scroll = self.progress_backlog_scroll.saturating_add(1),
            2 => {
                self.progress_workstream_scroll = self.progress_workstream_scroll.saturating_add(1)
            }
            3 => self.progress_alpha_scroll = self.progress_alpha_scroll.saturating_add(1),
            _ => {}
        }
    }

    pub fn progress_scroll_up(&mut self) {
        match self.progress_focus {
            0 => self.progress_scroll = self.progress_scroll.saturating_sub(1),
            1 => self.progress_backlog_scroll = self.progress_backlog_scroll.saturating_sub(1),
            2 => {
                self.progress_workstream_scroll = self.progress_workstream_scroll.saturating_sub(1)
            }
            3 => self.progress_alpha_scroll = self.progress_alpha_scroll.saturating_sub(1),
            _ => {}
        }
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

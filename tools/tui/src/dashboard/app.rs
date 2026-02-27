use std::path::PathBuf;

use ratatui::widgets::{ListState, TableState};

use super::data::DashboardData;
use super::parser;

pub const TAB_COUNT: usize = 6;
pub const TAB_NAMES: [&str; TAB_COUNT] = [
    "1:Overview",
    "2:Milestones",
    "3:Abilities",
    "4:Corner Cases",
    "5:Reviews",
    "6:Scripts",
];

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

    pub root: PathBuf,
}

impl App {
    pub fn new(root: PathBuf) -> Self {
        let data = parser::parse_all(&root);
        let mut milestone_table_state = TableState::default();
        milestone_table_state.select(Some(0));
        let mut corner_case_table_state = TableState::default();
        corner_case_table_state.select(Some(0));

        Self {
            current_tab: 0,
            data,
            should_quit: false,
            milestone_table_state,
            ability_list_state: ListState::default(),
            corner_case_table_state,
            show_gaps_only: false,
            scripts_table_state: TableState::default(),
            scripts_show_pending_only: false,
            root,
        }
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
        self.scripts_table_state.select(Some((sel + 1).min(len - 1)));
    }

    pub fn scripts_scroll_up(&mut self) {
        let sel = self.scripts_table_state.selected().unwrap_or(0);
        self.scripts_table_state.select(Some(sel.saturating_sub(1)));
    }

    pub fn toggle_scripts_pending_only(&mut self) {
        self.scripts_show_pending_only = !self.scripts_show_pending_only;
        self.scripts_table_state.select(Some(0));
    }
}

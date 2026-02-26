use std::{fs, path::PathBuf};

use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct DocFile {
    /// Display name shown in the file list (relative path from workspace root)
    pub display: String,
    /// Absolute path on disk
    pub path: PathBuf,
    /// Group label: "root", "docs", "memory", etc.
    pub group: String,
}

pub struct App {
    pub all_files: Vec<DocFile>,
    pub list_state: ListState,
    /// Current vertical scroll offset for the content panel
    pub scroll: u16,
    /// Cached content of the currently open file
    pub content: Option<String>,
    /// Cached approximate line count of `content` (for scroll capping)
    pub content_lines: u16,
    /// Search string (empty = no filter)
    pub search: String,
    pub search_mode: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(root: &PathBuf, initial_file: Option<String>) -> Self {
        let all_files = discover_docs(root);

        let mut app = Self {
            all_files,
            list_state: ListState::default(),
            scroll: 0,
            content: None,
            content_lines: 0,
            search: String::new(),
            search_mode: false,
            should_quit: false,
        };

        // Auto-select initial file or default to first
        let start_idx = if let Some(name) = initial_file {
            app.all_files
                .iter()
                .position(|f| f.display.contains(&name))
                .unwrap_or(0)
        } else {
            0
        };

        if !app.all_files.is_empty() {
            app.list_state.select(Some(start_idx));
            app.load_selected();
        }

        app
    }

    /// Files visible after applying the current search filter.
    pub fn visible_files(&self) -> Vec<&DocFile> {
        let q = self.search.to_lowercase();
        self.all_files
            .iter()
            .filter(|f| q.is_empty() || f.display.to_lowercase().contains(&q))
            .collect()
    }

    pub fn selected_file(&self) -> Option<&DocFile> {
        let sel = self.list_state.selected()?;
        self.visible_files().into_iter().nth(sel)
    }

    pub fn load_selected(&mut self) {
        self.scroll = 0;
        self.content = self.selected_file().and_then(|f| fs::read_to_string(&f.path).ok());
        self.content_lines = self
            .content
            .as_deref()
            .map(|s| s.lines().count() as u16)
            .unwrap_or(0);
    }

    // ─── navigation ──────────────────────────────────────────────────────────

    pub fn list_down(&mut self) {
        let len = self.visible_files().len();
        if len == 0 { return; }
        let sel = self.list_state.selected().unwrap_or(0);
        let next = (sel + 1).min(len - 1);
        self.list_state.select(Some(next));
        self.load_selected();
    }

    pub fn list_up(&mut self) {
        let sel = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(sel.saturating_sub(1)));
        self.load_selected();
    }

    pub fn content_down(&mut self, amount: u16) {
        self.scroll = self.scroll.saturating_add(amount).min(self.content_lines);
    }

    pub fn content_up(&mut self, amount: u16) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    // ─── search ──────────────────────────────────────────────────────────────

    pub fn enter_search(&mut self) {
        self.search_mode = true;
    }

    pub fn exit_search(&mut self) {
        self.search_mode = false;
        // Clamp selection to visible range
        let len = self.visible_files().len();
        if len == 0 {
            self.list_state.select(None);
            self.content = None;
        } else {
            let sel = self.list_state.selected().unwrap_or(0).min(len - 1);
            self.list_state.select(Some(sel));
            self.load_selected();
        }
    }

    pub fn search_push(&mut self, c: char) {
        self.search.push(c);
        // Reset selection to top of filtered list
        let len = self.visible_files().len();
        if len > 0 {
            self.list_state.select(Some(0));
            self.load_selected();
        }
    }

    pub fn search_pop(&mut self) {
        self.search.pop();
        let len = self.visible_files().len();
        if len > 0 {
            self.list_state.select(Some(0));
            self.load_selected();
        }
    }

    pub fn clear_search(&mut self) {
        self.search.clear();
        self.exit_search();
    }
}

// ─── file discovery ──────────────────────────────────────────────────────────

fn discover_docs(root: &PathBuf) -> Vec<DocFile> {
    let mut files: Vec<DocFile> = vec![];

    // Root-level .md files
    if let Ok(entries) = fs::read_dir(root) {
        let mut root_files: Vec<_> = entries
            .flatten()
            .filter(|e| {
                e.path().is_file()
                    && e.path().extension().and_then(|x| x.to_str()) == Some("md")
            })
            .collect();
        root_files.sort_by_key(|e| e.file_name());
        for e in root_files {
            files.push(DocFile {
                display: format!("root/{}", e.file_name().to_string_lossy()),
                path: e.path(),
                group: "root".to_string(),
            });
        }
    }

    // docs/ and memory/ subdirectories
    for dir in &["docs", "memory"] {
        let dir_path = root.join(dir);
        if let Ok(entries) = fs::read_dir(&dir_path) {
            let mut dir_files: Vec<_> = entries
                .flatten()
                .filter(|e| {
                    e.path().is_file()
                        && e.path().extension().and_then(|x| x.to_str()) == Some("md")
                })
                .collect();
            dir_files.sort_by_key(|e| e.file_name());
            for e in dir_files {
                files.push(DocFile {
                    display: format!("{}/{}", dir, e.file_name().to_string_lossy()),
                    path: e.path(),
                    group: dir.to_string(),
                });
            }
        }
    }

    files
}

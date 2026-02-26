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
    /// Maps each rendered list item index → file index in visible_files().
    /// `None` = group header (not selectable).
    pub list_map: Vec<Option<usize>>,
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
            list_map: Vec::new(),
            scroll: 0,
            content: None,
            content_lines: 0,
            search: String::new(),
            search_mode: false,
            should_quit: false,
        };

        app.rebuild_list_map();

        // Auto-select initial file or default to first
        let file_idx = if let Some(name) = initial_file {
            app.all_files
                .iter()
                .position(|f| f.display.contains(&name))
                .unwrap_or(0)
        } else {
            0
        };

        if !app.all_files.is_empty() {
            // Convert file index to list index (accounting for group headers)
            let list_idx = app.file_idx_to_list_idx(file_idx).unwrap_or(0);
            app.list_state.select(Some(list_idx));
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

    /// Rebuild the list_map from current visible files.
    /// Must be called whenever the file list or search filter changes.
    pub fn rebuild_list_map(&mut self) {
        let visible = self.visible_files();
        let mut map = Vec::new();
        let mut last_group = String::new();
        for (i, file) in visible.iter().enumerate() {
            if file.group != last_group {
                last_group = file.group.clone();
                map.push(None); // group header
            }
            map.push(Some(i)); // file entry
        }
        self.list_map = map;
    }

    /// Convert a file index (in visible_files) to a list widget index.
    fn file_idx_to_list_idx(&self, file_idx: usize) -> Option<usize> {
        self.list_map
            .iter()
            .position(|entry| *entry == Some(file_idx))
    }

    pub fn selected_file(&self) -> Option<&DocFile> {
        let sel = self.list_state.selected()?;
        let file_idx = (*self.list_map.get(sel)?)?;
        let visible = self.visible_files();
        visible.into_iter().nth(file_idx)
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
        let len = self.list_map.len();
        if len == 0 { return; }
        let sel = self.list_state.selected().unwrap_or(0);
        // Find next file entry (skip headers)
        let mut next = sel + 1;
        while next < len && self.list_map[next].is_none() {
            next += 1;
        }
        if next < len {
            self.list_state.select(Some(next));
            self.load_selected();
        }
    }

    pub fn list_up(&mut self) {
        let sel = self.list_state.selected().unwrap_or(0);
        if sel == 0 { return; }
        // Find previous file entry (skip headers)
        let mut prev = sel - 1;
        while prev > 0 && self.list_map[prev].is_none() {
            prev -= 1;
        }
        // If we landed on a header at index 0, stay put
        if self.list_map[prev].is_some() {
            self.list_state.select(Some(prev));
            self.load_selected();
        }
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
        self.rebuild_list_map();
        if self.list_map.is_empty() {
            self.list_state.select(None);
            self.content = None;
        } else {
            // Select first file entry (skip any leading header)
            let first_file = self.list_map.iter().position(|e| e.is_some()).unwrap_or(0);
            let sel = self.list_state.selected().unwrap_or(first_file).min(self.list_map.len() - 1);
            // If clamped selection landed on a header, advance to next file
            let sel = if self.list_map.get(sel).copied().flatten().is_none() {
                self.list_map.iter().skip(sel).position(|e| e.is_some()).map(|offset| sel + offset).unwrap_or(first_file)
            } else {
                sel
            };
            self.list_state.select(Some(sel));
            self.load_selected();
        }
    }

    pub fn search_push(&mut self, c: char) {
        self.search.push(c);
        self.rebuild_list_map();
        // Select first file entry
        let first_file = self.list_map.iter().position(|e| e.is_some());
        if let Some(idx) = first_file {
            self.list_state.select(Some(idx));
            self.load_selected();
        }
    }

    pub fn search_pop(&mut self) {
        self.search.pop();
        self.rebuild_list_map();
        let first_file = self.list_map.iter().position(|e| e.is_some());
        if let Some(idx) = first_file {
            self.list_state.select(Some(idx));
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

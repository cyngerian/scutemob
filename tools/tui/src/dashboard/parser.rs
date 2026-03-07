use std::{fs, path::Path};

use super::data::*;

// ─── public entry point ────────────────────────────────────────────────────

pub fn parse_all(root: &Path) -> DashboardData {
    let mut reviews = parse_milestone_reviews(root).unwrap_or_default();
    let (src_loc, test_loc) = compute_engine_loc(root);
    reviews.engine_loc = src_loc;
    reviews.test_loc = test_loc;

    DashboardData {
        current_state: parse_claude_md(root).unwrap_or_default(),
        abilities: parse_ability_coverage(root).unwrap_or_default(),
        milestones: parse_roadmap(root, &parse_reviews_for_review_status(root)).unwrap_or_default(),
        corner_cases: parse_corner_case_audit(root).unwrap_or_default(),
        reviews,
        scripts: count_scripts(root).unwrap_or_default(),
        cards: parse_card_worklist(root).unwrap_or_default(),
    }
}

// ─── helpers ────────────────────────────────────────────────────────────────

/// Split a markdown table row on `|`, trim cells, skip first/last empty.
fn table_cells(line: &str) -> Vec<String> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() < 3 {
        return vec![];
    }
    parts[1..parts.len() - 1]
        .iter()
        .map(|s| clean_cell(s))
        .collect()
}

/// Strip markdown bold markers and backticks from a cell value.
fn clean_cell(s: &str) -> String {
    s.trim().replace("**", "").replace('`', "")
}

/// Return true for separator rows like `|---|---|`.
fn is_separator(line: &str) -> bool {
    line.starts_with('|') && line.contains("---")
}

/// Extract the first integer from a string.
fn first_number(s: &str) -> u32 {
    s.split_whitespace()
        .find_map(|tok| tok.trim_matches(|c: char| !c.is_ascii_digit()).parse().ok())
        .unwrap_or(0)
}

// ─── CLAUDE.md ──────────────────────────────────────────────────────────────

fn parse_claude_md(root: &Path) -> anyhow::Result<CurrentState> {
    let content = fs::read_to_string(root.join("CLAUDE.md"))?;
    let mut state = CurrentState::default();
    let mut in_section = false;

    for line in content.lines() {
        if line.starts_with("## Current State") {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        if line.starts_with("## ") || line == "---" {
            break;
        }

        if let Some(rest) = line.strip_prefix("- **Active Milestone**: ") {
            // "M9.5 DONE — advancing to M10 (Networking Layer)"
            // Try to extract the advancing-to milestone.
            if let Some(idx) = rest.find("advancing to ") {
                let after = &rest[idx + "advancing to ".len()..];
                state.active_milestone =
                    after.split_whitespace().next().unwrap_or(rest).to_string();
            } else {
                // Just take the first token.
                state.active_milestone = rest.split_whitespace().next().unwrap_or(rest).to_string();
            }
        } else if let Some(rest) = line.strip_prefix("- **Status**: ") {
            state.status_line = rest.to_string();
            for part in rest.split(';') {
                let part = part.trim();
                if part.contains("tests passing") {
                    state.test_count = first_number(part);
                } else if part.contains("approved") {
                    // "71 approved" or "~78 approved + 10 pending_review scripts"
                    // first_number finds the leading count before "approved"
                    state.script_count = first_number(part);
                }
            }
        } else if let Some(rest) = line.strip_prefix("- **Last Updated**: ") {
            state.last_updated = rest.to_string();
        }
    }

    Ok(state)
}

// ─── ability-coverage.md ───────────────────────────────────────────────────

fn parse_ability_coverage(root: &Path) -> anyhow::Result<AbilityCoverage> {
    let content = fs::read_to_string(root.join("docs/mtg-engine-ability-coverage.md"))?;
    let mut coverage = AbilityCoverage::default();
    let mut mode = ParseMode::None;
    let mut current_section: Option<AbilitySection> = None;
    // Sections 1-12: | Ability | CR | Priority | Status | Engine File(s) | ...
    // Section 13:    | Pattern | Priority | Status | Engine File(s) | ...  (no CR column)
    let mut has_cr_col = true;
    let mut current_gap_priority = String::new();

    enum ParseMode {
        None,
        Summary,
        Section,
        Gaps,
    }

    for line in content.lines() {
        if line.starts_with("## Summary") {
            mode = ParseMode::Summary;
            continue;
        }

        if line.starts_with("## Section ") {
            // Save previous section.
            if let Some(sec) = current_section.take() {
                coverage.sections.push(sec);
            }
            let name = line.trim_start_matches("## ").to_string();
            current_section = Some(AbilitySection { name, rows: vec![] });
            mode = ParseMode::Section;
            has_cr_col = true; // reset; header row will correct this
            continue;
        }

        if line.starts_with("## Priority Gaps") {
            if let Some(sec) = current_section.take() {
                coverage.sections.push(sec);
            }
            mode = ParseMode::Gaps;
            current_gap_priority = String::new();
            continue;
        }

        if line.starts_with("## ")
            && !line.starts_with("## Section ")
            && !line.starts_with("## Summary")
        {
            // Different top-level section — stop section parsing.
            if let Some(sec) = current_section.take() {
                coverage.sections.push(sec);
            }
            mode = ParseMode::None;
            continue;
        }

        match mode {
            ParseMode::None => {}
            ParseMode::Gaps => {
                if line.starts_with("### ") {
                    // "### P2 Gaps (Commander staples)" → extract "P2"
                    current_gap_priority = line
                        .trim_start_matches("### ")
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    continue;
                }
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("**Resolved**") {
                    continue;
                }
                if !current_gap_priority.is_empty() {
                    let clean = trimmed.replace("**", "").replace('`', "");
                    coverage
                        .gap_notes
                        .push(format!("{}: {}", current_gap_priority, clean));
                }
            }
            ParseMode::Summary => {
                if !line.starts_with('|') {
                    continue;
                }
                if is_separator(line) {
                    continue;
                }
                let cells = table_cells(line);
                if cells.len() < 7 {
                    continue;
                }
                // Header row: "Priority", skip
                if cells[0].to_lowercase() == "priority" {
                    continue;
                }
                // Total row: starts with "Total"
                if cells[0].to_lowercase().contains("total") {
                    continue;
                }
                let row = PrioritySummary {
                    priority: cells[0].clone(),
                    total: cells[1].parse().unwrap_or(0),
                    validated: cells[2].parse().unwrap_or(0),
                    complete: cells[3].parse().unwrap_or(0),
                    partial: cells[4].parse().unwrap_or(0),
                    none: cells[5].parse().unwrap_or(0),
                    na: cells[6].parse().unwrap_or(0),
                };
                coverage.summary.push(row);
            }
            ParseMode::Section => {
                if !line.starts_with('|') {
                    continue;
                }
                if is_separator(line) {
                    continue;
                }
                let cells = table_cells(line);
                if cells.len() < 2 {
                    continue;
                }
                // Header row: first cell is "Ability" (sections 1-12) or "Pattern" (section 13).
                // Detect column layout: sections with a "CR" column have it at index 1.
                let first_lower = cells[0].to_lowercase();
                if first_lower == "ability" || first_lower == "pattern" {
                    has_cr_col = cells
                        .get(1)
                        .map(|c| c.trim().to_lowercase() == "cr")
                        .unwrap_or(true);
                    continue;
                }
                // Apply correct column offsets based on layout.
                // With CR:    [0]=name [1]=cr [2]=priority [3]=status [4]=engine [5]=card [6]=script [7]=notes
                // Without CR: [0]=name [1]=priority [2]=status [3]=engine [4]=card [5]=script [6]=depends [7]=notes
                let (cr_idx, prio_idx, status_idx, engine_idx, card_idx, script_idx) = if has_cr_col
                {
                    (Some(1usize), 2usize, 3usize, 4usize, 5usize, 6usize)
                } else {
                    (None, 1, 2, 3, 4, 5)
                };
                let row = AbilityRow {
                    name: cells.first().cloned().unwrap_or_default(),
                    cr: cr_idx
                        .and_then(|i| cells.get(i))
                        .cloned()
                        .unwrap_or_default(),
                    priority: cells.get(prio_idx).cloned().unwrap_or_default(),
                    status: cells.get(status_idx).cloned().unwrap_or_default(),
                    engine_files: cells.get(engine_idx).cloned().unwrap_or_default(),
                    card_def: cells.get(card_idx).cloned().unwrap_or_default(),
                    script: cells.get(script_idx).cloned().unwrap_or_default(),
                    notes: cells.get(7).cloned().unwrap_or_default(),
                };
                if let Some(sec) = current_section.as_mut() {
                    sec.rows.push(row);
                }
            }
        }
    }

    // Push final section.
    if let Some(sec) = current_section.take() {
        coverage.sections.push(sec);
    }

    Ok(coverage)
}

// ─── corner-case-audit.md ──────────────────────────────────────────────────

fn parse_corner_case_audit(root: &Path) -> anyhow::Result<CornerCaseAudit> {
    let content = fs::read_to_string(root.join("docs/mtg-engine-corner-case-audit.md"))?;
    let mut audit = CornerCaseAudit::default();

    enum ParseMode {
        None,
        Summary,
        Table,
    }
    let mut mode = ParseMode::None;

    for line in content.lines() {
        if line.starts_with("## Summary") {
            mode = ParseMode::Summary;
            continue;
        }
        if line.starts_with("## Corner Case Coverage Table") {
            mode = ParseMode::Table;
            continue;
        }
        if line.starts_with("## ") {
            mode = ParseMode::None;
            continue;
        }

        match mode {
            ParseMode::None => {}
            ParseMode::Summary => {
                if !line.starts_with('|') {
                    continue;
                }
                if is_separator(line) {
                    continue;
                }
                let cells = table_cells(line);
                if cells.len() < 2 {
                    continue;
                }
                if cells[0].to_lowercase() == "status" {
                    continue;
                }
                let count: u32 = cells[1].parse().unwrap_or(0);
                match cells[0].to_lowercase().as_str() {
                    "covered" => audit.covered = count,
                    "partial" => audit.partial = count,
                    "gap" => audit.gap = count,
                    "deferred" => audit.deferred = count,
                    _ => {}
                }
            }
            ParseMode::Table => {
                if !line.starts_with('|') {
                    continue;
                }
                if is_separator(line) {
                    continue;
                }
                let cells = table_cells(line);
                if cells.len() < 4 {
                    continue;
                }
                // Header: "#", skip
                if cells[0] == "#" {
                    continue;
                }
                let number: u32 = cells[0].parse().unwrap_or(0);
                if number == 0 {
                    continue;
                }

                // Status cell may be "**COVERED**" etc — already cleaned by clean_cell.
                let status = cells.get(3).cloned().unwrap_or_default();
                let milestone = cells.get(5).cloned().unwrap_or_default();

                audit.cases.push(CornerCase {
                    number,
                    name: cells.get(1).cloned().unwrap_or_default(),
                    cr_refs: cells.get(2).cloned().unwrap_or_default(),
                    status,
                    milestone,
                });
            }
        }
    }

    Ok(audit)
}

// ─── milestone-reviews.md ──────────────────────────────────────────────────

fn parse_milestone_reviews(root: &Path) -> anyhow::Result<ReviewStatistics> {
    let content = fs::read_to_string(root.join("docs/mtg-engine-milestone-reviews.md"))?;
    let mut stats = ReviewStatistics::default();
    let mut in_stats = false;

    for line in content.lines() {
        if line.starts_with("## Statistics") {
            in_stats = true;
            continue;
        }
        if !in_stats {
            continue;
        }
        if line.starts_with("## ") {
            break;
        }

        // Table rows
        if !line.starts_with('|') {
            continue;
        }
        if is_separator(line) {
            continue;
        }
        let cells = table_cells(line);
        if cells.len() < 2 {
            continue;
        }
        if cells[0].to_lowercase() == "metric" {
            continue;
        }

        let metric = cells[0].as_str();
        let value = &cells[1];

        match metric {
            "Total unique issue IDs" => stats.total_issues = first_number(value),
            "HIGH (OPEN)" => stats.high_open = first_number(value),
            "HIGH (CLOSED)" => stats.high_closed = first_number(value),
            "MEDIUM (OPEN)" => stats.medium_open = first_number(value),
            "MEDIUM (CLOSED)" => stats.medium_closed = first_number(value),
            "LOW (OPEN)" => stats.low_open = first_number(value),
            "LOW (CLOSED)" => stats.low_closed = first_number(value),
            "INFO" => stats.info = first_number(value),
            "Milestones reviewed" => stats.milestones_reviewed = first_number(value),
            _ => {}
        }
    }

    Ok(stats)
}

// ─── milestone-reviews.md (review status per milestone) ────────────────────

/// Extract review status per milestone ID from the Table of Contents section.
/// Returns a map of milestone ID → "RE-REVIEWED" | "REVIEWED" | "".
pub fn parse_reviews_for_review_status(root: &Path) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let content = match fs::read_to_string(root.join("docs/mtg-engine-milestone-reviews.md")) {
        Ok(c) => c,
        Err(_) => return map,
    };

    for line in content.lines() {
        // Lines like: `- [M0: ...](#...) **(RE-REVIEWED)**`
        if !line.starts_with("- [M") {
            continue;
        }
        // Extract milestone ID: between "[M" and ":"
        if let Some(start) = line.find("[M") {
            let rest = &line[start + 1..]; // "M0: ..."
            let id: String = rest.chars().take_while(|c| *c != ':').collect();
            let status = if line.contains("RE-REVIEWED") {
                "RE-REVIEWED"
            } else if line.contains("REVIEWED") {
                "REVIEWED"
            } else {
                ""
            };
            map.insert(id, status.to_string());
        }
    }

    map
}

// ─── roadmap.md ────────────────────────────────────────────────────────────

fn parse_roadmap(
    root: &Path,
    review_status: &std::collections::HashMap<String, String>,
) -> anyhow::Result<Vec<MilestoneStatus>> {
    let content = fs::read_to_string(root.join("docs/mtg-engine-roadmap.md"))?;

    // Parse the active milestone from CLAUDE.md (simple re-parse here).
    let active_id = parse_claude_md(root)
        .map(|s| s.active_milestone)
        .unwrap_or_default();

    let mut milestones: Vec<MilestoneStatus> = vec![];
    let mut current: Option<MilestoneStatus> = None;
    let mut in_deliverables = false;

    for line in content.lines() {
        // Milestone header: "### M0: Project Scaffold..."
        if line.starts_with("### M") {
            // Save previous.
            if let Some(m) = current.take() {
                milestones.push(m);
            }
            // Parse "### M0: Name" or "### M9.5: Name"
            let rest = line.trim_start_matches("### ");
            let (id, name) = if let Some(colon) = rest.find(':') {
                let id = rest[..colon].trim().to_string();
                let name = rest[colon + 1..].trim().to_string();
                (id, name)
            } else {
                (rest.to_string(), String::new())
            };

            let review = review_status.get(&id).cloned().unwrap_or_default();
            let is_active = id == active_id;

            current = Some(MilestoneStatus {
                id,
                name,
                total_deliverables: 0,
                completed_deliverables: 0,
                is_active,
                review_status: review,
            });
            in_deliverables = false;
            continue;
        }

        if line.starts_with("**Deliverables**") {
            in_deliverables = true;
            continue;
        }

        if line.starts_with("**Acceptance")
            || line.starts_with("**Dependencies")
            || line.starts_with("**Tests")
            || line.starts_with("**Goal")
            || (line.starts_with("**") && in_deliverables)
        {
            in_deliverables = false;
            continue;
        }

        if !in_deliverables {
            continue;
        }

        if let Some(m) = current.as_mut() {
            if line.starts_with("- [x]") {
                m.total_deliverables += 1;
                m.completed_deliverables += 1;
            } else if line.starts_with("- [ ]") {
                m.total_deliverables += 1;
            }
        }
    }

    if let Some(m) = current.take() {
        milestones.push(m);
    }

    Ok(milestones)
}

// ─── scripts directory ─────────────────────────────────────────────────────

fn count_scripts(root: &Path) -> anyhow::Result<ScriptCounts> {
    let scripts_dir = root.join("test-data/generated-scripts");
    let mut counts = ScriptCounts::default();

    if !scripts_dir.exists() {
        return Ok(counts);
    }

    let mut by_dir: Vec<(String, u32)> = vec![];
    let mut entries: Vec<super::data::ScriptEntry> = vec![];

    for entry in fs::read_dir(&scripts_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        let mut dir_count = 0u32;

        let mut dir_scripts: Vec<super::data::ScriptEntry> = vec![];
        for script in fs::read_dir(entry.path())? {
            let script = script?;
            let path = script.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            dir_count += 1;
            if let Some(se) = parse_script_entry(&path, &dir_name) {
                dir_scripts.push(se);
            }
        }
        // Sort each directory's scripts alphabetically by filename
        dir_scripts.sort_by(|a, b| a.filename.cmp(&b.filename));
        counts.total += dir_count;
        by_dir.push((dir_name, dir_count));
        entries.extend(dir_scripts);
    }

    by_dir.sort_by(|a, b| b.1.cmp(&a.1)); // sort descending by count
    counts.by_directory = by_dir;

    // Sort entries: pending_review first, then by dir+filename
    entries.sort_by(|a, b| {
        let a_pending = a.status == "pending_review";
        let b_pending = b.status == "pending_review";
        b_pending
            .cmp(&a_pending)
            .then(a.directory.cmp(&b.directory))
            .then(a.filename.cmp(&b.filename))
    });

    counts.approved = entries.iter().filter(|e| e.status == "approved").count() as u32;
    counts.pending_review = entries
        .iter()
        .filter(|e| e.status == "pending_review")
        .count() as u32;
    counts.entries = entries;

    Ok(counts)
}

fn parse_script_entry(path: &Path, dir: &str) -> Option<super::data::ScriptEntry> {
    let content = fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let metadata = json.get("metadata")?;
    let id = metadata
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let name = metadata
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let status = metadata
        .get("review_status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let filename = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    let assertion_count = count_assert_states(&json);

    Some(super::data::ScriptEntry {
        id,
        name,
        directory: dir.to_string(),
        filename,
        status,
        assertion_count,
    })
}

fn count_assert_states(json: &serde_json::Value) -> u32 {
    let script = match json.get("script").and_then(|s| s.as_array()) {
        Some(s) => s,
        None => return 0,
    };
    let mut count = 0u32;
    for step in script {
        if let Some(actions) = step.get("actions").and_then(|a| a.as_array()) {
            for action in actions {
                if action.get("type").and_then(|t| t.as_str()) == Some("assert_state") {
                    count += 1;
                }
            }
        }
    }
    count
}

// ─── card authoring worklist ────────────────────────────────────────────────

fn parse_card_worklist(root: &Path) -> anyhow::Result<CardWorklist> {
    let path = root.join("test-data/test-decks/_authoring_worklist.json");
    let content = fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let summary = json.get("summary").unwrap_or(&serde_json::Value::Null);
    let mut wl = CardWorklist {
        total: summary
            .get("total_cards")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        authored: summary
            .get("authored")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        ready: summary.get("ready").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        blocked: summary.get("blocked").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        deferred: summary
            .get("deferred")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        unknown: summary.get("unknown").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        entries: vec![],
        card_dsl: parse_card_dsl(root),
    };

    // Parse each category
    for (status_key, status_label) in &[
        ("authored", "authored"),
        ("ready", "ready"),
        ("blocked", "blocked"),
        ("deferred", "deferred"),
    ] {
        if let Some(arr) = json.get(*status_key).and_then(|v| v.as_array()) {
            for item in arr {
                wl.entries.push(parse_card_entry(item, status_label));
            }
        }
    }

    // Sort by appears_in_decks descending, then name ascending
    wl.entries.sort_by(|a, b| {
        b.appears_in_decks
            .cmp(&a.appears_in_decks)
            .then(a.name.cmp(&b.name))
    });

    Ok(wl)
}

/// Extract `CardDefinition { ... }` blocks from per-card files in `defs/`, keyed by card name.
fn parse_card_dsl(root: &Path) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let defs_dir = root.join("crates/engine/src/cards/defs");

    let entries = match fs::read_dir(&defs_dir) {
        Ok(e) => e,
        Err(_) => return map,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_stem().and_then(|s| s.to_str()) == Some("mod") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        extract_card_dsl_from_file(&content, &mut map);
    }

    map
}

/// Extract `CardDefinition { ... }` block from a single card file.
fn extract_card_dsl_from_file(content: &str, map: &mut std::collections::HashMap<String, String>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.contains("CardDefinition {") || trimmed.contains("CardDefinition{") {
            let start = i;
            let mut depth: i32 = 0;
            let mut end = i;
            for (j, line) in lines.iter().enumerate().skip(start) {
                for ch in line.chars() {
                    match ch {
                        '{' => depth += 1,
                        '}' => depth -= 1,
                        _ => {}
                    }
                }
                if depth <= 0 {
                    end = j;
                    break;
                }
            }
            let block: String = lines[start..=end].join("\n");
            let dedented = dedent_block(&block);
            if let Some(name) = extract_card_name(&block) {
                map.insert(name, dedented);
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
}

/// Strip common leading whitespace from all lines in a block.
fn dedent_block(block: &str) -> String {
    let min_indent = block
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    if min_indent == 0 {
        return block.to_string();
    }
    block
        .lines()
        .map(|l| {
            if l.len() >= min_indent {
                &l[min_indent..]
            } else {
                l.trim()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract the card name from a `name: "..."` line inside a CardDefinition block.
fn extract_card_name(block: &str) -> Option<String> {
    for line in block.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("name: \"") {
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

/// Count lines of `.rs` files under a directory, recursively.
fn count_lines_recursive(dir: &Path) -> u32 {
    let mut total = 0u32;
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            total += count_lines_recursive(&path);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(&path) {
                total += content.lines().count() as u32;
            }
        }
    }
    total
}

/// Compute engine source and test LOC by walking the filesystem.
/// Returns (source_loc, test_loc).
fn compute_engine_loc(root: &Path) -> (u32, u32) {
    let src_loc = count_lines_recursive(&root.join("crates/engine/src"));
    let test_loc = count_lines_recursive(&root.join("crates/engine/tests"));
    (src_loc, test_loc)
}

fn parse_card_entry(item: &serde_json::Value, status: &str) -> CardWorklistEntry {
    let name = item
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let appears_in_decks = item
        .get("appears_in_decks")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let types: Vec<String> = item
        .get("types")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let keywords: Vec<String> = item
        .get("keywords")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let blocking_keywords: Vec<String> = item
        .get("blocking_keywords")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let keyword_statuses: Vec<(String, String)> = item
        .get("keyword_statuses")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect()
        })
        .unwrap_or_default();

    CardWorklistEntry {
        name,
        appears_in_decks,
        types,
        keywords,
        status: status.to_string(),
        blocking_keywords,
        keyword_statuses,
    }
}

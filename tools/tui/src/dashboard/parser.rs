use std::{
    fs,
    path::Path,
};

use super::data::*;

// ─── public entry point ────────────────────────────────────────────────────

pub fn parse_all(root: &Path) -> DashboardData {
    DashboardData {
        current_state: parse_claude_md(root).unwrap_or_default(),
        abilities: parse_ability_coverage(root).unwrap_or_default(),
        milestones: parse_roadmap(root, &parse_reviews_for_review_status(root)).unwrap_or_default(),
        corner_cases: parse_corner_case_audit(root).unwrap_or_default(),
        reviews: parse_milestone_reviews(root).unwrap_or_default(),
        scripts: count_scripts(root).unwrap_or_default(),
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
                state.active_milestone = after
                    .split_whitespace()
                    .next()
                    .unwrap_or(rest)
                    .to_string();
            } else {
                // Just take the first token.
                state.active_milestone = rest
                    .split_whitespace()
                    .next()
                    .unwrap_or(rest)
                    .to_string();
            }
        } else if let Some(rest) = line.strip_prefix("- **Status**: ") {
            state.status_line = rest.to_string();
            for part in rest.split(';') {
                let part = part.trim();
                if part.contains("tests passing") {
                    state.test_count = first_number(part);
                } else if part.contains("approved") && !part.contains("pending_review") {
                    // "71 approved" or "71 approved + 3 pending_review scripts"
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
    let content = fs::read_to_string(
        root.join("docs/mtg-engine-ability-coverage.md"),
    )?;
    let mut coverage = AbilityCoverage::default();
    let mut mode = ParseMode::None;
    let mut current_section: Option<AbilitySection> = None;

    enum ParseMode {
        None,
        Summary,
        Section,
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
            let name = line
                .trim_start_matches("## ")
                .to_string();
            current_section = Some(AbilitySection { name, rows: vec![] });
            mode = ParseMode::Section;
            continue;
        }

        if line.starts_with("## ") && !line.starts_with("## Section ") && !line.starts_with("## Summary") {
            // Different top-level section — stop section parsing.
            if let Some(sec) = current_section.take() {
                coverage.sections.push(sec);
            }
            mode = ParseMode::None;
            continue;
        }

        match mode {
            ParseMode::None => {}
            ParseMode::Summary => {
                if !line.starts_with('|') { continue; }
                if is_separator(line) { continue; }
                let cells = table_cells(line);
                if cells.len() < 7 { continue; }
                // Header row: "Priority", skip
                if cells[0].to_lowercase() == "priority" { continue; }
                // Total row: starts with "Total"
                if cells[0].to_lowercase().contains("total") { continue; }
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
                if !line.starts_with('|') { continue; }
                if is_separator(line) { continue; }
                let cells = table_cells(line);
                if cells.len() < 2 { continue; }
                // Header row: first cell is "Ability", "Pattern", etc.
                let first_lower = cells[0].to_lowercase();
                if first_lower == "ability" || first_lower == "pattern" { continue; }
                let row = AbilityRow {
                    name: cells.first().cloned().unwrap_or_default(),
                    cr: cells.get(1).cloned().unwrap_or_default(),
                    priority: cells.get(2).cloned().unwrap_or_default(),
                    status: cells.get(3).cloned().unwrap_or_default(),
                    engine_files: cells.get(4).cloned().unwrap_or_default(),
                    card_def: cells.get(5).cloned().unwrap_or_default(),
                    script: cells.get(6).cloned().unwrap_or_default(),
                    notes: cells.get(8).cloned().unwrap_or_default(),
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
    let content = fs::read_to_string(
        root.join("docs/mtg-engine-corner-case-audit.md"),
    )?;
    let mut audit = CornerCaseAudit::default();

    enum ParseMode { None, Summary, Table }
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
                if !line.starts_with('|') { continue; }
                if is_separator(line) { continue; }
                let cells = table_cells(line);
                if cells.len() < 2 { continue; }
                if cells[0].to_lowercase() == "status" { continue; }
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
                if !line.starts_with('|') { continue; }
                if is_separator(line) { continue; }
                let cells = table_cells(line);
                if cells.len() < 4 { continue; }
                // Header: "#", skip
                if cells[0] == "#" { continue; }
                let number: u32 = cells[0].parse().unwrap_or(0);
                if number == 0 { continue; }

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
    let content = fs::read_to_string(
        root.join("docs/mtg-engine-milestone-reviews.md"),
    )?;
    let mut stats = ReviewStatistics::default();
    let mut in_stats = false;

    for line in content.lines() {
        if line.starts_with("## Statistics") {
            in_stats = true;
            continue;
        }
        if !in_stats { continue; }
        if line.starts_with("## ") { break; }

        // Parse LOC lines like "**Engine source LOC (M0-M9.4)**: ~17,800 lines"
        if line.starts_with("**Engine source LOC") {
            if let Some(idx) = line.find("~") {
                let num_str = &line[idx + 1..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .replace(',', "");
                stats.engine_loc = num_str.parse().unwrap_or(0);
            }
            continue;
        }
        if line.starts_with("**Engine test LOC") {
            if let Some(idx) = line.find("~") {
                let num_str = &line[idx + 1..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .replace(',', "");
                stats.test_loc = num_str.parse().unwrap_or(0);
            }
            continue;
        }

        // Table rows
        if !line.starts_with('|') { continue; }
        if is_separator(line) { continue; }
        let cells = table_cells(line);
        if cells.len() < 2 { continue; }
        if cells[0].to_lowercase() == "metric" { continue; }

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
        if !line.starts_with("- [M") { continue; }
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

        if line.starts_with("**Acceptance") || line.starts_with("**Dependencies")
            || line.starts_with("**Tests") || line.starts_with("**Goal")
            || (line.starts_with("**") && in_deliverables)
        {
            in_deliverables = false;
            continue;
        }

        if !in_deliverables { continue; }

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

    for entry in fs::read_dir(&scripts_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() { continue; }
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        let mut dir_count = 0u32;
        for script in fs::read_dir(entry.path())? {
            let script = script?;
            if script.path().extension().and_then(|e| e.to_str()) == Some("json") {
                dir_count += 1;
            }
        }
        counts.total += dir_count;
        by_dir.push((dir_name, dir_count));
    }

    by_dir.sort_by(|a, b| b.1.cmp(&a.1)); // sort descending by count
    counts.by_directory = by_dir;

    Ok(counts)
}

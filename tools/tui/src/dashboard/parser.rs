use std::{fs, path::Path};

use super::data::*;

// ─── public entry point ────────────────────────────────────────────────────

pub fn parse_all(root: &Path) -> DashboardData {
    let mut reviews = parse_milestone_reviews(root).unwrap_or_default();
    let (src_loc, test_loc) = compute_engine_loc(root);
    reviews.engine_loc = src_loc;
    reviews.test_loc = test_loc;

    let mut progress = parse_project_status(root).unwrap_or_default();
    progress.card_health = scan_card_health_live(root);

    let (live_cards, card_dsl) = scan_live_cards(root);

    DashboardData {
        current_state: parse_claude_md(root).unwrap_or_default(),
        abilities: parse_ability_coverage(root).unwrap_or_default(),
        milestones: parse_roadmap(root, &parse_reviews_for_review_status(root)).unwrap_or_default(),
        corner_cases: parse_corner_case_audit(root).unwrap_or_default(),
        reviews,
        scripts: count_scripts(root).unwrap_or_default(),
        progress,
        live_cards,
        card_dsl,
        worker_status: parse_worker_status(root),
    }
}

// ─── helpers ────────────────────────────────────────────────────────────────

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

fn clean_cell(s: &str) -> String {
    s.trim().replace("**", "").replace('`', "")
}

fn is_separator(line: &str) -> bool {
    line.starts_with('|') && line.contains("---")
}

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
            if let Some(idx) = rest.find("advancing to ") {
                let after = &rest[idx + "advancing to ".len()..];
                state.active_milestone =
                    after.split_whitespace().next().unwrap_or(rest).to_string();
            } else {
                state.active_milestone = rest.split_whitespace().next().unwrap_or(rest).to_string();
            }
        } else if let Some(rest) = line.strip_prefix("- **Status**: ") {
            for part in rest.split(';') {
                let part = part.trim();
                if part.contains("tests passing") {
                    state.test_count = first_number(part);
                } else if part.contains("approved") {
                    state.script_count = first_number(part);
                }
            }
        } else if let Some(rest) = line.strip_prefix("- **Last Updated**: ") {
            state.last_updated = rest.to_string();
        }
    }

    Ok(state)
}

// ─── ability-coverage.md (summary only) ─────────────────────────────────────

fn parse_ability_coverage(root: &Path) -> anyhow::Result<AbilityCoverage> {
    let content = fs::read_to_string(root.join("docs/mtg-engine-ability-coverage.md"))?;
    let mut coverage = AbilityCoverage::default();
    let mut in_summary = false;

    for line in content.lines() {
        if line.starts_with("## Summary") {
            in_summary = true;
            continue;
        }
        if in_summary && line.starts_with("## ") {
            break;
        }
        if !in_summary || !line.starts_with('|') || is_separator(line) {
            continue;
        }
        let cells = table_cells(line);
        if cells.len() < 3 {
            continue;
        }
        if cells[0].to_lowercase() == "priority" || cells[0].to_lowercase().contains("total") {
            continue;
        }
        coverage.summary.push(PrioritySummary {
            priority: cells[0].clone(),
            total: cells[1].parse().unwrap_or(0),
            validated: cells[2].parse().unwrap_or(0),
        });
    }

    Ok(coverage)
}

// ─── corner-case-audit.md (summary only) ────────────────────────────────────

fn parse_corner_case_audit(root: &Path) -> anyhow::Result<CornerCaseAudit> {
    let content = fs::read_to_string(root.join("docs/mtg-engine-corner-case-audit.md"))?;
    let mut audit = CornerCaseAudit::default();
    let mut in_summary = false;

    for line in content.lines() {
        if line.starts_with("## Summary") {
            in_summary = true;
            continue;
        }
        if in_summary && line.starts_with("## ") {
            break;
        }
        if !in_summary || !line.starts_with('|') || is_separator(line) {
            continue;
        }
        let cells = table_cells(line);
        if cells.len() < 2 || cells[0].to_lowercase() == "status" {
            continue;
        }
        let count: u32 = cells[1].parse().unwrap_or(0);
        match cells[0].to_lowercase().as_str() {
            "covered" => audit.covered = count,
            "gap" => audit.gap = count,
            _ => {}
        }
    }
    audit.total = audit.covered + audit.gap;
    // Also add partial/deferred to total
    Ok(audit)
}

// ─── milestone-reviews.md ───────────────────────────────────────────────────

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
        if !line.starts_with('|') || is_separator(line) {
            continue;
        }
        let cells = table_cells(line);
        if cells.len() < 2 || cells[0].to_lowercase() == "metric" {
            continue;
        }
        let value = &cells[1];
        match cells[0].as_str() {
            "HIGH (OPEN)" => stats.high_open = first_number(value),
            "HIGH (CLOSED)" => stats.high_closed = first_number(value),
            "MEDIUM (OPEN)" => stats.medium_open = first_number(value),
            "MEDIUM (CLOSED)" => stats.medium_closed = first_number(value),
            "LOW (OPEN)" => stats.low_open = first_number(value),
            "LOW (CLOSED)" => stats.low_closed = first_number(value),
            _ => {}
        }
    }

    Ok(stats)
}

// ─── milestone-reviews.md (review status per milestone) ─────────────────────

fn parse_reviews_for_review_status(root: &Path) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let content = match fs::read_to_string(root.join("docs/mtg-engine-milestone-reviews.md")) {
        Ok(c) => c,
        Err(_) => return map,
    };

    for line in content.lines() {
        if !line.starts_with("- [M") {
            continue;
        }
        if let Some(start) = line.find("[M") {
            let rest = &line[start + 1..];
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

// ─── roadmap.md ─────────────────────────────────────────────────────────────

fn parse_roadmap(
    root: &Path,
    review_status: &std::collections::HashMap<String, String>,
) -> anyhow::Result<Vec<MilestoneStatus>> {
    let content = fs::read_to_string(root.join("docs/mtg-engine-roadmap.md"))?;

    let active_id = parse_claude_md(root)
        .map(|s| s.active_milestone)
        .unwrap_or_default();

    let mut milestones: Vec<MilestoneStatus> = vec![];
    let mut current: Option<MilestoneStatus> = None;
    let mut in_deliverables = false;

    for line in content.lines() {
        if line.starts_with("### M") {
            if let Some(m) = current.take() {
                milestones.push(m);
            }
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
                is_future: false,
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

    // Mark future milestones (incomplete and not active)
    for m in &mut milestones {
        m.is_future = m.completion_pct() < 1.0 && !m.is_active;
    }

    Ok(milestones)
}

// ─── scripts directory (counts only) ────────────────────────────────────────

fn count_scripts(root: &Path) -> anyhow::Result<ScriptCounts> {
    let scripts_dir = root.join("test-data/generated-scripts");
    let mut counts = ScriptCounts::default();

    if !scripts_dir.exists() {
        return Ok(counts);
    }

    for entry in fs::read_dir(&scripts_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        for script in fs::read_dir(entry.path())? {
            let script = script?;
            let path = script.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            counts.total += 1;
            if let Ok(content) = fs::read_to_string(&path) {
                if content.contains("\"approved\"") {
                    counts.approved += 1;
                }
            }
        }
    }

    Ok(counts)
}

// ─── project-status.md parser ───────────────────────────────────────────────

fn parse_project_status(root: &Path) -> Option<ProjectProgress> {
    let path = root.join("docs/project-status.md");
    let content = fs::read_to_string(path).ok()?;
    let mut progress = ProjectProgress::default();

    let mut section = "";
    let mut in_table = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## Primitive Batches") {
            section = "batches";
            in_table = false;
            continue;
        } else if trimmed.starts_with("## Card Health") {
            section = "health";
            in_table = false;
            continue;
        } else if trimmed.starts_with("## Workstreams") {
            section = "workstreams";
            in_table = false;
            continue;
        } else if trimmed.starts_with("## Path to Alpha") {
            section = "alpha";
            in_table = false;
            continue;
        } else if trimmed.starts_with("## ") {
            section = "";
            in_table = false;
            continue;
        }

        if trimmed.starts_with("|---") || trimmed.starts_with("| ---") {
            in_table = true;
            continue;
        }
        if !trimmed.starts_with('|') {
            continue;
        }
        if !in_table {
            continue;
        }

        let cells = table_cells(trimmed);

        match section {
            "batches" if cells.len() >= 6 => {
                progress.primitive_batches.push(PrimitiveBatch {
                    batch: cells[0].clone(),
                    title: cells[1].clone(),
                    status: cells[2].clone(),
                    cards_fixed: cells[3].parse().unwrap_or(0),
                    cards_remaining: cells[4].parse().unwrap_or(0),
                    review: cells[5].clone(),
                });
            }
            "health" if cells.len() >= 2 => {
                let val: u32 = cells[1].parse().unwrap_or(0);
                let cat = cells[0].to_lowercase();
                if cat.contains("not yet") {
                    progress.card_health.not_authored = val;
                } else if cat.contains("total universe") {
                    progress.card_health.total_universe = val;
                } else if cat.contains("total authored") {
                    progress.card_health.total_authored = val;
                }
            }
            "workstreams" if cells.len() >= 3 => {
                progress.workstreams.push(WorkstreamEntry {
                    number: cells[0].clone(),
                    name: cells[1].clone(),
                    status: cells[2].clone(),
                });
            }
            "alpha" if cells.len() >= 4 => {
                progress.path_to_alpha.push(AlphaMilestone {
                    name: cells[0].clone(),
                    status: cells[1].clone(),
                    blocked_by: cells[2].clone(),
                    deliverable: cells[3].clone(),
                });
            }
            _ => {}
        }
    }

    Some(progress)
}

// ─── live card scanner ──────────────────────────────────────────────────────

fn scan_card_health_live(root: &Path) -> CardHealth {
    let defs_dir = root.join("crates/engine/src/cards/defs");
    let entries = match fs::read_dir(&defs_dir) {
        Ok(e) => e,
        Err(_) => return CardHealth::default(),
    };

    let mut h = CardHealth::default();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_stem().and_then(|s| s.to_str()) == Some("mod") {
            continue;
        }

        h.total_authored += 1;

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let has_todo = content.contains("TODO");
        let has_empty_abilities = content.contains("abilities: vec![]");

        match (has_todo, has_empty_abilities) {
            (false, false) => h.fully_implemented += 1,
            (false, true) => h.vanilla += 1,
            (true, true) => h.stripped += 1,
            (true, false) => h.partial += 1,
        }
    }

    h.has_todos = h.partial + h.stripped;
    h.total_universe = 1743;
    h.not_authored = h.total_universe.saturating_sub(h.total_authored);
    h
}

/// Scan all card def files, return per-card entries + DSL source map.
fn scan_live_cards(
    root: &Path,
) -> (
    Vec<LiveCardEntry>,
    std::collections::HashMap<String, String>,
) {
    let defs_dir = root.join("crates/engine/src/cards/defs");
    let entries = match fs::read_dir(&defs_dir) {
        Ok(e) => e,
        Err(_) => return (vec![], std::collections::HashMap::new()),
    };

    let mut cards: Vec<LiveCardEntry> = vec![];
    let mut dsl_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let file_stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some("mod") => continue,
            Some(s) => s.to_string(),
            None => continue,
        };

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let has_todo = content.contains("TODO");
        let has_empty_abilities = content.contains("abilities: vec![]");

        let status = match (has_todo, has_empty_abilities) {
            (false, false) => "ok",
            (false, true) => "vanilla",
            (true, true) => "stripped",
            (true, false) => "partial",
        };

        // Extract card name from file content
        let card_name =
            extract_card_name_from_content(&content).unwrap_or_else(|| title_case(&file_stem));

        // Extract TODO lines
        let todo_lines: Vec<String> = content
            .lines()
            .filter(|l| l.contains("TODO"))
            .map(|l| l.trim().to_string())
            .collect();

        // Extract DSL for detail view
        extract_card_dsl_from_file(&content, &card_name, &mut dsl_map);

        cards.push(LiveCardEntry {
            name: card_name,
            file_name: file_stem,
            status: status.to_string(),
            todo_lines,
        });
    }

    // Sort: todo cards first (partial, stripped), then ok, then vanilla. Within each group, by name.
    cards.sort_by(|a, b| {
        let order = |s: &str| -> u8 {
            match s {
                "stripped" => 0,
                "partial" => 1,
                "ok" => 2,
                "vanilla" => 3,
                _ => 4,
            }
        };
        order(&a.status)
            .cmp(&order(&b.status))
            .then(a.name.cmp(&b.name))
    });

    (cards, dsl_map)
}

fn extract_card_name_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("name: \"") {
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

fn title_case(slug: &str) -> String {
    slug.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_card_dsl_from_file(
    content: &str,
    card_name: &str,
    map: &mut std::collections::HashMap<String, String>,
) {
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
            map.insert(card_name.to_string(), dedented);
            i = end + 1;
        } else {
            i += 1;
        }
    }
}

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

// ─── engine LOC ─────────────────────────────────────────────────────────────

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

fn compute_engine_loc(root: &Path) -> (u32, u32) {
    let src_loc = count_lines_recursive(&root.join("crates/engine/src"));
    let test_loc = count_lines_recursive(&root.join("crates/engine/tests"));
    (src_loc, test_loc)
}

// ─── worker status (primitive-wip.md) ───────────────────────────────────────

fn parse_worker_status(root: &Path) -> Option<WorkerStatus> {
    let path = root.join("memory/primitive-wip.md");
    let content = fs::read_to_string(path).ok()?;
    let mut ws = WorkerStatus::default();

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("batch: ") {
            ws.batch = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("title: ") {
            ws.title = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("phase: ") {
            ws.phase = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("started: ") {
            ws.started = rest.trim().to_string();
        }
    }

    if ws.batch.is_empty() {
        None
    } else {
        Some(ws)
    }
}

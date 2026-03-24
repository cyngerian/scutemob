/// All parsed data used by the dashboard.
#[derive(Debug, Default)]
pub struct DashboardData {
    pub current_state: CurrentState,
    pub abilities: AbilityCoverage,
    pub milestones: Vec<MilestoneStatus>,
    pub corner_cases: CornerCaseAudit,
    pub reviews: ReviewStatistics,
    pub scripts: ScriptCounts,
    pub progress: ProjectProgress,
    pub live_cards: Vec<LiveCardEntry>,
    pub card_dsl: std::collections::HashMap<String, String>,
    pub worker_status: Option<WorkerStatus>,
}

/// Parsed from `docs/project-status.md`.
#[derive(Debug, Default)]
pub struct ProjectProgress {
    pub primitive_batches: Vec<PrimitiveBatch>,
    pub card_health: CardHealth,
    pub workstreams: Vec<WorkstreamEntry>,
    pub path_to_alpha: Vec<AlphaMilestone>,
}

#[derive(Debug, Default, Clone)]
pub struct PrimitiveBatch {
    pub batch: String,
    pub title: String,
    /// "done", "active", "planned"
    pub status: String,
    pub cards_fixed: u32,
    pub cards_remaining: u32,
    /// "clean", "fixed", "none", "—"
    pub review: String,
}

#[derive(Debug, Default)]
pub struct CardHealth {
    pub fully_implemented: u32,
    pub vanilla: u32,
    pub partial: u32,
    pub stripped: u32,
    pub has_todos: u32,
    pub not_authored: u32,
    pub total_universe: u32,
    pub total_authored: u32,
}

#[derive(Debug, Default, Clone)]
pub struct WorkstreamEntry {
    pub number: String,
    pub name: String,
    /// "done", "active", "stalled", "partial", "not-started", "retired"
    pub status: String,
}

#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct AlphaMilestone {
    pub name: String,
    pub status: String,
    pub blocked_by: String,
    pub deliverable: String,
}

/// Parsed from CLAUDE.md `## Current State` section.
#[derive(Debug, Default)]
pub struct CurrentState {
    pub active_milestone: String,
    pub test_count: u32,
    pub script_count: u32,
    pub last_updated: String,
}

/// Parsed from `docs/mtg-engine-ability-coverage.md` (summary only).
#[derive(Debug, Default)]
pub struct AbilityCoverage {
    pub summary: Vec<PrioritySummary>,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct PrioritySummary {
    pub priority: String,
    pub total: u32,
    pub validated: u32,
}

/// Parsed from `docs/mtg-engine-roadmap.md`.
#[derive(Debug, Default)]
pub struct MilestoneStatus {
    pub id: String,
    pub name: String,
    pub total_deliverables: u32,
    pub completed_deliverables: u32,
    pub is_active: bool,
    pub review_status: String,
    /// true if all deliverables complete (M0-M9.5); false for future milestones
    pub is_future: bool,
}

impl MilestoneStatus {
    pub fn completion_pct(&self) -> f64 {
        if self.total_deliverables == 0 {
            0.0
        } else {
            self.completed_deliverables as f64 / self.total_deliverables as f64
        }
    }
}

/// Parsed from `docs/mtg-engine-corner-case-audit.md` (summary only).
#[derive(Debug, Default)]
pub struct CornerCaseAudit {
    pub covered: u32,
    pub gap: u32,
    pub total: u32,
}

/// Parsed from `docs/mtg-engine-milestone-reviews.md` Statistics section.
#[derive(Debug, Default)]
pub struct ReviewStatistics {
    pub high_open: u32,
    pub high_closed: u32,
    pub medium_open: u32,
    pub medium_closed: u32,
    pub low_open: u32,
    pub low_closed: u32,
    pub engine_loc: u32,
    pub test_loc: u32,
}

/// Script counts (totals only).
#[derive(Debug, Default)]
pub struct ScriptCounts {
    pub total: u32,
    pub approved: u32,
}

/// A card definition file scanned from the filesystem.
#[derive(Debug, Default, Clone)]
pub struct LiveCardEntry {
    pub name: String,
    pub file_name: String,
    /// "ok", "partial", "stripped", "vanilla"
    pub status: String,
    /// TODO comment lines extracted from the file
    pub todo_lines: Vec<String>,
}

/// Parsed from `memory/primitive-wip.md`.
#[derive(Debug, Default, Clone)]
pub struct WorkerStatus {
    pub batch: String,
    pub title: String,
    pub phase: String,
    pub started: String,
}

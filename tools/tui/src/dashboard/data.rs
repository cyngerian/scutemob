/// All parsed data used by the dashboard.
#[derive(Debug, Default)]
pub struct DashboardData {
    pub current_state: CurrentState,
    pub abilities: AbilityCoverage,
    pub milestones: Vec<MilestoneStatus>,
    pub corner_cases: CornerCaseAudit,
    pub reviews: ReviewStatistics,
    pub scripts: ScriptCounts,
    pub cards: CardWorklist,
    pub progress: ProjectProgress,
}

/// Parsed from `docs/project-status.md`.
#[derive(Debug, Default)]
pub struct ProjectProgress {
    pub primitive_batches: Vec<PrimitiveBatch>,
    pub card_health: CardHealth,
    pub workstreams: Vec<WorkstreamEntry>,
    pub path_to_alpha: Vec<AlphaMilestone>,
    pub deferred_items: Vec<DeferredItem>,
    pub review_backlog: Vec<ReviewBacklogEntry>,
    pub review_progress_done: u32,
    pub review_progress_total: u32,
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
    pub complete: u32,
    pub has_todos: u32,
    pub wrong_state: u32,
    pub not_authored: u32,
    pub total_universe: u32,
    pub total_authored: u32,
    /// Cards with no TODO and non-empty abilities (fully implemented).
    pub fully_implemented: u32,
    /// Cards with no TODO and empty abilities (vanilla/intentional).
    pub vanilla: u32,
    /// Cards with TODOs but some abilities (partial implementation).
    pub partial: u32,
    /// Cards with TODOs and empty abilities (stripped).
    pub stripped: u32,
}

#[derive(Debug, Default, Clone)]
pub struct WorkstreamEntry {
    pub number: String,
    pub name: String,
    /// "done", "active", "stalled", "partial", "not-started", "retired"
    pub status: String,
    #[allow(dead_code)]
    pub last_activity: String,
    #[allow(dead_code)]
    pub next_action: String,
}

#[derive(Debug, Default, Clone)]
pub struct AlphaMilestone {
    pub name: String,
    pub status: String,
    #[allow(dead_code)]
    pub blocked_by: String,
    #[allow(dead_code)]
    pub deliverable: String,
}

#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct DeferredItem {
    pub item: String,
    pub deferred_from: String,
    pub blocked_until: String,
    pub impact: String,
}

#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct ReviewBacklogEntry {
    pub number: u32,
    pub batch: String,
    pub title: String,
    pub cards_fixed: u32,
    /// "pending", "in-review", "needs-fix", "fixing", "clean", "fixed"
    pub review_status: String,
    pub findings: String,
}

/// Parsed from CLAUDE.md `## Current State` section.
#[derive(Debug, Default)]
pub struct CurrentState {
    /// Short active milestone ID, e.g. "M10"
    pub active_milestone: String,
    /// Raw status line text
    pub status_line: String,
    pub test_count: u32,
    /// Approved script count
    pub script_count: u32,
    pub last_updated: String,
}

/// Parsed from `docs/mtg-engine-ability-coverage.md`.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct AbilityCoverage {
    pub summary: Vec<PrioritySummary>,
    pub sections: Vec<AbilitySection>,
    /// Open gap items from the `## Priority Gaps` section (non-resolved, non-empty).
    pub gap_notes: Vec<String>,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct PrioritySummary {
    pub priority: String, // "P1", "P2", etc.
    pub total: u32,
    pub validated: u32,
    pub complete: u32,
    pub partial: u32,
    pub none: u32,
    pub na: u32,
}

#[derive(Debug, Default)]
pub struct AbilitySection {
    pub name: String,
    pub rows: Vec<AbilityRow>,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct AbilityRow {
    pub name: String,
    pub cr: String,
    pub priority: String,
    pub status: String,
    pub engine_files: String,
    pub card_def: String,
    pub script: String,
    pub notes: String,
}

/// Parsed from `docs/mtg-engine-roadmap.md`.
#[derive(Debug, Default)]
pub struct MilestoneStatus {
    pub id: String, // "M0", "M1", "M9.5"
    pub name: String,
    pub total_deliverables: u32,
    pub completed_deliverables: u32,
    pub is_active: bool,
    /// "RE-REVIEWED", "REVIEWED", ""
    pub review_status: String,
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

/// Parsed from `docs/mtg-engine-corner-case-audit.md`.
#[derive(Debug, Default)]
pub struct CornerCaseAudit {
    pub covered: u32,
    pub partial: u32,
    pub gap: u32,
    pub deferred: u32,
    pub cases: Vec<CornerCase>,
}

impl CornerCaseAudit {
    pub fn total(&self) -> u32 {
        self.covered + self.partial + self.gap + self.deferred
    }
}

#[derive(Debug, Default)]
pub struct CornerCase {
    pub number: u32,
    pub name: String,
    pub cr_refs: String,
    /// "COVERED", "GAP", "DEFERRED", "PARTIAL"
    pub status: String,
    pub milestone: String,
}

/// Parsed from `docs/mtg-engine-milestone-reviews.md` Statistics section.
#[derive(Debug, Default)]
pub struct ReviewStatistics {
    pub total_issues: u32,
    pub high_open: u32,
    pub high_closed: u32,
    pub medium_open: u32,
    pub medium_closed: u32,
    pub low_open: u32,
    pub low_closed: u32,
    pub info: u32,
    pub milestones_reviewed: u32,
    pub engine_loc: u32,
    pub test_loc: u32,
}

/// One parsed game script entry.
#[derive(Debug, Default)]
pub struct ScriptEntry {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    /// Subdirectory name: "stack", "combat", "baseline", etc.
    pub directory: String,
    /// Filename without .json extension.
    pub filename: String,
    /// "approved" | "pending_review" | "unknown"
    pub status: String,
    /// Number of `assert_state` action blocks in the script.
    pub assertion_count: u32,
}

/// Parsed from `test-data/test-decks/_authoring_worklist.json`.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct CardWorklist {
    pub total: u32,
    pub authored: u32,
    pub ready: u32,
    pub blocked: u32,
    pub deferred: u32,
    pub unknown: u32,
    pub entries: Vec<CardWorklistEntry>,
    /// Raw Rust DSL source for each authored card, keyed by card name.
    pub card_dsl: std::collections::HashMap<String, String>,
}

#[derive(Debug, Default, Clone)]
pub struct CardWorklistEntry {
    pub name: String,
    pub appears_in_decks: u32,
    pub types: Vec<String>,
    pub keywords: Vec<String>,
    /// "ready", "blocked", or "deferred"
    pub status: String,
    /// For blocked cards: which keywords block them
    pub blocking_keywords: Vec<String>,
    /// keyword name → status string (e.g. "validated (P1)")
    pub keyword_statuses: Vec<(String, String)>,
}

/// Script counts by subdirectory, plus full entry list.
#[derive(Debug, Default)]
pub struct ScriptCounts {
    pub total: u32,
    pub approved: u32,
    pub pending_review: u32,
    pub by_directory: Vec<(String, u32)>,
    pub entries: Vec<ScriptEntry>,
}

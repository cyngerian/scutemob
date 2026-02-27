/// All parsed data used by the dashboard.
#[derive(Debug, Default)]
pub struct DashboardData {
    pub current_state: CurrentState,
    pub abilities: AbilityCoverage,
    pub milestones: Vec<MilestoneStatus>,
    pub corner_cases: CornerCaseAudit,
    pub reviews: ReviewStatistics,
    pub scripts: ScriptCounts,
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

/// Script counts by subdirectory.
#[derive(Debug, Default)]
pub struct ScriptCounts {
    pub total: u32,
    pub by_directory: Vec<(String, u32)>,
}

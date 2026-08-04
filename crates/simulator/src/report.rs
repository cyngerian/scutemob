//! Crash report serialization for fuzzer output.
//!
//! When the fuzzer finds an invariant violation, it captures the
//! full game state, command history, and violation details as a
//! JSON crash report for debugging.

use mtg_engine::{Command, PlayerId};
use serde::{Deserialize, Serialize};

use crate::invariants::InvariantViolation;

/// A crash report from a fuzzer game that hit an invariant violation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrashReport {
    pub seed: u64,
    pub player_count: usize,
    pub violation: InvariantViolation,
    pub command_history: Vec<Command>,
    pub turn_number: u32,
    pub total_commands: usize,
}

impl CrashReport {
    /// Write this crash report to a file as JSON.
    pub fn write_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}

/// Summary of a completed game (success or failure).
///
/// # `Default`, and why it matters (PB-DX32 Stage 1)
///
/// Constructed at **five** sites: `local_game.rs` (GameOver), `driver.rs` (start
/// failure and Halted), `bin/fuzzer.rs` (build failure) — and a fifth **outside this
/// crate**, `tools/play-server/src/main.rs`'s `#[cfg(test)]` module. PB-DX32 adds five
/// new fields to this struct over its four stages (`rejection_count`, `rejections`,
/// `waste`, `transient_violations`, `decision_coverage`); the out-of-crate site cannot
/// be taught about any of them without becoming a wire dependency on `crates/simulator`
/// internals that `tools/play-server` has no reason to know. `Default` closes that gap:
/// every field type here is `Default`-able, so the fifth site appends
/// `..Default::default()` once (`main.rs:3326`) and never needs to change again as this
/// struct grows. The two REAL construction sites (`local_game.rs`'s GameOver return and
/// `driver.rs`'s Halted arm) do **not** rely on `Default` — they go through
/// `LocalGame::result_snapshot`, which populates every field from live game state.
#[derive(Clone, Debug, Default)]
pub struct GameResult {
    pub seed: u64,
    pub winner: Option<PlayerId>,
    pub turn_count: u32,
    pub total_commands: usize,
    pub violations: Vec<InvariantViolation>,
    pub error: Option<GameDriverError>,
}

/// Errors that can occur during game execution (distinct from invariant violations).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameDriverError {
    /// The engine returned an error from process_command.
    EngineError(String),
    /// Game hit the max turn limit without ending.
    MaxTurnsReached(u32),
    /// No legal actions available for the acting player (stuck).
    NoLegalActions { player: PlayerId, turn: u32 },
    /// Infinite loop detected — same state hash repeated too many times.
    InfiniteLoop { turn: u32 },
}

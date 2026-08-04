//! Crash report serialization for fuzzer output.
//!
//! When the fuzzer finds an invariant violation, it captures the
//! full game state, command history, and violation details as a
//! JSON crash report for debugging.

use mtg_engine::{Command, PlayerId};
use serde::{Deserialize, Serialize};

use crate::invariants::InvariantViolation;
use crate::local_game::{RejectedCommand, WasteTally};

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
    /// SR-38 at run scale (PB-DX32 Stage 2, `OOS-SIM3-2`). Never gated or truncated —
    /// see [`crate::local_game::LocalGame::rejection_count`].
    pub rejection_count: u32,
    /// A bounded diagnosis sample of the rejections counted in `rejection_count` — see
    /// [`crate::local_game::LocalGame::rejections`] for the cap rule.
    pub rejections: Vec<RejectedCommand>,
    /// The SIM-5 tap/pool instrument, promoted (PB-DX32 Stage 3) — see
    /// [`crate::local_game::LocalGame::waste`].
    pub waste: WasteTally,
    /// The noise-floor split (PB-DX32 Stage 4, `OOS-SIM3-3` / `OOS-SIM3-4`):
    /// `no_orphaned_tokens` reports, reported but NOT counted toward `violations` and
    /// NOT halting `--stop-on-error` — see
    /// [`crate::local_game::LocalGame::transient_violations`].
    pub transient_violations: Vec<InvariantViolation>,
}

/// SR-38 at run scale. Measured at HEAD (2026-08-03) over 5 fuzz-shaped games,
/// 23,613 commands, 542 rejections = 22.953 per mille.
/// Pinned with headroom, NOT at zero: OOS-SIM5-3 (blocker refusals, the largest
/// family), OOS-SIM5-5 (modal per-mode target slices), OOS-SIM6-3 (auto-tap covers
/// CastSpell alone), OOS-CARDS2-4 (Aura offers refused by CR 303.4a) and
/// OOS-SIM4-2 are all open. Ratchet DOWNWARD as each closes; never raise it to
/// fit a measurement without naming the seed that justifies the rise.
pub const MAX_BOT_REJECTION_PER_MILLE: u32 = 30;

/// The SR-38 threshold for the Stage-2 TEST gate (T2.2), distinct from
/// [`MAX_BOT_REJECTION_PER_MILLE`] above (the fuzz BINARY's threshold, measured at
/// `--profile fuzz` over 200-turn games). Measured at Stage 0 (2026-08-03), on the
/// gate's OWN configuration — a `cargo test` DEBUG build, 3 seeds ([1, 2, 3]) x 25
/// turns x `RandomBot` x `build_fuzz_state`, `record_journal: false`: 2,767 commands,
/// 86 rejections = 31.081 per mille. Pinned at 40 (~30% headroom over 31.081). NOT
/// zero, for the same five open seeds `MAX_BOT_REJECTION_PER_MILLE` names above.
/// Ratchet DOWNWARD as each closes, and re-measure (do not guess) if the gate's own
/// seeds or turn cap ever change — this number is NOT interchangeable with
/// `MAX_BOT_REJECTION_PER_MILLE`, which is a different (200-turn, release-profile)
/// measurement of a different configuration.
pub const MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG: u32 = 40;

/// Measured at HEAD (2026-08-03), `RandomBot`, 5 fuzz-shaped games, 200 turns:
/// 1,986 wasted of 2,641 taps = 75%. Pinned at 85 with headroom.
/// **`RandomBot` picks `TapForMana` uniformly with no plan, so most of its taps are
/// wasted BY DESIGN OF THE BOT.** A value near 75 is ordinary behaviour and is not a
/// defect; a rise past 85 means the auto-tap or the atomic-sequence rollback
/// regressed. This can only be ratcheted toward zero by a PLANNING bot
/// (a successor to `OOS-SIM6-3`/`OOS-SIM2-1`), never by an engine fix.
pub const MAX_RANDOM_BOT_WASTED_TAP_PCT: u32 = 85;

/// The waste-ratio threshold for the Stage-3 TEST gate (T3.1), distinct from
/// [`MAX_RANDOM_BOT_WASTED_TAP_PCT`] above for the SAME reason
/// [`MAX_BOT_REJECTION_PER_MILLE_AT_GATE_CONFIG`] is distinct from
/// `MAX_BOT_REJECTION_PER_MILLE`: this is a DIFFERENT population. Measured (2026-08-03)
/// at T3.1's own configuration -- 3 seeds ([1, 2, 3]) x 25 turns x `RandomBot` x
/// `build_fuzz_state`, `record_journal: false` -- 87 of 97 taps wasted = 89.7%,
/// truncated to 89% (integer division, matching the assertion's own arithmetic).
/// **This is NOT the same number as the 75% (200-turn, `--profile fuzz`) measurement
/// behind `MAX_RANDOM_BOT_WASTED_TAP_PCT`, and the difference is real, not noise**: at
/// 25 turns a RandomBot's early taps have proportionally fewer high-value casts to
/// land on than a 200-turn game gets to accumulate, so the SHORTER game's waste ratio
/// is structurally higher. Pinned at 95 (a flat +6 percentage points over 89, not the
/// ~30% multiplicative headroom `_AT_GATE_CONFIG` per-mille constants use, since a
/// percentage is bounded at 100 and 89 x 1.3 would overshoot that ceiling
/// meaninglessly). Same caveat as `MAX_RANDOM_BOT_WASTED_TAP_PCT`: `RandomBot` wastes
/// taps BY DESIGN, so this can only be ratcheted down by a planning bot, never an
/// engine fix; re-measure (do not guess) if T3.1's seeds or turn cap ever change.
pub const MAX_RANDOM_BOT_WASTED_TAP_PCT_AT_GATE_CONFIG: u32 = 95;

/// Measured at Stage 0 (2026-08-03) on the SIM-5 A/B seeds (0/7/42, `HeuristicBot`, 25
/// turns, `sim5_bot_cast_discipline.rs`): `mana_pools_emptied` was 0, 1, 0 — max
/// observed = 1.
/// NOT zero: `OOS-SIM2-1` — the greedy solver leaves slack on casts that SUCCEED, so a
/// destroyed pool is not necessarily a wasted one. That seed is the reason this pin
/// exists at all; closing it is what lowers this to 0.
pub const MAX_HEURISTIC_POOLS_EMPTIED_PER_SEED: usize = 1;

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

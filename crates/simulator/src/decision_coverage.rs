//! Decision-point runtime coverage (PB-DX32 Stage 6, `OOS-SIM3-2`'s buildout half).
//!
//! `crates/engine/tests/core/decision_site_walk.rs`'s `ROWS` table names 22 places the
//! engine makes a choice a player should be making (CR-cited, one row per pattern). It
//! lives in an engine *integration-test module*, so this crate cannot import it
//! (`crates/simulator` cannot dev-depend on `crates/engine`'s test tree, and the engine
//! cannot dev-depend on the simulator — a cycle in spirit even where Cargo tolerates
//! dev-dep cycles). Inventing a parallel predicate table here would be the second copy
//! the plan's own design section names as the failure mode this module must not become.
//!
//! **What this module IS**: an id roster, split into two lists, and a fold hook. It
//! carries no predicate and no CR citation of its own — those live exactly once, in
//! `decision_site_walk.rs`. The roster is single-sourced by a MACHINE CHECK, not by
//! discipline: `crates/engine/tests/core/decision_gate.rs`'s
//! `runtime_decision_coverage_roster_matches_rows` reads this file as source text and
//! asserts the two tables agree, in both directions.
//!
//! **Five rows are observable at runtime, and they are exactly the five
//! `DecisionClass::Served` rows.** The mapping from `BlockingDecision` /
//! `EffectChoiceQuestion` to a row id is [`row_id_for`], EXHAUSTIVE with no wildcard on
//! BOTH enums (the SR-5 forcing pattern `local_game.rs:684-700` already applies to
//! `BlockingDecision` alone) — a new variant of either is a compile error here until
//! someone decides which row it observes, or that it observes none.
//!
//! **Seventeen rows are UNOBSERVABLE, and that is the finding, not a gap in the
//! instrument.** An `AutoChosen` row is one where the engine takes the choice INLINE
//! and leaves no artefact; the absence of an artefact is the SAME property that makes
//! the row a defect. There is no hook to add, because there is no hook. Three
//! alternatives were considered while writing this module and all three rejected:
//!
//! 1. *"a card def hitting row R was cast/resolved"* — needs a `row_hits`-shaped
//!    predicate table here, which is exactly the forbidden second copy.
//! 2. *"the corresponding `Effect` variant executed"* — needs a new engine
//!    instrumentation hook in `effects/mod.rs`. Out of this batch's footprint
//!    (`crates/simulator` + `crates/engine/tests`); a different batch's work.
//! 3. *"infer from events"* (e.g. `Proliferate` → `CounterAdded`) — rejected: the
//!    mapping is a judgement call, not injective (a sacrifice COST emits the same
//!    event as `SacrificePermanents`), and is a second drift surface with no gate.
//!
//! A successor that wants runtime coverage of an `AutoChosen` row should SERVE it
//! (give the engine a real hook, CR-cited, in `decision_site_walk.rs` and the engine
//! itself) — which turns it into a `Served` row and makes it observable for free. That
//! is the right incentive, and is why this module does not try to approximate coverage
//! any other way.
//!
//! **Counts are re-observation-weighted**, exactly like the invariant-violation counts
//! this same batch's Stage 4 documents: the `state.blocking_decision()` branch
//! (`local_game.rs:684`) is re-entered on every `advance()` loop iteration until the
//! decision is answered, and a bot whose answer is refused falls through to
//! `PassPriority` and re-observes the SAME decision next iteration. **The primary
//! output is therefore the boolean reached / never-reached partition** (see
//! [`DecisionCoverage::reached`] / [`DecisionCoverage::never_reached`]); the counts are
//! secondary and must carry this caveat wherever they are printed
//! (`bin/fuzzer.rs::print_decision_coverage`).

use mtg_engine::rules::engine::BlockingDecision;
use mtg_engine::{EffectChoiceQuestion, GameState};

/// Row ids that CAN be observed at runtime. These are exactly the
/// `DecisionClass::Served` rows of `crates/engine/tests/core/decision_site_walk.rs`,
/// and the correspondence is machine-checked by `decision_gate.rs`'s
/// `runtime_decision_coverage_roster_matches_rows`.
///
/// `may_pay_then_effect` joined on PB-DX45 (CR 118.12): both of the engine's
/// `try_pay_optional_cost` call sites now ask rather than pay. This list is the
/// reason the move is not a bookkeeping edit — a row that leaves
/// [`UNOBSERVABLE_ROW_IDS`] gets a counter that must actually be able to move,
/// and PB-DX45's simulator channel probe is what makes it move.
pub const OBSERVABLE_ROW_IDS: &[&str] = &[
    "triggered_targets",
    "search_library",
    "scry",
    "surveil",
    "discard_cards",
    "may_pay_then_effect",
];

/// Row ids with NO runtime hook, and why — one entry per row (16 total: 13
/// `AutoChosen` + 2 `Gated` + 1 `NoDecision`, per `decision_site_walk.rs::ROWS`;
/// `may_pay_then_effect` left this list on PB-DX45). An
/// `AutoChosen` row is one where the engine takes the choice INLINE and leaves no
/// artefact; a `Gated` row is barred from `Complete` entirely by the SR-33 family
/// (`effect_choose_gate.rs`), so no `Complete` def can ever reach it; `wheel_hand` is
/// `NoDecision` — CR 701.9's whole-hand discard has no "which card" pick to hook. The
/// absence of a hook in every case is the same property that makes the row a defect
/// (or, for the two `Gated` rows, the same property that makes the bar work) — these
/// counters can never move and are pinned unobservable rather than silently reported
/// as zero.
pub const UNOBSERVABLE_ROW_IDS: &[(&str, &str)] = &[
    (
        "proliferate",
        "AutoChosen -- auto-selects every eligible permanent/player inline; CR 701.34a \
         gives the choice to the player",
    ),
    (
        "wheel_hand",
        "NoDecision -- the whole hand is discarded (CR 701.9), so there is no 'which \
         card' pick to hook",
    ),
    (
        "sacrifice_permanents",
        "AutoChosen -- picks the n lowest ObjectIds inline; CR 701.21a gives the \
         choice to the controller",
    ),
    (
        "choose_color_or_type",
        "AutoChosen -- picks the most common color/type the controller already has \
         inline (CR 614.12a / 608.2d)",
    ),
    (
        "look_at_top_or_route",
        "AutoChosen -- deterministic routing inline (CR 608.2d); an upper bound on \
         real decisions, not an exact one",
    ),
    (
        "counter_unless_pays",
        "AutoChosen -- always counters inline; the controller is never offered the \
         pay (CR 118.12a)",
    ),
    (
        "modal_trigger",
        "AutoChosen -- a modal TRIGGERED ability's mode is fixed to mode 0 inline \
         (CR 603.3c)",
    ),
    (
        "change_targets",
        "AutoChosen -- declines an optional retarget and picks the lowest-id LEGAL \
         candidate for a mandatory one, inline (CR 115.7a/115.7d, PB-DX25c)",
    ),
    (
        "put_on_library",
        "AutoChosen -- sorted by ascending ObjectId inline; CR 608.2d/401.4 give the \
         order to the player",
    ),
    (
        "bolster_amass",
        "AutoChosen -- least-toughness/smallest-id inline; ties are the \
         CONTROLLER's choice under CR 701.39a/701.47a",
    ),
    (
        "connive",
        "AutoChosen -- the discard half is picked by min ObjectId inline, same as \
         plain DiscardCards before it was served",
    ),
    (
        "discover",
        "AutoChosen -- always casts the exiled card inline; CR 701.57a makes it \
         optional",
    ),
    (
        "may_pay_or_else",
        "Gated -- barred from Complete entirely by \
         effect_choose_gate.rs::no_complete_def_uses_the_may_pay_or_else_stub",
    ),
    (
        "add_mana_filter_choice",
        "AutoChosen -- always taps for one of each colour inline (CR 605.1a); held \
         at 0 Complete defs by authoring discipline alone",
    ),
    (
        "choose_stub",
        "Gated -- barred from Complete entirely by \
         effect_choose_gate.rs::no_complete_def_uses_the_choose_stub",
    ),
    (
        "the_ring_tempts_you",
        "AutoChosen -- ring-bearer is always the lowest id inline (CR 701.54a); held \
         at 0 Complete defs by authoring discipline alone",
    ),
];

/// The total row count. Must equal `decision_site_walk.rs::ROWS.len()` (22) — checked
/// by `decision_gate.rs`'s `runtime_decision_coverage_roster_matches_rows`, not
/// asserted here (this module has no way to read `ROWS`).
pub const ROW_COUNT: usize = OBSERVABLE_ROW_IDS.len() + UNOBSERVABLE_ROW_IDS.len();

/// A per-game (or per-run, once folded across games) tally of how many times each
/// observable-or-not row id was reached. Fixed-size and `Copy`, mirroring
/// `MechanicsTally`/`WasteTally` — retained per game across a whole fuzz run, which is
/// why this is an array and not a map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecisionCoverage {
    observations: [u32; ROW_COUNT],
}

impl Default for DecisionCoverage {
    // Written BY HAND, not derived: `[T; N]: Default` is macro-provided for `N <= 32`
    // only, and a hand-written impl is immune to that boundary moving as ROW_COUNT
    // grows past it.
    fn default() -> Self {
        DecisionCoverage {
            observations: [0; ROW_COUNT],
        }
    }
}

impl DecisionCoverage {
    /// The row id at a given index, across BOTH lists (observable first, then
    /// unobservable) — the single index space `observations` is keyed by.
    fn row_id_at(index: usize) -> &'static str {
        if index < OBSERVABLE_ROW_IDS.len() {
            OBSERVABLE_ROW_IDS[index]
        } else {
            UNOBSERVABLE_ROW_IDS[index - OBSERVABLE_ROW_IDS.len()].0
        }
    }

    fn index_of(row_id: &str) -> Option<usize> {
        (0..ROW_COUNT).find(|&i| Self::row_id_at(i) == row_id)
    }

    /// How many times `row_id` was observed. `0` for an id this table does not
    /// contain, or a row that was never reached.
    pub fn observations(&self, row_id: &str) -> u32 {
        Self::index_of(row_id)
            .map(|i| self.observations[i])
            .unwrap_or(0)
    }

    /// Record one observation of `row_id`. A no-op for an id this table does not
    /// contain (defensive — `row_id_for` can only ever return an id that IS in
    /// `OBSERVABLE_ROW_IDS`, per its own doc, so this branch is unreachable in
    /// practice and is here only so `observe` cannot panic on a caller error).
    pub fn observe(&mut self, row_id: &str) {
        if let Some(i) = Self::index_of(row_id) {
            self.observations[i] = self.observations[i].saturating_add(1);
        }
    }

    /// The observable rows that were reached at least once, i.e. `observations(id) >
    /// 0`. This — not the raw count — is the primary output (see the module doc's
    /// re-observation-weighting note).
    pub fn reached(&self) -> Vec<&'static str> {
        OBSERVABLE_ROW_IDS
            .iter()
            .copied()
            .filter(|id| self.observations(id) > 0)
            .collect()
    }

    /// The observable rows that were NEVER reached.
    pub fn never_reached(&self) -> Vec<&'static str> {
        OBSERVABLE_ROW_IDS
            .iter()
            .copied()
            .filter(|id| self.observations(id) == 0)
            .collect()
    }
}

/// Which `decision_site_walk.rs` row id, if any, a real `BlockingDecision` observes.
///
/// EXHAUSTIVE with no wildcard on BOTH enums (the SR-5 forcing pattern): a new
/// `BlockingDecision` or `EffectChoiceQuestion` variant is a compile error here until
/// someone decides which row it observes. `None` means "a real decision with no ROWS
/// row" — CR 514.1 cleanup discard is the only one today.
///
/// CR 603.3d / 608.2d / 701.9b / 701.22a / 701.23a / 701.25a / 514.1.
pub fn row_id_for(state: &GameState, decision: &BlockingDecision) -> Option<&'static str> {
    match decision {
        // CR 514.1: cleanup discard is a real out-of-band decision, but it has no
        // ROWS row (the audit's §3.1 rows are RESOLUTION/trigger-time choices; CR
        // 514.1 is a turn-based action, not a §3.1 effect row). Recorded explicitly
        // as "no row", not silently skipped, so the exhaustive match still forces a
        // classification if a future ROWS entry ever wants to claim it.
        BlockingDecision::CleanupDiscard { .. } => None,
        BlockingDecision::TriggerTargets { .. } => Some("triggered_targets"),
        BlockingDecision::EffectChoice { .. } => {
            state.pending_effect_choice().and_then(|pending| {
                match &pending.question {
                    EffectChoiceQuestion::SearchLibrary { .. } => Some("search_library"),
                    EffectChoiceQuestion::Scry { .. } => Some("scry"),
                    EffectChoiceQuestion::Surveil { .. } => Some("surveil"),
                    EffectChoiceQuestion::Discard { .. } => Some("discard_cards"),
                    // PB-DX28: CR 115.10's resolution-time untargeted object choice
                    // predates and sits outside the decision-point audit's fixed
                    // §3.1 taxonomy (`decision_site_walk.rs::ROWS`, 22 rows sourced
                    // from `docs/audits/decision-point-audit.md`) -- it is a NEW
                    // primitive, not one of that audit's rows. Recorded explicitly
                    // as "no row" here, mirroring `BlockingDecision::CleanupDiscard`
                    // above, rather than silently forcing a 23rd row into a table
                    // this batch does not own. Extending the audit itself (if
                    // warranted) is a separate task's call to make.
                    EffectChoiceQuestion::ChooseObject { .. } => None,
                    // PB-DX45: CR 118.12's optional cost IS one of the audit's
                    // §3.1 rows -- `may_pay_then_effect`, whose site cell used to
                    // read "pays iff affordable". It moves from `AutoChosen` to
                    // `Served` in `decision_site_walk.rs::ROWS` and out of
                    // `UNOBSERVABLE_ROW_IDS` into `OBSERVABLE_ROW_IDS` in the same
                    // commit as this arm, so the counter can actually move.
                    //
                    // `Effect::LookAtTopThenPlace`'s `place_cost` -- the second
                    // site PB-DX45 repairs -- reaches this same row id, which is
                    // correct: the row is named for the DECISION (CR 118.12's
                    // optional cost), not for the `Effect` variant that offers it.
                    EffectChoiceQuestion::PayOptionalCost { .. } => Some("may_pay_then_effect"),
                    // PB-DX50: CR 702.140c's over/under choice has no row in
                    // `decision_site_walk.rs::ROWS` -- that table's 22 rows are
                    // sourced from `docs/audits/decision-point-audit.md` §3.1,
                    // which never enumerated the mutate arm at all (it is a
                    // `StackObjectKind` resolution arm, not an `Effect` site, so
                    // the audit's own walk could not see it). Recorded explicitly
                    // as "no row" here, mirroring `ChooseObject` above, rather
                    // than forcing a 23rd row into a table this batch does not
                    // own. That the audit missed it is filed, not smuggled in.
                    EffectChoiceQuestion::MutateOnTop { .. } => None,
                }
            })
        }
    }
}

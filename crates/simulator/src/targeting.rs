//! CR 601.2c / CR 602.2b — target announcement for **bot** seats (SIM-5 fix (2),
//! G5 of `memory/playtest-triage-2026-08-02b.md`).
//!
//! # Why this module exists
//!
//! Bots announced **zero targets, always**. `random_bot::action_to_command` built an
//! `ActionParams::default()` and filled only `attackers`/`blockers`, and
//! `Bot::choose_targets` — implemented four times — had no call site outside the bot
//! impls themselves. So `params.rs`'s `CastSpell`/`ActivateAbility` arms forwarded an
//! empty `targets` vector and the engine refused every targeted cast at
//! `casting.rs:5931` (`"expected {}..={} target(s) but got {}"`). The auto-tapper does
//! not look at targets, so the bot had already tapped for the cast by then — that is
//! the wasted mana G5 reported. Fixing the atomicity (SIM-5 fix (1)) stops the mana
//! loss; this module is what lets a bot actually *cast the spell*.
//!
//! The human is unaffected because the play server surfaces target requirements in its
//! view layer (`tools/play-server/src/view.rs`'s `action_target_requirements` /
//! `action_option_view`) and the browser picks. This module mirrors that layer's
//! **semantics**, not its code — and where it deliberately differs, the difference is
//! called out below.
//!
//! # Nothing here re-derives a targeting rule
//!
//! Every legality decision is delegated to `crates/engine/src/rules/queries.rs`, the
//! engine's read-only advisory surface, which itself delegates to the very functions
//! `handle_cast_spell` / `handle_activate_ability` use (`card_def_target_requirements`,
//! `spell_mode_selection`, `per_mode_target_requirements`, `validate_targets_inner`).
//! Re-deriving target legality outside the engine is the drift class `OOS-RS-2` was;
//! see the M11-local session plan §1 fact 4.
//!
//! # Known limits, all deliberate and all recorded as seeds in the SIM-5 handoff
//!
//! * **`TargetRequirement::UpToN` slots are announced empty.** `target_count_range`
//!   gives them a minimum of 0 (`casting.rs:5903`), so declining is always legal, and
//!   "up to one target creature" on an unknown-polarity spell is a strategy question
//!   this module has no basis to answer (`OOS-SIM5-2`).
//! * **Auras are still unannounceable** (`OOS-CARDS2-4`). An Aura's target restriction
//!   lives in `KeywordAbility::Enchant`, not in a `TargetRequirement`, so
//!   `spell_target_requirements` returns an empty list for one and the CR 303.4a check
//!   at `casting.rs:3722` then refuses the cast. The predicate that would decide it
//!   (`rules::sba::get_enchant_target` / `matches_enchant_target`) is `pub(crate)`, and
//!   re-implementing it here is exactly the drift this module refuses to do. Post-fix
//!   (1) the attempt is a harmless no-op that shows up in `LocalGame::rejections()`.
//! * **Candidate choice is "the first legal one", not a strategy.** See
//!   [`plan_targets`].

use mtg_engine::{GameState, ObjectId, PlayerId, Target, TargetRequirement};

use crate::legal_actions::{self, LegalAction};

/// What a bot should announce for one action's targets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetPlan {
    /// The action announces no targets — either it is not a targeting action at all,
    /// or its requirement list is empty. `ActionParams::targets` stays empty, exactly
    /// as it was before SIM-5.
    NotTargeted,
    /// A legal announcement: one target per mandatory requirement, in requirement
    /// order. May be empty when every requirement is `UpToN` (see the module doc).
    Announce(Vec<Target>),
    /// At least one **mandatory** requirement has no legal candidate on this board, so
    /// CR 601.2c makes the action unannounceable however it is parameterised —
    /// `handle_cast_spell` will refuse it.
    ///
    /// # This is the predicate G5's fix (4) would need, and fix (4) is DEFERRED
    ///
    /// `OOS-SIM5-4`. G5 proposed an SR-38 offer gate that suppresses casts whose
    /// targets cannot be satisfied. The predicate exists (here), and wiring it into
    /// `StubProvider::legal_actions` is a short filter — but it was measured before
    /// being written, and it is not worth it *yet*:
    ///
    /// * **Value, measured.** Across seeds 0/7/42 at 25 turns the engine refused
    ///   **166** bot commands. Exactly **1** was a cast this filter would have
    ///   suppressed (`Victimize`, "expected 2..=2 target(s) but got 0", with no
    ///   creature card in the graveyard); up to 4 more were modal *activated
    ///   abilities* whose per-mode requirements `ability_target_requirements`
    ///   deliberately does not report (its own doc, "out of scope here"), so this
    ///   predicate cannot see them either. The remaining ~161 are `InsufficientMana`
    ///   and `activation condition not met` on activations (SIM-6's subject, and
    ///   explicitly out of this batch's scope) plus blocker-declaration refusals.
    /// * **It does not cover `OOS-CARDS2-4`**, which was fix (4)'s main advertised
    ///   benefit. An Aura's restriction is a `KeywordAbility::Enchant`, not a
    ///   `TargetRequirement`, so this returns [`TargetPlan::NotTargeted`] for one; the
    ///   predicates that would decide it (`rules::sba::get_enchant_target` /
    ///   `matches_enchant_target`) are `pub(crate)`. Covering Auras needs a new
    ///   **engine** query, which SIM-5 is forbidden to add (0 engine lines).
    /// * **Cost.** The filter runs a full candidate sweep per offered cast per
    ///   priority window, and `queries::legal_targets_per_slot`'s own doc asks for a
    ///   measurement and a per-`(state, source)` cache before it is put on a polled
    ///   path. Nothing here is cached today.
    /// * **Blast radius.** Removing offers shortens the action list, and `RandomBot`
    ///   picks `rng.random_range(0..legal.len())` — so every recorded fuzz seed and
    ///   every seeded play-server fixture re-rolls, for a 0.6%-of-refusals gain.
    ///
    /// Post fix (1) an unsatisfiable offer costs nothing anyway: the taps roll back
    /// and the refusal is recorded. The successor batch should scope it as an engine
    /// query (`can_announce_targets`, Enchant-aware) plus caching, not as a simulator
    /// filter.
    Unsatisfiable,
}

impl TargetPlan {
    /// The targets to put in `ActionParams::targets`. `Unsatisfiable` yields an empty
    /// vector — the caller has already been told the action is doomed and the engine
    /// is the arbiter either way.
    pub fn announced(&self) -> Vec<Target> {
        match self {
            TargetPlan::Announce(t) => t.clone(),
            TargetPlan::NotTargeted | TargetPlan::Unsatisfiable => Vec::new(),
        }
    }
}

/// The object whose controller/source context a target query runs against. Mirrors
/// `view.rs`'s `target_query_source`.
fn target_query_source(action: &LegalAction) -> Option<ObjectId> {
    match action {
        LegalAction::CastSpell { card, .. } => Some(*card),
        LegalAction::ActivateAbility { source, .. } => Some(*source),
        // Every other variant either takes no targets or carries them inside the
        // action itself (`ActivateBloodrush`, `CastWithMutate`, `ChooseTriggerTargets`),
        // and `params.rs` refuses a `targets` param on all of them with
        // `UnsupportedParam` — announcing one would be a `ParamError`, not a cast.
        _ => None,
    }
}

/// CR 601.2c / CR 602.2b — the target requirements the `Command` that
/// `action_to_command_with_params` will build actually announces.
///
/// **One deliberate divergence from `view.rs`'s `action_target_requirements`, and it
/// is the whole reason this is not a call into that function**: the modes are passed
/// as `legal_actions::spell_default_modes(state, card)` rather than `&[]`. `view.rs`
/// passes an empty slice because the *human* has not chosen their modes at render
/// time; a bot never announces modes at all, so `params.rs`'s `CastSpell` arm fills in
/// exactly that default list (CR 601.2b/700.2a, PB-DP3). Querying with `&[]` would
/// return `vec![]` for a per-mode-targeting card (`queries.rs`' divergence 1) and the
/// bot would announce nothing for a cast whose command *does* select a mode — the same
/// zero-target rejection SIM-5 exists to remove.
///
/// `alt_cost` is `None` for the same reason `view.rs` passes `None`: `params.rs` hard-
/// codes `alt_cost: None` on both arms.
pub fn action_target_requirements(
    state: &GameState,
    action: &LegalAction,
) -> Vec<TargetRequirement> {
    match action {
        LegalAction::CastSpell { card, .. } => mtg_engine::spell_target_requirements(
            state,
            *card,
            &legal_actions::spell_default_modes(state, *card),
            None,
        ),
        LegalAction::ActivateAbility {
            source,
            ability_index,
            ..
        } => mtg_engine::ability_target_requirements(state, *source, *ability_index),
        _ => Vec::new(),
    }
}

/// Choose the targets a bot announces for `action` (CR 601.2c).
///
/// One target per mandatory requirement, taken from that requirement's own candidate
/// list (`mtg_engine::legal_targets_per_slot`, which is parallel to the requirement
/// list by the engine's construction, not by an assumption made here).
///
/// # "The first legal candidate" is a policy, and this is it
///
/// `legal_targets_per_slot` enumerates deterministically — every live player in seat
/// order, then objects in ascending `ObjectId` — and this function takes the first
/// entry each slot offers. That is deliberate on three counts:
///
/// 1. **It consumes no randomness.** `random_bot::action_to_command` receives the RNG
///    that every recorded fuzz seed and every seeded play-server fixture replays; a
///    draw taken here would re-roll each of them for reasons unrelated to targeting.
/// 2. **There is no principled better choice available here.** Picking an opponent's
///    permanent is right for removal and wrong for a pump spell or an Aura, and
///    nothing at this layer knows a spell's polarity. A bot that targets *well* is a
///    `HeuristicBot` scoring question (`OOS-SIM5-1`), not a legality question.
///    **`OOS-SIM5-1` is more than a quality nit, and the seed says so**: candidates are
///    enumerated players-first in seat order, so *every* player-eligible slot
///    (`TargetPlayer`, `TargetAny`, `TargetCreatureOrPlayer`) resolves to **seat 1** —
///    which is the human's seat in a play-server game, and the bot's own seat when the
///    bot is seat 1. Every bot burn spell therefore points at one player. That is
///    legal, and it is what makes the A/B measurement stable, but it changes the
///    *character* of a seeded game rather than merely its strategic quality, so a
///    successor should not read the seed as cosmetic.
/// 3. **It is stable.** A seeded game's journal is the A/B instrument for G5; a
///    stable choice keeps the before/after comparison about the fix.
///
/// # Distinctness (CR 601.2c "another target")
///
/// A candidate already announced for an earlier slot is skipped when the slot has any
/// unused candidate left, because `casting::enforce_inter_target_distinctness`
/// (`casting.rs:6192`) rejects a repeat for the requirements that demand it, and
/// `legal_targets_per_slot` documents that it does **not** apply distinctness across
/// slots. When a slot's only candidates are already spoken for, the first is reused
/// rather than declaring the action unsatisfiable: a repeat is legal for requirements
/// that do not demand distinctness, and where it is not, the engine refuses the cast
/// and (post fix (1)) nothing is spent on it.
pub fn plan_targets(state: &GameState, player: PlayerId, action: &LegalAction) -> TargetPlan {
    let Some(source) = target_query_source(action) else {
        return TargetPlan::NotTargeted;
    };
    let requirements = action_target_requirements(state, action);
    if requirements.is_empty() {
        return TargetPlan::NotTargeted;
    }

    let per_slot = mtg_engine::legal_targets_per_slot(state, player, source, &requirements);
    let mut chosen: Vec<Target> = Vec::new();
    for (candidates, req) in per_slot.iter().zip(requirements.iter()) {
        // `UpToN` is the only requirement whose min differs from its max, and its min
        // is 0 — so this loop body runs once per MANDATORY slot and never for an
        // optional one.
        let (min, _max) = mtg_engine::target_count_range(std::slice::from_ref(req));
        for _ in 0..min {
            let pick = candidates
                .iter()
                .find(|c| !chosen.contains(c))
                .or_else(|| candidates.first());
            match pick {
                Some(target) => chosen.push(target.clone()),
                // CR 601.2c: a mandatory slot with no legal candidate makes the whole
                // announcement impossible.
                None => return TargetPlan::Unsatisfiable,
            }
        }
    }
    TargetPlan::Announce(chosen)
}

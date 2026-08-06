//! CR 115.7 — the "which object or player may become the new target of this
//! stack object" decision, encoded ONCE (PB-DX25c, `OOS-DX25b-3`).
//!
//! Before this module, `Effect::ChangeTargets` (`effects/mod.rs`) decided
//! legality itself: an object candidate was "the smallest `ObjectId` in the
//! same zone that isn't the current target", and a player candidate was "the
//! effect's controller if alive, else the smallest live `PlayerId`" — no
//! `TargetRequirement`, no protection/hexproof/shroud check, and (for
//! players) no `has_conceded` check. CR 115.7a says "another LEGAL target";
//! this module is what makes "legal" mean the same thing it means at cast
//! time, by delegating to `casting::validate_targets_inner` — the exact
//! collective legality arithmetic a real cast is checked against (CR 115.3,
//! CR 115.7e).
//!
//! **Why not `state::stack_registry`**: that module is deliberately a pure
//! classification module with no `GameState` dependency (see its own doc).
//! This decision needs `&GameState` *and* `rules::casting::validate_targets_
//! inner` — putting it there would make `state` depend on `rules`, inverting
//! the crate's layering. **Why not `rules::casting`**: that file is already
//! ~8,700 lines, and a separately-named module is what makes a source gate
//! over "does the `ChangeTargets` arm still contain a second decision"
//! expressible by name (`crates/engine/tests/core/pb_dx25c_retarget_
//! roster.rs`, R4). **Why not inline in `effects/mod.rs`**: that is where
//! the defect lived; a decision meant to be made once must be extractable
//! and gateable.
//!
//! CR 109.5 is load-bearing on the `caster`/`chooser` distinction below:
//! "you" on the VICTIM spell means the victim spell's own controller, not
//! the controller of the spell doing the redirecting. Misdirection's
//! `ctx.controller` (the Misdirection caster) is used ONLY to order
//! candidates (CR 115.7 lets a changing effect's controller choose FROM the
//! legal set; it does not let them redefine what "legal" means). Conflating
//! the two — using the redirecting player as `caster` for legality — is
//! exactly the class of error that let a "target opponent" spell get
//! redirected onto its own caster (`OOS-DX25b-3`).
use crate::state::targeting::{SpellTarget, Target};
use crate::state::{GameState, PlayerId, ZoneId};

/// CR 115.7a — the new target set for a stack object whose targets are being
/// changed, or `None` if CR 115.7a's fallback applies (leave everything
/// alone: the original targets, even if now illegal, are unchanged).
///
/// `chooser` is the player the changing effect instructs to choose (CR
/// 115.7: "allow a PLAYER to change the target(s)"), i.e.
/// `EffectContext.controller`. It is used ONLY to order candidates (the
/// chooser's own player-target is tried first, mirroring HEAD's
/// controller-preference) — never to decide legality. Legality is decided
/// relative to the VICTIM's own controller (`so.controller`), because CR
/// 109.5 makes "you" on the victim spell mean the victim spell's controller
/// (see the module doc).
///
/// `None` is returned, and nothing is written, in every one of these cases
/// (CR 115.7a's own fallback, or this module's fail-closed policy — see
/// `crates/card-types/src/state/stack.rs`'s `target_requirements` doc):
/// * `stack_index` does not name a live stack object.
/// * the stack object announced no targets (`targets.is_empty()`).
/// * no `TargetRequirement` list was recorded for this stack object
///   (`target_requirements.is_empty()`) — FAIL CLOSED: an empty list means
///   "nobody recorded what this object was validated against", not "there
///   is no requirement", and treating it as the latter would silently
///   reintroduce HEAD's unfiltered behaviour behind a friendlier-looking
///   call.
/// * some target index has NO legal replacement (CR 115.7a: "If a target
///   can't be changed to another legal target, the original target is
///   unchanged" — and CR 115.7a's own next sentence, "if all the targets
///   aren't changed ... none of them are", makes ANY such index abort the
///   WHOLE plan, not just that one index).
/// * the final candidate SET, re-validated as a whole (CR 115.7e — "only
///   the final set of targets is evaluated"), turns out illegal. The
///   per-index search below validates each TRIAL set as it goes, but a
///   trial set is not the final set until every index has been decided.
pub(crate) fn plan_target_change(
    state: &GameState,
    stack_index: usize,
    chooser: PlayerId,
) -> Option<Vec<SpellTarget>> {
    // Clone what is needed and drop the borrow immediately — every candidate
    // legality check below re-borrows `state` itself (via `validate_targets_
    // inner`), so holding a borrow of `state.stack_objects` across the loop
    // would not compile.
    let so = state.stack_objects.get(stack_index)?.clone();
    if so.targets.is_empty() {
        return None;
    }
    let reqs = so.target_requirements.clone();
    if reqs.is_empty() {
        return None;
    }
    // CR 702.16b protection checks read the VICTIM spell's own characteristics
    // (what `handle_cast_spell` passed as `Some(&chars)` when the victim was
    // first cast — `casting.rs`'s `announced_requirements` validation site).
    // `victim_card` doubles as `self_id` (CR 601.2c self-targeting prevention /
    // `TargetFilter.exclude_self`), matching that same call site's `card`
    // argument. For a non-card-owning stack kind (an ability) both are `None`;
    // that case is unreachable here today — a `ChangeTargets` victim is always
    // a `Spell`/`MutatingCreatureSpell`, because the only route to one is an
    // announced CARD id resolved through `stack_registry::stack_index_for_
    // announced_target`, and an ability's stack entry owns no card
    // (`state::stack_registry::card_in_stack_zone` returns `None` for every
    // ability variant) — so it is never in `state.objects` and can never be
    // announced by a player. Recorded as `OOS-DX25c-3`.
    let victim_card = crate::state::stack_registry::card_in_stack_zone(&so.kind);
    let source_chars =
        victim_card.and_then(|id| crate::rules::layers::calculate_characteristics(state, id));

    let candidates = retarget_candidates(state, chooser);

    // Greedy per-index search. Each trial is validated as a WHOLE SET (CR
    // 115.3 inter-target distinctness, CR 115.7e) via `validate_targets_
    // inner` — never index-matched against a stored per-slot requirement,
    // because `casting::validate_mapped_targets`' own doc states the
    // returned `Vec<SpellTarget>` does NOT preserve a target→slot mapping
    // (declaration order is kept, not requirement-slot order), so a stored
    // `target_requirements` list only supports set-level re-validation.
    //
    // Known incompleteness, stated rather than glossed (`OOS-DX25c-1`): this
    // search is GREEDY, so for target count > 1 it can fail to find a legal
    // assignment that exists (Bolt Bend's 2024-11-08 ruling: "you must change
    // the target if possible"). A complete search is combinatorial in the
    // candidate universe; the failure direction is "leave unchanged" — CR
    // 115.7a's own fallback — and the reachable population is measured as
    // zero (every `must_change: true` corpus user requires exactly one
    // target).
    let mut next: Vec<Target> = so.targets.iter().map(|st| st.target.clone()).collect();
    for i in 0..next.len() {
        let current = so.targets[i].target.clone();
        let pick = candidates.iter().find(|c| {
            **c != current && {
                let mut trial = next.clone();
                trial[i] = (*c).clone();
                crate::rules::casting::validate_targets_inner(
                    state,
                    &trial,
                    &reqs,
                    so.controller,
                    source_chars.as_ref(),
                    victim_card,
                )
                .is_ok()
            }
        })?; // CR 115.7a: one index with no legal change => change NOTHING.
        next[i] = pick.clone();
    }

    // CR 115.7e: the greedy loop above validated MIXED trial sets (some
    // indices already picked, some still original) — never the actual final
    // set. Re-validate the final set once, on its own, before committing to
    // it.
    crate::rules::casting::validate_targets_inner(
        state,
        &next,
        &reqs,
        so.controller,
        source_chars.as_ref(),
        victim_card,
    )
    .ok()?;

    // CR 608.2b: `zone_at_cast` must describe the NEW target's zone, not the
    // old one. HEAD copied the original target's `zone_at_cast` onto the new
    // target (`effects/mod.rs`, pre-PB-DX25c) — harmless only because the old
    // same-zone filter made old and new zones identical by construction; that
    // filter no longer exists (§3.3: CR 115.7a imposes no zone-identity
    // restriction, only the per-`TargetRequirement` zone each arm already
    // enforces via the validator above).
    Some(
        next.into_iter()
            .map(|target| {
                let zone_at_cast = match &target {
                    Target::Object(id) => state.objects().get(id).map(|o| o.zone),
                    Target::Player(_) => None,
                };
                SpellTarget {
                    target,
                    zone_at_cast,
                }
            })
            .collect(),
    )
}

/// The candidate universe for a retarget, in deterministic order.
///
/// Mirrors `rules::queries::legal_targets_per_slot`'s universe EXACTLY (see
/// that function's doc for why those three object zones and no others — the
/// exact zone set every arm of `casting::validate_object_satisfies_
/// requirement` can accept), with one inherited preference: the `chooser` is
/// offered first, matching HEAD's `Effect::ChangeTargets` controller-first
/// behaviour (`OOS-DX25b`-era `effects/mod.rs`) so that behaviour is not
/// silently dropped as a side effect of this batch's legality fix — this
/// batch changes what is LEGAL, not what is PREFERRED among legal
/// candidates.
///
/// Order:
/// 1. `Target::Player(chooser)`, if the chooser is still in the game
///    (`!has_lost && !has_conceded`).
/// 2. Every other player in `state.turn.turn_order`, in seat order, alive by
///    the same two flags (`rules::queries::legal_targets_per_slot`'s exact
///    test).
/// 3. Every object in `state.objects()` (ascending `ObjectId` —
///    `imbl::OrdMap` iteration order) whose zone is `Battlefield`, `Stack`,
///    or `Graveyard(_)`.
///
/// `pub(crate)` so the R6 gate and the test probes can assert this universe
/// is the same SET `legal_targets_per_slot` enumerates, by execution rather
/// than by this comment.
pub(crate) fn retarget_candidates(state: &GameState, chooser: PlayerId) -> Vec<Target> {
    let mut candidates: Vec<Target> = Vec::new();

    let chooser_alive = state
        .expect_player(chooser)
        .map(|ps| !ps.has_lost && !ps.has_conceded)
        .unwrap_or(false);
    if chooser_alive {
        candidates.push(Target::Player(chooser));
    }
    for &p in state.turn.turn_order.iter() {
        if p == chooser {
            continue; // already pushed first above, if alive.
        }
        if let Some(pl) = state.expect_player(p) {
            if !pl.has_lost && !pl.has_conceded {
                candidates.push(Target::Player(p));
            }
        }
    }

    for (id, obj) in state.objects().iter() {
        if matches!(
            obj.zone,
            ZoneId::Battlefield | ZoneId::Stack | ZoneId::Graveyard(_)
        ) {
            candidates.push(Target::Object(*id));
        }
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::card_definition::{TargetFilter, TargetRequirement};
    use crate::state::builder::{GameStateBuilder, ObjectSpec};
    use crate::state::game_object::ObjectId;
    use crate::state::types::CardType;

    fn p(n: u64) -> PlayerId {
        PlayerId(n)
    }

    /// R6 (plan §5.4): `retarget_candidates`'s universe is the SAME SET
    /// `rules::queries::legal_targets_per_slot` enumerates -- proved by
    /// EXECUTION, not by the doc comment above claiming it. Built as an
    /// in-source test because `retarget_candidates` is `pub(crate)`, invisible
    /// to `crates/engine/tests/` (the `casting.rs::validate_target_spell_
    /// with_single_target_self_and_kind_check` precedent PB-DX25b's own T8
    /// cited).
    ///
    /// The PLAYER half is recovered via a single `TargetPlayer` call (every
    /// alive player satisfies it unconditionally, no other requirement does).
    /// The OBJECT half is recovered via the UNION of three single-clause
    /// requirements, each of which accepts EVERY object in exactly one of the
    /// three zones `retarget_candidates` covers with NO type restriction:
    /// `TargetPermanent` (Battlefield, unconditional), `TargetSpell` (Stack,
    /// unconditional), `TargetCardInGraveyard(default filter)` (Graveyard(_),
    /// unconditional). **Residual, stated rather than glossed**: this proves
    /// the SET equality this specific fixture produces; it is not a proof
    /// that no OTHER zone or player-exclusion path could ever diverge.
    #[test]
    fn r6_candidate_universe_matches_legal_targets_per_slot() {
        let p1 = p(1);
        let p2 = p(2);
        let p3 = p(3);

        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .add_player(p3)
            .object(ObjectSpec::creature(p2, "R6 Battlefield Creature", 2, 2))
            .object(
                ObjectSpec::card(p2, "R6 Graveyard Card")
                    .in_zone(ZoneId::Graveyard(p2))
                    .with_types(vec![CardType::Instant]),
            )
            .object(
                ObjectSpec::card(p3, "R6 Stack Spell")
                    .in_zone(ZoneId::Stack)
                    .with_types(vec![CardType::Instant]),
            )
            .build()
            .unwrap();

        let chooser = p1;
        let dummy_source = ObjectId(999_999);

        let mut from_retarget: Vec<Target> = retarget_candidates(&state, chooser);
        from_retarget.sort_by_key(sort_key);
        from_retarget.dedup();

        let via_query = crate::rules::queries::legal_targets_per_slot(
            &state,
            chooser,
            dummy_source,
            &[
                TargetRequirement::TargetPlayer,
                TargetRequirement::TargetPermanent,
                TargetRequirement::TargetSpell,
                TargetRequirement::TargetCardInGraveyard(TargetFilter::default()),
            ],
        );
        let mut from_query: Vec<Target> = via_query.into_iter().flatten().collect();
        from_query.sort_by_key(sort_key);
        from_query.dedup();

        assert!(
            !from_retarget.is_empty(),
            "R6 non-vacuity: retarget_candidates must return a non-empty set \
             on this fixture (a player, a creature, a graveyard card and a \
             stack spell are all present)"
        );
        assert_eq!(
            from_retarget, from_query,
            "R6: retarget_candidates's universe must equal the union of \
             legal_targets_per_slot's TargetPlayer + TargetPermanent + \
             TargetSpell + TargetCardInGraveyard(default) slots -- \
             retarget: {from_retarget:?}, query: {from_query:?}"
        );
    }

    fn sort_key(t: &Target) -> (u8, u64) {
        match t {
            Target::Player(p) => (0, p.0),
            Target::Object(o) => (1, o.0),
        }
    }
}

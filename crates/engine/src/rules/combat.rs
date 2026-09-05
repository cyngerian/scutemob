//! Combat phase rules handler (CR 506-511).
//!
//! Handles the full combat phase:
//! - DeclareAttackers (CR 508): active player declares attackers and their targets
//! - DeclareBlockers (CR 509): defending players declare blockers
//! - OrderBlockers (CR 509.2): attacker chooses damage assignment order for multiple blockers
//! - Combat damage (CR 510): simultaneous damage, trample, deathtouch, first/double strike
//! - Commander damage tracking (CR 903.10a)
use super::abilities;
use super::casting;
use super::events::{CombatDamageAssignment, CombatDamageTarget, GameEvent};
use super::layers::calculate_characteristics;
use crate::state::combat::{AttackTarget, CombatState};
use crate::state::error::GameStateError;
use crate::state::game_object::{Designations, HybridManaPayment, ManaCost, ObjectId};
use crate::state::player::{CardId, PlayerId};
use crate::state::stubs::{FlushResumeSite, GameRestriction};
use crate::state::turn::Step;
use crate::state::types::{
    BlockingExceptionFilter, CardType, Color, CounterType, KeywordAbility, LandwalkType, SuperType,
};
use crate::state::zone::ZoneId;
use crate::state::GameState;
use imbl::{OrdMap, OrdSet};
use std::collections::{BTreeMap, BTreeSet};
// ---------------------------------------------------------------------------
// Declare Attackers
// ---------------------------------------------------------------------------
/// Handle a DeclareAttackers command (CR 508.1).
///
/// The active player announces which creatures are attacking and what they
/// attack. Non-Vigilance attackers become tapped (CR 508.1f). After declaring,
/// triggers are flushed and priority is granted to the active player.
///
/// `hybrid_choices`/`phyrexian_life_payments` (PB-DX6, CR 107.4e/107.4f via CR
/// 508.1h) index the CR 508.1h attack-tax TOTAL, not any printed cost -- see
/// `accumulate_attack_tax_total`'s doc for the canonical (copy-major) pip order,
/// which is defined there once and shared with `queries::attack_tax_total` so a
/// client can obtain the exact cost these choices index without re-deriving the
/// accumulation itself (the OOS-RS-2 drift class).
pub fn handle_declare_attackers(
    state: &mut GameState,
    player: PlayerId,
    attackers: Vec<(ObjectId, AttackTarget)>,
    enlist_choices: Vec<(ObjectId, ObjectId)>,
    exert_choices: Vec<ObjectId>,
    hybrid_choices: Vec<HybridManaPayment>,
    phyrexian_life_payments: Vec<bool>,
) -> Result<Vec<GameEvent>, GameStateError> {
    // Must be in the DeclareAttackers step.
    if state.turn.step != Step::DeclareAttackers {
        return Err(GameStateError::InvalidCommand(
            "DeclareAttackers is only valid in the DeclareAttackers step".into(),
        ));
    }
    // Must be the active player.
    if player != state.turn.active_player {
        return Err(GameStateError::InvalidCommand(
            "Only the active player can declare attackers".into(),
        ));
    }
    // Must have priority (CR 508.1 is a turn-based action but requires player to have priority).
    if state.turn.priority_holder != Some(player) {
        return Err(GameStateError::NotPriorityHolder {
            expected: state.turn.priority_holder,
            actual: player,
        });
    }
    // CR 508.1 (PB-DX21, OOS-M11-9): declaring attackers is a once-per-combat
    // turn-based action. Rejected HERE, before the CombatState init below and
    // before any validation, tapping (508.1f) or cost payment (508.1j), so a
    // refused re-declaration leaves the game byte-identical (CR 732: "the game
    // returns to the moment before the declaration").
    if state.combat.as_ref().is_some_and(|c| c.attackers_declared) {
        return Err(GameStateError::AlreadyDeclaredAttackers(player));
    }
    // (PB-DX51, `OOS-DX21-5`: the `CombatState` init that used to stand here has
    // moved BELOW the last `return Err` in this function -- see the block just above
    // `let mut events`. Nothing between here and there reads `state.combat` in a way
    // that distinguishes `None` from a fresh empty `CombatState`: the only reads on
    // this path are `calculate_characteristics`' attacking/blocking membership probes,
    // and `state.combat.as_ref().is_some_and(|c| c.attackers.contains_key(..))` is
    // `false` for both. In a real game the branch never fires at all, because
    // `turn_actions::begin_combat` already installed a `CombatState` at
    // `BeginningOfCombat`; it exists for fixtures that enter the step directly.)
    // Validate each attacker and collect vigilance flags for the tapping loop below.
    // MR-M6-12: capture has_vigilance here to avoid a second calculate_characteristics
    //           call in the tapping loop.
    let mut attacker_vigilance: Vec<(ObjectId, bool)> = Vec::with_capacity(attackers.len());
    for (attacker_id, target) in &attackers {
        let obj = state.object(*attacker_id)?;
        if obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(*attacker_id));
        }
        // CR 702.26b: Phased-out permanents are removed from combat and cannot attack.
        if obj.status.phased_out {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} is phased out and cannot attack",
                attacker_id
            )));
        }
        if obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: *attacker_id,
            });
        }
        // Must be a creature.
        let chars = calculate_characteristics(state, *attacker_id)
            .ok_or(GameStateError::ObjectNotFound(*attacker_id))?;
        if !chars.card_types.contains(&CardType::Creature) {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} is not a creature",
                attacker_id
            )));
        }
        // CR 702.3a: A creature with defender can't attack.
        if chars.keywords.contains(&KeywordAbility::Defender) {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} has defender and cannot attack",
                attacker_id
            )));
        }
        let has_vigilance = chars.keywords.contains(&KeywordAbility::Vigilance);
        let has_haste = chars.keywords.contains(&KeywordAbility::Haste);
        let obj = state.object(*attacker_id)?;
        // Must not already be tapped (unless Vigilance).
        if obj.status.tapped && !has_vigilance {
            return Err(GameStateError::PermanentAlreadyTapped(*attacker_id));
        }
        // CR 302.6 / CR 702.10: Summoning sickness prevents attacking unless the
        // creature has haste.
        if obj.has_summoning_sickness && !has_haste {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} has summoning sickness and cannot attack (no haste)",
                attacker_id
            )));
        }
        // MR-M6-01: validate attack target (CR 508.1, CR 903.6).
        // A player may only attack opponents or their planeswalkers.
        match target {
            AttackTarget::Player(pid) => {
                if *pid == player {
                    return Err(GameStateError::InvalidAttackTarget(
                        "a player cannot attack themselves".into(),
                    ));
                }
                // CR 508.1c / PB-AC8: CantAttackOwner -- keys on the attacker's OWNER,
                // not its controller (Alexios, Deimos of Kosmos changes control every
                // upkeep; it still can't attack the player who owns it).
                if *pid == obj.owner {
                    let restricted = state.restrictions.iter().any(|r| {
                        r.source == *attacker_id
                            && matches!(r.restriction, GameRestriction::CantAttackOwner)
                            && state
                                .objects
                                .get(&r.source)
                                .map(|o| o.zone == ZoneId::Battlefield)
                                .unwrap_or(false)
                    });
                    if restricted {
                        return Err(GameStateError::InvalidCommand(format!(
                            "Creature {:?} can't attack its owner (CR 508.1c)",
                            attacker_id
                        )));
                    }
                }
                let target_player = state
                    .players
                    .get(pid)
                    .ok_or(GameStateError::PlayerNotFound(*pid))?;
                if target_player.has_lost || target_player.has_conceded {
                    return Err(GameStateError::InvalidAttackTarget(format!(
                        "player {pid:?} is eliminated"
                    )));
                }
            }
            AttackTarget::Planeswalker(pw_id) => {
                let (pw_zone, pw_controller) = state
                    .objects
                    .get(pw_id)
                    .map(|pw| (pw.zone, pw.controller))
                    .ok_or_else(|| {
                        GameStateError::InvalidAttackTarget(format!(
                            "planeswalker object {pw_id:?} does not exist"
                        ))
                    })?;
                if pw_zone != ZoneId::Battlefield {
                    return Err(GameStateError::InvalidAttackTarget(format!(
                        "planeswalker object {pw_id:?} is not on the battlefield"
                    )));
                }
                if pw_controller == player {
                    return Err(GameStateError::InvalidAttackTarget(format!(
                        "planeswalker {pw_id:?} is controlled by the attacking player"
                    )));
                }
                let pw_chars = calculate_characteristics(state, *pw_id)
                    .ok_or(GameStateError::ObjectNotFound(*pw_id))?;
                if !pw_chars.card_types.contains(&CardType::Planeswalker) {
                    return Err(GameStateError::InvalidAttackTarget(format!(
                        "object {pw_id:?} is not a Planeswalker"
                    )));
                }
            }
        }
        attacker_vigilance.push((*attacker_id, has_vigilance));
    }
    // CR 508.1c / 508.1h / 508.1j: attack-cost restrictions (Propaganda, Ghostly Prison).
    //
    // "Creatures can't attack you unless their controller pays {N} for each creature they
    // control that's attacking you" is a RESTRICTION (CR 508.1c), not an optional cost
    // (CR 508.1g -- that rule covers exert-style "as it attacks" costs). The payment
    // machinery is CR 508.1h (determine the total, lock it in), CR 508.1i (mana-ability
    // window) and CR 508.1j ("they pay all costs in any order. Partial payments are not
    // allowed"). Costs from multiple sources are cumulative (Propaganda ruling).
    //
    // Affordability is checked HERE, before any state is mutated, so an unaffordable
    // declaration is rejected with the game untouched (CR 508.1 / CR 732: "the declaration
    // is illegal; the game returns to the moment before the declaration"). The DEBIT
    // happens after the tapping loop below, matching CR 508.1f -> 508.1j order.
    //
    // PB-DX6 (OOS-DP4-1, closed by this batch): hybrid and Phyrexian pips in an attack
    // tax are now PAYABLE -- `hybrid_choices`/`phyrexian_life_payments` on this command
    // index the CR 508.1h TOTAL, accumulated by `accumulate_attack_tax_total` (shared,
    // unchanged, with `queries::attack_tax_total` so the pip order cannot drift -- see
    // that function's own doc for the canonical copy-major order and the Norn's Annex
    // "individually" rulings that make it the only rules-correct shape, plan §5.2.1).
    // X is still rejected: `Command::DeclareAttackers` has no channel to announce it
    // (CR 107.3/601.2b), tracked as OOS-DX6-1 (replacing OOS-DP4-1).
    //
    // CR 508.1i (a mana-ability window between "total determined" and "total paid") is
    // NOT honoured here -- the engine determines and pays the total inside this single
    // command, so the tax must already be floating; that is a pre-existing deviation,
    // recorded as OOS-DP4-2, not fixed by this change.
    //
    // Fix cycle (E1, PB-DP4): the X rejection below is scoped to defenders an attacker
    // actually targets, not to the mere existence of a restriction anywhere on the
    // battlefield. Rejecting the WHOLE declaration whenever an X-taxed restriction
    // exists at all -- including an attack against a different, untaxed defender, a
    // planeswalker-only attack, or an empty `attackers: vec![]` -- would be wrong.
    let (attack_tax, flat_total, phyrexian_life, taxed_defenders): (
        Option<ManaCost>,
        ManaCost,
        u32,
        BTreeSet<PlayerId>,
    ) = {
        // CR 107.3/601.2b: defenders whose tax includes an X pip that
        // Command::DeclareAttackers has no channel to announce (hybrid and
        // Phyrexian pips ARE payable now -- see `accumulate_attack_tax_total`).
        // Renamed from `unpayable_tax_defenders` (PB-DX6) -- that name would be a
        // lying identifier now that hybrid/Phyrexian are no longer in this bucket,
        // exactly the OOS-DP7-2 class this suite keeps re-creating.
        let mut x_tax_defenders: BTreeSet<PlayerId> = BTreeSet::new();
        // All defenders with a live, nonzero CantAttackYouUnlessPay restriction
        // against them, payable or not -- CR 508.1d's has_uncosted_attack_target
        // needs the union. An X-taxed defender is, if anything, an even stronger
        // case for "not required to pay that cost" than a payable one.
        let mut taxed_defenders: BTreeSet<PlayerId> = BTreeSet::new();
        for restriction in state.restrictions.iter() {
            // Skip if source is no longer on the battlefield.
            let source_on_bf = state
                .objects
                .get(&restriction.source)
                .map(|o| matches!(o.zone, ZoneId::Battlefield))
                .unwrap_or(false);
            if !source_on_bf {
                continue;
            }
            if let GameRestriction::CantAttackYouUnlessPay { cost_per_creature } =
                &restriction.restriction
            {
                // E7 fix (PB-DP4), CR 118.5: a {0} restriction (all fields zero) is
                // unconditionally payable and must not mark its defender "taxed" --
                // has_uncosted_attack_target treats any taxed_defenders member as a
                // costed target, so a free restriction would wrongly close off an
                // otherwise-uncosted must-attack target. Unchanged by this batch.
                if *cost_per_creature == ManaCost::default() {
                    continue;
                }
                let defending_player = restriction.controller;
                taxed_defenders.insert(defending_player);
                if cost_per_creature.x_count > 0 {
                    x_tax_defenders.insert(defending_player);
                }
            }
        }
        // CR 508.1c: reject only if a DECLARED attacker actually targets a defender
        // whose tax carries an unannounced X (E1 fix's scoping, preserved) -- an
        // attack against a different, untaxed defender, a planeswalker-only attack,
        // or an empty declaration is never blocked by a restriction it doesn't
        // engage.
        for (_, target) in &attackers {
            if let AttackTarget::Player(defending_pid) = target {
                if x_tax_defenders.contains(defending_pid) {
                    return Err(GameStateError::InvalidCommand(format!(
                        "attack tax: an X in the attack cost against defender {:?} cannot \
                         be announced -- Command::DeclareAttackers carries no X-payment \
                         channel (CR 107.3/601.2b via CR 508.1h); see OOS-DX6-1.",
                        defending_pid
                    )));
                }
            }
        }
        // CR 508.1h: the total, via the SAME accumulation `queries::attack_tax_total`
        // calls -- two copies of this order is how OOS-RS2-1/OOS-DP4-1 happened.
        let total = accumulate_attack_tax_total(state, &attackers);
        // CR 508.1h/508.1j: affordability, checked before any mutation so an
        // unaffordable declaration leaves the game untouched (CR 732). Evaluated on
        // the PIPPED total, not the flattened one -- a cost_per_creature that is
        // entirely Phyrexian and entirely paid with life flattens to {0} with
        // phyrexian_life > 0, and gating on the flattened value here would silently
        // skip the whole payment (plan §13 risk 9, pinned by T11).
        let (flat_total, phyrexian_life) = if total != ManaCost::default() {
            // CR 107.4e/107.4f: flatten ONCE, against the accumulated total, never
            // against any individual restriction's cost_per_creature -- design (A),
            // plan §5.2.1. Design (B) (flatten-then-multiply) is rules-wrong: the
            // Norn's Annex rulings say each copy of a cost is chosen INDIVIDUALLY,
            // which (B) cannot express and which flattening the total once, after
            // full replication, preserves.
            let (flat, life) = total
                .flatten_hybrid_phyrexian(&hybrid_choices, &phyrexian_life_payments)
                .map_err(GameStateError::InvalidCommand)?;
            // CR 119.4, before any mutation. CR 119.4b: 0 life is always payable, so
            // the guard short-circuits on > 0 -- mirrors `rules/engine.rs`'s
            // `handle_turn_face_up` (`combined_life_cost > 0`) exactly.
            if life > 0 {
                let (life_ok, current_life) = state
                    .expect_player(player)
                    .map(|ps| (ps.life_total >= life as i32, ps.life_total))
                    .unwrap_or((false, 0));
                if !life_ok {
                    return Err(GameStateError::InsufficientLife {
                        player,
                        required: life,
                        actual: current_life,
                    });
                }
            }
            let (affordable, available) = state
                .expect_player(player)
                .map(|ps| {
                    (
                        casting::can_pay_cost(&ps.mana_pool, &flat),
                        ps.mana_pool.total(),
                    )
                })
                .unwrap_or((false, 0));
            if !affordable {
                // CR 106.6 (fix cycle, E5, PB-DP4): restricted mana cannot pay a
                // non-spell cost in this engine -- every `ManaRestriction` variant is
                // spell-scoped (see `player.rs::restriction_matches`) -- so
                // `can_pay_cost` above already excludes it, and `available` below is
                // already the unrestricted total. That is engine-internal reasoning,
                // not a fact a player needs in an error string, so it stays in this
                // comment. The message below states the FLATTENED cost -- that is
                // the thing that must actually be payable -- and the available
                // quantity WITHOUT asserting "not enough" as the cause -- the
                // shortfall can be a colour mismatch (required and available totals
                // equal) as easily as a quantity shortfall.
                return Err(GameStateError::InvalidCommand(format!(
                    "attack tax: the attacking player cannot pay the required {:?} for the \
                     declared attackers from their mana pool (CR 508.1h/508.1j, \
                     Propaganda/Ghostly Prison); {} unrestricted mana available.",
                    flat, available
                )));
            }
            (flat, life)
        } else {
            (ManaCost::default(), 0)
        };
        (
            if total == ManaCost::default() {
                None
            } else {
                Some(total)
            },
            flat_total,
            phyrexian_life,
            taxed_defenders,
        )
    };
    // CR 701.15b: A goaded creature must attack each combat if able.
    // For each creature on the battlefield controlled by the active player
    // that has at least one goading player in goaded_by: if the creature can
    // attack (not tapped without vigilance, no summoning sickness without haste,
    // no Defender), it must be in the attackers list.
    let declared_attacker_ids: OrdSet<ObjectId> = attackers.iter().map(|(id, _)| *id).collect();
    {
        let goaded_ids: Vec<ObjectId> = state
            .objects
            .values()
            .filter(|obj| {
                obj.zone == ZoneId::Battlefield
                    && obj.controller == player
                    && !obj.goaded_by.is_empty()
            })
            .map(|obj| obj.id)
            .collect();
        for goaded_id in goaded_ids {
            if declared_attacker_ids.contains(&goaded_id) {
                continue;
            }
            // Check if the creature is able to attack.
            let chars = crate::rules::layers::expect_characteristics(state, goaded_id);
            let obj = match state.expect_object(goaded_id) {
                Some(o) => o,
                None => continue,
            };
            let has_vigilance = chars.keywords.contains(&KeywordAbility::Vigilance);
            let has_haste = chars.keywords.contains(&KeywordAbility::Haste);
            let has_defender = chars.keywords.contains(&KeywordAbility::Defender);
            let is_tapped = obj.status.tapped;
            let has_sickness = obj.has_summoning_sickness;
            // CR 508.1d: a requirement is obeyed only to the extent it doesn't
            // violate a restriction -- including a CantAttackOwner exclusion (CR
            // 508.1c) AND a "not required to pay an attack cost" carve-out (CR
            // 508.1d itself: "If a creature can't attack unless a player pays a
            // cost, that player is not required to pay that cost"). If every
            // uncosted target is closed off, the creature has no legal attack
            // target at all -- it is not "able" to attack, so goad's must-attack
            // requirement does not force it. Mirrors the MustAttackEachCombat
            // block below. PB-DP4, closes OOS-RS3-4 (2014-07-18 Goblin
            // Rabblemaster ruling: "If there's a cost associated with having a
            // creature attack, you're not forced to pay that cost.").
            let no_legal_target =
                !has_uncosted_attack_target(state, player, goaded_id, &taxed_defenders);
            // Creature cannot attack if: tapped and no vigilance, or summoning sickness
            // and no haste, or has Defender, or (CR 508.1d) no legal attack target.
            let cannot_attack = (is_tapped && !has_vigilance)
                || (has_sickness && !has_haste)
                || has_defender
                || no_legal_target;
            if !cannot_attack {
                return Err(GameStateError::InvalidCommand(format!(
                    "Goaded creature {:?} must attack (CR 701.15b)",
                    goaded_id
                )));
            }
        }
    }
    // CR 701.15b: A goaded creature must attack a player other than the goading
    // player if able. For each declared attacker that is goaded, if its target is
    // one of the goading players, verify there is no other valid (non-goading) target.
    {
        let opponent_ids: Vec<PlayerId> = state
            .players
            .keys()
            .filter(|pid| **pid != player)
            .filter(|pid| {
                state
                    .expect_player(**pid)
                    .map(|p| !p.has_lost && !p.has_conceded)
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        for (attacker_id, target) in &attackers {
            let obj = match state.expect_object(*attacker_id) {
                Some(o) => o,
                None => continue,
            };
            if obj.goaded_by.is_empty() {
                continue;
            }
            if let AttackTarget::Player(target_pid) = target {
                if obj.goaded_by.contains(target_pid) {
                    // Check if any non-goading opponent exists.
                    let has_non_goading_target =
                        opponent_ids.iter().any(|pid| !obj.goaded_by.contains(pid));
                    if has_non_goading_target {
                        return Err(GameStateError::InvalidCommand(format!(
                            "Goaded creature {:?} must attack a player other than the goading player if able (CR 701.15b)",
                            attacker_id
                        )));
                    }
                }
            }
        }
    }
    // CR 508.1d: Creatures with "attacks each combat if able" must attack if able.
    // Similar to goaded enforcement but without directional restriction.
    {
        for (obj_id, obj) in &state.objects {
            if obj.zone != ZoneId::Battlefield || obj.controller != player {
                continue;
            }
            let chars = crate::rules::layers::expect_characteristics(state, *obj_id);
            if !chars
                .keywords
                .contains(&KeywordAbility::MustAttackEachCombat)
            {
                continue;
            }
            if declared_attacker_ids.contains(obj_id) {
                continue;
            }
            // Check if the creature is able to attack.
            let has_vigilance = chars.keywords.contains(&KeywordAbility::Vigilance);
            let has_haste = chars.keywords.contains(&KeywordAbility::Haste);
            let has_defender = chars.keywords.contains(&KeywordAbility::Defender);
            let is_tapped = obj.status.tapped;
            let has_sickness = obj.has_summoning_sickness;
            // CR 508.1d: a requirement is obeyed only to the extent it doesn't
            // violate a restriction -- including a CantAttackOwner exclusion (CR
            // 508.1c) AND a "not required to pay an attack cost" carve-out (CR
            // 508.1d itself: "that player is not required to pay that cost, even
            // if attacking with that creature would increase the number of
            // requirements being obeyed"). If every uncosted target is closed
            // off, the creature has no legal attack target at all -- it is not
            // "able" to attack, so the must-attack requirement does not force
            // it. (Alexios; also PB-DP4, closes OOS-RS3-4 for Goblin Rabblemaster
            // -- 2014-07-18 ruling: "If there's a cost associated with having a
            // creature attack, you're not forced to pay that cost.")
            let no_legal_target =
                !has_uncosted_attack_target(state, player, *obj_id, &taxed_defenders);
            let cannot_attack = (is_tapped && !has_vigilance)
                || (has_sickness && !has_haste)
                || has_defender
                || no_legal_target;
            if !cannot_attack {
                return Err(GameStateError::InvalidCommand(format!(
                    "Creature {:?} must attack each combat if able (CR 508.1d)",
                    obj_id
                )));
            }
        }
    }
    // ---- CR 702.154a / CR 508.1g: Validate enlist choices ----
    //
    // Each (enlisting_attacker_id, enlisted_creature_id) must satisfy:
    //  1. The attacker is in the declared_attacker_ids set.
    //  2. The attacker has the Enlist keyword (layer-aware check).
    //  3. The enlisted creature is on the battlefield, controlled by the player.
    //  4. The enlisted creature is NOT in the declared_attacker_ids set.
    //  5. The enlisted creature is untapped.
    //  6. The enlisted creature is a creature (layer-aware check).
    //  7. The enlisted creature does not have summoning sickness (or has haste).
    //  8. Each enlisted creature appears at most once across ALL enlist choices
    //     (ruling 2022-09-09: "a single creature can't be tapped for more than
    //     one enlist ability").
    //  9. For a given attacker, the number of enlist choices must not exceed
    //     the number of Enlist keyword instances on that attacker (CR 702.154d).
    // 10. The enlisted creature is not the same as the attacker (CR 702.154c).
    {
        let mut enlisted_ids_used: Vec<ObjectId> = Vec::new();
        let mut enlist_used_per_attacker: OrdMap<ObjectId, u32> = OrdMap::new();
        for (attacker_id, enlisted_id) in &enlist_choices {
            // Check 10: cannot enlist itself (CR 702.154c).
            if attacker_id == enlisted_id {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: creature {:?} cannot enlist itself (CR 702.154c)",
                    attacker_id
                )));
            }
            // Check 1: attacker is declared.
            if !declared_attacker_ids.contains(attacker_id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: creature {:?} is not a declared attacker",
                    attacker_id
                )));
            }
            // Check 2: attacker has Enlist keyword + check 9: instance count.
            let attacker_chars = calculate_characteristics(state, *attacker_id)
                .ok_or(GameStateError::ObjectNotFound(*attacker_id))?;
            let enlist_count = attacker_chars
                .keywords
                .iter()
                .filter(|kw| matches!(kw, KeywordAbility::Enlist))
                .count() as u32;
            if enlist_count == 0 {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: attacker {:?} does not have the Enlist keyword",
                    attacker_id
                )));
            }
            let used = enlist_used_per_attacker.entry(*attacker_id).or_insert(0);
            *used += 1;
            if *used > enlist_count {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: attacker {:?} has {} Enlist instance(s) but {} choices were made",
                    attacker_id, enlist_count, *used
                )));
            }
            // Check 4: enlisted creature is not attacking.
            if declared_attacker_ids.contains(enlisted_id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: creature {:?} is an attacker and cannot be enlisted",
                    enlisted_id
                )));
            }
            // Check 3: on battlefield, controlled by player.
            let enlisted_obj = state.object(*enlisted_id)?;
            if enlisted_obj.zone != ZoneId::Battlefield {
                return Err(GameStateError::ObjectNotOnBattlefield(*enlisted_id));
            }
            if enlisted_obj.controller != player {
                return Err(GameStateError::NotController {
                    player,
                    object_id: *enlisted_id,
                });
            }
            // Check 5: untapped.
            if enlisted_obj.status.tapped {
                return Err(GameStateError::PermanentAlreadyTapped(*enlisted_id));
            }
            // Check 6: is a creature.
            let enlisted_chars = calculate_characteristics(state, *enlisted_id)
                .ok_or(GameStateError::ObjectNotFound(*enlisted_id))?;
            if !enlisted_chars.card_types.contains(&CardType::Creature) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: object {:?} is not a creature",
                    enlisted_id
                )));
            }
            // Check 7: no summoning sickness (or has haste).
            let has_haste = enlisted_chars.keywords.contains(&KeywordAbility::Haste);
            let enlisted_obj_for_sickness = state.object(*enlisted_id)?;
            if enlisted_obj_for_sickness.has_summoning_sickness && !has_haste {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: creature {:?} has summoning sickness and no haste (CR 702.154a)",
                    enlisted_id
                )));
            }
            // Check 8: not already enlisted by another attacker.
            if enlisted_ids_used.contains(enlisted_id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Enlist: creature {:?} is already enlisted by another attacker \
                     (ruling 2022-09-09)",
                    enlisted_id
                )));
            }
            enlisted_ids_used.push(*enlisted_id);
        }
    }
    // ---- CR 701.43d / CR 508.1g: Validate exert choices ----
    //
    // Each exerted ObjectId must satisfy:
    //  1. It is a declared attacker.
    //  2. It has `KeywordAbility::Exert` (layer-aware check) -- the "you may exert [this
    //     creature] as it attacks" static ability (508.1g).
    //  3. It is NOT already `Designations::EXERTED` -- enforces "if this creature hasn't
    //     been exerted this turn" (card text; also structurally satisfies 701.43b's
    //     one-shot-per-untap-step model since a re-exert this turn would be a no-op).
    //  4. It is on the battlefield (701.43c) -- guaranteed here since it must already be
    //     a declared attacker.
    {
        let mut exert_ids_used: Vec<ObjectId> = Vec::new();
        for exerted_id in &exert_choices {
            if !declared_attacker_ids.contains(exerted_id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Exert: creature {:?} is not a declared attacker (CR 508.1g)",
                    exerted_id
                )));
            }
            let exert_chars = calculate_characteristics(state, *exerted_id)
                .ok_or(GameStateError::ObjectNotFound(*exerted_id))?;
            if !exert_chars.keywords.contains(&KeywordAbility::Exert) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Exert: attacker {:?} does not have the \"may exert as it attacks\" \
                     static ability (CR 701.43d)",
                    exerted_id
                )));
            }
            let exerted_obj = state.object(*exerted_id)?;
            if exerted_obj.zone != ZoneId::Battlefield {
                return Err(GameStateError::ObjectNotOnBattlefield(*exerted_id));
            }
            if exerted_obj.designations.contains(Designations::EXERTED) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Exert: creature {:?} has already been exerted this turn (CR 701.43a/b)",
                    exerted_id
                )));
            }
            if exert_ids_used.contains(exerted_id) {
                return Err(GameStateError::InvalidCommand(format!(
                    "Exert: creature {:?} appears more than once in exert_choices",
                    exerted_id
                )));
            }
            exert_ids_used.push(*exerted_id);
        }
    }
    // Initialize CombatState if not already set (may be set by BeginningOfCombat action).
    //
    // PB-DX51 (`OOS-DX21-5`, PB-DX21 review finding L2): this init used to run near the
    // top of the function, immediately after the CR 508.1 once-per-combat guard and
    // BEFORE the per-attacker validation loop, so a declaration refused for an illegal
    // attacker, an unmet CR 508.1c/d restriction, a bad enlist/exert choice or an
    // unaffordable CR 508.1h tax still left a fresh `CombatState` behind. It now stands
    // below EVERY `return Err` in this function, so a refused declaration leaves
    // `state.combat` exactly as it found it (CR 732: "the game returns to the moment
    // before the declaration"). Every line below this point is infallible.
    //
    // Behaviour-preserving and invisible through `process_command`, whose `Err` arm
    // carries no `GameState` and therefore discards every mutation by ownership
    // (`OOS-DX21-7`) -- so the pin is a DIRECT-handler probe, which is the only idiom
    // that can see it (`pb_dx51_combat_state_install::x1`).
    if state.combat.is_none() {
        state.combat = Some(CombatState::new(player));
    }
    let mut events = Vec::new();
    // Tap non-Vigilance attackers (CR 508.1f).
    // Uses pre-computed vigilance flags to avoid a redundant calculate_characteristics call.
    for (attacker_id, has_vigilance) in &attacker_vigilance {
        if !has_vigilance {
            if let Some(obj) = state.expect_object_mut(*attacker_id) {
                obj.status.tapped = true;
            }
            events.push(GameEvent::PermanentTapped {
                player,
                object_id: *attacker_id,
            });
        }
    }
    // CR 702.154a / CR 508.1j: Tap enlisted creatures as part of the
    // attack cost payment.
    for (_, enlisted_id) in &enlist_choices {
        if let Some(obj) = state.expect_object_mut(*enlisted_id) {
            obj.status.tapped = true;
        }
        events.push(GameEvent::PermanentTapped {
            player,
            object_id: *enlisted_id,
        });
    }
    // CR 508.1j: pay all attack costs. Partial payments are not allowed, and the
    // total was locked in during validation (CR 508.1h) before any state was
    // mutated, so this cannot fail. Placed after the CR 508.1f tapping loops to
    // match the CR's own step order (508.1f tap -> 508.1h total -> 508.1i mana
    // abilities -> 508.1j pay).
    //
    // CR 508.1i is NOT honoured: the engine determines and pays the total inside
    // a single DeclareAttackers command, so the player has no window to activate
    // mana abilities between the two. The tax must already be floating.
    // Pre-existing deviation, preserved (fixing it needs a two-phase declaration
    // = a new Command). Seed OOS-DP4-2.
    //
    // PB-DX6: pays `flat_total` (the CR 107.4e/107.4f-resolved cost validation
    // computed above) and deducts `phyrexian_life`, but the `ManaCostPaid` event
    // carries `attack_tax` -- the ORIGINAL, unflattened total -- mirroring
    // `casting.rs`/`abilities.rs`/`handle_turn_face_up`, which all emit the pipped
    // shape so event consumers see what was printed (CR 107.4e/107.4f).
    if let Some(tax) = &attack_tax {
        // Fix cycle (E6, PB-DP4): both events live INSIDE the `if let Some(ps)` --
        // a missing player must not produce an event describing a payment that
        // didn't happen (Architecture Invariant 4: events describe what happened,
        // not what was attempted).
        if let Some(ps) = state.expect_player_mut(player) {
            casting::pay_cost(&mut ps.mana_pool, &flat_total);
            // Architecture Invariant 4: a pool debit is a state change and must be
            // evented. Reuses the existing universal payment event -- no wire change.
            events.push(GameEvent::ManaCostPaid {
                player,
                cost: tax.clone(),
            });
            // CR 107.4f: pay life for any Phyrexian pip paid with life. A SIBLING
            // of the mana payment above, not nested inside a separate gate --
            // mirrors `handle_turn_face_up`'s identical site (plan §5.1/§5.2.3).
            // The `total != ManaCost::default()` guard above (evaluated on the
            // PIPPED total, plan §13 risk 9) is what lets an all-Phyrexian,
            // all-life cost_per_creature (flat_total == {0}) still reach this
            // block at all -- T11 pins it.
            if phyrexian_life > 0 {
                ps.life_total -= phyrexian_life as i32;
                events.push(GameEvent::LifeLost {
                    player,
                    amount: phyrexian_life,
                });
            }
        }
    }
    // SR-4 / review finding L1: `state.combat` is provably `Some` here -- the
    // guard above only fires against an already-`Some` `CombatState`, and
    // `:69-72`'s init runs unconditionally when it was `None`. Loud, not
    // silent, if a future edit ever breaks that: skipping the marker set below
    // would reopen the exact defect PB-DX21 closes.
    debug_assert!(
        state.combat.is_some(),
        "handle_declare_attackers: state.combat must be Some here (CR 508.1) -- \
         initialized above, and nothing between there and here clears it"
    );
    // Record attackers in combat state.
    if let Some(combat) = state.combat.as_mut() {
        for (attacker_id, target) in &attackers {
            // CR 508.1 / CR 508.8 (PB-DX51): the single mutator, which also sets
            // `had_attackers`. An EMPTY declaration never enters this loop, so the
            // marker stays clear and CR 508.8's skip still fires -- CR 508.1a's
            // "if any" makes the empty choice a completed declaration for
            // `attackers_declared` below and a non-declaration for CR 508.8.
            combat.add_attacker(*attacker_id, target.clone());
        }
        // CR 508.1 / 508.1a / 508.8 (PB-DX21): the turn-based action has now been
        // performed. Set on the SUCCESS path only -- every `return Err` above leaves
        // it clear, so a rejected declaration (an unaffordable CR 508.1h tax, an
        // illegal attacker, a goad violation) does NOT lock out a legal retry.
        // Set even when `attackers` is EMPTY: CR 508.1a's "if any" makes the empty
        // choice a completed declaration, and CR 508.8 defines the game's behaviour
        // for it. Mirrors `handle_declare_blockers`' `defenders_declared.insert`,
        // which is likewise inside this same shape.
        combat.attackers_declared = true;
    }
    // PB-AC6 / Raid, CR 508.1: mark that this player attacked this turn (one or more
    // attackers were declared). Only a declare-attackers action counts as "you
    // attacked" -- creatures put onto the battlefield already attacking (CR 508.4,
    // e.g. Ninjutsu, Aggravated Assault-style effects) do NOT set this flag
    // (Bloodsoaked Champion ruling).
    if !attackers.is_empty() {
        if let Some(ps) = state.expect_player_mut(player) {
            ps.attacked_this_turn = true;
            // PB-OS6(b) / PB-DX53 / CR 508.3d: capture the size of THIS declaration
            // for Condition::YouAttackedWithNOrMoreThisDeclaration. Only declared
            // attackers count; OVERWRITTEN (not accumulated) on MULTI-COMBAT turns
            // (CR 500.8 adds the phase and CR 506.1 gives each combat phase its own
            // declare-attackers step -- NOT CR 506.5, which defines "attacks alone";
            // e.g. `aurelia_the_warleader`'s extra combat phase) --
            // which is CORRECT for CR 508.3d, a per-declaration trigger gate
            // (Legion's Landing). PB-DX21 (`OOS-M11-9`) makes a second
            // `DeclareAttackers` in one combat phase an error
            // (`GameStateError::AlreadyDeclaredAttackers`, guarded earlier in this
            // function), so the only place this overwrites is across a
            // `BeginningOfCombat`-to-`BeginningOfCombat` boundary, which installs a
            // fresh `CombatState` -- exactly the boundary CR 508.3d's own trigger
            // re-fires at.
            ps.latest_attacker_declaration_size = attackers.len() as u32;
            // PB-DX53 / ruling 2007-10-01 (Windbrisk Heights): accumulate every
            // DECLARED attacker's ObjectId into the per-TURN set for
            // Condition::YouAttackedWithNOrMoreCreaturesThisTurn. Reads `attackers`
            // -- this function's own COMMAND parameter, the declared list -- and
            // NEVER `combat.attackers`, which also holds CR 508.4 entrants (PB-DX51
            // made `CombatState::add_attacker` the only production path into that
            // map, called above for token/Myriad/Ninjutsu entrants too). That is
            // what makes the CR 508.4 exclusion structural: an entrant is never a
            // parameter to THIS function, so it can never reach this insert.
            // `OrdSet::insert` on an ObjectId already present is a no-op, which is
            // the ruling's "counts only once" for a creature declared in two
            // different attack phases (CR 400.7 identity: a creature that left and
            // returned is a NEW object and is correctly counted again).
            for (attacker_id, _) in &attackers {
                ps.creatures_declared_as_attackers_this_turn
                    .insert(*attacker_id);
            }
        }
    }
    // CR 702.154a: Store enlist pairings for trigger collection in abilities.rs.
    if let Some(combat) = state.combat.as_mut() {
        combat.enlist_pairings = enlist_choices.clone();
    }
    // CR 701.43a/d: Set Designations::EXERTED on each exerted attacker and store the
    // exert choices in combat state for linked-trigger collection in abilities.rs.
    for exerted_id in &exert_choices {
        if let Some(obj) = state.expect_object_mut(*exerted_id) {
            obj.designations.insert(Designations::EXERTED);
        }
        events.push(GameEvent::PermanentExerted {
            object_id: *exerted_id,
        });
    }
    if let Some(combat) = state.combat.as_mut() {
        combat.exerted_attackers = exert_choices.iter().copied().collect();
    }
    // CR 702.147a: Tag creatures with decayed for EOC sacrifice.
    // "When this creature attacks, sacrifice it at end of combat."
    // Must be tagged here (when state is mutable) rather than in check_triggers
    // (which receives &GameState). The tag persists even if decayed is removed
    // later (ruling 2021-09-24: "Once a creature with decayed attacks, it will be
    // sacrificed at end of combat, even if it no longer has decayed at that time.").
    for (attacker_id, _) in &attackers {
        let has_decayed = calculate_characteristics(state, *attacker_id)
            .map(|c| c.keywords.contains(&KeywordAbility::Decayed))
            .unwrap_or(false);
        if has_decayed {
            if let Some(obj) = state.expect_object_mut(*attacker_id) {
                obj.decayed_sacrifice_at_eoc = true;
            }
        }
    }
    events.push(GameEvent::AttackersDeclared {
        attacking_player: player,
        attackers: attackers.clone(),
    });
    // Check and queue triggers from the attack declaration (e.g., SelfAttacks).
    // PB-DX15a (`OOS-DX24-7`): `Simultaneous` here is EXACTLY the pre-PB-DX15a
    // behaviour, not a new judgement. PB-DX24's fix cycle recorded this call site as
    // NOT AUDITED for CR 603.10a look-back granularity, and this batch did not audit
    // it either -- the parameter exists so that status is visible here instead of
    // buried in a comment in `abilities.rs`.
    let new_triggers = abilities::check_triggers_with_timing(
        state,
        &events,
        abilities::EventBatchTiming::Simultaneous,
    );
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    // Flush triggers before granting priority (CR 603.3).
    let trigger_events = abilities::flush_pending_triggers(state);
    events.extend(trigger_events);
    // CR 603.3 / CR 603.3d (PB-DP8): the batch suspended on a target choice.
    // CR 603.3b gives priority only AFTER every triggered ability of this
    // batch is on the stack, so stop here without granting it, and record
    // that this site owes the grant. `handle_choose_trigger_targets` resumes.
    if state.pending_trigger_targets.is_some() {
        abilities::mark_flush_resume_site(state, FlushResumeSite::GrantPriority);
        return Ok(events);
    }
    // CR 508.1 / CR 117.3a: declaring attackers is a turn-based action; the declaring
    // player is the active player (enforced at :46), so `Some(player)` is the active
    // player here. CR 117.4: reset the pass-round.
    state.turn.players_passed = OrdSet::new();
    state.turn.priority_holder = Some(player);
    events.push(GameEvent::PriorityGiven { player });
    Ok(events)
}
/// CR 508.1d: can a must-attack requirement actually force `creature` to attack?
///
/// "If a creature can't attack unless a player pays a cost, that player is not required to
/// pay that cost, even if attacking with that creature would increase the number of
/// requirements being obeyed." (CR 508.1d; Goblin Rabblemaster ruling 2014-07-18: "If
/// there's a cost associated with having a creature attack, you're not forced to pay that
/// cost.") So a requirement is only *obeyable* if some attack target exists that costs the
/// controller nothing.
///
/// Returns true iff at least one such target exists:
///   * a live opponent that is neither this creature's owner-under-CantAttackOwner
///     (CR 508.1c) nor a member of `taxed_defenders`, OR
///   * any opponent-controlled planeswalker on the battlefield -- attacking a planeswalker
///     is not "attacking you", so a CantAttackYouUnlessPay tax never applies to it
///     (CR 508.1c + the Propaganda ruling), and CantAttackOwner is about players.
///
/// Generalises the two hand-copied `no_legal_target` computations PB-DP4 replaced
/// (`handle_declare_attackers`, the goad block and the MustAttackEachCombat block).
/// PB-DP4 / DP-10, closes OOS-RS3-4.
fn has_uncosted_attack_target(
    state: &GameState,
    player: PlayerId,
    creature: ObjectId,
    taxed_defenders: &BTreeSet<PlayerId>,
) -> bool {
    let has_cant_attack_owner = state.restrictions.iter().any(|r| {
        r.source == creature
            && matches!(r.restriction, GameRestriction::CantAttackOwner)
            && state
                .objects
                .get(&r.source)
                .map(|o| o.zone == ZoneId::Battlefield)
                .unwrap_or(false)
    });
    let owner = state.expect_object(creature).map(|o| o.owner);
    let has_live_opponent_target = state.players.keys().any(|pid| {
        *pid != player
            && (!has_cant_attack_owner || Some(*pid) != owner)
            && !taxed_defenders.contains(pid)
            && state
                .expect_player(*pid)
                .map(|p| !p.has_lost && !p.has_conceded)
                .unwrap_or(false)
    });
    if has_live_opponent_target {
        return true;
    }
    // Any opponent-controlled planeswalker is an uncosted target: CantAttackYouUnlessPay
    // is scoped to "attack you" (a player), so it never applies to a planeswalker attack,
    // and CantAttackOwner is likewise scoped to the owning player. Layer-resolved
    // (W3-LC contract, CR 613.1f): an animated planeswalker or a Humility'd one must be
    // judged on its CURRENT types, never `obj.characteristics` directly.
    state.objects.values().any(|o| {
        o.zone == ZoneId::Battlefield
            && o.controller != player
            && crate::rules::layers::expect_characteristics(state, o.id)
                .card_types
                .contains(&CardType::Planeswalker)
    })
}
/// CR 508.1h (+ CR 107.4e/107.4f via the Norn's Annex "individually" rulings, quoted
/// in full at `memory/primitives/pb-plan-DX6.md` §1): accumulate `times` full copies
/// of `addend` into `total`, replicating its hybrid/Phyrexian pips along with its
/// numeric fields.
///
/// Attack taxes from multiple sources are cumulative (Propaganda ruling), and a
/// defender's total is `cost_per_creature` x the number of creatures attacking that
/// defender. `accumulate_attack_tax_total` (below) uses this function for BOTH halves
/// of that accumulation -- summing multiple restrictions into one defender's
/// per-creature cost (`times: 1` per restriction, in `state.restrictions` order),
/// THEN multiplying that per-creature cost by the attacker count (`times:
/// attacker_count`) -- which is exactly what makes the result **copy-major**
/// (`accumulate_attack_tax_total`'s own doc has the full contract): `times` full,
/// intact copies of `addend`'s pip list are appended in `addend`'s own order, never
/// each individual pip repeated `times` times on its own. That other shape is
/// **pip-major**, and is what `multiply_mana_cost` in `rules/engine.rs` does for
/// cumulative upkeep -- see that function's own doc for why the two must NOT be
/// merged (OOS-DP4-7, re-dispositioned by this batch, not closed).
///
/// X is rejected upstream of every call site (`Command::DeclareAttackers` has no
/// X-announcement channel -- CR 107.3/601.2b, OOS-DX6-1), so `addend.x_count` is
/// asserted zero here as an unreachable-by-construction tripwire, not handled.
fn add_mana_cost(total: &mut ManaCost, addend: &ManaCost, times: u32) {
    debug_assert!(
        addend.x_count == 0,
        "X attack tax reached add_mana_cost -- X has no announcement channel on \
         Command::DeclareAttackers and must be rejected before this point \
         (CR 107.3/601.2b, OOS-DX6-1): {addend:?}"
    );
    total.white += addend.white * times;
    total.blue += addend.blue * times;
    total.black += addend.black * times;
    total.red += addend.red * times;
    total.green += addend.green * times;
    total.colorless += addend.colorless * times;
    total.generic += addend.generic * times;
    for _ in 0..times {
        total.hybrid.extend(addend.hybrid.iter().cloned());
        total.phyrexian.extend(addend.phyrexian.iter().cloned());
    }
}
/// CR 508.1h: the unflattened attack-tax total for a candidate `attackers` set --
/// **the single, canonical accumulation**, shared by `handle_declare_attackers`'s own
/// validation and `queries::attack_tax_total` (read-only advisory), so the pip order
/// is defined in exactly one place. Two copies of this order is how OOS-RS2-1 /
/// OOS-DP4-1 happened in the first place (plan §5.3).
///
/// **The canonical pip order** (of `total.hybrid`, and independently of
/// `total.phyrexian` -- this is the contract `hybrid_choices`/`phyrexian_life_payments`
/// on `Command::DeclareAttackers` index against): defenders ascending by `PlayerId`
/// (the `BTreeMap` iteration this function relies on for SR-9b determinism) ->
/// within a defender, one complete copy of that defender's per-creature cost (itself
/// the concatenation of every `CantAttackYouUnlessPay` restriction against that
/// defender, in `state.restrictions` order -- Propaganda ruling: costs from multiple
/// sources are cumulative) per creature attacking that defender -> within a copy,
/// restrictions in `state.restrictions` iteration order. This is **copy-major**, not
/// pip-major: for a defender with per-creature pips `[r1, r2]` and 3 attackers the
/// result's hybrid vec is `[r1, r2, r1, r2, r1, r2]`, NOT `[r1, r1, r1, r2, r2, r2]`
/// (see `add_mana_cost`'s own doc for the mechanism, and why `multiply_mana_cost`
/// must not be reused here -- OOS-DP4-7). Creatures are indistinguishable for cost
/// purposes -- the order does not depend on WHICH creature is which, and no
/// attacker -> offset mapping is promised.
///
/// Restrictions whose `cost_per_creature` carries an `x_count > 0` pip are SKIPPED
/// here -- X has no announcement channel on `Command::DeclareAttackers`
/// (CR 107.3/601.2b, OOS-DX6-1) and its rejection is `handle_declare_attackers`'s own
/// responsibility (the `x_tax_defenders` bookkeeping there), not this function's. It
/// is the *restriction*, not the whole defender, that is skipped: a defender carrying
/// both an X restriction and a plain (non-X) restriction still contributes the plain
/// restriction's cost to the total; only a defender whose ONLY restriction carries an
/// X contributes nothing here. A `{0}` restriction (CR 118.5, PB-DP4's E7 fix) is
/// likewise skipped at the restriction level -- unconditionally payable, contributes
/// nothing.
///
/// Returns `ManaCost::default()` (never wrapped in `Option`) when no tax applies;
/// callers convert to `Option` at their own boundary (`queries::attack_tax_total`
/// does; `handle_declare_attackers` keeps working with the bare value).
pub(crate) fn accumulate_attack_tax_total(
    state: &GameState,
    attackers: &[(ObjectId, AttackTarget)],
) -> ManaCost {
    // CR 508.1h: per-defender per-creature cost, as a real ManaCost (not a u32).
    // BTreeMap, not HashMap: iteration order feeds the summed cost, and SR-9b
    // requires determinism.
    let mut tax_per_creature: BTreeMap<PlayerId, ManaCost> = BTreeMap::new();
    for restriction in state.restrictions.iter() {
        // Skip if source is no longer on the battlefield.
        let source_on_bf = state
            .objects
            .get(&restriction.source)
            .map(|o| matches!(o.zone, ZoneId::Battlefield))
            .unwrap_or(false);
        if !source_on_bf {
            continue;
        }
        if let GameRestriction::CantAttackYouUnlessPay { cost_per_creature } =
            &restriction.restriction
        {
            // X is not accumulated here -- see doc above; the caller's own
            // restriction scan handles X rejection.
            if cost_per_creature.x_count > 0 {
                continue;
            }
            // CR 118.5: a {0} restriction contributes nothing.
            if *cost_per_creature == ManaCost::default() {
                continue;
            }
            let entry = tax_per_creature.entry(restriction.controller).or_default();
            add_mana_cost(entry, cost_per_creature, 1);
        }
    }
    // CR 508.1c: attackers per taxed defender. Only player-attacks are taxed. A
    // creature attacking a planeswalker is attacking that planeswalker, not its
    // controller, so Propaganda does not apply (CR 508.1c + the Propaganda ruling).
    let mut attackers_per_player: BTreeMap<PlayerId, u32> = BTreeMap::new();
    for (_, target) in attackers {
        if let AttackTarget::Player(defending_pid) = target {
            if tax_per_creature.contains_key(defending_pid) {
                *attackers_per_player.entry(*defending_pid).or_insert(0) += 1;
            }
        }
    }
    // CR 508.1h: total cost, defenders visited ascending by PlayerId (the
    // `BTreeMap` iteration order) -- see this function's own doc for why that is
    // the canonical order's outer loop.
    let mut total = ManaCost::default();
    for (defending_pid, attacker_count) in &attackers_per_player {
        if let Some(cost_per) = tax_per_creature.get(defending_pid) {
            add_mana_cost(&mut total, cost_per, *attacker_count);
        }
    }
    total
}
/// CR 506.4: Remove a permanent from combat.
///
/// "A permanent is removed from combat if ... an effect specifically removes it
/// from combat ... A creature that's removed from combat stops being an attacking,
/// blocking, blocked, and/or unblocked creature." This does NOT untap the
/// permanent (CR 506.4b) -- callers that need "untap and remove from combat"
/// (e.g. Spires of Orazca / thaumatic_compass) must pair this with
/// `Effect::UntapPermanent` via `Effect::Sequence`.
///
/// Clears the object from `combat.attackers`, `combat.blockers`,
/// `combat.blocked_attackers`, and every `combat.damage_assignment_order` slot
/// (both as an attacker key and as an entry in any blocker list). Returns
/// whether anything was actually removed (i.e. the object was in combat at all).
///
/// Factored out of `apply_regeneration` (PB-OS6(g)) — that function's step 3 is a
/// behavior-identical caller of this helper.
pub(crate) fn remove_from_combat(state: &mut GameState, object_id: ObjectId) -> bool {
    let Some(combat) = state.combat.as_mut() else {
        return false;
    };
    let mut removed = false;
    if combat.attackers.remove(&object_id).is_some() {
        removed = true;
    }
    if combat.blockers.remove(&object_id).is_some() {
        removed = true;
    }
    if combat.blocked_attackers.remove(&object_id).is_some() {
        removed = true;
    }
    if combat.damage_assignment_order.remove(&object_id).is_some() {
        removed = true;
    }
    // imbl::OrdMap has no iter_mut, so rebuild while stripping object_id from any
    // blocker list.
    let mut any_blocker_list_changed = false;
    let updated: OrdMap<_, _> = combat
        .damage_assignment_order
        .iter()
        .map(|(attacker_id, order)| {
            let before_len = order.len();
            let filtered: Vec<_> = order
                .iter()
                .filter(|&&blocker| blocker != object_id)
                .copied()
                .collect();
            if filtered.len() != before_len {
                any_blocker_list_changed = true;
            }
            (*attacker_id, filtered)
        })
        .collect();
    combat.damage_assignment_order = updated;
    removed || any_blocker_list_changed
}
// ---------------------------------------------------------------------------
// Declare Blockers
// ---------------------------------------------------------------------------
/// CR 509.1a-c: may `blocker` legally be declared blocking `attacker`, for the
/// declaring `player`?
///
/// PB-DX55 (`OOS-SIM5-3`): this is the ONE per-pair restriction predicate. Before this
/// batch the engine held TWO hand-rolled copies inside `handle_declare_blockers` — the
/// per-pair loop below and the CR 702.39a provoke requirement's `continue`-shaped
/// satisfiability mirror — and the two were NOT identical: the provoke mirror omitted
/// the phased-out check, `CrossPlayerBlock`, and the within-batch/committed duplicate
/// check. Both callers now go through this one function, so a provoked creature's
/// requirement is judged "impossible" by the SAME rule a real declaration is validated
/// against, closing that divergence rather than merely re-describing it.
///
/// `already_blocking` names blocker ids already committed to a block within the SAME
/// batch being validated (`validate_block_declaration`'s within-declaration duplicate
/// check); an empty slice is correct for a standalone legality query (`queries::
/// legal_blocks`) or for the provoke satisfiability check, both of which only care
/// whether `blocker` is free to block `attacker` right now, independent of what else
/// this same submission might assign it to. Either way, a blocker already committed in
/// `state.combat.blockers` (a PRIOR player's declaration, or an already-accepted one of
/// this player's own) is always consulted directly from state, regardless of what is
/// passed here.
pub fn check_block_pair(
    state: &GameState,
    player: PlayerId,
    blocker_id: ObjectId,
    attacker_id: ObjectId,
    already_blocking: &[ObjectId],
) -> Result<(), GameStateError> {
    let obj = state.object(blocker_id)?;
    if obj.zone != ZoneId::Battlefield {
        return Err(GameStateError::ObjectNotOnBattlefield(blocker_id));
    }
    // CR 702.26b: Phased-out permanents cannot block.
    if obj.status.phased_out {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} is phased out and cannot block",
            blocker_id
        )));
    }
    if obj.controller != player {
        return Err(GameStateError::NotController {
            player,
            object_id: blocker_id,
        });
    }
    if obj.status.tapped {
        return Err(GameStateError::PermanentAlreadyTapped(blocker_id));
    }
    // Must be a creature.
    let blocker_chars = calculate_characteristics(state, blocker_id)
        .ok_or(GameStateError::ObjectNotFound(blocker_id))?;
    if !blocker_chars.card_types.contains(&CardType::Creature) {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} is not a creature",
            blocker_id
        )));
    }
    // CR 702.147a: A creature with decayed can't block.
    if blocker_chars.keywords.contains(&KeywordAbility::Decayed) {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} has decayed and cannot block (CR 702.147a)",
            blocker_id
        )));
    }
    // CR 509.1b: A creature with CantBlock can't block.
    if blocker_chars.keywords.contains(&KeywordAbility::CantBlock) {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} has CantBlock and cannot block (CR 509.1b)",
            blocker_id
        )));
    }
    // CR 701.60c: A suspected permanent has "This creature can't block."
    // Checked on the raw GameObject (like Decayed) so the restriction persists
    // even under ability-removal effects (Humility strips the Menace grant but
    // the designation and can't-block restriction remain).
    // TODO: Under true Humility, can't-block should also be removed; this is a
    // known minor inaccuracy deferred to a future session.
    if obj.designations.contains(Designations::SUSPECTED) {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} is suspected and cannot block (CR 701.60c)",
            blocker_id
        )));
    }
    // MR-M6-02: a creature can only block one attacker.
    // Check both existing combat.blockers and within-this-declaration duplicates.
    if already_blocking.contains(&blocker_id)
        || state
            .combat
            .as_ref()
            .map(|c| c.blockers.contains_key(&blocker_id))
            .unwrap_or(false)
    {
        return Err(GameStateError::DuplicateBlocker(blocker_id));
    }
    // CR 509.1a: a defending player can only block attackers that are attacking them,
    // a planeswalker they control, or a battle they protect. (Corrected cite: this used
    // to say CR 509.1c, which is the requirements-maximization rule for provoke, not
    // this restriction -- PB-DX55, `OOS-SIM5-3`.)
    // Also validates that the attacker is a declared attacker.
    let attacker_target = state
        .combat
        .as_ref()
        .and_then(|c| c.attackers.get(&attacker_id).cloned());
    match attacker_target {
        None => {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} is not a declared attacker",
                attacker_id
            )));
        }
        Some(AttackTarget::Player(pid)) if pid == player => {
            // Valid: this attacker is targeting the declaring player directly.
        }
        Some(AttackTarget::Planeswalker(pw_id)) => {
            // Valid only if the planeswalker is controlled by the declaring player.
            let pw_controller = state.objects.get(&pw_id).map(|o| o.controller);
            if pw_controller != Some(player) {
                return Err(GameStateError::CrossPlayerBlock {
                    blocker: blocker_id,
                    attacker: attacker_id,
                });
            }
        }
        Some(_) => {
            return Err(GameStateError::CrossPlayerBlock {
                blocker: blocker_id,
                attacker: attacker_id,
            });
        }
    }
    // CR 509.1b / CR 702.9a: A creature without flying or reach cannot block
    // a creature with flying.
    let attacker_chars = calculate_characteristics(state, attacker_id)
        .ok_or(GameStateError::ObjectNotFound(attacker_id))?;
    let attacker_has_flying = attacker_chars.keywords.contains(&KeywordAbility::Flying);
    let blocker_has_flying = blocker_chars.keywords.contains(&KeywordAbility::Flying);
    let blocker_has_reach = blocker_chars.keywords.contains(&KeywordAbility::Reach);
    if attacker_has_flying && !blocker_has_flying && !blocker_has_reach {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} cannot block {:?} (attacker has flying, blocker has neither flying nor reach)",
            blocker_id, attacker_id
        )));
    }
    // CR 509.1 / KeywordAbility::CantBeBlocked: a creature with this keyword
    // cannot be blocked at all. Applied by Rogue's Passage activated ability.
    if attacker_chars
        .keywords
        .contains(&KeywordAbility::CantBeBlocked)
    {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} cannot be blocked (CantBeBlocked keyword)",
            attacker_id
        )));
    }
    // CR 509.1b: CantBeBlockedExceptBy — the attacker can only be blocked by creatures
    // matching the exception filter. Each filter arm specifies what qualifies.
    for kw in attacker_chars.keywords.iter() {
        if let KeywordAbility::CantBeBlockedExceptBy(filter) = kw {
            let blocker_matches = match filter {
                BlockingExceptionFilter::HasKeyword(required_kw) => {
                    blocker_chars.keywords.contains(required_kw.as_ref())
                }
                BlockingExceptionFilter::HasAnyKeyword(required_kws) => required_kws
                    .iter()
                    .any(|k| blocker_chars.keywords.contains(k)),
            };
            if !blocker_matches {
                return Err(GameStateError::InvalidCommand(format!(
                    "Object {:?} cannot block {:?} (attacker has CantBeBlockedExceptBy; \
                     blocker does not match filter {:?})",
                    blocker_id, attacker_id, filter
                )));
            }
        }
    }
    // CR 702.13b: A creature with intimidate can't be blocked except by artifact creatures
    // and/or creatures that share a color with it.
    if attacker_chars
        .keywords
        .contains(&KeywordAbility::Intimidate)
    {
        let blocker_is_artifact_creature = blocker_chars.card_types.contains(&CardType::Artifact)
            && blocker_chars.card_types.contains(&CardType::Creature);
        let shares_a_color = attacker_chars
            .colors
            .iter()
            .any(|c| blocker_chars.colors.contains(c));
        if !blocker_is_artifact_creature && !shares_a_color {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot block {:?} (attacker has intimidate; \
                 blocker is neither an artifact creature nor shares a color)",
                blocker_id, attacker_id
            )));
        }
    }
    // CR 702.36b: A creature with fear can't be blocked except by artifact creatures
    // and/or black creatures.
    if attacker_chars.keywords.contains(&KeywordAbility::Fear) {
        let blocker_is_artifact_creature = blocker_chars.card_types.contains(&CardType::Artifact)
            && blocker_chars.card_types.contains(&CardType::Creature);
        let blocker_is_black = blocker_chars.colors.contains(&Color::Black);
        if !blocker_is_artifact_creature && !blocker_is_black {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot block {:?} (attacker has fear; \
                 blocker is neither an artifact creature nor black)",
                blocker_id, attacker_id
            )));
        }
    }
    // CR 702.28b: Shadow is a bidirectional evasion ability.
    // A creature with shadow can't be blocked by creatures without shadow,
    // and a creature without shadow can't be blocked by creatures with shadow.
    let attacker_has_shadow = attacker_chars.keywords.contains(&KeywordAbility::Shadow);
    let blocker_has_shadow = blocker_chars.keywords.contains(&KeywordAbility::Shadow);
    if attacker_has_shadow != blocker_has_shadow {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} cannot block {:?} (shadow mismatch: attacker shadow={}, blocker shadow={})",
            blocker_id, attacker_id, attacker_has_shadow, blocker_has_shadow
        )));
    }
    // CR 702.31b: Horsemanship is a unidirectional evasion ability.
    // A creature with horsemanship can't be blocked by creatures without horsemanship.
    // Unlike Shadow, a creature with horsemanship CAN block creatures without horsemanship.
    if attacker_chars
        .keywords
        .contains(&KeywordAbility::Horsemanship)
        && !blocker_chars
            .keywords
            .contains(&KeywordAbility::Horsemanship)
    {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} cannot block {:?} (attacker has horsemanship; \
             blocker does not have horsemanship)",
            blocker_id, attacker_id
        )));
    }
    // CR 702.118b: Skulk -- a creature with skulk can't be blocked by creatures
    // with greater power. Unlike Shadow, this is one-directional: it only restricts
    // what can block the skulk creature, not what the skulk creature can block.
    // Equal power IS allowed to block (strictly greater than, not greater-or-equal).
    if attacker_chars.keywords.contains(&KeywordAbility::Skulk) {
        let attacker_power = attacker_chars.power.unwrap_or(0);
        let blocker_power = blocker_chars.power.unwrap_or(0);
        if blocker_power > attacker_power {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot block {:?} (attacker has skulk with power {}; \
                 blocker has greater power {})",
                blocker_id, attacker_id, attacker_power, blocker_power
            )));
        }
    }
    // CR 701.54c (ring level >= 1): Ring-bearer can't be blocked by creatures with
    // greater power. Identical to Skulk's restriction, but triggered by the RING_BEARER
    // designation rather than a keyword ability.
    if let Some(attacker_obj) = state.expect_object(attacker_id) {
        if attacker_obj
            .designations
            .contains(crate::state::game_object::Designations::RING_BEARER)
        {
            let controller = attacker_obj.controller;
            if let Some(ps) = state.expect_player(controller) {
                if ps.ring_level >= 1 {
                    let attacker_power = attacker_chars.power.unwrap_or(0);
                    let blocker_power = blocker_chars.power.unwrap_or(0);
                    if blocker_power > attacker_power {
                        return Err(GameStateError::InvalidCommand(format!(
                            "Object {:?} cannot block ring-bearer {:?} \
                             (blocker power {} > ring-bearer power {}, CR 701.54c)",
                            blocker_id, attacker_id, blocker_power, attacker_power
                        )));
                    }
                }
            }
        }
    }
    // CR 702.16f: protection from blocking. A creature with protection from a quality
    // cannot be blocked by creatures that match that quality. The blocker is the source.
    let blocker_controller = state.objects.get(&blocker_id).map(|o| o.controller);
    if !super::protection::can_block(&attacker_chars.keywords, &blocker_chars, blocker_controller) {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} cannot block {:?} (attacker has protection from the blocker)",
            blocker_id, attacker_id
        )));
    }
    // CR 702.14c: A creature with landwalk can't be blocked as long as the defending
    // player controls at least one land with the specified type. Uses
    // `calculate_characteristics` to get post-layer subtypes (handles Blood Moon, etc.).
    for kw in attacker_chars.keywords.iter() {
        if let KeywordAbility::Landwalk(lw_type) = kw {
            let defender_has_matching_land = state.objects.values().any(|obj| {
                obj.zone == ZoneId::Battlefield && obj.controller == player && {
                    let chars = crate::rules::layers::expect_characteristics(state, obj.id);
                    chars.card_types.contains(&CardType::Land)
                        && match lw_type {
                            LandwalkType::BasicType(st) => chars.subtypes.contains(st),
                            LandwalkType::Nonbasic => !chars.supertypes.contains(&SuperType::Basic),
                        }
                }
            });
            if defender_has_matching_land {
                return Err(GameStateError::InvalidCommand(format!(
                    "Object {:?} cannot block {:?} (attacker has {:?} landwalk; \
                     defending player controls a matching land)",
                    blocker_id, attacker_id, lw_type
                )));
            }
        }
    }
    Ok(())
}

/// CR 509.1a-c / CR 702.110a / CR 702.39a: the WHOLE declaration -- 4 preamble guards,
/// [`check_block_pair`] over every pair, then the two batch guards (menace, provoke).
///
/// `handle_declare_blockers` calls this before mutating anything, so a rejected
/// declaration touches no state. Advisory callers (`queries::legal_blocks`,
/// `queries::check_block_pair`) can also call [`check_block_pair`] directly for a
/// single pair without going through the preamble/batch guards here.
pub fn validate_block_declaration(
    state: &GameState,
    player: PlayerId,
    blockers: &[(ObjectId, ObjectId)],
) -> Result<(), GameStateError> {
    // Must be in the DeclareBlockers step.
    if state.turn.step != Step::DeclareBlockers {
        return Err(GameStateError::InvalidCommand(
            "DeclareBlockers is only valid in the DeclareBlockers step".into(),
        ));
    }
    // Must not be the attacking player and must not have already declared blockers.
    {
        let combat = state
            .combat
            .as_ref()
            .ok_or_else(|| GameStateError::InvalidCommand("No active combat".into()))?;
        if player == combat.attacking_player {
            return Err(GameStateError::InvalidCommand(
                "The attacking player cannot declare blockers".into(),
            ));
        }
        // MR-M6-10: each defending player may only declare blockers once per combat step
        // (CR 509.1a — each defending player declares independently, not repeatedly).
        if combat.defenders_declared.contains(&player) {
            return Err(GameStateError::AlreadyDeclaredBlockers(player));
        }
    }
    // Track blocker IDs seen in this declaration to catch within-batch duplicates.
    let mut seen_blocker_ids: Vec<ObjectId> = Vec::with_capacity(blockers.len());
    // Validate each blocker against the ONE per-pair predicate.
    for (blocker_id, attacker_id) in blockers {
        check_block_pair(state, player, *blocker_id, *attacker_id, &seen_blocker_ids)?;
        seen_blocker_ids.push(*blocker_id);
    }
    // CR 702.110a: A creature with menace can't be blocked except by two or more creatures.
    // Check that no attacker with menace is being blocked by only one creature.
    {
        // Count how many blockers each attacker in this declaration has (summing over all declarations so far + this one).
        // `BTreeMap`, not `HashMap` (PB-DP9 fix-cycle Finding 4's widened
        // audit): the loop below returns on the FIRST menace violation it finds,
        // so with two offending attackers the `ObjectId` named in the error
        // message depended on iteration order. The accept/reject decision never
        // did, which is why this is hygiene rather than a correctness fix.
        use std::collections::BTreeMap;
        let mut blocker_count_for_attacker: BTreeMap<ObjectId, usize> = BTreeMap::new();
        // Existing blockers already recorded in combat state.
        if let Some(combat) = state.combat.as_ref() {
            for (_, &att) in &combat.blockers {
                *blocker_count_for_attacker.entry(att).or_insert(0) += 1;
            }
        }
        // New blockers being declared now.
        for (_, attacker_id) in blockers {
            *blocker_count_for_attacker.entry(*attacker_id).or_insert(0) += 1;
        }
        for (attacker_id, count) in &blocker_count_for_attacker {
            if *count == 1 {
                let chars = calculate_characteristics(state, *attacker_id)
                    .ok_or(GameStateError::ObjectNotFound(*attacker_id))?;
                if chars.keywords.contains(&KeywordAbility::Menace) {
                    return Err(GameStateError::InvalidCommand(format!(
                        "Object {:?} has menace and must be blocked by two or more creatures",
                        attacker_id
                    )));
                }
            }
        }
    }
    // CR 702.39a / CR 509.1c: Provoke forced-block requirements.
    //
    // Each provoked creature must block its provoking attacker if able. "If able" is
    // now decided by [`check_block_pair`] itself (an empty `already_blocking`: whether
    // this provoked creature is used elsewhere in THIS batch is a controller CHOICE,
    // not an impossibility, and must not make the requirement vanish). If the pairing
    // is legal and the creature is NOT in the blocker list blocking its provoking
    // attacker, the declaration is illegal (CR 509.1c -- must maximize obeyed
    // requirements without violating restrictions).
    {
        // Collect forced-block entries for this player (immutable borrow scope).
        let forced: Vec<(ObjectId, ObjectId)> = state
            .combat
            .as_ref()
            .map(|c| c.forced_blocks.iter().map(|(&k, &v)| (k, v)).collect())
            .unwrap_or_default();
        for (provoked_id, must_block_attacker) in forced {
            if check_block_pair(state, player, provoked_id, must_block_attacker, &[]).is_err() {
                // Not this player's creature, not on the battlefield, tapped, no
                // longer a declared attacker, or blocked by any per-pair restriction
                // (evasion, protection, phased-out, ...) -- the requirement is
                // impossible to satisfy, so it imposes no obligation (CR 509.1c).
                continue;
            }
            // The provoked creature CAN block the provoking attacker.
            // Check if it IS blocking it in this declaration.
            let is_blocking_required_attacker = blockers
                .iter()
                .any(|(b, a)| *b == provoked_id && *a == must_block_attacker);
            if !is_blocking_required_attacker {
                return Err(GameStateError::InvalidCommand(format!(
                    "Creature {:?} must block {:?} (provoke requirement, CR 702.39a / CR 509.1c)",
                    provoked_id, must_block_attacker
                )));
            }
        }
    }
    Ok(())
}

/// Handle a DeclareBlockers command (CR 509.1).
///
/// Any defending player may declare blockers during the DeclareBlockers step.
/// Priority is not required — this is a turn-based action for defending players.
/// Multiple defending players each declare independently (CR 509.1a).
pub fn handle_declare_blockers(
    state: &mut GameState,
    player: PlayerId,
    blockers: Vec<(ObjectId, ObjectId)>,
) -> Result<Vec<GameEvent>, GameStateError> {
    validate_block_declaration(state, player, &blockers)?;
    let mut events = Vec::new();
    // Record blockers in combat state.
    if let Some(combat) = state.combat.as_mut() {
        for (blocker_id, attacker_id) in &blockers {
            combat.blockers.insert(*blocker_id, *attacker_id);
            // CR 509.1h: track which attackers were declared blocked; this set is
            // never cleared even if blockers die, so is_blocked() remains correct.
            combat.blocked_attackers.insert(*attacker_id);
        }
        combat.defenders_declared.insert(player);
    }
    // CR 701.54c (ring level >= 3): Tag blockers of the ring-bearer for EOC sacrifice.
    // "Whenever your Ring-bearer becomes blocked by a creature, that creature's controller
    // sacrifices it at end of combat."
    //
    // We tag the blocker with `ring_block_sacrifice_at_eoc = true` here (in mutable context)
    // rather than emitting a RingBlockSacrifice PendingTrigger, because:
    //   1. The sacrifice must target the specific blocker (not a generic SacrificePermanents).
    //   2. The sacrifice must happen at end of combat (not when the trigger resolves).
    // This mirrors the Decayed EOC pattern (CR 702.147a) in handle_declare_attackers.
    //
    // TODO(M10+): Per CR 603.7, this is technically a delayed triggered ability. The current
    // TBA approach applies the sacrifice with no interaction window (can't Stifle). Refactor
    // when delayed trigger infrastructure is expanded.
    for (blocker_id, attacker_id) in &blockers {
        let (is_ring_bearer, ring_level) = {
            let obj = state.expect_object(*attacker_id);
            match obj {
                Some(o) => {
                    let bearer = o
                        .designations
                        .contains(crate::state::game_object::Designations::RING_BEARER);
                    let ctrl = o.controller;
                    let lvl = state
                        .expect_player(ctrl)
                        .map(|ps| ps.ring_level)
                        .unwrap_or(0);
                    (bearer, lvl)
                }
                None => (false, 0),
            }
        };
        if is_ring_bearer && ring_level >= 3 {
            if let Some(obj) = state.expect_object_mut(*blocker_id) {
                obj.ring_block_sacrifice_at_eoc = true;
            }
        }
    }
    // Always emit BlockersDeclared (even for empty declarations, to mark player done).
    events.push(GameEvent::BlockersDeclared {
        defending_player: player,
        blockers: blockers.clone(),
    });
    // Check and queue triggers from blocker declaration (e.g., SelfBlocks, Flanking).
    // PB-DX15a (`OOS-DX24-7`): `Simultaneous` here is EXACTLY the pre-PB-DX15a
    // behaviour, not a new judgement. PB-DX24's fix cycle recorded this call site as
    // NOT AUDITED for CR 603.10a look-back granularity, and this batch did not audit
    // it either -- the parameter exists so that status is visible here instead of
    // buried in a comment in `abilities.rs`.
    let new_triggers = abilities::check_triggers_with_timing(
        state,
        &events,
        abilities::EventBatchTiming::Simultaneous,
    );
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    // CR 603.3 / CR 509.3f: Flush any pending triggers (e.g., Flanking CR 702.25a,
    // SelfBlocks) so they appear on the stack before priority is granted.
    // This ensures triggered abilities from blocker declaration (like Flanking's -1/-1)
    // resolve BEFORE combat damage is dealt, which is correct per MTG rules.
    let trigger_events = abilities::flush_pending_triggers(state);
    events.extend(trigger_events);
    // CR 603.3 / CR 603.3d (PB-DP8): the batch suspended on a target choice.
    // CR 603.3b gives priority only AFTER every triggered ability of this
    // batch is on the stack, so stop here without granting it, and record
    // that this site owes the grant. `handle_choose_trigger_targets` resumes.
    if state.pending_trigger_targets.is_some() {
        abilities::mark_flush_resume_site(state, FlushResumeSite::GrantPriority);
        return Ok(events);
    }
    // CR 509.1: declaring blockers is a turn-based action; CR 117.3a gives the ACTIVE
    // player priority after it -- not the defending player who issued the command.
    // Grant priority to the active player so players can respond to triggers
    // (including Flanking triggers) before combat damage is dealt.
    //
    // CR 800.4j (second-closing-review HIGH-1): unless the active player has left
    // the game, in which case the next player in turn order receives it. This tail
    // used to write `Some(state.turn.active_player)` unconditionally, and an active
    // player eliminated during its own combat phase was handed priority back --
    // an unrecoverable deadlock, reachable with no PB-DP9 machinery on the path.
    // Probe: `test_509_declare_blockers_grant_skips_a_departed_active_player`.
    crate::rules::priority::grant_priority_to_active_player(state, &mut events);
    Ok(events)
}
// ---------------------------------------------------------------------------
// Order Blockers
// ---------------------------------------------------------------------------
/// Handle an OrderBlockers command (CR 509.2).
///
/// When an attacker has multiple blockers, its controller declares the order
/// in which damage is assigned. `order` is the blocker ObjectIds from front
/// (receives damage first) to back.
pub fn handle_order_blockers(
    state: &mut GameState,
    player: PlayerId,
    attacker: ObjectId,
    order: Vec<ObjectId>,
) -> Result<Vec<GameEvent>, GameStateError> {
    if state.turn.step != Step::DeclareBlockers {
        return Err(GameStateError::InvalidCommand(
            "OrderBlockers is only valid during the DeclareBlockers step".into(),
        ));
    }
    // Must be the attacking player.
    let combat = state
        .combat
        .as_ref()
        .ok_or_else(|| GameStateError::InvalidCommand("No active combat".into()))?;
    if player != combat.attacking_player {
        return Err(GameStateError::InvalidCommand(
            "Only the attacking player can order blockers".into(),
        ));
    }
    // Attacker must be a declared attacker.
    if !combat.attackers.contains_key(&attacker) {
        return Err(GameStateError::InvalidCommand(format!(
            "Object {:?} is not a declared attacker",
            attacker
        )));
    }
    // Validate all ordered blockers are actually blocking this attacker.
    let blocking_this: Vec<ObjectId> = combat
        .blockers
        .iter()
        .filter(|(_, &a)| a == attacker)
        .map(|(&b, _)| b)
        .collect();
    for blocker_id in &order {
        if !blocking_this.contains(blocker_id) {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} is not blocking attacker {:?}",
                blocker_id, attacker
            )));
        }
    }
    // MR-M6-03: the order must include every blocker assigned to this attacker
    // (CR 509.2 — the attacker's controller orders ALL blockers, not a subset).
    if order.len() != blocking_this.len() {
        return Err(GameStateError::IncompleteBlockerOrder {
            provided: order.len(),
            required: blocking_this.len(),
        });
    }
    if let Some(combat) = state.combat.as_mut() {
        combat.damage_assignment_order.insert(attacker, order);
    }
    Ok(Vec::new())
}
// ---------------------------------------------------------------------------
// Combat damage
// ---------------------------------------------------------------------------
/// Apply combat damage for the current step (CR 510).
///
/// `first_strike_step`: true when processing `Step::FirstStrikeDamage`,
/// false when processing `Step::CombatDamage`.
///
/// Creatures with FirstStrike or DoubleStrike deal damage in the first-strike
/// step; creatures with DoubleStrike or neither deal damage in the regular step.
///
/// Damage is assigned simultaneously (CR 510.2), then marked on objects/players
/// all at once. SBAs fire afterward (handled by `enter_step`).
pub fn apply_combat_damage(state: &mut GameState, first_strike_step: bool) -> Vec<GameEvent> {
    let Some(combat) = state.combat.as_ref() else {
        return Vec::new();
    };
    // Clone combat data to avoid borrow conflicts during damage application.
    let attackers = combat.attackers.clone();
    let blockers_map = combat.blockers.clone();
    let damage_order = combat.damage_assignment_order.clone();
    // CR 702.7b: snapshot of creatures with FS/DS at start of first-strike step.
    // Used by deals_damage_in_step to determine regular-step eligibility.
    let first_strike_snapshot = combat.first_strike_participants.clone();
    let mut assignments: Vec<CombatDamageAssignment> = Vec::new();
    // --- Attacker damage ---
    for (attacker_id, attack_target) in &attackers {
        if !deals_damage_in_step(
            state,
            *attacker_id,
            first_strike_step,
            &first_strike_snapshot,
        ) {
            continue;
        }
        let power = get_effective_power(state, *attacker_id);
        if power <= 0 {
            continue;
        }
        let has_trample = has_keyword(state, *attacker_id, KeywordAbility::Trample);
        let has_deathtouch = has_keyword(state, *attacker_id, KeywordAbility::Deathtouch);
        // Get ordered blockers (from damage_assignment_order or default OrdMap order).
        let ordered_blockers: Vec<ObjectId> = if let Some(order) = damage_order.get(attacker_id) {
            order
                .iter()
                .filter(|&&b| {
                    state
                        .objects
                        .get(&b)
                        .map(|o| o.zone == ZoneId::Battlefield)
                        .unwrap_or(false)
                })
                .copied()
                .collect()
        } else {
            blockers_map
                .iter()
                .filter(|(_, &a)| a == *attacker_id)
                .filter(|(&b, _)| {
                    state
                        .objects
                        .get(&b)
                        .map(|o| o.zone == ZoneId::Battlefield)
                        .unwrap_or(false)
                })
                .map(|(&b, _)| b)
                .collect()
        };
        if ordered_blockers.is_empty() {
            // CR 509.1h: a creature remains "blocked" even if all blockers leave.
            // Unblocked = was never blocked during declaration.
            let was_blocked = {
                let c = state.combat.as_ref().unwrap();
                c.is_blocked(*attacker_id)
            };
            if !was_blocked {
                // Truly unblocked — deal damage to attack target.
                push_player_or_pw_damage(
                    &mut assignments,
                    *attacker_id,
                    attack_target,
                    power as u32,
                );
            } else if has_trample {
                // Was blocked but all blockers gone: trample goes to player (CR 702.19d).
                push_player_or_pw_damage(
                    &mut assignments,
                    *attacker_id,
                    attack_target,
                    power as u32,
                );
            }
            // else: blocked (blocker gone), no trample → no player damage.
        } else {
            // Assign damage to blockers in order (CR 510.1c).
            let mut remaining = power;
            let last_idx = ordered_blockers.len() - 1;
            for (i, blocker_id) in ordered_blockers.iter().enumerate() {
                if remaining <= 0 {
                    break;
                }
                let is_last = i == last_idx;
                // Minimum lethal damage for this blocker.
                let lethal = if has_deathtouch {
                    1 // CR 702.2c: deathtouch makes 1 damage lethal for assignment purposes
                } else {
                    let toughness = get_effective_toughness(state, *blocker_id);
                    let already_damaged = state
                        .objects
                        .get(blocker_id)
                        .map(|o| o.damage_marked as i32)
                        .unwrap_or(0);
                    (toughness - already_damaged).max(0)
                };
                if is_last && has_trample {
                    // Last blocker with trample: assign minimum lethal, excess to player.
                    let to_blocker = remaining.min(lethal);
                    if to_blocker > 0 {
                        assignments.push(CombatDamageAssignment {
                            source: *attacker_id,
                            target: CombatDamageTarget::Creature(*blocker_id),
                            amount: to_blocker as u32,
                        });
                    }
                    let trample_amount = remaining - to_blocker;
                    if trample_amount > 0 {
                        push_player_or_pw_damage(
                            &mut assignments,
                            *attacker_id,
                            attack_target,
                            trample_amount as u32,
                        );
                    }
                    remaining = 0;
                } else {
                    // CR 510.1c: for the last blocker (no trample), all remaining power
                    // is assigned to it. For non-last blockers, exactly lethal must be
                    // assigned before moving excess to the next blocker in order.
                    let to_blocker = if is_last || remaining < lethal {
                        remaining
                    } else {
                        lethal
                    };
                    if to_blocker > 0 {
                        assignments.push(CombatDamageAssignment {
                            source: *attacker_id,
                            target: CombatDamageTarget::Creature(*blocker_id),
                            amount: to_blocker as u32,
                        });
                    }
                    remaining -= to_blocker;
                }
            }
        }
    }
    // --- Blocker damage (CR 510.1a: blockers also deal damage to attackers) ---
    for (blocker_id, attacker_id) in &blockers_map {
        let blocker_on_bf = state
            .objects
            .get(blocker_id)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);
        let attacker_on_bf = state
            .objects
            .get(attacker_id)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false);
        if !blocker_on_bf || !attacker_on_bf {
            continue;
        }
        if !deals_damage_in_step(
            state,
            *blocker_id,
            first_strike_step,
            &first_strike_snapshot,
        ) {
            continue;
        }
        let power = get_effective_power(state, *blocker_id);
        if power <= 0 {
            continue;
        }
        assignments.push(CombatDamageAssignment {
            source: *blocker_id,
            target: CombatDamageTarget::Creature(*attacker_id),
            amount: power as u32,
        });
    }
    if assignments.is_empty() {
        return Vec::new();
    }
    // CR 615.1: If all combat damage is prevented this turn, skip all assignments.
    if state.prevent_all_combat_damage {
        return Vec::new();
    }
    // CR 615: Remove assignments whose source or target has per-creature prevention this turn.
    let assignments: Vec<CombatDamageAssignment> = assignments
        .into_iter()
        .filter(|a| {
            if state.combat_damage_prevented_from.contains(&a.source) {
                return false;
            }
            match &a.target {
                CombatDamageTarget::Creature(id) | CombatDamageTarget::Planeswalker(id) => {
                    if state.combat_damage_prevented_to.contains(id) {
                        return false;
                    }
                }
                CombatDamageTarget::Player(_) => {}
            }
            true
        })
        .collect();
    if assignments.is_empty() {
        return Vec::new();
    }
    // --- Collect application info before mutating state ---
    // Pre-extract per-assignment: (source_deathtouch, source_lifelink, source_wither,
    // source_infect, source_toxic_total, source_controller, commander_info)
    // commander_info = Some((attacking_player_id, card_id)) if source is a commander.
    type DamageAppInfo = (
        bool,
        bool,
        bool,
        bool,
        u32,
        PlayerId,
        Option<(PlayerId, CardId)>,
    );
    let app_info: Vec<DamageAppInfo> = assignments
        .iter()
        .map(|a| {
            let obj = state.objects.get(&a.source);
            let chars = calculate_characteristics(state, a.source);
            let source_deathtouch = chars
                .as_ref()
                .map(|c| c.keywords.contains(&KeywordAbility::Deathtouch))
                .unwrap_or(false);
            // CR 702.15a: Damage dealt by a source with lifelink causes its controller to gain life.
            let source_lifelink = chars
                .as_ref()
                .map(|c| c.keywords.contains(&KeywordAbility::Lifelink))
                .unwrap_or(false);
            // CR 702.80a: Damage dealt to a creature by a source with wither places
            // -1/-1 counters instead of marking damage.
            let source_wither = chars
                .as_ref()
                .map(|c| c.keywords.contains(&KeywordAbility::Wither))
                .unwrap_or(false);
            // CR 702.90a: Damage dealt by a source with infect to a creature places
            // -1/-1 counters; to a player gives poison counters (CR 120.3b, CR 120.3d).
            let source_infect = chars
                .as_ref()
                .map(|c| c.keywords.contains(&KeywordAbility::Infect))
                .unwrap_or(false);
            // CR 702.164b: Total toxic value is the sum of all Toxic N values on the source.
            // Multiple instances are cumulative (not redundant like Infect).
            // Uses layer-resolved characteristics so ability-removal (Humility, Dress Down)
            // and ability-granting effects are correctly respected (CR 613).
            // NOTE: If two identical Toxic(N) values exist on the same object, OrdSet
            // deduplication means only one is counted. This is a known limitation (LOW);
            // no real-world card combination currently produces this in the engine.
            let source_toxic_total: u32 = chars
                .as_ref()
                .map(|c| {
                    c.keywords
                        .iter()
                        .filter_map(|kw| match kw {
                            KeywordAbility::Toxic(n) => Some(*n),
                            _ => None,
                        })
                        .sum()
                })
                .unwrap_or(0);
            let source_controller = obj.map(|o| o.controller).unwrap_or(PlayerId(0));
            let commander_info = obj.and_then(|o| {
                let controller = o.controller;
                let card_id = o.card_id.clone()?;
                let is_commander = state
                    .expect_player(controller)
                    .map(|p| p.commander_ids.iter().any(|c| *c == card_id))
                    .unwrap_or(false);
                if is_commander {
                    Some((controller, card_id))
                } else {
                    None
                }
            });
            (
                source_deathtouch,
                source_lifelink,
                source_wither,
                source_infect,
                source_toxic_total,
                source_controller,
                commander_info,
            )
        })
        .collect();
    // --- CR 614.1: Apply damage-doubling replacement effects before prevention ---
    // Doublers apply before preventers (CR 614.1 ordering: replacement effects modify
    // the event first, then prevention effects can prevent the modified amount).
    let mut doubling_events: Vec<GameEvent> = Vec::new();
    let doubled_amounts: Vec<u32> = assignments
        .iter()
        .map(|a| {
            let (doubled_dmg, devts) = crate::rules::replacement::apply_damage_doubling(
                state,
                a.source,
                a.amount,
                Some(&a.target),
            );
            doubling_events.extend(devts);
            doubled_dmg
        })
        .collect();
    // Update assignment amounts with doubled values.
    let assignments: Vec<CombatDamageAssignment> = assignments
        .into_iter()
        .zip(doubled_amounts.iter())
        .map(|(mut a, &amt)| {
            a.amount = amt;
            a
        })
        .collect();
    // --- CR 702.16e + CR 615: Apply protection then dynamic prevention ---
    // apply_damage_prevention checks protection (static) first, then dynamic shields.
    let mut prevention_events: Vec<GameEvent> = Vec::new();
    let final_amounts: Vec<u32> = assignments
        .iter()
        .map(|a| {
            let (final_dmg, pevts) = crate::rules::replacement::apply_damage_prevention(
                state, a.source, &a.target, a.amount,
            );
            prevention_events.extend(pevts);
            final_dmg
        })
        .collect();
    // Build the post-prevention assignment list for the CombatDamageDealt event.
    let final_assignments: Vec<CombatDamageAssignment> = assignments
        .iter()
        .zip(final_amounts.iter())
        .map(|(a, &amt)| CombatDamageAssignment {
            source: a.source,
            target: a.target.clone(),
            amount: amt,
        })
        .collect();
    // --- Apply damage and collect lifelink gains ---
    // lifelink_gains: controller → total damage dealt by their lifelink sources this step.
    let mut lifelink_gains: imbl::OrdMap<PlayerId, u32> = imbl::OrdMap::new();
    // Collect wither/infect counter events during the damage application loop.
    // These will be added to the event stream after the loop.
    let mut wither_counter_events: Vec<GameEvent> = Vec::new();
    // Collect PoisonCountersGiven events for infect damage to players.
    let mut poison_events: Vec<GameEvent> = Vec::new();
    for (
        (
            assignment,
            (
                source_deathtouch,
                source_lifelink,
                source_wither,
                source_infect,
                source_toxic_total,
                source_controller,
                commander_info,
            ),
        ),
        &final_dmg,
    ) in assignments
        .iter()
        .zip(app_info.iter())
        .zip(final_amounts.iter())
    {
        if final_dmg == 0 {
            // All damage prevented for this assignment — skip state mutation.
            continue;
        }
        match &assignment.target {
            CombatDamageTarget::Creature(obj_id) => {
                if let Some(obj) = state.expect_object_mut(*obj_id) {
                    if *source_wither || *source_infect {
                        // CR 702.80a / CR 702.90c / CR 120.3d: damage to a creature by a
                        // source with wither and/or infect places -1/-1 counters instead
                        // of marking damage. Multiple instances of wither/infect are
                        // redundant (CR 702.80d / CR 702.90f) — this fires at most once.
                        let cur = obj
                            .counters
                            .get(&CounterType::MinusOneMinusOne)
                            .copied()
                            .unwrap_or(0);
                        obj.counters
                            .insert(CounterType::MinusOneMinusOne, cur + final_dmg);
                        wither_counter_events.push(GameEvent::CounterAdded {
                            object_id: *obj_id,
                            counter: CounterType::MinusOneMinusOne,
                            count: final_dmg,
                        });
                    } else {
                        // CR 120.3e: normal damage marking.
                        obj.damage_marked += final_dmg;
                    }
                    if *source_deathtouch {
                        obj.deathtouch_damage = true;
                    }
                }
            }
            CombatDamageTarget::Player(player_id) => {
                if *source_infect {
                    // CR 702.90b / CR 120.3b: infect damage to a player gives poison
                    // counters instead of causing life loss.
                    if let Some(player) = state.expect_player_mut(*player_id) {
                        player.poison_counters += final_dmg;
                        // CR 702.54a: Bloodthirst counts infect damage even though
                        // it causes poison counters rather than life loss.
                        player.damage_received_this_turn += final_dmg;
                    }
                    poison_events.push(GameEvent::PoisonCountersGiven {
                        player: *player_id,
                        amount: final_dmg,
                        source: assignment.source,
                    });
                } else {
                    // CR 120.3a: normal damage causes life loss.
                    if let Some(player) = state.expect_player_mut(*player_id) {
                        player.life_total -= final_dmg as i32;
                        // CR 702.137a: track life lost this turn for Spectacle.
                        player.life_lost_this_turn += final_dmg;
                        // CR 702.54a: track damage received this turn for Bloodthirst.
                        player.damage_received_this_turn += final_dmg;
                    }
                }
                // CR 702.164c / CR 120.3g: Toxic -- give poison counters equal to the
                // source's total toxic value, in addition to the damage's other results.
                // Applies regardless of Infect (both can coexist: Infect adds damage-amount
                // poison counters, Toxic adds toxic-value poison counters independently).
                // The final_dmg == 0 guard above ensures we only reach here when damage
                // was actually dealt (CR 120.3g: "combat damage dealt to a player").
                if *source_toxic_total > 0 {
                    if let Some(player) = state.expect_player_mut(*player_id) {
                        player.poison_counters += *source_toxic_total;
                    }
                    poison_events.push(GameEvent::PoisonCountersGiven {
                        player: *player_id,
                        amount: *source_toxic_total,
                        source: assignment.source,
                    });
                }
                // Track commander damage (CR 903.10a).
                // Commander damage counts COMBAT damage dealt, not life lost — infect
                // damage still counts toward commander damage totals (CR 903.10a).
                if let Some((attacking_player, card_id)) = commander_info {
                    if player_id != attacking_player {
                        // Can't double-borrow players — read current value, then write.
                        let current = state
                            .expect_player(*player_id)
                            .and_then(|p| p.commander_damage_received.get(attacking_player))
                            .and_then(|m| m.get(card_id))
                            .copied()
                            .unwrap_or(0);
                        let new_val = current + final_dmg;
                        let inner = state
                            .expect_player(*player_id)
                            .and_then(|p| p.commander_damage_received.get(attacking_player))
                            .cloned()
                            .unwrap_or_default();
                        let mut new_inner = inner;
                        new_inner.insert(card_id.clone(), new_val);
                        if let Some(target_player) = state.expect_player_mut(*player_id) {
                            target_player
                                .commander_damage_received
                                .insert(*attacking_player, new_inner);
                        }
                    }
                }
            }
            CombatDamageTarget::Planeswalker(pw_id) => {
                // CR 306.8: Damage dealt to a planeswalker results in that many
                // loyalty counters being removed from it. This matches the non-combat
                // damage path in effects/mod.rs.
                // CR 400.7: the attacked planeswalker may have left the battlefield since
                // attackers were declared (combat.attackers still names it); its old id
                // then names nothing, so no loyalty is removed (LKI fizzle).
                if let Some(obj) = state.fizzle_object_mut(*pw_id) {
                    let cur = obj
                        .counters
                        .get(&CounterType::Loyalty)
                        .copied()
                        .unwrap_or(0);
                    let new_val = cur.saturating_sub(final_dmg);
                    obj.counters.insert(CounterType::Loyalty, new_val);
                }
            }
        }
        // CR 702.15a: Lifelink — source's controller gains life equal to damage dealt.
        if *source_lifelink {
            let entry = lifelink_gains.entry(*source_controller).or_insert(0);
            *entry += final_dmg;
        }
    }
    // Prevention events fire before CombatDamageDealt (they modify the event as it happens).
    let mut events = prevention_events;
    // CounterAdded events from wither/infect precede the CombatDamageDealt summary event.
    events.extend(wither_counter_events);
    // PoisonCountersGiven events from infect player damage precede CombatDamageDealt.
    events.extend(poison_events);
    events.push(GameEvent::CombatDamageDealt {
        assignments: final_assignments,
    });
    // Apply lifelink gains and emit LifeGained events.
    for (controller, amount) in &lifelink_gains {
        if let Some(player) = state.expect_player_mut(*controller) {
            player.life_total += *amount as i32;
            // PB-B: Track life gained this turn for Condition::ControllerGainedLifeThisTurn.
            player.life_gained_this_turn += *amount;
        }
        events.push(GameEvent::LifeGained {
            player: *controller,
            amount: *amount,
        });
    }
    events
}
// ---------------------------------------------------------------------------
// First strike step detection
// ---------------------------------------------------------------------------
/// Returns true if any combatant has FirstStrike or DoubleStrike,
/// meaning a separate first-strike damage step must occur (CR 510.4).
pub fn should_have_first_strike_step(state: &GameState) -> bool {
    let Some(combat) = state.combat.as_ref() else {
        return false;
    };
    // Check attackers.
    let attacker_has_fs = combat.attackers.keys().any(|&id| {
        has_keyword(state, id, KeywordAbility::FirstStrike)
            || has_keyword(state, id, KeywordAbility::DoubleStrike)
    });
    // Check blockers.
    let blocker_has_fs = combat.blockers.keys().any(|&id| {
        has_keyword(state, id, KeywordAbility::FirstStrike)
            || has_keyword(state, id, KeywordAbility::DoubleStrike)
    });
    attacker_has_fs || blocker_has_fs
}
// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------
/// Returns the effective power of an object using the layer system.
fn get_effective_power(state: &GameState, id: ObjectId) -> i32 {
    calculate_characteristics(state, id)
        .and_then(|c| c.power)
        .unwrap_or(0)
}
/// Returns the effective toughness of an object using the layer system.
fn get_effective_toughness(state: &GameState, id: ObjectId) -> i32 {
    calculate_characteristics(state, id)
        .and_then(|c| c.toughness)
        .unwrap_or(0)
}
/// Returns true if the object has the given keyword (via layer system).
fn has_keyword(state: &GameState, id: ObjectId, keyword: KeywordAbility) -> bool {
    calculate_characteristics(state, id)
        .map(|c| c.keywords.contains(&keyword))
        .unwrap_or(false)
}
/// Returns true if this creature deals damage in the given step.
///
/// CR 702.7b: First-strike creatures deal damage only in the first-strike step.
/// CR 702.4b: Double-strike creatures deal damage in BOTH steps.
/// CR 702.7b: Normal creatures deal damage only in the regular step.
///
/// For the regular step, eligibility is based on `first_strike_snapshot` —
/// the set of creatures that had FirstStrike or DoubleStrike at the START of
/// the first-strike step (CR 702.7b). This implements the CR 702.7c/702.4c/702.4d
/// edge cases:
/// - Gained FS after first step: not in snapshot → dealt damage in regular step (CR 702.7c).
/// - Lost FS after first step: in snapshot → excluded from regular step (CR 702.7c).
/// - Lost DS after first step: in snapshot → excluded from regular step (CR 702.4c).
/// - FS creature gained DS after first step: in snapshot → still excluded (CR 702.4d
///   says "will allow" regular damage — but FS-only creatures in snapshot are excluded;
///   the creature must have been DS at snapshot time to deal regular damage).
///
/// If `first_strike_snapshot` is empty (first-strike step never occurred), we fall back
/// to current keywords (correct for the no-FS-step case where everything deals in one step).
fn deals_damage_in_step(
    state: &GameState,
    id: ObjectId,
    first_strike_step: bool,
    first_strike_snapshot: &OrdSet<ObjectId>,
) -> bool {
    let has_first = has_keyword(state, id, KeywordAbility::FirstStrike);
    let has_double = has_keyword(state, id, KeywordAbility::DoubleStrike);
    if first_strike_step {
        // First-strike step: deal damage iff currently has FS or DS.
        has_first || has_double
    } else if !first_strike_snapshot.is_empty() {
        // Regular step, after a first-strike step occurred.
        // CR 702.7b: exclude creatures that had FS or DS at the start of the first step
        // (they already dealt damage then), unless they have DS (deal in both).
        // Use snapshot for "had first strike" determination, current keywords for DS.
        let was_in_first_step = first_strike_snapshot.contains(&id);
        // A creature in the snapshot had FS or DS. If it had DS then, it deals in both.
        // If it had only FS (no DS now), it is excluded. DS gained later still allows
        // regular step per CR 702.4d, but FS gained later doesn't exclude per CR 702.7c.
        if was_in_first_step {
            // Had FS or DS at snapshot time → only deal regular damage if has DS NOW.
            has_double
        } else {
            // Was NOT in snapshot (no FS or DS at step start) → deals regular damage
            // (even if it gained FS after: CR 702.7c says gaining FS after won't preclude
            // regular damage, but it also won't add it to the first step).
            true
        }
    } else {
        // No first-strike step occurred (snapshot empty) — use current keywords.
        has_double || !has_first
    }
}
/// Push a damage assignment to a player or planeswalker attack target.
fn push_player_or_pw_damage(
    assignments: &mut Vec<CombatDamageAssignment>,
    source: ObjectId,
    target: &AttackTarget,
    amount: u32,
) {
    match target {
        AttackTarget::Player(pid) => {
            assignments.push(CombatDamageAssignment {
                source,
                target: CombatDamageTarget::Player(*pid),
                amount,
            });
        }
        AttackTarget::Planeswalker(pw_id) => {
            assignments.push(CombatDamageAssignment {
                source,
                target: CombatDamageTarget::Planeswalker(*pw_id),
                amount,
            });
        }
    }
}

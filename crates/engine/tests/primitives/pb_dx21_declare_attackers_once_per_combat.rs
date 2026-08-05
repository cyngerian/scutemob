//! PB-DX21 (`scutemob-200`) — CR 508.1: attackers may be declared without limit
//! (`OOS-M11-9`).
//!
//! `crates/engine/src/rules/combat.rs::handle_declare_attackers` guarded on step,
//! active player, priority holder and per-attacker legality — and on nothing
//! else. A second `Command::DeclareAttackers` in the same combat re-ran the whole
//! body: it re-inserted into `combat.attackers` (overwriting a repeated key's
//! target), re-pushed `GameEvent::AttackersDeclared` and re-fired attack triggers
//! (CR 508.2a/508.3a-e say "only at the point the creature is declared"), and
//! re-assigned (not accumulated) `attackers_declared_this_turn`. CR 508.1 makes
//! the declaration a once-per-combat turn-based action; `CombatState` did not
//! record that it had happened.
//!
//! `memory/primitives/pb-plan-DX21.md` is authoritative. T1-T7 below are its §4;
//! T8 is included (not omitted) per its own "cheap, so include it" framing.
//!
//! **VERDICT 2 (plan §1.3), the decisive one**: the brief's stated preference —
//! key the guard on `!combat.attackers.is_empty()` — is refused. An EMPTY
//! declaration is a real, CR 508.1a-legal, live client action
//! (`params.rs:474`/`api.rs:298-306`, "a legal, irreversible 'I attack with
//! nothing'"), so `combat.attackers` cannot distinguish "declared nothing" from
//! "has not declared". T4 is the probe that makes this real; its own doc records
//! the second, alternate-guard revert that proves the distinction.

use mtg_engine::state::hash::HashInto;
use mtg_engine::state::stubs::ActiveRestriction;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AttackTarget,
    CardDefinition, CardRegistry, CombatState, Command, GameEvent, GameRestriction, GameState,
    GameStateBuilder, GameStateError, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;
use std::sync::Arc;

// ── Helpers (mirrors pb_dx6_unflattened_payment_sites.rs /
//    pb_dx1_lowered_intervening_if.rs) ────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn build_defs_and_registry() -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    (defs, registry)
}

fn enrich(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object {name:?} not found"))
}

/// A `Command::DeclareAttackers` with every optional-choice field empty —
/// convenience wrapper used by every probe below that doesn't exercise
/// enlist/exert/hybrid/Phyrexian choices.
fn declare_cmd(player: PlayerId, attackers: Vec<(ObjectId, AttackTarget)>) -> Command {
    Command::DeclareAttackers {
        player,
        attackers,
        enlist_choices: vec![],
        exert_choices: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }
}

fn add_restriction(
    state: &mut GameState,
    source: ObjectId,
    controller: PlayerId,
    restriction: GameRestriction,
) {
    state.restrictions_mut().push_back(ActiveRestriction {
        source,
        controller,
        restriction,
    });
}

/// Pass priority through whoever currently holds it, repeatedly, until `stop`
/// is satisfied. Mirrors `pb_dx1_lowered_intervening_if.rs::advance_until`
/// exactly (same idiom, needed here for T5's extra-combat drive).
fn advance_until(
    mut state: GameState,
    guard_max: usize,
    stop: impl Fn(&GameState) -> bool,
) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut guard = 0;
    while !stop(&state) {
        guard += 1;
        assert!(
            guard < guard_max,
            "advance_until: stop condition not reached after {guard_max} priority passes \
             (step={:?}, phase={:?}, in_extra_combat={}, stack_len={})",
            state.turn().step,
            state.turn().phase,
            state.turn().in_extra_combat,
            state.stack_objects().len()
        );
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or_else(|| panic!("no priority holder at guard={guard}"));
        let (new_state, events) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {holder:?} failed: {e:?}"));
        all_events.extend(events);
        state = new_state;
    }
    (state, all_events)
}

// ── T1 — (a) attack-target overwrite (plan §4 T1) ───────────────────────────

#[test]
/// CR 508.1 / CR 732 (PB-DX21, OOS-M11-9): a second `Command::DeclareAttackers`
/// in the same combat is REJECTED, and the FIRST declaration's attack target is
/// not overwritten.
///
/// Fixture: Samut, Voice of Dissent (`Complete`, real corpus def) — Vigilance
/// keeps her untapped after attacking, so `combat.rs`'s already-tapped check
/// (an accident, not a guard — plan §4 T1) cannot mask the defect.
///
/// Pre-fix behaviour (Stage 0, `memory/primitives/pb-DX21-stage0.md`): the
/// second command returned `Ok`, and `combat.attackers.insert(*attacker_id, ..)`
/// OVERWROTE Samut's target from p2 to p3 — the seed's "overwrites
/// combat.attackers" wording describes the per-key overwrite of an `OrdMap`,
/// not a replace of the whole map.
fn test_dx21_second_declaration_rejected_target_not_overwritten() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let samut = enrich(p1, "Samut, Voice of Dissent", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::four_player()
        .with_registry(registry)
        .object(samut)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let samut_id = find_by_name(&state, "Samut, Voice of Dissent");

    // (1) First declaration: Samut attacks p2.
    let (state, _events) = process_command(
        state,
        declare_cmd(p1, vec![(samut_id, AttackTarget::Player(p2))]),
    )
    .expect("first declaration must succeed");

    assert!(
        !state
            .object(samut_id)
            .expect("samut on battlefield")
            .status
            .tapped,
        "Vigilance -- Samut must stay untapped after attacking, so the already-tapped \
         check (an accident, not a guard) cannot mask this probe"
    );

    // (2) Second declaration in the SAME combat, a DIFFERENT target: rejected.
    // `state.clone()` is fed to the failing call so the ORIGINAL `state` survives
    // untouched -- mirrors the production caller pattern
    // (`crates/simulator/src/local_game.rs:1207`,
    // `process_command(self.state.clone(), command.clone())`), and is the only
    // way to inspect post-attempt state, since `process_command` returns no
    // `GameState` on `Err`.
    let err = process_command(
        state.clone(),
        declare_cmd(p1, vec![(samut_id, AttackTarget::Player(p3))]),
    )
    .expect_err("CR 508.1: a second declaration in the same combat must be rejected");
    assert!(
        matches!(err, GameStateError::AlreadyDeclaredAttackers(pid) if pid == p1),
        "expected AlreadyDeclaredAttackers(p1), got: {err:?}"
    );

    // (3) The target did not move.
    assert_eq!(
        state.combat().as_ref().unwrap().attackers.get(&samut_id),
        Some(&AttackTarget::Player(p2)),
        "the rejected re-declaration must not overwrite Samut's attack target"
    );
}

// ── T2 — (b) attack-trigger re-fire (plan §4 T2) ────────────────────────────

#[test]
/// CR 508.2a / 508.3a-e (PB-DX21, OOS-M11-9): "Whenever ~ attacks" triggers
/// only at the point a creature is DECLARED as an attacker -- exactly once per
/// combat, not once per (accepted) declaration attempt.
///
/// Fixture: Nadaar, Selfless Paladin -- `Complete` (`nadaar_selfless_paladin.
/// rs:81`), Vigilance + `TriggerCondition::WhenAttacks` with `once_per_turn:
/// false` (plan §4 T2 -- verified the sole such `Complete` def in the corpus
/// other than the excluded pair). Venture into the dungeon is choice-free in
/// this fixture: `dungeon_cards.rs::test_nadaar_attacks_ventures` resolves the
/// identical trigger with two plain `PassPriority` commands and no blocking
/// decision, so `GameEvent::VenturedIntoDungeon` is used as assertion (3)'s
/// observable, per plan §4 T2's own fallback instruction ("if it raises a
/// blocking decision, do not fight it" -- confirmed here it does not).
///
/// Pre-fix behaviour: two `AttackersDeclared` events and two ventures.
fn test_dx21_second_declaration_rejected_attack_trigger_fires_once() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let nadaar = enrich(p1, "Nadaar, Selfless Paladin", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(nadaar)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let nadaar_id = find_by_name(&state, "Nadaar, Selfless Paladin");

    // (1) First declaration.
    let (state, events1) = process_command(
        state,
        declare_cmd(p1, vec![(nadaar_id, AttackTarget::Player(p2))]),
    )
    .expect("first declaration must succeed");

    // (assertion 2) exactly one AttackersDeclared event across the whole combat
    // -- trivially true for a single accepted declaration; the point (proven
    // below by attempting a second) is that a REJECTED second attempt cannot
    // add a second one.
    let declared_count = events1
        .iter()
        .filter(|e| matches!(e, GameEvent::AttackersDeclared { .. }))
        .count();
    assert_eq!(
        declared_count, 1,
        "exactly one AttackersDeclared event from the first, accepted declaration"
    );

    // (assertion 1) Second declaration in the same combat: rejected.
    let err = process_command(
        state.clone(),
        declare_cmd(p1, vec![(nadaar_id, AttackTarget::Player(p2))]),
    )
    .expect_err("CR 508.1: a second declaration in the same combat must be rejected");
    assert!(
        matches!(err, GameStateError::AlreadyDeclaredAttackers(pid) if pid == p1),
        "expected AlreadyDeclaredAttackers(p1), got: {err:?}"
    );

    // (assertion 3) Resolve Nadaar's WhenAttacks trigger and count
    // VenturedIntoDungeon exactly once. Pre-fix, the rejected call above would
    // instead have SUCCEEDED and re-queued the trigger, so two ventures would
    // resolve here.
    let (state, t_events1) =
        process_command(state, Command::PassPriority { player: p1 }).expect("p1 passes");
    let (_state, t_events2) =
        process_command(state, Command::PassPriority { player: p2 }).expect("p2 passes");

    let ventured_count = t_events1
        .iter()
        .chain(t_events2.iter())
        .filter(|e| matches!(e, GameEvent::VenturedIntoDungeon { player, .. } if *player == p1))
        .count();
    assert_eq!(
        ventured_count, 1,
        "the WhenAttacks trigger must fire exactly once (CR 508.2a/508.3a) -- pre-fix, \
         a second (accepted) declaration re-fired it, producing two VenturedIntoDungeon \
         events"
    );
}

// ── T3 — (c) attackers_declared_this_turn raid-count clobber (plan §4 T3) ──

#[test]
/// CR 508.1 / PB-AC6 (PB-DX21, OOS-M11-9): a rejected re-declaration must not
/// clobber `PlayerState.attackers_declared_this_turn` (`combat.rs` sets it,
/// does not accumulate it) -- and the consequence that makes this a card
/// probe, not just a field probe: Windbrisk Heights' `Condition::
/// YouAttackedWithNOrMore(3)` must still hold, exercised through the real
/// activation path.
///
/// Pre-fix behaviour: the second (accepted, no-op) declaration of only the
/// vigilant creature reassigned `attackers_declared_this_turn` to 1, and
/// Windbrisk Heights' `{W},{T}` ability went dead for the rest of the turn.
fn test_dx21_second_declaration_rejected_raid_count_not_clobbered() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let samut = enrich(p1, "Samut, Voice of Dissent", ZoneId::Battlefield, &defs);
    let heights = enrich(p1, "Windbrisk Heights", ZoneId::Battlefield, &defs);
    let bear_b = ObjectSpec::creature(p1, "T3 Bear B", 2, 2).in_zone(ZoneId::Battlefield);
    let bear_c = ObjectSpec::creature(p1, "T3 Bear C", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(samut)
        .object(heights)
        .object(bear_b)
        .object(bear_c)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let samut_id = find_by_name(&state, "Samut, Voice of Dissent");
    let bear_b_id = find_by_name(&state, "T3 Bear B");
    let bear_c_id = find_by_name(&state, "T3 Bear C");
    let heights_id = find_by_name(&state, "Windbrisk Heights");

    // (1) Declare THREE attackers.
    let (state, _events) = process_command(
        state,
        declare_cmd(
            p1,
            vec![
                (samut_id, AttackTarget::Player(p2)),
                (bear_b_id, AttackTarget::Player(p2)),
                (bear_c_id, AttackTarget::Player(p2)),
            ],
        ),
    )
    .expect("first declaration of three attackers must succeed");
    assert_eq!(
        state.player(p1).unwrap().attackers_declared_this_turn,
        3,
        "CR 508.1/PB-AC6: three attackers declared this turn"
    );

    // (2)/(3) Attempt a second declaration naming only the vigilant creature:
    // rejected, and the raid count is UNCHANGED.
    let err = process_command(
        state.clone(),
        declare_cmd(p1, vec![(samut_id, AttackTarget::Player(p2))]),
    )
    .expect_err("CR 508.1: a second declaration in the same combat must be rejected");
    assert!(
        matches!(err, GameStateError::AlreadyDeclaredAttackers(pid) if pid == p1),
        "expected AlreadyDeclaredAttackers(p1), got: {err:?}"
    );
    assert_eq!(
        state.player(p1).unwrap().attackers_declared_this_turn,
        3,
        "a rejected re-declaration must not clobber attackers_declared_this_turn"
    );

    // (4) Windbrisk Heights' {W},{T} ability (activated_abilities[0] -- its
    // {T}: Add {W} mana ability is filtered out of that index, per the standing
    // ability-index gotcha) is still activatable: its
    // Condition::YouAttackedWithNOrMore(3) still holds. Exercised through the
    // REAL activation path -- a rejection here would carry the literal message
    // "activation condition not met" (abilities.rs's CR 602.5b check).
    let mut state = state;
    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool.white = 1;
    }
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: heights_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_ok(),
        "Windbrisk Heights' condition-gated ability must still be activatable after the \
         rejected re-declaration, got: {:?}",
        result.err()
    );
}

// ── T4 — the EMPTY declaration counts as a declaration (plan §4 T4) ────────

#[test]
/// CR 508.1a / 508.8 / 117.4 (PB-DX21, OOS-M11-9): an EMPTY declaration IS a
/// completed CR 508.1 turn-based action, and blocks a later non-empty
/// re-declaration -- refuting the brief's `!combat.attackers.is_empty()` guard
/// (plan §1.3). An `!attackers.is_empty()` guard passes steps (1)-(3) below and
/// fails nothing -- this test is the discriminator between the two candidate
/// implementations, not just between fixed and unfixed (verified below by
/// executing that SECOND revert, per plan §4 T4's mandate).
fn test_dx21_empty_declaration_counts_and_blocks_redeclaration() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let samut = enrich(p1, "Samut, Voice of Dissent", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(samut)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    let samut_id = find_by_name(&state, "Samut, Voice of Dissent");

    // (1) An EMPTY declaration.
    let (mut state, _events) = process_command(state, declare_cmd(p1, vec![]))
        .expect("CR 508.1a: declaring no attackers is legal");
    assert!(
        state.combat().as_ref().unwrap().attackers.is_empty(),
        "the map the brief wanted to key on is empty -- the marker must not be"
    );

    // The fourth consequence (stage 0): CR 117.4 -- the ACCEPTED empty
    // declaration itself resets the pass-round.
    assert!(
        state.turn().players_passed.is_empty(),
        "the accepted empty declaration resets players_passed"
    );

    // Simulate "mid-round, someone has already passed": in this single-attack
    // 2-player scenario, real PassPriority commands cannot produce "p1 still
    // holds priority AND players_passed is non-empty" (a full pass-round with
    // an empty stack ends the step), so the field is set directly -- this
    // mirrors the shape a real N>2-player table produces (one of several
    // players has passed; priority has not yet returned around to the active
    // player's own re-pass) without needing N players wired up here.
    state.turn_mut().players_passed.insert(p2);
    let passed_before_reject = state.turn().players_passed.clone();

    // (2) A later, non-empty declaration in the SAME combat: rejected.
    let err = process_command(
        state.clone(),
        declare_cmd(p1, vec![(samut_id, AttackTarget::Player(p2))]),
    )
    .expect_err(
        "CR 508.1a/508.8: the empty declaration already performed the once-per-combat \
         action; a later non-empty declaration must be rejected",
    );
    assert!(
        matches!(err, GameStateError::AlreadyDeclaredAttackers(pid) if pid == p1),
        "expected AlreadyDeclaredAttackers(p1), got: {err:?}"
    );

    // (3) `attackers` is still empty.
    assert!(
        state.combat().as_ref().unwrap().attackers.is_empty(),
        "attackers must still be empty after the rejected attempt"
    );

    // (4) The rejected re-declaration must not have reset the pass-round --
    // `combat.rs`'s `players_passed = OrdSet::new()` runs only on the SUCCESS
    // path.
    assert_eq!(
        state.turn().players_passed,
        passed_before_reject,
        "a rejected re-declaration must not hold the CR 117.4 pass-round open by \
         resetting players_passed"
    );
}

// ── T5 — the marker is per COMBAT, not per turn (plan §4 T5) ───────────────

#[test]
/// CR 500.8 / 506.5 (PB-DX21, OOS-M11-9): `attackers_declared` is per COMBAT
/// PHASE, not per turn -- an extra combat phase gets a fresh, clear marker.
/// Direct successor to MR-M11-09 (the extra-combat regression the client-side
/// per-turn `RepeatKey::DeclareAttackers` cap was patched to avoid).
///
/// Fixture: Aurelia, the Warleader -- `Complete`, Vigilance,
/// `WhenAttacks`/`once_per_turn: true` -> untap all + one additional combat
/// phase. Drive idiom mirrors `pb_dx1_lowered_intervening_if.rs::
/// test_dx1_aurelia_attack_trigger_fires_exactly_once_per_turn` exactly (same
/// `advance_until` helper, same stop predicate).
fn test_dx21_marker_is_per_combat_not_per_turn() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let aurelia = enrich(p1, "Aurelia, the Warleader", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(aurelia)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let aurelia_id = find_by_name(&state, "Aurelia, the Warleader");

    // Combat 1: declare Aurelia. Her marker is set.
    let (state, _events) = process_command(
        state,
        declare_cmd(p1, vec![(aurelia_id, AttackTarget::Player(p2))]),
    )
    .unwrap_or_else(|e| panic!("combat 1 DeclareAttackers failed: {e:?}"));
    assert!(
        state.combat().as_ref().unwrap().attackers_declared,
        "combat 1's marker must be set after a successful declaration"
    );

    // Drive priority through the trigger (untap + grant an extra combat),
    // EndOfCombat -> BeginningOfCombat of the extra combat, up to (but not
    // past) that combat's DeclareAttackers step.
    let (state, _ev) = advance_until(state, 60, |s| {
        s.turn().step == Step::DeclareAttackers
            && s.turn().in_extra_combat
            && s.stack_objects().is_empty()
    });

    // Non-vacuity: the extra combat was actually reached.
    assert!(
        state.turn().in_extra_combat,
        "the probe must actually reach the extra combat, or the assertions below are vacuous"
    );

    // `begin_combat` installed a FRESH `CombatState` -- the marker is clear.
    assert!(
        !state.combat().as_ref().unwrap().attackers_declared,
        "CR 500.8/506.5: the extra combat's fresh CombatState must have a clear marker"
    );

    // Combat 2's declaration must succeed -- the marker did not leak across the
    // combat-phase boundary.
    let (state, _events) = process_command(
        state,
        declare_cmd(p1, vec![(aurelia_id, AttackTarget::Player(p2))]),
    )
    .unwrap_or_else(|e| panic!("combat 2 DeclareAttackers must succeed: {e:?}"));
    assert!(
        state.combat().as_ref().unwrap().attackers_declared,
        "combat 2's marker must be set after ITS successful declaration"
    );
}

// ── T6 — the marker is set on the SUCCESS path only (plan §4 T6) ───────────

#[test]
/// CR 508.1/508.1j (PB-DX21): the marker is set on the SUCCESS path only. A
/// rejected declaration (here, an unaffordable CR 508.1h attack tax) leaves it
/// clear (or leaves `combat` unset -- the guard runs before the `CombatState`
/// init), and a subsequent LEGAL retry succeeds. This is the probe protecting
/// the ~20 existing retry-after-rejection call sites in
/// `pb_dx6_unflattened_payment_sites.rs` and
/// `pb_dp4_attack_tax_and_payment_deadline.rs`.
///
/// **Deliberately calls `mtg_engine::rules::combat::handle_declare_attackers`
/// directly (`&mut GameState`), not `process_command`** -- the ONLY exception
/// to this file's black-box-via-`process_command` convention (T7 is the other
/// one, for the same underlying reason). This was discovered empirically, not
/// assumed: a first draft of this probe used `process_command`, cloning the
/// state before the rejected attempt and inspecting the pristine ORIGINAL
/// afterward (the standard idiom -- see T1's doc comment, and
/// `local_game.rs:1207`). That draft did NOT redden under the "set on entry"
/// revert, because `process_command`'s signature --
/// `Result<(GameState, Vec<GameEvent>), GameStateError>` -- has no `GameState`
/// on the `Err` arm: ANY mutation performed before an `Err` return is
/// UNCONDITIONALLY discarded by Rust's ownership model, regardless of where in
/// the callee it happened. So "did the marker get set before the rejection"
/// is structurally unobservable through the public command API -- proven, not
/// presumed, by executing that draft against the "set on entry" revert and
/// watching it stay green. Calling the `&mut` function directly is the only
/// way to inspect the SAME instance across a rejected-then-retried pair.
///
/// Revert for THIS probe is different from T1-T5 (per plan §4 T6): move
/// `combat.attackers_declared = true;` up to just after the guard (set on
/// entry, not on the success path).
fn test_dx21_marker_set_on_success_path_only() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p2, "T6 Tax Source", 0, 4).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p1, "T6 Attacking Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let tax_source = find_by_name(&state, "T6 Tax Source");
    add_restriction(
        &mut state,
        tax_source,
        p2,
        GameRestriction::CantAttackYouUnlessPay {
            cost_per_creature: ManaCost {
                white: 1,
                ..Default::default()
            },
        },
    );
    let bear = find_by_name(&state, "T6 Attacking Bear");

    // (1) Attempt with an EMPTY pool -> Err (unaffordable attack tax). ONE
    // `state` instance, threaded through by `&mut`, across both attempts.
    let err = mtg_engine::rules::combat::handle_declare_attackers(
        &mut state,
        p1,
        vec![(bear, AttackTarget::Player(p2))],
        vec![],
        vec![],
        vec![],
        vec![],
    )
    .expect_err("empty pool cannot pay the CR 508.1h attack tax");
    assert!(
        matches!(err, GameStateError::InvalidCommand(_)),
        "expected InvalidCommand (affordability), got: {err:?}"
    );

    // (2) `combat` is either unset, or set but with a CLEAR marker -- which of
    // the two holds depends on whether a prior BeginningOfCombat ran; it did
    // not in this hand-built fixture, so `combat` is `None`. On the SAME
    // (mutated-in-place) `state` -- this is what the direct call buys us.
    match state.combat().as_ref() {
        None => {}
        Some(c) => assert!(
            !c.attackers_declared,
            "a rejected declaration must not set the marker"
        ),
    }

    // (3) Add mana and re-issue the IDENTICAL command on the SAME `state` ->
    // Ok. If the marker were set on ENTRY (the revert), step (1)'s rejected
    // call would already have left `attackers_declared == true`, and THIS
    // call would be rejected with `AlreadyDeclaredAttackers` -- a legal retry
    // turned illegal by the earlier failure.
    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool.white = 1;
    }
    let _events = mtg_engine::rules::combat::handle_declare_attackers(
        &mut state,
        p1,
        vec![(bear, AttackTarget::Player(p2))],
        vec![],
        vec![],
        vec![],
        vec![],
    )
    .expect("CR 508.1j: a legal retry after a rejected declaration must succeed");

    // (4) Marker set after the successful retry.
    assert!(
        state.combat().as_ref().unwrap().attackers_declared,
        "the marker must be set after the successful retry"
    );
}

// ── T7 — the new field is actually in the hash stream (plan §4 T7) ─────────

#[test]
/// Mandatory (plan §2.5 / §6.1): `tests/core/hash_schema.rs`'s
/// `canonical_fixture()` cannot populate `combat` (one of its five named
/// exclusions), so `stream_fingerprint` moves only via the v40
/// version-sentinel-byte mechanism for this bump -- `attackers_declared`'s own
/// bytes are covered by NO other gate. This direct `HashInto` unit test,
/// mirroring `pb_eng2_targets_announced.rs::
/// test_eng2_targets_announced_hashes_its_targets`, is the ONLY place they are
/// proven.
fn test_dx21_attackers_declared_is_hashed() {
    fn hash_combat(c: &CombatState) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        c.hash_into(&mut h);
        *h.finalize().as_bytes()
    }

    let clear = CombatState::new(p(1));
    let mut set = CombatState::new(p(1));
    set.attackers_declared = true;

    assert_ne!(
        hash_combat(&clear),
        hash_combat(&set),
        "two CombatState values differing ONLY in attackers_declared hashed identically \
         -- the field is not reaching the hash stream"
    );
}

// ── T8 — the CR 509.1a twin still holds (plan §4 T8) ────────────────────────

#[test]
/// CR 509.1a (PB-DX21): a companion asserting `AlreadyDeclaredBlockers` still
/// fires, so a future refactor of the attacker guard cannot collaterally break
/// its sibling. Not omitted -- plan §4 T8 calls it "optional but cheap";
/// `crates/engine/tests/combat/combat.rs:1701` already covers this exact
/// scenario, and this is a second, standalone instance living beside the new
/// attacker-side probes for a reader who only opens this file.
fn test_dx21_blockers_side_guard_unaffected() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareBlockers)
        .object(ObjectSpec::creature(p1, "T8 Attacker", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p2, "T8 Blocker", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let attacker = find_by_name(&state, "T8 Attacker");
    let blocker = find_by_name(&state, "T8 Blocker");
    *state.combat_mut() = Some(CombatState {
        attacking_player: p1,
        attackers: [(attacker, AttackTarget::Player(p2))].into_iter().collect(),
        blockers: imbl::OrdMap::new(),
        damage_assignment_order: imbl::OrdMap::new(),
        first_strike_participants: imbl::OrdSet::new(),
        attackers_declared: true,
        defenders_declared: imbl::OrdSet::new(),
        forced_blocks: imbl::OrdMap::new(),
        enlist_pairings: Vec::new(),
        blocked_attackers: imbl::OrdSet::new(),
        exerted_attackers: imbl::OrdSet::new(),
    });
    state.turn_mut().priority_holder = Some(p2);
    let _ = blocker;

    // p2 declares no blockers (valid first declaration).
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("first blocker declaration should succeed");

    // p2 tries to declare again: still rejected, unaffected by the new
    // attacker-side guard.
    let err = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect_err("re-declaring blockers should still be rejected");
    assert!(
        matches!(err, GameStateError::AlreadyDeclaredBlockers(pid) if pid == p2),
        "expected AlreadyDeclaredBlockers(p2), got: {err:?}"
    );
}

//! PB-DX51 (`scutemob-226`) — CR 508.8: the declare-blockers/combat-damage skip is a
//! HISTORICAL fact about what was declared or entered this combat, not a step-END read
//! of what survives in `combat.attackers`; plus CR 509.1a offer soundness (`b1`, in
//! `crates/simulator/tests/pb_dx51_blocker_offer.rs`) and the `OOS-DX21-5` init move
//! (`x1`, below). `memory/primitives/pb-plan-DX51.md` is authoritative; §2 is this
//! file's probe table.
//!
//! # The reproduction is `reconnaissance`, not a synthetic map mutation
//!
//! `crates/card-defs/src/defs/reconnaissance.rs` is `Completeness::Complete` and deck-legal:
//! *"{0}: Remove target attacking creature you control from combat and untap it."* — a real,
//! free, instant-speed activated ability whose effect is `Effect::RemoveFromCombat`
//! (CR 506.4, "an effect specifically removes it from combat"). `rules::combat::
//! remove_from_combat` itself is `pub(crate)` and unreachable from a `tests/` binary; its
//! only two production callers are this ability and `apply_regeneration` (CR 701.19a). t1,
//! t2 and t5 all drive Reconnaissance for real through `Command::ActivateAbility`, so the
//! defect these probes pin is live on a real corpus card, not a hand-poked map.
//!
//! **Correction recorded, not silently worked around**: `OOS-DX21-4`'s own recipe ("kill it
//! / phase it out / stop it being a creature with an instant") does not reproduce through any
//! of the three actual `combat.attackers`-emptying sites (`remove_from_combat`'s two callers
//! above, plus a THIRD, undocumented raw `combat.attackers.remove(..)` on the Ninjutsu bounce
//! path at `rules/abilities.rs:2361`) — none of those three routes is death, a zone move that
//! is not `RemoveFromCombat`/regeneration, phasing, or a Layer-4 type change. The engine has
//! **no** CR 506.4 cleanup at all on death, an ordinary zone move, phasing out, or a type
//! change that strips creature-ness; a stale attacker `ObjectId` (CR 400.7) simply stays in
//! `combat.attackers` under every one of those four causes, so the map is never emptied that
//! way and this batch's pre-fix skip cannot be reproduced through them. Reported back rather
//! than pinned as a probe: see this batch's final report for the corresponding filing.

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AttackTarget,
    CardDefinition, CardRegistry, CombatState, Command, GameEvent, GameState, GameStateBuilder,
    ObjectId, ObjectSpec, PlayerId, Step, Target, ZoneId,
};
use std::collections::HashMap;
use std::sync::Arc;

// ── Helpers (mirrors pb_dx21_declare_attackers_once_per_combat.rs /
//    pb_dx27_stale_blocker_repairs.rs) ──────────────────────────────────────

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

/// A `Command::DeclareAttackers` with every optional-choice field empty.
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

/// Reconnaissance's `{0}` activated ability (index 0 -- its only ability), targeting
/// `target_id`. Free, so no mana pool setup is required.
fn activate_reconnaissance(player: PlayerId, source: ObjectId, target_id: ObjectId) -> Command {
    Command::ActivateAbility {
        player,
        source,
        ability_index: 0,
        targets: vec![Target::Object(target_id)],
        discard_card: None,
        sacrifice_target: None,
        x_value: None,
        modes_chosen: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }
}

/// Pass priority for whoever holds it, repeatedly, until `stop` holds.
/// Mirrors `pb_dx21_declare_attackers_once_per_combat.rs::advance_until` exactly.
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
             (step={:?}, phase={:?}, stack_len={})",
            state.turn().step,
            state.turn().phase,
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

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Resolve the whole stack (mirrors `pb_dx27_stale_blocker_repairs.rs::resolve_stack`).
fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players).0;
    }
    state
}

// ── t1 — AC (a), the headline: one attacker, removed mid-step, steps NOT skipped ──

#[test]
/// CR 508.8 / CR 506.4 (PB-DX51, `OOS-DX21-4`, plan §2 t1): declare ONE attacker, then
/// remove it from combat at instant speed (Reconnaissance's real `{0}` ability, CR 506.4)
/// while the declare-attackers step is still open. CR 508.8's predicate is about what
/// **was declared this combat**, not what survives to step end — the declare-blockers and
/// combat-damage steps must still occur.
fn test_dx51_lone_attacker_removed_mid_step_does_not_skip() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let recon = enrich(p1, "Reconnaissance", ZoneId::Battlefield, &defs);
    let attacker = ObjectSpec::creature(p1, "T1 Attacker", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(recon)
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let recon_id = find_by_name(&state, "Reconnaissance");
    let attacker_id = find_by_name(&state, "T1 Attacker");

    // (1) CR 508.1: declare the lone attacker.
    let (state, _events) = process_command(
        state,
        declare_cmd(p1, vec![(attacker_id, AttackTarget::Player(p2))]),
    )
    .expect("declaring one attacker must succeed");
    assert!(
        state.combat().as_ref().unwrap().had_attackers,
        "precondition: had_attackers must be set by the declaration"
    );

    // (2) CR 506.4, mid-step: activate Reconnaissance targeting the attacker.
    let (state, _events) =
        process_command(state, activate_reconnaissance(p1, recon_id, attacker_id))
            .expect("Reconnaissance's {0} ability should activate for free");
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        state.combat().as_ref().unwrap().attackers.is_empty(),
        "CR 506.4: the attacker must be removed from combat.attackers"
    );
    assert!(
        state.combat().as_ref().unwrap().had_attackers,
        "CR 508.8: had_attackers is monotone -- the CR 506.4 removal must not clear it \
         (revert row R2 is what this line discriminates)"
    );

    // (3) Drive to the end of the DeclareAttackers step and assert what CR 508.8
    // actually asks: is the SKIP taken.
    let (state, _ev) = advance_until(state, 40, |s| s.turn().step != Step::DeclareAttackers);

    assert_eq!(
        state.turn().step,
        Step::DeclareBlockers,
        "CR 508.8: a creature WAS declared this combat -- declare-blockers must not be \
         skipped even though combat.attackers is now empty (the pre-PB-DX51 defect read \
         `attackers.is_empty()` at step end and would have jumped straight to EndOfCombat \
         here)"
    );
}

// ── t2 — AC (a), the CONSEQUENCE: partial removal, block+damage actually happen ────

#[test]
/// CR 509.1a / CR 510 (PB-DX51, plan §2 t2): the attacking player declares TWO attackers
/// and only ONE is removed mid-step. Because the surviving attacker keeps
/// `combat.attackers` non-empty, this scenario does **not** by itself discriminate the
/// CR 508.8 skip predicate the way t1 does (`!c.had_attackers && c.attackers.is_empty()`
/// is false either way once B is still present) — see this batch's final report for the
/// disclosure against the plan's revert-matrix prediction. What this probe DOES prove,
/// and what t1's fixture structurally cannot (t1's only attacker is the one removed, so
/// nothing is left to block or damage): that after a mid-step CR 506.4 removal, the
/// SURVIVING attacker's block is actually registered and combat damage is actually
/// dealt — "the step occurred" and "the blocks and damage happened" are different claims.
fn test_dx51_partial_removal_survivor_block_and_damage_actually_happen() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let recon = enrich(p1, "Reconnaissance", ZoneId::Battlefield, &defs);
    // C is 0 power / 4 toughness so it survives the trade with a pinned, non-lethal
    // damage_marked value -- an SBA-triggered death here would make the post-damage
    // assertion meaningless (a dead object's damage_marked is no longer observable
    // the same way, and a fresh ObjectId would be minted on the zone move, CR 400.7).
    let a = ObjectSpec::creature(p1, "T2 Attacker A", 2, 2).in_zone(ZoneId::Battlefield);
    let b = ObjectSpec::creature(p1, "T2 Attacker B", 2, 2).in_zone(ZoneId::Battlefield);
    let c = ObjectSpec::creature(p2, "T2 Blocker C", 0, 4).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(recon)
        .object(a)
        .object(b)
        .object(c)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let recon_id = find_by_name(&state, "Reconnaissance");
    let a_id = find_by_name(&state, "T2 Attacker A");
    let b_id = find_by_name(&state, "T2 Attacker B");
    let c_id = find_by_name(&state, "T2 Blocker C");

    // (1) Declare BOTH A and B attacking p2.
    let (state, _events) = process_command(
        state,
        declare_cmd(
            p1,
            vec![
                (a_id, AttackTarget::Player(p2)),
                (b_id, AttackTarget::Player(p2)),
            ],
        ),
    )
    .expect("declaring two attackers must succeed");

    // (2) Remove ONLY A, mid-step, via Reconnaissance.
    let (state, _events) = process_command(state, activate_reconnaissance(p1, recon_id, a_id))
        .expect("Reconnaissance should activate for free");
    let state = resolve_stack(state, &[p1, p2]);

    assert_eq!(
        state.combat().as_ref().unwrap().attackers.len(),
        1,
        "A removed, B remains attacking -- combat.attackers is non-empty (documented in \
         this test's own doc: this is why t2 cannot discriminate the skip predicate by \
         itself)"
    );
    assert!(
        state
            .combat()
            .as_ref()
            .unwrap()
            .attackers
            .contains_key(&b_id),
        "B must be the surviving attacker"
    );
    assert!(
        !state
            .combat()
            .as_ref()
            .unwrap()
            .attackers
            .contains_key(&a_id),
        "A must actually be gone"
    );

    // (3) Drive to DeclareBlockers.
    let (state, _ev) = advance_until(state, 40, |s| s.turn().step != Step::DeclareAttackers);
    assert_eq!(
        state.turn().step,
        Step::DeclareBlockers,
        "non-vacuity precondition for the rest of this probe: we must actually reach \
         DeclareBlockers or there is nothing left to assert"
    );

    // (4) p2 actually blocks B with C -- a real Command::DeclareBlockers, the
    // consequence, not just the step name.
    let (state, _events) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(c_id, b_id)],
        },
    )
    .expect("p2's block of the surviving attacker must be accepted");
    assert_eq!(
        state.combat().as_ref().unwrap().blockers.get(&c_id),
        Some(&b_id),
        "CR 509.1a: the block must actually be REGISTERED, not merely offered"
    );

    // (5) Drive through to CombatDamage. Turn-based actions (including
    // `combat_damage_step`) run synchronously as part of the same step-transition
    // call (`engine.rs::enter_step`), so damage is already marked once this returns.
    let (state, _ev) = advance_until(state, 40, |s| s.turn().step != Step::DeclareBlockers);
    assert_eq!(
        state.turn().step,
        Step::CombatDamage,
        "no first/double strike creatures in this fixture -- the regular CombatDamage \
         step must be next"
    );

    assert_eq!(
        state
            .object(c_id)
            .expect("blocker C is still on the battlefield -- toughness 4 survives 2 damage")
            .damage_marked,
        2,
        "CR 510.1c/510.2: combat damage must actually be DEALT, not merely have the step \
         occur -- the surviving attacker B (power 2) must mark exactly 2 damage on its \
         blocker C"
    );
}

// ── t3 — AC (b): an EMPTY declaration still skips (the PB-DX21 pin, protected) ─────

#[test]
/// CR 508.1a / CR 508.8 (PB-DX51, plan §2 t3): declaring **zero** attackers is a
/// completed CR 508.1 turn-based action ("if any") and CR 508.8 still demands the skip.
/// This is PB-DX21's `attackers_declared` pin, wrong-way-round protection for PB-DX51's
/// new `had_attackers` field: proving the fix did NOT become "never skip".
fn test_dx51_empty_declaration_still_skips() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p1, "T3 Bystander", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    // (1) An EMPTY declaration.
    let (state, _events) = process_command(state, declare_cmd(p1, vec![]))
        .expect("CR 508.1a: declaring no attackers is legal");
    assert!(
        !state.combat().as_ref().unwrap().had_attackers,
        "an EMPTY declaration must never enter CombatState::add_attacker's loop, so \
         had_attackers stays clear"
    );
    assert!(
        state.combat().as_ref().unwrap().attackers_declared,
        "precondition: the empty declaration IS a completed CR 508.1 turn-based action"
    );

    // (2) Drive to the end of the DeclareAttackers step.
    let (state, _ev) = advance_until(state, 20, |s| s.turn().step != Step::DeclareAttackers);

    assert_eq!(
        state.turn().step,
        Step::EndOfCombat,
        "CR 508.1a/508.8: no creature was declared as an attacker or put onto the \
         battlefield attacking -- declare-blockers and combat-damage must still be \
         skipped for an EMPTY declaration"
    );
}

// ── t4 — AC (c): a CR 508.4 entrant with NO declaration -> steps NOT skipped ───────

#[test]
/// CR 508.4 / CR 508.8 (PB-DX51, plan §2 t4): a creature put onto the battlefield
/// attacking (CR 508.4) with **no** CR 508.1 declaration at all (`attackers_declared`
/// stays `false`) must still prevent the skip -- even once THAT entrant is itself
/// removed from combat before the step ends, leaving `combat.attackers` empty with
/// `attackers_declared` false throughout the whole combat.
///
/// **Why the removal matters, not just the entry**: an entrant that is never removed
/// leaves `combat.attackers` non-empty at step end, which the PRE-PB-DX51 predicate
/// (`attackers.is_empty()` alone) already got right -- that shape does not discriminate
/// the fix (verified empirically: an earlier draft of this test without the removal
/// stayed GREEN under revert row R1). Combining the CR 508.4 entry with a REAL CR 506.4
/// removal (Reconnaissance, same route as t1/t2/t5) is what isolates `had_attackers` as
/// the ONLY thing keeping the skip from firing here -- `attackers_declared` is false
/// throughout (ruling out PB-DX21's field) and `attackers.is_empty()` is true at step
/// end (ruling out the pre-fix predicate).
///
/// **`CombatState::add_attacker` is called directly for the ENTRY half, and this says
/// so rather than hiding it**: there is no cheap real production route to "a CR 508.4
/// entrant with NO declaration this combat" -- both real CR 508.4 mechanisms in the
/// corpus (Myriad, Ninjutsu) fire off, or require, an attacker that WAS declared via CR
/// 508.1 first, so driving either one through a real card would leave
/// `attackers_declared == true` and stop being a "no declaration at all" fixture.
/// `add_attacker` is the same mutator all four real CR 508.4 production sites route
/// through; `r1` (`crates/engine/tests/core/pb_dx51_attacker_entry_roster.rs`) is what
/// polices those four sites, not this probe. The REMOVAL half is real production code
/// (`Effect::RemoveFromCombat` via Reconnaissance's `{0}` ability, `Command::
/// ActivateAbility`).
fn test_dx51_cr_508_4_entrant_without_declaration_does_not_skip() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let recon = enrich(p1, "Reconnaissance", ZoneId::Battlefield, &defs);
    let entrant = ObjectSpec::creature(p1, "T4 CR508.4 Entrant", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(recon)
        .object(entrant)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let recon_id = find_by_name(&state, "Reconnaissance");
    let entrant_id = find_by_name(&state, "T4 CR508.4 Entrant");

    // (1) No Command::DeclareAttackers is ever submitted -- the entrant is installed
    // directly via the ONE mutator every real CR 508.4 site routes through.
    *state.combat_mut() = Some(CombatState::new(p1));
    state
        .combat_mut()
        .as_mut()
        .unwrap()
        .add_attacker(entrant_id, AttackTarget::Player(p2));

    assert!(
        !state.combat().as_ref().unwrap().attackers_declared,
        "precondition: no CR 508.1 declaration was ever performed this combat"
    );
    assert!(
        state.combat().as_ref().unwrap().had_attackers,
        "precondition: add_attacker must mark had_attackers even with no declaration"
    );

    // (2) Remove the entrant, mid-step, via Reconnaissance's real `{0}` ability
    // (CR 506.4) -- the same production route as t1/t2/t5.
    let (state, _events) =
        process_command(state, activate_reconnaissance(p1, recon_id, entrant_id))
            .expect("Reconnaissance should activate for free");
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        state.combat().as_ref().unwrap().attackers.is_empty(),
        "precondition: the entrant is actually gone from combat.attackers"
    );
    assert!(
        !state.combat().as_ref().unwrap().attackers_declared,
        "precondition: still no CR 508.1 declaration happened, at any point in this combat"
    );
    assert!(
        state.combat().as_ref().unwrap().had_attackers,
        "precondition: had_attackers must still be set (monotone) after the removal"
    );

    // (3) Drive to the end of the DeclareAttackers step.
    let (state, _ev) = advance_until(state, 40, |s| s.turn().step != Step::DeclareAttackers);

    assert_eq!(
        state.turn().step,
        Step::DeclareBlockers,
        "CR 508.4/508.8: a creature was put onto the battlefield attacking with NO CR \
         508.1 declaration at all -- even after THAT entrant is itself removed from \
         combat, the skip must not fire"
    );
}

// ── t5 — had_attackers survives a real CR 506.4 removal (monotone) ─────────────────

#[test]
/// CR 506.4 / CR 508.8 (PB-DX51, plan §2 t5): `had_attackers` is monotone -- a real
/// CR 506.4 removal (Reconnaissance, same route as t1/t2) must not clear it. This is
/// the field-level companion to t1's step-level assertion.
fn test_dx51_had_attackers_survives_removal() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let recon = enrich(p1, "Reconnaissance", ZoneId::Battlefield, &defs);
    let attacker = ObjectSpec::creature(p1, "T5 Attacker", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(recon)
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let recon_id = find_by_name(&state, "Reconnaissance");
    let attacker_id = find_by_name(&state, "T5 Attacker");

    let (state, _events) = process_command(
        state,
        declare_cmd(p1, vec![(attacker_id, AttackTarget::Player(p2))]),
    )
    .expect("declaring one attacker must succeed");
    assert!(
        state.combat().as_ref().unwrap().had_attackers,
        "precondition: had_attackers is set by the declaration"
    );

    let (state, _events) =
        process_command(state, activate_reconnaissance(p1, recon_id, attacker_id))
            .expect("Reconnaissance should activate for free");
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        state.combat().as_ref().unwrap().attackers.is_empty(),
        "precondition: the attacker is actually gone from combat.attackers"
    );
    assert!(
        state.combat().as_ref().unwrap().had_attackers,
        "CR 508.8: had_attackers is monotone -- a real CR 506.4 removal (Reconnaissance) \
         must not clear the marker that a declaration happened this combat"
    );
}

// ── t6 — a fresh combat phase starts with had_attackers == false ───────────────────

#[test]
/// CR 500.8 / CR 506.5 (PB-DX51, plan §2 t6): a fresh `CombatState`, installed by the
/// real `begin_combat` production path (not a synthetic `CombatState::new` call in
/// isolation), starts with `had_attackers == false`. A companion declaration then
/// flips it, proving the field is load-bearing rather than dead-always-false.
fn test_dx51_fresh_combat_phase_starts_with_had_attackers_false() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(ObjectSpec::creature(p1, "T6 Attacker", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    // Drive through the PreCombatMain -> BeginningOfCombat transition for real; the
    // step-entry `begin_combat` turn-based action installs a fresh CombatState as
    // part of that same transition (`engine.rs::enter_step`).
    let (state, _ev) = advance_until(state, 20, |s| s.turn().step == Step::BeginningOfCombat);

    assert!(
        state.combat().is_some(),
        "non-vacuity: begin_combat must have installed a CombatState by the time we \
         observe BeginningOfCombat, or the assertion below is vacuous"
    );
    assert!(
        !state.combat().as_ref().unwrap().had_attackers,
        "CR 500.8/506.5: a freshly-installed CombatState must start with had_attackers \
         clear"
    );

    // Companion: drive into DeclareAttackers and actually declare, proving the field
    // is not dead-always-false.
    let (state, _ev) = advance_until(state, 20, |s| s.turn().step == Step::DeclareAttackers);
    let attacker_id = find_by_name(&state, "T6 Attacker");
    let (state, _events) = process_command(
        state,
        declare_cmd(p1, vec![(attacker_id, AttackTarget::Player(p2))]),
    )
    .expect("declaring must succeed");
    assert!(
        state.combat().as_ref().unwrap().had_attackers,
        "the SAME field must flip true once a real declaration happens in THIS combat"
    );
}

// ── x1 — AC 7324 (OOS-DX21-5): a refused DeclareAttackers leaves state.combat alone ─

#[test]
/// CR 508.1 (PB-DX51, `OOS-DX21-5`, plan §1.5 / §2 x1): a `DeclareAttackers` refused by
/// the per-attacker validation loop leaves `state.combat` exactly as it found it.
///
/// **Deliberately calls `mtg_engine::rules::combat::handle_declare_attackers` directly
/// (`&mut GameState`), not `process_command`** -- `process_command`'s `Err` arm carries
/// no `GameState`, so any mutation a rejected call would have made (pre-fix or
/// post-fix) is discarded by Rust's ownership model regardless of where in the callee
/// it happened, making an assertion through `process_command` structurally VACUOUS
/// (`OOS-DX21-7`, and this file's own revert row R5b demonstrates it directly).
fn test_dx51_refused_declaration_leaves_combat_state_untouched() {
    let p1 = p(1);
    let p2 = p(2);

    // (1) A refused declaration: the attacker is TAPPED (no Vigilance), which the
    // per-attacker validation loop rejects with PermanentAlreadyTapped -- one of
    // several rejection arms that all sit BELOW the CombatState init this test pins.
    let mut refused_state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(
            ObjectSpec::creature(p1, "X1 Tapped Creature", 2, 2)
                .in_zone(ZoneId::Battlefield)
                .tapped(),
        )
        .build()
        .unwrap();
    refused_state.turn_mut().priority_holder = Some(p1);
    let tapped_id = find_by_name(&refused_state, "X1 Tapped Creature");

    assert!(
        refused_state.combat().is_none(),
        "precondition: no CombatState exists before the (about-to-be-refused) call"
    );

    let err = mtg_engine::rules::combat::handle_declare_attackers(
        &mut refused_state,
        p1,
        vec![(tapped_id, AttackTarget::Player(p2))],
        vec![],
        vec![],
        vec![],
        vec![],
    )
    .expect_err("a tapped, non-Vigilance creature cannot be declared as an attacker");
    assert!(
        matches!(err, mtg_engine::GameStateError::PermanentAlreadyTapped(id) if id == tapped_id),
        "expected PermanentAlreadyTapped, got: {err:?}"
    );

    assert!(
        refused_state.combat().is_none(),
        "OOS-DX21-5: a REFUSED declaration must leave state.combat exactly as it found \
         it -- CR 732, \"the game returns to the moment before the declaration\""
    );

    // (2) A companion on a FRESH state: the ACCEPTED path still installs CombatState,
    // so this probe cannot be satisfied by an engine that simply never installs
    // CombatState at all.
    let mut accepted_state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(ObjectSpec::creature(p1, "X1 Legal Attacker", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();
    accepted_state.turn_mut().priority_holder = Some(p1);
    let legal_id = find_by_name(&accepted_state, "X1 Legal Attacker");

    assert!(
        accepted_state.combat().is_none(),
        "precondition: no CombatState exists before the (about-to-succeed) call"
    );

    let _events = mtg_engine::rules::combat::handle_declare_attackers(
        &mut accepted_state,
        p1,
        vec![(legal_id, AttackTarget::Player(p2))],
        vec![],
        vec![],
        vec![],
        vec![],
    )
    .expect("a legal, untapped, non-summoning-sick creature must be declarable");

    assert!(
        accepted_state.combat().is_some(),
        "the ACCEPTED path must still install a CombatState -- this probe must not be \
         satisfiable by an engine that never installs one at all"
    );
}

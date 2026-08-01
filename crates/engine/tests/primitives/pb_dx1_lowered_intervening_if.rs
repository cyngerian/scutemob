//! PB-DX1 (OOS-DP6-1): the intervening-if dropped in the runtime lowering.
//!
//! `build_face_ability_vectors` (`crates/engine/src/testing/replay_harness.rs`)
//! hardcodes `intervening_if: None` at all 34 push sites, so a card-def
//! intervening-if (CR 603.4) is checked at NEITHER the queue end
//! (`rules/abilities.rs::check_intervening_if`, 13 call sites) NOR the
//! resolution end (`rules/resolution.rs:2378`) for a lowered trigger. Aurelia,
//! the Warleader ("Whenever Aurelia attacks for the first time each turn, ...")
//! is `Complete`, deck-legal, and — because `Condition::IsFirstCombatPhase` is
//! never evaluated — grants herself an UNBOUNDED chain of extra combats: every
//! combat she attacks in re-triggers the ability.
//!
//! T1 (this file): the mandatory fail-before probe (plan `pb-plan-DX1.md` §8.1).
//! Production code is NOT touched in this commit — this test MUST fail against
//! pre-fix HEAD.

use mtg_engine::rules::engine::process_command;
use mtg_engine::{
    all_cards, enrich_spec_from_def, AttackTarget, CardDefinition, CardId, CardRegistry, Command,
    GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;

// ── Helpers (mirrors tests/primitives/pb_rs3_at_beginning_of_combat_sweep.rs and
//    tests/combat/additional_combat.rs::pass_until_step_advance) ───────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

fn load_defs_from(defs: &[CardDefinition]) -> HashMap<String, CardDefinition> {
    defs.iter().map(|d| (d.name.clone(), d.clone())).collect()
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found in state"))
}

/// Pass priority through whoever currently holds it, repeatedly, until `stop`
/// is satisfied. Handles BOTH stack resolution (all active players passing
/// with an unchanged stack resolves the top item) and step advancement (all
/// active players passing with an empty stack advances the step) uniformly —
/// there is no other legal action available at any point this test drives
/// through, so a single generic priority-pass loop is the correct engine-driven
/// idiom (mirrors `tests/combat/additional_combat.rs::pass_until_step_advance`,
/// generalised from "stop on step change" to an arbitrary predicate).
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

fn count_aurelia_triggers(events: &[GameEvent], aurelia_id: ObjectId) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                    if *source_object_id == aurelia_id
            )
        })
        .count()
}

/// CR 603.4 (intervening-if, both ends) / CR 508.3a (whenever ~ attacks) / CR
/// 500.8 (additional combat phase, CR 500.10a).
///
/// Aurelia, the Warleader: "Whenever Aurelia attacks for the first time each
/// turn, untap all creatures you control. After this phase, there is an
/// additional combat phase." Authored as `WhenAttacks` +
/// `intervening_if: Some(Condition::IsFirstCombatPhase)` — a `Complete`,
/// deck-legal def (`crates/card-defs/src/defs/aurelia_the_warleader.rs`).
///
/// Drives the REAL card def through the REAL engine: `Command::DeclareAttackers`
/// in the first combat, drain the stack (untap + grant one extra combat), drive
/// priority through DeclareBlockers/CombatDamage/EndOfCombat into the extra
/// combat's DeclareAttackers step, and declare Aurelia as an attacker again.
///
/// Pre-fix: `intervening_if` is dropped at BOTH ends of the lowering, so the
/// second declaration re-triggers the ability unconditionally — Aurelia grants
/// herself a THIRD combat. This assertion must FAIL against pre-fix HEAD.
#[test]
fn test_dx1_aurelia_attack_trigger_fires_exactly_once_per_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let all = all_cards();
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let aurelia_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Aurelia, the Warleader")
            .with_card_id(cid("aurelia-the-warleader"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let buddy_spec = ObjectSpec::creature(p1, "Test Attack Buddy", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(aurelia_spec)
        .object(buddy_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let aurelia_id = find_by_name(&state, "Aurelia, the Warleader");

    // ── Combat 1: declare Aurelia. CR 508.3a / 603.2 — this is her first
    //    attack this turn, so the trigger must queue (both pre- and post-fix). ──
    let (state, declare1_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(aurelia_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("combat 1 DeclareAttackers failed: {e:?}"));

    let mut all_events = declare1_events;
    assert_eq!(
        count_aurelia_triggers(&all_events, aurelia_id),
        1,
        "Aurelia's first attack this turn must queue the WhenAttacks trigger"
    );

    // Drive priority through: draining the trigger's resolution (untap + grant
    // one extra combat), then DeclareBlockers -> CombatDamage -> EndOfCombat
    // (which redirects into the extra combat via `turn.additional_phases`) ->
    // BeginningOfCombat -> the extra combat's DeclareAttackers step.
    let (state, ev) = advance_until(state, 60, |s| {
        s.turn().step == Step::DeclareAttackers
            && s.turn().in_extra_combat
            && s.stack_objects().is_empty()
    });
    all_events.extend(ev);

    assert_eq!(
        state.turn().additional_phases.len(),
        0,
        "the single extra combat granted by combat 1's trigger should already be \
         consumed (redirected into) by the time we reach its DeclareAttackers step"
    );

    // ── Combat 2 (the extra combat granted by combat 1's trigger): declare
    //    Aurelia again. She has vigilance, so she is untapped and can attack. ──
    let (state, declare2_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(aurelia_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("combat 2 DeclareAttackers failed: {e:?}"));
    all_events.extend(declare2_events);

    // Drain any resulting stack activity (pre-fix: the wrongly-queued second
    // trigger resolving, which would grant a THIRD combat).
    let (state, ev) = advance_until(state, 30, |s| s.stack_objects().is_empty());
    all_events.extend(ev);

    // CR 603.4 sentence 1 (queue-time gate, post-fix): the trigger must NOT
    // queue a second time — Aurelia's second attack this turn is not "the
    // first time" per `Condition::IsFirstCombatPhase`.
    assert_eq!(
        count_aurelia_triggers(&all_events, aurelia_id),
        1,
        "Aurelia's WhenAttacks trigger must fire exactly ONCE across the whole \
         turn (CR 603.4): pre-fix, the second attack (in the extra combat SHE \
         granted) re-triggers it unconditionally because the lowering drops the \
         intervening-if at both ends, producing an unbounded chain of extra \
         combats"
    );
    // CR 500.8 / 500.10a: no third combat should have been granted.
    assert_eq!(
        state.turn().additional_phases.len(),
        0,
        "no third combat phase should be queued -- Aurelia's second attack (in \
         the extra combat) must not re-trigger the ability"
    );
}

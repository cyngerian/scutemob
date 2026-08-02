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
//! T1: the mandatory fail-before probe (plan `pb-plan-DX1.md` §8.1). T2-T9
//! (below) are the rest of §8.2's table, landed once the fix itself has
//! landed (phase 5).

use mtg_engine::rules::abilities::{check_intervening_if, InterveningIfMoment};
use mtg_engine::rules::engine::process_command;
use mtg_engine::state::game_object::InterveningIf;
use mtg_engine::{
    all_cards, enrich_spec_from_def, AbilityDefinition, AttackTarget, CardDefinition, CardFace,
    CardId, CardRegistry, CardType, Color, Command, Condition, CounterType, Effect, EffectAmount,
    FaceDownKind, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, SubType, TriggerCondition,
    TurnFaceUpMethod, TypeLine, ZoneId,
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

/// Each named player passes priority once, in order (standard 2-player
/// resolve/advance idiom used throughout the test suite).
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

/// CR 603.2h (once each turn) / CR 508.3a (whenever ~ attacks) / CR 500.8
/// (additional combat phase, CR 500.10a).
///
/// Aurelia, the Warleader: "Whenever Aurelia attacks for the first time each
/// turn, untap all creatures you control. After this phase, there is an
/// additional combat phase." Authored as `WhenAttacks` + `once_per_turn: true`,
/// no `intervening_if` — a `Complete`, deck-legal def
/// (`crates/card-defs/src/defs/aurelia_the_warleader.rs`).
///
/// PB-DX1 review Finding 1: this test was originally written against a
/// `WhenAttacks` + `intervening_if: Some(Condition::IsFirstCombatPhase)`
/// authoring, and its assertions (fires exactly once, one extra combat, no
/// third) still hold byte-for-byte against the `once_per_turn: true`
/// authoring the finding required — the suppression mechanism changed
/// (CR 603.2h's once-per-turn gate, not CR 603.4's intervening-if gate) but
/// the observable behavior this test pins does not. CR 603.4 is still
/// exercised end-to-end elsewhere: T2/T3/T5/T12b.
///
/// Drives the REAL card def through the REAL engine: `Command::DeclareAttackers`
/// in the first combat, drain the stack (untap + grant one extra combat), drive
/// priority through DeclareBlockers/CombatDamage/EndOfCombat into the extra
/// combat's DeclareAttackers step, and declare Aurelia as an attacker again.
///
/// Pre-PB-DX1: `intervening_if` was dropped at BOTH ends of the lowering (this
/// card's ORIGINAL authoring, before the review fix), so the second declaration
/// re-triggered the ability unconditionally — Aurelia granted herself a THIRD
/// combat. Verified against pre-fix HEAD (`abilities.rs`/`resolution.rs`/
/// `replay_harness.rs` reverted to before Phase 0-4 landed) — FAILED with:
///
/// ```text
/// assertion `left == right` failed: Aurelia's WhenAttacks trigger must fire
/// exactly ONCE across the whole turn (CR 603.4): pre-fix, the second attack
/// (in the extra combat SHE granted) re-triggers it unconditionally because
/// the lowering drops the intervening-if at both ends, producing an unbounded
/// chain of extra combats
///   left: 2
///  right: 1
/// ```
///
/// The once_per_turn gate now independently produces the same "exactly once"
/// result for the correct CR 603.2h reason (review Finding 1).
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("combat 2 DeclareAttackers failed: {e:?}"));
    all_events.extend(declare2_events);

    // Drain any resulting stack activity (pre-fix: the wrongly-queued second
    // trigger resolving, which would grant a THIRD combat).
    let (state, ev) = advance_until(state, 30, |s| s.stack_objects().is_empty());
    all_events.extend(ev);

    // CR 603.2c/603.2h (once-per-turn gate): the trigger must NOT queue a
    // second time — Aurelia already has an ability-fired mark for this turn
    // from combat 1's attack.
    assert_eq!(
        count_aurelia_triggers(&all_events, aurelia_id),
        1,
        "Aurelia's WhenAttacks trigger must fire exactly ONCE across the whole \
         turn (CR 603.2h): pre-PB-DX1, the second attack (in the extra combat \
         SHE granted) re-triggered it unconditionally because the lowering \
         dropped both intervening_if and once_per_turn, producing an unbounded \
         chain of extra combats"
    );
    // CR 500.8 / 500.10a: no third combat should have been granted.
    assert_eq!(
        state.turn().additional_phases.len(),
        0,
        "no third combat phase should be queued -- Aurelia's second attack (in \
         the extra combat) must not re-trigger the ability"
    );
}

/// PB-DX1 review Finding 1: the scenario the def's ORIGINAL `WhenAttacks` +
/// `intervening_if: Some(Condition::IsFirstCombatPhase)` authoring got wrong.
/// Aurelia's FIRST attack of the turn happens in an extra combat GRANTED BY
/// ANOTHER SOURCE (Aggravated Assault / Moraug / World at War / Port Razer are
/// real Commander-legal examples; simulated directly via `turn.in_extra_combat`
/// -- mirrors the control-group idiom `tests/combat/additional_combat.rs` uses
/// for exactly this "which mechanism granted the extra combat" question, and
/// is the right tool here specifically because this probe is about Aurelia's
/// own trigger's response to already being in an extra combat, independent of
/// how the game got there).
///
/// Oracle: "Whenever Aurelia attacks for the first time each turn" -- this
/// literally is her first attack this turn, so the real card triggers.
/// `IsFirstCombatPhase` (`!turn.in_extra_combat`) instead asks "is this the
/// turn's first combat phase at all" and would read false here, wrongly
/// suppressing the trigger -- the exact HIGH finding. `once_per_turn: true`
/// tracks Aurelia's own attack history, not which combat phase this is, and
/// fires correctly.
///
/// Verified to FAIL against the old `IsFirstCombatPhase` authoring: reverted
/// the card def to `once_per_turn: false` / `intervening_if:
/// Some(Condition::IsFirstCombatPhase)` and re-ran -- the trigger never
/// queues (`count == 0`), confirming this probe catches exactly Finding 1's
/// failure mode.
#[test]
fn test_dx1_aurelia_first_attack_in_an_extra_combat_still_triggers() {
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

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(aurelia_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    // Some OTHER source already granted this combat as an extra combat, before
    // Aurelia has attacked at all this turn.
    state.turn_mut().in_extra_combat = true;

    let aurelia_id = find_by_name(&state, "Aurelia, the Warleader");

    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(aurelia_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("DeclareAttackers failed: {e:?}"));

    assert_eq!(
        count_aurelia_triggers(&declare_events, aurelia_id),
        1,
        "Aurelia's first attack of the turn must trigger even when it happens \
         in an extra combat granted by another source (CR 603.2h: 'for the \
         first time each turn' tracks HER attack history, not which combat \
         phase of the turn this is)"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn().additional_phases.len(),
        1,
        "the trigger must still resolve and grant its own additional combat phase"
    );
}

// ── T2-T9: the rest of plan §8.2's table ────────────────────────────────────

fn cid2(s: &str) -> CardId {
    CardId(s.to_string())
}

fn attacker_def_with_iif(card_id: &str, name: &str, iif: Condition, gain: i32) -> CardDefinition {
    CardDefinition {
        card_id: cid2(card_id),
        name: name.to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Whenever this attacks, if <condition>, gain life.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenAttacks,
            intervening_if: Some(iif),
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(gain),
            },
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}

/// Build a 2-player state with a single attacker-def creature on p1's
/// battlefield, at `Step::DeclareAttackers`, p1 active. Returns (state,
/// attacker_id, p1, p2).
fn build_attacker_state(
    def: CardDefinition,
    life: i32,
) -> (GameState, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let name = def.name.clone();
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut defs = HashMap::new();
    defs.insert(def.name.clone(), def);

    let spec = enrich_spec_from_def(
        ObjectSpec::card(p1, &name)
            .with_card_id(cid2(&name.to_lowercase().replace([' ', ','], "-")))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .player_life(p1, life)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let id = find_by_name(&state, &name);
    (state, id, p1, p2)
}

fn declare_attack(
    state: GameState,
    p1: PlayerId,
    p2: PlayerId,
    id: ObjectId,
) -> (GameState, Vec<GameEvent>) {
    process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("DeclareAttackers failed: {e:?}"))
}

fn life_of(state: &GameState, pid: PlayerId) -> i32 {
    state.players().get(&pid).map(|p| p.life_total).unwrap()
}

/// CR 603.4 sentence 1: a lowered card-def condition that is FALSE at trigger
/// time must gate the ability -- it never reaches the stack.
/// Pre-fix (`intervening_if: None` hardcoded): FAILS -- the trigger queues
/// unconditionally regardless of the (dropped) condition.
#[test]
fn test_dx1_lowered_condition_gates_at_queue_time() {
    // Life starts at 40 (builder default via `player_life`); 999 is unreachable.
    let def = attacker_def_with_iif(
        "dx1-t2-false-cond",
        "DX1 T2 False Cond Attacker",
        Condition::ControllerLifeAtLeast(999),
        5,
    );
    let (state, id, p1, p2) = build_attacker_state(def, 40);
    let (state, _) = declare_attack(state, p1, p2, id);

    assert!(
        state.stack_objects().is_empty(),
        "a lowered WhenAttacks trigger whose card-def intervening-if is FALSE at \
         declaration must NOT reach the stack (CR 603.4 s1); stack: {:?}",
        state.stack_objects()
    );
}

/// The non-regression twin of T2: a card-def condition that is TRUE at
/// declaration DOES queue and DOES resolve with its full effect. Passes
/// before and after (the pre-fix code queues unconditionally, so it would
/// also queue here "for the wrong reason" -- the value of this test is
/// pinning that the fix does not accidentally over-suppress).
#[test]
fn test_dx1_lowered_condition_true_still_fires() {
    let def = attacker_def_with_iif(
        "dx1-t4-true-cond",
        "DX1 T4 True Cond Attacker",
        Condition::ControllerLifeAtLeast(0),
        5,
    );
    let (state, id, p1, p2) = build_attacker_state(def, 40);
    let (state, declare_events) = declare_attack(state, p1, p2, id);

    assert_eq!(
        state.stack_objects().len(),
        1,
        "a TRUE card-def intervening-if must still queue the trigger (CR 603.4 s1)"
    );
    assert!(
        declare_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { source_object_id, .. } if *source_object_id == id)),
        "AbilityTriggered must be emitted for the queued trigger"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        life_of(&state, p1),
        45,
        "the ability must resolve with its full effect when the condition still holds"
    );
}

/// CR 603.4 sentence 2 -- THE test proving the fix is not queue-only (PB-DP6's
/// review caught exactly this shape at `resolution.rs:2299`: gated the queue
/// end, left the resolution end unconditional). Condition is TRUE at
/// declaration (queues) and FALSE at resolution (must resolve with NO
/// effect). Pre-fix: FAILS -- the effect executes regardless (the runtime
/// `InterveningIf` was always `None`, so BOTH check_intervening_if call sites
/// treat it as vacuously true).
#[test]
fn test_dx1_lowered_condition_rechecked_at_resolution() {
    let def = attacker_def_with_iif(
        "dx1-t3-flip-cond",
        "DX1 T3 Flip Cond Attacker",
        Condition::ControllerLifeAtLeast(20),
        5,
    );
    let (state, id, p1, p2) = build_attacker_state(def, 20);
    let (mut state, _) = declare_attack(state, p1, p2, id);

    assert_eq!(
        state.stack_objects().len(),
        1,
        "life 20 >= 20 must queue the trigger at declaration"
    );

    // Flip the condition false BETWEEN queue time and resolution -- direct
    // state mutation is the correct isolation tool here (unlike T1, this is
    // not the headline engine-driven probe; it exists specifically to prove
    // the RESOLUTION-time recheck runs in isolation from the queue-time gate).
    state.players_mut().get_mut(&p1).unwrap().life_total = 10;

    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "the ability must still resolve (be removed from the stack), just with no effect"
    );
    assert_eq!(
        life_of(&state, p1),
        10,
        "CR 603.4 s2: the condition is false at resolution (life 10 < 20), so the \
         ability's effect must NOT execute -- life must stay at 10, not 15"
    );
}

/// Hard constraint 3, at BOTH ends: a lowered trigger whose card-def condition
/// is one of the seven queue-time-unevaluable `Condition` variants must NEVER
/// be suppressed -- neither at the queue end nor (per Edit 2's Resolution arm)
/// at the resolution end. `Condition::TargetIsLegal { index: 0 }` is used with
/// no declared targets, which is nonsensical to evaluate and therefore exactly
/// the case `condition_is_queue_time_evaluable` exists to guard.
#[test]
fn test_dx1_unevaluable_condition_does_not_suppress() {
    let def = attacker_def_with_iif(
        "dx1-t5-unevaluable",
        "DX1 T5 Unevaluable Cond Attacker",
        Condition::TargetIsLegal { index: 0 },
        5,
    );
    let (state, id, p1, p2) = build_attacker_state(def, 40);
    let (state, _) = declare_attack(state, p1, p2, id);

    assert_eq!(
        state.stack_objects().len(),
        1,
        "an unevaluable condition must never suppress at the queue end (hard constraint 3)"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        life_of(&state, p1),
        45,
        "an unevaluable condition must never suppress at the resolution end either"
    );
}

/// CR 603.10a + hard constraint 3: a card-def intervening-if on a
/// leave-the-battlefield trigger (`WhenDies` -> row 1, `TriggerTimeLookBack`)
/// must NOT be evaluated against the current state -- the source has already
/// left, so a source-scoped condition like `SourceOnBattlefield` would read
/// false and wrongly suppress a trigger CR 603.4 requires to fire. Pins
/// §4.3's deviation in the OVER-firing direction: if a later change collapses
/// `InterveningIfMoment` to two values and evaluates this condition for real
/// at queue time, this test reddens.
#[test]
fn test_dx1_lookback_dies_trigger_not_suppressed() {
    let p1 = p(1);
    let p2 = p(2);

    let def = CardDefinition {
        card_id: cid2("dx1-t6-lookback-dies"),
        name: "DX1 T6 Lookback Dies".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "When this dies, if it's on the battlefield (LKI-impossible), gain 2 life."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenDies,
            intervening_if: Some(Condition::SourceOnBattlefield),
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };
    let mut defs = HashMap::new();
    defs.insert(def.name.clone(), def.clone());
    let registry = CardRegistry::new(vec![def.clone()]);

    let spec = enrich_spec_from_def(
        ObjectSpec::card(p1, &def.name)
            .with_card_id(cid2("dx1-t6-lookback-dies"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let obj_id = find_by_name(&state, &def.name);
    // Reduce toughness to 0 -> CR 704.5f SBA death (mirrors
    // tests/rules/trigger_variants.rs::test_when_leaves_battlefield_fires_on_death).
    state
        .objects_mut()
        .get_mut(&obj_id)
        .unwrap()
        .characteristics
        .toughness = Some(0);

    let initial_life = life_of(&state, p1);
    let (state, ev1) = pass_all(state, &[p1, p2]);
    let (state, ev2) = pass_all(state, &[p1, p2]);
    let all_events: Vec<GameEvent> = ev1.into_iter().chain(ev2).collect();

    let triggered_count = all_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 1,
        "a card-def WhenDies trigger whose intervening-if is a source-scoped \
         condition must still queue (CR 603.10a look-back carve-out; hard \
         constraint 3) -- events: {all_events:?}"
    );
    // Review Finding 2: the queue end queuing the trigger is only half the
    // claim -- the carve-out is only real end-to-end if the effect actually
    // executes at resolution. `InterveningIfMoment::ResolutionLookBack`
    // (below) makes this hold: a look-back trigger's card-def condition is
    // never re-evaluated against the current (post-move) state, at EITHER
    // end, mirroring the existing `SourceHadNoCounterOfType` precedent.
    assert_eq!(
        life_of(&state, p1),
        initial_life + 2,
        "the WhenDies trigger must not just queue but actually RESOLVE with its \
         effect -- CR 603.10a's look-back carve-out is only real if it holds at \
         BOTH ends, not just the one that puts the trigger on the stack"
    );
}

/// Regression pin for Edit 2: the two LEGACY `InterveningIf` variants
/// (`ControllerLifeAtLeast`, `SourceHadNoCounterOfType`) must answer
/// IDENTICALLY at all four `InterveningIfMoment` values -- their match arms
/// never read `moment` at all, so this is the direct proof that adding
/// `source`/`moment` to `check_intervening_if`'s signature did not change
/// legacy behavior. Includes `ResolutionLookBack` (review Finding 2).
#[test]
fn test_dx1_legacy_intervening_if_variants_unchanged() {
    let p1 = p(1);
    let p2 = p(2);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .build()
        .unwrap();
    // Life defaults to 40 (GameStateBuilder::add_player).
    let dummy_source = ObjectId(1);
    let moments = [
        InterveningIfMoment::TriggerTime,
        InterveningIfMoment::TriggerTimeLookBack,
        InterveningIfMoment::Resolution,
        InterveningIfMoment::ResolutionLookBack,
    ];

    for &m in &moments {
        assert!(
            check_intervening_if(
                &state,
                &InterveningIf::ControllerLifeAtLeast(30),
                p1,
                dummy_source,
                None,
                m,
                &[],
            ),
            "life 40 >= 30 must hold at {m:?}"
        );
        assert!(
            !check_intervening_if(
                &state,
                &InterveningIf::ControllerLifeAtLeast(50),
                p1,
                dummy_source,
                None,
                m,
                &[],
            ),
            "life 40 >= 50 must NOT hold at {m:?}"
        );
    }

    let mut counters = imbl::OrdMap::new();
    counters.insert(CounterType::PlusOnePlusOne, 2u32);

    for &m in &[
        InterveningIfMoment::TriggerTime,
        InterveningIfMoment::TriggerTimeLookBack,
        InterveningIfMoment::ResolutionLookBack,
    ] {
        assert!(
            !check_intervening_if(
                &state,
                &InterveningIf::SourceHadNoCounterOfType(CounterType::PlusOnePlusOne),
                p1,
                dummy_source,
                Some(&counters),
                m,
                &[],
            ),
            "the creature HAS +1/+1 counters, so 'had no counter' must be false at {m:?}"
        );
        assert!(
            check_intervening_if(
                &state,
                &InterveningIf::SourceHadNoCounterOfType(CounterType::Stun),
                p1,
                dummy_source,
                Some(&counters),
                m,
                &[],
            ),
            "the creature has no Stun counters, so 'had no counter' must be true at {m:?}"
        );
    }

    // At resolution, callers pass `None` for pre_death_counters by convention
    // (the source is in the graveyard with no counters) -- unconditionally true.
    // Both Resolution and ResolutionLookBack must agree here (the legacy variant
    // ignores `moment` entirely).
    for &m in &[
        InterveningIfMoment::Resolution,
        InterveningIfMoment::ResolutionLookBack,
    ] {
        assert!(check_intervening_if(
            &state,
            &InterveningIf::SourceHadNoCounterOfType(CounterType::PlusOnePlusOne),
            p1,
            dummy_source,
            None,
            m,
            &[],
        ));
    }
}

/// PB-OS4b/PB-RS4 contract, extended to the new lowering: a DFC's BACK face
/// carries its own `WhenAttacks` + card-def intervening-if, and after
/// `apply_face_change` (driven here via `Command::Transform`) the back
/// face's condition gates a declared attack. This test cannot be run
/// against pre-fix HEAD -- `InterveningIf::CardDef` does not exist there --
/// so it is a forward regression pin, not a fail-before probe (plan §8.2:
/// "passes before vacuously").
#[test]
fn test_dx1_face_change_carries_back_face_condition() {
    let p1 = p(1);
    let p2 = p(2);

    let def = CardDefinition {
        card_id: cid2("dx1-t8-dfc"),
        name: "DX1 T8 DFC Front".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Transform".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Transform)],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "DX1 T8 DFC Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Horror".to_string())].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "Whenever this attacks, if you have 999 or more life, gain 5 life."
                .to_string(),
            abilities: vec![AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                intervening_if: Some(Condition::ControllerLifeAtLeast(999)),
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(5),
                },
                targets: vec![],
                modes: None,
                trigger_zone: None,
            }],
            power: Some(4),
            toughness: Some(4),
            color_indicator: Some(vec![Color::Black]),
        }),
        ..Default::default()
    };
    let mut defs = HashMap::new();
    defs.insert(def.name.clone(), def.clone());
    let registry = CardRegistry::new(vec![def.clone()]);

    let spec = enrich_spec_from_def(
        ObjectSpec::card(p1, &def.name)
            .with_card_id(cid2("dx1-t8-dfc"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let obj_id = find_by_name(&state, "DX1 T8 DFC Front");
    assert!(!state.objects()[&obj_id].is_transformed);

    let (state, _) = process_command(
        state,
        Command::Transform {
            player: p1,
            permanent: obj_id,
        },
    )
    .expect("Transform should succeed");
    assert!(state.objects()[&obj_id].is_transformed);

    // Structural: the back face's runtime TriggeredAbilityDef must carry the
    // card-def condition through the lowering.
    let triggered = &state.objects()[&obj_id].characteristics.triggered_abilities;
    let back_face_trigger = triggered
        .iter()
        .find(|t| t.trigger_on == mtg_engine::TriggerEvent::SelfAttacks)
        .unwrap_or_else(|| panic!("no SelfAttacks trigger on the back face: {triggered:?}"));
    assert_eq!(
        back_face_trigger.intervening_if,
        Some(InterveningIf::CardDef(Box::new(
            Condition::ControllerLifeAtLeast(999)
        ))),
        "the back face's card-def intervening-if must survive apply_face_change's rebuild"
    );

    // Functional: the (false) condition actually gates a declared attack on the
    // NOW-back-faced permanent.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(obj_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("DeclareAttackers failed: {e:?}"));
    assert!(
        state.stack_objects().is_empty(),
        "the back face's FALSE intervening-if must gate its WhenAttacks trigger"
    );
}

/// SR-36: the roster of LOWERED-condition card defs carrying `intervening_if`
/// / `once_per_turn` is derived by enumerating `all_cards()` via
/// `effective_abilities` (both faces), never by grep. Becomes the permanent
/// gate that a newly-authored such def cannot land unnoticed.
#[test]
fn test_dx1_corpus_roster_is_enumerated_not_grepped() {
    fn is_lowered(tc: &TriggerCondition) -> bool {
        matches!(
            tc,
            TriggerCondition::WhenDies
                | TriggerCondition::WhenAttacks
                | TriggerCondition::WhenBlocks
                | TriggerCondition::WhenDealsCombatDamageToPlayer
                | TriggerCondition::WhenDealtDamage
                | TriggerCondition::WheneverOpponentCastsSpell { .. }
                | TriggerCondition::WheneverYouSurveil
                | TriggerCondition::WhenConnives
                | TriggerCondition::WheneverYouInvestigate
                | TriggerCondition::WheneverYouCastSpell { .. }
                | TriggerCondition::WheneverCreatureEntersBattlefield { .. }
                | TriggerCondition::WheneverPermanentEntersBattlefield { .. }
                | TriggerCondition::WhenMutates
                | TriggerCondition::WhenSelfBecomesTapped
                | TriggerCondition::WheneverPermanentUntaps { .. }
                | TriggerCondition::WhenCounterPlaced { .. }
                | TriggerCondition::WheneverCreatureDies { .. }
                | TriggerCondition::WheneverCreatureYouControlAttacks { .. }
                | TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer { .. }
                | TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { .. }
                | TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer
                | TriggerCondition::WhenEquippedCreatureDealsCombatDamage
                | TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer { .. }
                | TriggerCondition::WhenAnyCreatureDealsCombatDamageToOpponent
                | TriggerCondition::WheneverYouDiscard
                | TriggerCondition::WheneverOpponentDiscards
                | TriggerCondition::WheneverOpponentPlaysLand
                | TriggerCondition::WheneverYouSacrifice { .. }
                | TriggerCondition::WheneverYouAttack { .. }
                | TriggerCondition::WhenLeavesBattlefield
                | TriggerCondition::WheneverYouDrawACard
                | TriggerCondition::WheneverPlayerDrawsCard { .. }
                | TriggerCondition::WheneverYouGainLife
                | TriggerCondition::WhenBecomesTarget { .. }
        )
    }

    let all = all_cards();
    let mut lowered_with_iif: Vec<String> = Vec::new();
    let mut lowered_with_once_per_turn: Vec<String> = Vec::new();
    let mut lowered_count = 0usize;
    let mut total_triggered_count = 0usize;

    for def in &all {
        for is_transformed in [false, true] {
            for ability in def.effective_abilities(is_transformed) {
                if let AbilityDefinition::Triggered {
                    trigger_condition,
                    intervening_if,
                    once_per_turn,
                    ..
                } = ability
                {
                    total_triggered_count += 1;
                    if is_lowered(trigger_condition) {
                        lowered_count += 1;
                        if intervening_if.is_some() {
                            lowered_with_iif.push(def.name.clone());
                        }
                        if *once_per_turn {
                            lowered_with_once_per_turn.push(def.name.clone());
                        }
                    }
                }
            }
        }
    }

    lowered_with_iif.sort();
    lowered_with_iif.dedup();
    lowered_with_once_per_turn.sort();
    lowered_with_once_per_turn.dedup();

    // SR-5: assert the denominator, not just the derived set.
    assert!(
        total_triggered_count > 0,
        "non-vacuity: the corpus must contain at least one AbilityDefinition::Triggered"
    );
    assert!(
        lowered_count > 0,
        "non-vacuity: the LOWERED classification must match at least one ability"
    );

    assert_eq!(
        lowered_with_iif,
        vec![
            "Karlach, Fury of Avernus".to_string(),
            "Tatyova, Steward of Tides".to_string(),
        ],
        "plan §6.2's expected roster on a LOWERED condition, derived by \
         enumeration, AMENDED by review Finding 1: aurelia_the_warleader.rs no \
         longer carries an intervening_if (re-authored once_per_turn: true, no \
         intervening_if -- see the card def's comment and T1/the new \
         extra-combat probe) -- if this list changes, a card def was added or \
         edited and §6's disposition table (and this test) needs review"
    );

    // once_per_turn on a LOWERED condition (§10 / Phase 7 of the plan -- FIXED,
    // separate commit): this is a census over the CARD-DEF-authored field, not
    // the runtime-propagated one, so it is unaffected by whether Phase 7's fix
    // has landed. Seven defs total: the 3 that always propagated correctly
    // (rows 15/16/17 -- morbid_opportunist/spiteful_banditry/dusk_legion_duelist),
    // the 3 Phase 7 repaired (welcoming_vampire/elvish_warmaster/
    // whispering_wizard -- see T13-T15, which drive these through the real
    // engine and confirm the runtime now honors this field), and Aurelia (review
    // Finding 1's re-authoring, amended here after Phase 7 landed).
    eprintln!("LOWERED x once_per_turn=true roster: {lowered_with_once_per_turn:?}");
    assert_eq!(
        lowered_with_once_per_turn,
        vec![
            "Aurelia, the Warleader".to_string(),
            "Dusk Legion Duelist".to_string(),
            "Elvish Warmaster".to_string(),
            "Morbid Opportunist".to_string(),
            "Spiteful Banditry".to_string(),
            "Welcoming Vampire".to_string(),
            "Whispering Wizard".to_string(),
        ],
        "the roster of LOWERED-condition defs authored with once_per_turn: true, \
         derived by enumeration -- if this list changes, a card def was added or \
         edited and both §10 (Phase 7 coverage) and T9 need review"
    );
}

// ── T10-T11: rider fixes (plan §9.1, §9.2) ──────────────────────────────────

/// CR 708.8 (WhenTurnedFaceUp) + a synthetic card-def intervening-if.
///
/// Latent: no corpus `WhenTurnedFaceUp` def carries an `intervening_if` (T9's
/// enumeration confirms zero such defs exist), so this rider's probe is
/// necessarily a SYNTHETIC def -- there is no real card to drive it with.
/// Card-def / setup pattern mirrors
/// `tests/mechanics_m_z/morph.rs::when_turned_face_up_def` /
/// `build_state_with_face_down_object`.
fn turn_face_up_def_with_iif(iif: Condition) -> CardDefinition {
    CardDefinition {
        card_id: cid2("dx1-t10-face-up"),
        name: "DX1 T10 Face-Up Trigger".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Morph {2}. When this creature is turned face up, if <condition>, draw a \
                      card."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Morph {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenTurnedFaceUp,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: Some(iif),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// OOS-DP6-5 / CR 603.4 s2 / CR 708.8: `resolution.rs`'s `TurnFaceUpTrigger`
/// arm executed its effect unconditionally, ignoring `intervening_if`
/// entirely -- while the QUEUE end (`abilities.rs`'s `PermanentTurnedFaceUp`
/// arm, PB-DP6's Category-A site A12) already gated correctly. Condition TRUE
/// at declaration (queues), flipped FALSE before resolution: the draw must
/// NOT happen. Pre-fix: FAILS -- the draw happens regardless (verified by
/// reverting `resolution.rs`'s `TurnFaceUpTrigger` arm and re-running: FAILED,
/// hand count off by one).
#[test]
fn test_dx1_turn_face_up_intervening_if_rechecked_at_resolution() {
    let p1 = p(1);
    let p2 = p(2);

    let def = turn_face_up_def_with_iif(Condition::ControllerLifeAtLeast(20));
    let registry = CardRegistry::new(vec![def.clone()]);

    let spec = ObjectSpec::card(p1, &def.name)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(def.card_id.clone())
        .with_types(vec![CardType::Creature]);
    let library_card = ObjectSpec::card(p1, "DX1 T10 Library Filler").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .object(library_card)
        .player_life(p1, 20)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Mark the permanent face-down as a morph (mirrors
    // morph.rs::build_state_with_face_down_object).
    let face_down_id = find_by_name(&state, &def.name);
    {
        let obj = state.objects_mut().get_mut(&face_down_id).unwrap();
        obj.characteristics.power = def.power;
        obj.characteristics.toughness = def.toughness;
        obj.status.face_down = true;
        obj.face_down_as = Some(FaceDownKind::Morph);
    }
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn_mut().priority_holder = Some(p1);

    let initial_hand = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let (mut state, _) = process_command(
        state,
        Command::TurnFaceUp {
            player: p1,
            permanent: face_down_id,
            method: TurnFaceUpMethod::MorphCost,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("TurnFaceUp should succeed");

    assert_eq!(
        state.stack_objects().len(),
        1,
        "life 20 >= 20 must queue the WhenTurnedFaceUp trigger (queue end already \
         gated correctly by PB-DP6's A12 -- this is the baseline, not the fix)"
    );

    // Flip the condition false BETWEEN queue time and resolution.
    state.players_mut().get_mut(&p1).unwrap().life_total = 10;

    let (state, _) = pass_all(state, &[p1, p2]);
    let final_hand = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        final_hand, initial_hand,
        "CR 603.4 s2 / OOS-DP6-5: the intervening-if is FALSE at resolution (life \
         10 < 20), so the WhenTurnedFaceUp trigger must resolve with NO effect -- \
         no card should be drawn"
    );
}

/// A synthetic Haunt creature whose `HauntedCreatureDies` ability carries a
/// card-def `intervening_if`. Latent (OOS-DP6-9): no corpus Haunt def carries
/// one, so this is necessarily synthetic -- mirrors
/// `tests/mechanics_e_l/haunt.rs::haunt_creature_def`.
fn haunt_def_with_iif(card_id: &str, name: &str, iif: Condition) -> CardDefinition {
    CardDefinition {
        card_id: cid2(card_id),
        name: name.to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Haunt. When the creature it haunts dies, if <condition>, gain 2 life."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haunt),
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::HauntedCreatureDies,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: Some(iif),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Drive a haunt creature through CR 702.55a/b: it dies, its HauntExileTrigger
/// resolves, and it ends up in exile haunting `target_name`. Returns the state
/// with the haunt card safely in exile (not yet triggered by the target's
/// death) and the target's `ObjectId`.
fn haunt_setup_to_exiled(
    def: CardDefinition,
    target_name: &str,
    p1: PlayerId,
    p2: PlayerId,
) -> (GameState, ObjectId) {
    let haunt_name = def.name.clone();
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, &haunt_name, 2, 2)
                .with_card_id(card_id)
                .with_keyword(KeywordAbility::Haunt)
                .with_damage(2), // lethal -> SBA kills it
        )
        .object(ObjectSpec::creature(p2, target_name, 2, 2))
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Step 1: SBA kills the haunt creature -> HauntExileTrigger queues.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Step 2: HauntExileTrigger resolves -> haunt card exiled, haunting_target set.
    let target_id = find_by_name(&state, target_name);
    let (state, _) = pass_all(state, &[p1, p2]);

    (state, target_id)
}

/// OOS-DP6-9 / CR 603.4, BOTH ends, CR 702.55c. Two independent scenarios:
///
/// Part A (queue end): the card-def intervening-if is FALSE the whole time.
/// Pre-fix, the queue site (`abilities.rs`'s haunt-exiled loop) never read
/// `intervening_if` at all, so `HauntedCreatureDiesTrigger` queued
/// unconditionally. Post-fix it must NOT queue.
///
/// Part B (resolution end): the condition is TRUE when the target dies
/// (queues) and flipped FALSE before resolution -- the effect must not fire.
/// Pre-fix, `resolution.rs`'s `find_map` never read `intervening_if` either,
/// so the effect always executed.
///
/// Both cited pre-fix mechanisms were verified to independently produce the
/// wrong outcome by reverting `resolution.rs`'s haunt arm and (for Part A)
/// `abilities.rs`'s haunt-exiled loop and re-running this test: both FAILED.
#[test]
fn test_dx1_haunt_intervening_if_gated_at_both_ends() {
    let p1 = p(1);
    let p2 = p(2);

    // ── Part A: queue end -- condition FALSE throughout. ──
    {
        let def = haunt_def_with_iif(
            "dx1-t11a-haunt",
            "DX1 T11a Haunt Creature",
            Condition::ControllerLifeAtLeast(999),
        );
        let (mut state, target_id) = haunt_setup_to_exiled(def, "DX1 T11a Target", p1, p2);

        // Kill the haunted target creature.
        let target_obj = state.objects_mut().get_mut(&target_id).unwrap();
        target_obj.damage_marked = 2;

        let (state, _) = pass_all(state, &[p1, p2]);
        assert!(
            state.stack_objects().is_empty(),
            "Part A (queue end): a FALSE card-def intervening-if must gate \
             HauntedCreatureDiesTrigger -- it must never reach the stack \
             (CR 603.4 s1, CR 702.55c); stack: {:?}",
            state.stack_objects()
        );
        // Review Finding 7: a suppressed trigger must still clear
        // `haunting_target` (CR 702.55c) -- otherwise the exiled card keeps
        // haunting a now-dead creature's ObjectId, the recycled-ObjectId
        // hazard `resolution.rs`'s own clear exists to prevent.
        let haunt_card_obj = state
            .objects()
            .values()
            .find(|o| o.characteristics.name == "DX1 T11a Haunt Creature")
            .unwrap_or_else(|| panic!("haunt card should still be in exile"));
        assert_eq!(
            haunt_card_obj.haunting_target, None,
            "a suppressed HauntedCreatureDiesTrigger must still clear \
             haunting_target on the exiled haunt card"
        );
    }

    // ── Part B: resolution end -- condition TRUE at queue time, flipped FALSE
    //    before resolution. ──
    {
        let def = haunt_def_with_iif(
            "dx1-t11b-haunt",
            "DX1 T11b Haunt Creature",
            Condition::ControllerLifeAtLeast(20),
        );
        let (mut state, target_id) = haunt_setup_to_exiled(def, "DX1 T11b Target", p1, p2);
        state.players_mut().get_mut(&p1).unwrap().life_total = 20;

        // Kill the haunted target creature -- condition TRUE, trigger queues.
        let target_obj = state.objects_mut().get_mut(&target_id).unwrap();
        target_obj.damage_marked = 2;
        let (mut state, _) = pass_all(state, &[p1, p2]);
        assert_eq!(
            state.stack_objects().len(),
            1,
            "Part B setup: life 20 >= 20 must queue HauntedCreatureDiesTrigger"
        );

        // Flip the condition false BETWEEN queue time and resolution.
        state.players_mut().get_mut(&p1).unwrap().life_total = 10;

        let (state, _) = pass_all(state, &[p1, p2]);
        assert_eq!(
            life_of(&state, p1),
            10,
            "Part B (resolution end): CR 603.4 s2 -- the condition is FALSE at \
             resolution (life 10 < 20), so the haunt effect (GainLife 2) must NOT \
             execute; life must stay at 10, not 12"
        );
    }
}

// ── T13-T15: the once_per_turn rider (plan §10) ─────────────────────────────
//
// `once_per_turn` was hardcoded `false` at 31 of the 34 lowering push sites
// (rows 15/16/17 already propagated it correctly, and were left unchanged).
// `flush_pending_triggers`'s once-per-turn gate (`abilities.rs`) reads the
// RUNTIME `characteristics.triggered_abilities[idx].once_per_turn` FIRST and
// only falls back to the card registry when that lookup MISSES -- for a
// lowered trigger it always HIT (with the wrong, hardcoded-false value), so
// the registry's true value was never consulted. No change to
// `flush_pending_triggers` itself was needed -- its gate logic was already
// correct; only the 31 lowering sites needed to stop discarding the field.
// Three `Complete`, deck-legal, REAL corpus defs over-fired as a result.

use mtg_engine::rules::command::CastSpellData;

fn count_in_zone(state: &GameState, zone: ZoneId) -> usize {
    state.objects().values().filter(|o| o.zone == zone).count()
}

fn count_named_on_battlefield(state: &GameState, name: &str) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == name)
        .count()
}

fn cast_spell(state: GameState, player: PlayerId, card: ObjectId) -> (GameState, Vec<GameEvent>) {
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {e:?}"))
}

fn vanilla_creature_def(
    card_id: &str,
    name: &str,
    power: i32,
    toughness: i32,
    subtypes: Vec<SubType>,
) -> CardDefinition {
    CardDefinition {
        card_id: cid2(card_id),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: subtypes.into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        power: Some(power),
        toughness: Some(toughness),
        abilities: vec![],
        ..Default::default()
    }
}

fn noncreature_instant_def(card_id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: cid2(card_id),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        // No `AbilityDefinition::Spell` needed -- this test only cares that
        // the spell is CAST (Whispering Wizard's trigger is `WheneverYouCastSpell`,
        // not a resolution effect); it resolves as a no-op and goes to the
        // graveyard.
        abilities: vec![],
        ..Default::default()
    }
}

/// CR 603.2c/603.2h — Welcoming Vampire ("...draw a card. This ability
/// triggers only once each turn."). Two qualifying ETB events (power <= 2
/// creatures entering under this player's control) in one turn must produce
/// exactly ONE draw, not two. Pre-fix: FAILS (fires twice — confirmed by T9's
/// enumeration that this def's card-def `once_per_turn: true` was silently
/// dropped by the lowering).
#[test]
fn test_dx1_once_per_turn_welcoming_vampire() {
    let p1 = p(1);
    let p2 = p(2);

    let mut all = all_cards();
    all.push(vanilla_creature_def(
        "dx1-t13-filler-a",
        "DX1 T13 Filler A",
        1,
        1,
        vec![],
    ));
    all.push(vanilla_creature_def(
        "dx1-t13-filler-b",
        "DX1 T13 Filler B",
        1,
        1,
        vec![],
    ));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let vampire_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Welcoming Vampire")
            .with_card_id(cid2("welcoming-vampire"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let filler_a = enrich_spec_from_def(
        ObjectSpec::card(p1, "DX1 T13 Filler A")
            .with_card_id(cid2("dx1-t13-filler-a"))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );
    let filler_b = enrich_spec_from_def(
        ObjectSpec::card(p1, "DX1 T13 Filler B")
            .with_card_id(cid2("dx1-t13-filler-b"))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );
    let mut library = Vec::new();
    for i in 0..5 {
        library.push(
            ObjectSpec::card(p1, &format!("DX1 T13 Library {i}")).in_zone(ZoneId::Library(p1)),
        );
    }

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(vampire_spec)
        .object(filler_a)
        .object(filler_b);
    for lib_obj in library {
        state = state.object(lib_obj);
    }
    let mut state = state
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn_mut().priority_holder = Some(p1);

    let initial_library = count_in_zone(&state, ZoneId::Library(p1));

    let filler_a_id = find_by_name(&state, "DX1 T13 Filler A");
    let (state, _) = cast_spell(state, p1, filler_a_id);
    let (state, _) = advance_until(state, 20, |s| s.stack_objects().is_empty());

    let filler_b_id = find_by_name(&state, "DX1 T13 Filler B");
    let (state, _) = cast_spell(state, p1, filler_b_id);
    let (state, _) = advance_until(state, 20, |s| s.stack_objects().is_empty());

    let final_library = count_in_zone(&state, ZoneId::Library(p1));
    assert_eq!(
        initial_library - final_library,
        1,
        "CR 603.2c/603.2h: Welcoming Vampire's draw must fire exactly ONCE this \
         turn even though two qualifying creatures entered -- library should \
         shrink by exactly 1, not 2"
    );
}

/// CR 603.2c/603.2h — Elvish Warmaster ("...create a 1/1 green Elf Warrior
/// creature token. This ability triggers only once each turn."). Two
/// qualifying Elf ETB events in one turn must create exactly ONE token.
///
/// This def's once_per_turn bug is WORSE than "fires an extra time": the
/// created token is ITSELF an Elf entering the battlefield under this
/// player's control, and `exclude_self` only excludes the trigger SOURCE
/// (Warmaster), not other Elves -- so an ungated trigger re-fires on its own
/// token, which creates another token, which re-fires again... Verified by
/// reverting `build_face_ability_vectors` and re-running: `advance_until`'s
/// 20-pass guard panics ("stop condition not reached") rather than a clean
/// assertion mismatch, because the pre-fix cascade does not terminate within
/// budget. once_per_turn is CR 603.2h's own mechanism for preventing exactly
/// this shape of self-reinforcing loop.
#[test]
fn test_dx1_once_per_turn_elvish_warmaster() {
    let p1 = p(1);
    let p2 = p(2);

    let elf_subtype = || vec![SubType("Elf".to_string())];
    let mut all = all_cards();
    all.push(vanilla_creature_def(
        "dx1-t14-filler-a",
        "DX1 T14 Elf Filler A",
        1,
        1,
        elf_subtype(),
    ));
    all.push(vanilla_creature_def(
        "dx1-t14-filler-b",
        "DX1 T14 Elf Filler B",
        1,
        1,
        elf_subtype(),
    ));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let warmaster_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Elvish Warmaster")
            .with_card_id(cid2("elvish-warmaster"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let filler_a = enrich_spec_from_def(
        ObjectSpec::card(p1, "DX1 T14 Elf Filler A")
            .with_card_id(cid2("dx1-t14-filler-a"))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );
    let filler_b = enrich_spec_from_def(
        ObjectSpec::card(p1, "DX1 T14 Elf Filler B")
            .with_card_id(cid2("dx1-t14-filler-b"))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(warmaster_spec)
        .object(filler_a)
        .object(filler_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn_mut().priority_holder = Some(p1);

    let filler_a_id = find_by_name(&state, "DX1 T14 Elf Filler A");
    let (state, _) = cast_spell(state, p1, filler_a_id);
    let (state, _) = advance_until(state, 20, |s| s.stack_objects().is_empty());

    let filler_b_id = find_by_name(&state, "DX1 T14 Elf Filler B");
    let (state, _) = cast_spell(state, p1, filler_b_id);
    let (state, _) = advance_until(state, 20, |s| s.stack_objects().is_empty());

    assert_eq!(
        count_named_on_battlefield(&state, "Elf Warrior"),
        1,
        "CR 603.2c/603.2h: Elvish Warmaster must create exactly ONE Elf Warrior \
         token this turn even though two qualifying Elves entered"
    );
}

/// CR 603.2c/603.2h — Whispering Wizard ("...create a 1/1 white Spirit
/// creature token with flying. This ability triggers only once each turn.").
/// Two noncreature spells cast in one turn must create exactly ONE token.
#[test]
fn test_dx1_once_per_turn_whispering_wizard() {
    let p1 = p(1);
    let p2 = p(2);

    let mut all = all_cards();
    all.push(noncreature_instant_def(
        "dx1-t15-instant-a",
        "DX1 T15 Instant A",
    ));
    all.push(noncreature_instant_def(
        "dx1-t15-instant-b",
        "DX1 T15 Instant B",
    ));
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let wizard_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Whispering Wizard")
            .with_card_id(cid2("whispering-wizard"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let instant_a = enrich_spec_from_def(
        ObjectSpec::card(p1, "DX1 T15 Instant A")
            .with_card_id(cid2("dx1-t15-instant-a"))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );
    let instant_b = enrich_spec_from_def(
        ObjectSpec::card(p1, "DX1 T15 Instant B")
            .with_card_id(cid2("dx1-t15-instant-b"))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(wizard_spec)
        .object(instant_a)
        .object(instant_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn_mut().priority_holder = Some(p1);

    let instant_a_id = find_by_name(&state, "DX1 T15 Instant A");
    let (state, _) = cast_spell(state, p1, instant_a_id);
    let (state, _) = advance_until(state, 20, |s| s.stack_objects().is_empty());

    let instant_b_id = find_by_name(&state, "DX1 T15 Instant B");
    let (state, _) = cast_spell(state, p1, instant_b_id);
    let (state, _) = advance_until(state, 20, |s| s.stack_objects().is_empty());

    assert_eq!(
        count_named_on_battlefield(&state, "Spirit"),
        1,
        "CR 603.2c/603.2h: Whispering Wizard must create exactly ONE Spirit \
         token this turn even though two noncreature spells were cast"
    );
}

// ── T12: Karlach, Fury of Avernus (plan §6.4 -- known_wrong -> Complete) ────
//
// MCP ruling (2022-06-10, #11): "Karlach doesn't have to be among the
// attacking creatures." Flipped from `WhenAttacks` (Karlach must personally
// attack) to `WheneverYouAttack { filter: None }` (CR 508.1 -- any attack by
// the controller), now expressible because PB-DX1 propagates `intervening_if`
// through BOTH rows' lowering.

/// MCP ruling #11 for Karlach, Fury of Avernus, driven through the REAL card
/// def: a creature OTHER than Karlach attacks (Karlach does not attack), and
/// her `WheneverYouAttack` trigger must still fire -- untapping the actual
/// attacker, granting it first strike, and creating an additional combat
/// phase. This is the exact defect the prior `known_wrong` note named; it is
/// unreachable under the old `WhenAttacks` (Karlach-must-personally-attack)
/// modelling.
#[test]
fn test_dx1_karlach_fires_without_personally_attacking() {
    let p1 = p(1);
    let p2 = p(2);

    let all = all_cards();
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let karlach_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Karlach, Fury of Avernus")
            .with_card_id(cid2("karlach-fury-of-avernus"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let buddy_spec = ObjectSpec::creature(p1, "DX1 T12 Buddy", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(karlach_spec)
        .object(buddy_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let karlach_id = find_by_name(&state, "Karlach, Fury of Avernus");
    let buddy_id = find_by_name(&state, "DX1 T12 Buddy");

    // Declare ONLY the buddy creature as an attacker -- Karlach herself does
    // not attack.
    let (state, declare_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(buddy_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("DeclareAttackers failed: {e:?}"));

    assert_eq!(
        count_aurelia_triggers(&declare_events, karlach_id),
        1,
        "MCP ruling #11: Karlach's WheneverYouAttack trigger must fire even \
         though she is not among the attacking creatures"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        !state.objects()[&buddy_id].status.tapped,
        "the actual attacker (buddy) should be untapped by Karlach's effect"
    );
    assert_eq!(
        state.turn().additional_phases.len(),
        1,
        "an additional combat phase should be granted"
    );
}

/// Plan §6.4 T12: mirrors T1's Aurelia shape for Karlach. Combat 1: Karlach
/// attacks personally (first combat phase this turn -> trigger fires, untaps
/// herself, grants an extra combat). Combat 2 (the extra combat she granted):
/// she attacks again (untapped by her own effect) -> the trigger must NOT
/// fire a second time -- no third combat.
#[test]
fn test_dx1_karlach_extra_combat_once_per_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let all = all_cards();
    let defs = load_defs_from(&all);
    let registry = CardRegistry::new(all);

    let karlach_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Karlach, Fury of Avernus")
            .with_card_id(cid2("karlach-fury-of-avernus"))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(karlach_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let karlach_id = find_by_name(&state, "Karlach, Fury of Avernus");

    let (state, declare1_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(karlach_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("combat 1 DeclareAttackers failed: {e:?}"));

    let mut all_events = declare1_events;
    assert_eq!(
        count_aurelia_triggers(&all_events, karlach_id),
        1,
        "Karlach's first attack this turn must queue the WheneverYouAttack trigger"
    );

    let (state, ev) = advance_until(state, 60, |s| {
        s.turn().step == Step::DeclareAttackers
            && s.turn().in_extra_combat
            && s.stack_objects().is_empty()
    });
    all_events.extend(ev);
    assert_eq!(
        state.turn().additional_phases.len(),
        0,
        "the single extra combat should already be consumed by the time we \
         reach its DeclareAttackers step"
    );

    let (state, declare2_events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(karlach_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("combat 2 DeclareAttackers failed: {e:?}"));
    all_events.extend(declare2_events);

    let (state, ev) = advance_until(state, 30, |s| s.stack_objects().is_empty());
    all_events.extend(ev);

    assert_eq!(
        count_aurelia_triggers(&all_events, karlach_id),
        1,
        "CR 603.4: Karlach's WheneverYouAttack trigger must fire exactly ONCE \
         across the whole turn -- her second attack (in the extra combat she \
         granted) must not re-trigger it"
    );
    assert_eq!(
        state.turn().additional_phases.len(),
        0,
        "no third combat phase should be queued"
    );
}

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

    let (state, ev1) = pass_all(state, &[p1, p2]);
    let (_state, ev2) = pass_all(state, &[p1, p2]);
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
}

/// Regression pin for Edit 2: the two LEGACY `InterveningIf` variants
/// (`ControllerLifeAtLeast`, `SourceHadNoCounterOfType`) must answer
/// IDENTICALLY at all three `InterveningIfMoment` values -- their match arms
/// never read `moment` at all, so this is the direct proof that adding
/// `source`/`moment` to `check_intervening_if`'s signature did not change
/// legacy behavior.
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
            ),
            "life 40 >= 50 must NOT hold at {m:?}"
        );
    }

    let mut counters = imbl::OrdMap::new();
    counters.insert(CounterType::PlusOnePlusOne, 2u32);

    for &m in &[
        InterveningIfMoment::TriggerTime,
        InterveningIfMoment::TriggerTimeLookBack,
    ] {
        assert!(
            !check_intervening_if(
                &state,
                &InterveningIf::SourceHadNoCounterOfType(CounterType::PlusOnePlusOne),
                p1,
                dummy_source,
                Some(&counters),
                m,
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
            ),
            "the creature has no Stun counters, so 'had no counter' must be true at {m:?}"
        );
    }

    // At resolution, callers pass `None` for pre_death_counters by convention
    // (the source is in the graveyard with no counters) -- unconditionally true.
    assert!(check_intervening_if(
        &state,
        &InterveningIf::SourceHadNoCounterOfType(CounterType::PlusOnePlusOne),
        p1,
        dummy_source,
        None,
        InterveningIfMoment::Resolution,
    ));
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
            "Aurelia, the Warleader".to_string(),
            "Karlach, Fury of Avernus".to_string(),
            "Tatyova, Steward of Tides".to_string(),
        ],
        "plan §6.2's expected 3-def roster (aurelia/karlach/tatyova) on a LOWERED \
         condition, derived by enumeration -- if this list changes, a card def was \
         added or edited and §6's disposition table (and this test) needs review"
    );

    // once_per_turn on a LOWERED condition is a SEPARATE bug (§10 / Phase 7 of the
    // plan, NOT fixed in this call's scope) -- this assertion pins the CURRENT
    // card-def-authored intent (independent of whether the runtime lowering
    // actually propagates it), i.e. it is a census of which defs the Phase 7
    // rider must cover, not a claim that the engine honors it yet.
    eprintln!("LOWERED x once_per_turn=true roster: {lowered_with_once_per_turn:?}");
    assert!(
        !lowered_with_once_per_turn.is_empty(),
        "non-vacuity: at least the 3 sites that already propagate once_per_turn \
         (rows 15/16/17) should be authored with once_per_turn: true on a card def"
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

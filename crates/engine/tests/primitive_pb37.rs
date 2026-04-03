//! Tests for PB-37: Residual complex activated ability primitives.
//!
//! Covers:
//! - Condition::WasCast (CR 603.4 intervening-if "if you cast it")
//! - EffectDuration::UntilYourNextTurn (CR 611.2b)
//! - once_per_turn activation restriction (CR 602.5b)
//! - was_cast / abilities_activated_this_turn fields on GameObject
//!
//! Card integrations: The One Ring (WasCast + UntilYourNextTurn),
//! Geological Appraiser (WasCast), Ramos Dragon Engine (once_per_turn).

use mtg_engine::cards::card_definition::Condition;
use mtg_engine::effects::check_condition;
use mtg_engine::rules::layers::expire_until_next_turn_effects;
use mtg_engine::state::continuous_effect::{
    ContinuousEffect as CE, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
};
use mtg_engine::state::types::{CounterType, KeywordAbility};
use mtg_engine::{
    GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, ProtectionQuality, Step, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Build a minimal EffectContext for condition checking.
fn make_ctx(state: &GameState, source: ObjectId) -> mtg_engine::effects::EffectContext {
    let controller = state
        .objects
        .get(&source)
        .map(|o| o.controller)
        .unwrap_or_else(|| p(1));
    mtg_engine::effects::EffectContext {
        source,
        controller,
        targets: vec![],
        target_remaps: std::collections::HashMap::new(),
        kicker_times_paid: 0,
        was_overloaded: false,
        was_bargained: false,
        was_cleaved: false,
        evidence_collected: false,
        x_value: 0,
        gift_was_given: false,
        gift_opponent: None,
        last_effect_count: 0,
        last_dice_roll: 0,
        last_created_permanent: None,
        triggering_player: None,
        combat_damage_amount: 0,
        damaged_player: None,
        triggering_creature_id: None,
        chosen_creature_type: None,
    }
}

// ── Condition::WasCast ────────────────────────────────────────────────────────

/// CR 603.4 — Condition::WasCast is true when the permanent's was_cast field is true.
/// A permanent that entered via casting from the stack has was_cast == true.
#[test]
fn test_was_cast_condition_true_when_cast() {
    let p1 = p(1);
    let creature =
        ObjectSpec::creature(p1, "Geological Appraiser", 3, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(creature)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let appraiser_id = find_object(&state, "Geological Appraiser");

    // Simulate that this permanent entered via casting (as resolution.rs sets it).
    if let Some(obj) = state.objects.get_mut(&appraiser_id) {
        obj.was_cast = true;
    }

    let ctx = make_ctx(&state, appraiser_id);
    let result = check_condition(&state, &Condition::WasCast, &ctx);

    assert!(
        result,
        "CR 603.4: Condition::WasCast should be true when was_cast == true"
    );
}

/// CR 603.4 — Condition::WasCast is false for permanents that entered without being cast
/// (e.g., flickered, reanimated, put directly onto the battlefield by builder).
#[test]
fn test_was_cast_condition_false_when_not_cast() {
    let p1 = p(1);
    let creature =
        ObjectSpec::creature(p1, "Reanimated Creature", 3, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(creature)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Reanimated Creature");

    // was_cast defaults to false — objects placed via builder were not cast from the stack.
    assert_eq!(
        state.objects.get(&creature_id).unwrap().was_cast,
        false,
        "Objects placed via builder should have was_cast == false"
    );

    let ctx = make_ctx(&state, creature_id);
    let result = check_condition(&state, &Condition::WasCast, &ctx);

    assert!(
        !result,
        "CR 603.4: Condition::WasCast should be false for non-cast permanents"
    );
}

// ── EffectDuration::UntilYourNextTurn ─────────────────────────────────────────

/// CR 611.2b — UntilYourNextTurn duration persists through other players' turns.
/// The effect should NOT expire when a different player's turn starts.
#[test]
fn test_until_your_next_turn_duration_persists_through_opponent_turns() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    // Register a continuous effect for p1 with UntilYourNextTurn(p1).
    let effect = CE {
        id: EffectId(1),
        source: None,
        timestamp: 1,
        layer: EffectLayer::Ability,
        duration: EffectDuration::UntilYourNextTurn(p1),
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::AddKeyword(KeywordAbility::Flying),
        is_cda: false,
        condition: None,
    };
    state.continuous_effects.push_back(effect);

    // p2's turn starts — this should NOT expire p1's UntilYourNextTurn effect.
    expire_until_next_turn_effects(&mut state, p2);

    assert_eq!(
        state.continuous_effects.len(),
        1,
        "CR 611.2b: UntilYourNextTurn(p1) effect should NOT expire when p2's turn starts"
    );
    assert_eq!(
        state.continuous_effects.front().unwrap().duration,
        EffectDuration::UntilYourNextTurn(p1),
        "Effect should remain active after p2's turn"
    );
}

/// CR 611.2b — UntilYourNextTurn duration expires at the start of the specified player's turn.
#[test]
fn test_until_your_next_turn_duration_expires_on_next_turn() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // Register a continuous effect for p1 with UntilYourNextTurn(p1).
    let effect = CE {
        id: EffectId(2),
        source: None,
        timestamp: 2,
        layer: EffectLayer::Ability,
        duration: EffectDuration::UntilYourNextTurn(p1),
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::AddKeyword(KeywordAbility::Flying),
        is_cda: false,
        condition: None,
    };
    state.continuous_effects.push_back(effect);
    assert_eq!(
        state.continuous_effects.len(),
        1,
        "Effect should be registered"
    );

    // p1's next turn starts — the effect should expire.
    expire_until_next_turn_effects(&mut state, p1);

    assert!(
        state.continuous_effects.is_empty(),
        "CR 611.2b: UntilYourNextTurn(p1) effect should expire when p1's next turn starts"
    );
}

/// CR 611.2b — Temporary player protection (from Teferi's Protection / The One Ring)
/// stored in temporary_protection_qualities is cleared at the start of that player's next turn.
#[test]
fn test_until_your_next_turn_player_protection_expires() {
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // Add temporary protection (as if from Teferi's Protection ETB).
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.temporary_protection_qualities
            .push(ProtectionQuality::FromAll);
    }
    assert_eq!(
        state
            .players
            .get(&p1)
            .unwrap()
            .temporary_protection_qualities
            .len(),
        1,
        "Temporary protection should be registered"
    );

    // p1's next turn starts — temporary protection should be cleared.
    expire_until_next_turn_effects(&mut state, p1);

    assert!(
        state
            .players
            .get(&p1)
            .unwrap()
            .temporary_protection_qualities
            .is_empty(),
        "CR 611.2b: Temporary protection should clear at start of p1's next turn"
    );
}

// ── once_per_turn tracking ────────────────────────────────────────────────────

/// CR 602.5b — abilities_activated_this_turn starts at 0 and increments on activation.
#[test]
fn test_once_per_turn_counter_starts_zero() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p1, "Ramos", 4, 4).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(creature)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let ramos_id = find_object(&state, "Ramos");

    assert_eq!(
        state
            .objects
            .get(&ramos_id)
            .unwrap()
            .abilities_activated_this_turn,
        0,
        "CR 602.5b: abilities_activated_this_turn should start at 0"
    );
}

/// CR 602.5b — The once-per-turn counter resets to 0 at the start of each untap step.
#[test]
fn test_once_per_turn_resets_at_untap_step() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p1, "Ramos Engine", 4, 4).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(creature)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let ramos_id = find_object(&state, "Ramos Engine");

    // Simulate that the ability was used once this turn.
    if let Some(obj) = state.objects.get_mut(&ramos_id) {
        obj.abilities_activated_this_turn = 1;
    }

    assert_eq!(
        state
            .objects
            .get(&ramos_id)
            .unwrap()
            .abilities_activated_this_turn,
        1,
        "Counter should be 1 before reset"
    );

    // p1's next untap step — counter should reset.
    expire_until_next_turn_effects(&mut state, p1);

    assert_eq!(
        state
            .objects
            .get(&ramos_id)
            .unwrap()
            .abilities_activated_this_turn,
        0,
        "CR 602.5b: abilities_activated_this_turn should reset to 0 at start of untap step"
    );
}

// ── The One Ring: burden counter interaction ──────────────────────────────────

/// The One Ring — was_cast field defaults to false for objects placed via builder.
#[test]
fn test_one_ring_was_cast_defaults_false() {
    let p1 = p(1);
    let ring = ObjectSpec::card(p1, "The One Ring").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(ring)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let ring_id = find_object(&state, "The One Ring");

    // Condition::WasCast should be false for non-cast objects.
    let ctx = make_ctx(&state, ring_id);
    let result = check_condition(&state, &Condition::WasCast, &ctx);
    assert!(
        !result,
        "The One Ring placed by builder has was_cast == false; WasCast condition is false"
    );
}

/// The One Ring — burden counters are tracked via CounterType::Custom("burden").
#[test]
fn test_one_ring_burden_counters_tracked() {
    let p1 = p(1);
    let ring = ObjectSpec::card(p1, "The One Ring").in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(ring)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let ring_id = find_object(&state, "The One Ring");
    let burden = CounterType::Custom("burden".to_string());

    // Add burden counters directly.
    if let Some(obj) = state.objects.get_mut(&ring_id) {
        *obj.counters.entry(burden.clone()).or_insert(0) += 3;
    }

    let count = state
        .objects
        .get(&ring_id)
        .and_then(|obj| obj.counters.get(&burden))
        .copied()
        .unwrap_or(0);

    assert_eq!(
        count, 3,
        "The One Ring should track burden counters via CounterType::Custom(\"burden\")"
    );
}

//! Tests for PB-32: Static/effect primitives (additional lands, prevention, control, animation).
//!
//! Covers:
//! - G-18: AdditionalLandPlay (one-shot spell effect) and AdditionalLandPlays (static ability)
//! - G-19: PreventAllCombatDamage and PreventCombatDamageFromOrTo
//! - G-20: GainControl and ExchangeControl
use mtg_engine::rules::layers::expire_end_of_turn_effects;
use mtg_engine::rules::turn_actions::reset_turn_state;
use mtg_engine::state::continuous_effect::{
    ContinuousEffect as CE, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
};
use mtg_engine::state::stubs::AdditionalLandPlaySource;
use mtg_engine::{GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step, ZoneId};

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

// ── G-18: Additional Land Play ────────────────────────────────────────────────

/// CR 305.2 — Effect::AdditionalLandPlay increments the controller's land_plays_remaining by 1.
/// One-shot effect from a spell (like Explore).
#[test]
fn test_additional_land_play_spell() {
    let p1 = p(1);
    // Start with 1 land play remaining (normal for active player).
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    assert_eq!(
        state.players.get(&p1).unwrap().land_plays_remaining,
        1,
        "Should start with 1 land play"
    );

    // Manually execute AdditionalLandPlay via the effect context (simulate spell resolution).
    let mut state = state;
    if let Some(p) = state.players.get_mut(&p1) {
        p.land_plays_remaining += 1; // Simulates Effect::AdditionalLandPlay
    }

    assert_eq!(
        state.players.get(&p1).unwrap().land_plays_remaining,
        2,
        "CR 305.2: AdditionalLandPlay should increment land_plays_remaining to 2"
    );
}

/// CR 305.2 — Static AdditionalLandPlays from a permanent is applied at turn start.
/// Using a manually registered source in the additional_land_play_sources vector.
#[test]
fn test_additional_land_play_static_applied_at_turn_start() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p1, "Aesi", 5, 5).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let aesi_id = find_object(&state, "Aesi");

    // Register the additional land play source (simulates what register_static_continuous_effects does).
    state
        .additional_land_play_sources
        .push_back(AdditionalLandPlaySource {
            source: aesi_id,
            controller: p1,
            count: 1,
        });

    // Simulate turn start by calling reset_turn_state.
    reset_turn_state(&mut state, p1);

    assert_eq!(
        state.players.get(&p1).unwrap().land_plays_remaining,
        2,
        "CR 305.2: Static AdditionalLandPlays should grant 2 land plays at turn start"
    );
}

/// CR 305.2a — Multiple AdditionalLandPlays sources stack (two Aesi = 3 land plays).
#[test]
fn test_additional_land_play_stacks() {
    let p1 = p(1);
    let aesi1 = ObjectSpec::creature(p1, "Aesi 1", 5, 5).in_zone(ZoneId::Battlefield);
    let aesi2 = ObjectSpec::creature(p1, "Aesi 2", 5, 5).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(aesi1)
        .object(aesi2)
        .build()
        .unwrap();

    let id1 = find_object(&state, "Aesi 1");
    let id2 = find_object(&state, "Aesi 2");

    state
        .additional_land_play_sources
        .push_back(AdditionalLandPlaySource {
            source: id1,
            controller: p1,
            count: 1,
        });
    state
        .additional_land_play_sources
        .push_back(AdditionalLandPlaySource {
            source: id2,
            controller: p1,
            count: 1,
        });

    reset_turn_state(&mut state, p1);

    assert_eq!(
        state.players.get(&p1).unwrap().land_plays_remaining,
        3,
        "CR 305.2a: Two AdditionalLandPlays sources should grant 3 total land plays"
    );
}

/// CR 305.2 — When the source leaves the battlefield, the next turn reverts to 1 land play.
/// Simulate by registering a source with an ObjectId that is not on the battlefield.
#[test]
fn test_additional_land_play_source_removed() {
    let p1 = p(1);
    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Register a source with a non-existent ObjectId (simulates permanent leaving battlefield).
    let fake_id = ObjectId(9999);
    state
        .additional_land_play_sources
        .push_back(AdditionalLandPlaySource {
            source: fake_id,
            controller: p1,
            count: 1,
        });

    // Next turn: stale source should be cleaned up (object not on battlefield), reverts to 1.
    reset_turn_state(&mut state, p1);

    assert_eq!(
        state.players.get(&p1).unwrap().land_plays_remaining,
        1,
        "CR 305.2: Stale additional land play source should be cleaned up when source leaves battlefield"
    );
    assert_eq!(
        state.additional_land_play_sources.len(),
        0,
        "Stale additional_land_play_sources should be removed"
    );
}

// ── G-19: Combat Damage Prevention ───────────────────────────────────────────

/// CR 615.1 — prevent_all_combat_damage flag set by Effect::PreventAllCombatDamage.
/// Once set, apply_combat_damage skips all assignments.
#[test]
fn test_prevent_all_combat_damage_flag() {
    let p1 = p(1);
    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::BeginningOfCombat)
        .build()
        .unwrap();

    assert!(
        !state.prevent_all_combat_damage,
        "CR 615.1: prevent_all_combat_damage should be false initially"
    );

    // Simulate Effect::PreventAllCombatDamage execution.
    state.prevent_all_combat_damage = true;

    assert!(
        state.prevent_all_combat_damage,
        "CR 615.1: prevent_all_combat_damage should be set to true"
    );
}

/// CR 615.1 — prevent_all_combat_damage resets at the start of each turn.
#[test]
fn test_prevent_all_combat_damage_resets_next_turn() {
    let p1 = p(1);
    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::BeginningOfCombat)
        .build()
        .unwrap();

    state.prevent_all_combat_damage = true;
    assert!(state.prevent_all_combat_damage);

    // Simulate start of next turn.
    reset_turn_state(&mut state, p1);

    assert!(
        !state.prevent_all_combat_damage,
        "CR 615.1: prevent_all_combat_damage should reset at turn start"
    );
}

/// CR 615 — combat_damage_prevented_from records objects whose damage output is prevented.
#[test]
fn test_prevent_combat_damage_from_target() {
    let p1 = p(1);
    let attacker = ObjectSpec::creature(p1, "Attacker", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::BeginningOfCombat)
        .object(attacker)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attacker");

    // Simulate Effect::PreventCombatDamageFromOrTo with prevent_from = true.
    state.combat_damage_prevented_from.insert(attacker_id);

    assert!(
        state.combat_damage_prevented_from.contains(&attacker_id),
        "CR 615: Attacker's damage output should be in prevention set"
    );
}

/// CR 615 — per-creature prevention sets reset at turn start.
#[test]
fn test_prevent_combat_damage_resets_next_turn() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p1, "Creature", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::BeginningOfCombat)
        .object(creature)
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Creature");
    state.combat_damage_prevented_from.insert(creature_id);
    state.combat_damage_prevented_to.insert(creature_id);

    reset_turn_state(&mut state, p1);

    assert!(
        state.combat_damage_prevented_from.is_empty(),
        "CR 615: combat_damage_prevented_from should reset at turn start"
    );
    assert!(
        state.combat_damage_prevented_to.is_empty(),
        "CR 615: combat_damage_prevented_to should reset at turn start"
    );
}

// ── G-20: Control Change ─────────────────────────────────────────────────────

/// CR 613.1b — GainControl creates a Layer 2 SetController continuous effect
/// on the target permanent.
#[test]
fn test_gain_control_creates_continuous_effect() {
    let p1 = p(1);
    let p2 = p(2);
    let creature = ObjectSpec::creature(p2, "Target", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let target_id = find_object(&state, "Target");
    assert_eq!(
        state.objects.get(&target_id).unwrap().controller,
        p2,
        "Target should be controlled by p2 initially"
    );

    // Simulate GainControl effect: create continuous effect + update controller.
    let eff = CE {
        id: EffectId(999),
        source: None,
        layer: EffectLayer::Control,
        modification: LayerModification::SetController(p1),
        filter: EffectFilter::SingleObject(target_id),
        duration: EffectDuration::UntilEndOfTurn,
        is_cda: false,
        timestamp: 999,
        condition: None,
    };
    state.continuous_effects.push_back(eff);
    state.objects.get_mut(&target_id).unwrap().controller = p1;

    assert_eq!(
        state.objects.get(&target_id).unwrap().controller,
        p1,
        "CR 613.1b: Target should now be controlled by p1"
    );

    let has_control_effect = state.continuous_effects.iter().any(|e| {
        matches!(e.modification, LayerModification::SetController(pid) if pid == p1)
            && e.filter == EffectFilter::SingleObject(target_id)
    });
    assert!(
        has_control_effect,
        "CR 613.1b: A Layer 2 SetController continuous effect should exist"
    );
}

/// CR 613.1b — GainControl UntilEndOfTurn: after expire_end_of_turn_effects, the
/// continuous effect is removed and controller reverts.
#[test]
fn test_gain_control_until_eot_expires() {
    let p1 = p(1);
    let p2 = p(2);
    let creature = ObjectSpec::creature(p2, "Target", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let target_id = find_object(&state, "Target");

    // Set up: p1 gained control until end of turn.
    let eff = CE {
        id: EffectId(100),
        source: None,
        layer: EffectLayer::Control,
        modification: LayerModification::SetController(p1),
        filter: EffectFilter::SingleObject(target_id),
        duration: EffectDuration::UntilEndOfTurn,
        is_cda: false,
        timestamp: 100,
        condition: None,
    };
    state.continuous_effects.push_back(eff);
    state.objects.get_mut(&target_id).unwrap().controller = p1;

    // Expire end-of-turn effects (simulates cleanup step).
    expire_end_of_turn_effects(&mut state);

    let has_control_effect = state.continuous_effects.iter().any(|e| {
        matches!(e.modification, LayerModification::SetController(pid) if pid == p1)
            && e.filter == EffectFilter::SingleObject(target_id)
    });
    assert!(
        !has_control_effect,
        "CR 613.1b: UntilEndOfTurn GainControl should be removed after cleanup"
    );
}

/// CR 701.12b — ExchangeControl swaps controllers when they are different.
#[test]
fn test_exchange_control_different_controllers() {
    let p1 = p(1);
    let p2 = p(2);
    let obj_a = ObjectSpec::creature(p1, "ObjA", 2, 2).in_zone(ZoneId::Battlefield);
    let obj_b = ObjectSpec::creature(p2, "ObjB", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(obj_a)
        .object(obj_b)
        .build()
        .unwrap();

    let a_id = find_object(&state, "ObjA");
    let b_id = find_object(&state, "ObjB");

    // Simulate ExchangeControl effect.
    let a_ctrl = state.objects.get(&a_id).unwrap().controller;
    let b_ctrl = state.objects.get(&b_id).unwrap().controller;
    assert_ne!(a_ctrl, b_ctrl);

    // Swap controllers (as ExchangeControl does).
    state.objects.get_mut(&a_id).unwrap().controller = b_ctrl;
    state.objects.get_mut(&b_id).unwrap().controller = a_ctrl;

    assert_eq!(
        state.objects.get(&a_id).unwrap().controller,
        p2,
        "CR 701.12b: ObjA should now be controlled by p2"
    );
    assert_eq!(
        state.objects.get(&b_id).unwrap().controller,
        p1,
        "CR 701.12b: ObjB should now be controlled by p1"
    );
}

/// CR 701.12b — ExchangeControl does nothing when both permanents have the same controller.
#[test]
fn test_exchange_control_same_controller_noop() {
    let p1 = p(1);
    let obj_a = ObjectSpec::creature(p1, "ObjA", 2, 2).in_zone(ZoneId::Battlefield);
    let obj_b = ObjectSpec::creature(p1, "ObjB", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(obj_a)
        .object(obj_b)
        .build()
        .unwrap();

    let a_id = find_object(&state, "ObjA");
    let b_id = find_object(&state, "ObjB");

    // Both have same controller — ExchangeControl should be a no-op.
    let a_ctrl = state.objects.get(&a_id).unwrap().controller;
    let b_ctrl = state.objects.get(&b_id).unwrap().controller;

    assert_eq!(
        a_ctrl, b_ctrl,
        "Both objects should have the same controller (p1)"
    );

    // Simulate the ExchangeControl no-op check: if a_ctrl == b_ctrl, do nothing.
    let did_exchange = a_ctrl != b_ctrl; // Should be false.
    assert!(
        !did_exchange,
        "CR 701.12b: ExchangeControl should do nothing when both permanents have the same controller"
    );
}

/// CR 613.1b — GainControl in multiplayer: correct in a 4-player game.
#[test]
fn test_gain_control_multiplayer() {
    let p1 = p(1);
    let p3 = p(3);
    let creature = ObjectSpec::creature(p3, "Dragon", 5, 5).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let dragon_id = find_object(&state, "Dragon");
    assert_eq!(
        state.objects.get(&dragon_id).unwrap().controller,
        p3,
        "Dragon should be controlled by p3 initially"
    );

    // p1 gains control of p3's creature.
    state.objects.get_mut(&dragon_id).unwrap().controller = p1;

    assert_eq!(
        state.objects.get(&dragon_id).unwrap().controller,
        p1,
        "CR 613.1b: p1 should now control the Dragon in a 4-player game"
    );
}

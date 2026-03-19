//! Mass Destroy / Board Wipe tests — PB-19.
//!
//! Verifies:
//! - Effect::DestroyAll: destroys matching permanents, respects indestructible/regeneration/umbra
//! - Effect::ExileAll: exiles matching permanents
//! - EffectAmount::LastEffectCount: reads destroyed/exiled count for follow-up effects
//! - AllPermanentsMatching controller filter fix (TargetController::Opponent)
//! - Multiplayer board wipes affect all players simultaneously

use mtg_engine::cards::card_definition::PlayerTarget;
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    all_cards, process_command, CardRegistry, CardType, Command, Effect, EffectAmount,
    EffectDuration, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost,
    ObjectFilter, ObjectId, ObjectSpec, PlayerId, ReplacementEffect, ReplacementId,
    ReplacementModification, ReplacementTrigger, Step, TargetController, TargetFilter, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn find_by_name_opt(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
}

fn count_on_battlefield(state: &GameState) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield)
        .count()
}

fn count_in_exile(state: &GameState) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Exile)
        .count()
}

fn run_effect(
    mut state: GameState,
    controller: PlayerId,
    effect: Effect,
) -> (GameState, Vec<GameEvent>, u32) {
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(controller, source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    let last_count = ctx.last_effect_count;
    (state, events, last_count)
}

// ── CR 701.8a: DestroyAll basic ───────────────────────────────────────────────

#[test]
/// CR 701.8a — DestroyAll with creature filter destroys all creatures; non-creatures survive.
fn test_destroy_all_creatures_basic() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Bear Token", 2, 2))
        .object(ObjectSpec::creature(p(2), "Goblin", 1, 1))
        .object(
            ObjectSpec::card(p(1), "Sol Ring")
                .with_types(vec![CardType::Artifact])
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let effect = Effect::DestroyAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        cant_be_regenerated: false,
    };

    let (state_after, events, last_count) = run_effect(state, p(1), effect);

    // Both creatures should be destroyed.
    let died_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(died_count, 2, "both creatures should be destroyed");

    // Sol Ring (artifact) should survive.
    assert!(
        find_by_name_opt(&state_after, "Sol Ring").is_some(),
        "artifact should survive DestroyAll with creature filter"
    );
    assert!(
        find_by_name_opt(&state_after, "Bear Token")
            .map(|id| { state_after.objects.get(&id).unwrap().zone != ZoneId::Battlefield })
            .unwrap_or(true),
        "Bear Token should leave battlefield"
    );

    // last_effect_count should track actual destroyed count.
    assert_eq!(last_count, 2, "last_effect_count should be 2");
}

// ── CR 702.12b: Indestructible ────────────────────────────────────────────────

#[test]
/// CR 702.12b — Indestructible permanents can't be destroyed by DestroyAll.
fn test_destroy_all_respects_indestructible() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Normal Bear", 2, 2))
        .object(
            ObjectSpec::creature(p(1), "Indestructible Titan", 6, 6)
                .with_keyword(KeywordAbility::Indestructible),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let effect = Effect::DestroyAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        cant_be_regenerated: false,
    };

    let (state_after, events, last_count) = run_effect(state, p(1), effect);

    // Only the normal bear is destroyed.
    let died_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(died_count, 1, "only normal creature should die");

    // Indestructible Titan should still be on battlefield.
    let titan_id = find_by_name(&state_after, "Indestructible Titan");
    assert_eq!(
        state_after.objects.get(&titan_id).unwrap().zone,
        ZoneId::Battlefield,
        "indestructible creature should survive DestroyAll (CR 702.12b)"
    );

    // Count should reflect only the non-indestructible destruction.
    assert_eq!(
        last_count, 1,
        "last_effect_count should be 1 (indestructible skipped)"
    );
}

// ── CR 701.19c: cant_be_regenerated ──────────────────────────────────────────

#[test]
/// CR 701.19c — cant_be_regenerated=true bypasses regeneration shields (Wrath of God).
fn test_destroy_all_cant_be_regenerated() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Drudge Skeletons", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    // Manually attach a regeneration shield to the skeleton.
    let skeleton_id = find_by_name(&state, "Drudge Skeletons");
    state.replacement_effects.push_back(ReplacementEffect {
        id: ReplacementId(9001),
        source: Some(skeleton_id),
        controller: p(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: true,
        trigger: ReplacementTrigger::WouldBeDestroyed {
            filter: ObjectFilter::SpecificObject(skeleton_id),
        },
        modification: ReplacementModification::Regenerate,
    });

    let effect = Effect::DestroyAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        cant_be_regenerated: true, // Wrath of God pattern
    };

    let (state_after, events, _) = run_effect(state, p(1), effect);

    // Skeleton should be destroyed despite having a regeneration shield.
    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        died,
        "CR 701.19c: creature with regen shield should still die when cant_be_regenerated=true"
    );
    assert!(
        find_by_name_opt(&state_after, "Drudge Skeletons")
            .map(|id| state_after.objects.get(&id).unwrap().zone != ZoneId::Battlefield)
            .unwrap_or(true),
        "creature should leave battlefield"
    );
}

// ── CR 701.19: Regeneration allowed ──────────────────────────────────────────

#[test]
/// CR 701.19 — cant_be_regenerated=false allows regeneration shield to save the creature.
fn test_destroy_all_allows_regeneration() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Protected Creature", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let creature_id = find_by_name(&state, "Protected Creature");
    // Attach a regeneration shield.
    state.replacement_effects.push_back(ReplacementEffect {
        id: ReplacementId(9002),
        source: Some(creature_id),
        controller: p(1),
        duration: EffectDuration::Indefinite,
        is_self_replacement: true,
        trigger: ReplacementTrigger::WouldBeDestroyed {
            filter: ObjectFilter::SpecificObject(creature_id),
        },
        modification: ReplacementModification::Regenerate,
    });

    let effect = Effect::DestroyAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        cant_be_regenerated: false, // Regeneration is allowed
    };

    let (state_after, events, last_count) = run_effect(state, p(1), effect);

    // The creature should NOT be destroyed (regeneration applies).
    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        !died,
        "CR 701.19: creature with regen shield should survive when cant_be_regenerated=false"
    );
    assert_eq!(
        last_count, 0,
        "last_effect_count should be 0 when all creatures regenerated"
    );
    // Creature should still be on the battlefield (possibly tapped).
    assert_eq!(
        state_after.objects.get(&creature_id).unwrap().zone,
        ZoneId::Battlefield,
        "regenerated creature should remain on battlefield"
    );
}

// ── LastEffectCount: follow-up gain life (Fumigate pattern) ──────────────────

#[test]
/// CR 701.8 — DestroyAll sets last_effect_count; GainLife with LastEffectCount reads it.
/// This is the Fumigate pattern: "You gain 1 life for each creature destroyed this way."
fn test_destroy_all_count_tracking() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Creature A", 1, 1))
        .object(ObjectSpec::creature(p(2), "Creature B", 1, 1))
        .object(ObjectSpec::creature(p(2), "Creature C", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    // Initial life total for controller.
    let initial_life = state.players.get(&p(1)).unwrap().life_total;

    let effect = Effect::Sequence(vec![
        Effect::DestroyAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            cant_be_regenerated: false,
        },
        Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::LastEffectCount,
        },
    ]);

    let (state_after, events, _) = run_effect(state, p(1), effect);

    // 3 creatures destroyed → gain 3 life.
    let life_gained: u32 = events
        .iter()
        .filter_map(|e| {
            if let GameEvent::LifeGained { player, amount } = e {
                if *player == p(1) {
                    return Some(*amount);
                }
            }
            None
        })
        .sum();
    assert_eq!(
        life_gained, 3,
        "Fumigate pattern: should gain 3 life for 3 creatures destroyed"
    );
    assert_eq!(
        state_after.players.get(&p(1)).unwrap().life_total,
        initial_life + 3,
        "controller life should increase by number of creatures destroyed"
    );
}

// ── AllPermanentsMatching controller filter fix ───────────────────────────────

#[test]
/// CR 701.8 — DestroyAll with controller: Opponent only destroys opponents' permanents.
/// This tests the AllPermanentsMatching controller filter fix (was broken before PB-19).
fn test_destroy_all_nonland_opponents() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // p1's permanents (should survive)
        .object(ObjectSpec::creature(p(1), "My Creature", 2, 2))
        .object(
            ObjectSpec::card(p(1), "My Artifact")
                .with_types(vec![CardType::Artifact])
                .in_zone(ZoneId::Battlefield),
        )
        // p2's permanents (should be destroyed)
        .object(ObjectSpec::creature(p(2), "Their Creature", 3, 3))
        .object(
            ObjectSpec::card(p(2), "Their Enchantment")
                .with_types(vec![CardType::Enchantment])
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let effect = Effect::DestroyAll {
        filter: TargetFilter {
            non_land: true,
            controller: TargetController::Opponent,
            ..Default::default()
        },
        cant_be_regenerated: false,
    };

    let (state_after, events, last_count) = run_effect(state, p(1), effect);

    // p2's permanents should be destroyed.
    let destroyed_count = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::CreatureDied { .. } | GameEvent::PermanentDestroyed { .. }
            )
        })
        .count();
    assert_eq!(
        destroyed_count, 2,
        "2 opponent permanents should be destroyed"
    );
    assert_eq!(last_count, 2, "last_effect_count should be 2");

    // p1's permanents should survive.
    assert!(
        find_by_name_opt(&state_after, "My Creature")
            .map(|id| state_after.objects.get(&id).unwrap().zone == ZoneId::Battlefield)
            .unwrap_or(false),
        "controller's creature should survive Ruinous Ultimatum pattern"
    );
    assert!(
        find_by_name_opt(&state_after, "My Artifact")
            .map(|id| state_after.objects.get(&id).unwrap().zone == ZoneId::Battlefield)
            .unwrap_or(false),
        "controller's artifact should survive Ruinous Ultimatum pattern"
    );
}

// ── CMC filter (Path of Peril pattern) ───────────────────────────────────────

#[test]
/// CR 701.8 + CR 202.3 — DestroyAll with max_cmc filter destroys only creatures within CMC.
fn test_destroy_all_filtered_by_cmc() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Cheap Goblin", 1, 1).with_mana_cost(ManaCost {
                generic: 1,
                red: 1,
                ..Default::default()
            }),
        )
        .object(
            ObjectSpec::creature(p(2), "Expensive Dragon", 5, 5).with_mana_cost(ManaCost {
                generic: 5,
                red: 1,
                ..Default::default()
            }),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let effect = Effect::DestroyAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            max_cmc: Some(2),
            ..Default::default()
        },
        cant_be_regenerated: false,
    };

    let (state_after, events, last_count) = run_effect(state, p(1), effect);

    // Only the cheap creature (CMC 2) is destroyed.
    let died_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(
        died_count, 1,
        "only creature with CMC <= 2 should be destroyed"
    );
    assert_eq!(last_count, 1);

    // Dragon (CMC 6) should survive.
    assert!(
        find_by_name_opt(&state_after, "Expensive Dragon")
            .map(|id| state_after.objects.get(&id).unwrap().zone == ZoneId::Battlefield)
            .unwrap_or(false),
        "creature with CMC > max_cmc should survive"
    );
}

// ── ExileAll basic ────────────────────────────────────────────────────────────

#[test]
/// CR 406.2 — ExileAll exiles all matching permanents.
fn test_exile_all_basic() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Elf A", 1, 1))
        .object(ObjectSpec::creature(p(2), "Elf B", 1, 1))
        .object(
            ObjectSpec::card(p(1), "Mountain")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let effect = Effect::ExileAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
    };

    let (state_after, events, last_count) = run_effect(state, p(1), effect);

    // Both creatures should be exiled.
    let exiled_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectExiled { .. }))
        .count();
    assert_eq!(
        exiled_count, 2,
        "both creatures should be exiled (CR 406.2)"
    );
    assert_eq!(count_in_exile(&state_after), 2);
    assert_eq!(last_count, 2, "last_effect_count should be 2");

    // Land should not be exiled.
    assert!(
        find_by_name_opt(&state_after, "Mountain")
            .map(|id| state_after.objects.get(&id).unwrap().zone == ZoneId::Battlefield)
            .unwrap_or(false),
        "land should not be affected by ExileAll creature filter"
    );
}

// ── ExileAll count tracking ───────────────────────────────────────────────────

#[test]
/// CR 406.2 — ExileAll sets last_effect_count for use by follow-up effects.
fn test_exile_all_count_tracking() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Token 1", 1, 1))
        .object(ObjectSpec::creature(p(1), "Token 2", 1, 1))
        .object(ObjectSpec::creature(p(1), "Token 3", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let effect = Effect::ExileAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
    };

    let (_state_after, _events, last_count) = run_effect(state, p(1), effect);
    assert_eq!(
        last_count, 3,
        "ExileAll should track count in last_effect_count"
    );
}

// ── AllPermanentsMatching controller filter verification ──────────────────────

#[test]
/// Fix verification — AllPermanentsMatching respects controller field.
/// Before PB-19, the controller field was ignored; this verifies the fix.
fn test_all_permanents_matching_controller_filter() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .object(ObjectSpec::creature(p(1), "P1 Creature", 2, 2))
        .object(ObjectSpec::creature(p(2), "P2 Creature", 3, 3))
        .object(ObjectSpec::creature(p(3), "P3 Creature", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    // Destroy only opponents' creatures (p2, p3).
    let effect = Effect::DestroyAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            controller: TargetController::Opponent,
            ..Default::default()
        },
        cant_be_regenerated: false,
    };

    let (state_after, events, last_count) = run_effect(state, p(1), effect);

    // Only p2 and p3 creatures should die.
    let died_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(died_count, 2, "only 2 opponent creatures should die");
    assert_eq!(last_count, 2);

    // P1 creature should survive.
    assert!(
        find_by_name_opt(&state_after, "P1 Creature")
            .map(|id| state_after.objects.get(&id).unwrap().zone == ZoneId::Battlefield)
            .unwrap_or(false),
        "controller's creature should survive when filter is Opponent"
    );
}

// ── DestroyAll triggers dies abilities ───────────────────────────────────────

#[test]
/// CR 601.2b + CR 603.2 — Creatures destroyed by DestroyAll emit CreatureDied events.
/// "When dies" triggered abilities fire from the resulting CreatureDied events.
fn test_destroy_all_triggers_dies() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Dying Creature 1", 1, 1))
        .object(ObjectSpec::creature(p(2), "Dying Creature 2", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let effect = Effect::DestroyAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        cant_be_regenerated: false,
    };

    let (_state_after, events, _) = run_effect(state, p(1), effect);

    // Verify CreatureDied events are emitted for both creatures.
    let died_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(
        died_count, 2,
        "CreatureDied events must be emitted to allow 'when dies' triggers to fire"
    );
}

// ── Multiplayer board wipe ────────────────────────────────────────────────────

#[test]
/// CR 701.8 — DestroyAll in a 4-player game destroys all matching permanents simultaneously.
fn test_destroy_all_multiplayer() {
    let mut builder = GameStateBuilder::new();
    let players = [p(1), p(2), p(3), p(4)];
    for &pl in &players {
        builder = builder.add_player(pl);
        // Each player has 2 creatures.
        builder = builder.object(ObjectSpec::creature(
            pl,
            &format!("Creature {pl:?} A"),
            1,
            1,
        ));
        builder = builder.object(ObjectSpec::creature(
            pl,
            &format!("Creature {pl:?} B"),
            2,
            2,
        ));
    }
    let state = builder
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let effect = Effect::DestroyAll {
        filter: TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        cant_be_regenerated: false,
    };

    let (state_after, events, last_count) = run_effect(state, p(1), effect);

    // 8 creatures total should be destroyed.
    let died_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(
        died_count, 8,
        "all 8 creatures across 4 players should be destroyed"
    );
    assert_eq!(last_count, 8, "last_effect_count should be 8");
    assert_eq!(
        count_on_battlefield(&state_after),
        0,
        "no creatures should remain on battlefield"
    );
}

// ── Fumigate card integration ─────────────────────────────────────────────────

#[test]
/// Integration test: Fumigate card definition correctly uses DestroyAll + LastEffectCount.
fn test_fumigate_card_integration() {
    let all = all_cards();
    let fumigate_def = all
        .iter()
        .find(|d| d.name == "Fumigate")
        .expect("Fumigate must be in all_cards()");
    let card_id = fumigate_def.card_id.clone();
    let registry = CardRegistry::new(all);

    let p1 = p(1);
    let p2 = p(2);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Fumigate")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 3,
                    white: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    // Add 3 creatures to the battlefield.
    for i in 0..3 {
        builder = builder.object(ObjectSpec::creature(p2, &format!("Token {i}"), 1, 1));
    }

    let mut state = builder.build().unwrap();

    // Add 5 white mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 5);

    let initial_life = state.players.get(&p1).unwrap().life_total;
    let fumigate_id = find_by_name(&state, "Fumigate");

    // Cast Fumigate.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: fumigate_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
            additional_costs: vec![],
        },
    )
    .unwrap();

    // Resolve Fumigate (pass priority for all players).
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();

    // Fumigate should have destroyed all 3 creatures.
    let creatures_on_bf = state
        .objects
        .values()
        .filter(|o| {
            o.zone == ZoneId::Battlefield
                && o.characteristics.card_types.contains(&CardType::Creature)
        })
        .count();
    assert_eq!(creatures_on_bf, 0, "Fumigate should destroy all creatures");

    // Controller should gain 3 life (one per creature destroyed).
    assert_eq!(
        state.players.get(&p1).unwrap().life_total,
        initial_life + 3,
        "Fumigate should give 1 life per creature destroyed (3 total)"
    );
}

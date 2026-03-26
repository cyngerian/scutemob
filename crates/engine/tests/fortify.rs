//! Fortify keyword ability tests (CR 702.67).
//!
//! Fortify is an activated ability of Fortification cards:
//! "[Cost]: Attach this Fortification to target land you control.
//! Activate only as a sorcery."
//!
//! Key rules:
//! - CR 702.67a: Fortify is a sorcery-speed activated ability targeting "a land you control."
//! - CR 301.6: Fortification rules parallel Equipment rules (via CR 301.5 analog for lands).
//! - CR 701.3b: Attaching to the same target does nothing.
//! - CR 701.3c: New timestamp on reattach (layer ordering).
//! - CR 704.5n: If the fortified permanent is no longer a land, Fortification becomes unattached.

use mtg_engine::state::{
    ActivatedAbility, ActivationCost, ContinuousEffect, EffectId, GameStateError,
};
use mtg_engine::{
    calculate_characteristics, process_command, CardEffectTarget, CardType, Command, Effect,
    EffectDuration, EffectFilter, EffectLayer, GameEvent, GameStateBuilder, KeywordAbility,
    LayerModification, ManaColor, ManaCost, ObjectId, PlayerId, Step, SubType, Target,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Build an ActivatedAbility representing Fortify {N} (sorcery-speed, no tap required).
fn fortify_ability(generic_mana: u32) -> ActivatedAbility {
    ActivatedAbility {
        targets: vec![],
        cost: ActivationCost {
            requires_tap: false,
            mana_cost: if generic_mana > 0 {
                Some(ManaCost {
                    generic: generic_mana,
                    ..Default::default()
                })
            } else {
                None
            },
            sacrifice_self: false,
            discard_card: false,
            discard_self: false,
            forage: false,
            sacrifice_filter: None,
            remove_counter_cost: None,
        },
        description: format!("Fortify {{{}}}", generic_mana),
        effect: Some(Effect::AttachFortification {
            fortification: CardEffectTarget::Source,
            target: CardEffectTarget::DeclaredTarget { index: 0 },
        }),
        sorcery_speed: true,
        activation_condition: None,
    }
}

// ── Test 1: Basic fortify attaches to land ────────────────────────────────────

#[test]
/// CR 702.67a — fortify attaches the Fortification to the targeted land you control.
fn test_fortify_basic_attaches_to_land() {
    let p1 = p(1);
    let p2 = p(2);

    let fortification = mtg_engine::ObjectSpec::artifact(p1, "Test Garrison")
        .with_subtypes(vec![SubType("Fortification".to_string())])
        .with_activated_ability(fortify_ability(3));

    let land = mtg_engine::ObjectSpec::land(p1, "Test Plains")
        .with_subtypes(vec![SubType("Plains".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(fortification)
        .object(land)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 mana to pay Fortify {3}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let fort_id = find_object(&state, "Test Garrison");
    let land_id = find_object(&state, "Test Plains");

    // Activate fortify targeting the land.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: fort_id,
            ability_index: 0,
            targets: vec![Target::Object(land_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // Pass priority to resolve (p1 then p2 pass).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Fortification is attached to the land.
    let fort_obj = state.objects.get(&fort_id).expect("fortification exists");
    assert_eq!(
        fort_obj.attached_to,
        Some(land_id),
        "fortification.attached_to should be the land"
    );

    let land_obj = state.objects.get(&land_id).expect("land exists");
    assert!(
        land_obj.attachments.contains(&fort_id),
        "land.attachments should contain the fortification"
    );

    // FortificationAttached event emitted.
    assert!(
        resolve_events.iter().any(|e| matches!(
            e,
            GameEvent::FortificationAttached { fortification_id, target_id, controller }
            if *fortification_id == fort_id && *target_id == land_id && *controller == p1
        )),
        "FortificationAttached event expected"
    );
}

// ── Test 2: Sorcery-speed restriction (not main phase) ────────────────────────

#[test]
/// CR 702.67a, CR 602.5d — fortify can only be activated during main phase.
fn test_fortify_sorcery_speed_only() {
    let p1 = p(1);
    let p2 = p(2);

    let fortification = mtg_engine::ObjectSpec::artifact(p1, "Test Garrison")
        .with_subtypes(vec![SubType("Fortification".to_string())])
        .with_activated_ability(fortify_ability(3));

    let land = mtg_engine::ObjectSpec::land(p1, "Test Plains")
        .with_subtypes(vec![SubType("Plains".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(fortification)
        .object(land)
        .active_player(p1)
        .at_step(Step::DeclareAttackers) // NOT main phase
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let fort_id = find_object(&state, "Test Garrison");
    let land_id = find_object(&state, "Test Plains");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: fort_id,
            ability_index: 0,
            targets: vec![Target::Object(land_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        matches!(result, Err(GameStateError::NotMainPhase)),
        "fortify during non-main phase should return NotMainPhase, got {:?}",
        result
    );
}

// ── Test 3: Target must be a land ─────────────────────────────────────────────

#[test]
/// CR 702.67a — fortify can only target "a land you control."
/// Targeting a creature (not a land) is rejected at activation time.
fn test_fortify_target_must_be_land() {
    let p1 = p(1);
    let p2 = p(2);

    let fortification = mtg_engine::ObjectSpec::artifact(p1, "Test Garrison")
        .with_subtypes(vec![SubType("Fortification".to_string())])
        .with_activated_ability(fortify_ability(3));

    // A creature, not a land.
    let creature = mtg_engine::ObjectSpec::creature(p1, "Test Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(fortification)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let fort_id = find_object(&state, "Test Garrison");
    let creature_id = find_object(&state, "Test Bear");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: fort_id,
            ability_index: 0,
            targets: vec![Target::Object(creature_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "fortify targeting a creature should return InvalidTarget, got {:?}",
        result
    );
}

// ── Test 4: Target must be controlled by activating player ────────────────────

#[test]
/// CR 702.67a — fortify can only target "a land you control."
/// Targeting an opponent's land is rejected at activation time.
fn test_fortify_requires_controller_ownership() {
    let p1 = p(1);
    let p2 = p(2);

    let fortification = mtg_engine::ObjectSpec::artifact(p1, "Test Garrison")
        .with_subtypes(vec![SubType("Fortification".to_string())])
        .with_activated_ability(fortify_ability(3));

    // Land controlled by p2.
    let opponent_land = mtg_engine::ObjectSpec::land(p2, "Opponent Plains")
        .with_subtypes(vec![SubType("Plains".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(fortification)
        .object(opponent_land)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let fort_id = find_object(&state, "Test Garrison");
    let land_id = find_object(&state, "Opponent Plains");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: fort_id,
            ability_index: 0,
            targets: vec![Target::Object(land_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "fortify targeting opponent's land should return InvalidTarget, got {:?}",
        result
    );
}

// ── Test 5: Reattach detaches from previous land ─────────────────────────────

#[test]
/// CR 301.6 (via 301.5c analog) — Fortification can't be attached to more than one land;
/// moving it detaches from the old land and attaches to the new one.
fn test_fortify_moves_between_lands() {
    let p1 = p(1);
    let p2 = p(2);

    let fortification = mtg_engine::ObjectSpec::artifact(p1, "Test Garrison")
        .with_subtypes(vec![SubType("Fortification".to_string())])
        .with_activated_ability(fortify_ability(3));

    let land_a = mtg_engine::ObjectSpec::land(p1, "Land A")
        .with_subtypes(vec![SubType("Plains".to_string())]);

    let land_b = mtg_engine::ObjectSpec::land(p1, "Land B")
        .with_subtypes(vec![SubType("Forest".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(fortification)
        .object(land_a)
        .object(land_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let fort_id = find_object(&state, "Test Garrison");
    let land_a_id = find_object(&state, "Land A");
    let land_b_id = find_object(&state, "Land B");

    // Manually pre-attach to Land A.
    state.objects.get_mut(&fort_id).unwrap().attached_to = Some(land_a_id);
    state
        .objects
        .get_mut(&land_a_id)
        .unwrap()
        .attachments
        .push_back(fort_id);

    // Now fortify targeting Land B.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: fort_id,
            ability_index: 0,
            targets: vec![Target::Object(land_b_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    // Fortification is now attached to Land B.
    assert_eq!(
        state.objects.get(&fort_id).unwrap().attached_to,
        Some(land_b_id),
        "fortification should be attached to Land B"
    );

    // Land A no longer has the fortification in its attachments.
    assert!(
        !state
            .objects
            .get(&land_a_id)
            .unwrap()
            .attachments
            .contains(&fort_id),
        "Land A should not have fortification in attachments"
    );

    // Land B has the fortification.
    assert!(
        state
            .objects
            .get(&land_b_id)
            .unwrap()
            .attachments
            .contains(&fort_id),
        "Land B should have fortification in attachments"
    );
}

// ── Test 6: SBA unattaches Fortification if land loses land type ───────────────

#[test]
/// CR 704.5n — if the fortified permanent stops being a land, the Fortification
/// becomes unattached. (Uses the existing SBA implementation.)
fn test_fortify_sba_unattaches_from_nonland() {
    let p1 = p(1);
    let p2 = p(2);

    let fortification = mtg_engine::ObjectSpec::artifact(p1, "Test Garrison")
        .with_subtypes(vec![SubType("Fortification".to_string())])
        .with_activated_ability(fortify_ability(3));

    // A land.
    let land = mtg_engine::ObjectSpec::land(p1, "Animated Plains")
        .with_subtypes(vec![SubType("Plains".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(fortification)
        .object(land)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let fort_id = find_object(&state, "Test Garrison");
    let land_id = find_object(&state, "Animated Plains");

    // Manually pre-attach to land.
    state.objects.get_mut(&fort_id).unwrap().attached_to = Some(land_id);
    state
        .objects
        .get_mut(&land_id)
        .unwrap()
        .attachments
        .push_back(fort_id);

    // Strip Land type from the object (simulating a continuous effect that removed it).
    state
        .objects
        .get_mut(&land_id)
        .unwrap()
        .characteristics
        .card_types
        .remove(&CardType::Land);

    // Trigger SBA check by passing priority (the engine runs SBAs after each priority pass).
    state.turn.priority_holder = Some(p1);
    let (state, _) = pass_all(state, &[p1, p2]);

    // SBA should have unattached the Fortification since the target is no longer a land.
    let fort_obj = state
        .objects
        .get(&fort_id)
        .expect("fortification still exists");
    assert_eq!(
        fort_obj.attached_to, None,
        "fortification should be unattached after land loses Land type (CR 704.5n)"
    );
}

// ── Test 7: Static ability grants to fortified land via AttachedLand filter ───

#[test]
/// CR 301.6 + CR 604.2 + CR 613 — a Fortification with a static ability
/// (e.g., "fortified land has indestructible") grants that ability to the
/// attached land via EffectFilter::AttachedLand and calculate_characteristics.
fn test_fortify_static_ability_grants_to_land() {
    let p1 = p(1);
    let p2 = p(2);

    let fortification = mtg_engine::ObjectSpec::artifact(p1, "Test Garrison")
        .with_subtypes(vec![SubType("Fortification".to_string())])
        .with_activated_ability(fortify_ability(3));

    let land = mtg_engine::ObjectSpec::land(p1, "Test Plains")
        .with_subtypes(vec![SubType("Plains".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(fortification)
        .object(land)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let fort_id = find_object(&state, "Test Garrison");
    let land_id = find_object(&state, "Test Plains");

    // Manually register a continuous effect on the fortification, simulating
    // what register_static_continuous_effects does at ETB time.
    // This grants Indestructible to whatever the fortification is attached to.
    {
        let eff_id = state.next_object_id().0;
        state.timestamp_counter += 1;
        let ts = state.timestamp_counter;
        state.continuous_effects.push_back(ContinuousEffect {
            id: EffectId(eff_id),
            source: Some(fort_id),
            timestamp: ts,
            layer: EffectLayer::Ability,
            duration: EffectDuration::WhileSourceOnBattlefield,
            filter: EffectFilter::AttachedLand,
            modification: LayerModification::AddKeywords(
                [KeywordAbility::Indestructible].into_iter().collect(),
            ),
            is_cda: false,
            condition: None,
        });
    }

    // Before fortifying: land should NOT have Indestructible (fortification is unattached).
    let chars_before =
        calculate_characteristics(&state, land_id).expect("land exists before fortifying");
    assert!(
        !chars_before
            .keywords
            .contains(&KeywordAbility::Indestructible),
        "land should NOT have Indestructible before fortification is attached"
    );

    // Fortify targeting the land.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: fort_id,
            ability_index: 0,
            targets: vec![Target::Object(land_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    let (state, _) = pass_all(state, &[p1, p2]);

    // Fortification is attached.
    assert_eq!(
        state.objects.get(&fort_id).unwrap().attached_to,
        Some(land_id),
        "fortification should be attached to the land"
    );

    // After fortifying: land should have Indestructible via layer-computed characteristics.
    // CR 613 (Layer 6 / Ability layer): static ability from attached fortification applies.
    let chars_after =
        calculate_characteristics(&state, land_id).expect("land exists after fortifying");
    assert!(
        chars_after.keywords.contains(&KeywordAbility::Indestructible),
        "land should have Indestructible from fortification's static ability after fortifying (CR 604 + CR 613)"
    );
}

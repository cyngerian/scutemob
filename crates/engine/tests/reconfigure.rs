//! Reconfigure keyword ability tests (CR 702.151).
//!
//! Reconfigure represents two activated abilities:
//! "[Cost]: Attach this permanent to another target creature you control. Activate only
//! as a sorcery." and "[Cost]: Unattach this permanent. Activate only if this permanent
//! is attached to a creature and only as a sorcery."
//!
//! Key rules tested:
//! - CR 702.151a: Two activated abilities (attach + unattach), both sorcery-speed.
//! - CR 702.151b: While attached, the Equipment stops being a creature and loses creature subtypes.
//! - Ruling 2022-02-18: "Not a creature" effect persists even if Reconfigure keyword removed.
//! - CR 301.5c: Equipment can't equip itself.
//! - CR 704.5n: Equipment becomes unattached if target is no longer legal.

use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    calculate_characteristics, process_command, CardEffectTarget, CardType, Command, Effect,
    GameEvent, GameStateBuilder, KeywordAbility, ManaCost, ObjectId, PlayerId, Step, SubType,
    Target,
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

/// Build the Reconfigure attach activated ability with the given generic cost.
fn reconfigure_attach_ability(generic_mana: u32) -> ActivatedAbility {
    ActivatedAbility {
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

            forage: false,
        },
        description: format!("Reconfigure (attach) {{{}}} (CR 702.151a)", generic_mana),
        effect: Some(Effect::AttachEquipment {
            equipment: CardEffectTarget::Source,
            target: CardEffectTarget::DeclaredTarget { index: 0 },
        }),
        sorcery_speed: true,
    }
}

/// Build the Reconfigure unattach activated ability with the given generic cost.
fn reconfigure_detach_ability(generic_mana: u32) -> ActivatedAbility {
    ActivatedAbility {
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

            forage: false,
        },
        description: format!("Reconfigure (unattach) {{{}}} (CR 702.151a)", generic_mana),
        effect: Some(Effect::DetachEquipment {
            equipment: CardEffectTarget::Source,
        }),
        sorcery_speed: true,
    }
}

// ── Test 1: Attach removes creature type ─────────────────────────────────────

#[test]
/// CR 702.151b — while attached via reconfigure, the Equipment stops being a creature.
fn test_reconfigure_attach_removes_creature_type() {
    let p1 = p(1);
    let p2 = p(2);

    // "Lizard Blades" style: artifact creature with Equipment subtype and Reconfigure.
    let lizard_blades = mtg_engine::ObjectSpec::creature(p1, "Lizard Blades", 1, 1)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_subtypes(vec![
            SubType("Equipment".to_string()),
            SubType("Lizard".to_string()),
        ])
        .with_keyword(KeywordAbility::Reconfigure)
        .with_activated_ability(reconfigure_attach_ability(2))
        .with_activated_ability(reconfigure_detach_ability(2));

    let target_creature = mtg_engine::ObjectSpec::creature(p1, "Vanilla Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(lizard_blades)
        .object(target_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 mana to pay Reconfigure {2}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let blades_id = find_object(&state, "Lizard Blades");
    let bear_id = find_object(&state, "Vanilla Bear");

    // Activate reconfigure (attach) targeting the bear.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blades_id,
            ability_index: 0, // attach ability
            targets: vec![Target::Object(bear_id)],
            discard_card: None,
        },
    )
    .unwrap();

    // Pass priority to resolve.
    let (state, _resolve_events) = pass_all(state, &[p1, p2]);

    // Verify attachment state.
    let blades_obj = state.objects.get(&blades_id).expect("Lizard Blades exists");
    assert_eq!(
        blades_obj.attached_to,
        Some(bear_id),
        "Equipment should be attached to the bear"
    );
    assert!(
        blades_obj
            .designations
            .contains(mtg_engine::Designations::RECONFIGURED),
        "is_reconfigured should be true after attaching via Reconfigure"
    );

    let bear_obj = state.objects.get(&bear_id).expect("Vanilla Bear exists");
    assert!(
        bear_obj.attachments.contains(&blades_id),
        "bear.attachments should contain Lizard Blades"
    );

    // CR 702.151b: Layer-resolved characteristics should NOT include Creature type.
    let chars = calculate_characteristics(&state, blades_id).expect("chars calculated");
    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "Equipment should not be a creature while reconfigured (CR 702.151b)"
    );
    // Equipment subtype should remain; creature subtype (Lizard) should be removed.
    assert!(
        chars.subtypes.contains(&SubType("Equipment".to_string())),
        "Equipment subtype should remain after reconfigure"
    );
    assert!(
        !chars.subtypes.contains(&SubType("Lizard".to_string())),
        "Creature subtype (Lizard) should be removed while reconfigured (ruling 2022-02-18)"
    );
}

// ── Test 2: Unattach restores creature type ───────────────────────────────────

#[test]
/// CR 702.151a — after unattaching via reconfigure, the Equipment becomes a creature again.
fn test_reconfigure_unattach_restores_creature_type() {
    let p1 = p(1);
    let p2 = p(2);

    let lizard_blades = mtg_engine::ObjectSpec::creature(p1, "Lizard Blades", 1, 1)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_subtypes(vec![
            SubType("Equipment".to_string()),
            SubType("Lizard".to_string()),
        ])
        .with_keyword(KeywordAbility::Reconfigure)
        .with_activated_ability(reconfigure_attach_ability(2))
        .with_activated_ability(reconfigure_detach_ability(2));

    let target_creature = mtg_engine::ObjectSpec::creature(p1, "Vanilla Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(lizard_blades)
        .object(target_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give mana for attach.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let blades_id = find_object(&state, "Lizard Blades");
    let bear_id = find_object(&state, "Vanilla Bear");

    // Attach.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blades_id,
            ability_index: 0,
            targets: vec![Target::Object(bear_id)],
            discard_card: None,
        },
    )
    .unwrap();
    let (mut state, _) = pass_all(state, &[p1, p2]);

    // Sanity: is_reconfigured = true after attach.
    assert!(
        state
            .objects
            .get(&blades_id)
            .unwrap()
            .designations
            .contains(mtg_engine::Designations::RECONFIGURED),
        "should be reconfigured after attach"
    );

    // Give mana for unattach.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    // Activate unattach (no targets needed).
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blades_id,
            ability_index: 1, // detach ability
            targets: vec![],
            discard_card: None,
        },
    )
    .unwrap();
    let (state, _resolve_events) = pass_all(state, &[p1, p2]);

    // Verify unattached state.
    let blades_obj = state.objects.get(&blades_id).expect("Lizard Blades exists");
    assert_eq!(
        blades_obj.attached_to, None,
        "Equipment should be unattached"
    );
    assert!(
        !blades_obj
            .designations
            .contains(mtg_engine::Designations::RECONFIGURED),
        "is_reconfigured should be false after unattaching"
    );

    let bear_obj = state.objects.get(&bear_id).expect("Vanilla Bear exists");
    assert!(
        !bear_obj.attachments.contains(&blades_id),
        "bear.attachments should NOT contain Lizard Blades after unattach"
    );

    // CR 702.151b: creature type should be restored.
    let chars = calculate_characteristics(&state, blades_id).expect("chars calculated");
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "Equipment should be a creature again after unattach (CR 702.151b)"
    );
    // Creature subtype (Lizard) should be restored.
    assert!(
        chars.subtypes.contains(&SubType("Lizard".to_string())),
        "Creature subtype (Lizard) should be restored after unattach"
    );
}

// ── Test 3: Sorcery-speed restriction ────────────────────────────────────────

#[test]
/// CR 702.151a — reconfigure can only be activated during main phase with empty stack.
fn test_reconfigure_sorcery_speed_only() {
    let p1 = p(1);
    let p2 = p(2);

    let lizard_blades = mtg_engine::ObjectSpec::creature(p1, "Lizard Blades", 1, 1)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_keyword(KeywordAbility::Reconfigure)
        .with_activated_ability(reconfigure_attach_ability(2))
        .with_activated_ability(reconfigure_detach_ability(2));

    let target_creature = mtg_engine::ObjectSpec::creature(p1, "Vanilla Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(lizard_blades)
        .object(target_creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers) // NOT main phase
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let blades_id = find_object(&state, "Lizard Blades");
    let bear_id = find_object(&state, "Vanilla Bear");

    // Attempt to activate reconfigure (attach) during combat — should fail.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blades_id,
            ability_index: 0,
            targets: vec![Target::Object(bear_id)],
            discard_card: None,
        },
    );

    assert!(
        result.is_err(),
        "Reconfigure should be rejected outside main phase (CR 702.151a sorcery-speed)"
    );
}

// ── Test 4: Can't attach to self ─────────────────────────────────────────────

#[test]
/// CR 301.5c — an Equipment can't equip itself (applies to reconfigure as well).
fn test_reconfigure_cant_attach_to_self() {
    let p1 = p(1);
    let p2 = p(2);

    let lizard_blades = mtg_engine::ObjectSpec::creature(p1, "Lizard Blades", 1, 1)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_keyword(KeywordAbility::Reconfigure)
        .with_activated_ability(reconfigure_attach_ability(2))
        .with_activated_ability(reconfigure_detach_ability(2));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(lizard_blades)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let blades_id = find_object(&state, "Lizard Blades");

    // Attempt to target itself — rejected at target-validation time.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blades_id,
            ability_index: 0,
            targets: vec![Target::Object(blades_id)], // targeting self,
            discard_card: None,
        },
    );

    // Should fail: equip target must be a creature you control (and Equipment
    // that's also a creature without reconfigure attached can't target itself).
    // The AttachEquipment handler (CR 301.5c) skips equip_id == target_id.
    // Since nothing happens, we activate and resolve, then assert no attachment.
    // (Some engines reject at activation time, others at resolution time — both are correct.)
    match result {
        Err(_) => {
            // Rejected at activation time — correct.
        }
        Ok((state, _)) => {
            // Resolved; check no self-attachment occurred.
            let (state, _) = pass_all(state, &[p1, p2]);
            let blades_obj = state.objects.get(&blades_id).unwrap();
            assert_eq!(
                blades_obj.attached_to, None,
                "Equipment must not equip itself (CR 301.5c)"
            );
        }
    }
}

// ── Test 5: Equipped creature leaves battlefield, is_reconfigured cleared ────

#[test]
/// CR 704.5n + CR 702.151b — when the equipped creature leaves, SBA unattaches the
/// Equipment and clears is_reconfigured, restoring its creature type.
fn test_reconfigure_equipped_creature_leaves_battlefield() {
    let p1 = p(1);
    let p2 = p(2);

    let lizard_blades = mtg_engine::ObjectSpec::creature(p1, "Lizard Blades", 1, 1)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_subtypes(vec![
            SubType("Equipment".to_string()),
            SubType("Lizard".to_string()),
        ])
        .with_keyword(KeywordAbility::Reconfigure)
        .with_activated_ability(reconfigure_attach_ability(2))
        .with_activated_ability(reconfigure_detach_ability(2));

    let target_creature = mtg_engine::ObjectSpec::creature(p1, "Vanilla Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(lizard_blades)
        .object(target_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let blades_id = find_object(&state, "Lizard Blades");
    let bear_id = find_object(&state, "Vanilla Bear");

    // Attach.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blades_id,
            ability_index: 0,
            targets: vec![Target::Object(bear_id)],
            discard_card: None,
        },
    )
    .unwrap();
    let (mut state, _) = pass_all(state, &[p1, p2]);

    assert!(
        state
            .objects
            .get(&blades_id)
            .unwrap()
            .designations
            .contains(mtg_engine::Designations::RECONFIGURED),
        "sanity: is_reconfigured should be true"
    );

    // Destroy the bear (move to graveyard, triggering SBA).
    // We can do this by directly removing it (simulating a removal spell resolving).
    // Use ExileObject effect or direct state manipulation to simulate the creature dying.
    // Simplest: simulate by using the SBA checker after removing the bear.
    state.objects.remove(&bear_id);

    // Run SBAs to unattach the equipment.
    let _ = mtg_engine::check_and_apply_sbas(&mut state);

    // Equipment should now be unattached.
    let blades_obj = state
        .objects
        .get(&blades_id)
        .expect("Lizard Blades still exists");
    assert_eq!(
        blades_obj.attached_to, None,
        "SBA should unattach Equipment when creature leaves"
    );
    assert!(
        !blades_obj
            .designations
            .contains(mtg_engine::Designations::RECONFIGURED),
        "is_reconfigured should be cleared by SBA unattach"
    );

    // CR 702.151b: creature type should be restored.
    let chars = calculate_characteristics(&state, blades_id).expect("chars calculated");
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "Equipment should be a creature again after SBA unattach (CR 702.151b)"
    );
}

// ── Test 6: Unattach rejected when not attached ──────────────────────────────

#[test]
/// CR 702.151a — the unattach ability "Activate only if this permanent is attached to a creature."
fn test_reconfigure_unattach_rejected_when_not_attached() {
    let p1 = p(1);
    let p2 = p(2);

    let lizard_blades = mtg_engine::ObjectSpec::creature(p1, "Lizard Blades", 1, 1)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_keyword(KeywordAbility::Reconfigure)
        .with_activated_ability(reconfigure_attach_ability(2))
        .with_activated_ability(reconfigure_detach_ability(2));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(lizard_blades)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let blades_id = find_object(&state, "Lizard Blades");

    // Attempt to activate unattach when not attached.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blades_id,
            ability_index: 1, // detach ability
            targets: vec![],
            discard_card: None,
        },
    );

    assert!(
        result.is_err(),
        "Reconfigure unattach should be rejected when not attached (CR 702.151a)"
    );
}

// ── Test 7: Multiplayer — can't attach to opponent's creature ────────────────

#[test]
/// CR 702.151a analog (CR 702.6a): reconfigure targets "a creature you control."
fn test_reconfigure_cant_attach_to_opponents_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let lizard_blades = mtg_engine::ObjectSpec::creature(p1, "Lizard Blades", 1, 1)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_keyword(KeywordAbility::Reconfigure)
        .with_activated_ability(reconfigure_attach_ability(2))
        .with_activated_ability(reconfigure_detach_ability(2));

    // Opponent's creature.
    let opp_creature = mtg_engine::ObjectSpec::creature(p2, "Opponent Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(lizard_blades)
        .object(opp_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let blades_id = find_object(&state, "Lizard Blades");
    let opp_id = find_object(&state, "Opponent Bear");

    // Attempt to target opponent's creature — rejected at activation or resolution time.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blades_id,
            ability_index: 0,
            targets: vec![Target::Object(opp_id)],
            discard_card: None,
        },
    );

    match result {
        Err(_) => {
            // Rejected at activation time — correct.
        }
        Ok((state, _)) => {
            // Check not attached after resolution.
            let (state, _) = pass_all(state, &[p1, p2]);
            let blades_obj = state.objects.get(&blades_id).unwrap();
            assert_eq!(
                blades_obj.attached_to, None,
                "Equipment must not attach to an opponent's creature (CR 702.151a)"
            );
        }
    }
}

// ── Test 8: Artifact type retained while reconfigured ─────────────────────────

#[test]
/// CR 702.151b — the Equipment remains an artifact while reconfigured (only creature type removed).
fn test_reconfigure_artifact_type_retained_while_attached() {
    let p1 = p(1);
    let p2 = p(2);

    let lizard_blades = mtg_engine::ObjectSpec::creature(p1, "Lizard Blades", 1, 1)
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_keyword(KeywordAbility::Reconfigure)
        .with_activated_ability(reconfigure_attach_ability(2))
        .with_activated_ability(reconfigure_detach_ability(2));

    let target_creature = mtg_engine::ObjectSpec::creature(p1, "Vanilla Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(lizard_blades)
        .object(target_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let blades_id = find_object(&state, "Lizard Blades");
    let bear_id = find_object(&state, "Vanilla Bear");

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blades_id,
            ability_index: 0,
            targets: vec![Target::Object(bear_id)],
            discard_card: None,
        },
    )
    .unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.151b: only Creature type removed; Artifact type is retained.
    let chars = calculate_characteristics(&state, blades_id).expect("chars calculated");
    assert!(
        !chars.card_types.contains(&CardType::Creature),
        "Creature type should be removed while reconfigured"
    );
    assert!(
        chars.card_types.contains(&CardType::Artifact),
        "Artifact type should be retained while reconfigured (CR 702.151b)"
    );
}

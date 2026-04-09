//! Treasure token tests (CR 111.10a, CR 605.1a, CR 605.3b).
//!
//! Treasure tokens are a predefined token type introduced in CR 111.10a:
//! "A Treasure token is a colorless Treasure artifact token with
//! '{{T}}, Sacrifice this token: Add one mana of any color.'"
//!
//! Key rules verified:
//! - Treasure tokens are colorless artifact tokens with the Treasure subtype (CR 111.10a).
//! - Treasure's mana ability is a mana ability per CR 605.1a (no target, produces mana).
//! - Mana abilities resolve immediately without the stack (CR 605.3b).
//! - Sacrifice is a cost paid before mana is produced (CR 602.2c).
//! - Summoning sickness does NOT affect artifacts (CR 302.6 only restricts creatures).
//! - Tokens cease to exist in non-battlefield zones as an SBA (CR 704.5d).
//! - Only the controller can activate the ability (CR 602.2).

use mtg_engine::{
    check_and_apply_sbas, process_command, treasure_token_spec, CardType, Command, GameEvent,
    GameState, GameStateBuilder, ManaAbility, ManaColor, ObjectId, ObjectSpec, PlayerId, Step,
    SubType, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Find an object in the game state by name (panics if not found).
fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Find an object by name in a specific zone. Returns None if not found.
fn find_by_name_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Count objects with a given name on the battlefield.
fn count_on_battlefield(state: &GameState, name: &str) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .count()
}

/// Build a Treasure token `ObjectSpec` for direct placement on the battlefield.
fn treasure_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::artifact(owner, name)
        .with_subtypes(vec![SubType("Treasure".to_string())])
        .with_mana_ability(ManaAbility::treasure())
        .token()
}

// ── Test 1: treasure_token_spec characteristics ───────────────────────────────

#[test]
/// CR 111.10a — `treasure_token_spec(1)` produces a spec for a colorless Treasure
/// artifact token with the correct mana ability (sacrifice_self=true, any_color=true).
fn test_treasure_token_spec_characteristics() {
    let spec = treasure_token_spec(1);
    assert_eq!(spec.name, "Treasure");
    assert_eq!(spec.power, 0);
    assert_eq!(spec.toughness, 0);
    assert!(spec.colors.is_empty(), "Treasure is colorless");
    assert!(
        spec.card_types.contains(&CardType::Artifact),
        "Treasure is an artifact"
    );
    assert!(
        spec.subtypes.contains(&SubType("Treasure".to_string())),
        "Treasure has subtype Treasure"
    );
    assert_eq!(
        spec.mana_abilities.len(),
        1,
        "Treasure has exactly one mana ability"
    );
    let ma = &spec.mana_abilities[0];
    assert!(ma.requires_tap, "Treasure ability requires {{T}}");
    assert!(
        ma.sacrifice_self,
        "Treasure ability requires sacrificing itself"
    );
    assert!(ma.any_color, "Treasure ability produces any color");
}

// ── Test 2: Treasure token on battlefield via ObjectSpec ─────────────────────

#[test]
/// CR 111.10a — A Treasure token placed on the battlefield is an artifact token
/// with the correct subtype and has the sacrifice-for-mana mana ability.
fn test_treasure_token_has_mana_ability() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(treasure_spec(p1, "Treasure"))
        .build()
        .unwrap();

    let obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Treasure")
        .expect("Treasure token should be on battlefield");

    assert!(obj.is_token, "Treasure should be a token");
    assert!(
        obj.characteristics.card_types.contains(&CardType::Artifact),
        "Treasure is an Artifact"
    );
    assert!(
        obj.characteristics
            .subtypes
            .contains(&SubType("Treasure".to_string())),
        "Treasure has Treasure subtype"
    );
    assert_eq!(
        obj.characteristics.mana_abilities.len(),
        1,
        "Treasure has exactly 1 mana ability"
    );
    let ma = &obj.characteristics.mana_abilities[0];
    assert!(ma.sacrifice_self, "mana ability has sacrifice_self=true");
    assert!(ma.any_color, "mana ability has any_color=true");
}

// ── Test 3: Sacrifice Treasure for mana ─────────────────────────────────────

#[test]
/// CR 605.1a / CR 111.10a — Activating Treasure's mana ability (ability_index: 0)
/// taps then sacrifices the token and adds 1 colorless mana to the player's pool.
fn test_treasure_sacrifice_for_mana() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(treasure_spec(p1, "Treasure"))
        .build()
        .unwrap();

    let treasure_id = find_by_name(&state, "Treasure");

    let (new_state, events) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: treasure_id,
            ability_index: 0,
        },
    )
    .unwrap();

    // Mana was added.
    assert_eq!(
        new_state.player(p1).unwrap().mana_pool.colorless,
        1,
        "Player should have 1 colorless mana"
    );

    // PermanentTapped event was emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTapped {
                player,
                object_id,
                ..
            } if *player == p1 && *object_id == treasure_id
        )),
        "PermanentTapped event should be emitted"
    );

    // PermanentDestroyed event was emitted (Treasure is not a creature).
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentDestroyed { object_id, .. } if *object_id == treasure_id
        )),
        "PermanentDestroyed event should be emitted when Treasure is sacrificed"
    );

    // ManaAdded event was emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                player,
                color: ManaColor::Colorless,
                amount: 1,
                ..
            } if *player == p1
        )),
        "ManaAdded event should be emitted"
    );

    // Treasure is now in the graveyard (post-sacrifice, pre-SBA).
    assert!(
        find_by_name_in_zone(&new_state, "Treasure", ZoneId::Graveyard(p1)).is_some(),
        "Treasure should be in graveyard after sacrifice"
    );

    // Treasure is no longer on the battlefield.
    assert_eq!(
        count_on_battlefield(&new_state, "Treasure"),
        0,
        "Treasure should no longer be on the battlefield"
    );
}

// ── Test 4: Mana is available immediately (no stack) ────────────────────────

#[test]
/// CR 605.3b — Mana abilities resolve immediately. After sacrificing a Treasure,
/// the mana is in the pool and the stack is empty.
fn test_treasure_mana_resolves_immediately_no_stack() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(treasure_spec(p1, "Treasure"))
        .build()
        .unwrap();

    let treasure_id = find_by_name(&state, "Treasure");

    let (new_state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: treasure_id,
            ability_index: 0,
        },
    )
    .unwrap();

    // Stack must be empty — mana abilities do not use the stack (CR 605.3b).
    assert!(
        new_state.stack_objects.is_empty(),
        "CR 605.3b: Mana abilities don't use the stack; stack should remain empty"
    );

    // Priority is still held by p1 (CR 605.5: mana abilities don't reset priority).
    assert_eq!(
        new_state.turn.priority_holder,
        Some(p1),
        "CR 605.5: Player retains priority after activating a mana ability"
    );

    // Mana is in the pool.
    assert_eq!(
        new_state.player(p1).unwrap().mana_pool.colorless,
        1,
        "Mana should be in pool immediately after mana ability activation"
    );
}

// ── Test 5: Sacrifice multiple Treasures in sequence ────────────────────────

#[test]
/// CR 605.3a — Multiple mana abilities can be activated in sequence.
/// Three Treasures can each be sacrificed, yielding 3 colorless mana.
fn test_treasure_sacrifice_multiple_in_sequence() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(treasure_spec(p1, "Treasure-A"))
        .object(treasure_spec(p1, "Treasure-B"))
        .object(treasure_spec(p1, "Treasure-C"))
        .build()
        .unwrap();

    let id_a = find_by_name(&state, "Treasure-A");
    let id_b = find_by_name(&state, "Treasure-B");
    let id_c = find_by_name(&state, "Treasure-C");

    let (s1, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: id_a,
            ability_index: 0,
        },
    )
    .unwrap();

    let (s2, _) = process_command(
        s1,
        Command::TapForMana {
            player: p1,
            source: id_b,
            ability_index: 0,
        },
    )
    .unwrap();

    let (s3, _) = process_command(
        s2,
        Command::TapForMana {
            player: p1,
            source: id_c,
            ability_index: 0,
        },
    )
    .unwrap();

    assert_eq!(
        s3.player(p1).unwrap().mana_pool.colorless,
        3,
        "Sacrificing 3 Treasures should produce 3 colorless mana"
    );

    assert_eq!(
        count_on_battlefield(&s3, "Treasure-A")
            + count_on_battlefield(&s3, "Treasure-B")
            + count_on_battlefield(&s3, "Treasure-C"),
        0,
        "All 3 Treasures should be off the battlefield"
    );
}

// ── Test 6: Already-tapped Treasure cannot be activated ─────────────────────

#[test]
/// CR 602.2b — If a permanent is already tapped, the {{T}} cost cannot be paid.
/// A tapped Treasure cannot have its mana ability activated.
fn test_treasure_already_tapped_cannot_activate() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(treasure_spec(p1, "Treasure").tapped())
        .build()
        .unwrap();

    let treasure_id = find_by_name(&state, "Treasure");

    let result = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: treasure_id,
            ability_index: 0,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: Activating a tapped Treasure should fail"
    );
    assert!(
        matches!(
            result.err().unwrap(),
            mtg_engine::GameStateError::PermanentAlreadyTapped(_)
        ),
        "Error should be PermanentAlreadyTapped"
    );
}

// ── Test 7: Summoning sickness does NOT prevent Treasure activation ──────────

#[test]
/// CR 302.6 — Summoning sickness only restricts creatures from using {{T}} abilities.
/// Artifacts (like Treasure tokens) are not creatures, so summoning sickness
/// does NOT prevent activating the {{T}} mana ability.
///
/// The builder sets has_summoning_sickness = false for test-placed permanents
/// (they are treated as having been on the battlefield since the turn started).
/// We verify the Treasure is NOT a creature, which is the key property — the
/// summoning sickness check only applies to CardType::Creature.
fn test_treasure_not_affected_by_summoning_sickness() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(treasure_spec(p1, "Treasure"))
        .build()
        .unwrap();

    let treasure_id = find_by_name(&state, "Treasure");

    // Verify Treasure is not a creature — this is why summoning sickness can't apply.
    let obj = state.object(treasure_id).unwrap();
    assert!(
        !obj.characteristics.card_types.contains(&CardType::Creature),
        "Treasure is not a creature; summoning sickness cannot apply to it"
    );

    // Verify the mana ability activation succeeds — Treasure is an artifact, not a creature.
    let result = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: treasure_id,
            ability_index: 0,
        },
    );

    assert!(
        result.is_ok(),
        "CR 302.6: Summoning sickness should not prevent Treasure activation, got: {:?}",
        result.err()
    );
}

// ── Test 8: Token ceases to exist after sacrifice (SBA) ─────────────────────

#[test]
/// CR 704.5d — Tokens in non-battlefield zones cease to exist as a state-based action.
/// After a Treasure is sacrificed (moved to graveyard), running SBAs removes it entirely.
fn test_treasure_token_ceases_to_exist_after_sba() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(treasure_spec(p1, "Treasure"))
        .build()
        .unwrap();

    let treasure_id = find_by_name(&state, "Treasure");

    // Sacrifice the Treasure.
    let (after_sacrifice, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: treasure_id,
            ability_index: 0,
        },
    )
    .unwrap();

    // Token is in the graveyard before SBA check.
    assert!(
        after_sacrifice
            .objects
            .values()
            .any(|o| o.characteristics.name == "Treasure" && o.zone == ZoneId::Graveyard(p1)),
        "Treasure should be in graveyard before SBA"
    );

    // Run SBAs — token should cease to exist.
    let mut after_sba = after_sacrifice;
    let sba_events = check_and_apply_sbas(&mut after_sba);

    assert!(
        sba_events
            .iter()
            .any(|e| matches!(e, GameEvent::TokenCeasedToExist { .. })),
        "CR 704.5d: TokenCeasedToExist event should be emitted"
    );

    // Token no longer exists in any zone.
    assert!(
        !after_sba
            .objects
            .values()
            .any(|o| o.characteristics.name == "Treasure"),
        "CR 704.5d: Token should no longer exist in any zone after SBA"
    );
}

// ── Test 9: Opponent cannot activate another player's Treasure ───────────────

#[test]
/// CR 602.2 — Only the controller of a permanent can activate its abilities.
/// Player 2 cannot activate Player 1's Treasure token.
fn test_treasure_cannot_be_activated_by_opponent() {
    let p1 = p(1);
    let p2 = p(2);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p2) // p2 has priority
        .object(treasure_spec(p1, "Treasure")) // p1 owns and controls the Treasure
        .build()
        .unwrap();

    let treasure_id = find_by_name(&state, "Treasure");

    // p2 tries to activate p1's Treasure.
    let result = process_command(
        state,
        Command::TapForMana {
            player: p2,
            source: treasure_id,
            ability_index: 0,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: Non-controller should not be able to activate another player's Treasure"
    );
    assert!(
        matches!(
            result.err().unwrap(),
            mtg_engine::GameStateError::NotController { .. }
        ),
        "Error should be NotController"
    );
}

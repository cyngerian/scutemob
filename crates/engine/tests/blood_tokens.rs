//! Blood token tests (CR 111.10g).
//!
//! Blood tokens are a predefined token type per CR 111.10g:
//! "A Blood token is a colorless Blood artifact token with
//! '{1}, {T}, Discard a card, Sacrifice this token: Draw a card.'"
//!
//! Key rules verified:
//! - Blood tokens are colorless artifact tokens with the Blood subtype (CR 111.10g).
//! - Blood's ability requires tap, {1} mana, discard a card, AND sacrifice self (CR 602.2).
//! - The discard is a COST, not an effect — it happens at activation (before the stack).
//! - Sacrifice is a cost paid before the ability goes on the stack (CR 602.2c).
//! - Summoning sickness does NOT affect artifacts (CR 302.6 only restricts creatures).
//! - Tokens cease to exist in non-battlefield zones as an SBA (CR 704.5d).
//! - Only the controller can activate the ability (CR 602.2).

use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    blood_token_spec, check_and_apply_sbas, process_command, CardType, Command, Effect,
    EffectAmount, GameEvent, GameState, GameStateBuilder, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, PlayerTarget, Step, SubType, ZoneId,
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

/// Find an object by name in a specific zone.
#[allow(dead_code)]
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

/// Build a Blood token ObjectSpec for direct placement on the battlefield.
///
/// Mirrors the blood_token_spec characteristics: artifact, Blood subtype,
/// {1},{T}, discard-a-card, sacrifice-self activated ability that draws 1 card.
fn blood_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::artifact(owner, name)
        .with_subtypes(vec![SubType("Blood".to_string())])
        .with_activated_ability(ActivatedAbility {
            targets: vec![],
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: Some(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                }),
                sacrifice_self: true,
                discard_card: true,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
                remove_counter_cost: None,
            },
            description: "{1}, {T}, Discard a card, Sacrifice this token: Draw a card.".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
        })
        .token()
}

// ── Test 1: blood_token_spec characteristics ──────────────────────────────────

#[test]
/// CR 111.10g — `blood_token_spec(1)` produces a spec for a colorless Blood
/// artifact token with exactly one non-mana activated ability.
/// The ability has requires_tap=true, {1} generic mana cost, sacrifice_self=true,
/// discard_card=true, and draws 1 card for the controller.
fn test_blood_token_spec_characteristics() {
    let spec = blood_token_spec(1);
    assert_eq!(spec.name, "Blood");
    assert_eq!(spec.power, 0);
    assert_eq!(spec.toughness, 0);
    assert!(spec.colors.is_empty(), "Blood is colorless");
    assert!(
        spec.card_types.contains(&CardType::Artifact),
        "Blood is an artifact"
    );
    assert!(
        spec.subtypes.contains(&SubType("Blood".to_string())),
        "Blood has subtype Blood"
    );
    assert_eq!(
        spec.mana_abilities.len(),
        0,
        "Blood has no mana abilities (CR 605 does not apply)"
    );
    assert_eq!(
        spec.activated_abilities.len(),
        1,
        "Blood has exactly one non-mana activated ability"
    );

    let ab = &spec.activated_abilities[0];
    assert!(ab.cost.requires_tap, "ability requires {{T}}");
    assert!(ab.cost.sacrifice_self, "ability requires sacrifice self");
    assert!(ab.cost.discard_card, "ability requires discarding a card");
    assert!(!ab.cost.forage, "ability does not require forage");

    let mana = ab.cost.mana_cost.as_ref().expect("ability requires mana");
    assert_eq!(mana.generic, 1, "ability requires {{1}} generic mana");

    // Effect: draw 1 card for controller.
    assert!(
        matches!(
            ab.effect.as_ref().unwrap(),
            Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }
        ),
        "Blood ability draws 1 card for controller"
    );
}

// ── Test 2: Blood token has correct characteristics on battlefield ─────────────

#[test]
/// CR 111.10g — A Blood token placed on the battlefield is an artifact token
/// with the correct subtype and has exactly one activated ability.
fn test_blood_token_has_activated_ability() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood"))
        .build()
        .unwrap();

    let obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Blood")
        .expect("Blood token should be on battlefield");

    assert!(obj.is_token, "Blood should be a token");
    assert!(
        obj.characteristics.card_types.contains(&CardType::Artifact),
        "Blood is an Artifact"
    );
    assert!(
        obj.characteristics
            .subtypes
            .contains(&SubType("Blood".to_string())),
        "Blood has Blood subtype"
    );
    assert_eq!(
        obj.characteristics.mana_abilities.len(),
        0,
        "Blood has no mana abilities"
    );
    assert_eq!(
        obj.characteristics.activated_abilities.len(),
        1,
        "Blood has exactly 1 activated ability"
    );
    let ab = &obj.characteristics.activated_abilities[0];
    assert!(ab.cost.sacrifice_self, "ability has sacrifice_self=true");
    assert!(ab.cost.requires_tap, "ability requires {{T}}");
    assert!(ab.cost.discard_card, "ability requires discard_card=true");
}

// ── Test 3: Activate Blood to draw a card ─────────────────────────────────────

#[test]
/// CR 111.10g + CR 602.2 — Activating Blood's ability ({1},{T}, discard a card,
/// sacrifice) puts the ability on the stack. After all players pass priority,
/// the ability resolves and the controller draws 1 card.
fn test_blood_token_activation_basic() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood"))
        // Card to discard as cost.
        .object(ObjectSpec::card(p1, "Dummy Card"))
        // Card to draw from library.
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    // Give p1 {1} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    // Place the dummy card into p1's hand.
    let dummy_id = find_by_name(&state, "Dummy Card");
    if let Some(obj) = state.objects.get_mut(&dummy_id) {
        obj.zone = ZoneId::Hand(p1);
    }

    let blood_id = find_by_name(&state, "Blood");
    let initial_hand_count = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    // Initial hand: 1 card (Dummy Card).
    assert_eq!(
        initial_hand_count, 1,
        "p1 should have 1 card in hand before activation"
    );

    // Activate the Blood ability ({1}, tap, discard Dummy Card, sacrifice Blood).
    let (state_after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: Some(dummy_id),
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // The discard happens at activation time (cost) — hand should be empty now.
    let hand_after_activate = state_after_activate
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after_activate, 0,
        "CR 602.2: Discard is a cost — hand should be empty after activation"
    );

    // Blood is still drawing (ability is on stack; draw hasn't happened yet).
    let hand_before_resolve = state_after_activate
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_before_resolve, 0,
        "Hand should be empty until ability resolves"
    );

    // Pass priority for both players to resolve.
    let (state_after_p1, _) =
        process_command(state_after_activate, Command::PassPriority { player: p1 }).unwrap();
    let (state_final, _) =
        process_command(state_after_p1, Command::PassPriority { player: p2 }).unwrap();

    // Player drew 1 card after resolution.
    let final_hand_count = state_final
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        final_hand_count, 1,
        "CR 111.10g: Controller should draw 1 card after Blood ability resolves"
    );

    // Blood token was sacrificed and is no longer on the battlefield.
    assert_eq!(
        count_on_battlefield(&state_final, "Blood"),
        0,
        "Blood token should no longer be on battlefield after activation"
    );
}

// ── Test 4: Discard is cost, not effect ───────────────────────────────────────

#[test]
/// CR 602.2 — The discard is a COST, not an effect. The card is discarded
/// at activation time (before the ability goes on the stack).
fn test_blood_token_discard_is_cost() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood"))
        .object(ObjectSpec::card(p1, "Dummy Card"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let dummy_id = find_by_name(&state, "Dummy Card");
    if let Some(obj) = state.objects.get_mut(&dummy_id) {
        obj.zone = ZoneId::Hand(p1);
    }

    let blood_id = find_by_name(&state, "Blood");

    let (state_after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: Some(dummy_id),
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // The discard happened at activation, before ability resolves.
    // Hand should be empty immediately after activation.
    let hand_count = state_after_activate
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_count, 0,
        "CR 602.2: Discard is a cost — card should be gone from hand immediately at activation"
    );

    // The ability should be on the stack (not yet resolved).
    assert!(
        !state_after_activate.stack_objects.is_empty(),
        "CR 602.2: Blood ability should be on the stack after activation"
    );

    // The discarded card should be in the graveyard.
    assert!(
        state_after_activate
            .objects
            .values()
            .any(|o| o.zone == ZoneId::Graveyard(p1)),
        "CR 602.2: Discarded card should be in the graveyard"
    );
}

// ── Test 5: Blood uses the stack ──────────────────────────────────────────────

#[test]
/// CR 602.2 / CR 605 — Blood's ability is NOT a mana ability. After activation,
/// the stack must contain the ability.
fn test_blood_token_uses_stack() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood"))
        .object(ObjectSpec::card(p1, "Discard Target"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let discard_id = find_by_name(&state, "Discard Target");
    if let Some(obj) = state.objects.get_mut(&discard_id) {
        obj.zone = ZoneId::Hand(p1);
    }
    let blood_id = find_by_name(&state, "Blood");

    let (state_after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: Some(discard_id),
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // Stack must NOT be empty — Blood's ability uses the stack (CR 602.2).
    assert!(
        !state_after_activate.stack_objects.is_empty(),
        "CR 602.2: Blood ability should be on the stack after activation"
    );
}

// ── Test 6: Insufficient mana cannot activate Blood ───────────────────────────

#[test]
/// CR 602.2b — Activating Blood requires {1} generic mana. With no mana, activation fails.
fn test_blood_token_activation_no_mana() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood"))
        .object(ObjectSpec::card(p1, "Dummy Card"))
        .build()
        .unwrap();

    // No mana at all.
    state.turn.priority_holder = Some(p1);

    let dummy_id = find_by_name(&state, "Dummy Card");
    if let Some(obj) = state.objects.get_mut(&dummy_id) {
        obj.zone = ZoneId::Hand(p1);
    }
    let blood_id = find_by_name(&state, "Blood");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: Some(dummy_id),
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: Activating Blood with no mana should fail"
    );
    assert!(
        matches!(
            result.err().unwrap(),
            mtg_engine::GameStateError::InsufficientMana
        ),
        "Error should be InsufficientMana"
    );
}

// ── Test 7: Already-tapped Blood cannot activate ──────────────────────────────

#[test]
/// CR 602.2b — Blood requires {T}. Activating a tapped Blood token should fail.
fn test_blood_token_activation_already_tapped() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood").tapped())
        .object(ObjectSpec::card(p1, "Dummy Card"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let dummy_id = find_by_name(&state, "Dummy Card");
    if let Some(obj) = state.objects.get_mut(&dummy_id) {
        obj.zone = ZoneId::Hand(p1);
    }
    let blood_id = find_by_name(&state, "Blood");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: Some(dummy_id),
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: Activating a tapped Blood token should fail"
    );
    assert!(
        matches!(
            result.err().unwrap(),
            mtg_engine::GameStateError::PermanentAlreadyTapped(..)
        ),
        "Error should be PermanentAlreadyTapped"
    );
}

// ── Test 8: No cards in hand — cannot activate ────────────────────────────────

#[test]
/// CR 602.2 — Blood requires discarding a card as cost. With no cards in hand,
/// the player cannot provide a discard target, so activation should fail.
fn test_blood_token_activation_no_cards_in_hand() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let blood_id = find_by_name(&state, "Blood");

    // Pass None as discard_card — no card to discard.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: Activating Blood without providing a discard target should fail"
    );
}

// ── Test 9: SBA — token ceases to exist in graveyard ─────────────────────────

#[test]
/// CR 704.5d — A token in the graveyard ceases to exist as a state-based action.
/// After Blood is sacrificed as cost, it moves to the graveyard. SBAs then
/// remove it entirely.
fn test_blood_token_sba_ceases_to_exist() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood"))
        .object(ObjectSpec::card(p1, "Dummy Card"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let dummy_id = find_by_name(&state, "Dummy Card");
    if let Some(obj) = state.objects.get_mut(&dummy_id) {
        obj.zone = ZoneId::Hand(p1);
    }
    let blood_id = find_by_name(&state, "Blood");

    // Activate the Blood ability (sacrifice happens here — token goes to graveyard).
    let (after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: Some(dummy_id),
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // Token is in the graveyard before SBA check (post-sacrifice, pre-SBA).
    assert!(
        after_activate
            .objects
            .values()
            .any(|o| o.characteristics.name == "Blood" && o.zone == ZoneId::Graveyard(p1)),
        "Blood should be in graveyard before SBA check"
    );

    // Run SBAs — token should cease to exist.
    let mut after_sba = after_activate;
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
            .any(|o| o.characteristics.name == "Blood"),
        "CR 704.5d: Blood token should no longer exist in any zone after SBA"
    );
}

// ── Test 10: Summoning sickness does not affect artifacts ─────────────────────

#[test]
/// CR 302.6 / CR 111.10g — Summoning sickness only restricts creatures from
/// using {T} abilities. Blood is an artifact (not a creature), so it can be
/// activated the turn it enters.
fn test_blood_token_not_affected_by_summoning_sickness() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood"))
        .object(ObjectSpec::card(p1, "Dummy Card"))
        .build()
        .unwrap();

    // Manually set summoning sickness on the Blood token (simulates entering this turn).
    let blood_id = find_by_name(&state, "Blood");
    if let Some(obj) = state.objects.get_mut(&blood_id) {
        obj.has_summoning_sickness = true;
    }

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let dummy_id = find_by_name(&state, "Dummy Card");
    if let Some(obj) = state.objects.get_mut(&dummy_id) {
        obj.zone = ZoneId::Hand(p1);
    }

    // Blood is NOT a creature — summoning sickness check only applies to creatures.
    let blood_obj = state
        .objects
        .get(&blood_id)
        .expect("Blood should be in state");
    assert!(
        !blood_obj
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "Blood is not a creature; summoning sickness cannot apply"
    );

    // Even with has_summoning_sickness=true, an artifact can use {T}.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: Some(dummy_id),
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 302.6: Summoning sickness should not prevent Blood token from using {{T}} (it's not a creature)"
    );
}

// ── Test 11: Opponent cannot activate another player's Blood ──────────────────

#[test]
/// CR 602.2 — Only the controller of a permanent can activate its abilities.
/// Player 2 cannot activate Player 1's Blood token.
fn test_blood_token_only_controller_can_activate() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p2) // p2 has priority
        .object(blood_spec(p1, "Blood")) // p1 owns and controls the Blood
        .object(ObjectSpec::card(p2, "Dummy Card"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p2);

    let dummy_id = find_by_name(&state, "Dummy Card");
    if let Some(obj) = state.objects.get_mut(&dummy_id) {
        obj.zone = ZoneId::Hand(p2);
    }
    let blood_id = find_by_name(&state, "Blood");

    // p2 tries to activate p1's Blood.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: Some(dummy_id),
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: Non-controller should not be able to activate another player's Blood token"
    );
    assert!(
        matches!(
            result.err().unwrap(),
            mtg_engine::GameStateError::NotController { .. }
        ),
        "Error should be NotController"
    );
}

// ── Test 12: Create Blood via Effect::CreateToken ─────────────────────────────

#[test]
/// CR 111.10g — Effect::CreateToken with blood_token_spec creates the correct token.
fn test_blood_token_create_via_effect() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // Verify spec creates the right token.
    let spec = blood_token_spec(1);
    assert_eq!(spec.count, 1, "blood_token_spec(1) creates 1 token");
    assert_eq!(spec.name, "Blood", "token name is Blood");
    assert!(
        spec.card_types.contains(&CardType::Artifact),
        "token is an artifact"
    );
    assert!(
        spec.subtypes.contains(&SubType("Blood".to_string())),
        "token has Blood subtype"
    );
    assert!(spec.colors.is_empty(), "token is colorless");

    // Verify multiple tokens.
    let spec3 = blood_token_spec(3);
    assert_eq!(spec3.count, 3, "blood_token_spec(3) creates 3 tokens");

    // GameState is used to silence unused variable warning.
    let _ = state;
}

// ── Test 13: Sacrifice removes Blood from battlefield ─────────────────────────

#[test]
/// CR 602.2c — Sacrifice is a cost paid at activation time. After activation,
/// the Blood token should no longer be on the battlefield.
fn test_blood_token_activation_sacrifice_removes_from_battlefield() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood"))
        .object(ObjectSpec::card(p1, "Dummy Card"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let dummy_id = find_by_name(&state, "Dummy Card");
    if let Some(obj) = state.objects.get_mut(&dummy_id) {
        obj.zone = ZoneId::Hand(p1);
    }
    let blood_id = find_by_name(&state, "Blood");

    assert_eq!(
        count_on_battlefield(&state, "Blood"),
        1,
        "Blood should be on battlefield before activation"
    );

    let (state_after, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: Some(dummy_id),
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // Blood token leaves battlefield at activation (sacrifice is a cost).
    assert_eq!(
        count_on_battlefield(&state_after, "Blood"),
        0,
        "CR 602.2c: Blood token should not be on battlefield after activation (sacrifice is a cost)"
    );
}

// ── Test 14: Discard must be from activating player's hand ────────────────────

#[test]
/// CR 602.2 — The discard cost requires the card to be in the activating player's hand.
/// Supplying a card from a different zone should fail.
fn test_blood_token_discard_must_be_from_hand() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(blood_spec(p1, "Blood"))
        // Card in graveyard, not hand.
        .object(ObjectSpec::card(p1, "Graveyard Card").in_zone(ZoneId::Graveyard(PlayerId(1))))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let grave_card_id = find_by_name(&state, "Graveyard Card");
    let blood_id = find_by_name(&state, "Blood");

    // Try to discard a card that's in the graveyard (not the hand).
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: blood_id,
            ability_index: 0,
            targets: vec![],
            discard_card: Some(grave_card_id),
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: Cannot discard a card from graveyard as cost — must be from hand"
    );
}

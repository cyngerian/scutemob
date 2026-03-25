//! Clue token tests (CR 111.10f).
//!
//! Clue tokens are a predefined token type per CR 111.10f:
//! "A Clue token is a colorless Clue artifact token with
//! '{2}, Sacrifice this token: Draw a card.'"
//!
//! Key rules verified:
//! - Clue tokens are colorless artifact tokens with the Clue subtype (CR 111.10f).
//! - Clue's ability does NOT require {T} — a tapped Clue can still be activated.
//! - Clue's ability is NOT a mana ability (CR 605). It uses the stack (CR 602.2).
//! - Sacrifice is a cost paid before the ability goes on the stack (CR 602.2b, CR 601.2h).
//! - The ability resolves after all players pass priority (CR 608).
//! - Summoning sickness does NOT affect artifacts (CR 302.6 only restricts creatures).
//! - Tokens cease to exist in non-battlefield zones as an SBA (CR 704.5d).
//! - Only the controller can activate the ability (CR 602.2).

use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    check_and_apply_sbas, clue_token_spec, process_command, CardType, Command, Effect,
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

/// Find an object by name in a specific zone. Returns None if not found.
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

/// Build a Clue token `ObjectSpec` for direct placement on the battlefield.
///
/// This mirrors the clue_token_spec characteristics: artifact, Clue subtype,
/// {2}, sacrifice-self activated ability that draws 1 card.
/// CRITICAL DIFFERENCE from Food: requires_tap is FALSE.
fn clue_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::artifact(owner, name)
        .with_subtypes(vec![SubType("Clue".to_string())])
        .with_activated_ability(ActivatedAbility {
            targets: vec![],
            cost: ActivationCost {
                requires_tap: false, // KEY DIFFERENCE from Food — no tap required
                mana_cost: Some(ManaCost {
                    generic: 2,
                    ..ManaCost::default()
                }),
                sacrifice_self: true,
                discard_card: false,
                discard_self: false,
                forage: false,
                sacrifice_filter: None,
            },
            description: "{2}, Sacrifice this token: Draw a card.".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
        })
        .token()
}

// ── Test 1: clue_token_spec characteristics ───────────────────────────────────

#[test]
/// CR 111.10f — `clue_token_spec(1)` produces a spec for a colorless Clue
/// artifact token with exactly one non-mana activated ability.
/// The ability has requires_tap=false (unlike Food), {2} generic mana cost,
/// sacrifice_self=true, and draws 1 card for the controller.
fn test_clue_token_spec_characteristics() {
    let spec = clue_token_spec(1);
    assert_eq!(spec.name, "Clue");
    assert_eq!(spec.power, 0);
    assert_eq!(spec.toughness, 0);
    assert!(spec.colors.is_empty(), "Clue is colorless");
    assert!(
        spec.card_types.contains(&CardType::Artifact),
        "Clue is an artifact"
    );
    assert!(
        spec.subtypes.contains(&SubType("Clue".to_string())),
        "Clue has subtype Clue"
    );
    assert_eq!(
        spec.mana_abilities.len(),
        0,
        "Clue has no mana abilities (CR 605 does not apply)"
    );
    assert_eq!(
        spec.activated_abilities.len(),
        1,
        "Clue has exactly one non-mana activated ability"
    );
    let ab = &spec.activated_abilities[0];
    assert!(
        !ab.cost.requires_tap,
        "CR 111.10f: Clue ability does NOT require {{T}} (unlike Food)"
    );
    assert!(
        ab.cost.sacrifice_self,
        "Clue ability requires sacrificing itself"
    );
    assert_eq!(
        ab.cost.mana_cost.as_ref().map(|mc| mc.generic).unwrap_or(0),
        2,
        "Clue ability requires {{2}} generic mana"
    );
}

// ── Test 2: Clue token on battlefield via ObjectSpec ──────────────────────────

#[test]
/// CR 111.10f — A Clue token placed on the battlefield is an artifact token
/// with the correct subtype and has exactly one activated ability with requires_tap=false.
fn test_clue_token_has_activated_ability() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(clue_spec(p1, "Clue"))
        .build()
        .unwrap();

    let obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Clue")
        .expect("Clue token should be on battlefield");

    assert!(obj.is_token, "Clue should be a token");
    assert!(
        obj.characteristics.card_types.contains(&CardType::Artifact),
        "Clue is an Artifact"
    );
    assert!(
        obj.characteristics
            .subtypes
            .contains(&SubType("Clue".to_string())),
        "Clue has Clue subtype"
    );
    assert_eq!(
        obj.characteristics.mana_abilities.len(),
        0,
        "Clue has no mana abilities"
    );
    assert_eq!(
        obj.characteristics.activated_abilities.len(),
        1,
        "Clue has exactly 1 activated ability"
    );
    let ab = &obj.characteristics.activated_abilities[0];
    assert!(
        ab.cost.sacrifice_self,
        "activated ability has sacrifice_self=true"
    );
    assert!(
        !ab.cost.requires_tap,
        "CR 111.10f: Clue ability does NOT require {{T}} (unlike Food)"
    );
}

// ── Test 3: Activate Clue to draw a card ──────────────────────────────────────

#[test]
/// CR 111.10f + CR 602.2 — Activating Clue's ability ({2}, sacrifice) puts the
/// ability on the stack. After all players pass priority, the ability resolves and
/// the controller draws 1 card.
///
/// A dummy card is placed in p1's library so the draw is not a no-op
/// (DrawCards on empty library is silently a no-op per gotchas-infra.md).
fn test_clue_activate_draw_card() {
    let p1 = p(1);
    let p2 = p(2);
    // A dummy artifact placed in the library gives p1 something to draw.
    let library_card = ObjectSpec::artifact(p1, "DummyCard").in_zone(ZoneId::Library(p1));
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(clue_spec(p1, "Clue"))
        .object(library_card)
        .build()
        .unwrap();

    // Give p1 {2} generic mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let clue_id = find_by_name(&state, "Clue");
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Activate the Clue ability (sacrifice + {2}, no tap needed).
    let (state_after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: clue_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // Hand size should not have changed yet (effect is on stack).
    let hand_after_activate = state_after_activate
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after_activate, initial_hand_size,
        "Hand size should not increase until ability resolves"
    );

    // Pass priority for both players to resolve.
    let (state_after_p1, _) =
        process_command(state_after_activate, Command::PassPriority { player: p1 }).unwrap();

    let (state_final, _) =
        process_command(state_after_p1, Command::PassPriority { player: p2 }).unwrap();

    // Player drew 1 card after resolution.
    let final_hand_size = state_final
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        final_hand_size,
        initial_hand_size + 1,
        "CR 111.10f: Controller should draw 1 card after Clue ability resolves"
    );

    // Clue token was sacrificed and is no longer on battlefield.
    assert_eq!(
        count_on_battlefield(&state_final, "Clue"),
        0,
        "Clue token should no longer be on battlefield"
    );
}

// ── Test 4: Clue uses the stack (NOT a mana ability) ─────────────────────────

#[test]
/// CR 602.2 / CR 605 — Clue's ability is NOT a mana ability. After activation,
/// the stack must contain the ability.
fn test_clue_uses_stack_not_mana_ability() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(clue_spec(p1, "Clue"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let clue_id = find_by_name(&state, "Clue");

    let (state_after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: clue_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // Stack must NOT be empty — Clue's ability uses the stack (CR 602.2).
    assert!(
        !state_after_activate.stack_objects.is_empty(),
        "CR 602.2: Clue ability should be on the stack after activation"
    );
}

// ── Test 5: Sacrifice is cost, paid before ability goes on stack ──────────────

#[test]
/// CR 602.2b / CR 601.2h — Sacrifice is a cost, not an effect. The Clue token is
/// gone from the battlefield the moment the ability is activated, before it resolves.
fn test_clue_sacrifice_is_cost_not_effect() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(clue_spec(p1, "Clue"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let clue_id = find_by_name(&state, "Clue");
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let (state_after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: clue_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .unwrap();

    // Clue is already off the battlefield immediately after activation (cost paid).
    assert_eq!(
        count_on_battlefield(&state_after_activate, "Clue"),
        0,
        "CR 602.2b / CR 601.2h: Clue token is sacrificed as a cost before ability resolves"
    );

    // But player has NOT drawn yet (effect is on stack, not resolved).
    let hand_after_activate = state_after_activate
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        hand_after_activate, initial_hand_size,
        "Player should not have drawn yet — ability is on the stack"
    );
}

// ── Test 6: Tapped Clue CAN still be activated (unlike Food) ─────────────────

#[test]
/// CR 111.10f — Clue does NOT require {T} in its cost. Therefore a tapped Clue
/// can still have its ability activated. This is the INVERSE of Food's test 6.
fn test_clue_tapped_can_still_activate() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(clue_spec(p1, "Clue").tapped()) // Clue is tapped
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let clue_id = find_by_name(&state, "Clue");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: clue_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 111.10f: A tapped Clue can still be activated (no {{T}} cost), got: {:?}",
        result.err()
    );
}

// ── Test 7: Summoning sickness does NOT prevent Clue activation ───────────────

#[test]
/// CR 602.5a / CR 302.6 — Summoning sickness only restricts creatures with {T}
/// abilities. Clue is an artifact (not a creature) and does not require {T}.
/// Activation should succeed.
fn test_clue_not_affected_by_summoning_sickness() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(clue_spec(p1, "Clue"))
        .build()
        .unwrap();

    // Verify Clue is NOT a creature — this is why summoning sickness cannot apply.
    let clue_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Clue")
        .expect("Clue token should be on battlefield");
    assert!(
        !clue_obj
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "Clue is not a creature; summoning sickness cannot apply to it"
    );

    // Give mana and attempt activation.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let clue_id = find_by_name(&state, "Clue");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: clue_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 302.6: Summoning sickness should not prevent Clue activation, got: {:?}",
        result.err()
    );
}

// ── Test 8: Token ceases to exist after SBA (CR 704.5d) ──────────────────────

#[test]
/// CR 704.5d — Tokens in non-battlefield zones cease to exist as a state-based action.
/// After a Clue is sacrificed (moved to graveyard), running SBAs removes it entirely.
fn test_clue_token_ceases_to_exist_after_sba() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(clue_spec(p1, "Clue"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let clue_id = find_by_name(&state, "Clue");

    // Activate the Clue ability (sacrifice happens here — token goes to graveyard).
    let (after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: clue_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
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
            .any(|o| o.characteristics.name == "Clue" && o.zone == ZoneId::Graveyard(p1)),
        "Clue should be in graveyard before SBA check"
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
            .any(|o| o.characteristics.name == "Clue"),
        "CR 704.5d: Token should no longer exist in any zone after SBA"
    );
}

// ── Test 9: Opponent cannot activate another player's Clue ────────────────────

#[test]
/// CR 602.2 — Only the controller of a permanent can activate its abilities.
/// Player 2 cannot activate Player 1's Clue token.
fn test_clue_opponent_cannot_activate() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p2) // p2 has priority
        .object(clue_spec(p1, "Clue")) // p1 owns and controls the Clue
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p2);

    let clue_id = find_by_name(&state, "Clue");

    // p2 tries to activate p1's Clue.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: clue_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: Non-controller should not be able to activate another player's Clue"
    );
    assert!(
        matches!(
            result.err().unwrap(),
            mtg_engine::GameStateError::NotController { .. }
        ),
        "Error should be NotController"
    );
}

// ── Test 10: Insufficient mana cannot activate Clue ───────────────────────────

#[test]
/// CR 602.2b — Activating Clue requires {2} generic mana. With only 1 mana, activation fails.
fn test_clue_insufficient_mana_cannot_activate() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(clue_spec(p1, "Clue"))
        .build()
        .unwrap();

    // Only 1 mana — not enough for {2}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let clue_id = find_by_name(&state, "Clue");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: clue_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: Activating Clue with insufficient mana should fail"
    );
}

// ── Test 11: Create Clue via Effect::CreateToken ──────────────────────────────

#[test]
/// CR 111.10f — Using `clue_token_spec` with `Effect::CreateToken` creates a Clue
/// token on the battlefield whose `activated_abilities` are correctly propagated by
/// `make_token` from the `TokenSpec` into the resulting `GameObject`.
///
/// This test exercises the `make_token` path in `effects/mod.rs` directly (unlike
/// test 2, which places a token via `ObjectSpec` bypassing `make_token`).
fn test_clue_create_via_effect() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source = ObjectId(0);
    let spec = clue_token_spec(1);
    let effect = Effect::CreateToken { spec };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    // Find the Clue token on the battlefield.
    let clue_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Clue" && o.zone == ZoneId::Battlefield)
        .expect("CR 111.10f: Clue token should be on battlefield after Effect::CreateToken");

    assert!(
        clue_obj.is_token,
        "created object should be flagged as a token"
    );

    // make_token must propagate activated_abilities from TokenSpec to Characteristics.
    assert_eq!(
        clue_obj.characteristics.activated_abilities.len(),
        1,
        "make_token must copy activated_abilities from TokenSpec (Clue has exactly 1)"
    );

    let ab = &clue_obj.characteristics.activated_abilities[0];

    // Verify cost: {2}, sacrifice self — NO tap.
    assert!(
        !ab.cost.requires_tap,
        "CR 111.10f: Clue ability does NOT require {{T}}"
    );
    assert!(
        ab.cost.sacrifice_self,
        "CR 111.10f: Clue ability requires sacrificing itself as a cost"
    );
    assert_eq!(
        ab.cost.mana_cost.as_ref().map(|mc| mc.generic).unwrap_or(0),
        2,
        "CR 111.10f: Clue ability requires {{2}} generic mana"
    );

    // Verify effect: DrawCards { Controller, Fixed(1) }.
    assert!(
        matches!(
            ab.effect.as_ref().unwrap(),
            Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }
        ),
        "CR 111.10f: Clue ability effect should be DrawCards(Controller, 1)"
    );
}

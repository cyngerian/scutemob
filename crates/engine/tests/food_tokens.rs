//! Food token tests (CR 111.10b).
//!
//! Food tokens are a predefined token type per CR 111.10b:
//! "A Food token is a colorless Food artifact token with
//! '{2}, {T}, Sacrifice this token: You gain 3 life.'"
//!
//! Key rules verified:
//! - Food tokens are colorless artifact tokens with the Food subtype (CR 111.10b).
//! - Food's ability is NOT a mana ability (CR 605). It uses the stack (CR 602.2).
//! - Sacrifice is a cost paid before the ability goes on the stack (CR 602.2b, CR 601.2h).
//! - The ability resolves after all players pass priority (CR 608).
//! - Summoning sickness does NOT affect artifacts (CR 302.6 only restricts creatures).
//! - Tokens cease to exist in non-battlefield zones as an SBA (CR 704.5d).
//! - Only the controller can activate the ability (CR 602.2).

use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    check_and_apply_sbas, food_token_spec, process_command, CardType, Command, Effect,
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

/// Build a Food token `ObjectSpec` for direct placement on the battlefield.
///
/// This mirrors the food_token_spec characteristics: artifact, Food subtype,
/// {2},{T}, sacrifice-self activated ability that gains 3 life.
fn food_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::artifact(owner, name)
        .with_subtypes(vec![SubType("Food".to_string())])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: Some(ManaCost {
                    generic: 2,
                    ..ManaCost::default()
                }),
                sacrifice_self: true,
                forage: false,
            },
            description: "{2}, {T}, Sacrifice this token: You gain 3 life.".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(3),
            }),
            sorcery_speed: false,
        })
        .token()
}

// ── Test 1: food_token_spec characteristics ───────────────────────────────────

#[test]
/// CR 111.10b — `food_token_spec(1)` produces a spec for a colorless Food
/// artifact token with exactly one non-mana activated ability.
/// The ability has requires_tap=true, {2} generic mana cost, sacrifice_self=true,
/// and grants 3 life to the controller.
fn test_food_token_spec_characteristics() {
    let spec = food_token_spec(1);
    assert_eq!(spec.name, "Food");
    assert_eq!(spec.power, 0);
    assert_eq!(spec.toughness, 0);
    assert!(spec.colors.is_empty(), "Food is colorless");
    assert!(
        spec.card_types.contains(&CardType::Artifact),
        "Food is an artifact"
    );
    assert!(
        spec.subtypes.contains(&SubType("Food".to_string())),
        "Food has subtype Food"
    );
    assert_eq!(
        spec.mana_abilities.len(),
        0,
        "Food has no mana abilities (CR 605 does not apply)"
    );
    assert_eq!(
        spec.activated_abilities.len(),
        1,
        "Food has exactly one non-mana activated ability"
    );
    let ab = &spec.activated_abilities[0];
    assert!(ab.cost.requires_tap, "Food ability requires {{T}}");
    assert!(
        ab.cost.sacrifice_self,
        "Food ability requires sacrificing itself"
    );
    assert_eq!(
        ab.cost.mana_cost.as_ref().map(|mc| mc.generic).unwrap_or(0),
        2,
        "Food ability requires {{2}} generic mana"
    );
}

// ── Test 2: Food token on battlefield via ObjectSpec ─────────────────────────

#[test]
/// CR 111.10b — A Food token placed on the battlefield is an artifact token
/// with the correct subtype and has exactly one activated ability.
fn test_food_token_has_activated_ability() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(food_spec(p1, "Food"))
        .build()
        .unwrap();

    let obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Food")
        .expect("Food token should be on battlefield");

    assert!(obj.is_token, "Food should be a token");
    assert!(
        obj.characteristics.card_types.contains(&CardType::Artifact),
        "Food is an Artifact"
    );
    assert!(
        obj.characteristics
            .subtypes
            .contains(&SubType("Food".to_string())),
        "Food has Food subtype"
    );
    assert_eq!(
        obj.characteristics.mana_abilities.len(),
        0,
        "Food has no mana abilities"
    );
    assert_eq!(
        obj.characteristics.activated_abilities.len(),
        1,
        "Food has exactly 1 activated ability"
    );
    let ab = &obj.characteristics.activated_abilities[0];
    assert!(
        ab.cost.sacrifice_self,
        "activated ability has sacrifice_self=true"
    );
    assert!(ab.cost.requires_tap, "activated ability requires {{T}}");
}

// ── Test 3: Activate Food to gain 3 life ─────────────────────────────────────

#[test]
/// CR 111.10b + CR 602.2 — Activating Food's ability ({2},{T}, sacrifice) puts the
/// ability on the stack. After all players pass priority, the ability resolves and
/// the controller gains 3 life.
fn test_food_activate_gain_3_life() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(food_spec(p1, "Food"))
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

    let food_id = find_by_name(&state, "Food");
    let initial_life = state.player(p1).unwrap().life_total;

    // Activate the Food ability (sacrifice + tap + {2}).
    let (state_after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: food_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    // Life should not have changed yet (effect is on stack).
    assert_eq!(
        state_after_activate.player(p1).unwrap().life_total,
        initial_life,
        "Life should not increase until ability resolves"
    );

    // Pass priority for both players to resolve.
    let (state_after_p1, _) =
        process_command(state_after_activate, Command::PassPriority { player: p1 }).unwrap();

    let (state_final, _) =
        process_command(state_after_p1, Command::PassPriority { player: p2 }).unwrap();

    // Player gained 3 life after resolution.
    assert_eq!(
        state_final.player(p1).unwrap().life_total,
        initial_life + 3,
        "CR 111.10b: Controller should gain 3 life after Food ability resolves"
    );

    // Food token was sacrificed and is no longer on battlefield.
    assert_eq!(
        count_on_battlefield(&state_final, "Food"),
        0,
        "Food token should no longer be on battlefield"
    );
}

// ── Test 4: Food uses the stack (NOT a mana ability) ─────────────────────────

#[test]
/// CR 602.2 / CR 605 — Food's ability is NOT a mana ability. After activation,
/// the stack must contain the ability (unlike Treasure, which resolves immediately).
fn test_food_uses_stack_not_mana_ability() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(food_spec(p1, "Food"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let food_id = find_by_name(&state, "Food");

    let (state_after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: food_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    // Stack must NOT be empty — Food's ability uses the stack (CR 602.2).
    assert!(
        !state_after_activate.stack_objects.is_empty(),
        "CR 602.2: Food ability should be on the stack after activation"
    );
}

// ── Test 5: Sacrifice is cost, paid before ability goes on stack ──────────────

#[test]
/// CR 602.2b / CR 601.2h — Sacrifice is a cost, not an effect. The Food token is gone from the
/// battlefield the moment the ability is activated, before it resolves.
fn test_food_sacrifice_is_cost_not_effect() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(food_spec(p1, "Food"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let food_id = find_by_name(&state, "Food");
    let initial_life = state.player(p1).unwrap().life_total;

    let (state_after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: food_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    // Food is already off the battlefield immediately after activation (cost paid).
    assert_eq!(
        count_on_battlefield(&state_after_activate, "Food"),
        0,
        "CR 602.2b / CR 601.2h: Food token is sacrificed as a cost before ability resolves"
    );

    // But player has NOT gained life yet (effect is on stack, not resolved).
    assert_eq!(
        state_after_activate.player(p1).unwrap().life_total,
        initial_life,
        "Player should not have gained life yet — ability is on the stack"
    );
}

// ── Test 6: Already-tapped Food cannot be activated ──────────────────────────

#[test]
/// CR 602.2b — If a permanent is already tapped, the {T} cost cannot be paid.
/// A tapped Food token cannot have its ability activated.
fn test_food_already_tapped_cannot_activate() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(food_spec(p1, "Food").tapped())
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let food_id = find_by_name(&state, "Food");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: food_id,
            ability_index: 0,
            targets: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: Activating a tapped Food token should fail"
    );
    assert!(
        matches!(
            result.err().unwrap(),
            mtg_engine::GameStateError::PermanentAlreadyTapped(_)
        ),
        "Error should be PermanentAlreadyTapped"
    );
}

// ── Test 7: Summoning sickness does NOT prevent Food activation ───────────────

#[test]
/// CR 602.5a / CR 302.6 — Summoning sickness only restricts {T} abilities on creatures.
/// Food tokens are artifacts, not creatures, so summoning sickness cannot prevent
/// activating the ability.
fn test_food_not_affected_by_summoning_sickness() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(food_spec(p1, "Food"))
        .build()
        .unwrap();

    // Verify Food is NOT a creature — this is why summoning sickness cannot apply.
    let food_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Food")
        .expect("Food token should be on battlefield");
    assert!(
        !food_obj
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "Food is not a creature; summoning sickness cannot apply to it"
    );

    // Give mana and attempt activation.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let food_id = find_by_name(&state, "Food");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: food_id,
            ability_index: 0,
            targets: vec![],
        },
    );

    assert!(
        result.is_ok(),
        "CR 302.6: Summoning sickness should not prevent Food activation, got: {:?}",
        result.err()
    );
}

// ── Test 8: Token ceases to exist after SBA (CR 704.5d) ──────────────────────

#[test]
/// CR 704.5d — Tokens in non-battlefield zones cease to exist as a state-based action.
/// After a Food is sacrificed (moved to graveyard), running SBAs removes it entirely.
fn test_food_token_ceases_to_exist_after_sba() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(food_spec(p1, "Food"))
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let food_id = find_by_name(&state, "Food");

    // Activate the Food ability (sacrifice happens here — token goes to graveyard).
    let (after_activate, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: food_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    // Token is in the graveyard before SBA check (post-sacrifice, pre-SBA).
    assert!(
        after_activate
            .objects
            .values()
            .any(|o| o.characteristics.name == "Food" && o.zone == ZoneId::Graveyard(p1)),
        "Food should be in graveyard before SBA check"
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
            .any(|o| o.characteristics.name == "Food"),
        "CR 704.5d: Token should no longer exist in any zone after SBA"
    );
}

// ── Test 9: Opponent cannot activate another player's Food ────────────────────

#[test]
/// CR 602.2 — Only the controller of a permanent can activate its abilities.
/// Player 2 cannot activate Player 1's Food token.
fn test_food_opponent_cannot_activate() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p2) // p2 has priority
        .object(food_spec(p1, "Food")) // p1 owns and controls the Food
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p2);

    let food_id = find_by_name(&state, "Food");

    // p2 tries to activate p1's Food.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: food_id,
            ability_index: 0,
            targets: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: Non-controller should not be able to activate another player's Food"
    );
    assert!(
        matches!(
            result.err().unwrap(),
            mtg_engine::GameStateError::NotController { .. }
        ),
        "Error should be NotController"
    );
}

// ── Test 10: Insufficient mana cannot activate Food ───────────────────────────

#[test]
/// CR 602.2b — Activating Food requires {2} generic mana. With only 1 mana, activation fails.
fn test_food_insufficient_mana_cannot_activate() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(food_spec(p1, "Food"))
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

    let food_id = find_by_name(&state, "Food");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: food_id,
            ability_index: 0,
            targets: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: Activating Food with insufficient mana should fail"
    );
}

// ── Test 11: Create Food via Effect::CreateToken ──────────────────────────────

#[test]
/// CR 111.10b — Using `food_token_spec` with `Effect::CreateToken` creates a Food
/// token on the battlefield whose `activated_abilities` are correctly propagated by
/// `make_token` from the `TokenSpec` into the resulting `GameObject`.
///
/// This test exercises the `make_token` path in `effects/mod.rs` directly (unlike
/// test 2, which places a token via `ObjectSpec` bypassing `make_token`).
fn test_food_create_via_effect() {
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
    let spec = food_token_spec(1);
    let effect = Effect::CreateToken { spec };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    // Find the Food token on the battlefield.
    let food_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Food" && o.zone == ZoneId::Battlefield)
        .expect("CR 111.10b: Food token should be on battlefield after Effect::CreateToken");

    assert!(
        food_obj.is_token,
        "created object should be flagged as a token"
    );

    // make_token must propagate activated_abilities from TokenSpec to Characteristics.
    assert_eq!(
        food_obj.characteristics.activated_abilities.len(),
        1,
        "make_token must copy activated_abilities from TokenSpec (Food has exactly 1)"
    );

    let ab = &food_obj.characteristics.activated_abilities[0];

    // Verify cost: {2}, {T}, sacrifice self.
    assert!(
        ab.cost.requires_tap,
        "CR 111.10b: Food ability requires {{T}}"
    );
    assert!(
        ab.cost.sacrifice_self,
        "CR 111.10b: Food ability requires sacrificing itself as a cost"
    );
    assert_eq!(
        ab.cost.mana_cost.as_ref().map(|mc| mc.generic).unwrap_or(0),
        2,
        "CR 111.10b: Food ability requires {{2}} generic mana"
    );

    // Verify effect: GainLife { Controller, Fixed(3) }.
    assert!(
        matches!(
            ab.effect.as_ref().unwrap(),
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(3),
            }
        ),
        "CR 111.10b: Food ability effect should be GainLife(Controller, 3)"
    );
}

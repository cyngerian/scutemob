//! Embalm keyword ability tests (CR 702.128).
//!
//! Embalm is an activated ability that functions while the card is in a graveyard.
//! "Embalm [cost]" means "[Cost], Exile this card from your graveyard: Create a token
//! that's a copy of this card, except it's white, it has no mana cost, and it's a
//! Zombie in addition to its other types. Activate only as a sorcery." (CR 702.128a)
//!
//! Key rules verified:
//! - Card is exiled as part of the activation cost (not at resolution) (CR 702.128a).
//! - Token is white only (all original colors replaced) (CR 702.128a, CR 707.9b).
//! - Token has no mana cost (mana value 0) (CR 702.128a, CR 707.9d).
//! - Token is a Zombie in addition to its other types (CR 702.128a, CR 707.9a).
//! - Token keeps printed abilities from the card (CR 707.2 copiable values).
//! - Sorcery-speed restriction: active player only, main phase, empty stack (CR 702.128a).
//! - Token has summoning sickness (CR 302.6).
//! - Embalm is NOT a cast: no SpellCast event (ruling 2017-04-18).
//! - Requires mana payment; error on insufficient mana (CR 602.2b).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Color,
    Command, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, SubType, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Graveyard(owner)).is_some()
}

fn in_exile(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Exile).is_some()
}

/// Pass priority for all listed players once.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
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

/// Sacred Cat: {W}, 1/1, Cat creature, Lifelink, Embalm {W}.
/// The canonical embalm test card (white creature → white Zombie Cat token).
fn sacred_cat_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("sacred-cat".to_string()),
        name: "Sacred Cat".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Cat".to_string())].into_iter().collect(),
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        oracle_text: "Lifelink\nEmbalm {W}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Keyword(KeywordAbility::Embalm),
            AbilityDefinition::Embalm {
                cost: ManaCost {
                    white: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Build an ObjectSpec for Sacred Cat enriched with its card definition, placed in graveyard.
fn cat_in_graveyard(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Sacred Cat")
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("sacred-cat".to_string()))
        .with_keyword(KeywordAbility::Lifelink)
        .with_keyword(KeywordAbility::Embalm)
}

// ── Test 1: Basic embalm — card exiled immediately, token created on resolution ─

#[test]
/// CR 702.128a — Activate embalm on a creature in the graveyard; card is exiled
/// immediately as cost; token enters the battlefield when the ability resolves.
fn test_embalm_basic_create_token() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cat_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {W} mana for embalm cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Sacred Cat");

    // p1 activates embalm.
    let (state, activate_events) = process_command(
        state,
        Command::EmbalmCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EmbalmCard should succeed");

    // AbilityActivated event emitted.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "CR 702.128a: AbilityActivated event expected when embalm is activated"
    );

    // Card is ALREADY in exile (exiled as cost, unlike Unearth).
    assert!(
        !in_graveyard(&state, "Sacred Cat", p1),
        "CR 702.128a: card should be exiled immediately (cost payment), not in graveyard"
    );
    assert!(
        in_exile(&state, "Sacred Cat"),
        "CR 702.128a: card should be in exile after activation (exiled as cost)"
    );

    // Ability is on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "CR 702.128a: EmbalmAbility should be on the stack"
    );

    // Both players pass priority → ability resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Token created event emitted.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::TokenCreated { player, .. } if *player == p1)),
        "CR 702.128a: TokenCreated event expected after embalm resolves"
    );

    // PermanentEnteredBattlefield event emitted.
    assert!(
        resolve_events.iter().any(
            |e| matches!(e, GameEvent::PermanentEnteredBattlefield { player, .. } if *player == p1)
        ),
        "CR 702.128a: PermanentEnteredBattlefield event expected after embalm resolves"
    );

    // Token is on the battlefield.
    assert!(
        on_battlefield(&state, "Sacred Cat"),
        "CR 702.128a: a token named 'Sacred Cat' should be on the battlefield"
    );

    // Token is indeed a token.
    let token_id = find_in_zone(&state, "Sacred Cat", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();
    assert!(
        token_obj.is_token,
        "CR 702.128a: the object on battlefield should be a token, not the original card"
    );

    // Original card remains in exile (not returned).
    assert!(
        in_exile(&state, "Sacred Cat"),
        "CR 702.128a: original card should remain in exile after token is created"
    );
}

// ── Test 2: Token color is White only ────────────────────────────────────────

#[test]
/// CR 702.128a — "except it's white": token color is White only, replacing all
/// original colors. CR 707.9b: the modified color becomes the copiable value.
fn test_embalm_token_is_white() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cat_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Sacred Cat");

    let (state, _) = process_command(
        state,
        Command::EmbalmCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EmbalmCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let token_id = find_in_zone(&state, "Sacred Cat", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();

    // Token must be White only.
    assert!(
        token_obj.characteristics.colors.contains(&Color::White),
        "CR 702.128a: embalm token must be White"
    );
    assert_eq!(
        token_obj.characteristics.colors.len(),
        1,
        "CR 702.128a / CR 707.9b: embalm token should have exactly one color (White)"
    );
}

// ── Test 3: Token has no mana cost ────────────────────────────────────────────

#[test]
/// CR 702.128a — "it has no mana cost": token's mana cost is None.
/// CR 707.9d: mana value of the token is 0.
fn test_embalm_token_has_no_mana_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cat_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Sacred Cat");

    let (state, _) = process_command(
        state,
        Command::EmbalmCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EmbalmCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let token_id = find_in_zone(&state, "Sacred Cat", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();

    // Token must have no mana cost.
    assert!(
        token_obj.characteristics.mana_cost.is_none(),
        "CR 702.128a / CR 707.9d: embalm token must have no mana cost (mana value 0)"
    );
}

// ── Test 4: Token is a Zombie in addition to its other types ─────────────────

#[test]
/// CR 702.128a — "it's a Zombie in addition to its other types": token has the
/// Zombie subtype added to its existing subtypes. CR 707.9a exception: the
/// "in addition to" clause means existing type CDAs are also copied.
fn test_embalm_token_is_zombie() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cat_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Sacred Cat");

    let (state, _) = process_command(
        state,
        Command::EmbalmCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EmbalmCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let token_id = find_in_zone(&state, "Sacred Cat", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();

    // Token must have Zombie subtype.
    assert!(
        token_obj
            .characteristics
            .subtypes
            .contains(&SubType("Zombie".to_string())),
        "CR 702.128a: embalm token must have the Zombie subtype"
    );

    // Token must also have its original Cat subtype.
    assert!(
        token_obj
            .characteristics
            .subtypes
            .contains(&SubType("Cat".to_string())),
        "CR 702.128a: embalm token must retain its original Cat subtype"
    );

    // Token must be a Creature card type.
    assert!(
        token_obj
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "CR 702.128a: embalm token must be a Creature"
    );
}

// ── Test 5: Token keeps printed abilities ─────────────────────────────────────

#[test]
/// CR 707.2 (copiable values) — The token copies the printed abilities of the
/// original card (oracle text). Sacred Cat has Lifelink; the token must also
/// have Lifelink.
fn test_embalm_token_keeps_abilities() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cat_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Sacred Cat");

    let (state, _) = process_command(
        state,
        Command::EmbalmCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EmbalmCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let token_id = find_in_zone(&state, "Sacred Cat", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();

    // Token must have Lifelink (copied from printed abilities).
    assert!(
        token_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Lifelink),
        "CR 707.2: embalm token should retain Lifelink from the original card's printed abilities"
    );
}

// ── Test 6: Sorcery speed restriction ─────────────────────────────────────────

#[test]
/// CR 702.128a — "Activate only as a sorcery": embalm can only be activated
/// during the active player's own main phase with an empty stack.
fn test_embalm_sorcery_speed_restriction() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    // Test 1: Cannot activate during opponent's turn (active player is p2).
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(cat_in_graveyard(p1))
            .active_player(p2) // p2 is active
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::White, 1);
        state.turn.priority_holder = Some(p1);

        let card_obj_id = find_object(&state, "Sacred Cat");

        let result = process_command(
            state,
            Command::EmbalmCard {
                player: p1,
                card: card_obj_id,
            },
        );
        assert!(
            result.is_err(),
            "CR 702.128a: embalm cannot be activated during opponent's turn"
        );
    }

    // Test 2: Cannot activate during combat.
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(cat_in_graveyard(p1))
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .build()
            .unwrap();

        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::White, 1);
        state.turn.priority_holder = Some(p1);

        let card_obj_id = find_object(&state, "Sacred Cat");

        let result = process_command(
            state,
            Command::EmbalmCard {
                player: p1,
                card: card_obj_id,
            },
        );
        assert!(
            result.is_err(),
            "CR 702.128a: embalm cannot be activated during combat"
        );
    }

    // Test 3: Cannot activate when card is not in graveyard (e.g., already in exile).
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(
                ObjectSpec::card(p1, "Sacred Cat")
                    .in_zone(ZoneId::Exile) // Wrong zone
                    .with_card_id(CardId("sacred-cat".to_string()))
                    .with_keyword(KeywordAbility::Embalm),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::White, 1);
        state.turn.priority_holder = Some(p1);

        let card_obj_id = find_object(&state, "Sacred Cat");

        let result = process_command(
            state,
            Command::EmbalmCard {
                player: p1,
                card: card_obj_id,
            },
        );
        assert!(
            result.is_err(),
            "CR 702.128a: embalm cannot be activated when card is not in graveyard"
        );
    }
}

// ── Test 7: Card exiled as cost (before ability resolves) ─────────────────────

#[test]
/// CR 702.128a, ruling 2017-07-14 — The card is exiled immediately as part of
/// the activation cost, before the ability goes on the stack. Unlike Unearth
/// where the card stays in the graveyard until resolution, Embalm's exile is
/// the cost payment itself.
fn test_embalm_card_exiled_as_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cat_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Sacred Cat");

    // Activate embalm.
    let (state, _) = process_command(
        state,
        Command::EmbalmCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EmbalmCard should succeed");

    // BEFORE resolution (ability is on the stack), card is already in exile.
    assert!(
        !state.stack_objects.is_empty(),
        "EmbalmAbility should be on the stack before resolution"
    );
    assert!(
        !in_graveyard(&state, "Sacred Cat", p1),
        "CR 702.128a: card is NOT in graveyard after activation (exiled as cost)"
    );
    assert!(
        in_exile(&state, "Sacred Cat"),
        "CR 702.128a: card IS in exile immediately after activation (exiled as cost)"
    );
}

// ── Test 8: Embalm is not a cast ──────────────────────────────────────────────

#[test]
/// Ruling 2017-04-18 — Embalm is an activated ability, not a spell cast.
/// No SpellCast event fires, and spells_cast_this_turn is unchanged.
fn test_embalm_is_not_a_cast() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cat_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let initial_spells_cast = state.players.get(&p1).unwrap().spells_cast_this_turn;

    let card_obj_id = find_object(&state, "Sacred Cat");

    let (state, activate_events) = process_command(
        state,
        Command::EmbalmCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EmbalmCard should succeed");

    // No SpellCast event.
    assert!(
        !activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "Embalm is NOT a cast: no SpellCast event should fire"
    );

    // spells_cast_this_turn is unchanged.
    let after_spells_cast = state.players.get(&p1).unwrap().spells_cast_this_turn;
    assert_eq!(
        initial_spells_cast, after_spells_cast,
        "Embalm is NOT a cast: spells_cast_this_turn should not change"
    );
}

// ── Test 9: Card exiled as cost — ability state verification ─────────────────

#[test]
/// CR 702.128a, ruling 2017-07-14 — After activation but before resolution,
/// the card is in exile (exiled as cost) and the EmbalmAbility is on the stack.
/// This verifies the two-phase nature: cost paid (exile) → ability on stack → resolve.
fn test_embalm_ability_on_stack_card_in_exile() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cat_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Sacred Cat");

    let (state, activate_events) = process_command(
        state,
        Command::EmbalmCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EmbalmCard should succeed");

    // Exactly one EmbalmAbility is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.128a: exactly one EmbalmAbility should be on the stack"
    );

    // Card is in exile (exiled as cost, not waiting for resolution).
    assert!(
        !in_graveyard(&state, "Sacred Cat", p1),
        "CR 702.128a: card is NOT in graveyard after activation"
    );
    assert!(
        in_exile(&state, "Sacred Cat"),
        "CR 702.128a: card IS in exile as part of cost payment (ruling 2017-07-14)"
    );

    // ObjectExiled event was emitted during activation.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::ObjectExiled { player, .. } if *player == p1)),
        "CR 702.128a: ObjectExiled event should fire during activation (cost payment)"
    );

    // No token yet — need to resolve the ability first.
    assert!(
        !on_battlefield(&state, "Sacred Cat"),
        "CR 702.128a: no token on battlefield until ability resolves"
    );
}

// ── Test 10: Requires mana payment ───────────────────────────────────────────

#[test]
/// CR 602.2b — Embalm requires paying the embalm cost. Attempting to activate
/// without sufficient mana returns an error.
fn test_embalm_requires_mana_payment() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cat_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // No mana added — cannot pay {W}.
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Sacred Cat");

    let result = process_command(
        state,
        Command::EmbalmCard {
            player: p1,
            card: card_obj_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: embalm without sufficient mana should return an error"
    );
}

// ── Test 11: Multiplayer — only active player can embalm ─────────────────────

#[test]
/// CR 702.128a sorcery speed — Embalm can only be activated by the active player.
/// A non-active player attempting to embalm gets an error.
fn test_embalm_multiplayer_only_active_player() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    // p1 is active; p3 tries to embalm.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(cat_in_graveyard(p3))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p3)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p3);

    let card_obj_id = find_object(&state, "Sacred Cat");

    let result = process_command(
        state,
        Command::EmbalmCard {
            player: p3,
            card: card_obj_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.128a: non-active player should not be able to activate embalm"
    );
}

// ── Test 12: Token has summoning sickness ────────────────────────────────────

#[test]
/// CR 302.6 — Tokens enter the battlefield with summoning sickness. The embalm
/// token has has_summoning_sickness = true when it enters the battlefield.
fn test_embalm_token_has_summoning_sickness() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![sacred_cat_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(cat_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Sacred Cat");

    let (state, _) = process_command(
        state,
        Command::EmbalmCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EmbalmCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let token_id = find_in_zone(&state, "Sacred Cat", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();

    assert!(
        token_obj.has_summoning_sickness,
        "CR 302.6: embalm token should have summoning sickness when it enters the battlefield"
    );
}

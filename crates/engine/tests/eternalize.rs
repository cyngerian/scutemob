//! Eternalize keyword ability tests (CR 702.129).
//!
//! Eternalize is an activated ability that functions while the card is in a graveyard.
//! "Eternalize [cost]" means "[Cost], Exile this card from your graveyard: Create a token
//! that's a copy of this card, except it's black, it's 4/4, it has no mana cost, and it's
//! a Zombie in addition to its other types. Activate only as a sorcery." (CR 702.129a)
//!
//! Key rules verified:
//! - Card is exiled as part of the activation cost (not at resolution) (CR 702.129a).
//! - Token is black only (all original colors replaced) (CR 702.129a, CR 707.9b).
//! - Token is 4/4 (original P/T overridden) (CR 702.129a, CR 707.9b).
//! - Token has no mana cost (mana value 0) (CR 702.129a, CR 707.9d).
//! - Token is a Zombie in addition to its other types (CR 702.129a).
//! - Token keeps printed keyword abilities from the card (CR 707.2 copiable values).
//! - Sorcery-speed restriction: active player only, main phase, empty stack (CR 702.129a).
//! - Token has summoning sickness (CR 302.6).
//! - Eternalize is NOT a cast: no SpellCast event (ruling 2017-07-14).
//! - Requires mana payment; error on insufficient mana (CR 602.2b).

use mtg_engine::state::types::AltCostKind;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Color,
    Command, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, StackObject, StackObjectKind, Step, SubType, TypeLine, ZoneId,
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

/// Proven Combatant: {U}, 1/1, Human Warrior creature, Eternalize {4}{U}{U}.
/// The canonical eternalize test card (blue creature → black 4/4 Zombie Human Warrior token).
fn proven_combatant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("proven-combatant".to_string()),
        name: "Proven Combatant".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Human".to_string()), SubType("Warrior".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        oracle_text: "Eternalize {4}{U}{U}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Eternalize),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Eternalize,
                details: None,
                cost: ManaCost {
                    generic: 4,
                    blue: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Haste Warrior: {R}, 2/2, Human Warrior creature, Haste, Eternalize {4}{R}{R}.
/// Used for testing that printed keyword abilities are retained on the token.
fn haste_warrior_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("haste-warrior".to_string()),
        name: "Haste Warrior".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Human".to_string()), SubType("Warrior".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        oracle_text: "Haste\nEternalize {4}{R}{R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Keyword(KeywordAbility::Eternalize),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Eternalize,
                details: None,
                cost: ManaCost {
                    generic: 4,
                    red: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Build an ObjectSpec for Proven Combatant enriched with its card definition, placed in graveyard.
fn combatant_in_graveyard(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Proven Combatant")
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("proven-combatant".to_string()))
        .with_keyword(KeywordAbility::Eternalize)
}

// ── Test 1: Basic eternalize — card exiled immediately, token created on resolution ─

#[test]
/// CR 702.129a — Activate eternalize on a creature in the graveyard; card is exiled
/// immediately as cost; token enters the battlefield when the ability resolves.
fn test_eternalize_basic_flow() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(combatant_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {4}{U}{U} mana for eternalize cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Proven Combatant");

    // p1 activates eternalize.
    let (state, activate_events) = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EternalizeCard should succeed");

    // AbilityActivated event emitted.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "CR 702.129a: AbilityActivated event expected when eternalize is activated"
    );

    // Card is ALREADY in exile (exiled as cost, unlike Unearth).
    assert!(
        !in_graveyard(&state, "Proven Combatant", p1),
        "CR 702.129a: card should be exiled immediately (cost payment), not in graveyard"
    );
    assert!(
        in_exile(&state, "Proven Combatant"),
        "CR 702.129a: card should be in exile after activation (exiled as cost)"
    );

    // Ability is on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "CR 702.129a: EternalizeAbility should be on the stack"
    );

    // Both players pass priority → ability resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // Token created event emitted.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::TokenCreated { player, .. } if *player == p1)),
        "CR 702.129a: TokenCreated event expected after eternalize resolves"
    );

    // PermanentEnteredBattlefield event emitted.
    assert!(
        resolve_events.iter().any(
            |e| matches!(e, GameEvent::PermanentEnteredBattlefield { player, .. } if *player == p1)
        ),
        "CR 702.129a: PermanentEnteredBattlefield event expected after eternalize resolves"
    );

    // Token is on the battlefield.
    assert!(
        on_battlefield(&state, "Proven Combatant"),
        "CR 702.129a: a token named 'Proven Combatant' should be on the battlefield"
    );

    // Token is indeed a token.
    let token_id = find_in_zone(&state, "Proven Combatant", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();
    assert!(
        token_obj.is_token,
        "CR 702.129a: the object on battlefield should be a token, not the original card"
    );

    // Original card remains in exile (not returned).
    assert!(
        in_exile(&state, "Proven Combatant"),
        "CR 702.129a: original card should remain in exile after token is created"
    );
}

// ── Test 2: Token is Black 4/4 ────────────────────────────────────────────────

#[test]
/// CR 702.129a — "except it's black, it's 4/4": token color is Black only (replacing
/// all original colors) and P/T is overridden to 4/4 (regardless of original P/T).
/// CR 707.9b: both modifications become copiable values.
fn test_eternalize_token_is_black_4_4() {
    let p1 = p(1);
    let p2 = p(2);

    // Proven Combatant is blue 1/1 — token must be black 4/4.
    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(combatant_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Proven Combatant");

    let (state, _) = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EternalizeCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let token_id = find_in_zone(&state, "Proven Combatant", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();

    // Token must be Black only (original Blue replaced).
    assert!(
        token_obj.characteristics.colors.contains(&Color::Black),
        "CR 702.129a: eternalize token must be Black"
    );
    assert_eq!(
        token_obj.characteristics.colors.len(),
        1,
        "CR 702.129a / CR 707.9b: eternalize token should have exactly one color (Black)"
    );

    // Token must not retain the original blue color.
    assert!(
        !token_obj.characteristics.colors.contains(&Color::Blue),
        "CR 702.129a: eternalize token must not be Blue (original color replaced)"
    );

    // Token P/T must be 4/4 (not the original 1/1).
    assert_eq!(
        token_obj.characteristics.power,
        Some(4),
        "CR 702.129a / CR 707.9b: eternalize token power must be 4"
    );
    assert_eq!(
        token_obj.characteristics.toughness,
        Some(4),
        "CR 702.129a / CR 707.9b: eternalize token toughness must be 4"
    );
}

// ── Test 3: Card exiled as cost (before ability resolves) ─────────────────────

#[test]
/// CR 702.129a, ruling 2017-07-14 — The card is exiled immediately as part of
/// the activation cost, before the ability goes on the stack. Unlike Unearth
/// where the card stays in the graveyard until resolution, Eternalize's exile is
/// the cost payment itself.
fn test_eternalize_card_exiled_as_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(combatant_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Proven Combatant");

    // Activate eternalize.
    let (state, activate_events) = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EternalizeCard should succeed");

    // BEFORE resolution (ability is on the stack), card is already in exile.
    assert!(
        !state.stack_objects.is_empty(),
        "EternalizeAbility should be on the stack before resolution"
    );
    assert!(
        !in_graveyard(&state, "Proven Combatant", p1),
        "CR 702.129a: card is NOT in graveyard after activation (exiled as cost)"
    );
    assert!(
        in_exile(&state, "Proven Combatant"),
        "CR 702.129a: card IS in exile immediately after activation (exiled as cost)"
    );

    // ObjectExiled event was emitted during activation.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::ObjectExiled { player, .. } if *player == p1)),
        "CR 702.129a: ObjectExiled event should fire during activation (cost payment)"
    );

    // No token yet — need to resolve the ability first.
    assert!(
        !on_battlefield(&state, "Proven Combatant"),
        "CR 702.129a: no token on battlefield until ability resolves"
    );
}

// ── Test 4: Sorcery speed restriction ─────────────────────────────────────────

#[test]
/// CR 702.129a — "Activate only as a sorcery": eternalize can only be activated
/// during the active player's own main phase with an empty stack.
fn test_eternalize_sorcery_speed() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    // Test 1: Cannot activate during opponent's turn (active player is p2).
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(combatant_in_graveyard(p1))
            .active_player(p2) // p2 is active
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Blue, 4);
        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Blue, 2);
        state.turn.priority_holder = Some(p1);

        let card_obj_id = find_object(&state, "Proven Combatant");

        let result = process_command(
            state,
            Command::EternalizeCard {
                player: p1,
                card: card_obj_id,
            },
        );
        assert!(
            result.is_err(),
            "CR 702.129a: eternalize cannot be activated during opponent's turn"
        );
    }

    // Test 2: Cannot activate during combat (DeclareAttackers step).
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(combatant_in_graveyard(p1))
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .build()
            .unwrap();

        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Blue, 4);
        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Blue, 2);
        state.turn.priority_holder = Some(p1);

        let card_obj_id = find_object(&state, "Proven Combatant");

        let result = process_command(
            state,
            Command::EternalizeCard {
                player: p1,
                card: card_obj_id,
            },
        );
        assert!(
            result.is_err(),
            "CR 702.129a: eternalize cannot be activated during combat"
        );
    }
}

// ── Test 5: Not in graveyard ──────────────────────────────────────────────────

#[test]
/// CR 702.129a — Eternalize can only be activated when the card is in the player's
/// own graveyard. Attempting to activate when the card is in exile or hand fails.
fn test_eternalize_not_in_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    // Card is in exile, not graveyard.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Proven Combatant")
                .in_zone(ZoneId::Exile) // Wrong zone
                .with_card_id(CardId("proven-combatant".to_string()))
                .with_keyword(KeywordAbility::Eternalize),
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
        .add(ManaColor::Blue, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Proven Combatant");

    let result = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.129a: eternalize cannot be activated when card is not in graveyard"
    );
}

// ── Test 6: Insufficient mana ─────────────────────────────────────────────────

#[test]
/// CR 602.2b — Eternalize requires paying the eternalize cost. Attempting to activate
/// without sufficient mana returns an error.
fn test_eternalize_insufficient_mana() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(combatant_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only 1 blue — can't pay {4}{U}{U}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Proven Combatant");

    let result = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: eternalize without sufficient mana should return an error"
    );
}

// ── Test 7: Token retains printed keywords (Haste) ───────────────────────────

#[test]
/// CR 707.2 (copiable values), ruling 2017-07-14 — The token copies the printed
/// keyword abilities of the original card. A Haste Warrior eternalize token must
/// also have Haste.
fn test_eternalize_token_retains_printed_keywords() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![haste_warrior_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Haste Warrior")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("haste-warrior".to_string()))
                .with_keyword(KeywordAbility::Haste)
                .with_keyword(KeywordAbility::Eternalize),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 4 colorless + 2 red mana to pay the {4}{R}{R} eternalize cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Haste Warrior");

    let (state, _) = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EternalizeCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let token_id = find_in_zone(&state, "Haste Warrior", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();

    // Token must have Haste (copied from printed abilities).
    assert!(
        token_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Haste),
        "CR 707.2: eternalize token should retain Haste from the original card's printed abilities"
    );

    // Token must be a 4/4 Black Zombie Warrior.
    assert_eq!(
        token_obj.characteristics.power,
        Some(4),
        "CR 702.129a: token must be 4/4 (not original 2/2)"
    );
    assert!(
        token_obj.characteristics.colors.contains(&Color::Black),
        "CR 702.129a: token must be Black"
    );
}

// ── Test 8: Eternalize is not a cast ──────────────────────────────────────────

#[test]
/// Ruling 2017-07-14 — Eternalize is an activated ability, not a spell cast.
/// No SpellCast event fires, and spells_cast_this_turn is unchanged.
fn test_eternalize_not_a_cast() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(combatant_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let initial_spells_cast = state.players.get(&p1).unwrap().spells_cast_this_turn;

    let card_obj_id = find_object(&state, "Proven Combatant");

    let (state, activate_events) = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EternalizeCard should succeed");

    // No SpellCast event.
    assert!(
        !activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "Eternalize is NOT a cast: no SpellCast event should fire"
    );

    // spells_cast_this_turn is unchanged.
    let after_spells_cast = state.players.get(&p1).unwrap().spells_cast_this_turn;
    assert_eq!(
        initial_spells_cast, after_spells_cast,
        "Eternalize is NOT a cast: spells_cast_this_turn should not change"
    );
}

// ── Test 9: Token is a Zombie in addition to its other types ──────────────────

#[test]
/// CR 702.129a — "Zombie in addition to its other types": token has the Zombie
/// subtype added to its existing subtypes without removing them.
fn test_eternalize_token_zombie_subtype() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(combatant_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Proven Combatant");

    let (state, _) = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EternalizeCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let token_id = find_in_zone(&state, "Proven Combatant", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();

    // Token must have Zombie subtype.
    assert!(
        token_obj
            .characteristics
            .subtypes
            .contains(&SubType("Zombie".to_string())),
        "CR 702.129a: eternalize token must have the Zombie subtype"
    );

    // Token must also retain its original Human and Warrior subtypes.
    assert!(
        token_obj
            .characteristics
            .subtypes
            .contains(&SubType("Human".to_string())),
        "CR 702.129a: eternalize token must retain its original Human subtype"
    );
    assert!(
        token_obj
            .characteristics
            .subtypes
            .contains(&SubType("Warrior".to_string())),
        "CR 702.129a: eternalize token must retain its original Warrior subtype"
    );

    // Token must be a Creature card type.
    assert!(
        token_obj
            .characteristics
            .card_types
            .contains(&CardType::Creature),
        "CR 702.129a: eternalize token must be a Creature"
    );
}

// ── Test 10: Token has no mana cost ──────────────────────────────────────────

#[test]
/// CR 702.129a — "it has no mana cost": token's mana cost is None (mana value 0).
/// CR 707.9d: the CDA that might define color from mana cost is not copied.
fn test_eternalize_no_mana_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(combatant_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Proven Combatant");

    let (state, _) = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EternalizeCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let token_id = find_in_zone(&state, "Proven Combatant", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();

    // Token must have no mana cost.
    assert!(
        token_obj.characteristics.mana_cost.is_none(),
        "CR 702.129a / CR 707.9d: eternalize token must have no mana cost (mana value 0)"
    );
}

// ── Test 11: Non-empty stack blocks eternalize ────────────────────────────────

#[test]
/// CR 702.129a — "Activate only as a sorcery" requires the stack to be empty.
/// Also, CR 702.61a: split second prevents activated abilities while on the stack.
/// Here we verify the empty-stack requirement by manually pushing an item onto the stack.
fn test_eternalize_split_second_blocks() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(combatant_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    // Push a dummy UnearthAbility onto the stack to simulate a non-empty stack
    // (as would happen with a split-second spell or any other stack item).
    let fake_object_id = ObjectId(9999);
    let fake_stack_id = state.next_object_id();
    state.stack_objects.push_back(StackObject {
        id: fake_stack_id,
        controller: p2,
        kind: StackObjectKind::UnearthAbility {
            source_object: fake_object_id,
        },
        targets: Vec::new(),
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: test objects are not cleave casts.
        was_cleaved: false,
        // CR 715.3d: test objects are not adventure casts.
        was_cast_as_adventure: false,
        // CR 702.47a: test objects have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
        triggering_creature_id: None,
        cast_from_top_with_bonus: false,
        sacrificed_creature_powers: vec![],
    });

    let card_obj_id = find_object(&state, "Proven Combatant");

    let result = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    );

    // Stack is non-empty — sorcery-speed check (empty stack required) must prevent eternalize.
    assert!(
        result.is_err(),
        "CR 702.129a / CR 702.61a: eternalize cannot be activated with non-empty stack"
    );
}

// ── Test 12: Token keyword retained — Eternalize keyword present on token ─────

#[test]
/// CR 702.129a, ruling 2017-07-14 — The token copies the printed abilities
/// of the original card, including the Eternalize keyword marker itself.
/// This verifies that keywords are correctly propagated to the token.
fn test_eternalize_keyword_retained() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![proven_combatant_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(combatant_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let card_obj_id = find_object(&state, "Proven Combatant");

    let (state, _) = process_command(
        state,
        Command::EternalizeCard {
            player: p1,
            card: card_obj_id,
        },
    )
    .expect("EternalizeCard should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    let token_id = find_in_zone(&state, "Proven Combatant", ZoneId::Battlefield)
        .expect("token should be on battlefield");
    let token_obj = state.objects.get(&token_id).unwrap();

    // Token must have Eternalize keyword (copied from printed abilities, CR 707.2).
    assert!(
        token_obj
            .characteristics
            .keywords
            .contains(&KeywordAbility::Eternalize),
        "CR 707.2: eternalize token should retain the Eternalize keyword from the printed card"
    );

    // Token should also have summoning sickness (CR 302.6).
    assert!(
        token_obj.has_summoning_sickness,
        "CR 302.6: eternalize token should have summoning sickness when it enters the battlefield"
    );
}

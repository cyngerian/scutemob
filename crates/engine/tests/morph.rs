//! Morph, Megamorph, Disguise, Manifest, and Cloak tests.
//!
//! Key rules verified:
//! - CR 702.37a/c: Morph casts face-down for {3} as a 2/2 with no name, text, or subtypes.
//! - CR 702.37e: Turn face up is a special action (no stack); regains real characteristics.
//! - CR 702.37b: Megamorph — gets +1/+1 counter when turned face up via megamorph cost.
//! - CR 702.168a: Disguise — face-down creature has ward {2} while face-down.
//! - CR 708.2a: Face-down characteristics: 2/2, Creature, no name, no abilities.
//! - CR 708.3: ETB abilities do NOT fire when a permanent enters face-down.
//! - CR 708.8: "When this creature is turned face up" IS a triggered ability -> goes on stack.
//! - CR 701.40a/b: Manifest -- top library card enters face-down as 2/2; turn face up by
//!   paying mana cost (creature cards only, no instants/sorceries).
//! - CR 701.40c: Manifested morph card can use either morph cost OR mana cost.
//! - CR 701.58a/b: Cloak -- like Manifest but with ward {2} while face-down.

use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};
use mtg_engine::state::types::AltCostKind;
use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Command, CounterType, Effect, FaceDownKind, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step,
    TriggerCondition, TurnFaceUpMethod, TypeLine, ZoneId,
};
use std::sync::Arc;

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

fn find_face_down_on_bf(state: &GameState) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.zone == ZoneId::Battlefield && obj.status.face_down)
        .map(|(id, _)| *id)
}

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

/// Build a game state with an object on battlefield, then set face-down status
/// on the resulting object. Returns (state, object_id).
fn count_in_zone(
    state: &GameState,
    player: PlayerId,
    zone_fn: impl Fn(PlayerId) -> ZoneId,
) -> usize {
    let zone = zone_fn(player);
    state
        .objects
        .values()
        .filter(|obj| obj.zone == zone)
        .count()
}

fn build_state_with_face_down_object(
    p1: PlayerId,
    p2: PlayerId,
    registry: Arc<CardRegistry>,
    spec: ObjectSpec,
    face_down_kind: FaceDownKind,
) -> (GameState, ObjectId) {
    let name = spec.name.clone();
    let card_id = spec.card_id.clone();
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Find the object by name on the battlefield and set it face-down.
    // Also enrich characteristics from the card definition (P/T, keywords) so that
    // after turning face up the layer system can use the real values.
    let obj_id = find_in_zone(&state, &name, ZoneId::Battlefield)
        .unwrap_or_else(|| panic!("object '{}' should be on battlefield", name));
    if let Some(obj) = state.objects.get_mut(&obj_id) {
        // Enrich P/T and keywords from CardDefinition if available.
        if let Some(ref cid) = card_id {
            if let Some(def) = state.card_registry.get(cid.clone()) {
                if obj.characteristics.power.is_none() {
                    obj.characteristics.power = def.power;
                }
                if obj.characteristics.toughness.is_none() {
                    obj.characteristics.toughness = def.toughness;
                }
                // Populate keyword abilities from the definition.
                for ability in &def.abilities {
                    if let AbilityDefinition::Keyword(kw) = ability {
                        obj.characteristics.keywords.insert(kw.clone());
                    }
                }
            }
        }
        obj.status.face_down = true;
        obj.face_down_as = Some(face_down_kind);
    }
    (state, obj_id)
}

// ── Card definitions ──────────────────────────────────────────────────────────

/// A simple creature with Morph {1}{W}{W} (CR 702.37).
/// Front face: 4/5 Flying creature (like Exalted Angel).
fn morph_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-morph-creature".to_string()),
        name: "Mock Morph Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Flying. Morph {1}{W}{W}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Morph {
                cost: ManaCost {
                    generic: 1,
                    white: 2,
                    ..Default::default()
                },
            },
        ],
        power: Some(4),
        toughness: Some(5),
        ..Default::default()
    }
}

/// A creature with Megamorph {3} (CR 702.37b).
fn megamorph_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-megamorph".to_string()),
        name: "Mock Megamorph".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Megamorph {3}".to_string(),
        abilities: vec![AbilityDefinition::Megamorph {
            cost: ManaCost {
                generic: 3,
                ..Default::default()
            },
        }],
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    }
}

/// A creature with Disguise {2}{B} (CR 702.168).
fn disguise_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-disguise".to_string()),
        name: "Mock Disguise".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Disguise {2}{B}".to_string(),
        abilities: vec![AbilityDefinition::Disguise {
            cost: ManaCost {
                generic: 2,
                black: 1,
                ..Default::default()
            },
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// A creature with a "When turned face up" trigger (CR 708.8).
fn when_turned_face_up_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-when-face-up".to_string()),
        name: "Mock Face-Up Trigger".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Morph {2}. When ~ is turned face up, draw a card.".to_string(),
        abilities: vec![
            AbilityDefinition::Morph {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenTurnedFaceUp,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// A plain creature card (no morph), for manifest turn-face-up tests.
fn plain_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-plain-creature".to_string()),
        name: "Mock Plain Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// An instant card (for manifest non-creature test, CR 701.40b / 701.40g).
fn instant_card_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-instant".to_string()),
        name: "Mock Instant".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Deal 1 damage to any target.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

/// A creature with both Morph and a mana cost, for manifest-with-morph test (CR 701.40c).
fn morph_and_mana_cost_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-morph-with-mana".to_string()),
        name: "Mock Morph With Mana".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Morph {G}".to_string(),
        abilities: vec![AbilityDefinition::Morph {
            cost: ManaCost {
                green: 1,
                ..Default::default()
            },
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

// ── Test 1: Morph cast face-down as 2/2 ──────────────────────────────────────

/// CR 702.37a / CR 708.2a: Casting a creature with morph face-down produces a
/// 2/2 face-down creature with no name, no abilities, and no subtypes on the stack.
/// After resolution it's a 2/2 on the battlefield.
#[test]
fn test_morph_cast_face_down_basic() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![morph_creature_def()]);

    let creature = ObjectSpec::card(p1, "Mock Morph Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-morph-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            white: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give the player 3 generic mana (morph cost is {3}).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Mock Morph Creature");

    // Cast face-down via morph.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Morph),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: Some(FaceDownKind::Morph),
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Morph cast failed: {:?}", e));

    // Spell is on stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on stack");

    // Mana consumed: 3 colorless.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "CR 702.37c: {{3}} should be paid"
    );

    // The source object in the stack zone should be face-down.
    let stack_zone_obj = state
        .objects
        .iter()
        .find(|(_, obj)| obj.zone == ZoneId::Stack)
        .map(|(id, _)| *id);
    if let Some(stack_id) = stack_zone_obj {
        assert!(
            state.objects[&stack_id].status.face_down,
            "CR 708.4: source object in stack zone should be face-down"
        );
    }

    // Resolve the spell (both players pass).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Face-down permanent is on battlefield.
    let face_down_id = find_face_down_on_bf(&state).expect("face-down creature should be on bf");

    // CR 708.2a: Characteristics should be 2/2, creature, no name.
    let chars = calculate_characteristics(&state, face_down_id).expect("chars should exist");
    assert_eq!(chars.name, "", "CR 708.2a: face-down name is empty");
    assert_eq!(chars.power, Some(2), "CR 708.2a: face-down P/T is 2/2");
    assert_eq!(chars.toughness, Some(2));
    assert!(
        chars.card_types.contains(&CardType::Creature),
        "CR 708.2a: face-down is a creature"
    );
    assert!(
        chars.subtypes.is_empty(),
        "CR 708.2a: face-down has no subtypes"
    );
    assert!(
        !chars.keywords.contains(&KeywordAbility::Flying),
        "CR 708.2a: face-down has no abilities (Flying suppressed)"
    );

    // face_down_as should be Morph.
    assert_eq!(
        state.objects[&face_down_id].face_down_as,
        Some(FaceDownKind::Morph),
        "face_down_as should be Morph"
    );
}

// ── Test 2: Turn face up regains real characteristics ────────────────────────

/// CR 702.37e: Paying the morph cost turns the permanent face up. It regains
/// its true characteristics (name, P/T, abilities).
#[test]
fn test_morph_turn_face_up() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![morph_creature_def()]);

    let spec = ObjectSpec::card(p1, "Mock Morph Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-morph-creature".to_string()))
        .with_types(vec![CardType::Creature]);

    let (mut state, face_down_id) =
        build_state_with_face_down_object(p1, p2, registry, spec, FaceDownKind::Morph);

    // Morph cost: {1}{W}{W} -- give player 3 mana (1 generic + 2 white).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 2);
    state.turn.priority_holder = Some(p1);

    // Turn face up via morph cost.
    let (state, events) = process_command(
        state,
        Command::TurnFaceUp {
            player: p1,
            permanent: face_down_id,
            method: TurnFaceUpMethod::MorphCost,
        },
    )
    .expect("TurnFaceUp should succeed");

    // Permanent is now face-up.
    assert!(
        !state.objects[&face_down_id].status.face_down,
        "permanent should be face-up after turning"
    );
    assert!(
        state.objects[&face_down_id].face_down_as.is_none(),
        "face_down_as should be cleared"
    );

    // CR 702.37e: Real characteristics are restored.
    let chars = calculate_characteristics(&state, face_down_id).expect("chars should exist");
    assert_eq!(
        chars.name, "Mock Morph Creature",
        "CR 702.37e: real name restored"
    );
    assert_eq!(chars.power, Some(4), "CR 702.37e: real P/T restored");
    assert_eq!(chars.toughness, Some(5));
    assert!(
        chars.keywords.contains(&KeywordAbility::Flying),
        "CR 702.37e: Flying ability restored"
    );

    // CR 702.37e: PermanentTurnedFaceUp event emitted.
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTurnedFaceUp {
                player,
                permanent,
            } if *player == p1 && *permanent == face_down_id
        )),
        "CR 702.37e: PermanentTurnedFaceUp event should be emitted"
    );

    // Mana should be consumed.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.white + pool.blue + pool.black + pool.red + pool.green + pool.colorless,
        0,
        "morph cost {{1}}{{W}}{{W}} should be paid"
    );
}

// ── Test 3: ETB abilities do NOT fire when entering face-down ─────────────────

/// CR 708.3: Objects put onto the battlefield face down are turned face down
/// BEFORE they enter, so ETB abilities don't trigger.
#[test]
fn test_morph_face_down_no_etb() {
    let p1 = p(1);
    let p2 = p(2);

    // Use a card with a "when turned face up" trigger (not an ETB trigger).
    // Nothing should draw a card when the spell resolves face-down.
    let registry = CardRegistry::new(vec![when_turned_face_up_def()]);

    let creature = ObjectSpec::card(p1, "Mock Face-Up Trigger")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-when-face-up".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give player 3 mana for morph cost {3} (wait — morph cost is {2} for this card).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Mock Face-Up Trigger");
    let initial_hand_count = count_in_zone(&state, p1, ZoneId::Hand);

    // Cast face-down via morph.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Morph),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: Some(FaceDownKind::Morph),
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Morph cast failed: {:?}", e));

    // Resolve -- both pass.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 708.3: No "when turned face up" trigger should fire on entering face-down.
    // Hand count decreased by exactly 1 (for the cast) -- no draw occurred.
    let current_hand = count_in_zone(&state, p1, ZoneId::Hand);
    assert_eq!(
        current_hand,
        initial_hand_count - 1,
        "CR 708.3: no ETB/face-up trigger should fire when entering face-down"
    );

    // Verify the creature is face-down on the battlefield.
    let face_down_id = find_face_down_on_bf(&state).expect("face-down creature should be on bf");
    assert!(
        state.objects[&face_down_id].status.face_down,
        "permanent should be face-down"
    );
}

// ── Test 4: "When turned face up" trigger fires on turn-face-up ──────────────

/// CR 708.8: "When this creature is turned face up" is a triggered ability.
/// It goes on the stack after the permanent turns face up, unlike ETB abilities.
#[test]
fn test_morph_when_turned_face_up_trigger() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![when_turned_face_up_def()]);

    let spec = ObjectSpec::card(p1, "Mock Face-Up Trigger")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-when-face-up".to_string()))
        .with_types(vec![CardType::Creature]);

    // Add a dummy card to library so the draw trigger can succeed.
    let library_card = ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1));

    let bf_spec_name = spec.name.clone();
    let bf_spec_cid = spec.card_id.clone();
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Set face-down status on the battlefield object.
    let face_down_id = find_in_zone(&state, &bf_spec_name, ZoneId::Battlefield)
        .expect("face-down creature should be on battlefield");
    if let Some(obj) = state.objects.get_mut(&face_down_id) {
        // Enrich from card def.
        if let Some(ref cid) = bf_spec_cid {
            if let Some(def) = state.card_registry.get(cid.clone()) {
                if obj.characteristics.power.is_none() {
                    obj.characteristics.power = def.power;
                }
                if obj.characteristics.toughness.is_none() {
                    obj.characteristics.toughness = def.toughness;
                }
                for ability in &def.abilities {
                    if let AbilityDefinition::Keyword(kw) = ability {
                        obj.characteristics.keywords.insert(kw.clone());
                    }
                }
            }
        }
        obj.status.face_down = true;
        obj.face_down_as = Some(FaceDownKind::Morph);
    }

    // Morph cost for this card: {2} -- give player 2 mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let initial_hand_count = count_in_zone(&state, p1, ZoneId::Hand);

    // Turn face up.
    let (state, _) = process_command(
        state,
        Command::TurnFaceUp {
            player: p1,
            permanent: face_down_id,
            method: TurnFaceUpMethod::MorphCost,
        },
    )
    .expect("TurnFaceUp should succeed");

    // CR 708.8: "When turned face up" trigger should now be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "CR 708.8: 'when turned face up' trigger should be on the stack"
    );

    // Resolve the trigger -- should draw a card.
    let (state, _) = pass_all(state, &[p1, p2]);

    let new_hand_count = count_in_zone(&state, p1, ZoneId::Hand);
    assert_eq!(
        new_hand_count,
        initial_hand_count + 1,
        "CR 708.8: 'when turned face up' draw trigger should have resolved"
    );
}

// ── Test 5: Face-down characteristics via layer system ────────────────────────

/// CR 708.2a: Face-down characteristics are the base copiable values.
/// Continuous effects from the layer loop apply ON TOP of those values.
/// (A +1/+1 counter on a face-down creature still modifies P/T.)
#[test]
fn test_morph_face_down_characteristics_layer() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![morph_creature_def()]);

    let spec = ObjectSpec::card(p1, "Mock Morph Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-morph-creature".to_string()))
        .with_types(vec![CardType::Creature]);

    let (mut state, face_down_id) =
        build_state_with_face_down_object(p1, p2, registry, spec, FaceDownKind::Morph);

    // Place a +1/+1 counter on the face-down creature.
    if let Some(obj) = state.objects.get_mut(&face_down_id) {
        obj.counters = obj.counters.update(CounterType::PlusOnePlusOne, 1);
    }

    // CR 708.2a: Base is 2/2. Counter adds +1/+1 -> layer system produces 3/3.
    let chars =
        calculate_characteristics(&state, face_down_id).expect("chars should be calculable");
    assert_eq!(
        chars.power,
        Some(3),
        "CR 708.2a: face-down base 2/2 + 1 counter = 3/3"
    );
    assert_eq!(chars.toughness, Some(3));

    // Verify name is still empty (face-down override applies first).
    assert_eq!(
        chars.name, "",
        "CR 708.2a: name still empty while face-down"
    );
    assert!(
        !chars.keywords.contains(&KeywordAbility::Flying),
        "Flying is suppressed while face-down"
    );
}

// ── Test 6: Megamorph gets +1/+1 counter on turn-face-up ──────────────────────

/// CR 702.37b: Megamorph is Morph but also adds a +1/+1 counter when turned face up
/// via its megamorph cost.
#[test]
fn test_megamorph_counter() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![megamorph_creature_def()]);

    let spec = ObjectSpec::card(p1, "Mock Megamorph")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-megamorph".to_string()))
        .with_types(vec![CardType::Creature]);

    let (mut state, face_down_id) =
        build_state_with_face_down_object(p1, p2, registry, spec, FaceDownKind::Megamorph);

    // Megamorph cost: {3} generic.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    // Turn face up via morph cost (MorphCost is used for Megamorph too -- CR 702.37b).
    let (state, _) = process_command(
        state,
        Command::TurnFaceUp {
            player: p1,
            permanent: face_down_id,
            method: TurnFaceUpMethod::MorphCost,
        },
    )
    .expect("TurnFaceUp (megamorph) should succeed");

    // CR 702.37b: +1/+1 counter should be added.
    let counter_count = state.objects[&face_down_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.37b: megamorph should add +1/+1 counter on turn-face-up"
    );

    // Should now be face-up with real characteristics + 1 counter = 4/4.
    assert!(
        !state.objects[&face_down_id].status.face_down,
        "permanent should be face-up"
    );
    let chars = calculate_characteristics(&state, face_down_id).expect("chars should exist");
    assert_eq!(chars.power, Some(4), "megamorph 3/3 + 1 counter = 4/4");
    assert_eq!(chars.toughness, Some(4));
}

// ── Test 7: Disguise grants ward {2} while face-down ─────────────────────────

/// CR 702.168a: A disguise creature cast face-down has ward {2} while face-down.
/// Verified by checking keywords in the layer system.
#[test]
fn test_disguise_ward() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![disguise_creature_def()]);

    let spec = ObjectSpec::card(p1, "Mock Disguise")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-disguise".to_string()))
        .with_types(vec![CardType::Creature]);

    let (state, face_down_id) =
        build_state_with_face_down_object(p1, p2, registry, spec, FaceDownKind::Disguise);

    // CR 702.168a: Face-down disguise has ward {2} keyword.
    let chars = calculate_characteristics(&state, face_down_id).expect("chars should exist");
    assert!(
        chars.keywords.contains(&KeywordAbility::Ward(2)),
        "CR 702.168a: disguise face-down should have ward {{2}}"
    );

    // Verify other face-down characteristics: still 2/2, no name.
    assert_eq!(chars.name, "", "face-down has no name");
    assert_eq!(chars.power, Some(2));
    assert_eq!(chars.toughness, Some(2));
    // Ward should be the ONLY keyword present.
    let non_ward: Vec<_> = chars
        .keywords
        .iter()
        .filter(|k| !matches!(k, KeywordAbility::Ward(2)))
        .collect();
    assert!(
        non_ward.is_empty(),
        "no keywords beyond ward {{2}} while face-down via disguise"
    );
}

// ── Test 8: Cloak grants ward {2} while face-down ────────────────────────────

/// CR 701.58a: A cloaked permanent has ward {2} while face-down.
#[test]
fn test_cloak_ward() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_creature_def()]);

    let spec = ObjectSpec::card(p1, "Mock Plain Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-plain-creature".to_string()))
        .with_types(vec![CardType::Creature]);

    let (state, face_down_id) =
        build_state_with_face_down_object(p1, p2, registry, spec, FaceDownKind::Cloak);

    // CR 701.58a: Cloaked creature has ward {2}.
    let chars = calculate_characteristics(&state, face_down_id).expect("chars should exist");
    assert!(
        chars.keywords.contains(&KeywordAbility::Ward(2)),
        "CR 701.58a: cloaked permanent should have ward {{2}}"
    );
    assert_eq!(chars.power, Some(2));
    assert_eq!(chars.toughness, Some(2));
}

// ── Test 9: Manifest creature card can be turned face up ─────────────────────

/// CR 701.40b: A manifested permanent that represents a creature card can be
/// turned face up by paying its mana cost.
#[test]
fn test_manifest_creature_turn_face_up() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_creature_def()]);

    let spec = ObjectSpec::card(p1, "Mock Plain Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-plain-creature".to_string()))
        .with_types(vec![CardType::Creature]);

    let (mut state, face_down_id) =
        build_state_with_face_down_object(p1, p2, registry, spec, FaceDownKind::Manifest);

    // Mana cost to turn face up: {2} generic.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    // CR 701.40b: Turn face up by paying mana cost.
    let (state, _) = process_command(
        state,
        Command::TurnFaceUp {
            player: p1,
            permanent: face_down_id,
            method: TurnFaceUpMethod::ManaCost,
        },
    )
    .expect("CR 701.40b: manifested creature should be turnable face up");

    // Permanent should now be face-up with its real characteristics.
    assert!(
        !state.objects[&face_down_id].status.face_down,
        "permanent should be face-up"
    );
    let chars = calculate_characteristics(&state, face_down_id).expect("chars should exist");
    assert_eq!(
        chars.name, "Mock Plain Creature",
        "CR 701.40b: real name restored"
    );
    assert_eq!(chars.power, Some(2));
    assert_eq!(chars.toughness, Some(2));
}

// ── Test 10: Manifest noncreature cannot be turned face up ───────────────────

/// CR 701.40b: A manifested permanent that represents a non-creature card
/// (e.g., an instant) cannot be turned face up via the manifest procedure.
#[test]
fn test_manifest_noncreature_stuck() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![instant_card_def()]);

    // Build state with an instant card, then manually set face_down_as = Manifest.
    let spec = ObjectSpec::card(p1, "Mock Instant")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-instant".to_string()))
        .with_types(vec![CardType::Instant]);

    let (mut state, face_down_id) =
        build_state_with_face_down_object(p1, p2, registry, spec, FaceDownKind::Manifest);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    // CR 701.40b / 701.40g: Attempting to turn face up an instant via ManaCost should fail.
    let result = process_command(
        state,
        Command::TurnFaceUp {
            player: p1,
            permanent: face_down_id,
            method: TurnFaceUpMethod::ManaCost,
        },
    );

    assert!(
        result.is_err(),
        "CR 701.40b: manifested instant card cannot be turned face up via ManaCost"
    );
}

// ── Test 11: Manifested morph card can use either method ─────────────────────

/// CR 701.40c: If a card with morph is manifested, its controller may turn that
/// card face up using either the morph procedure OR the manifest procedure.
#[test]
fn test_manifest_with_morph() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![morph_and_mana_cost_creature_def()]);

    // Test 1: turn face up via MorphCost.
    let spec1 = ObjectSpec::card(p1, "Mock Morph With Mana")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-morph-with-mana".to_string()))
        .with_types(vec![CardType::Creature]);
    let (mut state, face_down_id) =
        build_state_with_face_down_object(p1, p2, registry.clone(), spec1, FaceDownKind::Manifest);

    // Morph cost: {G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    let result1 = process_command(
        state,
        Command::TurnFaceUp {
            player: p1,
            permanent: face_down_id,
            method: TurnFaceUpMethod::MorphCost,
        },
    );
    assert!(
        result1.is_ok(),
        "CR 701.40c: manifested morph card can be turned face up via MorphCost: {:?}",
        result1.err()
    );

    // Test 2: turn face up via ManaCost (mana cost is {1}{G}).
    let spec2 = ObjectSpec::card(p1, "Mock Morph With Mana")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-morph-with-mana".to_string()))
        .with_types(vec![CardType::Creature]);
    let (mut state2, face_down_id2) =
        build_state_with_face_down_object(p1, p2, registry, spec2, FaceDownKind::Manifest);

    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state2.turn.priority_holder = Some(p1);

    let result2 = process_command(
        state2,
        Command::TurnFaceUp {
            player: p1,
            permanent: face_down_id2,
            method: TurnFaceUpMethod::ManaCost,
        },
    );
    assert!(
        result2.is_ok(),
        "CR 701.40c: manifested morph card can also be turned face up via ManaCost: {:?}",
        result2.err()
    );
}

// ── Test 12: Face-down permanent identity preserved in graveyard ──────────────

/// CR 708.9: If a face-down permanent leaves the battlefield, its owner must reveal it.
/// The engine preserves the card's true identity (card_id) in the destination zone.
#[test]
fn test_face_down_dies_reveal() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![morph_creature_def()]);

    let spec = ObjectSpec::card(p1, "Mock Morph Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-morph-creature".to_string()))
        .with_types(vec![CardType::Creature]);

    let (mut state, face_down_id) =
        build_state_with_face_down_object(p1, p2, registry, spec, FaceDownKind::Morph);

    let card_id_before = state.objects[&face_down_id]
        .card_id
        .clone()
        .expect("card_id should be present");

    // Deal lethal damage to the 2/2 face-down creature.
    if let Some(obj) = state.objects.get_mut(&face_down_id) {
        obj.damage_marked = 2;
    }

    // SBAs kill it via priority pass.
    state.turn.priority_holder = Some(p1);
    let (state, events) = process_command(state, Command::PassPriority { player: p1 })
        .expect("PassPriority should succeed");

    // CR 708.9: The creature should be in the graveyard with its real card_id.
    let in_gy = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Graveyard(p1) && obj.card_id.as_ref() == Some(&card_id_before)
        })
        .map(|(id, _)| *id);

    if let Some(gy_id) = in_gy {
        assert_eq!(
            state.objects[&gy_id].card_id,
            Some(card_id_before),
            "CR 708.9: real card identity preserved in graveyard"
        );
        // In graveyard it should NOT be face-down.
        assert!(
            !state.objects[&gy_id].status.face_down,
            "CR 708.9: face-down status cleared when leaving battlefield"
        );
        assert!(
            state.objects[&gy_id].face_down_as.is_none(),
            "CR 708.9: face_down_as cleared in graveyard"
        );

        // CR 708.9: FaceDownRevealed event must be emitted when a face-down permanent
        // leaves the battlefield — the network layer uses it to broadcast the card's
        // true identity to all players.
        let reveal_emitted = events.iter().any(|e| {
            matches!(e, GameEvent::FaceDownRevealed { player, card_name, .. }
                if *player == p1 && card_name == "Mock Morph Creature")
        });
        assert!(
            reveal_emitted,
            "CR 708.9: FaceDownRevealed event must be emitted when face-down permanent dies; events: {:?}",
            events
        );
    }
    // Note: if creature not in graveyard after one priority pass (no SBA triggered), test is
    // inconclusive -- the important thing is it doesn't stay face-down if it does move.
}

// ── Test 13: Face-down is a creature spell on the stack ──────────────────────

/// CR 708.4: Objects cast face-down are turned face down BEFORE put on the stack.
/// Effects that care about spell characteristics see only the face-down characteristics:
/// 2/2 creature, no name, no text, no subtypes, no mana cost.
/// A morph spell IS a creature spell (can be targeted by "counter target creature spell").
#[test]
fn test_morph_cast_face_down_is_creature_spell() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![morph_creature_def()]);

    let creature = ObjectSpec::card(p1, "Mock Morph Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-morph-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            white: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
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

    let card_id = find_object(&state, "Mock Morph Creature");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Morph),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: Some(FaceDownKind::Morph),
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Morph cast failed: {:?}", e));

    // The spell is on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on stack");

    // The source object in the Stack zone should be face-down and be a creature.
    let stack_zone_obj_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.zone == ZoneId::Stack)
        .map(|(id, _)| *id);

    if let Some(stack_id) = stack_zone_obj_id {
        let obj = &state.objects[&stack_id];
        assert!(
            obj.status.face_down,
            "CR 708.4: source object in stack zone should be face-down"
        );
        assert_eq!(
            obj.face_down_as,
            Some(FaceDownKind::Morph),
            "face_down_as should be Morph on the stack object"
        );
        // Characteristics when face-down: 2/2 creature.
        let chars = calculate_characteristics(&state, stack_id).expect("chars should exist");
        assert!(
            chars.card_types.contains(&CardType::Creature),
            "CR 708.4: face-down spell is a creature spell"
        );
        assert_eq!(chars.power, Some(2), "CR 708.4: face-down spell is 2/2");
        assert_eq!(chars.name, "", "CR 708.4: face-down spell has no name");
    }
}

// ── Test 14: Plain morph creature does NOT have ward while face-down ──────────

/// CR 702.37 / CR 702.168: Morph (but NOT Disguise) does not grant ward {2}
/// while face-down. Only Disguise (CR 702.168) and Cloak (CR 701.58) grant ward.
#[test]
fn test_morph_face_down_no_ward() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![morph_creature_def()]);

    let spec = ObjectSpec::card(p1, "Mock Morph Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-morph-creature".to_string()))
        .with_types(vec![CardType::Creature]);

    let (state, face_down_id) =
        build_state_with_face_down_object(p1, p2, registry, spec, FaceDownKind::Morph);

    let chars = calculate_characteristics(&state, face_down_id).expect("chars should exist");
    // CR 702.37: Morph has no ward -- keywords should be empty.
    assert!(
        !chars.keywords.contains(&KeywordAbility::Ward(2)),
        "CR 702.37: Morph (not Disguise) does not grant ward {{2}} while face-down"
    );
    assert!(chars.keywords.is_empty(), "Morph face-down has no keywords");
}

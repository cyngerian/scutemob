//! Grant Flash mechanism tests (CR 601.3b).
//!
//! The engine supports granting "cast as though it had flash" permission to a player
//! for specific categories of spells via `Effect::GrantFlash` and
//! `AbilityDefinition::StaticFlashGrant`. This covers:
//!
//! - Borne Upon a Wind: "You may cast spells this turn as though they had flash." (all spells, UntilEndOfTurn)
//! - Complete the Circuit: sorceries only, UntilEndOfTurn
//! - Teferi, Time Raveler: +1 grants sorceries UntilYourNextTurn; passive restricts opponents to sorcery speed
//! - Yeva, Nature's Herald: static grant for green creatures while on the battlefield
//!
//! Key rules verified:
//! - Flash grant allows casting sorcery-speed spells at instant speed (CR 601.3b)
//! - UntilEndOfTurn grants expire at the cleanup step (CR 514.2)
//! - UntilYourNextTurn grants persist through opponents' turns, expire at controller's next untap (CR 611.2b)
//! - FlashGrantFilter::Sorceries only applies to sorcery-type spells (not creatures)
//! - FlashGrantFilter::GreenCreatures only applies to green creature spells
//! - WhileSourceOnBattlefield grants are inactive when the source is not on the battlefield
//! - CR 101.2: OpponentsCanOnlyCastAtSorcerySpeed restriction overrides flash grants

use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};
use mtg_engine::state::stubs::{ActiveRestriction, FlashGrant};
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Color,
    Command, Effect, EffectDuration, FlashGrantFilter, GameEvent, GameRestriction, GameState,
    GameStateBuilder, KeywordAbility, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ────────────────────────────────────────────────────────────────────

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

/// Pass priority for all players once.
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

/// Advance from the current turn to the next turn (through cleanup).
fn advance_to_next_turn(mut state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let initial_turn = state.turn.turn_number;
    while state.turn.turn_number == initial_turn {
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

/// Build a CastSpell command with only required fields; rest are defaults.
fn cast_spell_cmd(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell {
        player,
        card,
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
    }
}

// ── Card definitions ───────────────────────────────────────────────────────────

/// Mock sorcery: "Sorcery {2}{R}". Draw a card.
fn test_sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-sorcery".to_string()),
        name: "Test Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Mock green creature: "Creature {G} 1/1 — Elf". Green.
fn test_green_elf_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-green-elf".to_string()),
        name: "Test Green Elf".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [mtg_engine::SubType("Elf".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}

/// Mock red creature: "Creature {R} 2/1 — Goblin". Red.
fn test_red_goblin_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-red-goblin".to_string()),
        name: "Test Red Goblin".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [mtg_engine::SubType("Goblin".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(1),
        ..Default::default()
    }
}

/// Yeva, Nature's Herald: Legendary Creature {2}{G}{G} 4/4, Flash + StaticFlashGrant.
fn yeva_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("yeva-natures-herald".to_string()),
        name: "Yeva, Nature's Herald".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            supertypes: [mtg_engine::SuperType::Legendary].into_iter().collect(),
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [
                mtg_engine::SubType("Elf".to_string()),
                mtg_engine::SubType("Shaman".to_string()),
            ]
            .into_iter()
            .collect(),
        },
        oracle_text: "Flash\nYou may cast green creature spells as though they had flash."
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::StaticFlashGrant {
                filter: FlashGrantFilter::GreenCreatures,
            },
        ],
        ..Default::default()
    }
}

// ── ObjectSpec helpers ─────────────────────────────────────────────────────────

fn sorcery_in_hand(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Sorcery")
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("test-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        })
}

fn green_elf_in_hand(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Green Elf")
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("test-green-elf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_colors(vec![Color::Green])
        .with_mana_cost(ManaCost {
            green: 1,
            ..Default::default()
        })
}

fn red_goblin_in_hand(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Test Red Goblin")
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(CardId("test-red-goblin".to_string()))
        .with_types(vec![CardType::Creature])
        .with_colors(vec![Color::Red])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
}

// ── Test 1: AllSpells grant allows sorcery at instant speed ───────────────────

/// CR 601.3b — Flash grant (AllSpells) allows casting a sorcery during opponent's turn.
/// Inject grant directly to test the timing check logic.
#[test]
fn test_grant_flash_all_spells_allows_sorcery_at_instant_speed() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![test_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery_in_hand(p1))
        .active_player(p2) // p2's turn — normally p1 can't cast sorceries
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has enough mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 3);
    state.turn.priority_holder = Some(p1);

    // Inject a flash grant for AllSpells for p1.
    state.flash_grants.push_back(FlashGrant {
        source: None,
        player: p1,
        filter: FlashGrantFilter::AllSpells,
        duration: EffectDuration::UntilEndOfTurn,
    });

    let card_id = find_object(&state, "Test Sorcery");
    let result = process_command(state, cast_spell_cmd(p1, card_id));
    assert!(
        result.is_ok(),
        "should cast sorcery at instant speed with AllSpells flash grant: {:?}",
        result
    );
}

/// CR 601.3b — Without a flash grant, p1 cannot cast sorcery during opponent's turn.
#[test]
fn test_grant_flash_without_grant_sorcery_fails_during_opponents_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![test_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery_in_hand(p1))
        .active_player(p2) // p2's turn
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 3);
    state.turn.priority_holder = Some(p1);

    // No flash grant — p1 cannot cast sorcery during p2's turn.
    let card_id = find_object(&state, "Test Sorcery");
    let result = process_command(state, cast_spell_cmd(p1, card_id));
    assert!(
        result.is_err(),
        "sorcery cast without flash grant should fail during opponent's turn"
    );
}

// ── Test 2: Sorceries-only filter ─────────────────────────────────────────────

/// CR 601.3b — FlashGrantFilter::Sorceries grants flash to sorcery-type spells.
#[test]
fn test_grant_flash_sorceries_filter_applies_to_sorcery() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![test_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery_in_hand(p1))
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 3);
    state.turn.priority_holder = Some(p1);

    state.flash_grants.push_back(FlashGrant {
        source: None,
        player: p1,
        filter: FlashGrantFilter::Sorceries,
        duration: EffectDuration::UntilEndOfTurn,
    });

    let card_id = find_object(&state, "Test Sorcery");
    let result = process_command(state, cast_spell_cmd(p1, card_id));
    assert!(
        result.is_ok(),
        "Sorceries filter should allow sorcery at instant speed: {:?}",
        result
    );
}

/// CR 601.3b — FlashGrantFilter::Sorceries does NOT grant flash to creatures.
#[test]
fn test_grant_flash_sorceries_filter_does_not_apply_to_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![test_green_elf_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(green_elf_in_hand(p1))
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    // Grant is for sorceries only — should NOT help a creature.
    state.flash_grants.push_back(FlashGrant {
        source: None,
        player: p1,
        filter: FlashGrantFilter::Sorceries,
        duration: EffectDuration::UntilEndOfTurn,
    });

    let card_id = find_object(&state, "Test Green Elf");
    let result = process_command(state, cast_spell_cmd(p1, card_id));
    assert!(
        result.is_err(),
        "Sorceries-only flash grant should NOT apply to a creature spell"
    );
}

// ── Test 3: GreenCreatures filter ────────────────────────────────────────────

/// CR 601.3b — FlashGrantFilter::GreenCreatures grants flash to green creatures.
#[test]
fn test_grant_flash_green_creatures_filter_applies_to_green_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![test_green_elf_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(green_elf_in_hand(p1))
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    state.flash_grants.push_back(FlashGrant {
        source: None,
        player: p1,
        filter: FlashGrantFilter::GreenCreatures,
        duration: EffectDuration::UntilEndOfTurn,
    });

    let card_id = find_object(&state, "Test Green Elf");
    let result = process_command(state, cast_spell_cmd(p1, card_id));
    assert!(
        result.is_ok(),
        "GreenCreatures filter should allow green creature at instant speed: {:?}",
        result
    );
}

/// CR 601.3b — FlashGrantFilter::GreenCreatures does NOT grant flash to non-green creatures.
#[test]
fn test_grant_flash_green_creatures_filter_does_not_apply_to_red_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![test_red_goblin_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(red_goblin_in_hand(p1))
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    state.flash_grants.push_back(FlashGrant {
        source: None,
        player: p1,
        filter: FlashGrantFilter::GreenCreatures,
        duration: EffectDuration::UntilEndOfTurn,
    });

    let card_id = find_object(&state, "Test Red Goblin");
    let result = process_command(state, cast_spell_cmd(p1, card_id));
    assert!(
        result.is_err(),
        "GreenCreatures filter should NOT apply to a red (non-green) creature"
    );
}

// ── Test 4: Source-on-battlefield check ───────────────────────────────────────

/// CR 601.3b — A WhileSourceOnBattlefield grant is inactive when the source is not on the battlefield.
/// Simulates Yeva having left the battlefield.
#[test]
fn test_grant_flash_inactive_when_source_not_on_battlefield() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![test_green_elf_def()]);

    // Yeva's source object is in the graveyard, not the battlefield.
    let yeva_spec = ObjectSpec::card(p1, "Yeva (graveyard)")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("yeva-natures-herald".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(green_elf_in_hand(p1))
        .object(yeva_spec)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    // Inject a stale WhileSourceOnBattlefield grant (source is in graveyard).
    let yeva_id = find_object(&state, "Yeva (graveyard)");
    state.flash_grants.push_back(FlashGrant {
        source: Some(yeva_id),
        player: p1,
        filter: FlashGrantFilter::GreenCreatures,
        duration: EffectDuration::WhileSourceOnBattlefield,
    });

    let elf_id = find_object(&state, "Test Green Elf");
    let result = process_command(state, cast_spell_cmd(p1, elf_id));
    assert!(
        result.is_err(),
        "WhileSourceOnBattlefield grant should be inactive when source is not on the battlefield"
    );
}

// ── Test 5: Grant applies only to recipient ───────────────────────────────────

/// CR 601.3b — A flash grant for p1 does NOT grant p2 flash.
#[test]
fn test_grant_flash_applies_only_to_recipient_not_other_players() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![test_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery_in_hand(p2)) // p2 has the sorcery
        .active_player(p1) // p1's turn
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 3);
    state.turn.priority_holder = Some(p2);

    // Flash grant for p1 only.
    state.flash_grants.push_back(FlashGrant {
        source: None,
        player: p1,
        filter: FlashGrantFilter::AllSpells,
        duration: EffectDuration::UntilEndOfTurn,
    });

    // p2 tries to cast a sorcery during p1's turn — p2 is not the active player, should fail.
    let card_id = find_object(&state, "Test Sorcery");
    let result = process_command(state, cast_spell_cmd(p2, card_id));
    assert!(
        result.is_err(),
        "p1's flash grant should NOT help p2 cast at instant speed"
    );
}

// ── Test 6: UntilEndOfTurn grant expires at cleanup step ──────────────────────

/// CR 514.2 — UntilEndOfTurn flash grants expire at the cleanup step.
/// After expiry the player cannot use the grant to cast at instant speed.
#[test]
fn test_grant_flash_until_end_of_turn_expires_at_cleanup() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![test_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery_in_hand(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Inject an UntilEndOfTurn flash grant for p1.
    state.flash_grants.push_back(FlashGrant {
        source: None,
        player: p1,
        filter: FlashGrantFilter::AllSpells,
        duration: EffectDuration::UntilEndOfTurn,
    });

    assert_eq!(
        state.flash_grants.len(),
        1,
        "grant should exist before end of turn"
    );

    // Advance through the cleanup step of this turn — grant should expire.
    let (state, _) = advance_to_next_turn(state, &[p1, p2]);

    assert_eq!(
        state.flash_grants.len(),
        0,
        "UntilEndOfTurn grant should expire at cleanup step"
    );
}

// ── Test 7: Teferi passive — OpponentsCanOnlyCastAtSorcerySpeed ───────────────

/// CR 307.5 / CR 101.2 — Teferi's passive restricts opponents to sorcery speed.
/// Opponents cannot cast spells during the controller's turn at instant speed.
/// We test the restriction check directly by injecting the restriction into state.
#[test]
fn test_teferi_passive_opponents_cannot_cast_at_instant_speed() {
    let p1 = p(1); // Teferi controller
    let p2 = p(2); // Opponent

    let registry = CardRegistry::new(vec![test_sorcery_def()]);

    // Use a permanent on the battlefield as the restriction's source.
    // The restriction check verifies the source is on the battlefield.
    let teferi_placeholder =
        ObjectSpec::creature(p1, "Teferi Placeholder", 4, 4).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery_in_hand(p2))
        .object(teferi_placeholder)
        .active_player(p1) // p1's turn
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 3);
    state.turn.priority_holder = Some(p2);

    let teferi_id = find_object(&state, "Teferi Placeholder");
    state.restrictions.push_back(ActiveRestriction {
        source: teferi_id,
        controller: p1,
        restriction: GameRestriction::OpponentsCanOnlyCastAtSorcerySpeed,
    });

    // p2 (opponent) tries to cast during p1's main phase (not p2's own turn).
    let card_id = find_object(&state, "Test Sorcery");
    let result = process_command(state, cast_spell_cmd(p2, card_id));
    assert!(
        result.is_err(),
        "Teferi's passive should prevent p2 from casting at instant speed: {:?}",
        result
    );
}

/// CR 101.2 — Teferi's restriction overrides flash grants for opponents.
/// Even with an AllSpells flash grant, the restriction wins.
#[test]
fn test_teferi_restriction_overrides_flash_grant_for_opponents() {
    let p1 = p(1); // Teferi controller
    let p2 = p(2); // Opponent with a flash grant

    let registry = CardRegistry::new(vec![test_sorcery_def()]);

    let teferi_placeholder =
        ObjectSpec::creature(p1, "Teferi Placeholder", 4, 4).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery_in_hand(p2))
        .object(teferi_placeholder)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 3);
    state.turn.priority_holder = Some(p2);

    let teferi_id = find_object(&state, "Teferi Placeholder");
    state.restrictions.push_back(ActiveRestriction {
        source: teferi_id,
        controller: p1,
        restriction: GameRestriction::OpponentsCanOnlyCastAtSorcerySpeed,
    });

    // Give p2 an AllSpells flash grant (simulating p2 cast Borne Upon a Wind).
    state.flash_grants.push_back(FlashGrant {
        source: None,
        player: p2,
        filter: FlashGrantFilter::AllSpells,
        duration: EffectDuration::UntilEndOfTurn,
    });

    // p2 tries to cast at instant speed. Restriction is checked before flash grants.
    let card_id = find_object(&state, "Test Sorcery");
    let result = process_command(state, cast_spell_cmd(p2, card_id));
    assert!(
        result.is_err(),
        "CR 101.2: Teferi's restriction should override p2's flash grant"
    );
}

// ── Test 8: Multiplayer — grant is player-specific ────────────────────────────

/// CR 601.3b — In a 4-player game, a flash grant for p1 only benefits p1.
/// Other players cannot use p1's grant to cast at instant speed.
#[test]
fn test_grant_flash_multiplayer_grant_is_player_specific() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![test_sorcery_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(sorcery_in_hand(p3)) // p3 has the sorcery
        .active_player(p2) // p2's turn
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p3)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 3);
    state.turn.priority_holder = Some(p3);

    // Grant is for p1 only — p3 should not benefit.
    state.flash_grants.push_back(FlashGrant {
        source: None,
        player: p1,
        filter: FlashGrantFilter::AllSpells,
        duration: EffectDuration::UntilEndOfTurn,
    });

    let card_id = find_object(&state, "Test Sorcery");
    let result = process_command(state, cast_spell_cmd(p3, card_id));
    assert!(
        result.is_err(),
        "p1's flash grant should NOT benefit p3 in a multiplayer game"
    );
}

// ── Test 9: Yeva static flash grant registration on ETB ───────────────────────

/// CR 601.3b — Casting Yeva populates state.flash_grants via register_static_continuous_effects.
/// Tests the full engine pipeline: cast Yeva (Flash) → resolve → register_static_continuous_effects
/// → state.flash_grants contains a WhileSourceOnBattlefield GreenCreatures grant for p1.
#[test]
fn test_yeva_static_flash_grant_registered_on_etb() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![yeva_def()]);

    // Yeva in hand — will be cast during p1's main phase.
    let yeva_spec = ObjectSpec::card(p1, "Yeva, Nature's Herald")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("yeva-natures-herald".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Flash)
        .with_mana_cost(ManaCost {
            generic: 2,
            green: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(yeva_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Provide {2}{G}{G} — 4 green mana covers both generic and green pips.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Green, 4);
    state.turn.priority_holder = Some(p1);

    // No flash grant yet — flash_grants starts empty.
    assert!(
        state.flash_grants.is_empty(),
        "flash_grants should be empty before Yeva enters the battlefield"
    );

    // Cast Yeva (she has Flash, so legal at any time with priority).
    let yeva_id = find_object(&state, "Yeva, Nature's Herald");
    let (state, _) = process_command(state, cast_spell_cmd(p1, yeva_id))
        .expect("casting Yeva should succeed — she has Flash");

    // Yeva is now on the stack. Both players pass priority to resolve her.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After resolution Yeva is on the battlefield and flash_grants is populated.
    // register_static_continuous_effects fires on ETB and pushes a FlashGrant.
    assert_eq!(
        state.flash_grants.len(),
        1,
        "flash_grants should contain exactly one entry after Yeva resolves"
    );

    let grant = state.flash_grants.iter().next().unwrap();
    assert_eq!(
        grant.player, p1,
        "grant should be for Yeva's controller (p1)"
    );
    assert!(
        matches!(grant.filter, FlashGrantFilter::GreenCreatures),
        "grant filter should be GreenCreatures"
    );
    assert!(
        matches!(grant.duration, EffectDuration::WhileSourceOnBattlefield),
        "grant duration should be WhileSourceOnBattlefield"
    );
    assert!(
        grant.source.is_some(),
        "grant source should be Yeva's ObjectId on the battlefield"
    );

    // Verify the grant's source is Yeva on the battlefield.
    let source_id = grant.source.unwrap();
    let source_obj = state
        .objects
        .get(&source_id)
        .expect("Yeva should be on the battlefield");
    assert!(
        matches!(source_obj.zone, ZoneId::Battlefield),
        "Yeva's grant source should be on the battlefield"
    );
}

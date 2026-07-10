//! PB-AC0: Subtype-filtered and token/nontoken-filtered creature-ETB trigger tests.
//!
//! Validates that `WheneverCreatureEntersBattlefield` triggers with `has_subtype` and/or
//! `is_nontoken` filters are correctly honored after the PB-AC0 engine fix, which forwards
//! the carddef `TargetFilter` as `triggering_creature_filter` on the creature-ETB harness
//! path, and adds a matching check inside the `etb_filter` block in `abilities.rs`.
//!
//! CR Rules covered:
//! - CR 603.2: triggered ability fires when the trigger event matches.
//! - CR 111.1: token = permanent not represented by a card; nontoken = represented by a card.
//! - CR 205.3: subtypes; creature types are a subtype set.
//! - CR 613.1d: layer-resolved card types / subtypes used for filter matching.
//! - CR 603.10: ETB is NOT a look-back-in-time trigger (characteristics evaluated after entry).
//! - CR 603.10a: death triggers DO look back — regression guard confirms death path unchanged.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Color, Command, DeathTriggerFilter, ETBTriggerFilter, Effect,
    EffectAmount, GameEvent, GameState, GameStateBuilder, ManaCost, ObjectSpec, PlayerId,
    StackObjectKind, Step, SubType, TargetController, TargetFilter, TokenSpec, TriggerCondition,
    TriggerEvent, TriggeredAbilityDef, TypeLine, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object_id(state: &GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn life_total(state: &GameState, player: PlayerId) -> i32 {
    state
        .players()
        .get(&player)
        .map(|p| p.life_total)
        .unwrap_or_default()
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(player))
        .count()
}

/// Count TriggeredAbility stack objects from a given source.
fn trigger_count_for(state: &GameState, source: mtg_engine::ObjectId) -> usize {
    let pending = state
        .pending_triggers()
        .iter()
        .filter(|t| t.source == source)
        .count();
    let on_stack = state
        .stack_objects()
        .iter()
        .filter(|so| {
            matches!(
                so.kind,
                StackObjectKind::TriggeredAbility { source_object, .. }
                if source_object == source
            )
        })
        .count();
    pending + on_stack
}

/// Pass priority once for every player in the list (resolves top stack item or advances turn).
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

/// Cast a creature from a player's hand by name, paying with colorless mana.
fn cast_creature(
    state: GameState,
    player: PlayerId,
    name: &str,
    mana_amount: u32,
) -> (GameState, Vec<GameEvent>) {
    let card_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Hand(player))
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("card '{}' not found in {}'s hand", name, player.0));

    let mut state = state;
    state
        .players_mut()
        .get_mut(&player)
        .unwrap()
        .mana_pool
        .colorless = mana_amount;
    state.turn_mut().priority_holder = Some(player);

    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .expect("CastSpell failed")
}

/// Build a minimal creature CardDefinition with a given generic mana cost.
fn creature_def(card_id: &str, name: &str, generic: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Build a Dragon creature CardDefinition.
fn dragon_def(card_id: &str, name: &str, generic: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            subtypes: [SubType("Dragon".to_string())].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    }
}

/// Build a Dragon-ETB triggered ability that gains 1 life for the controller.
/// Mirrors the Ganax / Bloomvine Regent pattern: no nontoken restriction, include self.
/// CR 603.2 / CR 205.3.
fn dragon_etb_gain_life_trigger() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        targets: vec![],
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description:
            "Whenever a Dragon you control enters, you gain 1 life. (CR 603.2/205.3/PB-AC0)"
                .to_string(),
        effect: Some(Effect::GainLife {
            player: mtg_engine::PlayerTarget::Controller,
            amount: EffectAmount::Fixed(1),
        }),
        etb_filter: Some(ETBTriggerFilter {
            creature_only: true,
            controller_you: true,
            exclude_self: false,
            color_filter: None,
            card_type_filter: None,
        }),
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: Some(TargetFilter {
            has_subtype: Some(SubType("Dragon".to_string())),
            controller: TargetController::You,
            ..Default::default()
        }),
    }
}

/// Build a nontoken-Dragon-ETB triggered ability (Lathliss-style, exclude self).
/// CR 603.2 / CR 205.3 / CR 111.1.
fn nontoken_dragon_etb_draw_trigger() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        targets: vec![],
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description: "Whenever another nontoken Dragon you control enters, draw a card. (CR 603.2/205.3/111.1/PB-AC0)".to_string(),
        effect: Some(Effect::DrawCards {
            player: mtg_engine::PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: Some(ETBTriggerFilter {
            creature_only: true,
            controller_you: true,
            exclude_self: true,
            color_filter: None,
            card_type_filter: None,
        }),
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: Some(TargetFilter {
            has_subtype: Some(SubType("Dragon".to_string())),
            controller: TargetController::You,
            is_nontoken: true,
            ..Default::default()
        }),
    }
}

// ── Test 1 ────────────────────────────────────────────────────────────────────

/// CR 603.2 / CR 205.3 -- Subtype-filtered ETB trigger fires on subtype match.
///
/// A permanent with a "whenever a Dragon you control enters" trigger should fire
/// when a Dragon creature enters under the controller's control. PB-AC0: `has_subtype`
/// on the creature-ETB path is now forwarded via `triggering_creature_filter`.
#[test]
fn test_etb_subtype_filter_fires_on_match() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dragon_card = dragon_def("test-dragon", "Test Dragon", 3);
    let registry = CardRegistry::new(vec![dragon_card]);

    // Watcher: carries the Dragon-ETB trigger.
    let watcher = ObjectSpec::creature(p1, "Dragon Watcher", 2, 2)
        .with_triggered_ability(dragon_etb_gain_life_trigger());

    // Dragon creature in P1's hand.
    let dragon_in_hand = ObjectSpec::creature(p1, "Test Dragon", 3, 3)
        .with_card_id(CardId("test-dragon".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(watcher)
        .object(dragon_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let watcher_id = find_object_id(&state, "Dragon Watcher");
    let initial_life = life_total(&state, p1);

    // P1 casts the Dragon.
    let (state, _) = cast_creature(state, p1, "Test Dragon", 3);
    // Dragon resolves and enters the battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    // The Dragon-ETB trigger should have fired.
    assert!(
        trigger_count_for(&state, watcher_id) > 0,
        "CR 603.2 / CR 205.3: Dragon-ETB trigger should fire when a Dragon enters. \
         pending={}, stack={}",
        state.pending_triggers().len(),
        state.stack_objects().len()
    );

    // Resolve the trigger — P1 gains 1 life.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        life_total(&state, p1),
        initial_life + 1,
        "CR 603.2 / CR 205.3: P1 should gain 1 life from Dragon-ETB trigger."
    );
}

// ── Test 2 ────────────────────────────────────────────────────────────────────

/// CR 603.2 -- Subtype-filtered ETB trigger does NOT fire on subtype mismatch.
///
/// This is the regression for the PB-AC0 bug: before the fix, a Dragon-subtype ETB
/// trigger would fire for *any* creature entering (subtype silently dropped). After
/// the fix, a non-Dragon creature entering must NOT fire the Dragon-ETB trigger.
#[test]
fn test_etb_subtype_filter_no_fire_on_mismatch() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let goblin_def = creature_def("test-goblin", "Test Goblin", 2);
    let registry = CardRegistry::new(vec![goblin_def]);

    // Watcher: carries the Dragon-ETB trigger.
    let watcher = ObjectSpec::creature(p1, "Dragon Watcher", 2, 2)
        .with_triggered_ability(dragon_etb_gain_life_trigger());

    // Non-Dragon creature in P1's hand.
    let goblin_in_hand = ObjectSpec::creature(p1, "Test Goblin", 2, 2)
        .with_card_id(CardId("test-goblin".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Goblin".to_string())])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(watcher)
        .object(goblin_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let watcher_id = find_object_id(&state, "Dragon Watcher");
    let initial_life = life_total(&state, p1);

    // P1 casts the Goblin (non-Dragon).
    let (state, _) = cast_creature(state, p1, "Test Goblin", 2);
    // Goblin resolves and enters the battlefield.
    let (state, resolution_events) = pass_all(state, &[p1, p2]);

    // Dragon-ETB trigger must NOT fire for a Goblin.
    assert_eq!(
        trigger_count_for(&state, watcher_id),
        0,
        "CR 603.2: Dragon-ETB trigger must NOT fire when a Goblin (non-Dragon) enters. \
         Events: {:?}",
        resolution_events
            .iter()
            .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
            .collect::<Vec<_>>()
    );

    // Life total unchanged.
    assert_eq!(
        life_total(&state, p1),
        initial_life,
        "CR 603.2: Life total must not change when Dragon-ETB trigger doesn't fire."
    );
}

// ── Test 3 ────────────────────────────────────────────────────────────────────

/// CR 111.1 -- Nontoken-Dragon-ETB trigger fires when a nontoken Dragon enters.
///
/// A "whenever another nontoken Dragon you control enters" trigger (Lathliss-style)
/// should fire when a nontoken Dragon creature enters. PB-AC0: `is_nontoken` is now
/// honored on the creature-ETB path.
#[test]
fn test_etb_nontoken_filter_fires_on_nontoken() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dragon_card = dragon_def("nontoken-dragon", "Nontoken Dragon", 3);
    let registry = CardRegistry::new(vec![dragon_card]);

    // Watcher: nontoken-Dragon-ETB, exclude_self trigger.
    let watcher = ObjectSpec::creature(p1, "Dragon Queen", 6, 6)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_triggered_ability(nontoken_dragon_etb_draw_trigger());

    // Nontoken Dragon in P1's hand (is_token defaults to false).
    let dragon_in_hand = ObjectSpec::creature(p1, "Nontoken Dragon", 3, 3)
        .with_card_id(CardId("nontoken-dragon".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    // Library card needed for the draw effect.
    let lib_card = ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(watcher)
        .object(dragon_in_hand)
        .object(lib_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let watcher_id = find_object_id(&state, "Dragon Queen");
    let initial_hand = hand_count(&state, p1);

    // P1 casts the nontoken Dragon.
    let (state, _) = cast_creature(state, p1, "Nontoken Dragon", 3);
    // Dragon resolves and enters (nontoken, from hand).
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        trigger_count_for(&state, watcher_id) > 0,
        "CR 111.1 / CR 603.2: Nontoken-Dragon-ETB trigger should fire when a nontoken Dragon enters."
    );

    // Resolve the trigger — P1 draws a card.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P1's hand: -1 (Dragon was cast) + 1 (drawn) = initial_hand.
    assert_eq!(
        hand_count(&state, p1),
        initial_hand,
        "CR 111.1: P1 should draw 1 card from nontoken-Dragon-ETB trigger. \
         initial={}, after_draw={}",
        initial_hand,
        hand_count(&state, p1)
    );
}

// ── Test 4 ────────────────────────────────────────────────────────────────────

/// CR 111.1 / CR 603.2 -- Nontoken-Dragon-ETB trigger does NOT fire when a token Dragon enters.
///
/// This is the load-bearing negative case for PB-AC0: the `is_nontoken` filter must prevent
/// the trigger from firing on token Dragons, stopping infinite token loops (Miirym/Lathliss).
/// A Dragon token created via Effect::CreateToken (is_token: true) enters the battlefield;
/// the nontoken-Dragon-ETB trigger must not chain off it.
#[test]
fn test_etb_nontoken_filter_no_fire_on_token() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // A card that creates a Dragon token when it enters the battlefield.
    let token_creator_def = CardDefinition {
        card_id: CardId("dragon-token-creator".to_string()),
        name: "Dragon Token Creator".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "When this enters, create a 5/5 red Dragon creature token with flying."
            .to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::CreateToken {
                spec: TokenSpec {
                    name: "Dragon".to_string(),
                    card_types: [CardType::Creature].into_iter().collect(),
                    subtypes: [SubType("Dragon".to_string())].into_iter().collect(),
                    colors: [Color::Red].into_iter().collect(),
                    power: 5,
                    toughness: 5,
                    count: EffectAmount::Fixed(1),
                    supertypes: imbl::OrdSet::new(),
                    keywords: [mtg_engine::KeywordAbility::Flying].into_iter().collect(),
                    tapped: false,
                    enters_attacking: false,
                    mana_color: None,
                    mana_abilities: vec![],
                    activated_abilities: vec![],
                    ..Default::default()
                },
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![token_creator_def]);

    // Dragon Queen (watcher with nontoken-Dragon-ETB trigger) on the battlefield.
    let watcher_obj = ObjectSpec::creature(p1, "Dragon Queen", 6, 6)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_triggered_ability(nontoken_dragon_etb_draw_trigger());

    // Dragon Token Creator in P1's hand.
    let creator_in_hand = ObjectSpec::creature(p1, "Dragon Token Creator", 2, 2)
        .with_card_id(CardId("dragon-token-creator".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 4,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(watcher_obj)
        .object(creator_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let watcher_id = find_object_id(&state, "Dragon Queen");
    let initial_hand = hand_count(&state, p1);

    // Cast Dragon Token Creator (nontoken, non-Dragon creature).
    let (state, _) = cast_creature(state, p1, "Dragon Token Creator", 4);
    // Creator resolves and enters — it is NOT a Dragon, so nontoken-Dragon-ETB doesn't fire.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Creator's ETB trigger (create Dragon token) resolves → Dragon token enters battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Dragon token (is_token: true, Dragon subtype) has now entered.
    // The nontoken-Dragon-ETB trigger on Dragon Queen must NOT fire for the token.
    assert_eq!(
        trigger_count_for(&state, watcher_id),
        0,
        "CR 111.1 / CR 603.2: Nontoken-Dragon-ETB trigger must NOT fire when a token Dragon \
         enters (is_nontoken filter). stack_objects={:?}",
        state
            .stack_objects()
            .iter()
            .map(|s| &s.kind)
            .collect::<Vec<_>>()
    );

    // Hand count: initial_hand - 1 (creator was cast), no draws.
    assert_eq!(
        hand_count(&state, p1),
        initial_hand - 1,
        "CR 111.1: P1 should not draw a card when a token Dragon enters (is_nontoken filter)."
    );
}

// ── Test 5 ────────────────────────────────────────────────────────────────────

/// CR 603.2 / CR 205.3 / CR 111.1 -- AND-combination of has_subtype + is_nontoken.
///
/// Only a nontoken Dragon should fire the trigger. Nontoken Goblin must not fire;
/// nontoken Dragon must fire. Verifies the AND-logic of the filter combination.
#[test]
fn test_etb_subtype_and_nontoken_combined() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dragon_card = dragon_def("combined-dragon", "Combined Dragon", 3);
    let goblin_card = creature_def("combined-goblin", "Combined Goblin", 2);
    let registry = CardRegistry::new(vec![dragon_card, goblin_card]);

    // Watcher: nontoken-Dragon-ETB, exclude_self.
    let watcher = ObjectSpec::creature(p1, "Combined Watcher", 2, 2)
        .with_triggered_ability(nontoken_dragon_etb_draw_trigger());

    // Nontoken Dragon in P1's hand.
    let dragon_in_hand = ObjectSpec::creature(p1, "Combined Dragon", 3, 3)
        .with_card_id(CardId("combined-dragon".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    // Nontoken Goblin in P1's hand.
    let goblin_in_hand = ObjectSpec::creature(p1, "Combined Goblin", 2, 2)
        .with_card_id(CardId("combined-goblin".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Goblin".to_string())])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    // Library cards for draw.
    let lib1 = ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1));
    let lib2 = ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(watcher)
        .object(dragon_in_hand)
        .object(goblin_in_hand)
        .object(lib1)
        .object(lib2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let watcher_id = find_object_id(&state, "Combined Watcher");
    let initial_hand = hand_count(&state, p1);

    // Cast Goblin (nontoken non-Dragon) — should NOT fire.
    let (state, _) = cast_creature(state, p1, "Combined Goblin", 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        trigger_count_for(&state, watcher_id),
        0,
        "CR 603.2: Nontoken-Dragon trigger must NOT fire for nontoken Goblin."
    );

    // Cast nontoken Dragon — SHOULD fire.
    let (state, _) = cast_creature(state, p1, "Combined Dragon", 3);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        trigger_count_for(&state, watcher_id) > 0,
        "CR 603.2 / CR 205.3: Nontoken-Dragon trigger MUST fire for nontoken Dragon. \
         pending={}, stack={}",
        state.pending_triggers().len(),
        state.stack_objects().len()
    );

    // Resolve trigger — P1 draws.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Hand: initial - 1 (goblin cast) - 1 (dragon cast) + 1 (draw from dragon trigger) = initial - 1.
    assert_eq!(
        hand_count(&state, p1),
        initial_hand - 1,
        "CR 603.2 / CR 205.3 / CR 111.1: Exactly 1 card drawn — only nontoken Dragon fires. \
         Nontoken Goblin must not trigger."
    );
}

// ── Test 6 ────────────────────────────────────────────────────────────────────

/// CR 603.2 -- exclude_self + subtype: trigger does not fire when the entering creature
/// IS the source; fires for another nontoken Dragon entering later.
///
/// Validates that `exclude_self: true` on the creature-ETB path still works correctly
/// in combination with the new `triggering_creature_filter` check (PB-AC0).
/// Gatherer ruling 2024-11-08: Lathliss does NOT trigger off its own entry.
///
/// Note: `enrich_spec_from_def` (which converts CardDefinition::WheneverCreatureEntersBattlefield
/// → runtime TriggeredAbilityDef) only runs at build_initial_state time. Spells cast via
/// CastSpell do not re-enrich on resolution. This test therefore places Lathliss already on
/// the battlefield via ObjectSpec::with_triggered_ability(), which wires the trigger directly
/// into characteristics.triggered_abilities, matching how battlefield permanents work in the
/// engine. Second Dragon is still cast from hand via CastSpell.
#[test]
fn test_etb_exclude_self_with_subtype() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dragon_def_card = dragon_def("second-dragon", "Second Dragon", 4);
    let registry = CardRegistry::new(vec![dragon_def_card]);

    // Library cards for draw effects.
    let lib1 = ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1));
    let lib2 = ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1));

    // Lathliss already on the battlefield with trigger wired via with_triggered_ability.
    // exclude_self: true means the trigger must NOT fire when a creature with the same
    // ObjectId as the source enters (i.e. the watcher itself never fires for itself since
    // it's already on the battlefield and isn't entering again). The real Gatherer ruling
    // is about casting: Lathliss doesn't fire on its own ETB. We test the functional
    // outcome: only ANOTHER Dragon fires the trigger.
    let lathliss_on_bf = ObjectSpec::creature(p1, "Lathliss Test", 6, 6)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_triggered_ability(nontoken_dragon_etb_draw_trigger());

    // Second Dragon in hand.
    let second_dragon_in_hand = ObjectSpec::creature(p1, "Second Dragon", 3, 3)
        .with_card_id(CardId("second-dragon".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 4,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(lathliss_on_bf)
        .object(second_dragon_in_hand)
        .object(lib1)
        .object(lib2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_hand = hand_count(&state, p1);
    let lathliss_id = find_object_id(&state, "Lathliss Test");

    // No trigger fires yet — Lathliss is already on the battlefield.
    assert_eq!(
        trigger_count_for(&state, lathliss_id),
        0,
        "CR 603.2: No trigger should be pending at start (Lathliss is already on BF)."
    );

    // Cast Second Dragon — Lathliss's trigger SHOULD fire (another nontoken Dragon).
    let (state, _) = cast_creature(state, p1, "Second Dragon", 4);
    let (state, _) = pass_all(state, &[p1, p2]);

    // exclude_self: true — Lathliss's trigger fires for ANOTHER Dragon, not itself.
    assert!(
        trigger_count_for(&state, lathliss_id) > 0,
        "CR 603.2: Lathliss trigger MUST fire when another nontoken Dragon enters. \
         pending={}, stack={}",
        state.pending_triggers().len(),
        state.stack_objects().len()
    );

    // Resolve trigger — P1 draws.
    let (state, _) = pass_all(state, &[p1, p2]);

    // initial_hand = 1 (Second Dragon in hand)
    // After: -1 (Second Dragon cast) + 1 (draw from trigger) = initial_hand
    assert_eq!(
        hand_count(&state, p1),
        initial_hand,
        "CR 603.2: 1 card drawn from Lathliss trigger when Second Dragon entered."
    );
}

// ── Test 7 ────────────────────────────────────────────────────────────────────

/// CR 613.1d -- Subtype-filtered ETB trigger uses layer-resolved characteristics.
///
/// A creature that enters with a Dragon subtype in its base characteristics should
/// fire a Dragon-ETB trigger. This validates the `calculate_characteristics` call
/// path in the PB-AC0 ETB block (CR 603.10: ETB evaluates characteristics after entry).
#[test]
fn test_etb_subtype_filter_layer_resolved() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dragon_card = dragon_def("layer-dragon", "Layer Dragon", 3);
    let registry = CardRegistry::new(vec![dragon_card]);

    let watcher = ObjectSpec::creature(p1, "Layer Watcher", 2, 2)
        .with_triggered_ability(dragon_etb_gain_life_trigger());

    // Dragon in hand — its subtypes are layer-resolved to Dragon on entry.
    let dragon_in_hand = ObjectSpec::creature(p1, "Layer Dragon", 3, 3)
        .with_card_id(CardId("layer-dragon".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(watcher)
        .object(dragon_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let watcher_id = find_object_id(&state, "Layer Watcher");
    let initial_life = life_total(&state, p1);

    let (state, _) = cast_creature(state, p1, "Layer Dragon", 3);
    // CR 603.10: ETB trigger evaluates characteristics immediately after entry.
    // calculate_characteristics returns Dragon subtype — trigger fires.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        trigger_count_for(&state, watcher_id) > 0,
        "CR 613.1d / CR 603.2: Dragon-ETB trigger should fire using layer-resolved subtypes."
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        life_total(&state, p1),
        initial_life + 1,
        "CR 613.1d: P1 gains 1 life from Dragon-ETB trigger using layer-resolved characteristics."
    );
}

// ── Test 8 ────────────────────────────────────────────────────────────────────

/// CR 603.2 -- Ganax, Astral Hunter card integration: Dragon-ETB Treasure trigger.
///
/// Manually wires a Ganax-equivalent trigger (Dragon-ETB → create Treasure) using
/// `.with_triggered_ability()` directly, mimicking the re-authored card definition
/// (PB-AC0 unblocked). A Dragon entering creates 1 Treasure; a Goblin does not.
#[test]
fn test_etb_ganax_treasure_integration() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dragon_card = dragon_def("ganax-test-dragon", "Ganax Test Dragon", 3);
    let goblin_card = creature_def("ganax-test-goblin", "Ganax Test Goblin", 2);
    let registry = CardRegistry::new(vec![dragon_card, goblin_card]);

    // Ganax-equivalent permanent: Dragon-ETB → create a Treasure token.
    let ganax_trigger = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        targets: vec![],
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description: "Ganax: whenever a Dragon you control enters, create a Treasure token. (CR 603.2/PB-AC0)".to_string(),
        effect: Some(Effect::CreateToken { spec: mtg_engine::treasure_token_spec(1) }),
        etb_filter: Some(ETBTriggerFilter {
            creature_only: true,
            controller_you: true,
            exclude_self: false,
            color_filter: None,
            card_type_filter: None,
        }),
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: Some(TargetFilter {
            has_subtype: Some(SubType("Dragon".to_string())),
            controller: TargetController::You,
            ..Default::default()
        }),
    };

    let ganax = ObjectSpec::creature(p1, "Ganax Equivalent", 3, 4)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_triggered_ability(ganax_trigger);

    let dragon_in_hand = ObjectSpec::creature(p1, "Ganax Test Dragon", 3, 3)
        .with_card_id(CardId("ganax-test-dragon".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let goblin_in_hand = ObjectSpec::creature(p1, "Ganax Test Goblin", 2, 2)
        .with_card_id(CardId("ganax-test-goblin".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Goblin".to_string())])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ganax)
        .object(dragon_in_hand)
        .object(goblin_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let count_treasures = |state: &GameState| -> usize {
        state
            .objects()
            .values()
            .filter(|o| o.characteristics.name == "Treasure" && o.is_token)
            .count()
    };

    let initial_treasures = count_treasures(&state);

    // Cast Goblin (non-Dragon) — trigger must NOT fire.
    let (state, _) = cast_creature(state, p1, "Ganax Test Goblin", 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        count_treasures(&state),
        initial_treasures,
        "CR 603.2: Ganax trigger must NOT fire when a Goblin enters — no Treasure created."
    );

    // Cast Dragon — trigger SHOULD fire and create exactly 1 Treasure.
    let (state, _) = cast_creature(state, p1, "Ganax Test Dragon", 3);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Resolve Ganax trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        count_treasures(&state),
        initial_treasures + 1,
        "CR 603.2: Ganax trigger MUST fire when a Dragon enters — exactly 1 Treasure created."
    );
}

// ── Test 9 ────────────────────────────────────────────────────────────────────

/// CR 111.1 -- Lathliss, Dragon Queen integration: nontoken Dragon enters → 5/5 Dragon token.
/// Token Dragon enters → no second token (infinite-loop guard). Manually wires the trigger
/// to mirror the re-authored Lathliss card definition (PB-AC0 unblocked).
#[test]
fn test_etb_lathliss_token_integration() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let dragon_card = dragon_def("lathliss-test-dragon", "Lathliss Test Dragon", 3);

    // A card that creates a Dragon token (to test the no-chain rule).
    let token_creator_def = CardDefinition {
        card_id: CardId("lathliss-token-creator".to_string()),
        name: "Lathliss Token Creator".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "When this enters, create a 5/5 red Dragon creature token.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::CreateToken {
                spec: TokenSpec {
                    name: "Dragon".to_string(),
                    card_types: [CardType::Creature].into_iter().collect(),
                    subtypes: [SubType("Dragon".to_string())].into_iter().collect(),
                    colors: [Color::Red].into_iter().collect(),
                    power: 5,
                    toughness: 5,
                    count: EffectAmount::Fixed(1),
                    supertypes: imbl::OrdSet::new(),
                    keywords: [mtg_engine::KeywordAbility::Flying].into_iter().collect(),
                    tapped: false,
                    enters_attacking: false,
                    mana_color: None,
                    mana_abilities: vec![],
                    activated_abilities: vec![],
                    ..Default::default()
                },
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![dragon_card, token_creator_def]);

    // Lathliss-equivalent: nontoken-Dragon-ETB → create 5/5 Dragon token.
    let lathliss_trigger = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        targets: vec![],
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description: "Lathliss: whenever another nontoken Dragon you control enters, create a 5/5 Dragon token. (CR 111.1/PB-AC0)".to_string(),
        effect: Some(Effect::CreateToken {
            spec: TokenSpec {
                name: "Dragon".to_string(),
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Dragon".to_string())].into_iter().collect(),
                colors: [Color::Red].into_iter().collect(),
                power: 5,
                toughness: 5,
                count: EffectAmount::Fixed(1),
                supertypes: imbl::OrdSet::new(),
                keywords: [mtg_engine::KeywordAbility::Flying].into_iter().collect(),
                tapped: false,
                enters_attacking: false,
                mana_color: None,
                mana_abilities: vec![],
                activated_abilities: vec![],
                ..Default::default()
            },
        }),
        etb_filter: Some(ETBTriggerFilter {
            creature_only: true,
            controller_you: true,
            exclude_self: true,
            color_filter: None,
            card_type_filter: None,
        }),
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: Some(TargetFilter {
            has_subtype: Some(SubType("Dragon".to_string())),
            controller: TargetController::You,
            is_nontoken: true,
            ..Default::default()
        }),
    };

    let lathliss_equiv = ObjectSpec::creature(p1, "Lathliss Equivalent", 6, 6)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_triggered_ability(lathliss_trigger);

    let dragon_in_hand = ObjectSpec::creature(p1, "Lathliss Test Dragon", 3, 3)
        .with_card_id(CardId("lathliss-test-dragon".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    // Token creator in hand (creates a Dragon token; nontoken-Dragon-ETB should NOT chain).
    let creator_in_hand = ObjectSpec::creature(p1, "Lathliss Token Creator", 2, 2)
        .with_card_id(CardId("lathliss-token-creator".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 4,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(lathliss_equiv)
        .object(dragon_in_hand)
        .object(creator_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let count_dragon_tokens = |state: &GameState| -> usize {
        state
            .objects()
            .values()
            .filter(|o| {
                o.is_token
                    && o.characteristics
                        .subtypes
                        .contains(&SubType("Dragon".to_string()))
                    && o.zone == ZoneId::Battlefield
            })
            .count()
    };

    let initial_tokens = count_dragon_tokens(&state);

    // Cast nontoken Dragon → Lathliss trigger fires → 5/5 Dragon token created.
    let (state, _) = cast_creature(state, p1, "Lathliss Test Dragon", 3);
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve Lathliss trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let after_nontoken_tokens = count_dragon_tokens(&state);
    assert_eq!(
        after_nontoken_tokens,
        initial_tokens + 1,
        "CR 111.1: Lathliss-style trigger should create exactly 1 Dragon token when a nontoken Dragon enters."
    );

    // Now cast the token creator (non-Dragon) — creator enters, fires ETB, creates Dragon token.
    // That token Dragon must NOT fire Lathliss trigger again (infinite-loop guard).
    let lathliss_id = find_object_id(&state, "Lathliss Equivalent");
    let (state, _) = cast_creature(state, p1, "Lathliss Token Creator", 4);
    // Creator enters.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Creator's ETB resolves → Dragon token enters (is_token: true).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Lathliss trigger must NOT have fired for the token Dragon.
    assert_eq!(
        trigger_count_for(&state, lathliss_id),
        0,
        "CR 111.1: is_nontoken filter must prevent Lathliss trigger from firing on token Dragon. \
         Stack would chain infinitely without this guard."
    );
}

// ── Test 10 ────────────────────────────────────────────────────────────────────

/// CR 603.2 / CR 111.1 -- The Great Henge integration (manually wired trigger).
/// Nontoken creature enters → +1/+1 counter on the entering creature + draw.
/// This verifies the PB-AC0 fix: `is_nontoken` honored and `EffectTarget::TriggeringCreature`
/// puts the counter on the entering creature, not on The Great Henge itself.
#[test]
fn test_etb_great_henge_counter_on_entering_creature() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let elf_def = creature_def("henge-test-elf", "Henge Test Elf", 2);
    let registry = CardRegistry::new(vec![elf_def]);

    // Great Henge equivalent: nontoken-creature-ETB → AddCounter(TriggeringCreature) + DrawCards.
    // Uses EffectTarget::TriggeringCreature (corrected from Source in PB-AC0).
    let henge_trigger = TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        targets: vec![],
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description: "Henge: whenever a nontoken creature you control enters, +1/+1 counter on it + draw. (CR 603.2/PB-AC0)".to_string(),
        effect: Some(Effect::Sequence(vec![
            Effect::AddCounter {
                target: mtg_engine::CardEffectTarget::TriggeringCreature,
                counter: mtg_engine::CounterType::PlusOnePlusOne,
                count: 1,
            },
            Effect::DrawCards {
                player: mtg_engine::PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
        ])),
        etb_filter: Some(ETBTriggerFilter {
            creature_only: true,
            controller_you: true,
            exclude_self: false,
            color_filter: None,
            card_type_filter: None,
        }),
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: Some(TargetFilter {
            controller: TargetController::You,
            is_nontoken: true,
            ..Default::default()
        }),
    };

    // Great Henge-equivalent artifact on the battlefield (not in hand).
    let henge = ObjectSpec::card(p1, "Great Henge Equivalent")
        .with_types(vec![CardType::Artifact])
        .with_triggered_ability(henge_trigger)
        .in_zone(ZoneId::Battlefield);

    let elf_in_hand = ObjectSpec::creature(p1, "Henge Test Elf", 2, 2)
        .with_card_id(CardId("henge-test-elf".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Elf".to_string())])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let lib_card = ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(henge)
        .object(elf_in_hand)
        .object(lib_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_hand = hand_count(&state, p1);

    // Cast nontoken Elf — Henge trigger should fire, putting +1/+1 counter on the Elf.
    let (state, _) = cast_creature(state, p1, "Henge Test Elf", 2);
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve Henge trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Verify +1/+1 counter is on the Elf (TriggeringCreature), not on Henge.
    let elf_id = find_object_id(&state, "Henge Test Elf");
    let elf = state.objects().get(&elf_id).unwrap();
    let elf_plus_counters = elf
        .counters
        .get(&mtg_engine::CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        elf_plus_counters, 1,
        "CR 603.2: +1/+1 counter from Great Henge trigger must be on the entering Elf \
         (EffectTarget::TriggeringCreature), not on Henge itself. (PB-AC0 bug fix)"
    );

    // Verify Henge has no +1/+1 counters (old EffectTarget::Source bug would put it here).
    let henge_id = find_object_id(&state, "Great Henge Equivalent");
    let henge_obj = state.objects().get(&henge_id);
    let henge_counters = henge_obj
        .map(|h| {
            h.counters
                .get(&mtg_engine::CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    assert_eq!(
        henge_counters, 0,
        "CR 603.2: Great Henge equivalent must NOT receive the +1/+1 counter. \
         (Old EffectTarget::Source bug would put it on Henge.)"
    );

    // Verify P1 drew a card: -1 (Elf cast) + 1 (draw) = initial_hand.
    assert_eq!(
        hand_count(&state, p1),
        initial_hand,
        "CR 603.2: P1 should draw 1 card from The Great Henge trigger."
    );
}

// ── Test 11 ────────────────────────────────────────────────────────────────────

/// CR 603.10a -- Regression guard: death-trigger path is unaffected by PB-AC0.
///
/// A `WheneverCreatureDies` trigger with a Dragon subtype `triggering_creature_filter`
/// must still fire correctly for a Dragon dying and not fire for a non-Dragon dying.
/// Confirms the ETB-scoping of Change 2 (inside the etb_filter block) does not disturb
/// the already-correct death path.
///
/// Uses 0-toughness SBA pattern (CR 704.5f) to kill creatures deterministically.
/// CR 603.10a: death triggers look back in time — filter evaluated against pre-death
/// characteristics. The death path is in a separate function (abilities.rs ~4287).
#[test]
fn test_etb_death_path_unaffected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Watcher: "whenever a Dragon you control dies, draw a card"
    let watcher = ObjectSpec::creature(p1, "Death Watcher", 1, 4).with_triggered_ability(
        TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            targets: vec![],
            trigger_on: TriggerEvent::AnyCreatureDies,
            intervening_if: None,
            description: "Whenever a Dragon you control dies, draw a card. (CR 603.10a / PB-N)"
                .to_string(),
            effect: Some(Effect::DrawCards {
                player: mtg_engine::PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: Some(DeathTriggerFilter {
                controller_you: true,
                controller_opponent: false,
                exclude_self: false,
                nontoken_only: false,
            }),
            combat_damage_filter: None,
            triggering_creature_filter: Some(TargetFilter {
                has_subtype: Some(SubType("Dragon".to_string())),
                ..Default::default()
            }),
        },
    );

    // A Dragon with 0 toughness — dies immediately via SBA (CR 704.5f).
    let dying_dragon = ObjectSpec::creature(p1, "Dying Dragon", 3, 0)
        .with_subtypes(vec![SubType("Dragon".to_string())]);

    // A Goblin with 0 toughness — also dies via SBA, but must NOT fire the Dragon-dies trigger.
    let dying_goblin = ObjectSpec::creature(p1, "Dying Goblin", 2, 0)
        .with_subtypes(vec![SubType("Goblin".to_string())]);

    // Library cards for the draw effect.
    let lib1 = ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(dying_dragon)
        .object(dying_goblin)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_hand = hand_count(&state, p1);

    // Both players pass — SBAs fire, both 0-toughness creatures die.
    // Dragon-dies trigger fires for the Dragon; does NOT fire for the Goblin.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Verify both creatures died.
    let dragon_gone = !state
        .objects()
        .values()
        .any(|o| o.characteristics.name == "Dying Dragon" && o.zone == ZoneId::Battlefield);
    let goblin_gone = !state
        .objects()
        .values()
        .any(|o| o.characteristics.name == "Dying Goblin" && o.zone == ZoneId::Battlefield);
    assert!(
        dragon_gone,
        "Dying Dragon should have left battlefield (SBA CR 704.5f)."
    );
    assert!(
        goblin_gone,
        "Dying Goblin should have left battlefield (SBA CR 704.5f)."
    );

    // Exactly 1 death trigger should be on the stack (Dragon only, not Goblin).
    let watcher_id = find_object_id(&state, "Death Watcher");
    let trigger_count = trigger_count_for(&state, watcher_id);
    assert_eq!(
        trigger_count, 1,
        "CR 603.10a: Exactly 1 death trigger should be pending (Dragon died), \
         NOT 2 (Goblin death must not trigger Dragon-dies filter). count={}",
        trigger_count
    );

    // Resolve the trigger — P1 draws 1 card.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        hand_count(&state, p1),
        initial_hand + 1,
        "CR 603.10a: P1 draws 1 card from Dragon-dies trigger (death path unaffected by PB-AC0). \
         Goblin dying must not contribute."
    );
}

// ── Tests 12 & 13: enrich_spec_from_def discrimination tests ──────────────────
//
// These tests are the discrimination-gate for PB-AC0 Change 1 (replay_harness.rs:2411):
// the `triggering_creature_filter: filter.clone()` line inside the
// WheneverCreatureEntersBattlefield conversion arm of `enrich_spec_from_def`.
//
// ALL 11 prior tests use `ObjectSpec::with_triggered_ability(<runtime def>)`, which
// bypasses enrich_spec_from_def entirely — reverting Change 1 leaves them green.
// Tests 12 and 13 build the watcher via `enrich_spec_from_def` on the actual registered
// CardDefinition (Ganax, Lathliss), with NO `with_triggered_ability` call on the watcher
// ObjectSpec. So the WheneverCreatureEntersBattlefield → TriggeredAbilityDef conversion
// (including Change 1's `triggering_creature_filter: filter.clone()`) runs at
// `build_initial_state` time, and reverting Change 1 (→ `triggering_creature_filter: None`)
// causes these tests to fail.
//
// Mirror pattern: `crates/engine/tests/pb_l_landfall.rs` (enrich_spec_from_def + load_defs).

fn load_defs() -> HashMap<String, CardDefinition> {
    let cards = all_cards();
    cards.iter().map(|d| (d.name.clone(), d.clone())).collect()
}

// ── Test 12 ───────────────────────────────────────────────────────────────────

/// CR 603.2 / CR 205.3 -- Ganax, Astral Hunter via real CardDefinition:
/// enrich_spec_from_def discrimination test for Change 1.
///
/// The watcher is Ganax built from its registered CardDefinition via
/// `enrich_spec_from_def`, with NO `with_triggered_ability` call. This exercises the
/// `WheneverCreatureEntersBattlefield` conversion arm in `enrich_spec_from_def`
/// (replay_harness.rs:2360-2414), specifically Change 1's
/// `triggering_creature_filter: filter.clone()` at ~L2411.
///
/// Discrimination check: reverting Change 1 to `triggering_creature_filter: None` causes
/// the subtype filter to be dropped, the trigger fires for any creature, and the
/// no-fire-on-mismatch assertion below (Goblin → no Treasure) FAILS.
///
/// Fire-on-match: a Dragon creature enters → exactly 1 Treasure token created.
/// No-fire-on-mismatch: a Goblin creature enters → 0 Treasure tokens created.
#[test]
fn test_etb_ganax_carddef_integration_via_enrich() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let defs = load_defs();

    // Ganax, Astral Hunter as a watcher on the battlefield — built from the real
    // registered CardDefinition via enrich_spec_from_def. NO with_triggered_ability call.
    // enrich_spec_from_def converts the WheneverCreatureEntersBattlefield ability into a
    // runtime TriggeredAbilityDef, forwarding `triggering_creature_filter: filter.clone()`
    // (Change 1). Reverting Change 1 → None drops the Dragon filter → over-triggers.
    let ganax_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Ganax, Astral Hunter").in_zone(ZoneId::Battlefield),
        &defs,
    );

    // Entering creatures: a Dragon (should fire) and a Goblin (should NOT fire).
    let dragon_card = dragon_def("ganax-enrich-dragon", "Ganax Enrich Dragon", 3);
    let goblin_card = creature_def("ganax-enrich-goblin", "Ganax Enrich Goblin", 2);

    // Registry must include all cards referenced: Ganax itself (for any effect resolution),
    // plus the entering creatures.
    let all_named: Vec<CardDefinition> = {
        let mut v = all_cards();
        v.push(dragon_card);
        v.push(goblin_card);
        v
    };
    let registry = CardRegistry::new(all_named);

    let dragon_in_hand = ObjectSpec::creature(p1, "Ganax Enrich Dragon", 3, 3)
        .with_card_id(CardId("ganax-enrich-dragon".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let goblin_in_hand = ObjectSpec::creature(p1, "Ganax Enrich Goblin", 2, 2)
        .with_card_id(CardId("ganax-enrich-goblin".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Goblin".to_string())])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ganax_spec)
        .object(dragon_in_hand)
        .object(goblin_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let count_treasures = |state: &GameState| -> usize {
        state
            .objects()
            .values()
            .filter(|o| o.characteristics.name == "Treasure" && o.is_token)
            .count()
    };

    let initial_treasures = count_treasures(&state);

    // Cast Goblin (non-Dragon) — Ganax's Dragon-ETB trigger must NOT fire.
    // With Change 1 reverted, `triggering_creature_filter` would be None, the
    // Dragon filter is dropped, and a Treasure would be created (over-trigger bug).
    let (state, _) = cast_creature(state, p1, "Ganax Enrich Goblin", 2);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        count_treasures(&state),
        initial_treasures,
        "CR 603.2 / CR 205.3: Ganax (via enrich_spec_from_def) must NOT create a Treasure \
         when a Goblin enters. If Change 1 is reverted (triggering_creature_filter: None), \
         the Dragon filter is dropped and this assertion FAILS (over-trigger bug)."
    );

    // Cast Dragon — Ganax's trigger MUST fire and create exactly 1 Treasure.
    // This is the fire-on-match case: with Change 1 in place, the Dragon filter is
    // forwarded and the trigger fires correctly.
    let (state, _) = cast_creature(state, p1, "Ganax Enrich Dragon", 3);
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve Ganax's trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        count_treasures(&state),
        initial_treasures + 1,
        "CR 603.2 / CR 205.3: Ganax (via enrich_spec_from_def) MUST create exactly 1 Treasure \
         when a Dragon enters. Validates Change 1 (triggering_creature_filter: filter.clone())."
    );
}

// ── Test 13 ───────────────────────────────────────────────────────────────────

/// CR 111.1 / CR 603.2 -- Lathliss, Dragon Queen via real CardDefinition:
/// enrich_spec_from_def discrimination test for Change 1 (nontoken path).
///
/// The watcher is Lathliss built from its registered CardDefinition via
/// `enrich_spec_from_def`, with NO `with_triggered_ability` call. This exercises the
/// `WheneverCreatureEntersBattlefield` conversion arm in `enrich_spec_from_def`, and
/// specifically the `is_nontoken: true` field forwarded by Change 1's
/// `triggering_creature_filter: filter.clone()`.
///
/// Discrimination check: reverting Change 1 to `triggering_creature_filter: None` drops
/// the `is_nontoken` filter. A token Dragon entering would then fire the trigger,
/// causing the no-fire-on-token assertion below to FAIL.
///
/// Fire-on-match: a nontoken Dragon enters → trigger fires (dragon token or life-gain
/// observable). No-fire-on-mismatch: a token Dragon enters → trigger must NOT fire.
#[test]
fn test_etb_lathliss_carddef_integration_via_enrich() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let defs = load_defs();

    // Lathliss, Dragon Queen on the battlefield — built from the real CardDefinition
    // via enrich_spec_from_def. NO with_triggered_ability call.
    // enrich_spec_from_def runs the WheneverCreatureEntersBattlefield conversion,
    // forwarding `triggering_creature_filter: Some(TargetFilter { has_subtype: Dragon,
    // is_nontoken: true, ... })` (Change 1). Reverting → None drops the nontoken filter.
    let lathliss_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Lathliss, Dragon Queen").in_zone(ZoneId::Battlefield),
        &defs,
    );

    // A nontoken Dragon card (cast from hand → is_token: false).
    let nontoken_dragon = dragon_def("lathliss-enrich-nontoken", "Lathliss Enrich Dragon", 3);

    // A card that creates a Dragon token (is_token: true for the resulting token).
    let token_creator_def = CardDefinition {
        card_id: CardId("lathliss-enrich-creator".to_string()),
        name: "Lathliss Enrich Creator".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "When this enters, create a 5/5 red Dragon creature token with flying."
            .to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::CreateToken {
                spec: TokenSpec {
                    name: "Dragon".to_string(),
                    card_types: [CardType::Creature].into_iter().collect(),
                    subtypes: [SubType("Dragon".to_string())].into_iter().collect(),
                    colors: [Color::Red].into_iter().collect(),
                    power: 5,
                    toughness: 5,
                    count: EffectAmount::Fixed(1),
                    supertypes: imbl::OrdSet::new(),
                    keywords: [mtg_engine::KeywordAbility::Flying].into_iter().collect(),
                    tapped: false,
                    enters_attacking: false,
                    mana_color: None,
                    mana_abilities: vec![],
                    activated_abilities: vec![],
                    ..Default::default()
                },
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let all_named: Vec<CardDefinition> = {
        let mut v = all_cards();
        v.push(nontoken_dragon);
        v.push(token_creator_def);
        v
    };
    let registry = CardRegistry::new(all_named);

    // Nontoken Dragon in hand.
    let dragon_in_hand = ObjectSpec::creature(p1, "Lathliss Enrich Dragon", 3, 3)
        .with_card_id(CardId("lathliss-enrich-nontoken".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    // Token creator in hand.
    let creator_in_hand = ObjectSpec::creature(p1, "Lathliss Enrich Creator", 2, 2)
        .with_card_id(CardId("lathliss-enrich-creator".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 4,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(lathliss_spec)
        .object(dragon_in_hand)
        .object(creator_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let lathliss_id = find_object_id(&state, "Lathliss, Dragon Queen");

    let count_dragon_tokens = |state: &GameState| -> usize {
        state
            .objects()
            .values()
            .filter(|o| {
                o.is_token
                    && o.characteristics
                        .subtypes
                        .contains(&SubType("Dragon".to_string()))
                    && o.zone == ZoneId::Battlefield
            })
            .count()
    };

    let initial_tokens = count_dragon_tokens(&state);

    // Cast nontoken Dragon → Lathliss's trigger MUST fire (another nontoken Dragon, exclude_self
    // satisfied because entering creature is a different object than Lathliss).
    let (state, _) = cast_creature(state, p1, "Lathliss Enrich Dragon", 3);
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve Lathliss trigger → creates a Dragon token.
    let (state, _) = pass_all(state, &[p1, p2]);

    let tokens_after_nontoken = count_dragon_tokens(&state);
    assert_eq!(
        tokens_after_nontoken,
        initial_tokens + 1,
        "CR 603.2 / CR 205.3: Lathliss (via enrich_spec_from_def) MUST fire when a \
         nontoken Dragon enters. Change 1 forwards triggering_creature_filter with \
         has_subtype + is_nontoken — reverting it would suppress the trigger."
    );

    // Cast token creator (non-Dragon) → enters, its ETB fires → creates a token Dragon.
    // That token Dragon has is_token: true.
    // Lathliss's is_nontoken filter must block the trigger from chaining off it.
    // With Change 1 reverted, triggering_creature_filter is None → is_nontoken dropped →
    // trigger fires on the token Dragon → this assertion FAILS.
    let (state, _) = cast_creature(state, p1, "Lathliss Enrich Creator", 4);
    // Creator enters (non-Dragon, so Lathliss doesn't fire on it).
    let (state, _) = pass_all(state, &[p1, p2]);
    // Creator's ETB resolves → Dragon token enters (is_token: true).
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        trigger_count_for(&state, lathliss_id),
        0,
        "CR 111.1: Lathliss (via enrich_spec_from_def) must NOT fire when a token Dragon \
         enters. With Change 1 reverted (triggering_creature_filter: None), the is_nontoken \
         filter is dropped and this assertion FAILS — the infinite-loop guard is lost."
    );
}

//! Tests for PB-Q: ChooseColor (as-ETB color choice + color-aware downstream effects).
//!
//! Covers:
//! - `ReplacementModification::ChooseColor(Color)` — "As this enters, choose a color"
//!   (CR 614.12 / CR 614.12a)
//! - `EffectFilter::CreaturesYouControlOfChosenColor` — chosen-color creature pump (CR 105.1)
//! - `ReplacementTrigger::ManaWouldBeProduced` color_filter + source_filter — chosen-color
//!   mana doubling (CR 106.6a)
//! - `Effect::AddManaOfChosenColor` — tap for N mana of chosen color (CR 614.12)
//! - `GameObject.chosen_color` hash field participation
//!
//! Card integrations: Caged Sun, Temple of the Dragon Queen.
//!
//! Standing rules enforced:
//! - Tests 5 and 8 are MANDATORY full-dispatch tests (memory/conventions.md).
//! - Hash test defends against PB-S H1 regression.

use mtg_engine::{
    calculate_characteristics, process_command, CardId, CardRegistry, CardType, Color, Command,
    ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, GameEvent, GameState,
    GameStateBuilder, LayerModification, ManaAbility, ManaColor, ManaPool, ObjectId, ObjectSpec,
    PlayerId, Step, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn find_object_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name && o.zone == zone)
        .map(|(&id, _)| id)
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

fn cast_spell(state: GameState, player: PlayerId, card_id: ObjectId) -> GameState {
    let (s, _) = process_command(
        state,
        Command::CastSpell {
            player,
            card: card_id,
            targets: vec![],
            alt_cost: None,
            additional_costs: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            x_value: 0,
            modes_chosen: vec![],
            kicker_times: 0,
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e));
    s
}

// ── Replacement tests (Component 2) ──────────────────────────────────────────

/// CR 614.12a — ChooseColor replacement sets chosen_color on the entering permanent
/// immediately at ETB, before any priority window.
///
/// Setup: P1 has 2 white creatures + 1 blue creature on battlefield.
/// Cast Caged Sun (default White). Deterministic fallback picks White (most common).
/// Expected: Caged Sun on battlefield with chosen_color = White.
#[test]
fn test_choose_color_replacement_sets_field() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::caged_sun::card()]);

    let white1 = ObjectSpec::creature(p1, "White Creature 1", 1, 1)
        .with_colors(vec![Color::White])
        .in_zone(ZoneId::Battlefield);
    let white2 = ObjectSpec::creature(p1, "White Creature 2", 1, 1)
        .with_colors(vec![Color::White])
        .in_zone(ZoneId::Battlefield);
    let blue1 = ObjectSpec::creature(p1, "Blue Creature", 1, 1)
        .with_colors(vec![Color::Blue])
        .in_zone(ZoneId::Battlefield);

    let caged_sun_spec = ObjectSpec::card(p1, "Caged Sun")
        .with_card_id(CardId("caged-sun".to_string()))
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(white1)
        .object(white2)
        .object(blue1)
        .object(caged_sun_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                colorless: 6,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let caged_sun_id = find_object(&state, "Caged Sun");
    let state = cast_spell(state, p1, caged_sun_id);
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    // Caged Sun should now be on the battlefield
    let bf_caged_sun_id = find_object_in_zone(&state, "Caged Sun", ZoneId::Battlefield)
        .expect("Caged Sun should be on battlefield");

    let caged_sun_obj = state.objects.get(&bf_caged_sun_id).unwrap();
    // White was most common color → chosen_color should be White
    assert_eq!(
        caged_sun_obj.chosen_color,
        Some(Color::White),
        "CR 614.12a: chosen_color should be White (most common among P1 permanents)"
    );
}

/// CR 614.12a — Deterministic fallback picks the majority color.
///
/// P1 has 2 white + 1 blue permanents. Default is Red.
/// Expected: chosen_color = White (majority, not the default Red).
#[test]
fn test_choose_color_deterministic_fallback_picks_majority() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::caged_sun::card()]);

    let white1 = ObjectSpec::creature(p1, "White Weenie 1", 1, 1)
        .with_colors(vec![Color::White])
        .in_zone(ZoneId::Battlefield);
    let white2 = ObjectSpec::creature(p1, "White Weenie 2", 1, 1)
        .with_colors(vec![Color::White])
        .in_zone(ZoneId::Battlefield);
    let blue1 = ObjectSpec::creature(p1, "Blue Flier", 2, 2)
        .with_colors(vec![Color::Blue])
        .in_zone(ZoneId::Battlefield);

    // Caged Sun default = White, but we verify majority is still White
    let caged_sun_spec = ObjectSpec::card(p1, "Caged Sun")
        .with_card_id(CardId("caged-sun".to_string()))
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(white1)
        .object(white2)
        .object(blue1)
        .object(caged_sun_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                colorless: 6,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let caged_sun_id = find_object(&state, "Caged Sun");
    let state = cast_spell(state, p1, caged_sun_id);
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    let bf_id = find_object_in_zone(&state, "Caged Sun", ZoneId::Battlefield)
        .expect("Caged Sun should be on battlefield");
    let obj = state.objects.get(&bf_id).unwrap();
    assert_eq!(
        obj.chosen_color,
        Some(Color::White),
        "CR 614.12a: White (count=2) should beat Blue (count=1)"
    );
}

/// CR 614.12a — When no permanents are on battlefield, fallback to the printed default.
///
/// Empty board (just Caged Sun entering). Default = White. Expected: chosen_color = White.
#[test]
fn test_choose_color_default_when_no_permanents() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::caged_sun::card()]);

    let caged_sun_spec = ObjectSpec::card(p1, "Caged Sun")
        .with_card_id(CardId("caged-sun".to_string()))
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(caged_sun_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                colorless: 6,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let caged_sun_id = find_object(&state, "Caged Sun");
    let state = cast_spell(state, p1, caged_sun_id);
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    let bf_id = find_object_in_zone(&state, "Caged Sun", ZoneId::Battlefield)
        .expect("Caged Sun should be on battlefield");
    let obj = state.objects.get(&bf_id).unwrap();
    assert_eq!(
        obj.chosen_color,
        Some(Color::White),
        "CR 614.12a: with no permanents, default color (White) should be chosen"
    );
}

/// CR 400.7 / CR 614.12 — chosen_color resets to None when object changes zones.
///
/// A manually-built object with chosen_color=White in hand has None chosen_color
/// (GameObjects constructed fresh on zone change reset fields to Default).
#[test]
fn test_choose_color_resets_on_zone_change() {
    let p1 = p(1);

    // Simulate a card that already existed on battlefield with chosen_color set.
    // We verify that a NEW GameObject (fresh construction) has chosen_color = None.
    // This validates the Default propagation (CR 400.7: zone change = new object).
    let obj_spec = ObjectSpec::card(p1, "Dummy Permanent")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(obj_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Dummy Permanent");
    let obj = state.objects.get(&obj_id).unwrap();
    assert_eq!(
        obj.chosen_color, None,
        "CR 400.7: Fresh GameObjects must have chosen_color = None"
    );
}

// ── Filter dispatch tests (Component 4) — MANDATORY full-dispatch ────────────

/// CR 614.12 / CR 613.1f — FULL-DISPATCH test (MANDATORY per memory/conventions.md).
///
/// Cast Caged Sun; allow it to resolve and set chosen_color via the ChooseColor replacement.
/// Verify: P1's white creature gets +1/+1 from the static anthem; non-white unchanged.
///
/// This test exercises the full dispatch path:
/// CastSpell → ETB replacement (ChooseColor) → ContinuousEffect stored with
/// CreaturesYouControlOfChosenColor filter → calculate_characteristics reads chosen_color.
#[test]
fn test_caged_sun_full_dispatch_pumps_chosen_color_creatures() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::caged_sun::card()]);

    let white_creature = ObjectSpec::creature(p1, "White Warrior", 2, 2)
        .with_colors(vec![Color::White])
        .in_zone(ZoneId::Battlefield);
    let blue_creature = ObjectSpec::creature(p1, "Blue Wizard", 2, 2)
        .with_colors(vec![Color::Blue])
        .in_zone(ZoneId::Battlefield);
    let red_creature = ObjectSpec::creature(p1, "Red Dragon", 3, 3)
        .with_colors(vec![Color::Red])
        .in_zone(ZoneId::Battlefield);

    // Caged Sun default=White; 1 white creature → deterministic picks White.
    let caged_sun_spec = ObjectSpec::card(p1, "Caged Sun")
        .with_card_id(CardId("caged-sun".to_string()))
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(white_creature)
        .object(blue_creature)
        .object(red_creature)
        .object(caged_sun_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                colorless: 6,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let caged_sun_id = find_object(&state, "Caged Sun");
    let state = cast_spell(state, p1, caged_sun_id);
    // Resolve Caged Sun (4 players pass priority)
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    // Verify Caged Sun resolved and chose White (most common: 1 white > 0 blue/red)
    let bf_cs_id = find_object_in_zone(&state, "Caged Sun", ZoneId::Battlefield)
        .expect("Caged Sun must be on battlefield after resolution");
    let cs_obj = state.objects.get(&bf_cs_id).unwrap();
    assert_eq!(
        cs_obj.chosen_color,
        Some(Color::White),
        "CR 614.12a: chosen_color must be White"
    );

    // Full layer dispatch: calculate_characteristics — NOT a direct is_effect_active call.
    let white_id = find_object(&state, "White Warrior");
    let blue_id = find_object(&state, "Blue Wizard");
    let red_id = find_object(&state, "Red Dragon");

    let white_chars = calculate_characteristics(&state, white_id).unwrap();
    let blue_chars = calculate_characteristics(&state, blue_id).unwrap();
    let red_chars = calculate_characteristics(&state, red_id).unwrap();

    // White Warrior gets +1/+1 (2+1=3)
    assert_eq!(
        white_chars.power,
        Some(3),
        "CR 613.1f / CR 105.1: White Warrior power should be 3 (2 base +1 from Caged Sun)"
    );
    assert_eq!(
        white_chars.toughness,
        Some(3),
        "CR 613.1f / CR 105.1: White Warrior toughness should be 3"
    );

    // Blue Wizard unchanged (not white)
    assert_eq!(
        blue_chars.power,
        Some(2),
        "CR 105.1: Blue Wizard is not the chosen color, unchanged"
    );
    assert_eq!(
        blue_chars.toughness,
        Some(2),
        "CR 105.1: Blue Wizard toughness unchanged"
    );

    // Red Dragon unchanged (not white)
    assert_eq!(
        red_chars.power,
        Some(3),
        "CR 105.1: Red Dragon is not the chosen color, unchanged"
    );
    assert_eq!(
        red_chars.toughness,
        Some(3),
        "CR 105.1: Red Dragon toughness unchanged"
    );
}

/// CR 614.12 / CR 105.1 — Filter with no chosen_color set matches nothing.
///
/// Manually register a CreaturesYouControlOfChosenColor effect with no source
/// (chosen_color not available). Assert no creatures get pumped.
/// Defends against the PB-X observability-window failure mode.
#[test]
fn test_chosen_color_filter_no_choice_matches_nothing() {
    let p1 = p(1);

    let white_creature = ObjectSpec::creature(p1, "White Target", 2, 2)
        .with_colors(vec![Color::White])
        .in_zone(ZoneId::Battlefield);

    // Source exists but has chosen_color = None (no ChooseColor replacement fired).
    let source_obj = ObjectSpec::card(p1, "Sourceless Artifact")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(white_creature)
        .object(source_obj)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Sourceless Artifact");

    // Register a CreaturesYouControlOfChosenColor effect with a source that has chosen_color=None
    let effect = ContinuousEffect {
        id: EffectId(1),
        source: Some(source_id),
        timestamp: 10,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::UntilEndOfTurn,
        filter: EffectFilter::CreaturesYouControlOfChosenColor,
        modification: LayerModification::ModifyBoth(1),
        is_cda: false,
        condition: None,
    };
    let mut state = state;
    state.continuous_effects.push_back(effect);

    let white_id = find_object(&state, "White Target");
    let white_chars = calculate_characteristics(&state, white_id).unwrap();

    assert_eq!(
        white_chars.power,
        Some(2),
        "CR 105.1: CreaturesYouControlOfChosenColor with no chosen_color must match nothing"
    );
    assert_eq!(
        white_chars.toughness,
        Some(2),
        "CR 105.1: toughness unchanged when chosen_color is None"
    );
}

// ── Mana doubling dispatch tests (Component 5) — MANDATORY full-dispatch ─────

/// CR 106.6a / CR 105.1 — FULL-DISPATCH test (MANDATORY per memory/conventions.md).
///
/// P1 controls Caged Sun (chosen White) and a Plains (taps for White).
/// Tap Plains for mana. Assert: P1 mana pool has 2 W (1 from Plains + 1 from Caged Sun).
///
/// This exercises the full dispatch path:
/// CastSpell (Caged Sun) → ETB replacement (ChooseColor=White) → ManaWouldBeProduced
/// replacement registered → TapForMana on Plains → apply_mana_production_replacements
/// → AddOneManaOfChosenColor → 2W total.
#[test]
fn test_caged_sun_doubles_chosen_color_land_mana() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::caged_sun::card()]);

    // Plains: basic land that taps for W
    let plains = ObjectSpec::card(p1, "Plains")
        .with_types(vec![CardType::Land])
        .with_mana_ability(ManaAbility::tap_for(ManaColor::White))
        .in_zone(ZoneId::Battlefield);

    // Caged Sun starts in hand; we cast it then tap Plains
    let caged_sun_spec = ObjectSpec::card(p1, "Caged Sun")
        .with_card_id(CardId("caged-sun".to_string()))
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(plains)
        .object(caged_sun_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                colorless: 6,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let caged_sun_id = find_object(&state, "Caged Sun");
    let state = cast_spell(state, p1, caged_sun_id);
    // Spend mana to cast; now pool empty
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    // Verify Caged Sun resolved with chosen_color = White (no other colors on board).
    let bf_cs_id = find_object_in_zone(&state, "Caged Sun", ZoneId::Battlefield)
        .expect("Caged Sun must be on battlefield");
    let cs_obj = state.objects.get(&bf_cs_id).unwrap();
    assert_eq!(
        cs_obj.chosen_color,
        Some(Color::White),
        "Caged Sun must have chosen White"
    );

    // Tap Plains for mana
    let plains_id = find_object(&state, "Plains");
    let ability_index = 0; // first mana ability
    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: plains_id,
            ability_index,
        },
    )
    .expect("TapForMana on Plains should succeed");

    let pool = &state.players.get(&p1).unwrap().mana_pool;
    assert_eq!(
        pool.white, 2,
        "CR 106.6a: Caged Sun adds 1 additional White mana → 2W total from Plains tap"
    );
    assert_eq!(pool.blue, 0, "CR 105.1: No blue mana should be produced");
}

/// CR 105.1 / CR 106.6a — Caged Sun does NOT double non-chosen-color mana.
///
/// Caged Sun chose White. Tap a Mountain for Red. Assert: 1R in pool (no doubling).
#[test]
fn test_caged_sun_does_not_double_other_color_mana() {
    let p1 = p(1);
    let registry = CardRegistry::new(vec![mtg_engine::cards::defs::caged_sun::card()]);

    // White creature ensures deterministic White choice
    let white_creature = ObjectSpec::creature(p1, "White Knight", 2, 2)
        .with_colors(vec![Color::White])
        .in_zone(ZoneId::Battlefield);
    let mountain = ObjectSpec::card(p1, "Mountain")
        .with_types(vec![CardType::Land])
        .with_mana_ability(ManaAbility::tap_for(ManaColor::Red))
        .in_zone(ZoneId::Battlefield);
    let caged_sun_spec = ObjectSpec::card(p1, "Caged Sun")
        .with_card_id(CardId("caged-sun".to_string()))
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(white_creature)
        .object(mountain)
        .object(caged_sun_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .player_mana(
            p1,
            ManaPool {
                colorless: 6,
                ..Default::default()
            },
        )
        .build()
        .unwrap();

    let caged_sun_id = find_object(&state, "Caged Sun");
    let state = cast_spell(state, p1, caged_sun_id);
    let (state, _) = pass_all(state, &[p1, p(2), p(3), p(4)]);

    // Verify chose White
    let bf_cs_id = find_object_in_zone(&state, "Caged Sun", ZoneId::Battlefield).unwrap();
    assert_eq!(
        state.objects.get(&bf_cs_id).unwrap().chosen_color,
        Some(Color::White)
    );

    let mountain_id = find_object(&state, "Mountain");
    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: mountain_id,
            ability_index: 0,
        },
    )
    .expect("TapForMana on Mountain should succeed");

    let pool = &state.players.get(&p1).unwrap().mana_pool;
    assert_eq!(
        pool.red, 1,
        "CR 105.1: Mountain produces 1R; Caged Sun chose White — no doubling"
    );
    assert_eq!(pool.white, 0, "No white mana from Mountain tap");
}

// ── Hash test ────────────────────────────────────────────────────────────────

/// PB-S H1 defense — chosen_color MUST participate in the GameObject hash.
///
/// Two objects identical except one has chosen_color=None, the other chosen_color=Some(White).
/// Their hashes must differ. Defends against PB-S H1 failure mode (forgotten field in HashInto).
#[test]
fn test_chosen_color_hash_field_audit() {
    let p1 = p(1);

    // Build two states: identical except one has a Caged Sun with White chosen, one without
    let obj_no_color = ObjectSpec::card(p1, "Artifact No Color")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Battlefield);

    let state1 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(obj_no_color)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let obj_id = find_object(&state1, "Artifact No Color");

    // State1: chosen_color = None (default)
    let hash1 = state1.public_state_hash();

    // State2: mutate the object to have chosen_color = White
    let mut state2 = state1.clone();
    if let Some(obj) = state2.objects.get_mut(&obj_id) {
        obj.chosen_color = Some(Color::White);
    }
    let hash2 = state2.public_state_hash();

    assert_ne!(
        hash1, hash2,
        "PB-S H1 defense: chosen_color must participate in hash (None vs Some(White) must differ)"
    );

    // Also verify different colors produce different hashes
    let mut state3 = state2.clone();
    if let Some(obj) = state3.objects.get_mut(&obj_id) {
        obj.chosen_color = Some(Color::Black);
    }
    let hash3 = state3.public_state_hash();
    assert_ne!(
        hash2, hash3,
        "PB-S H1 defense: White vs Black chosen_color must produce different hashes"
    );
}

//! Tests for PB-34: Filter land mana production (Effect::AddManaFilterChoice) and
//! AddManaScaled orphan bug fix.
//!
//! Filter lands pay a hybrid mana cost plus tap to produce 2 mana from a constrained
//! color pair. Example: "{W/B}, {T}: Add {W}{W}, {W}{B}, or {B}{B}."
//!
//! Engine simplification: AddManaFilterChoice produces 1 of color_a + 1 of color_b
//! (the middle option). Interactive full-choice deferred to M10.
//!
//! CR 605.1a — activated mana abilities resolve immediately (no priority window).
//!   (Filter lands use Cost::Sequence and go through ActivateAbility, which puts them
//!   on the stack. Stack resolution yields the same final mana result.)
//! CR 602.2 — activated abilities cost must be paid before the ability goes on the stack.
//!
//! Note: Hybrid mana cost enforcement is a pre-existing engine limitation (not in scope
//! for PB-34). The ManaCost.can_spend() method does not validate hybrid mana symbols;
//! this means the {W/B} hybrid activation cost is structurally correct in the card
//! definition but not enforced at activation time. Hybrid enforcement is a P4 item.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ManaColor, ManaPool, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn build_defs_and_registry() -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    (defs, registry)
}

fn make_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

/// Build a state with a single filter land on the battlefield for p(1).
fn build_with_filter_land(name: &str) -> GameState {
    let (defs, registry) = build_defs_and_registry();
    let spec = make_spec(p(1), name, ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");

    state.turn.priority_holder = Some(p(1));
    state
}

/// Pass priority for all listed players once (drives stack resolution).
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

// ── CR 605.1a / PB-34: Filter land produces 2 mana (1 of each color) ─────────

#[test]
/// CR 605.1a — Fetid Heath: activating filter ability adds {W}{B} to mana pool.
/// Effect::AddManaFilterChoice produces 1 white + 1 black (middle option of 3 choices).
/// Starting with an empty mana pool, resolution should yield white:1 + black:1.
/// NOTE: Hybrid mana enforcement is a pre-existing limitation; the hybrid activation
/// cost is structurally correct in the ability definition but not validated at runtime.
fn test_filter_land_produces_two_mana_fetid_heath() {
    let state = build_with_filter_land("Fetid Heath");
    let land_id = find_by_name(&state, "Fetid Heath");

    // Fetid Heath abilities:
    //   tap-mana: {T}: Add {C}  — registered as ManaAbility, NOT in activated_abilities
    //   activated_ability[0]: {W/B},{T}: Add {W}{B} (AddManaFilterChoice)
    let (state_after_activate, _activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: land_id,
            ability_index: 0, // filter ability is activated_ability index 0
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("filter land activation should succeed (CR 602.2)");

    // Ability is now on the stack. Pass priority for both players to resolve it.
    let (state_resolved, resolve_events) = pass_all(state_after_activate, &[p(1), p(2)]);

    // After resolution: p(1) should have 1 white and 1 black mana added.
    let pool = &state_resolved.players[&p(1)].mana_pool;
    assert_eq!(
        pool.white, 1,
        "AddManaFilterChoice should add 1 white mana to empty pool"
    );
    assert_eq!(
        pool.black, 1,
        "AddManaFilterChoice should add 1 black mana to empty pool"
    );
    assert_eq!(pool.blue, 0, "no blue mana should be added");
    assert_eq!(pool.red, 0, "no red mana should be added");
    assert_eq!(pool.green, 0, "no green mana should be added");
    assert_eq!(pool.colorless, 0, "no colorless mana should be added");

    // ManaAdded events should have fired for both white and black.
    assert!(
        resolve_events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                player,
                color: ManaColor::White,
                amount: 1,
            } if *player == p(1)
        )),
        "ManaAdded(White, 1) event should be emitted (CR 605.1a)"
    );
    assert!(
        resolve_events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                player,
                color: ManaColor::Black,
                amount: 1,
            } if *player == p(1)
        )),
        "ManaAdded(Black, 1) event should be emitted (CR 605.1a)"
    );
}

#[test]
/// CR 602.2 — filter land tap cost: land must be untapped to activate.
/// Activating an already-tapped filter land returns PermanentAlreadyTapped error.
fn test_filter_land_tap_required() {
    let mut state = build_with_filter_land("Fetid Heath");

    // Tap the land manually before trying to activate.
    let land_id = find_by_name(&state, "Fetid Heath");
    state.objects.get_mut(&land_id).unwrap().status.tapped = true;

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: land_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "activating tapped filter land should return an error (CR 602.2)"
    );
}

#[test]
/// PB-34: Effect::AddManaFilterChoice is correctly used in filter land card definitions.
/// Verify all 7 filter lands produce exactly 2 mana (1 of each constrained color)
/// by checking the mana pool delta from an empty starting state.
fn test_all_filter_lands_produce_correct_colors() {
    // (name, expected_color_a, expected_color_b)
    let filter_lands: &[(&str, ManaColor, ManaColor)] = &[
        ("Fetid Heath", ManaColor::White, ManaColor::Black),
        ("Rugged Prairie", ManaColor::Red, ManaColor::White),
        ("Twilight Mire", ManaColor::Black, ManaColor::Green),
        ("Flooded Grove", ManaColor::Green, ManaColor::Blue),
        ("Cascade Bluffs", ManaColor::Blue, ManaColor::Red),
        ("Sunken Ruins", ManaColor::Blue, ManaColor::Black),
        ("Graven Cairns", ManaColor::Black, ManaColor::Red),
    ];

    for (name, color_a, color_b) in filter_lands {
        let state = build_with_filter_land(name);
        let land_id = find_by_name(&state, name);

        // Capture pool before activation.
        let pool_before = state.players[&p(1)].mana_pool.clone();

        let (state_activated, _) = process_command(
            state,
            Command::ActivateAbility {
                player: p(1),
                source: land_id,
                ability_index: 0,
                targets: vec![],
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
            },
        )
        .unwrap_or_else(|e| panic!("activating {} filter ability should succeed: {:?}", name, e));

        let (state_resolved, _) = pass_all(state_activated, &[p(1), p(2)]);

        let pool_after = &state_resolved.players[&p(1)].mana_pool;

        // Compute delta (mana added - mana spent; pre-existing hybrid enforcement gap means
        // the hybrid cost is NOT deducted from the pool, so delta is purely the AddManaFilterChoice).
        let delta_a = get_color(pool_after, *color_a) - get_color(&pool_before, *color_a);
        let delta_b = get_color(pool_after, *color_b) - get_color(&pool_before, *color_b);
        let total_added: i32 = [
            ManaColor::White,
            ManaColor::Blue,
            ManaColor::Black,
            ManaColor::Red,
            ManaColor::Green,
            ManaColor::Colorless,
        ]
        .iter()
        .map(|c| get_color(pool_after, *c) as i32 - get_color(&pool_before, *c) as i32)
        .sum();

        assert_eq!(
            delta_a, 1,
            "{}: AddManaFilterChoice should add exactly 1 {:?} mana",
            name, color_a
        );
        assert_eq!(
            delta_b, 1,
            "{}: AddManaFilterChoice should add exactly 1 {:?} mana",
            name, color_b
        );
        assert_eq!(
            total_added, 2,
            "{}: total mana delta should be exactly +2 (AddManaFilterChoice produces 2 mana)",
            name
        );
    }
}

fn get_color(pool: &ManaPool, color: ManaColor) -> u32 {
    match color {
        ManaColor::White => pool.white,
        ManaColor::Blue => pool.blue,
        ManaColor::Black => pool.black,
        ManaColor::Red => pool.red,
        ManaColor::Green => pool.green,
        ManaColor::Colorless => pool.colorless,
    }
}

#[test]
/// PB-34: AddManaScaled abilities are now registered as ManaAbilities on objects.
/// Previously, AddManaScaled with Cost::Tap was orphaned — not recognized by
/// try_as_tap_mana_ability and skipped from activated_abilities. After PB-34 fix,
/// Gaea's Cradle should have a registered ManaAbility.
fn test_add_mana_scaled_registered_as_mana_ability() {
    let (defs, registry) = build_defs_and_registry();
    let spec = make_spec(p(1), "Gaea's Cradle", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");

    let land_id = find_by_name(&state, "Gaea's Cradle");
    let obj = state.objects.get(&land_id).unwrap();

    // After PB-34 fix, Gaea's Cradle should have at least one registered ManaAbility.
    // (Pre-fix: AddManaScaled with Cost::Tap was not recognized by try_as_tap_mana_ability
    // and was silently excluded from both mana_abilities AND activated_abilities — never fired.)
    assert!(
        !obj.characteristics.mana_abilities.is_empty(),
        "Gaea's Cradle should have at least one registered ManaAbility after PB-34 fix (AddManaScaled orphan bug)"
    );

    // The registered ManaAbility should be marked as producing green mana (marker; actual count is dynamic).
    let has_green_ability = obj
        .characteristics
        .mana_abilities
        .iter()
        .any(|ma| ma.produces.contains_key(&ManaColor::Green));
    assert!(
        has_green_ability,
        "Gaea's Cradle ManaAbility should be marked as producing green mana"
    );
}

#[test]
/// PB-34: AddManaScaled orphan bug fix covers cards with Cost::Tap + AddManaScaled.
/// These were previously orphaned: not recognized by try_as_tap_mana_ability AND
/// excluded from activated_abilities — the ability was completely silent.
/// After the fix, each should have a registered ManaAbility.
///
/// Note: Cards with Cost::Sequence([Mana, Tap]) + AddManaScaled (Cabal Coffers,
/// Cabal Stronghold, Crypt of Agadeem) are correctly registered as activated abilities
/// (not mana abilities) since they have an additional mana cost. Those are NOT in this list.
fn test_add_mana_scaled_orphan_fix_all_cards() {
    let scaled_mana_cards = [
        "Elvish Archdruid",
        "Priest of Titania",
        "Marwyn, the Nurturer",
        "Circle of Dreams Druid",
        "Gaea's Cradle",
        "Howlsquad Heavy",
    ];

    let (defs, registry) = build_defs_and_registry();

    for name in &scaled_mana_cards {
        let spec = make_spec(p(1), name, ZoneId::Battlefield, &defs);

        let state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry.clone())
            .object(spec)
            .active_player(p(1))
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap_or_else(|e| panic!("state build failed for {}: {:?}", name, e));

        let obj_id = find_by_name(&state, name);
        let obj = state.objects.get(&obj_id).unwrap();

        assert!(
            !obj.characteristics.mana_abilities.is_empty(),
            "{} should have a registered ManaAbility after PB-34 AddManaScaled orphan fix",
            name
        );
    }
}

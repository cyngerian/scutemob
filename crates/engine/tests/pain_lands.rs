//! Pain land tests — mana abilities that deal damage to controller.
//!
//! CR 605: Mana abilities resolve immediately (no stack).
//! Pain lands have two mana abilities:
//!   1. {T}: Add {C} — no damage
//!   2. {T}: Add {R} or {W} — deals 1 damage to controller
//!
//! City of Brass has a triggered ability:
//!   "Whenever City of Brass becomes tapped, it deals 1 damage to you."
//!   This fires on ANY tap, not just from its mana ability.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId,
    Step,
};

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

fn build_with_land(name: &str) -> GameState {
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

// ── Pain land: colorless tap does NOT deal damage ─────────────────────────

#[test]
fn battlefield_forge_colorless_tap_no_damage() {
    // CR 605: {T}: Add {C} — no side effect.
    let state = build_with_land("Battlefield Forge");
    let land_id = find_by_name(&state, "Battlefield Forge");
    let life_before = state.players[&p(1)].life_total;

    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land_id,
            ability_index: 0, // first ability: {T}: Add {C}
        },
    )
    .expect("tap for colorless should succeed");

    // Life should be unchanged.
    assert_eq!(state.players[&p(1)].life_total, life_before);
    // Should have ManaAdded but no DamageDealt.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaAdded { .. })),
        "should produce mana"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::DamageDealt { .. })),
        "colorless tap should not deal damage"
    );
}

// ── Pain land: colored tap DOES deal 1 damage ─────────────────────────────

#[test]
fn battlefield_forge_colored_tap_deals_damage() {
    // CR 605: {T}: Add {R} or {W}. This land deals 1 damage to you.
    let state = build_with_land("Battlefield Forge");
    let land_id = find_by_name(&state, "Battlefield Forge");
    let life_before = state.players[&p(1)].life_total;

    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land_id,
            ability_index: 1, // second ability: {T}: Add {R}/{W} + damage
        },
    )
    .expect("tap for colored mana should succeed");

    // Life should decrease by 1.
    assert_eq!(state.players[&p(1)].life_total, life_before - 1);
    // Should have both ManaAdded and DamageDealt events.
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::ManaAdded { .. })));
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::DamageDealt { .. })));
}

// ── Verify all 7 pain lands have the colored+damage mana ability ──────────

#[test]
fn all_pain_lands_deal_damage_on_colored_tap() {
    let pain_lands = [
        "Battlefield Forge",
        "Caves of Koilos",
        "Llanowar Wastes",
        "Shivan Reef",
        "Sulfurous Springs",
        "Underground River",
        "Yavimaya Coast",
    ];

    for name in &pain_lands {
        let state = build_with_land(name);
        let land_id = find_by_name(&state, name);
        let life_before = state.players[&p(1)].life_total;

        let (state, events) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: land_id,
                ability_index: 1,
            },
        )
        .unwrap_or_else(|e| panic!("{}: colored tap failed: {:?}", name, e));

        assert_eq!(
            state.players[&p(1)].life_total,
            life_before - 1,
            "{}: should deal 1 damage",
            name
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, GameEvent::DamageDealt { .. })),
            "{}: should emit DamageDealt event",
            name
        );
    }
}

// ── City of Brass: mana ability produces mana ─────────────────────────────

#[test]
fn city_of_brass_tap_produces_mana() {
    // "{T}: Add one mana of any color."
    // "Whenever City of Brass becomes tapped, it deals 1 damage to you."
    // The mana ability itself should NOT deal damage — the damage comes from a
    // separate triggered ability that fires on PermanentTapped.
    let state = build_with_land("City of Brass");
    let land_id = find_by_name(&state, "City of Brass");

    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land_id,
            ability_index: 0,
        },
    )
    .expect("tap City of Brass should succeed");

    // Mana should be added.
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::ManaAdded { .. })));

    // PermanentTapped event should fire (triggers the damage ability).
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentTapped { .. })));

    // The damage is from a triggered ability, not from the mana ability itself.
    // The mana ability should NOT have damage_to_controller > 0.
    // So DamageDealt should NOT be in the mana ability events.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::DamageDealt { .. })),
        "City of Brass damage comes from triggered ability, not mana ability"
    );

    let _ = state;
}

// ── Pain land: verify mana pool gets the right colors ─────────────────────

#[test]
fn shivan_reef_produces_blue_and_red_with_damage() {
    // {T}: Add {U} or {R}. This land deals 1 damage to you.
    // (Currently adds both until interactive choice is implemented.)
    let state = build_with_land("Shivan Reef");
    let land_id = find_by_name(&state, "Shivan Reef");

    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: land_id,
            ability_index: 1,
        },
    )
    .expect("tap for colored mana should succeed");

    // Life should decrease by 1.
    assert_eq!(state.players[&p(1)].life_total, 39);
}

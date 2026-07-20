//! Tests for PB-RS2: activated-cost and mana-ability hybrid/Phyrexian pip payment
//! (OOS-RS-2 + OOS-OS8-1).
//!
//! STEP 0 PROBE (written before any production edit — see `memory/primitive-wip.md`
//! for the recorded pre-fix `cargo test` output). Both probes assert `Ok(_)` here,
//! documenting that a `{B/R}`-cost activation currently succeeds for FREE with an
//! empty mana pool — CR 107.4e/602.2b violated. Once the fix lands (Command schema
//! gains `hybrid_choices`/`phyrexian_life_payments`, and `handle_activate_ability` /
//! `handle_tap_for_mana` flatten before paying), these are inverted to
//! `Err(InsufficientMana)`-asserting, permanently-kept regression tests.

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, ActivatedAbility,
    ActivationCost, CardDefinition, CardRegistry, Command, Effect, EffectAmount, GameState,
    GameStateBuilder, HybridMana, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId,
    PlayerTarget, Step, ZoneId,
};
use std::collections::HashMap;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object {name:?} not found"))
}

/// PB-RS2 §0.1: probe A, `abilities.rs` (stack-using activated ability) path.
#[test]
fn probe_hybrid_pip_is_currently_free_activated_ability() {
    let source_spec =
        ObjectSpec::artifact(p(1), "Test Filter Rock").with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                mana_cost: Some(ManaCost {
                    hybrid: vec![HybridMana::ColorColor(ManaColor::Black, ManaColor::Red)],
                    ..Default::default()
                }),
                ..Default::default()
            },
            effect: Some(Effect::GainLife {
                amount: EffectAmount::Fixed(1),
                player: PlayerTarget::Controller,
            }),
            ..Default::default()
        });

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(source_spec)
        .build()
        .expect("state builds");

    let source = find_by_name(&state, "Test Filter Rock");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    );

    assert!(
        result.is_ok(),
        "OOS-RS-2 pre-fix probe: a {{B/R}} activated-ability cost with an EMPTY mana pool \
         currently activates for FREE (CR 107.4e violated). If this now fails, the bug is \
         already gone somewhere this plan did not find — stop and re-scope: {result:?}"
    );
}

/// PB-RS2 §0.2: probe B, `mana.rs` (mana ability) path — the one that covers the 7
/// shipped filter lands.
#[test]
fn probe_hybrid_pip_is_currently_free_mana_ability() {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    let land = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Graven Cairns")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Graven Cairns")),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(land)
        .build()
        .expect("state builds");

    let source = find_by_name(&state, "Graven Cairns");
    let result = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source,
            ability_index: 1, // {B/R},{T}: Add {B}{B}/{B}{R}/{R}{R} — ability 0 is {T}: Add {C}
            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    );

    let pool_total = result
        .as_ref()
        .ok()
        .map(|(s, _)| s.player(p(1)).unwrap().mana_pool.total())
        .unwrap_or(0);
    assert!(
        result.is_ok() && pool_total == 2,
        "OOS-RS-2 pre-fix probe: Graven Cairns's {{B/R}} filter ability with an EMPTY pool \
         currently produces 2 mana from NOTHING (a live shipped-card bug). If this now fails \
         or produces a different amount, the bug is already gone somewhere this plan did not \
         find — stop and re-scope: {result:?}, pool_total={pool_total}"
    );
}

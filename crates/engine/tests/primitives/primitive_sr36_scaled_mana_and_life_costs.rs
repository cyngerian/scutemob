//! Tests for SR-36 (`scutemob-92`): SF-8 (`{T}` + `AddManaScaled` produced exactly 1 mana,
//! not the scaled amount) and SF-9 (`Cost::PayLife` was silently unpaid for non-mana
//! activated abilities). Findings: `memory/card-authoring/sr34-engine-findings-2026-07-17.md`.
//!
//! SF-8 existed *because* the two tests that touched `AddManaScaled` asserted registration
//! shape (`mana_abilities.len() == 1`) and never activated — the finding says so in its own
//! words: "a data-model test can pin a defect as a requirement". Every test in this file
//! activates the ability and asserts the mana produced / life total, never shape alone,
//! per `tests/core/effect_choose_gate.rs` (SR-33) and `primitive_sr34_composite_mana_costs.rs`
//! (SR-34).

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AbilityDefinition,
    CardDefinition, CardId, CardRegistry, Command, Cost, Effect, EffectAmount, GameEvent,
    GameState, GameStateBuilder, GameStateError, ManaColor, ObjectId, ObjectSpec, PlayerId,
    PlayerTarget, Step, SubType, ZoneId,
};

// ── Helpers (duplicated per-file per SR-9a convention — see primitive_sr34_composite_mana_costs.rs) ──

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn defs_map() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
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

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn pool_amount(state: &GameState, player: PlayerId, color: ManaColor) -> u32 {
    let pool = &state.player(player).expect("player exists").mana_pool;
    match color {
        ManaColor::White => pool.white,
        ManaColor::Blue => pool.blue,
        ManaColor::Black => pool.black,
        ManaColor::Red => pool.red,
        ManaColor::Green => pool.green,
        ManaColor::Colorless => pool.colorless,
    }
}

/// Find the index of a non-mana activated ability by matching its effect — never hardcode
/// an `ability_index`. Staff of Compleation's `{T}, Pay 2 life: Add one mana of any color`
/// ability lowers into a `ManaAbility` (it is excluded from `activated_abilities`, SF-6
/// shape), so every ability after it shifts down by one; matching on effect content is the
/// only way to name an ability that survives that shift without re-deriving the index by
/// hand.
fn find_activated_ability_index(
    state: &GameState,
    id: ObjectId,
    pred: impl Fn(&Effect) -> bool,
) -> usize {
    state.objects()[&id]
        .characteristics
        .activated_abilities
        .iter()
        .position(|ab| ab.effect.as_ref().is_some_and(&pred))
        .unwrap_or_else(|| panic!("no matching activated ability found on {id:?}"))
}

/// Register every battlefield permanent's replacement abilities manually — `GameStateBuilder`
/// does not run ETB hooks (pattern from `composite_cost_mana_source_is_multiplied_by_a_mana_production_replacement`
/// in `primitive_sr34_composite_mana_costs.rs`).
fn register_all_battlefield_replacements(state: &mut GameState, registry: &Arc<CardRegistry>) {
    let battlefield: Vec<(ObjectId, PlayerId, Option<CardId>)> = state
        .objects()
        .iter()
        .filter(|(_, obj)| matches!(obj.zone, ZoneId::Battlefield))
        .map(|(id, obj)| (*id, obj.controller, obj.card_id.clone()))
        .collect();
    for (obj_id, controller, card_id) in &battlefield {
        mtg_engine::rules::replacement::register_permanent_replacement_abilities(
            state,
            *obj_id,
            controller.to_owned(),
            card_id.as_ref(),
            registry,
        );
    }
}

fn creature_filler(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 2, 2)
}

fn elf_filler(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1).with_subtypes(vec![SubType("Elf".to_string())])
}

// ── SF-8: Gaea's Cradle — dynamic mana amount, not a `{colour: 1}` marker ─────────────

/// CR 605.1a: Gaea's Cradle with ZERO creatures produces ZERO green. This is the
/// load-bearing case: pre-fix, `try_as_tap_mana_ability`'s `AddManaScaled` arm registered
/// `produces = {Green: 1}` as a marker and `handle_tap_for_mana` read it literally, so the
/// bug produced exactly **1** green regardless of board state — the same wrong answer a
/// "1 creature → 1 green" test would coincidentally also pass. Only the 0 case (and the 3
/// case below) can tell the fix from the bug.
#[test]
fn gaea_cradle_zero_creatures_produces_zero_green() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let cradle = make_spec(p(1), "Gaea's Cradle", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(cradle)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let cradle_id = find_by_name(&state, "Gaea's Cradle");
    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: cradle_id,
            ability_index: 0,

            chosen_color: None,
        },
    )
    .expect("Gaea's Cradle activation should succeed with zero creatures");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Green),
        0,
        "0 creatures controlled must produce 0 green (CR 605.1a) — the pre-fix bug produced \
         exactly 1 here regardless of board state"
    );
}

/// CR 605.1a: Gaea's Cradle with 3 creatures produces 3 green (not 1, the pre-fix marker
/// value).
#[test]
fn gaea_cradle_three_creatures_produces_three_green() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let cradle = make_spec(p(1), "Gaea's Cradle", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(cradle)
        .object(creature_filler(p(1), "Bear One"))
        .object(creature_filler(p(1), "Bear Two"))
        .object(creature_filler(p(1), "Bear Three"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let cradle_id = find_by_name(&state, "Gaea's Cradle");
    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: cradle_id,
            ability_index: 0,

            chosen_color: None,
        },
    )
    .expect("Gaea's Cradle activation should succeed");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Green),
        3,
        "3 creatures controlled must produce 3 green (CR 605.1a)"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                color: ManaColor::Green,
                amount: 3,
                ..
            }
        )),
        "ManaAdded(Green, 3) should be emitted, not ManaAdded(Green, 1)"
    );
}

/// CR 605.1a / CR 106.12b: Nyxbloom Ancient (itself a creature) triples a *scaled* mana
/// source's real production, not the `{colour: 1}` marker — this is the positive
/// consequence of substituting the resolved amount into step 7b's `base_preview` before
/// `apply_mana_production_replacements` runs. 3 creatures (Nyxbloom + 2 fillers) → base 3
/// green → tripled → 9 green.
#[test]
fn gaea_cradle_scaled_amount_is_multiplied_by_a_mana_production_replacement() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let cradle = make_spec(p(1), "Gaea's Cradle", ZoneId::Battlefield, &defs);
    let nyxbloom = make_spec(p(1), "Nyxbloom Ancient", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(Arc::clone(&registry))
        .object(cradle)
        .object(nyxbloom)
        .object(creature_filler(p(1), "Bear One"))
        .object(creature_filler(p(1), "Bear Two"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));
    register_all_battlefield_replacements(&mut state, &registry);

    let cradle_id = find_by_name(&state, "Gaea's Cradle");
    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: cradle_id,
            ability_index: 0,

            chosen_color: None,
        },
    )
    .expect("Gaea's Cradle activation should succeed");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Green),
        9,
        "3 creatures (Nyxbloom counts itself) x Nyxbloom's 3x multiplier = 9 green, not 3 \
         (unmultiplied) and not 3 (the marker tripled)"
    );
}

// ── SF-8: Elvish Archdruid — the filter is live, not a raw permanent count ──────────

/// CR 605.1a: Elvish Archdruid's `{T}: Add {G} for each Elf you control` must count only
/// Elves — a raw permanent count would include the non-Elf filler too. Board: Archdruid
/// (an Elf, counts itself) + one more Elf + one non-Elf → 2 green, not 3 (raw count) and
/// not 1 (the pre-fix marker).
#[test]
fn elvish_archdruid_counts_only_elves() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let archdruid = make_spec(p(1), "Elvish Archdruid", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(archdruid)
        .object(elf_filler(p(1), "Extra Elf"))
        .object(creature_filler(p(1), "Non-Elf Bear"))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let archdruid_id = find_by_name(&state, "Elvish Archdruid");
    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: archdruid_id,
            ability_index: 0,

            chosen_color: None,
        },
    )
    .expect("Elvish Archdruid activation should succeed");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Green),
        2,
        "must count Archdruid + Extra Elf (2), excluding Non-Elf Bear — a raw permanent \
         count would give 3, the pre-fix marker would give 1"
    );
}

// ── SF-8 + Finding-A deletion: Cabal Coffers is now a real mana ability ─────────────

/// CR 605.1a / CR 605.3b: Cabal Coffers's `{2}, {T}: Add {B} for each Swamp you control` —
/// previously excluded from mana-ability lowering by the SR-34 Finding-A guard (deleting
/// that guard is what SF-8 unblocks) — now registers as a real `ManaAbility`: it pays its
/// `{2}` generic component, produces the correct scaled black, and never touches the
/// stack. 3 Swamps → 3 black; `{2}` colorless must leave the pool.
#[test]
fn cabal_coffers_is_a_real_mana_ability() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let coffers = make_spec(p(1), "Cabal Coffers", ZoneId::Battlefield, &defs);
    assert_eq!(
        coffers.mana_abilities.len(),
        1,
        "Cabal Coffers must register as a ManaAbility now that the Finding-A exclusion is gone"
    );
    assert!(
        coffers.activated_abilities.is_empty(),
        "the same ability must not ALSO appear in activated_abilities (SF-6 exclusion)"
    );

    let swamp1 = make_spec(p(1), "Swamp", ZoneId::Battlefield, &defs);
    let swamp2 = make_spec(p(1), "Swamp", ZoneId::Battlefield, &defs);
    let swamp3 = make_spec(p(1), "Swamp", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(coffers)
        .object(swamp1)
        .object(swamp2)
        .object(swamp3)
        .player_mana(
            p(1),
            mtg_engine::ManaPool {
                colorless: 2,
                ..Default::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let coffers_id = find_by_name(&state, "Cabal Coffers");
    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: coffers_id,
            ability_index: 0,

            chosen_color: None,
        },
    )
    .expect("Cabal Coffers activation should succeed via TapForMana (CR 605.3b)");

    assert!(
        state.stack_objects().is_empty(),
        "CR 605.3b: a mana ability must not use the stack"
    );
    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Colorless),
        0,
        "the {{2}} generic cost must actually leave the pool"
    );
    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Black),
        3,
        "3 Swamps controlled must produce 3 black"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaCostPaid { player, cost } if *player == p(1) && cost.generic == 2
        )),
        "ManaCostPaid{{generic:2}} should be emitted"
    );
}

// ── SF-9: fetchlands, Doom Whisperer, Staff of Compleation ──────────────────────────

/// CR 118.3 / CR 119.4: Arid Mesa's `{T}, Pay 1 life, Sacrifice this land: Search...` pays
/// its life cost on activation. Before SF-9, `flatten_cost_into` mapped `Cost::PayLife(_)`
/// to nothing and `ActivationCost` had no life field at all, so this (and the other 10
/// fetchlands) shipped `Complete` while paying zero life for a legal deck's most-played
/// lands.
#[test]
fn fetchland_pays_life_on_activation() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let mesa = make_spec(p(1), "Arid Mesa", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(mesa)
        .player_life(p(1), 40)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let mesa_id = find_by_name(&state, "Arid Mesa");
    let ability_index =
        find_activated_ability_index(&state, mesa_id, |e| matches!(e, Effect::Sequence(_)));
    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: mesa_id,
            ability_index,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    )
    .expect("Arid Mesa activation should succeed");

    assert_eq!(
        state.player(p(1)).unwrap().life_total,
        39,
        "Arid Mesa's Pay 1 life must leave the pool at 39, not 40"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { player, amount: 1 } if *player == p(1))),
        "LifeLost{{amount:1}} should be emitted"
    );
}

/// CR 118.3 / CR 119.4: Doom Whisperer's `Pay 2 life: Surveil 2` (no `{T}`, freely
/// repeatable) pays its life cost. Before SF-9 this was probed at 40 -> 40 (free repeatable
/// surveil).
#[test]
fn doom_whisperer_pays_life_for_repeatable_surveil() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let demon = make_spec(p(1), "Doom Whisperer", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(demon)
        .player_life(p(1), 40)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let demon_id = find_by_name(&state, "Doom Whisperer");
    let ability_index =
        find_activated_ability_index(&state, demon_id, |e| matches!(e, Effect::Surveil { .. }));
    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: demon_id,
            ability_index,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    )
    .expect("Doom Whisperer's Pay 2 life ability should succeed at life 40");

    assert_eq!(
        state.player(p(1)).unwrap().life_total,
        38,
        "Doom Whisperer's Pay 2 life must leave the pool at 38, not 40"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { player, amount: 2 } if *player == p(1))),
        "LifeLost{{amount:2}} should be emitted"
    );
}

/// CR 118.3 / CR 119.4: Staff of Compleation's `{T}, Pay 3 life: Proliferate` pays its life
/// cost. Before SF-9 this and the draw ability below were probed at 40 -> 40 (a free
/// repeatable proliferate and draw on a `Complete` card).
#[test]
fn staff_of_compleation_proliferate_pays_life() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let staff = make_spec(p(1), "Staff of Compleation", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(staff)
        .player_life(p(1), 40)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let staff_id = find_by_name(&state, "Staff of Compleation");
    let ability_index =
        find_activated_ability_index(&state, staff_id, |e| matches!(e, Effect::Proliferate));
    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: staff_id,
            ability_index,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    )
    .expect("Staff of Compleation's Pay 3 life: Proliferate should succeed at life 40");

    assert_eq!(
        state.player(p(1)).unwrap().life_total,
        37,
        "Staff of Compleation's Pay 3 life must leave the pool at 37, not 40"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { player, amount: 3 } if *player == p(1))),
        "LifeLost{{amount:3}} should be emitted"
    );
}

/// CR 118.3 / CR 119.4: Staff of Compleation's `{T}, Pay 4 life: Draw a card` pays its life
/// cost.
#[test]
fn staff_of_compleation_draw_pays_life() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let staff = make_spec(p(1), "Staff of Compleation", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(staff)
        .player_life(p(1), 40)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let staff_id = find_by_name(&state, "Staff of Compleation");
    let ability_index =
        find_activated_ability_index(&state, staff_id, |e| matches!(e, Effect::DrawCards { .. }));
    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: staff_id,
            ability_index,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    )
    .expect("Staff of Compleation's Pay 4 life: Draw a card should succeed at life 40");

    assert_eq!(
        state.player(p(1)).unwrap().life_total,
        36,
        "Staff of Compleation's Pay 4 life must leave the pool at 36, not 40"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { player, amount: 4 } if *player == p(1))),
        "LifeLost{{amount:4}} should be emitted"
    );
}

// ── SF-9: CR 119.4b short-circuit and insufficient life ─────────────────────────────

/// CR 119.4b: a player at negative life may still activate an ability with NO life cost.
/// Staff of Compleation's `{5}: Untap this artifact` has no `Cost::PayLife` component, so
/// its `ActivationCost::life_cost` defaults to 0 — the check must short-circuit on
/// `life_cost > 0` rather than reading `life_total >= life_cost` unguarded (which would
/// wrongly reject at negative life for a `life_cost: 0` ability, since the source's own
/// `>=` check is against 0 unconditionally only when guarded).
#[test]
fn non_mana_ability_life_cost_zero_is_legal_at_negative_life() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let staff = make_spec(p(1), "Staff of Compleation", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(staff)
        .player_life(p(1), -3)
        .player_mana(
            p(1),
            mtg_engine::ManaPool {
                colorless: 5,
                ..Default::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let staff_id = find_by_name(&state, "Staff of Compleation");
    let ability_index = find_activated_ability_index(&state, staff_id, |e| {
        matches!(e, Effect::UntapPermanent { .. })
    });
    let (state, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: staff_id,
            ability_index,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    )
    .expect(
        "a life_cost:0 ability must activate at any life total, including negative (CR 119.4b)",
    );

    assert_eq!(
        state.player(p(1)).unwrap().life_total,
        -3,
        "a life_cost:0 ability must not change life_total"
    );
}

/// CR 118.3 / CR 119.4: a player who cannot pay the life cost is rejected with
/// `InsufficientLife`, and the caller's pre-command state is untouched.
#[test]
fn non_mana_ability_insufficient_life_is_rejected() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let demon = make_spec(p(1), "Doom Whisperer", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(demon)
        .player_life(p(1), 1)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let demon_id = find_by_name(&state, "Doom Whisperer");
    let ability_index =
        find_activated_ability_index(&state, demon_id, |e| matches!(e, Effect::Surveil { .. }));
    let probe_state = state.clone();
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: demon_id,
            ability_index,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    );

    assert!(
        matches!(
            result,
            Err(GameStateError::InsufficientLife {
                required: 2,
                actual: 1,
                ..
            })
        ),
        "a player at 1 life cannot pay 2 life: {result:?}"
    );
    assert_eq!(
        probe_state.player(p(1)).unwrap().life_total,
        1,
        "the caller's pre-command state must be untouched"
    );
}

// ── Non-vacuity control: type-check the helper's predicate signature ────────────────

/// Sanity check that `find_activated_ability_index` reads live `Characteristics`, not a
/// cached/stale copy — if the object under test has no matching ability at all, the helper
/// panics rather than silently returning index 0 and asserting against the wrong ability.
#[test]
#[should_panic(expected = "no matching activated ability found")]
fn find_activated_ability_index_panics_when_nothing_matches() {
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());
    let demon = make_spec(p(1), "Doom Whisperer", ZoneId::Battlefield, &defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(demon)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let demon_id = find_by_name(&state, "Doom Whisperer");
    // Doom Whisperer has no Effect::Proliferate ability.
    let _ = find_activated_ability_index(&state, demon_id, |e| matches!(e, Effect::Proliferate));
}

// ── SF-8: the other two ex-Finding-A cards, upgraded Partial -> Complete by SR-36 ──────
//
// Cabal Stronghold and Crypt of Agadeem were `Partial` with one recorded blocker each: the
// CR 605.1a/605.3b mis-registration SF-8 fixed. SR-36 (`scutemob-92`) upgraded both to
// `Complete`, so the amount their upgrade rests on is pinned here by activation. Without
// these, the two cards would be Complete on the strength of a marker note alone — the
// `megrim.rs` calibration error (CLAUDE.md), where a note's claim and the card's real
// behaviour are independent facts.
//
// Each board contains a decoy the count MUST exclude, so a filter that silently degraded to
// a raw count (several `TargetFilter` fields are ignored by `matches_filter` — CLAUDE.md)
// would fail rather than pass with a coincidentally-equal number.

/// CR 605.1a: Cabal Stronghold's `{3},{T}: Add {B} for each basic Swamp you control`
/// produces one black per **basic** Swamp.
///
/// The decoy is **Bayou** — a Land with the `Swamp` subtype and no `Basic` supertype —
/// specifically so this test pins `TargetFilter::basic`. An earlier version used Cabal
/// Coffers, which is not a Swamp at all (`types(&[CardType::Land])`, no subtypes), so
/// `matches_filter` rejected it on `has_subtype` and never reached the `basic` check:
/// deleting `basic: true` from the def left the count at 2 and this test green (SR-36
/// review, Finding 2). A decoy must fail on exactly the field under test, or it pins
/// nothing.
#[test]
fn cabal_stronghold_counts_only_basic_swamps() {
    let defs = defs_map();
    let stronghold = make_spec(p(1), "Cabal Stronghold", ZoneId::Battlefield, &defs);
    assert_eq!(
        stronghold.mana_abilities.len(),
        2,
        "both the {{T}}: Add {{C}} arm and the scaled arm must register (SF-8)"
    );
    assert!(
        stronghold.activated_abilities.is_empty(),
        "neither ability may ALSO appear in activated_abilities (SF-6 exclusion)"
    );

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(all_cards()))
        .object(stronghold)
        .object(make_spec(p(1), "Swamp", ZoneId::Battlefield, &defs))
        .object(make_spec(p(1), "Swamp", ZoneId::Battlefield, &defs))
        // A Swamp by subtype but NOT basic: reaches the `basic` check and must fail it.
        .object(make_spec(p(1), "Bayou", ZoneId::Battlefield, &defs))
        .player_mana(
            p(1),
            mtg_engine::ManaPool {
                colorless: 5,
                ..Default::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let id = find_by_name(&state, "Cabal Stronghold");
    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: id,
            ability_index: 1,

            chosen_color: None,
        },
    )
    .expect("Cabal Stronghold's scaled arm should activate via TapForMana (CR 605.3b)");

    assert!(
        state.stack_objects().is_empty(),
        "CR 605.3b: a mana ability must not use the stack"
    );
    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Black),
        2,
        "2 basic Swamps must produce 2 black (pre-SF-8 this was the constant 1)"
    );
    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Colorless),
        2,
        "the {{3}} generic cost must actually leave the pool (5 - 3)"
    );
}

/// CR 605.1a: Crypt of Agadeem's `{2},{T}: Add {B} for each black creature card in your
/// graveyard` counts by colour. A green creature card in the same graveyard must not count
/// — `TargetFilter::colors` must be live through `EffectAmount::CardCount`.
#[test]
fn crypt_of_agadeem_counts_only_black_creature_cards_in_graveyard() {
    let defs = defs_map();
    let crypt = make_spec(p(1), "Crypt of Agadeem", ZoneId::Battlefield, &defs);
    assert_eq!(
        crypt.mana_abilities.len(),
        2,
        "both the {{T}}: Add {{B}} arm and the scaled arm must register (SF-8)"
    );
    assert!(
        crypt.activated_abilities.is_empty(),
        "neither ability may ALSO appear in activated_abilities (SF-6 exclusion)"
    );

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(all_cards()))
        .object(crypt)
        .object(make_spec(
            p(1),
            "Doom Whisperer",
            ZoneId::Graveyard(p(1)),
            &defs,
        ))
        .object(make_spec(
            p(1),
            "Razaketh, the Foulblooded",
            ZoneId::Graveyard(p(1)),
            &defs,
        ))
        // Green creature card in the same graveyard: the colour filter must exclude it.
        .object(make_spec(
            p(1),
            "Elvish Archdruid",
            ZoneId::Graveyard(p(1)),
            &defs,
        ))
        .player_mana(
            p(1),
            mtg_engine::ManaPool {
                colorless: 5,
                ..Default::default()
            },
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));

    let id = find_by_name(&state, "Crypt of Agadeem");
    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source: id,
            ability_index: 1,

            chosen_color: None,
        },
    )
    .expect("Crypt of Agadeem's scaled arm should activate via TapForMana (CR 605.3b)");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Black),
        2,
        "2 black creature cards must produce 2 black; the green Elvish Archdruid must not count"
    );
    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Colorless),
        3,
        "the {{2}} generic cost must actually leave the pool (5 - 2)"
    );
}

// ── SR-38 SG-2: the non-`Controller` refusal in `try_as_tap_mana_ability` ──────────────
//
// SR-36 added a guard: an `Effect::AddManaScaled` whose `player` is not
// `PlayerTarget::Controller` is *not* lowered into a `ManaAbility`, because the stackless
// `TapForMana` path always pays the activating player. No real card exercises it (every
// scaled mana source in the corpus pays its controller — verified via `all_cards()`), so its
// deletion would otherwise be silent. This test pins the branch and, via a controller-paying
// control case, proves the `PlayerTarget` guard is the sole cause of the difference.

/// A synthetic land whose single `{T}` ability adds a scaled amount of black mana to `player`.
fn scaled_mana_def(name: &str, player: PlayerTarget) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddManaScaled {
                player,
                color: ManaColor::Black,
                count: EffectAmount::Fixed(1),
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    }
}

#[test]
fn opponent_scaled_mana_stays_a_stack_ability() {
    // Paying EACH OPPONENT: the stackless TapForMana path cannot express "pay another
    // player", so `mana_ability_lowering` declines and the ability stays on the stack, where
    // `Effect::AddManaScaled`'s stack-resolution arm handles an arbitrary PlayerTarget.
    let mut defs = defs_map();
    let opp = scaled_mana_def("SG2 Opponent Scaled", PlayerTarget::EachOpponent);
    defs.insert(opp.name.clone(), opp);
    let spec = enrich_spec_from_def(
        ObjectSpec::card(p(1), "SG2 Opponent Scaled").in_zone(ZoneId::Battlefield),
        &defs,
    );
    assert!(
        spec.mana_abilities.is_empty(),
        "an AddManaScaled paying a non-controller must NOT lower to a mana ability (CR 605.3b)"
    );
    assert_eq!(
        spec.activated_abilities.len(),
        1,
        "it must remain a stack-using activated ability so its PlayerTarget resolves correctly"
    );

    // Control (non-vacuity): the IDENTICAL ability paying the CONTROLLER *does* lower — so the
    // `PlayerTarget::Controller` guard is the only reason the opponent case above differs.
    let mut defs2 = defs_map();
    let ctrl = scaled_mana_def("SG2 Controller Scaled", PlayerTarget::Controller);
    defs2.insert(ctrl.name.clone(), ctrl);
    let spec2 = enrich_spec_from_def(
        ObjectSpec::card(p(1), "SG2 Controller Scaled").in_zone(ZoneId::Battlefield),
        &defs2,
    );
    assert_eq!(
        spec2.mana_abilities.len(),
        1,
        "the controller-paying variant lowers to a mana ability (SR-36)"
    );
    assert!(
        spec2.activated_abilities.is_empty(),
        "a lowered mana ability is excluded from activated_abilities (SF-6)"
    );
}

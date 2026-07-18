//! PB-EF10: sacrifice-driven `EffectAmount` / runtime `max_cmc` / "if you do" `Condition`.
//!
//! Three independent sub-gaps from finding EF-W-MISS-7, backed by a single data-model
//! change (`SacrificedCreatureLki` struct replacing the old `Vec<i32>` powers):
//!
//! 1. `EffectAmount::ToughnessOfSacrificedCreature` — the LKI-toughness twin of the
//!    existing `PowerOfSacrificedCreature` (CR 608.2b/608.2i).
//! 2. `TargetFilter.max_cmc_amount` (runtime search cap) + companion
//!    `EffectAmount::ManaValueOfSacrificedCreature` (CR 202.3/608.2h).
//! 3. `Condition::SacrificeFired` — "if you do" gating on a resolution-time
//!    `Effect::SacrificePermanents` (CR 608.2c/608.2h).
//!
//! CR Rules covered: 608.2b, 608.2c, 608.2h, 608.2i, 701.21a, 202.3, 613.1d, 400.7.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    process_command, AdditionalCost, CardId, CardRegistry, Command, ContinuousEffect,
    EffectAmount, EffectDuration, EffectFilter, EffectId, EffectLayer, GameEvent, GameState,
    GameStateBuilder, LayerModification, ManaPool, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner))
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

fn count_in_zone(state: &GameState, zone: ZoneId) -> usize {
    state.objects().values().filter(|o| o.zone == zone).count()
}

/// Pass priority for all listed players once, accumulating events.
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

/// Drain the stack completely (repeated pass_all rounds) — needed when a resolving
/// spell/ability itself puts more objects on the stack (e.g. ETB triggers).
#[allow(dead_code)]
fn drain_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        let (s, _) = pass_all(state, players);
        state = s;
        guard += 1;
        assert!(guard < 20, "drain_stack: stack did not empty after 20 rounds");
    }
    state
}

fn anthem_toughness_effect(id: u64, amount: i32) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: None,
        timestamp: 100,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::ModifyToughness(amount),
        is_cda: false,
        condition: None,
    }
}

#[allow(dead_code)]
fn add_library_cards(
    builder: GameStateBuilder,
    owner: PlayerId,
    n: usize,
    prefix: &str,
) -> GameStateBuilder {
    let mut b = builder;
    for i in 0..n {
        b = b.object(
            ObjectSpec::card(owner, &format!("{} {}", prefix, i))
                .with_card_id(CardId(format!("{}-lib-{}", prefix, i)))
                .in_zone(ZoneId::Library(owner)),
        );
    }
    b
}

// ═══════════════════════════════════════════════════════════════════════════
// Sub-gap 1: EffectAmount::ToughnessOfSacrificedCreature
// ═══════════════════════════════════════════════════════════════════════════

/// CR 608.2b — a manually-constructed EffectContext carrying a captured 2/5
/// creature's LKI resolves ToughnessOfSacrificedCreature to 5, not its power (2).
#[test]
fn test_toughness_of_sacrificed_creature_basic() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::{Effect, PlayerTarget};

    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let life_before = state.players().get(&p1).map(|ps| ps.life_total).unwrap();

    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    ctx.sacrificed_creature_lki = vec![mtg_engine::SacrificedCreatureLki {
        power: 2,
        toughness: 5,
        mana_value: 3,
    }];

    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::ToughnessOfSacrificedCreature,
    };
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    let life_after = state.players().get(&p1).map(|ps| ps.life_total).unwrap();
    assert_eq!(
        life_after - life_before,
        5,
        "CR 608.2b: ToughnessOfSacrificedCreature should read toughness (5), not power (2)"
    );
}

/// CR 613.1d/608.2h — a creature under a +0/+2 anthem is layer-resolved to a higher
/// toughness; sacrificing it must capture the BOOSTED value (anthem counted at the
/// sacrifice moment), not the printed/base toughness.
#[test]
fn test_toughness_of_sacrificed_creature_reads_layer_resolved() {
    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def};
    use std::collections::HashMap;

    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    // Miren, the Moaning Well: {3},{T},Sacrifice a creature: gain life = toughness.
    let miren = enrich_spec_from_def(
        ObjectSpec::card(p1, "Miren, the Moaning Well")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Miren, the Moaning Well")),
        &defs,
    );

    // Base 2/2 bear; anthem gives +0/+2 toughness -> resolves as 2/4.
    let bear = ObjectSpec::creature(p1, "Anthem Toughness Bear", 2, 2)
        .with_card_id(CardId("anthem-toughness-bear".to_string()))
        .in_zone(ZoneId::Battlefield);

    let builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(miren)
        .object(bear)
        .add_continuous_effect(anthem_toughness_effect(1, 2));

    let state = builder.build().unwrap();
    let players = [p1, p2, p3, p4];

    let bear_id = find_obj(&state, "Anthem Toughness Bear");
    let miren_id = find_obj(&state, "Miren, the Moaning Well");

    let chars = mtg_engine::calculate_characteristics(&state, bear_id)
        .expect("calculate_characteristics should work");
    assert_eq!(
        chars.toughness,
        Some(4),
        "CR 613.1d: anthem-boosted Bear should resolve as 2/4 on the battlefield"
    );

    // Grant {3} generic mana for Miren's activation cost.
    let mut state = state;
    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool = ManaPool {
            colorless: 3,
            ..Default::default()
        };
    }

    let life_before = state.players().get(&p1).map(|ps| ps.life_total).unwrap();

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: miren_id,
            // Mind Stone gotcha: {T}: Add {C} is a mana ability and is filtered out of
            // `activated_abilities` by enrich_spec_from_def, so the sacrifice ability
            // (Miren's second printed ability) lands at index 0, not 1.
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(bear_id),
            x_value: None,
            modes_chosen: vec![],
        },
    )
    .expect("Miren activation should succeed");

    assert!(
        in_graveyard(&state, "Anthem Toughness Bear", p1),
        "Bear must be sacrificed into P1's graveyard"
    );

    let state = drain_stack(state, &players);

    let life_after = state.players().get(&p1).map(|ps| ps.life_total).unwrap();
    assert_eq!(
        life_after - life_before,
        4,
        "CR 608.2b/608.2h/613.1d: gained life should be the LKI toughness (4, anthem-boosted), \
         not the printed toughness (2)"
    );
}

/// DECOY: sacrifice a 1/3 creature (power != toughness); assert
/// ToughnessOfSacrificedCreature == 3 (toughness). Fails if the resolver arm was
/// copy-pasted from PowerOfSacrificedCreature and reads `.power` (which is 1) instead.
#[test]
fn test_toughness_amount_reads_toughness_not_power() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::{Effect, PlayerTarget};

    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let life_before = state.players().get(&p1).map(|ps| ps.life_total).unwrap();

    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    // power=1, toughness=3 — a copy-pasted `.power` read would gain 1, not 3.
    ctx.sacrificed_creature_lki = vec![mtg_engine::SacrificedCreatureLki {
        power: 1,
        toughness: 3,
        mana_value: 0,
    }];

    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::ToughnessOfSacrificedCreature,
    };
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    let life_after = state.players().get(&p1).map(|ps| ps.life_total).unwrap();
    assert_eq!(
        life_after - life_before,
        3,
        "DECOY: ToughnessOfSacrificedCreature must read toughness (3), not power (1) — \
         a copy-paste-from-PowerOfSacrificedCreature bug would gain 1 here"
    );
}

/// CR 608.2b/608.2i — integration: Momentous Fall sacrifices a 3/4 creature as an
/// additional cost; both LKI reads live in one card (draw 3, gain 4).
#[test]
fn test_momentous_fall_draws_power_gains_toughness() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def};
    use std::collections::HashMap;

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    let momentous_fall = enrich_spec_from_def(
        ObjectSpec::card(p1, "Momentous Fall")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(card_name_to_id("Momentous Fall")),
        &defs,
    );

    let beast = ObjectSpec::creature(p1, "Sacrificial Beast 3-4", 3, 4)
        .with_card_id(CardId("sacrificial-beast-3-4".to_string()))
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(momentous_fall)
        .object(beast);

    builder = add_library_cards(builder, p1, 10, "P1Lib");

    let mut state = builder.build().unwrap();

    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool = ManaPool {
            colorless: 2,
            green: 2,
            ..Default::default()
        };
    }

    let fall_id = find_obj(&state, "Momentous Fall");
    let beast_id = find_obj(&state, "Sacrificial Beast 3-4");

    let lib_before = count_in_zone(&state, ZoneId::Library(p1));
    let life_before = state.players().get(&p1).map(|ps| ps.life_total).unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: fall_id,
            targets: vec![],
            modes_chosen: vec![],
            x_value: 0,
            kicker_times: 0,
            additional_costs: vec![AdditionalCost::Sacrifice {
                ids: vec![beast_id],
                lki: vec![],
            }],
            alt_cost: None,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        })),
    )
    .expect("Cast Momentous Fall should succeed");

    assert!(
        in_graveyard(&state, "Sacrificial Beast 3-4", p1),
        "CR 118.8: the 3/4 Beast must be sacrificed as additional cost"
    );

    let (state, _) = pass_all(state, &players);

    let lib_after = count_in_zone(&state, ZoneId::Library(p1));
    let life_after = state.players().get(&p1).map(|ps| ps.life_total).unwrap();

    assert_eq!(
        lib_before - lib_after,
        3,
        "PB-EF10: Momentous Fall should draw 3 cards (sacrificed creature's power)"
    );
    assert_eq!(
        life_after - life_before,
        4,
        "PB-EF10: Momentous Fall should gain 4 life (sacrificed creature's toughness)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Sub-gap 2: runtime search cap (max_cmc_amount + ManaValueOfSacrificedCreature)
// ═══════════════════════════════════════════════════════════════════════════

/// CR 202.3/608.2h — direct: ctx carries a sacrificed creature with mana_value=3;
/// SearchLibrary filter max_cmc_amount = Sum(Fixed(2), ManaValueOfSacrificedCreature)
/// (cap 5). Library has a MV-5 and a MV-6 creature. Assert MV-5 found, MV-6 not.
#[test]
fn test_search_max_cmc_amount_caps_by_runtime_value() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::{Effect, ManaCost, PlayerTarget, TargetFilter, ZoneTarget};

    let p1 = p(1);
    let p2 = p(2);

    let mv5 = ObjectSpec::creature(p1, "Five Drop", 5, 5)
        .with_card_id(CardId("five-drop".to_string()))
        .with_mana_cost(ManaCost {
            generic: 5,
            ..Default::default()
        })
        .in_zone(ZoneId::Library(p1));
    let mv6 = ObjectSpec::creature(p1, "Six Drop", 6, 6)
        .with_card_id(CardId("six-drop".to_string()))
        .with_mana_cost(ManaCost {
            generic: 6,
            ..Default::default()
        })
        .in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(mv5)
        .object(mv6)
        .build()
        .unwrap();

    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    ctx.sacrificed_creature_lki = vec![mtg_engine::SacrificedCreatureLki {
        power: 0,
        toughness: 0,
        mana_value: 3,
    }];

    let effect = Effect::SearchLibrary {
        player: PlayerTarget::Controller,
        filter: TargetFilter {
            has_card_type: Some(mtg_engine::CardType::Creature),
            max_cmc_amount: Some(Box::new(EffectAmount::Sum(
                Box::new(EffectAmount::Fixed(2)),
                Box::new(EffectAmount::ManaValueOfSacrificedCreature),
            ))),
            ..Default::default()
        },
        reveal: false,
        destination: ZoneTarget::Battlefield { tapped: false },
        shuffle_before_placing: false,
        also_search_graveyard: false,
    };

    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        on_battlefield(&state, "Five Drop"),
        "PB-EF10: MV-5 creature (cap = 2+3 = 5) should be found and placed"
    );
    assert!(
        !on_battlefield(&state, "Six Drop"),
        "PB-EF10: MV-6 creature exceeds the runtime cap (5) and should NOT be found"
    );
}

/// DECOY: assert the found card is exactly MV-5 (= 2 + 3): fails if the `+2` is
/// dropped (cap 3, MV-5 rejected) OR if the sac MV is dropped (cap 2, MV-5 rejected).
/// Pins both summands.
#[test]
fn test_search_cap_uses_both_terms() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::{Effect, ManaCost, PlayerTarget, TargetFilter, ZoneTarget};

    let p1 = p(1);
    let p2 = p(2);

    let mv5 = ObjectSpec::creature(p1, "Exactly Five", 5, 5)
        .with_card_id(CardId("exactly-five".to_string()))
        .with_mana_cost(ManaCost {
            generic: 5,
            ..Default::default()
        })
        .in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(mv5)
        .build()
        .unwrap();

    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    ctx.sacrificed_creature_lki = vec![mtg_engine::SacrificedCreatureLki {
        power: 0,
        toughness: 0,
        mana_value: 3,
    }];

    let effect = Effect::SearchLibrary {
        player: PlayerTarget::Controller,
        filter: TargetFilter {
            has_card_type: Some(mtg_engine::CardType::Creature),
            max_cmc_amount: Some(Box::new(EffectAmount::Sum(
                Box::new(EffectAmount::Fixed(2)),
                Box::new(EffectAmount::ManaValueOfSacrificedCreature),
            ))),
            ..Default::default()
        },
        reveal: false,
        destination: ZoneTarget::Battlefield { tapped: false },
        shuffle_before_placing: false,
        also_search_graveyard: false,
    };

    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        on_battlefield(&state, "Exactly Five"),
        "DECOY: cap must be exactly 2+3=5 — if the +2 term or the sacrificed MV term \
         is dropped, this MV-5 card would be wrongly rejected"
    );
}

/// CR 608.2h — integration: Eldritch Evolution sacrifices an MV-2 creature (cap
/// 2+2=4); a MV-4 creature is found and put onto the battlefield, a MV-5 is not,
/// and Eldritch Evolution itself is exiled (not left in the graveyard).
#[test]
fn test_eldritch_evolution_finds_up_to_two_plus_sac_mv() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def, ManaCost};
    use std::collections::HashMap;

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    let evolution = enrich_spec_from_def(
        ObjectSpec::card(p1, "Eldritch Evolution")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(card_name_to_id("Eldritch Evolution")),
        &defs,
    );

    let sac_creature = ObjectSpec::creature(p1, "MV Two Sac", 2, 2)
        .with_card_id(CardId("mv-two-sac".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Battlefield);

    let mv4 = ObjectSpec::creature(p1, "MV Four Lib", 4, 4)
        .with_card_id(CardId("mv-four-lib".to_string()))
        .with_mana_cost(ManaCost {
            generic: 4,
            ..Default::default()
        })
        .in_zone(ZoneId::Library(p1));

    let mv5 = ObjectSpec::creature(p1, "MV Five Lib", 5, 5)
        .with_card_id(CardId("mv-five-lib".to_string()))
        .with_mana_cost(ManaCost {
            generic: 5,
            ..Default::default()
        })
        .in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(evolution)
        .object(sac_creature)
        .object(mv4)
        .object(mv5)
        .build()
        .unwrap();

    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool = ManaPool {
            colorless: 1,
            green: 2,
            ..Default::default()
        };
    }

    let evolution_id = find_obj(&state, "Eldritch Evolution");
    let sac_id = find_obj(&state, "MV Two Sac");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: evolution_id,
            targets: vec![],
            modes_chosen: vec![],
            x_value: 0,
            kicker_times: 0,
            additional_costs: vec![AdditionalCost::Sacrifice {
                ids: vec![sac_id],
                lki: vec![],
            }],
            alt_cost: None,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        })),
    )
    .expect("Cast Eldritch Evolution should succeed");

    assert!(
        in_graveyard(&state, "MV Two Sac", p1),
        "CR 118.8: the MV-2 creature must be sacrificed as additional cost"
    );

    let state = drain_stack(state, &players);

    assert!(
        on_battlefield(&state, "MV Four Lib"),
        "PB-EF10: MV-4 creature (cap = 2+2 = 4) should be found and placed"
    );
    assert!(
        !on_battlefield(&state, "MV Five Lib"),
        "PB-EF10: MV-5 creature exceeds the runtime cap (4) and should NOT be found"
    );
    assert!(
        !in_graveyard(&state, "Eldritch Evolution", p1)
            && !on_battlefield(&state, "Eldritch Evolution"),
        "CR 707/self_exile_on_resolution: Eldritch Evolution should be exiled, not in \
         the graveyard or battlefield"
    );
}

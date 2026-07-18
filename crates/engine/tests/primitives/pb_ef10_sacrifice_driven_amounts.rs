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
    process_command, AdditionalCost, CardId, CardRegistry, Command, ContinuousEffect, EffectAmount,
    EffectDuration, EffectFilter, EffectId, EffectLayer, GameEvent, GameState, GameStateBuilder,
    LayerModification, ManaPool, ObjectId, ObjectSpec, PlayerId, Step, Target, ZoneId,
    HASH_SCHEMA_VERSION, PROTOCOL_VERSION,
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
        assert!(
            guard < 20,
            "drain_stack: stack did not empty after 20 rounds"
        );
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

// ═══════════════════════════════════════════════════════════════════════════
// Sub-gap 3: Condition::SacrificeFired
// ═══════════════════════════════════════════════════════════════════════════

/// CR 608.2c — resolve Sequence[SacrificePermanents{1,creature}, Conditional{
/// SacrificeFired, <marker effect> }] with a creature present; assert the
/// conditional branch ran (marker: a fixed life gain).
#[test]
fn test_sacrifice_fired_true_when_sacrificed() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::{Condition, Effect, PlayerTarget, TargetFilter};

    let p1 = p(1);
    let p2 = p(2);

    let creature = ObjectSpec::creature(p1, "Fodder Creature", 1, 1)
        .with_card_id(CardId("fodder-creature".to_string()))
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature)
        .build()
        .unwrap();

    let life_before = state.players().get(&p1).map(|ps| ps.life_total).unwrap();

    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let effect = Effect::Sequence(vec![
        Effect::SacrificePermanents {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
            filter: Some(TargetFilter {
                has_card_type: Some(mtg_engine::CardType::Creature),
                ..Default::default()
            }),
        },
        Effect::Conditional {
            condition: Condition::SacrificeFired,
            if_true: Box::new(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(7),
            }),
            if_false: Box::new(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(0),
            }),
        },
    ]);

    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        in_graveyard(&state, "Fodder Creature", p1),
        "The creature should have been sacrificed"
    );

    let life_after = state.players().get(&p1).map(|ps| ps.life_total).unwrap();
    assert_eq!(
        life_after - life_before,
        7,
        "CR 608.2c: Condition::SacrificeFired should be true and gate the +7 life branch"
    );
    assert!(
        ctx.sacrifice_fired,
        "ctx.sacrifice_fired should be latched true after a successful sacrifice"
    );
}

/// CR 608.2c + Victimize ruling 2020-11-10 — same Sequence, but the controller
/// controls no creature; assert sacrifice_fired == false and the conditional
/// branch did NOT run. This is also the DECOY: fails if the executor sets
/// sacrifice_fired = true unconditionally regardless of whether anything moved.
#[test]
fn test_sacrifice_fired_false_when_none_available() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::{Condition, Effect, PlayerTarget, TargetFilter};

    let p1 = p(1);
    let p2 = p(2);

    // No creatures on the battlefield at all.
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
    let effect = Effect::Sequence(vec![
        Effect::SacrificePermanents {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
            filter: Some(TargetFilter {
                has_card_type: Some(mtg_engine::CardType::Creature),
                ..Default::default()
            }),
        },
        Effect::Conditional {
            condition: Condition::SacrificeFired,
            if_true: Box::new(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(7),
            }),
            if_false: Box::new(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(0),
            }),
        },
    ]);

    let _events = execute_effect(&mut state, &effect, &mut ctx);

    let life_after = state.players().get(&p1).map(|ps| ps.life_total).unwrap();
    assert_eq!(
        life_after - life_before,
        0,
        "CR 608.2c: with no creature to sacrifice, SacrificeFired must be false — \
         the if_true (+7) branch must NOT have run"
    );
    assert!(
        !ctx.sacrifice_fired,
        "DECOY: ctx.sacrifice_fired must be false when nothing was actually sacrificed \
         (fails if the executor sets it unconditionally)"
    );
}

/// CR 608.2c/608.2h + Victimize ruling — integration: two creature cards in
/// graveyard, one creature on the battlefield; cast Victimize; assert both cards
/// return tapped under controller and the on-board creature is sacrificed.
#[test]
fn test_victimize_returns_both_when_creature_sacrificed() {
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

    let victimize = enrich_spec_from_def(
        ObjectSpec::card(p1, "Victimize")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(card_name_to_id("Victimize")),
        &defs,
    );

    let fodder = ObjectSpec::creature(p1, "Living Fodder", 1, 1)
        .with_card_id(CardId("living-fodder".to_string()))
        .in_zone(ZoneId::Battlefield);

    let gy_a = ObjectSpec::creature(p1, "Graveyard Ally A", 2, 2)
        .with_card_id(CardId("graveyard-ally-a".to_string()))
        .in_zone(ZoneId::Graveyard(p1));

    let gy_b = ObjectSpec::creature(p1, "Graveyard Ally B", 3, 3)
        .with_card_id(CardId("graveyard-ally-b".to_string()))
        .in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(victimize)
        .object(fodder)
        .object(gy_a)
        .object(gy_b)
        .build()
        .unwrap();

    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool = ManaPool {
            colorless: 2,
            black: 1,
            ..Default::default()
        };
    }

    let victimize_id = find_obj(&state, "Victimize");
    let gy_a_id = find_obj(&state, "Graveyard Ally A");
    let gy_b_id = find_obj(&state, "Graveyard Ally B");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: victimize_id,
            targets: vec![Target::Object(gy_a_id), Target::Object(gy_b_id)],
            modes_chosen: vec![],
            x_value: 0,
            kicker_times: 0,
            additional_costs: vec![],
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
    .expect("Cast Victimize should succeed");

    let state = drain_stack(state, &players);

    assert!(
        in_graveyard(&state, "Living Fodder", p1),
        "CR 701.21a: the on-board creature should have been sacrificed"
    );
    assert!(
        on_battlefield(&state, "Graveyard Ally A"),
        "CR 608.2c: SacrificeFired should gate the return — Ally A should be back"
    );
    assert!(
        on_battlefield(&state, "Graveyard Ally B"),
        "CR 608.2c: SacrificeFired should gate the return — Ally B should be back"
    );

    let a_tapped = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Graveyard Ally A")
        .map(|o| o.status.tapped)
        .unwrap_or(false);
    let b_tapped = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Graveyard Ally B")
        .map(|o| o.status.tapped)
        .unwrap_or(false);
    assert!(a_tapped, "Victimize returns creatures tapped");
    assert!(b_tapped, "Victimize returns creatures tapped");
}

/// Victimize ruling 2020-11-10 — integration: two graveyard targets but NO
/// creature to sacrifice; assert neither card returns (mandatory sac fails, the
/// "if you do" clause does not fire).
#[test]
fn test_victimize_no_return_when_no_creature_to_sacrifice() {
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

    let victimize = enrich_spec_from_def(
        ObjectSpec::card(p1, "Victimize")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(card_name_to_id("Victimize")),
        &defs,
    );

    // No creature on the battlefield for P1 at all.
    let gy_a = ObjectSpec::creature(p1, "Lonely Ally A", 2, 2)
        .with_card_id(CardId("lonely-ally-a".to_string()))
        .in_zone(ZoneId::Graveyard(p1));

    let gy_b = ObjectSpec::creature(p1, "Lonely Ally B", 3, 3)
        .with_card_id(CardId("lonely-ally-b".to_string()))
        .in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(victimize)
        .object(gy_a)
        .object(gy_b)
        .build()
        .unwrap();

    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool = ManaPool {
            colorless: 2,
            black: 1,
            ..Default::default()
        };
    }

    let victimize_id = find_obj(&state, "Victimize");
    let gy_a_id = find_obj(&state, "Lonely Ally A");
    let gy_b_id = find_obj(&state, "Lonely Ally B");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: victimize_id,
            targets: vec![Target::Object(gy_a_id), Target::Object(gy_b_id)],
            modes_chosen: vec![],
            x_value: 0,
            kicker_times: 0,
            additional_costs: vec![],
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
    .expect("Cast Victimize should succeed even with nothing to sacrifice");

    let state = drain_stack(state, &players);

    assert!(
        !on_battlefield(&state, "Lonely Ally A"),
        "Victimize ruling: with no creature to sacrifice, neither card should return"
    );
    assert!(
        !on_battlefield(&state, "Lonely Ally B"),
        "Victimize ruling: with no creature to sacrifice, neither card should return"
    );
}

/// CR 608.2b — one target leaves the graveyard before resolution (illegal target);
/// assert the sacrifice still happens and the legal card returns.
#[test]
fn test_victimize_one_illegal_target_still_sacs_and_returns_other() {
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

    let victimize = enrich_spec_from_def(
        ObjectSpec::card(p1, "Victimize")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(card_name_to_id("Victimize")),
        &defs,
    );

    let fodder = ObjectSpec::creature(p1, "Fodder For Partial", 1, 1)
        .with_card_id(CardId("fodder-for-partial".to_string()))
        .in_zone(ZoneId::Battlefield);

    let gy_legal = ObjectSpec::creature(p1, "Legal Target", 2, 2)
        .with_card_id(CardId("legal-target".to_string()))
        .in_zone(ZoneId::Graveyard(p1));

    let gy_will_leave = ObjectSpec::creature(p1, "Soon Gone Target", 3, 3)
        .with_card_id(CardId("soon-gone-target".to_string()))
        .in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(victimize)
        .object(fodder)
        .object(gy_legal)
        .object(gy_will_leave)
        .build()
        .unwrap();

    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool = ManaPool {
            colorless: 2,
            black: 1,
            ..Default::default()
        };
    }

    let victimize_id = find_obj(&state, "Victimize");
    let legal_id = find_obj(&state, "Legal Target");
    let will_leave_id = find_obj(&state, "Soon Gone Target");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: victimize_id,
            targets: vec![Target::Object(legal_id), Target::Object(will_leave_id)],
            modes_chosen: vec![],
            x_value: 0,
            kicker_times: 0,
            additional_costs: vec![],
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
    .expect("Cast Victimize should succeed");

    // Remove the second target from the graveyard before resolution (simulate it
    // becoming an illegal target, e.g. exiled by an opponent's response).
    let mut state = state;
    let _ =
        mtg_engine::state::test_util::move_object_to_zone(&mut state, will_leave_id, ZoneId::Exile);

    let state = drain_stack(state, &players);

    assert!(
        in_graveyard(&state, "Fodder For Partial", p1),
        "CR 608.2b: the sacrifice still happens even though one target became illegal"
    );
    assert!(
        on_battlefield(&state, "Legal Target"),
        "CR 608.2b: the still-legal target should be returned to the battlefield"
    );
    assert!(
        !on_battlefield(&state, "Soon Gone Target"),
        "The target that left the graveyard should not be on the battlefield"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Hash / version parity
// ═══════════════════════════════════════════════════════════════════════════

/// Two ctx with distinct sacrificed_creature_lki (differing toughness, differing
/// mana_value) hash differently; equal ones hash equal. Also asserts distinct
/// resolve_amount behavior for the two new EffectAmount variants.
#[test]
fn test_hash_new_effect_amount_variants_distinct() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let hash_amount = |amount: &EffectAmount| -> [u8; 32] {
        let mut hasher = Hasher::new();
        amount.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    let h_power = hash_amount(&EffectAmount::PowerOfSacrificedCreature);
    let h_toughness = hash_amount(&EffectAmount::ToughnessOfSacrificedCreature);
    let h_mv = hash_amount(&EffectAmount::ManaValueOfSacrificedCreature);
    let h_fixed = hash_amount(&EffectAmount::Fixed(0));

    assert_ne!(
        h_power, h_toughness,
        "PowerOfSacrificedCreature and ToughnessOfSacrificedCreature must hash distinctly"
    );
    assert_ne!(
        h_toughness, h_mv,
        "ToughnessOfSacrificedCreature and ManaValueOfSacrificedCreature must hash distinctly"
    );
    assert_ne!(
        h_mv, h_fixed,
        "ManaValueOfSacrificedCreature and Fixed(0) must hash distinctly"
    );
}

/// AdditionalCost::Sacrifice with LKI structs differing in one field hash
/// differently (proves all three fields feed the stream).
#[test]
fn test_sacrificed_creature_lki_struct_hash() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;
    use mtg_engine::SacrificedCreatureLki;

    let hash_ac = |ac: &AdditionalCost| -> [u8; 32] {
        let mut hasher = Hasher::new();
        ac.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    let base = AdditionalCost::Sacrifice {
        ids: vec![ObjectId(1)],
        lki: vec![SacrificedCreatureLki {
            power: 2,
            toughness: 3,
            mana_value: 4,
        }],
    };
    let diff_power = AdditionalCost::Sacrifice {
        ids: vec![ObjectId(1)],
        lki: vec![SacrificedCreatureLki {
            power: 99,
            toughness: 3,
            mana_value: 4,
        }],
    };
    let diff_toughness = AdditionalCost::Sacrifice {
        ids: vec![ObjectId(1)],
        lki: vec![SacrificedCreatureLki {
            power: 2,
            toughness: 99,
            mana_value: 4,
        }],
    };
    let diff_mv = AdditionalCost::Sacrifice {
        ids: vec![ObjectId(1)],
        lki: vec![SacrificedCreatureLki {
            power: 2,
            toughness: 3,
            mana_value: 99,
        }],
    };

    let h_base = hash_ac(&base);
    assert_ne!(
        h_base,
        hash_ac(&diff_power),
        "power must feed the hash stream"
    );
    assert_ne!(
        h_base,
        hash_ac(&diff_toughness),
        "toughness must feed the hash stream"
    );
    assert_ne!(
        h_base,
        hash_ac(&diff_mv),
        "mana_value must feed the hash stream"
    );

    // Equal structs hash equal.
    let base_again = AdditionalCost::Sacrifice {
        ids: vec![ObjectId(1)],
        lki: vec![SacrificedCreatureLki {
            power: 2,
            toughness: 3,
            mana_value: 4,
        }],
    };
    assert_eq!(
        h_base,
        hash_ac(&base_again),
        "identical LKI must hash identically"
    );
}

/// Assert PROTOCOL_VERSION == 15 and HASH_SCHEMA_VERSION == 53 (the machine-forced
/// values for this batch). See crates/engine/src/rules/protocol.rs and
/// crates/engine/src/state/hash.rs for the authoritative bump.
#[test]
fn test_pb_ef10_version_sentinels() {
    assert_eq!(
        PROTOCOL_VERSION, 16,
        "PROTOCOL_VERSION should be 15 after PB-EF10 (TargetFilter.max_cmc_amount / \
         AdditionalCost::Sacrifice reshape)"
    );
    assert_eq!(
        HASH_SCHEMA_VERSION, 54u8,
        "HASH_SCHEMA_VERSION should be 53 after PB-EF10"
    );
}

/// PB-EF10 review LOW #2 (regression pin): `Effect::MoveZone` must honor
/// `ZoneTarget::Battlefield { tapped }` — the `dest_tapped()` application was
/// previously wired only into the `SearchLibrary` matched-card path and never
/// called from `MoveZone`, so any "return ~ to the battlefield tapped" effect
/// silently entered untapped. This is Victimize's return path AND shipped-Complete
/// `reassembling_skeleton` ("Return this card from your graveyard to the battlefield
/// **tapped**"). Isolated here so a future refactor that drops `dest_tapped` from
/// `MoveZone` fails independently of Victimize's other logic. CR 400.7: the moved
/// card is a NEW object on the battlefield.
#[test]
fn test_move_zone_returns_to_battlefield_tapped() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::{CardEffectTarget, Effect, ZoneTarget};

    // Assert both flag values so the test is non-vacuous: `tapped: true` must tap and
    // `tapped: false` must NOT — a `MoveZone` that ignores `dest_tapped` fails the
    // first assertion (enters untapped despite `tapped: true`).
    for want_tapped in [true, false] {
        let p1 = p(1);
        let p2 = p(2);

        let skeleton = ObjectSpec::creature(p1, "Tapped Return Test", 1, 1)
            .with_card_id(CardId("tapped-return-test".to_string()))
            .in_zone(ZoneId::Graveyard(p1));

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(vec![]))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .object(skeleton)
            .build()
            .unwrap();

        let gy_id = find_obj(&state, "Tapped Return Test");

        // `EffectTarget::Source` resolves to `ctx.source` — mirror reassembling_skeleton,
        // which returns itself from the graveyard.
        let mut ctx = EffectContext::new(p1, gy_id, vec![]);
        let effect = Effect::MoveZone {
            target: CardEffectTarget::Source,
            to: ZoneTarget::Battlefield {
                tapped: want_tapped,
            },
            controller_override: None,
        };
        let _events = execute_effect(&mut state, &effect, &mut ctx);

        assert!(
            on_battlefield(&state, "Tapped Return Test"),
            "MoveZone should have returned the card to the battlefield"
        );

        // CR 400.7: find the NEW battlefield object by name.
        let tapped = state
            .objects()
            .iter()
            .find(|(_, o)| {
                o.characteristics.name == "Tapped Return Test" && o.zone == ZoneId::Battlefield
            })
            .map(|(_, o)| o.status.tapped)
            .expect("returned object should be on the battlefield");

        assert_eq!(
            tapped, want_tapped,
            "MoveZone must apply ZoneTarget::Battlefield {{ tapped: {want_tapped} }} \
             (dest_tapped); got tapped={tapped}"
        );
    }
}

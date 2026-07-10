//! Tests for PB-AC3: Dynamic P/T & count amounts (CDA residual).
//!
//! Covers three net-new engine primitives:
//! - `EffectAmount::AttackingCreatureCount { controller, filter }` (CR 508.1/509)
//! - `EffectAmount::TappedCreatureCount { controller, filter }` (CR 613 / status.tapped)
//! - `LayerModification::SetBothDynamic { amount }` (CR 613.4b, Layer 7b dynamic base-P/T
//!   set, locked in at resolution per CR 608.2h/107.3k)
//!
//! Plus one coordinator-decision alias (`EffectAmount::HandSize`, CR 400) and the
//! already-shipped power-based token count (`TokenSpec.count: EffectAmount::PowerOf`,
//! CR 111.1), and card-integration tests against the actual shipped card definitions
//! (Keep Watch, Throne of the God-Pharaoh, Krenko Tin Street Kingpin, Mirror Entity).

use mtg_engine::cards::card_definition::{ContinuousEffectDef, EffectTarget};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::state::test_util;
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, AbilityDefinition, AttackTarget,
    CardType, Color, CombatState, CounterType, Effect, EffectAmount, EffectDuration, EffectFilter,
    EffectLayer, GameStateBuilder, LayerModification, ObjectId, ObjectSpec, PlayerId, PlayerTarget,
    Step, SubType, TokenSpec, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Build a name -> `CardDefinition` map for `enrich_spec_from_def` (mirrors builder_tests).
fn defs_map() -> std::collections::HashMap<String, mtg_engine::CardDefinition> {
    all_cards()
        .into_iter()
        .map(|c| (c.name.clone(), c))
        .collect()
}

/// Look up a shipped `CardDefinition` by exact name from the full card registry list.
fn find_card_def(name: &str) -> mtg_engine::CardDefinition {
    all_cards()
        .into_iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("card def '{}' not found in all_cards()", name))
}

// ── AttackingCreatureCount: resolve_amount (effect path) ──────────────────────

/// CR 508.1 / 613.1d — `AttackingCreatureCount { EachPlayer, None }` counts every
/// currently-declared attacker regardless of controller (Keep Watch mechanic).
/// Setup: p1 controls two creatures declared as attackers; p2 has a library of 3 cards.
/// `DrawCards { Controller, AttackingCreatureCount{EachPlayer, None} }` should draw 2.
#[test]
fn test_attacking_creature_count_basic() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker A", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker B", 2, 2))
        .object(ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 3").in_zone(ZoneId::Library(p1)))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let a_id = find_object(&state, "Attacker A");
    let b_id = find_object(&state, "Attacker B");

    *state.combat_mut() = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(a_id, AttackTarget::Player(p2));
        cs.attackers.insert(b_id, AttackTarget::Player(p2));
        cs
    });

    let hand_before = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let effect = Effect::DrawCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::AttackingCreatureCount {
            controller: PlayerTarget::EachPlayer,
            filter: None,
        },
    };
    let mut ctx = EffectContext::new(p1, a_id, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let hand_after = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    assert_eq!(
        hand_after - hand_before,
        2,
        "CR 508.1: should draw one card per attacking creature (2 attackers)"
    );
}

/// CR 508.1 — outside combat (`state.combat() == None`), `AttackingCreatureCount` resolves
/// to 0 (negative case).
#[test]
fn test_attacking_creature_count_zero_outside_combat() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Idle Bear", 2, 2))
        .build()
        .unwrap();
    assert!(state.combat().is_none());

    let source = find_object(&state, "Idle Bear");
    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::AttackingCreatureCount {
            controller: PlayerTarget::EachPlayer,
            filter: None,
        },
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players().get(&p1).unwrap().life_total,
        40,
        "no combat active -> AttackingCreatureCount resolves to 0 -> no life gained"
    );
}

/// CR 508.1 — creatures on the battlefield that are NOT declared attackers are not
/// counted, even if other creatures controlled by the same player are attacking.
#[test]
fn test_attacking_creature_count_ignores_nonattacking() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker", 2, 2))
        .object(ObjectSpec::creature(p1, "Bench Warmer", 2, 2))
        .object(ObjectSpec::creature(p1, "Also Idle", 2, 2))
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attacker");
    *state.combat_mut() = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::AttackingCreatureCount {
            controller: PlayerTarget::EachPlayer,
            filter: None,
        },
    };
    let mut ctx = EffectContext::new(p1, attacker_id, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players().get(&p1).unwrap().life_total,
        41,
        "only the 1 declared attacker counts, not the 2 idle creatures"
    );
}

// ── TappedCreatureCount: resolve_amount (effect path) ─────────────────────────

/// CR 613 / status.tapped — `TappedCreatureCount { Controller, None }` counts tapped
/// creatures controlled by the effect's controller (Throne of the God-Pharaoh mechanic).
/// p1 controls 3 creatures, 2 tapped; each opponent should lose 2 life.
#[test]
fn test_tapped_creature_count_basic() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Tapped A", 1, 1).tapped())
        .object(ObjectSpec::creature(p1, "Tapped B", 1, 1).tapped())
        .object(ObjectSpec::creature(p1, "Untapped C", 1, 1))
        .build()
        .unwrap();

    let source = find_object(&state, "Tapped A");
    let effect = Effect::LoseLife {
        player: PlayerTarget::EachOpponent,
        amount: EffectAmount::TappedCreatureCount {
            controller: PlayerTarget::Controller,
            filter: None,
        },
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players().get(&p2).unwrap().life_total,
        38,
        "2 tapped creatures -> opponent loses 2 life"
    );
}

/// CR 613 — `TappedCreatureCount { Controller, .. }` only counts the effect controller's
/// own tapped creatures, not an opponent's, even though the opponent has more.
#[test]
fn test_tapped_creature_count_controller_scope() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "P1 Untapped", 1, 1))
        .object(ObjectSpec::creature(p2, "P2 Tapped A", 1, 1).tapped())
        .object(ObjectSpec::creature(p2, "P2 Tapped B", 1, 1).tapped())
        .build()
        .unwrap();

    let source = find_object(&state, "P1 Untapped");
    let effect = Effect::LoseLife {
        player: PlayerTarget::EachOpponent,
        amount: EffectAmount::TappedCreatureCount {
            controller: PlayerTarget::Controller,
            filter: None,
        },
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players().get(&p2).unwrap().life_total,
        40,
        "p1 (the controller) has 0 tapped creatures, so no life should be lost \
         even though p2 has 2 tapped creatures"
    );
}

/// CR 702.26d — phased-out permanents are excluded from `TappedCreatureCount`, even if
/// their `status.tapped` flag happens to be true.
#[test]
fn test_tapped_creature_count_excludes_phased_out() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Tapped Present", 1, 1).tapped())
        .object(ObjectSpec::creature(p1, "Tapped Phased Out", 1, 1).tapped())
        .build()
        .unwrap();

    let phased_id = find_object(&state, "Tapped Phased Out");
    state
        .objects_mut()
        .get_mut(&phased_id)
        .unwrap()
        .status
        .phased_out = true;

    let source = find_object(&state, "Tapped Present");
    let effect = Effect::LoseLife {
        player: PlayerTarget::EachOpponent,
        amount: EffectAmount::TappedCreatureCount {
            controller: PlayerTarget::Controller,
            filter: None,
        },
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players().get(&p2).unwrap().life_total,
        39,
        "only the 1 phased-in tapped creature counts; the phased-out one is excluded"
    );
}

// ── HandSize alias ──────────────────────────────────────────────────────────

/// CR 400 — `EffectAmount::HandSize { player }` is a convenience alias that must
/// produce the identical result to `CardCount { zone: Hand, .. }`. p1 has 3 cards in hand.
#[test]
fn test_hand_size_matches_card_count_hand() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Source Bear", 2, 2))
        .object(ObjectSpec::card(p1, "Hand Card 1").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Hand Card 2").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Hand Card 3").in_zone(ZoneId::Hand(p1)))
        .build()
        .unwrap();

    let source = find_object(&state, "Source Bear");

    // Reset to a known baseline life total for a clean assertion.
    state.players_mut().get_mut(&p1).unwrap().life_total = 0;

    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::HandSize {
            player: PlayerTarget::Controller,
        },
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players().get(&p1).unwrap().life_total,
        3,
        "HandSize should count the 3 cards in p1's hand, identically to CardCount{{Hand}}"
    );
}

// ── SetBothDynamic: layer 7b dynamic base-P/T set ──────────────────────────────

/// CR 613.4b / 608.2h / 107.3k — activating a Mirror-Entity-style `{X}` ability with
/// `SetBothDynamic { XValue }` sets every matching creature's BASE power and toughness
/// to X. Also verifies the CR 608.2h lock-in substitution: the stored `ContinuousEffect`
/// must carry a concrete `SetPowerToughness(v, v)`, never the dynamic placeholder.
#[test]
fn test_set_both_dynamic_sets_base_pt() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Board Bear A", 1, 1))
        .object(ObjectSpec::creature(p1, "Board Bear B", 1, 1))
        .object(ObjectSpec::creature(p2, "Opponent Bear", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source = find_object(&state, "Board Bear A");
    let bear_b = find_object(&state, "Board Bear B");
    let opp_bear = find_object(&state, "Opponent Bear");

    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtSet,
            modification: LayerModification::SetBothDynamic {
                amount: Box::new(EffectAmount::XValue),
            },
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    ctx.x_value = 3;
    execute_effect(&mut state, &effect, &mut ctx);

    // CR 608.2h: the stored effect must be a concrete SetPowerToughness(3, 3), not
    // the dynamic placeholder (locked in at resolution).
    let dynamic_in_effects = state
        .continuous_effects()
        .iter()
        .any(|e| matches!(&e.modification, LayerModification::SetBothDynamic { .. }));
    assert!(
        !dynamic_in_effects,
        "CR 608.2h: SetBothDynamic must be substituted before storage"
    );
    let has_set_3_3 = state.continuous_effects().iter().any(|e| {
        matches!(
            &e.modification,
            LayerModification::SetPowerToughness {
                power: 3,
                toughness: 3
            }
        )
    });
    assert!(
        has_set_3_3,
        "CR 608.2h: stored effect should be SetPowerToughness(3, 3)"
    );

    let chars_a = calculate_characteristics(&state, source).unwrap();
    assert_eq!(chars_a.power, Some(3), "Board Bear A base P/T set to 3/3");
    assert_eq!(chars_a.toughness, Some(3));

    let chars_b = calculate_characteristics(&state, bear_b).unwrap();
    assert_eq!(
        chars_b.power,
        Some(3),
        "CreaturesYouControl applies to ALL of p1's creatures, not just the source"
    );

    let chars_opp = calculate_characteristics(&state, opp_bear).unwrap();
    assert_eq!(
        chars_opp.power,
        Some(1),
        "opponent's creature is untouched (filter is CreaturesYouControl)"
    );
}

/// CR 611.2c / 608.2h — after the ability resolves and X is locked in, a NEW creature
/// that later matches the `CreaturesYouControl` filter also gets the SAME locked-in
/// value (membership re-evaluated continuously), not a stale or zero value.
#[test]
fn test_set_both_dynamic_locked_at_resolution() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Original Bear", 1, 1))
        .object(ObjectSpec::creature(p1, "Late Arrival Bear", 1, 1).in_zone(ZoneId::Hand(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source = find_object(&state, "Original Bear");
    let late_arrival_hand_id = find_object(&state, "Late Arrival Bear");

    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtSet,
            modification: LayerModification::SetBothDynamic {
                amount: Box::new(EffectAmount::XValue),
            },
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    ctx.x_value = 3;
    execute_effect(&mut state, &effect, &mut ctx);

    // A new creature enters the battlefield AFTER the ability already resolved (and
    // locked X=3). CR 400.7: moving zones assigns a fresh ObjectId.
    let (new_id, _old) =
        test_util::move_object_to_zone(&mut state, late_arrival_hand_id, ZoneId::Battlefield)
            .unwrap();

    let chars = calculate_characteristics(&state, new_id).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "CR 611.2c: a creature entering after resolution still gets the locked-in X=3 \
         because the continuous effect's filter membership is re-evaluated continuously, \
         while the VALUE itself stays locked at 3 (not re-derived from a live X)"
    );
}

/// CR 613.4b vs 613.4c — SetBothDynamic (Layer 7b, sets BASE P/T) applies before a
/// +1/+1 counter (Layer 7c, modifies). Base set to 3/3, then a +1/+1 counter -> 4/4.
#[test]
fn test_set_both_dynamic_then_counter_layer_order() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Countered Bear", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source = find_object(&state, "Countered Bear");

    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtSet,
            modification: LayerModification::SetBothDynamic {
                amount: Box::new(EffectAmount::XValue),
            },
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    ctx.x_value = 3;
    execute_effect(&mut state, &effect, &mut ctx);

    // CR 613.4c: a +1/+1 counter modifies on top of the Layer-7b set value.
    state
        .objects_mut()
        .get_mut(&source)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 1);

    let chars = calculate_characteristics(&state, source).unwrap();
    assert_eq!(
        chars.power,
        Some(4),
        "CR 613.4b then 613.4c: base set to 3 (7b), then +1/+1 counter (7c) -> 4"
    );
    assert_eq!(chars.toughness, Some(4));
}

/// CR 613.4c — SetBothDynamic (Layer 7b) plus an external anthem (ModifyBoth, Layer 7c)
/// both apply: base 3/3 + anthem +1/+1 = 4/4.
#[test]
fn test_set_both_dynamic_then_anthem() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Anthemed Bear", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source = find_object(&state, "Anthemed Bear");

    let set_effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtSet,
            modification: LayerModification::SetBothDynamic {
                amount: Box::new(EffectAmount::XValue),
            },
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    ctx.x_value = 3;
    execute_effect(&mut state, &set_effect, &mut ctx);

    // External anthem: +1/+1 to creatures you control (Layer 7c modify).
    let anthem_effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(1),
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx2 = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &anthem_effect, &mut ctx2);

    let chars = calculate_characteristics(&state, source).unwrap();
    assert_eq!(
        chars.power,
        Some(4),
        "CR 613.4b set to 3/3, CR 613.4c anthem +1/+1 -> 4/4"
    );
    assert_eq!(chars.toughness, Some(4));
}

/// Full-dispatch: creatures gain all creature types (Layer 4 type-changing
/// AddAllCreatureTypes, CR 613.1d) alongside the base-P/T set (Layer 7b),
/// mirroring Mirror Entity's full ability text.
#[test]
fn test_set_both_dynamic_with_all_creature_types() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Shifting Bear", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source = find_object(&state, "Shifting Bear");

    let pt_effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::PtSet,
            modification: LayerModification::SetBothDynamic {
                amount: Box::new(EffectAmount::XValue),
            },
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let types_effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::TypeChange,
            modification: LayerModification::AddAllCreatureTypes,
            filter: EffectFilter::CreaturesYouControl,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    ctx.x_value = 3;
    execute_effect(&mut state, &pt_effect, &mut ctx);
    let mut ctx2 = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &types_effect, &mut ctx2);

    let chars = calculate_characteristics(&state, source).unwrap();
    assert_eq!(chars.power, Some(3));
    assert_eq!(chars.toughness, Some(3));
    assert!(
        chars.subtypes.contains(&SubType("Goblin".to_string())),
        "CR 205.3m: AddAllCreatureTypes grants every creature type, e.g. Goblin"
    );
    assert!(
        chars.subtypes.contains(&SubType("Dragon".to_string())),
        "CR 205.3m: AddAllCreatureTypes grants every creature type, e.g. Dragon"
    );
}

// ── Power-based token count (already-shipped TokenSpec.count: EffectAmount::PowerOf) ──

/// CR 111.1 / CR 613.1: a token-creation effect can size its count off a live,
/// layer-resolved power (Krenko, Tin Street Kingpin mechanic). A +1/+1 counter added
/// in the same Sequence before token creation is reflected in the token count.
#[test]
fn test_power_based_token_count() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Goblin Boss", 1, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source = find_object(&state, "Goblin Boss");

    let effect = Effect::Sequence(vec![
        Effect::AddCounter {
            target: EffectTarget::Source,
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        },
        Effect::CreateToken {
            spec: TokenSpec {
                name: "Goblin Token".to_string(),
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                colors: [Color::Red].into_iter().collect(),
                power: 1,
                toughness: 1,
                count: EffectAmount::PowerOf(EffectTarget::Source),
                supertypes: Default::default(),
                keywords: Default::default(),
                tapped: false,
                enters_attacking: false,
                mana_color: None,
                mana_abilities: vec![],
                activated_abilities: vec![],
                ..Default::default()
            },
        },
    ]);
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let token_count = state
        .objects()
        .values()
        .filter(|o| {
            o.characteristics.name == "Goblin Token" && o.zone == ZoneId::Battlefield && o.is_token
        })
        .count();

    // Goblin Boss: base power 1 + the counter just added = 2 -> 2 tokens.
    assert_eq!(
        token_count, 2,
        "power 2 (1 base + 1 counter added in the same Sequence) -> 2 tokens"
    );
}

// ── Card-integration tests (actual shipped CardDefinition entries) ────────────

/// CR 508.1 — Keep Watch's shipped `AbilityDefinition::Spell` effect draws one card
/// per attacking creature.
#[test]
fn test_keep_watch_draws_per_attacker() {
    let def = find_card_def("Keep Watch");
    let effect = match &def.abilities[0] {
        AbilityDefinition::Spell { effect, .. } => effect.clone(),
        other => panic!("expected AbilityDefinition::Spell, got {:?}", other),
    };

    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker A", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker B", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker C", 2, 2))
        .object(ObjectSpec::card(p1, "Library Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 2").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Library Card 3").in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let a = find_object(&state, "Attacker A");
    let b = find_object(&state, "Attacker B");
    let c = find_object(&state, "Attacker C");
    *state.combat_mut() = Some({
        let mut cs = CombatState::new(p1);
        cs.attackers.insert(a, AttackTarget::Player(p2));
        cs.attackers.insert(b, AttackTarget::Player(p2));
        cs.attackers.insert(c, AttackTarget::Player(p2));
        cs
    });

    let hand_before = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    let mut ctx = EffectContext::new(p1, a, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);
    let hand_after = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    assert_eq!(hand_after - hand_before, 3, "3 attackers -> draw 3 cards");
}

/// CR 613 — Throne of the God-Pharaoh's shipped `AbilityDefinition::Triggered` effect
/// drains life from opponents equal to the controller's tapped creature count.
#[test]
fn test_throne_end_step_drains_per_tapped_creature() {
    let def = find_card_def("Throne of the God-Pharaoh");
    let effect = match &def.abilities[0] {
        AbilityDefinition::Triggered { effect, .. } => effect.clone(),
        other => panic!("expected AbilityDefinition::Triggered, got {:?}", other),
    };

    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Tapped Servant A", 1, 1).tapped())
        .object(ObjectSpec::creature(p1, "Tapped Servant B", 1, 1).tapped())
        .object(ObjectSpec::creature(p1, "Tapped Servant C", 1, 1).tapped())
        .build()
        .unwrap();

    let source = find_object(&state, "Tapped Servant A");
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players().get(&p2).unwrap().life_total,
        37,
        "3 tapped creatures -> opponent loses 3 life"
    );
}

/// CR 111.1 — Krenko, Tin Street Kingpin's shipped `AbilityDefinition::Triggered`
/// effect creates a number of tokens equal to its power (after the +1/+1 counter
/// from the same trigger has been applied).
#[test]
fn test_krenko_tokens_equal_power() {
    let def = find_card_def("Krenko, Tin Street Kingpin");
    let effect = match &def.abilities[0] {
        AbilityDefinition::Triggered { effect, .. } => effect.clone(),
        other => panic!("expected AbilityDefinition::Triggered, got {:?}", other),
    };

    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Krenko, Tin Street Kingpin", 1, 2))
        .build()
        .unwrap();

    let krenko = find_object(&state, "Krenko, Tin Street Kingpin");
    let mut ctx = EffectContext::new(p1, krenko, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let goblin_count = state
        .objects()
        .values()
        .filter(|o| o.characteristics.name == "Goblin" && o.zone == ZoneId::Battlefield)
        .count();

    // Krenko: base power 1 + the +1/+1 counter this trigger adds = 2 -> 2 tokens.
    assert_eq!(
        goblin_count, 2,
        "Krenko's power after its own +1/+1 counter (1 + 1 = 2) -> 2 Goblin tokens"
    );
}

/// CR 613.4b — Mirror Entity's shipped `AbilityDefinition::Activated` ability pumps
/// creatures you control to base X/X and grants all creature types.
#[test]
fn test_mirror_entity_pumps_and_types() {
    let def = find_card_def("Mirror Entity");
    // abilities[0] = Keyword(Changeling), abilities[1] = Activated{X}.
    let (effect, _cost) = match &def.abilities[1] {
        AbilityDefinition::Activated { effect, cost, .. } => (effect.clone(), cost.clone()),
        other => panic!("expected AbilityDefinition::Activated, got {:?}", other),
    };

    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Mirror Entity", 1, 1))
        .object(ObjectSpec::creature(p1, "Ally Bear", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source = find_object(&state, "Mirror Entity");
    let ally = find_object(&state, "Ally Bear");

    let mut ctx = EffectContext::new(p1, source, vec![]);
    ctx.x_value = 4;
    execute_effect(&mut state, &effect, &mut ctx);

    let chars_source = calculate_characteristics(&state, source).unwrap();
    assert_eq!(chars_source.power, Some(4));
    assert_eq!(chars_source.toughness, Some(4));
    assert!(chars_source
        .subtypes
        .contains(&SubType("Merfolk".to_string())));

    let chars_ally = calculate_characteristics(&state, ally).unwrap();
    assert_eq!(
        chars_ally.power,
        Some(4),
        "the ability pumps ALL creatures you control, not just Mirror Entity itself"
    );
    assert!(chars_ally
        .subtypes
        .contains(&SubType("Merfolk".to_string())));
}

// ── Hash schema ─────────────────────────────────────────────────────────────

/// PB-AC3 bumped `HASH_SCHEMA_VERSION` 29 -> 30 (new `EffectAmount` variants
/// `AttackingCreatureCount`/`TappedCreatureCount`/`HandSize` at discriminants 19/20/21,
/// new `LayerModification::SetBothDynamic` at discriminant 28, and the `RemoveSuperType`
/// hash-collision fix reassigning it to discriminant 29). If you bumped again, update
/// this test and the `state/hash.rs` history block.
#[test]
fn test_hash_schema_version_is_30() {
    assert_eq!(mtg_engine::HASH_SCHEMA_VERSION, 37u8);
}

/// Hash soundness / determinism: the three new `EffectAmount` variants and the new
/// `LayerModification::SetBothDynamic` variant each (a) hash deterministically across
/// repeated calls and (b) hash distinctly from each other and from pre-existing
/// neighboring variants (guards against a repeat of the pre-existing disc-26 collision
/// this batch fixed between `RemoveSuperType` and `ModifyPowerDynamic`).
#[test]
fn test_hash_distinguishes_new_variants_and_fixes_collision() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let hash_amount = |a: &EffectAmount| -> [u8; 32] {
        let mut h = Hasher::new();
        a.hash_into(&mut h);
        *h.finalize().as_bytes()
    };
    let hash_lm = |m: &LayerModification| -> [u8; 32] {
        let mut h = Hasher::new();
        m.hash_into(&mut h);
        *h.finalize().as_bytes()
    };

    let attacking = EffectAmount::AttackingCreatureCount {
        controller: PlayerTarget::EachPlayer,
        filter: None,
    };
    let tapped = EffectAmount::TappedCreatureCount {
        controller: PlayerTarget::Controller,
        filter: None,
    };
    let hand_size = EffectAmount::HandSize {
        player: PlayerTarget::Controller,
    };

    // Determinism: hashing twice yields the same digest.
    assert_eq!(hash_amount(&attacking), hash_amount(&attacking));
    assert_eq!(hash_amount(&tapped), hash_amount(&tapped));
    assert_eq!(hash_amount(&hand_size), hash_amount(&hand_size));

    // Distinctness among the three new variants.
    assert_ne!(hash_amount(&attacking), hash_amount(&tapped));
    assert_ne!(hash_amount(&attacking), hash_amount(&hand_size));
    assert_ne!(hash_amount(&tapped), hash_amount(&hand_size));

    // Distinctness against a filtered variant of the same kind (controller differs).
    let attacking_controller_scoped = EffectAmount::AttackingCreatureCount {
        controller: PlayerTarget::Controller,
        filter: None,
    };
    assert_ne!(
        hash_amount(&attacking),
        hash_amount(&attacking_controller_scoped)
    );

    // SetBothDynamic hashes deterministically and distinctly from its sibling
    // pre-existing *Dynamic variants (same amount, different variant tag).
    let set_both = LayerModification::SetBothDynamic {
        amount: Box::new(EffectAmount::XValue),
    };
    let modify_both = LayerModification::ModifyBothDynamic {
        amount: Box::new(EffectAmount::XValue),
        negate: false,
    };
    assert_eq!(hash_lm(&set_both), hash_lm(&set_both));
    assert_ne!(hash_lm(&set_both), hash_lm(&modify_both));

    // Collision fix regression guard: RemoveSuperType (reassigned 26 -> 29) must no
    // longer collide with ModifyPowerDynamic (still 26).
    let remove_super_type =
        LayerModification::RemoveSuperType(mtg_engine::state::types::SuperType::Legendary);
    let modify_power_dynamic = LayerModification::ModifyPowerDynamic {
        amount: Box::new(EffectAmount::XValue),
        negate: false,
    };
    assert_ne!(
        hash_lm(&remove_super_type),
        hash_lm(&modify_power_dynamic),
        "PB-AC3 collision fix: RemoveSuperType and ModifyPowerDynamic must hash \
         distinctly (previously both hashed discriminant prefix 26u8)"
    );
}

// ── PB-AC3 card backfill regression: dying-0/0 CDA fixes ──────────────────────

/// CR 613.4c / 107.3 — Ashaya, Soul of the Wild is a `*/*` whose power and toughness
/// each equal the number of lands you control. Regression guard: this card previously
/// shipped with `power/toughness: None` but NO CDA ability, so it resolved to 0/0 and
/// died to SBA on entry. The `CdaPowerToughness { PermanentCount{Land, Controller} }`
/// ability must yield P/T = land count.
#[test]
fn test_ashaya_pt_equals_lands_you_control() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = mtg_engine::CardRegistry::new(all_cards());

    let ashaya_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Ashaya, Soul of the Wild")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(mtg_engine::card_name_to_id("Ashaya, Soul of the Wild")),
        &defs,
    );
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(std::sync::Arc::clone(&registry))
        .object(ashaya_spec);
    // Three lands you control; one land an opponent controls (must NOT count).
    for _ in 0..3 {
        builder = builder.object(ObjectSpec::land(p1, "Forest"));
    }
    builder = builder.object(ObjectSpec::land(p2, "Forest"));
    let mut state = builder.build().unwrap();

    let ashaya = find_object(&state, "Ashaya, Soul of the Wild");
    // Builder bypasses the ETB path; register the CDA static effect manually.
    let card_id = state.objects().get(&ashaya).and_then(|o| o.card_id.clone());
    mtg_engine::rules::replacement::register_static_continuous_effects(
        &mut state,
        ashaya,
        card_id.as_ref(),
        &registry,
    );

    let chars = calculate_characteristics(&state, ashaya).unwrap();
    assert_eq!(
        chars.power,
        Some(3),
        "Ashaya power = 3 lands you control (opponent's land excluded)"
    );
    assert_eq!(
        chars.toughness,
        Some(3),
        "Ashaya toughness = lands you control"
    );
}

/// CR 613.4c — Multani, Yavimaya's Avatar is a base 0/0 that gets +1/+1 for each land you
/// control AND each land card in your graveyard (a `Sum` of two counts). Regression guard:
/// previously shipped as a base 0/0 with no pump (dying to SBA). Setup: 2 lands on the
/// battlefield + 2 land cards in the graveyard -> +4/+4 -> 4/4.
#[test]
fn test_multani_pt_sums_lands_and_graveyard_lands() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = mtg_engine::CardRegistry::new(all_cards());

    let multani_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Multani, Yavimaya's Avatar")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(mtg_engine::card_name_to_id("Multani, Yavimaya's Avatar")),
        &defs,
    );
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(std::sync::Arc::clone(&registry))
        .object(multani_spec);
    // Two lands on the battlefield.
    for _ in 0..2 {
        builder = builder.object(ObjectSpec::land(p1, "Forest"));
    }
    // Two land cards in the graveyard.
    for _ in 0..2 {
        builder = builder.object(ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Graveyard(p1)));
    }
    let mut state = builder.build().unwrap();

    let multani = find_object(&state, "Multani, Yavimaya's Avatar");
    // Builder bypasses the ETB path; register the CDA static effect manually.
    let card_id = state
        .objects()
        .get(&multani)
        .and_then(|o| o.card_id.clone());
    mtg_engine::rules::replacement::register_static_continuous_effects(
        &mut state,
        multani,
        card_id.as_ref(),
        &registry,
    );

    let chars = calculate_characteristics(&state, multani).unwrap();
    assert_eq!(
        chars.power,
        Some(4),
        "Multani power = 2 battlefield lands + 2 graveyard land cards = 4"
    );
    assert_eq!(
        chars.toughness,
        Some(4),
        "Multani toughness = 2 battlefield lands + 2 graveyard land cards = 4"
    );
}

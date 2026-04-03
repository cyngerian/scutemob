//! Tests for PB-D: Chosen Creature Type primitives.
//!
//! Covers:
//! - `EffectFilter::CreaturesYouControlOfChosenType` — layer 7c anthem for matching creatures
//! - `EffectFilter::OtherCreaturesYouControlOfChosenType` — same but excludes source (Morophon)
//! - `TriggerCondition::WheneverYouCastSpell { chosen_subtype_filter: true }` — Vanquisher's Banner
//! - `SpellCostModifier.colored_mana_reduction` — Morophon colored mana discount definition
//! - `EffectContext.chosen_creature_type` — spell-level type choice in Sequence
//! - `Effect::DestroyAll` with `exclude_chosen_subtype: true` — Kindred Dominance
//! - `Condition::TopCardIsCreatureOfChosenType` — Herald's Horn upkeep draw
//!
//! CR references: 205.3m, 601.2f, 614.1c

use mtg_engine::cards::card_definition::Condition;
use mtg_engine::effects::{check_condition, execute_effect, EffectContext};
use mtg_engine::state::continuous_effect::{
    ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
};
use mtg_engine::state::types::SubType;
use mtg_engine::{
    all_cards, calculate_characteristics, CardDefinition, Effect, GameState, GameStateBuilder,
    ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn load_defs() -> HashMap<String, CardDefinition> {
    let cards = all_cards();
    cards.iter().map(|d| (d.name.clone(), d.clone())).collect()
}

fn make_ctx(state: &GameState, source: ObjectId) -> EffectContext {
    let controller = state
        .objects
        .get(&source)
        .map(|o| o.controller)
        .unwrap_or(p(1));
    EffectContext::new(controller, source, vec![])
}

fn make_effect(
    id: u64,
    source_id: ObjectId,
    layer: EffectLayer,
    modification: LayerModification,
    filter: EffectFilter,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: Some(source_id),
        timestamp: id,
        layer,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter,
        modification,
        is_cda: false,
        condition: None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 614.1c — Effect::ChooseCreatureType sets chosen_creature_type on the source permanent
/// and also sets ctx.chosen_creature_type for spell-level use in Sequences.
#[test]
fn test_chosen_creature_type_etb_sets_type() {
    let p1 = p(1);

    let source = ObjectSpec::artifact(p1, "Banner Source").in_zone(ZoneId::Battlefield);
    // An Elf on the battlefield so ChooseCreatureType picks "Elf".
    let elf = ObjectSpec::creature(p1, "Elvish Mystic", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(elf)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Banner Source");
    let mut state2 = state;
    let mut ctx = make_ctx(&state2, source_id);

    let _events = execute_effect(
        &mut state2,
        &Effect::ChooseCreatureType {
            default: SubType("Human".to_string()),
        },
        &mut ctx,
    );

    // The source permanent should have chosen_creature_type set.
    let chosen_on_perm = state2
        .objects
        .get(&source_id)
        .and_then(|o| o.chosen_creature_type.clone());
    assert_eq!(
        chosen_on_perm,
        Some(SubType("Elf".to_string())),
        "chosen_creature_type on permanent should be 'Elf' (most common) — CR 614.1c"
    );

    // ctx.chosen_creature_type should also be set for spell-level use.
    assert_eq!(
        ctx.chosen_creature_type,
        Some(SubType("Elf".to_string())),
        "ctx.chosen_creature_type should be set for Sequence propagation"
    );
}

/// CR 205.3m — CreaturesYouControlOfChosenType anthem applies +1/+1 to matching creatures only.
#[test]
fn test_chosen_type_anthem_basic() {
    let p1 = p(1);

    let banner = ObjectSpec::artifact(p1, "Vanquisher Banner").in_zone(ZoneId::Battlefield);
    let elf = ObjectSpec::creature(p1, "Test Elf", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human = ObjectSpec::creature(p1, "Test Human", 1, 1)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(banner)
        .object(elf)
        .object(human)
        .build()
        .unwrap();

    let banner_id = find_object(&state, "Vanquisher Banner");
    let elf_id = find_object(&state, "Test Elf");
    let human_id = find_object(&state, "Test Human");

    let mut state2 = state;
    // Set chosen type on the banner.
    if let Some(obj) = state2.objects.get_mut(&banner_id) {
        obj.chosen_creature_type = Some(SubType("Elf".to_string()));
    }

    // Register the anthem as a continuous effect.
    let effect = make_effect(
        1,
        banner_id,
        EffectLayer::PtModify,
        LayerModification::ModifyBoth(1),
        EffectFilter::CreaturesYouControlOfChosenType,
    );
    state2.continuous_effects.push_back(effect);

    let elf_chars = calculate_characteristics(&state2, elf_id).unwrap();
    let human_chars = calculate_characteristics(&state2, human_id).unwrap();

    assert_eq!(
        elf_chars.power,
        Some(2),
        "Elf should get +1 power from anthem (CR 205.3m)"
    );
    assert_eq!(
        elf_chars.toughness,
        Some(2),
        "Elf should get +1 toughness from anthem"
    );
    assert_eq!(
        human_chars.power,
        Some(1),
        "Human should NOT get +1 from anthem"
    );
    assert_eq!(
        human_chars.toughness,
        Some(1),
        "Human should NOT get +1 toughness"
    );
}

/// CR 205.3m — OtherCreaturesYouControlOfChosenType excludes the source permanent itself.
#[test]
fn test_chosen_type_anthem_other() {
    let p1 = p(1);

    let lord = ObjectSpec::creature(p1, "Lord Elf", 6, 6)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let other_elf = ObjectSpec::creature(p1, "Other Elf", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(lord)
        .object(other_elf)
        .build()
        .unwrap();

    let lord_id = find_object(&state, "Lord Elf");
    let other_elf_id = find_object(&state, "Other Elf");

    let mut state2 = state;
    if let Some(obj) = state2.objects.get_mut(&lord_id) {
        obj.chosen_creature_type = Some(SubType("Elf".to_string()));
    }

    let effect = make_effect(
        1,
        lord_id,
        EffectLayer::PtModify,
        LayerModification::ModifyBoth(1),
        EffectFilter::OtherCreaturesYouControlOfChosenType,
    );
    state2.continuous_effects.push_back(effect);

    let lord_chars = calculate_characteristics(&state2, lord_id).unwrap();
    let other_chars = calculate_characteristics(&state2, other_elf_id).unwrap();

    assert_eq!(
        lord_chars.power,
        Some(6),
        "Lord should NOT benefit from its own OtherCreatures anthem"
    );
    assert_eq!(
        other_chars.power,
        Some(2),
        "Other Elf should get +1 from OtherCreatures anthem (CR 205.3m)"
    );
}

/// CR 601.2f — SpellCostModifier.colored_mana_reduction on Morophon's definition is set correctly.
#[test]
fn test_chosen_type_cost_reduction_colored() {
    let defs = load_defs();
    let morophon = defs
        .get("Morophon, the Boundless")
        .expect("Morophon, the Boundless should be in defs");

    assert!(
        !morophon.spell_cost_modifiers.is_empty(),
        "Morophon should have spell_cost_modifiers"
    );
    let modifier = &morophon.spell_cost_modifiers[0];
    let cr = modifier
        .colored_mana_reduction
        .as_ref()
        .expect("Morophon should have colored_mana_reduction");

    assert_eq!(
        modifier.change, 0,
        "Morophon has no generic change (only colored)"
    );
    assert_eq!(cr.white, 1, "Morophon reduces W by 1 (CR 601.2f)");
    assert_eq!(cr.blue, 1, "Morophon reduces U by 1 (CR 601.2f)");
    assert_eq!(cr.black, 1, "Morophon reduces B by 1 (CR 601.2f)");
    assert_eq!(cr.red, 1, "Morophon reduces R by 1 (CR 601.2f)");
    assert_eq!(cr.green, 1, "Morophon reduces G by 1 (CR 601.2f)");
}

/// CR 601.2f — Herald's Horn gives {1} reduction for creature spells of the chosen type.
#[test]
fn test_chosen_type_cost_reduction_generic() {
    let defs = load_defs();
    let horn = defs
        .get("Herald's Horn")
        .expect("Herald's Horn should be in defs");

    assert!(
        !horn.spell_cost_modifiers.is_empty(),
        "Herald's Horn should have spell_cost_modifiers"
    );
    let modifier = &horn.spell_cost_modifiers[0];
    assert_eq!(
        modifier.change, -1,
        "Herald's Horn reduces generic cost by 1 (CR 601.2f)"
    );

    use mtg_engine::cards::card_definition::SpellCostFilter;
    assert!(
        matches!(modifier.filter, SpellCostFilter::HasChosenCreatureSubtype),
        "Herald's Horn modifier filter should be HasChosenCreatureSubtype"
    );
}

/// CR 205.3m — EffectContext.chosen_creature_type propagates to exclude_chosen_subtype filter.
/// Kindred Dominance: ChooseCreatureType sets ctx type, DestroyAll skips creatures of that type.
#[test]
fn test_chosen_type_spell_level_choice_propagates() {
    use mtg_engine::cards::card_definition::TargetFilter;
    use mtg_engine::CardType;

    let p1 = p(1);

    // Two Elves and one Human so that ChooseCreatureType deterministically picks "Elf".
    let elf = ObjectSpec::creature(p1, "Survivor Elf", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let elf2 = ObjectSpec::creature(p1, "Survivor Elf 2", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human = ObjectSpec::creature(p1, "Doomed Human", 1, 1)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);
    let source = ObjectSpec::artifact(p1, "Spell Source").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(elf)
        .object(elf2)
        .object(human)
        .object(source)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Spell Source");
    let mut state2 = state;
    let mut ctx = EffectContext::new(p1, source_id, vec![]);

    // Execute Sequence: ChooseCreatureType (picks Elf — most common, 2 vs 1) + DestroyAll(exclude_chosen).
    let _events = execute_effect(
        &mut state2,
        &Effect::Sequence(vec![
            Effect::ChooseCreatureType {
                default: SubType("Elf".to_string()),
            },
            Effect::DestroyAll {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    exclude_chosen_subtype: true,
                    ..Default::default()
                },
                cant_be_regenerated: false,
            },
        ]),
        &mut ctx,
    );

    assert!(
        ctx.chosen_creature_type.is_some(),
        "ctx.chosen_creature_type should be set after ChooseCreatureType in Sequence"
    );

    let elf_id = find_object(&state2, "Survivor Elf");
    let human_id = find_object(&state2, "Doomed Human");

    assert_eq!(
        state2.objects.get(&elf_id).map(|o| o.zone.clone()),
        Some(ZoneId::Battlefield),
        "Elf should survive (of chosen type) — CR 205.3m"
    );
    assert!(
        matches!(
            state2.objects.get(&human_id).map(|o| &o.zone),
            Some(ZoneId::Graveyard(_))
        ),
        "Human should be destroyed (not of chosen type) — CR 205.3m"
    );
}

/// CR 205.3m — Vanquisher's Banner definition has chosen_subtype_filter=true on its cast trigger.
#[test]
fn test_chosen_type_cast_trigger_definition() {
    let defs = load_defs();
    let banner = defs
        .get("Vanquisher's Banner")
        .expect("Vanquisher's Banner should be in defs");

    use mtg_engine::cards::card_definition::{AbilityDefinition, TriggerCondition};
    let has_chosen_filter = banner.abilities.iter().any(|ab| {
        if let AbilityDefinition::Triggered {
            trigger_condition:
                TriggerCondition::WheneverYouCastSpell {
                    chosen_subtype_filter,
                    ..
                },
            ..
        } = ab
        {
            *chosen_subtype_filter
        } else {
            false
        }
    });

    assert!(
        has_chosen_filter,
        "Vanquisher's Banner should have WheneverYouCastSpell with chosen_subtype_filter=true"
    );
}

/// CR 205.3m — DestroyAll with exclude_chosen_subtype only destroys non-chosen-type creatures.
#[test]
fn test_chosen_type_destroy_all_except() {
    use mtg_engine::cards::card_definition::TargetFilter;
    use mtg_engine::CardType;

    let p1 = p(1);

    let elf1 = ObjectSpec::creature(p1, "Elf A", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let elf2 = ObjectSpec::creature(p1, "Elf B", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human1 = ObjectSpec::creature(p1, "Human A", 1, 1)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human2 = ObjectSpec::creature(p1, "Human B", 1, 1)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);
    let source = ObjectSpec::artifact(p1, "Spell Src").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(elf1)
        .object(elf2)
        .object(human1)
        .object(human2)
        .object(source)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Spell Src");
    let mut state2 = state;
    // Pre-set chosen_creature_type on ctx directly.
    let mut ctx = EffectContext::new(p1, source_id, vec![]);
    ctx.chosen_creature_type = Some(SubType("Elf".to_string()));

    let _events = execute_effect(
        &mut state2,
        &Effect::DestroyAll {
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                exclude_chosen_subtype: true,
                ..Default::default()
            },
            cant_be_regenerated: false,
        },
        &mut ctx,
    );

    let elf_a_id = find_object(&state2, "Elf A");
    let elf_b_id = find_object(&state2, "Elf B");
    let human_a_id = find_object(&state2, "Human A");
    let human_b_id = find_object(&state2, "Human B");

    assert_eq!(
        state2.objects.get(&elf_a_id).map(|o| o.zone.clone()),
        Some(ZoneId::Battlefield),
        "Elf A should survive (chosen type)"
    );
    assert_eq!(
        state2.objects.get(&elf_b_id).map(|o| o.zone.clone()),
        Some(ZoneId::Battlefield),
        "Elf B should survive (chosen type)"
    );
    assert!(
        matches!(
            state2.objects.get(&human_a_id).map(|o| &o.zone),
            Some(ZoneId::Graveyard(_))
        ),
        "Human A should be destroyed"
    );
    assert!(
        matches!(
            state2.objects.get(&human_b_id).map(|o| &o.zone),
            Some(ZoneId::Graveyard(_))
        ),
        "Human B should be destroyed"
    );
}

/// CR 614.1c — Condition::TopCardIsCreatureOfChosenType: true when top is a creature of chosen type.
#[test]
fn test_top_card_creature_of_chosen_type_true() {
    let p1 = p(1);

    let source = ObjectSpec::artifact(p1, "Horn Source").in_zone(ZoneId::Battlefield);
    let top = ObjectSpec::creature(p1, "Library Elf", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(source)
        .object(top)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Horn Source");
    let mut state2 = state;
    if let Some(obj) = state2.objects.get_mut(&source_id) {
        obj.chosen_creature_type = Some(SubType("Elf".to_string()));
    }

    let ctx = make_ctx(&state2, source_id);
    let result = check_condition(&state2, &Condition::TopCardIsCreatureOfChosenType, &ctx);

    assert!(
        result,
        "TopCardIsCreatureOfChosenType should be true — CR 614.1c"
    );
}

/// CR 614.1c — TopCardIsCreatureOfChosenType: false when top card is wrong creature type.
#[test]
fn test_top_card_creature_of_chosen_type_false() {
    let p1 = p(1);

    let source = ObjectSpec::artifact(p1, "Horn Source").in_zone(ZoneId::Battlefield);
    let top = ObjectSpec::creature(p1, "Library Human", 1, 1)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(source)
        .object(top)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Horn Source");
    let mut state2 = state;
    if let Some(obj) = state2.objects.get_mut(&source_id) {
        obj.chosen_creature_type = Some(SubType("Elf".to_string()));
    }

    let ctx = make_ctx(&state2, source_id);
    let result = check_condition(&state2, &Condition::TopCardIsCreatureOfChosenType, &ctx);

    assert!(
        !result,
        "TopCardIsCreatureOfChosenType should be false (wrong type)"
    );
}

/// CR 205.3m — Pact of the Serpent def: ChosenTypeCreatureCount used for draw + life loss.
#[test]
fn test_pact_of_the_serpent_definition() {
    let defs = load_defs();
    let pact = defs
        .get("Pact of the Serpent")
        .expect("Pact of the Serpent should be in defs");

    use mtg_engine::cards::card_definition::{AbilityDefinition, Effect, EffectAmount};
    fn has_chosen_count(e: &Effect) -> bool {
        match e {
            Effect::Sequence(effects) => effects.iter().any(has_chosen_count),
            Effect::DrawCards {
                count: EffectAmount::ChosenTypeCreatureCount { .. },
                ..
            } => true,
            Effect::LoseLife {
                amount: EffectAmount::ChosenTypeCreatureCount { .. },
                ..
            } => true,
            _ => false,
        }
    }

    let has_it = pact.abilities.iter().any(|ab| {
        if let AbilityDefinition::Spell { effect, .. } = ab {
            has_chosen_count(effect)
        } else {
            false
        }
    });

    assert!(
        has_it,
        "Pact of the Serpent should use ChosenTypeCreatureCount for draw and life loss"
    );
}

/// CR 205.3m — ChosenTypeCreatureCount via GainLife effect: gain life = number of Elves.
#[test]
fn test_chosen_type_permanent_count_via_effect() {
    use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};

    let p1 = p(1);

    let source = ObjectSpec::artifact(p1, "Count Source").in_zone(ZoneId::Battlefield);
    let elf1 = ObjectSpec::creature(p1, "Count Elf 1", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let elf2 = ObjectSpec::creature(p1, "Count Elf 2", 1, 1)
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Battlefield);
    let human = ObjectSpec::creature(p1, "Count Human", 1, 1)
        .with_subtypes(vec![SubType("Human".to_string())])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .object(elf1)
        .object(elf2)
        .object(human)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Count Source");
    let mut state2 = state;
    if let Some(obj) = state2.objects.get_mut(&source_id) {
        obj.chosen_creature_type = Some(SubType("Elf".to_string()));
    }

    let initial_life = state2.players.get(&p1).map(|ps| ps.life_total).unwrap_or(0);
    let mut ctx = make_ctx(&state2, source_id);

    // GainLife with ChosenTypeCreatureCount: should gain 2 life (2 Elves controlled).
    let _events = execute_effect(
        &mut state2,
        &Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::ChosenTypeCreatureCount {
                controller: PlayerTarget::Controller,
            },
        },
        &mut ctx,
    );

    let final_life = state2.players.get(&p1).map(|ps| ps.life_total).unwrap_or(0);
    assert_eq!(
        final_life - initial_life,
        2,
        "Should gain 2 life (2 Elves controlled) from ChosenTypeCreatureCount (CR 205.3m)"
    );
}

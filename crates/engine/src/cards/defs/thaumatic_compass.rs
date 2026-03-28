// Thaumatic Compass // Spires of Orazca — DFC Artifact // Land (Transform)
// Front: {2}, {T}: Search your library for a basic land card, reveal it, put it into your hand, then shuffle.
//         At the beginning of your end step, if you control seven or more lands, transform Thaumatic Compass.
// Back:  Spires of Orazca — Land
//         {T}: Add {C}.
//         {T}: Tap target creature an opponent controls.
//
// DSL gap: at-beginning-of-end-step conditional-transform trigger not expressible.
// Front search ability and back tap abilities are faithfully implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thaumatic-compass-spires-of-orazca"),
        name: "Thaumatic Compass".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{2}, {T}: Search your library for a basic land card, reveal it, put it into your hand, then shuffle.\nAt the beginning of your end step, if you control seven or more lands, transform Thaumatic Compass.".to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
                    reveal: true,
                    destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            // TODO: at beginning of your end step, if you control seven or more lands, transform
            //   (needs TriggerCondition::AtBeginningOfYourEndStep + Condition::YouControlNOrMoreLands(7) + TransformSelf effect)
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Spires of Orazca".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "{T}: Add {C}.\n{T}: Tap target creature an opponent controls.".to_string(),
            power: None,
            toughness: None,
            abilities: vec![
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 1),
                    },
                    timing_restriction: None,
                    targets: vec![],
                    activation_condition: None,
                    activation_zone: None,
                },
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::TapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    timing_restriction: None,
                    targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    })],
                    activation_condition: None,
                    activation_zone: None,
                },
            ],
            color_indicator: None,
        }),
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}

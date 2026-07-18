// Thaumatic Compass // Spires of Orazca — DFC Artifact // Land (Transform)
// Front: {2}, {T}: Search your library for a basic land card, reveal it, put it into your hand, then shuffle.
//         At the beginning of your end step, if you control seven or more lands, transform Thaumatic Compass.
// Back:  Spires of Orazca — Land
//         {T}: Add {C}.
//         {T}: Tap target creature an opponent controls.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thaumatic-compass-spires-of-orazca"),
        name: "Thaumatic Compass".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{2}, {T}: Search your library for a basic land card, reveal it, put it into \
                      your hand, then shuffle.\nAt the beginning of your end step, if you control \
                      seven or more lands, transform Thaumatic Compass."
            .to_string(),
        power: None,
        toughness: None,
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 2,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
                    reveal: true,
                    destination: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            // CR 701.27a/f: "At the beginning of your end step, if you control seven or
            // more lands, transform Thaumatic Compass." (PB-EF5)
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::TransformSelf,
                intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 7,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                }),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Spires of Orazca".to_string(),
            mana_cost: None,
            types: types(&[CardType::Land]),
            oracle_text: "{T}: Add {C}.\n{T}: Tap target creature an opponent controls."
                .to_string(),
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
                    once_per_turn: false,
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
                    once_per_turn: false,
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
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}

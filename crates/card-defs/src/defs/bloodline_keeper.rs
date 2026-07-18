// Bloodline Keeper // Lord of Lineage — {2}{B}{B} DFC Creature — Vampire (Transform)
// Front: Flying. {T}: Create a 2/2 black Vampire creature token with flying.
//        {B}: Transform this creature. Activate only if you control five or more Vampires.
// Back:  Lord of Lineage — Flying. Other Vampire creatures you control get +2/+2.
//        {T}: Create a 2/2 black Vampire creature token with flying.
use crate::cards::helpers::*;

fn vampire_token() -> TokenSpec {
    TokenSpec {
        name: "Vampire".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Vampire".to_string())].into_iter().collect(),
        colors: [Color::Black].into_iter().collect(),
        power: 2,
        toughness: 2,
        count: EffectAmount::Fixed(1),
        supertypes: imbl::OrdSet::new(),
        keywords: [KeywordAbility::Flying].into_iter().collect(),
        tapped: false,
        enters_attacking: false,
        mana_color: None,
        mana_abilities: vec![],
        activated_abilities: vec![],
        ..Default::default()
    }
}

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodline-keeper-lord-of-lineage"),
        name: "Bloodline Keeper".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 2,
            ..Default::default()
        }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Flying\n{T}: Create a 2/2 black Vampire creature token with flying.\n{B}: \
                      Transform this creature. Activate only if you control five or more Vampires."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            // "{T}: Create a 2/2 black Vampire creature token with flying."
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::CreateToken {
                    spec: vampire_token(),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // CR 701.27a/602.5b: "{B}: Transform this creature. Activate only if you
            // control five or more Vampires." (PB-EF5)
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    black: 1,
                    ..Default::default()
                }),
                effect: Effect::TransformSelf,
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 5,
                    filter: TargetFilter {
                        has_subtype: Some(SubType("Vampire".to_string())),
                        ..Default::default()
                    },
                }),
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Lord of Lineage".to_string(),
            mana_cost: None,
            types: creature_types(&["Vampire"]),
            oracle_text: "Flying\nOther Vampire creatures you control get +2/+2.\n{T}: Create a \
                          2/2 black Vampire creature token with flying."
                .to_string(),
            power: Some(5),
            toughness: Some(5),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Flying),
                // CR 613.1c: "Other Vampire creatures you control get +2/+2."
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(2),
                        filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType(
                            "Vampire".to_string(),
                        )),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        condition: None,
                    },
                },
                AbilityDefinition::Activated {
                    cost: Cost::Tap,
                    effect: Effect::CreateToken {
                        spec: vampire_token(),
                    },
                    timing_restriction: None,
                    targets: vec![],
                    activation_condition: None,
                    activation_zone: None,
                    once_per_turn: false,
                    modes: None,
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

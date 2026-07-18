// Docent of Perfection // Final Iteration — {3}{U}{U} DFC Creature — Insect Horror // Eldrazi Insect (Transform)
// Front: Flying. Whenever you cast an instant or sorcery spell, create a 1/1 blue Human
//        Wizard creature token. Then if you control three or more Wizards, transform this
//        creature.
// Back:  Flying. Wizards you control get +2/+1 and have flying.
//        Whenever you cast an instant or sorcery spell, create a 1/1 blue Human Wizard
//        creature token.
use crate::cards::helpers::*;

fn wizard_token() -> TokenSpec {
    TokenSpec {
        name: "Human Wizard".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Human".to_string()), SubType("Wizard".to_string())]
            .into_iter()
            .collect(),
        colors: [Color::Blue].into_iter().collect(),
        power: 1,
        toughness: 1,
        count: EffectAmount::Fixed(1),
        supertypes: imbl::OrdSet::new(),
        keywords: imbl::OrdSet::new(),
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
        card_id: cid("docent-of-perfection-final-iteration"),
        name: "Docent of Perfection".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            blue: 2,
            ..Default::default()
        }),
        types: creature_types(&["Insect", "Horror"]),
        oracle_text: "Flying\nWhenever you cast an instant or sorcery spell, create a 1/1 blue \
                      Human Wizard creature token. Then if you control three or more Wizards, \
                      transform this creature."
            .to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            // CR 701.27a/f: "Whenever you cast an instant or sorcery spell, create a 1/1
            // blue Human Wizard creature token. Then if you control three or more
            // Wizards, transform this creature." (PB-EF5)
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery]),
                    noncreature_only: false,
                    chosen_subtype_filter: false,
                    spell_subtype_filter: None,
                },
                effect: Effect::Sequence(vec![
                    Effect::CreateToken {
                        spec: wizard_token(),
                    },
                    // "Then if you control three or more Wizards" -- checked AFTER the
                    // token above has been created, so the new token counts.
                    Effect::Conditional {
                        condition: Condition::YouControlNOrMoreWithFilter {
                            count: 3,
                            filter: TargetFilter {
                                has_subtype: Some(SubType("Wizard".to_string())),
                                ..Default::default()
                            },
                        },
                        if_true: Box::new(Effect::TransformSelf),
                        if_false: Box::new(Effect::Nothing),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Final Iteration".to_string(),
            mana_cost: None,
            types: creature_types(&["Eldrazi", "Insect"]),
            oracle_text: "Flying\nWizards you control get +2/+1 and have flying.\nWhenever you \
                          cast an instant or sorcery spell, create a 1/1 blue Human Wizard \
                          creature token."
                .to_string(),
            power: Some(6),
            toughness: Some(5),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Flying),
                // CR 613.1c/613.1f: "Wizards you control get +2/+1 and have flying."
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPower(2),
                        filter: EffectFilter::CreaturesYouControlWithSubtype(SubType(
                            "Wizard".to_string(),
                        )),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        condition: None,
                    },
                },
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyToughness(1),
                        filter: EffectFilter::CreaturesYouControlWithSubtype(SubType(
                            "Wizard".to_string(),
                        )),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        condition: None,
                    },
                },
                AbilityDefinition::Static {
                    continuous_effect: ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                        filter: EffectFilter::CreaturesYouControlWithSubtype(SubType(
                            "Wizard".to_string(),
                        )),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        condition: None,
                    },
                },
                // CR 701.27a: back face does not itself transform further -- no "then
                // transform" clause here, matching the printed oracle text.
                AbilityDefinition::Triggered {
                    once_per_turn: false,
                    trigger_condition: TriggerCondition::WheneverYouCastSpell {
                        during_opponent_turn: false,
                        spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery]),
                        noncreature_only: false,
                        chosen_subtype_filter: false,
                        spell_subtype_filter: None,
                    },
                    effect: Effect::CreateToken {
                        spec: wizard_token(),
                    },
                    intervening_if: None,
                    targets: vec![],
                    modes: None,
                    trigger_zone: None,
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

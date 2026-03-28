// Elvish Warmaster — {1}{G}, Creature — Elf Warrior 2/2
// Whenever one or more other Elves you control enter, create a 1/1 green Elf Warrior
// creature token. This ability triggers only once each turn.
// {5}{G}{G}: Elves you control get +2/+2 and gain deathtouch until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-warmaster"),
        name: "Elvish Warmaster".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "Whenever one or more other Elves you control enter, create a 1/1 green Elf Warrior creature token. This ability triggers only once each turn.\n{5}{G}{G}: Elves you control get +2/+2 and gain deathtouch until end of turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Once each turn" + "other Elves entering" — both not in DSL.
            // Using generic creature ETB (overbroad but token is correct type).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Elf Warrior".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Elf".to_string()), SubType("Warrior".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 613.4c / CR 613.1f: "{5}{G}{G}: Elves you control get +2/+2 and gain
            // deathtouch until end of turn."
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 5, green: 2, ..Default::default() }),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(2),
                            filter: EffectFilter::CreaturesYouControlWithSubtype(
                                SubType("Elf".to_string()),
                            ),
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                            filter: EffectFilter::CreaturesYouControlWithSubtype(
                                SubType("Elf".to_string()),
                            ),
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}

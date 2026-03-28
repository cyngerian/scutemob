// Creeping Tar Pit
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("creeping-tar-pit"),
        name: "Creeping Tar Pit".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {U} or {B}.\n{1}{U}{B}: Until end of turn, this land becomes a 3/2 blue and black Elemental creature. It's still a land. It can't be blocked this turn.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {U} or {B}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Choose {
                    prompt: "Add {U} or {B}?".to_string(),
                    choices: vec![
                        Effect::AddMana {
                            player: PlayerTarget::Controller,
                            mana: mana_pool(0, 1, 0, 0, 0, 0),
                        },
                        Effect::AddMana {
                            player: PlayerTarget::Controller,
                            mana: mana_pool(0, 0, 1, 0, 0, 0),
                        },
                    ],
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // CR 613.1d/613.4b: {1}{U}{B}: Until end of turn, this land becomes a 3/2 blue
            // and black Elemental creature that can't be blocked. It's still a land.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, blue: 1, black: 1, ..Default::default() }),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddCardTypes(
                                [CardType::Creature].into_iter().collect(),
                            ),
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddSubtypes(
                                [SubType("Elemental".to_string())].into_iter().collect(),
                            ),
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtSet,
                            modification: LayerModification::SetPowerToughness {
                                power: 3,
                                toughness: 2,
                            },
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::ColorChange,
                            modification: LayerModification::SetColors(
                                [Color::Blue, Color::Black].into_iter().collect(),
                            ),
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeywords(
                                [KeywordAbility::CantBeBlocked].into_iter().collect(),
                            ),
                            filter: EffectFilter::Source,
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

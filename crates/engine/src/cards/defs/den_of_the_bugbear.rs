// Den of the Bugbear
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("den-of-the-bugbear"),
        name: "Den of the Bugbear".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "If you control two or more other lands, this land enters tapped.\n{T}: Add {R}.\n{3}{R}: Until end of turn, this land becomes a 3/2 red Goblin creature with \"Whenever this creature attacks, create a 1/1 red Goblin creature token that's tapped and attacking.\" It's still a land.".to_string(),
        abilities: vec![
            // CR 614.1c: "If you control two or more other lands, this land enters tapped."
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::Not(Box::new(
                    Condition::ControlAtLeastNOtherLands(2),
                ))),
            },
            // {T}: Add {R}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 1, 0, 0) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // CR 613.1d/613.4b: {3}{R}: Until end of turn, this land becomes a 3/2 red Goblin creature.
            // Note: "Whenever this creature attacks, create a 1/1 red Goblin token" omitted —
            // granting triggered abilities via layers is not in DSL (AddTriggeredAbility missing).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 3, red: 1, ..Default::default() }),
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
                                [SubType("Goblin".to_string())].into_iter().collect(),
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
                                [Color::Red].into_iter().collect(),
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

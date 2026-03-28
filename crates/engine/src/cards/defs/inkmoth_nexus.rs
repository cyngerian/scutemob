// Inkmoth Nexus — Land
// {T}: Add {C}.
// {1}: Becomes a 1/1 Phyrexian Blinkmoth artifact creature with flying and infect until EOT.
//      Still a land.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("inkmoth-nexus"),
        name: "Inkmoth Nexus".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}: This land becomes a 1/1 Phyrexian Blinkmoth artifact creature with flying and infect until end of turn. It's still a land. (It deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)".to_string(),
        abilities: vec![
            // {T}: Add {C}
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
            // {1}: Animate — becomes 1/1 Phyrexian Blinkmoth artifact creature with flying + infect
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddCardTypes(
                                [CardType::Artifact, CardType::Creature].into_iter().collect(),
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
                                [SubType("Phyrexian".to_string()), SubType("Blinkmoth".to_string())]
                                    .into_iter().collect(),
                            ),
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtSet,
                            modification: LayerModification::SetPowerToughness { power: 1, toughness: 1 },
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeywords(
                                [KeywordAbility::Flying, KeywordAbility::Infect].into_iter().collect(),
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

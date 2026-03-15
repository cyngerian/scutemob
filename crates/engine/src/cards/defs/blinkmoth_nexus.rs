// Blinkmoth Nexus — Land
// {T}: Add {C}.
// {1}: Becomes a 1/1 Blinkmoth artifact creature with flying until EOT. Still a land.
// {1}, {T}: Target Blinkmoth creature gets +1/+1 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blinkmoth-nexus"),
        name: "Blinkmoth Nexus".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}: This land becomes a 1/1 Blinkmoth artifact creature with flying until end of turn. It's still a land.\n{1}, {T}: Target Blinkmoth creature gets +1/+1 until end of turn.".to_string(),
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
            },
            // {1}: Animate — becomes 1/1 Blinkmoth artifact creature with flying until EOT
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
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::TypeChange,
                            modification: LayerModification::AddSubtypes(
                                [SubType("Blinkmoth".to_string())].into_iter().collect(),
                            ),
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtSet,
                            modification: LayerModification::SetPowerToughness { power: 1, toughness: 1 },
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                        }),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
            },
            // {1}, {T}: Target Blinkmoth creature gets +1/+1 until end of turn.
            // TODO: Target filter should restrict to Blinkmoth creatures only.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(1),
                        filter: EffectFilter::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                    }),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
            },
        ],
        ..Default::default()
    }
}

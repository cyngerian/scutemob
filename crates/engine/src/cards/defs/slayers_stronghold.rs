// Slayers' Stronghold — Land, {T}: Add {C}. {R}{W},{T}: Target creature +2/+0 vigilance haste (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("slayers-stronghold"),
        name: "Slayers' Stronghold".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{R}{W}, {T}: Target creature gets +2/+0 and gains vigilance and haste until end of turn.".to_string(),
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
            // {R}{W}, {T}: Target creature gets +2/+0 and gains vigilance and haste until EOT.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { red: 1, white: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: crate::state::EffectLayer::PtModify,
                            modification: crate::state::LayerModification::ModifyPower(2),
                            filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                            duration: crate::state::EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: crate::state::EffectLayer::Ability,
                            modification: crate::state::LayerModification::AddKeywords(
                                [KeywordAbility::Vigilance, KeywordAbility::Haste]
                                    .into_iter()
                                    .collect(),
                            ),
                            filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                            duration: crate::state::EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

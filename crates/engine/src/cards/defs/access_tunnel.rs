// Access Tunnel — Land, {T}: Add {C}; {3},{T}: target creature (power 3 or less) can't be blocked
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("access-tunnel"),
        name: "Access Tunnel".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{3}, {T}: Target creature with power 3 or less can't be blocked this turn.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // {3}, {T}: Target creature with power 3 or less can't be blocked this turn.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: crate::state::EffectLayer::Ability,
                        modification: crate::state::LayerModification::AddKeyword(
                            KeywordAbility::CantBeBlocked,
                        ),
                        filter: crate::state::EffectFilter::DeclaredTarget { index: 0 },
                        duration: crate::state::EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    max_power: Some(3),
                    ..Default::default()
                })],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

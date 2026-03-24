// Castle Embereth — This land enters tapped unless you control a Mountain. {T}: Add {R}.
// {1}{R}{R}, {T}: Creatures you control get +1/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("castle-embereth"),
        name: "Castle Embereth".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Mountain.\n{T}: Add {R}.\n{1}{R}{R}, {T}: Creatures you control get +1/+0 until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlLandWithSubtypes(vec![SubType("Mountain".to_string())])),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // {1}{R}{R}, {T}: Creatures you control get +1/+0 until end of turn.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, red: 2, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: crate::state::EffectLayer::PtModify,
                        modification: crate::state::LayerModification::ModifyPower(1),
                        filter: crate::state::EffectFilter::CreaturesYouControl,
                        duration: crate::state::EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

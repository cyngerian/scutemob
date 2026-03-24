// Goldnight Commander — {3}{W}, Creature — Human Cleric Soldier 2/2
// Whenever another creature you control enters, creatures you control get +1/+1 until
// end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goldnight-commander"),
        name: "Goldnight Commander".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Cleric", "Soldier"]),
        oracle_text: "Whenever another creature you control enters, creatures you control get +1/+1 until end of turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(1),
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

// Mirror Entity — {2}{W}, Creature — Shapeshifter 1/1
// Changeling (This card is every creature type.)
// {X}: Until end of turn, creatures you control have base power and toughness X/X and
// gain all creature types.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mirror-entity"),
        name: "Mirror Entity".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Shapeshifter"]),
        oracle_text: "Changeling (This card is every creature type.)\n{X}: Until end of turn, creatures you control have base power and toughness X/X and gain all creature types.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Changeling),
            // CR 107.3k / 613.4b / 205.3m: {X}: Until EOT, creatures you control have base
            // P/T X/X (Layer 7b, PB-AC3 SetBothDynamic — locked in at resolution) and gain
            // all creature types (Layer 6, AddAllCreatureTypes).
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { x_count: 1, ..Default::default() }),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtSet,
                            modification: LayerModification::SetBothDynamic {
                                amount: Box::new(EffectAmount::XValue),
                            },
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddAllCreatureTypes,
                            filter: EffectFilter::CreaturesYouControl,
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

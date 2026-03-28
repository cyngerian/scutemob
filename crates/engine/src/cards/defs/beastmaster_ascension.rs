// Beastmaster Ascension — {2}{G}, Enchantment
// Whenever a creature you control attacks, you may put a quest counter on this enchantment.
// As long as this enchantment has seven or more quest counters on it, creatures you control
// get +5/+5.
//
// CR 508.1m / PB-23: "Whenever a creature you control attacks" — WheneverCreatureYouControlAttacks.
// CR 604.2 / CR 613.1c (Layer 7c): "As long as this has 7+ quest counters, creatures you
// control get +5/+5." Implemented as conditional static with Condition::SourceHasCounters.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("beastmaster-ascension"),
        name: "Beastmaster Ascension".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control attacks, you may put a quest counter on this enchantment.\nAs long as this enchantment has seven or more quest counters on it, creatures you control get +5/+5.".to_string(),
        abilities: vec![
            // CR 508.1m: "Whenever a creature you control attacks, put a quest counter on this."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Quest,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 604.2 / CR 613.1c (Layer 7c): "As long as this enchantment has seven or more
            // quest counters on it, creatures you control get +5/+5."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(5),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::SourceHasCounters {
                        counter: CounterType::Quest,
                        min: 7,
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

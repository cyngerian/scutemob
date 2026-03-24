// Garruk's Uprising — {2}{G}, Enchantment
// When this enchantment enters, if you control a creature with power 4 or greater,
// draw a card.
// Creatures you control have trample.
// Whenever a creature you control with power 4 or greater enters, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("garruks-uprising"),
        name: "Garruk's Uprising".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters, if you control a creature with power 4 or greater, draw a card.\nCreatures you control have trample.\nWhenever a creature you control with power 4 or greater enters, draw a card.".to_string(),
        abilities: vec![
            // TODO: "If you control creature with power 4+" intervening-if on ETB.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Trample),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        min_power: Some(4),
                        ..Default::default()
                    }),
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

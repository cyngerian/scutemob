// Temur Ascendancy — {G}{U}{R}, Enchantment
// Creatures you control have haste.
// Whenever a creature you control with power 4 or greater enters, you may draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("temur-ascendancy"),
        name: "Temur Ascendancy".to_string(),
        mana_cost: Some(ManaCost { green: 1, blue: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control have haste.\nWhenever a creature you control with power 4 or greater enters, you may draw a card.".to_string(),
        abilities: vec![
            // Creatures you control have haste.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Whenever a creature you control with power 4+ enters, may draw.
            // TODO: "may draw" — optional draw, implementing as mandatory.
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

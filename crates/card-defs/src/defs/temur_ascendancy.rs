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
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        min_power: Some(4),
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::known_wrong("known_wrong — 'you may draw a card' is implemented as a MANDATORY draw; no optional-trigger primitive exists (Effect::Choose always takes the first option, effects/mod.rs:3190)."),
        ..Default::default()
    }
}

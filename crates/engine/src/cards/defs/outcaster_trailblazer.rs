// Outcaster Trailblazer — {2}{G}, Creature — Human Druid 4/2
// When this creature enters, add one mana of any color.
// Whenever another creature you control with power 4 or greater enters, draw a card.
// Plot {2}{G}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("outcaster-trailblazer"),
        name: "Outcaster Trailblazer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Human", "Druid"]),
        oracle_text: "When this creature enters, add one mana of any color.\nWhenever another creature you control with power 4 or greater enters, draw a card.\nPlot {2}{G}".to_string(),
        power: Some(4),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                intervening_if: None,
                targets: vec![],
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
            AbilityDefinition::Keyword(KeywordAbility::Plot),
        ],
        ..Default::default()
    }
}

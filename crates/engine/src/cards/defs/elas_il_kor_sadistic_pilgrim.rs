// Elas il-Kor, Sadistic Pilgrim
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elas-il-kor-sadistic-pilgrim"),
        name: "Elas il-Kor, Sadistic Pilgrim".to_string(),
        mana_cost: Some(ManaCost { white: 1, black: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Phyrexian", "Kor", "Cleric"]),
        oracle_text: "Deathtouch
Whenever another creature you control enters, you gain 1 life.
Whenever another creature you control dies, each opponent loses 1 life.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            // "Whenever another creature you control enters" — ETB trigger.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: DSL gap — "Whenever another creature you control dies" trigger.
            // WheneverCreatureDies has no controller filter.
        ],
        ..Default::default()
    }
}

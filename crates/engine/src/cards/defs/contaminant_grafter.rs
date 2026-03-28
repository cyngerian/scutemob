// Contaminant Grafter — {4}{G}, Creature — Phyrexian Druid 5/5
// Trample, toxic 1
// Whenever one or more creatures you control deal combat damage to one or more players,
// proliferate.
// Corrupted — At the beginning of your end step, if an opponent has three or more poison
// counters, draw a card, then you may put a land card from your hand onto the battlefield.
//
// TODO: "May put a land from hand onto battlefield" not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("contaminant-grafter"),
        name: "Contaminant Grafter".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Druid"]),
        oracle_text: "Trample, toxic 1\nWhenever one or more creatures you control deal combat damage to one or more players, proliferate.\nCorrupted \u{2014} At the beginning of your end step, if an opponent has three or more poison counters, draw a card, then you may put a land card from your hand onto the battlefield.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
            // CR 510.3a / CR 603.2c: "Whenever one or more creatures you control deal combat
            // damage to one or more players, proliferate." — batch trigger.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter: None },
                effect: Effect::Proliferate,
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Corrupted end step: draw if opponent has 3+ poison
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: Some(Condition::OpponentHasPoisonCounters(3)),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "May put a land from hand onto battlefield" not in DSL.
        ],
        ..Default::default()
    }
}

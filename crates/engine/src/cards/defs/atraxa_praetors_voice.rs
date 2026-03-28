// Atraxa, Praetors' Voice — {G}{W}{U}{B}, Legendary Creature — Phyrexian Angel Horror 4/4
// Flying, vigilance, deathtouch, lifelink
// At the beginning of your end step, proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("atraxa-praetors-voice"),
        name: "Atraxa, Praetors' Voice".to_string(),
        mana_cost: Some(ManaCost { green: 1, white: 1, blue: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Angel", "Horror"],
        ),
        oracle_text: "Flying, vigilance, deathtouch, lifelink\nAt the beginning of your end step, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                effect: Effect::Proliferate,
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

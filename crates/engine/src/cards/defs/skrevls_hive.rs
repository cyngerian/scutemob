// Skrelv's Hive — {1}{W}, Enchantment
// Upkeep trigger: lose 1 life + create 1/1 colorless Phyrexian Mite artifact creature token
//   with toxic 1 and "can't block".
// Corrupted static: creatures you control with toxic have lifelink if opponent has 3+ poison.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skrevls-hive"),
        name: "Skrelv's Hive".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your upkeep, you lose 1 life and create a 1/1 colorless Phyrexian Mite artifact creature token with toxic 1 and \"This token can't block.\"\nCorrupted \u{2014} As long as an opponent has three or more poison counters, creatures you control with toxic have lifelink.".to_string(),
        abilities: vec![
            // Upkeep trigger: lose 1 life and create a 1/1 colorless Phyrexian Mite artifact
            // creature token with toxic 1 and "can't block".
            // Toxic 1 keyword and CantBlock restriction both applied to tokens.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::Sequence(vec![
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Phyrexian Mite".to_string(),
                            card_types: [CardType::Artifact, CardType::Creature]
                                .into_iter()
                                .collect(),
                            subtypes: [SubType("Phyrexian".to_string()), SubType("Mite".to_string())]
                                .into_iter()
                                .collect(),
                            colors: im::OrdSet::new(),
                            supertypes: im::OrdSet::new(),
                            power: 1,
                            toughness: 1,
                            count: 1,
                            // CR 509.1b: "This token can't block."
                            keywords: [KeywordAbility::Toxic(1), KeywordAbility::CantBlock]
                                .into_iter()
                                .collect(),
                            tapped: false,
                            enters_attacking: false,
                            mana_color: None,
                            mana_abilities: vec![],
                            activated_abilities: vec![],
                            ..Default::default()
                        },
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: Corrupted — "As long as an opponent has three or more poison counters,
            // creatures you control with toxic have lifelink."
            // DSL gap: Static grant with filter "creatures you control with toxic" not
            // expressible — EffectFilter::CreaturesYouControl lacks a keyword filter.
            // Additionally, Corrupted keyword marker is not in KeywordAbility enum.
        ],
        ..Default::default()
    }
}

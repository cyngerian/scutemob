// Bitterblossom — {1}{B}, Kindred Enchantment — Faerie
// At the beginning of your upkeep, you lose 1 life and create a 1/1 black Faerie Rogue
// creature token with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bitterblossom"),
        name: "Bitterblossom".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types_sub(&[CardType::Kindred, CardType::Enchantment], &["Faerie"]),
        oracle_text: "At the beginning of your upkeep, you lose 1 life and create a 1/1 black Faerie Rogue creature token with flying.".to_string(),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::Sequence(vec![
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Faerie Rogue".to_string(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [
                                SubType("Faerie".to_string()),
                                SubType("Rogue".to_string()),
                            ]
                            .into_iter()
                            .collect(),
                            colors: [Color::Black].into_iter().collect(),
                            power: 1,
                            toughness: 1,
                            count: 1,
                            supertypes: im::OrdSet::new(),
                            keywords: [KeywordAbility::Flying].into_iter().collect(),
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
            },
        ],
        ..Default::default()
    }
}

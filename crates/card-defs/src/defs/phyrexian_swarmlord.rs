// Phyrexian Swarmlord — {4}{G}{G}, Creature — Phyrexian Insect Horror 4/4
// Infect (This creature deals damage to creatures in the form of -1/-1 counters and
// to players in the form of poison counters.)
// At the beginning of your upkeep, create a 1/1 green Phyrexian Insect creature token
// with infect for each poison counter your opponents have.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phyrexian-swarmlord"),
        name: "Phyrexian Swarmlord".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Insect", "Horror"]),
        oracle_text: "Infect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)\nAt the beginning of your upkeep, create a 1/1 green Phyrexian Insect creature token with infect for each poison counter your opponents have.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Infect),
            // CR 111.1 / CR 608.2h: At beginning of upkeep, create X 1/1 green Phyrexian Insect
            // tokens with infect where X = total poison counters on all opponents.
            // EffectAmount::PlayerCounterCount with EachOpponent sums across all opponents
            // (PB-CC-A + PB-TS unblock: PlayerCounterCount exists, TokenSpec.count now EffectAmount).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Phyrexian Insect".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [
                            SubType("Phyrexian".to_string()),
                            SubType("Insect".to_string()),
                        ]
                        .into_iter()
                        .collect(),
                        colors: [Color::Green].into_iter().collect(),
                        supertypes: imbl::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        keywords: [KeywordAbility::Infect].into_iter().collect(),
                        count: EffectAmount::PlayerCounterCount {
                            player: PlayerTarget::EachOpponent,
                            counter: CounterType::Poison,
                        },
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

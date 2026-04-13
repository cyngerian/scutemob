// Utvara Hellkite — {6}{R}{R}, Creature — Dragon 6/6
// Flying
// Whenever a Dragon you control attacks, create a 6/6 red Dragon creature token
// with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("utvara-hellkite"),
        name: "Utvara Hellkite".to_string(),
        mana_cost: Some(ManaCost { generic: 6, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever a Dragon you control attacks, create a 6/6 red Dragon creature token with flying.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 508.1m: "Whenever a Dragon you control attacks, create a 6/6 red Dragon token."
            // PB-N: Dragon subtype filter now available via triggering_creature_filter.
            // Fixed: was filter: None (over-triggered on non-Dragon attackers). PB-N closes this gap.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Dragon".to_string(),
                        power: 6,
                        toughness: 6,
                        colors: [Color::Red].into_iter().collect(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Dragon".to_string())].into_iter().collect(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
                        count: 1,
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

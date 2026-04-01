// Imperial Recruiter — {2}{R}, Creature — Human Advisor 1/1
// When this creature enters, search your library for a creature card with power
// 2 or less, reveal it, put it into your hand, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("imperial-recruiter"),
        name: "Imperial Recruiter".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Human", "Advisor"]),
        oracle_text: "When this creature enters, search your library for a creature card with power 2 or less, reveal it, put it into your hand, then shuffle.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::SearchLibrary {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        max_power: Some(2),
                        ..Default::default()
                    },
                    destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    reveal: true,
                    player: PlayerTarget::Controller,
                    also_search_graveyard: false,
                    shuffle_before_placing: false,
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

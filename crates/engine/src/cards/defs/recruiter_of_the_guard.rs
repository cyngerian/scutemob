// Recruiter of the Guard — {2}{W}, Creature — Human Soldier 1/1
// When this creature enters, you may search your library for a creature card
// with toughness 2 or less, reveal it, put it into your hand, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("recruiter-of-the-guard"),
        name: "Recruiter of the Guard".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Soldier"]),
        oracle_text: "When this creature enters, you may search your library for a creature card with toughness 2 or less, reveal it, put it into your hand, then shuffle.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                // TODO: TargetFilter lacks max_toughness — searching for creature card
                // with toughness 2 or less. Using has_card_type creature filter only.
                // When max_toughness is added, set max_toughness: Some(2).
                effect: Effect::SearchLibrary {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
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

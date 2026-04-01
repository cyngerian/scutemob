// Woodland Bellower — {4}{G}{G}, Creature — Beast 6/5
// When this creature enters, you may search your library for a nonlegendary green
// creature card with mana value 3 or less, put it onto the battlefield, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("woodland-bellower"),
        name: "Woodland Bellower".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
        types: creature_types(&["Beast"]),
        oracle_text: "When this creature enters, you may search your library for a nonlegendary green creature card with mana value 3 or less, put it onto the battlefield, then shuffle.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                // "nonlegendary green creature with mana value 3 or less"
                // TODO: TargetFilter lacks non_legendary — this searches any green creature
                // with MV 3 or less. When non_legendary is added, set it.
                effect: Effect::SearchLibrary {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        colors: Some([Color::Green].into_iter().collect()),
                        max_cmc: Some(3),
                        ..Default::default()
                    },
                    destination: ZoneTarget::Battlefield { tapped: false },
                    reveal: false,
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

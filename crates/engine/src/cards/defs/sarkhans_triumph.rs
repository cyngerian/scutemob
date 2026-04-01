// Sarkhan's Triumph — {2}{R}, Instant
// Search your library for a Dragon creature card, reveal it, put it into your hand,
// then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sarkhans-triumph"),
        name: "Sarkhan's Triumph".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Search your library for a Dragon creature card, reveal it, put it into your hand, then shuffle.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_subtype: Some(SubType("Dragon".to_string())),
                        ..Default::default()
                    },
                    reveal: true,
                    destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

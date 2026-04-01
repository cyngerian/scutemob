// Eladamri's Call — {G}{W}, Instant
// Search your library for a creature card, reveal it, put it into your hand,
// then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eladamris-call"),
        name: "Eladamri's Call".to_string(),
        mana_cost: Some(ManaCost { green: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Search your library for a creature card, reveal that card, put it into your hand, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
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
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

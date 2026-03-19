// Worldly Tutor — {G}, Instant: search library for a creature card, reveal it, shuffle, put on top
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("worldly-tutor"),
        name: "Worldly Tutor".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Search your library for a creature card, reveal it, then shuffle and put the card on top.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::SearchLibrary {
                player: PlayerTarget::Controller,
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                reveal: true,
                destination: ZoneTarget::Library {
                    owner: PlayerTarget::Controller,
                    position: LibraryPosition::Top,
                },
                // CR 701.23: "shuffle and put the card on top" — shuffle first, then place on top.
                // Ruling 2016-06-08: this is a single action; card ends on top after shuffle.
                shuffle_before_placing: true,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

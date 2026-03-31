// Skyshroud Claim — {3}{G}, Sorcery
// Search your library for up to two Forest cards, put them onto the battlefield,
// then shuffle.
//
// Note: searches for any Forest card (not just basic Forest) — any land with the
// Forest subtype qualifies (including shock lands, etc.). Enters untapped.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skyshroud-claim"),
        name: "Skyshroud Claim".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for up to two Forest cards, put them onto the battlefield, then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // First Forest card — any Forest subtype (basic: false),
                // enters untapped (oracle says "put them onto the battlefield", not tapped).
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        has_subtype: Some(SubType("Forest".to_string())),
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                // Second Forest card.
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        has_subtype: Some(SubType("Forest".to_string())),
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle { player: PlayerTarget::Controller },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

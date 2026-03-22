// 41. Cultivate — {2G}, Sorcery; search for 2 basic lands, one to battlefield
// tapped, one to hand, then shuffle.
use crate::cards::helpers::*;
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cultivate"),
        name: "Cultivate".to_string(),
        mana_cost: Some(ManaCost { green: 1, generic: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for up to two basic land cards, reveal those cards, and put one onto the battlefield tapped and the other into your hand. Then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
                    reveal: true,
                    destination: ZoneTarget::Battlefield { tapped: true },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
                    reveal: true,
                    destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
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

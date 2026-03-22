// 44. Explosive Vegetation — {3G}, Sorcery; search for up to two basic lands,
// put them onto battlefield tapped, then shuffle.
use crate::cards::helpers::*;
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("explosive-vegetation"),
        name: "Explosive Vegetation".to_string(),
        mana_cost: Some(ManaCost { green: 1, generic: 3, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for up to two basic land cards and put them onto the battlefield tapped. Then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: true },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: basic_land_filter(),
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: true },
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

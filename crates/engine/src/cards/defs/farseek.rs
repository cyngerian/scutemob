// Farseek — {1}{G}, Sorcery.
// "Search your library for a Plains, Island, Swamp, or Mountain card and put it
// onto the battlefield tapped. Then shuffle."
// CR 701.15: Library search spell — fetches a non-Forest basic dual land (or shock).
// Simplification: basic_land_filter() (finds any basic land). The real Farseek
// explicitly excludes Forests but can find shocklands with the right subtypes.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("farseek"),
        name: "Farseek".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for a Plains, Island, Swamp, or Mountain card and put it onto the battlefield tapped. Then shuffle.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

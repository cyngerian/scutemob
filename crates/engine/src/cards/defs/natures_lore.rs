// Nature's Lore — {1}{G}, Sorcery.
// "Search your library for a Forest card and put that card onto the battlefield.
// Then shuffle."
// CR 701.15: Fetches a Forest (basic or nonbasic with Forest subtype) untapped.
// Simplification: basic_land_filter() — finds any basic land. The real card
// fetches untapped (ZoneTarget::Battlefield { tapped: false }).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("natures-lore"),
        name: "Nature's Lore".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for a Forest card and put that card onto the battlefield. Then shuffle.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: false },
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

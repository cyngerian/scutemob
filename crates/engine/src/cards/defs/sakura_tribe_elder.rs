// Sakura-Tribe Elder — {1}{G}, Creature — Snake Shaman 1/1.
// "Sacrifice Sakura-Tribe Elder: Search your library for a basic land card, put
// it onto the battlefield tapped, then shuffle."
// CR 602.2: Activated ability — sacrifice self to ramp a basic land.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sakura-tribe-elder"),
        name: "Sakura-Tribe Elder".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Snake", "Shaman"]),
        oracle_text: "Sacrifice Sakura-Tribe Elder: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::SacrificeSelf,
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                timing_restriction: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
    }
}

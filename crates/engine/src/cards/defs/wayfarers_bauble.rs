// 7. Wayfarer's Bauble — {1}, Artifact, {2}, tap, sacrifice: search your library for
// a basic land, put it onto the battlefield tapped, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wayfarers-bauble"),
        name: "Wayfarer's Bauble".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{2}, {T}, Sacrifice Wayfarer's Bauble: Search your library for a basic land card and put it onto the battlefield tapped. Then shuffle.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![
                Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                Cost::Tap,
                Cost::SacrificeSelf,
            ]),
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
        }],
        ..Default::default()
    }
}

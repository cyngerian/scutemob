// Prismatic Vista — Land.
// "{T}, Pay 1 life, Sacrifice Prismatic Vista: Search your library for a basic land
// card, put it onto the battlefield, then shuffle."
// Enters untapped (unlike Evolving Wilds/Terramorphic Expanse which say "tapped").
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("prismatic-vista"),
        name: "Prismatic Vista".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Pay 1 life, Sacrifice Prismatic Vista: Search your library for a basic land card, put it onto the battlefield, then shuffle.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::PayLife(1),
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        // CR 701.23: Oracle says "put it onto the battlefield" (no tapped).
                        // Prismatic Vista enters untapped unlike Evolving Wilds/Terramorphic Expanse.
                        destination: ZoneTarget::Battlefield { tapped: false },
                        shuffle_before_placing: false,
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

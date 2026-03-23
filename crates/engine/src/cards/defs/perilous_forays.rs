// Perilous Forays — {3}{G}{G}, Enchantment
// "{1}, Sacrifice a creature: Search your library for a land card with a basic land type,
// put it onto the battlefield tapped, then shuffle."
// "Land card with a basic land type" (Plains, Island, Swamp, Mountain, Forest) is closely
// approximated by basic_land_filter() which requires basic supertype. Non-basic duals with
// basic land types (Tropical Island etc.) would be missed, but basic_land_filter is the
// closest available approximation in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("perilous-forays"),
        name: "Perilous Forays".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "{1}, Sacrifice a creature: Search your library for a land card with a basic land type, put it onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::Sequence(vec![
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
                timing_restriction: None,
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

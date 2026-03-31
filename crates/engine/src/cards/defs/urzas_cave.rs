// Urza's Cave — Land — Urza's Cave.
// "{T}: Add {C}."
// "{3}, {T}, Sacrifice this land: Search your library for a land card,
// put it onto the battlefield tapped, then shuffle."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("urzas-cave"),
        name: "Urza's Cave".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Urza's Cave"]),
        oracle_text: "{T}: Add {C}.\n{3}, {T}, Sacrifice this land: Search your library for a land card, put it onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // {3}, {T}, Sacrifice this land: Search your library for a land card,
            // put it onto the battlefield tapped, then shuffle.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

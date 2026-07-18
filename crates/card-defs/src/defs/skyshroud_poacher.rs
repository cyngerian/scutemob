// Skyshroud Poacher — {2}{G}{G}, Creature — Human Rebel 2/2
// {3}, {T}: Search your library for an Elf permanent card, put it onto the
// battlefield, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skyshroud-poacher"),
        name: "Skyshroud Poacher".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 2,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Rebel"]),
        oracle_text: "{3}, {T}: Search your library for an Elf permanent card, put it onto the \
                      battlefield, then shuffle."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![
                Cost::Mana(ManaCost {
                    generic: 3,
                    ..Default::default()
                }),
                Cost::Tap,
            ]),
            // "Elf permanent card" — subtype Elf AND one of the permanent card types
            // (CR 108.4b). has_card_types uses OR semantics, so a non-permanent Elf
            // card cannot be put onto the battlefield.
            effect: Effect::SearchLibrary {
                player: PlayerTarget::Controller,
                filter: TargetFilter {
                    has_subtype: Some(SubType("Elf".to_string())),
                    has_card_types: vec![
                        CardType::Artifact,
                        CardType::Battle,
                        CardType::Creature,
                        CardType::Enchantment,
                        CardType::Land,
                        CardType::Planeswalker,
                    ],
                    ..Default::default()
                },
                reveal: false,
                destination: ZoneTarget::Battlefield { tapped: false },
                shuffle_before_placing: false,
                also_search_graveyard: false,
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

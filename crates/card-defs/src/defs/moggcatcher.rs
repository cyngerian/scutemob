// Moggcatcher — {2}{R}{R}, Creature — Human Mercenary 2/2
// {3}, {T}: Search your library for a Goblin permanent card, put it onto the
// battlefield, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("moggcatcher"),
        name: "Moggcatcher".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Mercenary"]),
        oracle_text: "{3}, {T}: Search your library for a Goblin permanent card, put it onto the \
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
            // "Goblin permanent card" — subtype Goblin AND one of the permanent card
            // types (CR 108.4b: land/creature/artifact/enchantment/planeswalker/battle
            // are the permanent types). has_card_types uses OR semantics, so an
            // instant/sorcery with the Goblin subtype (there are none, but the filter
            // must be correct regardless) cannot be found onto the battlefield.
            effect: Effect::SearchLibrary {
                player: PlayerTarget::Controller,
                filter: TargetFilter {
                    has_subtype: Some(SubType("Goblin".to_string())),
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
            modes: None,
        }],
        ..Default::default()
    }
}

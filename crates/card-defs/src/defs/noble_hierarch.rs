// Noble Hierarch — {G}, Creature — Human Druid 0/1
// Exalted; {T}: Add {G}, {W}, or {U}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("noble-hierarch"),
        name: "Noble Hierarch".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Druid"]),
        oracle_text: "Exalted (Whenever a creature you control attacks alone, that creature gets \
                      +1/+1 until end of turn.)\n{T}: Add {G}, {W}, or {U}."
            .to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            // SR-33 (CR 305.6 / 605.1a): the printed mana ability, modelled
            // explicitly — one activated ability per colour, as `forest.rs`
            // does. The engine does not derive mana abilities from basic land
            // subtypes, so a def that leaves this to CR 305.6 produces nothing
            // at all. `TapForMana { ability_index }` picks the colour.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            AbilityDefinition::Keyword(KeywordAbility::Exalted),
        ],
        ..Default::default()
    }
}

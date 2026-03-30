// Elvish Harbinger — {2}{G}, Creature — Elf Druid 1/2
// When this creature enters, you may search your library for an Elf card,
//   reveal it, then shuffle and put that card on top.
// {T}: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-harbinger"),
        name: "Elvish Harbinger".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "When this creature enters, you may search your library for an Elf card, reveal it, then shuffle and put that card on top.\n{T}: Add one mana of any color.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // ETB: search library for an Elf card, reveal, shuffle, put on top
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    },
                    reveal: true,
                    destination: ZoneTarget::Library {
                        owner: PlayerTarget::Controller,
                        position: LibraryPosition::Top,
                    },
                    shuffle_before_placing: true,
                    also_search_graveyard: false,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // {T}: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
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

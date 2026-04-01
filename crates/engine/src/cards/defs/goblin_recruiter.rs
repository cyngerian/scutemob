// Goblin Recruiter — {1}{R}, Creature — Goblin 1/1
// When this enters, search your library for any number of Goblin cards, reveal those
// cards, then shuffle and put them on top in any order.
// TODO: "any number of" Goblin cards and "put them on top in any order" — SearchLibrary
// fetches a single card deterministically (first by ObjectId). The "any number" and
// "any order" choices are not expressible in the current DSL.
// Approximated as: search for one Goblin card and put it on top of library (shuffle first).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-recruiter"),
        name: "Goblin Recruiter".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "When this enters, search your library for any number of Goblin cards, reveal those cards, then shuffle and put them on top in any order.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                // TODO: "any number" of Goblins and "put on top in any order" not expressible.
                // Approximated as one Goblin to top of library (shuffle before placing).
                effect: Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_subtype: Some(SubType("Goblin".to_string())),
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
        ],
        ..Default::default()
    }
}

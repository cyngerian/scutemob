// Sakura-Tribe Scout — {G}, Creature — Snake Shaman Scout 1/1
// {T}: You may put a land card from your hand onto the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sakura-tribe-scout"),
        name: "Sakura-Tribe Scout".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Snake", "Shaman", "Scout"]),
        oracle_text: "{T}: You may put a land card from your hand onto the battlefield."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            // CR 608.2: "may" is embedded in PutLandFromHandOntoBattlefield's optionality.
            effect: Effect::PutLandFromHandOntoBattlefield { tapped: false },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

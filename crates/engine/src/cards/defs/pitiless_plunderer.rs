// Pitiless Plunderer — {3}{B}, Creature — Human Pirate 1/4.
// "Whenever another creature you control dies, create a Treasure token."
// TODO: DSL gap — WheneverCreatureDies triggers on ALL creature deaths, not
// just "another creature you control." Empty abilities per W5 policy until
// death trigger filtering is supported.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pitiless-plunderer"),
        name: "Pitiless Plunderer".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Pirate"]),
        oracle_text: "Whenever another creature you control dies, create a Treasure token.".to_string(),
        power: Some(1),
        toughness: Some(4),
        abilities: vec![],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}

// Pitiless Plunderer — {3}{B}, Creature — Human Pirate 1/4.
// "Whenever another creature you control dies, create a Treasure token."
// Uses death trigger + CreateToken with treasure_token_spec().
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
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverCreatureDies,
            // TODO: should be "another creature you control" — WheneverCreatureDies
            // triggers on any creature death, not just yours excluding self
            effect: Effect::CreateToken {
                spec: treasure_token_spec(1),
            },
            intervening_if: None,
        }],
        back_face: None,
    }
}

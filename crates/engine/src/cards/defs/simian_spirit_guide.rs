// Simian Spirit Guide — {2}{R}, Creature — Ape Spirit 2/2
// Exile this card from your hand: Add {R}.
//
// TODO: Cost::ExileFromHand does not exist. Same gap as Elvish Spirit Guide.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("simian-spirit-guide"),
        name: "Simian Spirit Guide".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Ape", "Spirit"]),
        oracle_text: "Exile this card from your hand: Add {R}.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: Cost::ExileFromHand not in DSL. Same gap as Elvish Spirit Guide.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost::default()),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: ManaPool { red: 1, ..Default::default() },
                },
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

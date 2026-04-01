// Elvish Spirit Guide — {2}{G}, Creature — Elf Spirit 2/2
// Exile this creature from your hand: Add {G}.
//
// TODO: Cost::ExileFromHand does not exist. The ability should be activatable
// from hand (activation_zone: Some(Zone::Hand)) with exile-self as cost.
// Approximated with activation_zone + Effect::Nothing.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-spirit-guide"),
        name: "Elvish Spirit Guide".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Spirit"]),
        oracle_text: "Exile this card from your hand: Add {G}.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: Cost::ExileFromHand not in DSL. Needs exile-self-from-hand cost
            // + AddMana({G}) effect. activation_zone: Hand is correct but cost is wrong.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost::default()),
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: ManaPool { green: 1, ..Default::default() },
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

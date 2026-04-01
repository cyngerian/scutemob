// Cunning Evasion — {1}{U}, Enchantment
// Whenever a creature you control becomes blocked, you may return it to its owner's hand.
//
// TODO: TriggerCondition::WhenBecomesBlocked does not exist in DSL.
//   "Whenever a creature you control becomes blocked" fires during combat when an attacker
//   has been assigned a blocker. No such TriggerCondition variant exists.
//   Omitted per W5 policy until WhenBecomesBlocked trigger is added.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cunning-evasion"),
        name: "Cunning Evasion".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control becomes blocked, you may return it to its owner's hand.".to_string(),
        abilities: vec![
            // TODO: TriggerCondition::WhenBecomesBlocked not in DSL.
            //   Need a trigger that fires when a creature you control is blocked during combat.
        ],
        ..Default::default()
    }
}

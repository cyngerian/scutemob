// Megrim — {2}{B}, Enchantment
// Whenever an opponent discards a card, this enchantment deals 2 damage to that player.
//
// TODO: Requires TriggerCondition::WheneverOpponentDiscards which does not exist in the DSL.
// The damage target "that player" also requires the trigger to pass the discarding player
// as a target reference, which is not supported. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("megrim"),
        name: "Megrim".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent discards a card, this enchantment deals 2 damage to that player.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

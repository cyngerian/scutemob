// Retreat to Kazandu — {2}{G}, Enchantment
// Landfall — Whenever a land you control enters, choose one —
// • Put a +1/+1 counter on target creature.
// • You gain 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("retreat-to-kazandu"),
        name: "Retreat to Kazandu".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Landfall — Whenever a land you control enters, choose one —\n• Put a +1/+1 counter on target creature.\n• You gain 2 life.".to_string(),
        abilities: vec![
            // TODO: DSL gap — modal triggered ability. TriggerCondition doesn't support
            // modes directly. Would need modes on triggered ability effect or a modal
            // triggered ability variant. Implementing as +1/+1 counter mode only
            // would be wrong game state (missing life gain option).
        ],
        ..Default::default()
    }
}

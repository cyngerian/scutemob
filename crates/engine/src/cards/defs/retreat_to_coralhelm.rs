// Retreat to Coralhelm — {2}{U}, Enchantment
// Landfall — Whenever a land you control enters, choose one —
// • You may tap or untap target creature.
// • Scry 1.
//
// TODO: Modal triggered ability — AbilityDefinition::Triggered has no modes field.
//   The landfall trigger with "choose one" requires modal support on triggered abilities.
//   Scry 1 mode is expressible but tap/untap choice + modality are not.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("retreat-to-coralhelm"),
        name: "Retreat to Coralhelm".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Landfall — Whenever a land you control enters, choose one —\n• You may tap or untap target creature.\n• Scry 1.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

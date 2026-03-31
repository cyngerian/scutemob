// Exquisite Blood — {4}{B}, Enchantment
// Whenever an opponent loses life, you gain that much life.
//
// TODO: TriggerCondition::WheneverOpponentLosesLife does not exist in the DSL.
// Also needs EffectAmount::TriggeringAmount for "that much life".
// W5: no partial implementation — abilities empty.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("exquisite-blood"),
        name: "Exquisite Blood".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent loses life, you gain that much life.".to_string(),
        abilities: vec![
            // TODO: TriggerCondition::WheneverOpponentLosesLife not in DSL.
            // Also needs EffectAmount::TriggeringAmount for "that much life".
            // W5: omitted to avoid wrong game state.
        ],
        ..Default::default()
    }
}

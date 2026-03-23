// Awakening Zone — {2}{G}, Enchantment
// At the beginning of your upkeep, you may create a 0/1 colorless Eldrazi Spawn creature
// token. It has "Sacrifice this token: Add {C}."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("awakening-zone"),
        name: "Awakening Zone".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Enchantment], &[]),
        oracle_text: "At the beginning of your upkeep, you may create a 0/1 colorless Eldrazi Spawn creature token. It has \"Sacrifice this token: Add {C}.\"".to_string(),
        abilities: vec![
            // TODO: Upkeep trigger with optional token creation — upkeep CardDef triggers
            // fire but "you may" optional not in DSL. Token also needs sac-for-mana ability.
        ],
        ..Default::default()
    }
}

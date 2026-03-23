// Curiosity — {U}, Enchantment — Aura
// Enchant creature
// Whenever enchanted creature deals damage to an opponent, you may draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("curiosity"),
        name: "Curiosity".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nWhenever enchanted creature deals damage to an opponent, you may draw a card.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // TODO: "Whenever enchanted creature deals damage to an opponent" —
            //   per-creature damage trigger not in DSL.
        ],
        ..Default::default()
    }
}

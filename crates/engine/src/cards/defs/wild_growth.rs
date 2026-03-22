// Wild Growth — {G}, Enchantment — Aura
// Enchant land
// Whenever enchanted land is tapped for mana, its controller adds an additional {G}.
//
// TODO: "Whenever enchanted land is tapped for mana, add {G}" — mana trigger on enchanted
//   land is not expressible. Needs a trigger that fires when the attached land produces mana.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wild-growth"),
        name: "Wild Growth".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant land\nWhenever enchanted land is tapped for mana, its controller adds an additional {G}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Land)),
        ],
        ..Default::default()
    }
}

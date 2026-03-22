// Elvish Guidance — {2}{G}, Enchantment — Aura
// Enchant land
// Whenever enchanted land is tapped for mana, its controller adds an additional {G}
// for each Elf on the battlefield.
//
// TODO: Mana trigger on enchanted land + count-based mana (Elves on battlefield).
//   Not expressible — same gap as Wild Growth plus count scaling.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-guidance"),
        name: "Elvish Guidance".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant land\nWhenever enchanted land is tapped for mana, its controller adds an additional {G} for each Elf on the battlefield.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Land)),
        ],
        ..Default::default()
    }
}

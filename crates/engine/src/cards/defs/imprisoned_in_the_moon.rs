// Imprisoned in the Moon — {2}{U}, Enchantment — Aura
// Enchant creature, land, or planeswalker
// Enchanted permanent is a colorless land with "{T}: Add {C}" and loses all other
//   card types and abilities.
//
// TODO: Complex layer interaction — enchanted permanent becomes colorless land,
//   loses all types/abilities, gains "{T}: Add {C}". Requires Layer 4 (type change),
//   Layer 5 (color change), Layer 6 (remove all abilities + grant mana ability).
//   Not expressible as a simple static effect. Implementing only the Aura/Enchant keyword.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("imprisoned-in-the-moon"),
        name: "Imprisoned in the Moon".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature, land, or planeswalker\nEnchanted permanent is a colorless land with \"{T}: Add {C}\" and loses all other card types and abilities.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Permanent)),
            // TODO: Layer 4/5/6 type+color+ability overwrite (complex layer interaction)
        ],
        ..Default::default()
    }
}

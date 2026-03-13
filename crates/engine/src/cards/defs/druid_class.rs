// Druid Class — {1}{G}, Enchantment — Class
// Class leveling mechanic; Level 1: Landfall gain 1 life; Level 2: extra land; Level 3: animate land
// TODO: Class level mechanic not supported in DSL (no level-up cost, no level tracking)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("druid-class"),
        name: "Druid Class".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Enchantment], &["Class"]),
        oracle_text: "(Gain the next level as a sorcery to add its ability.)\nLandfall — Whenever a land you control enters, you gain 1 life.\n{2}{G}: Level 2\nYou may play an additional land on each of your turns.\n{4}{G}: Level 3\nWhen this Class becomes level 3, target land you control becomes a creature with haste and \"This creature's power and toughness are each equal to the number of lands you control.\" It's still a land.".to_string(),
        abilities: vec![
            // TODO: Class level-up mechanic not in DSL; full oracle text requires level tracking,
            // extra land play, and animate-land effects which are not currently expressible.
        ],
        ..Default::default()
    }
}

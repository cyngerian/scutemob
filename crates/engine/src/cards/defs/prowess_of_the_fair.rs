// Prowess of the Fair — {1}{B}, Kindred Enchantment — Elf
// Whenever another nontoken Elf is put into your graveyard from the battlefield,
// you may create a 1/1 green Elf Warrior creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("prowess-of-the-fair"),
        name: "Prowess of the Fair".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        // Kindred Enchantment — Elf: represented as Enchantment with Elf subtype
        types: types_sub(&[CardType::Enchantment], &["Elf"]),
        oracle_text: "Whenever another nontoken Elf is put into your graveyard from the battlefield, you may create a 1/1 green Elf Warrior creature token.".to_string(),
        abilities: vec![
            // TODO: "Whenever another nontoken Elf is put into your graveyard from the battlefield"
            // — WheneverCreatureDies is overbroad (fires for all creatures, not just nontoken Elves
            // you control). DSL lacks a triggered condition with subtype + nontoken + controller
            // filter. Per W5 policy, leaving empty to avoid wrong game state.
        ],
        ..Default::default()
    }
}

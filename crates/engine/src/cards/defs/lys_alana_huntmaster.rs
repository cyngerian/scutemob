// Lys Alana Huntmaster — {2}{G}{G}, Creature — Elf Warrior 3/3
// Whenever you cast an Elf spell, you may create a 1/1 green Elf Warrior creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lys-alana-huntmaster"),
        name: "Lys Alana Huntmaster".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "Whenever you cast an Elf spell, you may create a 1/1 green Elf Warrior creature token.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: WheneverYouCastSpell lacks spell-type filter (Elf spells only).
            //   Overbroad trigger removed to avoid wrong game state.
        ],
        ..Default::default()
    }
}

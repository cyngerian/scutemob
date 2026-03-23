// Selvala, Heart of the Wilds — {1}{G}{G}, Legendary Creature — Elf Scout 2/3
// Whenever another creature enters, its controller may draw a card if its power is
// greater than each other creature's power.
// {G}, {T}: Add X mana in any combination of colors, where X is the greatest power
// among creatures you control.
//
// TODO: Both abilities require dynamic power comparisons not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("selvala-heart-of-the-wilds"),
        name: "Selvala, Heart of the Wilds".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Scout"]),
        oracle_text: "Whenever another creature enters, its controller may draw a card if its power is greater than each other creature's power.\n{G}, {T}: Add X mana in any combination of colors, where X is the greatest power among creatures you control.".to_string(),
        power: Some(2),
        toughness: Some(3),
        // TODO: Power comparison triggers + greatest-power mana not expressible.
        abilities: vec![],
        ..Default::default()
    }
}

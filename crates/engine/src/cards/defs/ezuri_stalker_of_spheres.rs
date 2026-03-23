// Ezuri, Stalker of Spheres — {2}{G}{U}, Legendary Creature — Phyrexian Elf Warrior 3/3
// When Ezuri enters, you may pay {3}. If you do, proliferate twice.
// Whenever you proliferate, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ezuri-stalker-of-spheres"),
        name: "Ezuri, Stalker of Spheres".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Elf", "Warrior"],
        ),
        oracle_text: "When Ezuri enters, you may pay {3}. If you do, proliferate twice.\nWhenever you proliferate, draw a card.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: "May pay {3}" optional ETB cost not in DSL (MayPaySelf).
            //   Free proliferate without payment would be wrong game state.
            // TODO: "Whenever you proliferate" trigger not in DSL.
        ],
        ..Default::default()
    }
}

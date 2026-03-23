// Springheart Nantuko — {1}{G}, Enchantment Creature — Insect Monk 1/1
// Bestow {1}{G}. Enchanted creature gets +1/+1.
// Landfall — Whenever a land you control enters, you may pay {1}{G} if attached to a
// creature you control. If you do, create a token copy of that creature. If you didn't,
// create a 1/1 green Insect token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("springheart-nantuko"),
        name: "Springheart Nantuko".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Insect", "Monk"]),
        oracle_text: "Bestow {1}{G}\nEnchanted creature gets +1/+1.\nLandfall \u{2014} Whenever a land you control enters, you may pay {1}{G} if this permanent is attached to a creature you control. If you do, create a token that's a copy of that creature. If you didn't create a token this way, create a 1/1 green Insect creature token.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // TODO: Bestow {1}{G} — Bestow keyword not in DSL
            // TODO: Enchanted creature gets +1/+1 — Aura static grant
            // TODO: Landfall trigger — conditional copy-or-Insect branch not in DSL
        ],
        ..Default::default()
    }
}

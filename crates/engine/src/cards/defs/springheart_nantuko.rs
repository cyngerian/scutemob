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
            // TODO: Blockers (NOT Landfall — that trigger exists via
            // TriggerCondition::WheneverPermanentEntersBattlefield + Land + You filter,
            // CR 207.2c):
            //   1. Bestow keyword (AltCostKind::Bestow not in DSL).
            //   2. Aura static grant (+1/+1 to enchanted creature).
            //   3. Conditional copy-or-Insect-fallback branch within a single triggered effect
            //      (pay {1}{G} if attached → copy token, else 1/1 Insect fallback).
        ],
        ..Default::default()
    }
}

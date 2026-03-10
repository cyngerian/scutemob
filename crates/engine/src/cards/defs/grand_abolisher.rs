// Grand Abolisher — {W}{W}, Creature — Human Cleric 2/2.
// "During your turn, your opponents can't cast spells or activate abilities
// of artifacts, creatures, or enchantments."
// TODO: DSL gap — timing restriction on opponents not expressible.
// Only the creature body is defined.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grand-abolisher"),
        name: "Grand Abolisher".to_string(),
        mana_cost: Some(ManaCost { white: 2, ..Default::default() }),
        types: creature_types(&["Human", "Cleric"]),
        oracle_text: "During your turn, your opponents can't cast spells or activate abilities of artifacts, creatures, or enchantments.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        // TODO: static ability restricting opponent actions during your turn
        back_face: None,
    }
}

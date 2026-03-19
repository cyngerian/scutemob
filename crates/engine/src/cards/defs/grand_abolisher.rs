// Grand Abolisher — {W}{W}, Creature — Human Cleric 2/2.
// "During your turn, your opponents can't cast spells or activate abilities
// of artifacts, creatures, or enchantments."
// PB-18: Both the casting restriction and the ability-activation restriction are
// implemented via AbilityDefinition::StaticRestriction { OpponentsCantCastOrActivateDuringYourTurn }.
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
        abilities: vec![
            // PB-18: "During your turn, your opponents can't cast spells or activate
            // abilities of artifacts, creatures, or enchantments."
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::OpponentsCantCastOrActivateDuringYourTurn,
            },
        ],
        ..Default::default()
    }
}

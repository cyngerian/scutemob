// Drannith Magistrate — {1}{W}, Creature — Human Wizard 1/3
// Your opponents can't cast spells from anywhere other than their hands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("drannith-magistrate"),
        name: "Drannith Magistrate".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Your opponents can't cast spells from anywhere other than their hands.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::OpponentsCantCastFromNonHand,
            },
        ],
        ..Default::default()
    }
}

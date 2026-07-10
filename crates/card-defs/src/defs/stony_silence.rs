// Stony Silence — {1}{W}, Enchantment
// Activated abilities of artifacts can't be activated.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stony-silence"),
        name: "Stony Silence".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Activated abilities of artifacts can't be activated.".to_string(),
        abilities: vec![
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::ArtifactAbilitiesCantBeActivated,
            },
        ],
        ..Default::default()
    }
}

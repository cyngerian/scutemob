// Azusa, Lost but Seeking — {2}{G}, Legendary Creature — Human Monk 1/2
// You may play two additional lands on each of your turns.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("azusa-lost-but-seeking"),
        name: "Azusa, Lost but Seeking".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Monk"]),
        oracle_text: "You may play two additional lands on each of your turns.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // CR 305.2: Static ability granting two additional land plays per turn.
            AbilityDefinition::AdditionalLandPlays { count: 2 },
        ],
        ..Default::default()
    }
}

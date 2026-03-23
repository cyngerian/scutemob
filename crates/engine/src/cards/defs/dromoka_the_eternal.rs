// Dromoka, the Eternal — {3}{G}{W}, Legendary Creature — Dragon 5/5
// Flying
// Whenever a Dragon you control attacks, bolster 2.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dromoka-the-eternal"),
        name: "Dromoka, the Eternal".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\nWhenever a Dragon you control attacks, bolster 2.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — "Whenever a Dragon you control attacks" trigger condition +
            // Bolster 2. Effect::Bolster exists but attack trigger with subtype filter
            // does not.
        ],
        ..Default::default()
    }
}

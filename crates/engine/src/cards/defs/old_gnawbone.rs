// Old Gnawbone — {5}{G}{G}, Legendary Creature — Dragon 7/7
// Flying
// Whenever a creature you control deals combat damage to a player, create that many
// Treasure tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("old-gnawbone"),
        name: "Old Gnawbone".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\nWhenever a creature you control deals combat damage to a player, create that many Treasure tokens.".to_string(),
        power: Some(7),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Whenever a creature you control deals combat damage" —
            //   per-creature combat damage trigger not in DSL.
        ],
        ..Default::default()
    }
}

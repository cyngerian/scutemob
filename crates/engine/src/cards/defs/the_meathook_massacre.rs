// The Meathook Massacre
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-meathook-massacre"),
        name: "The Meathook Massacre".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Enchantment], &[]),
        oracle_text: "When The Meathook Massacre enters, each creature gets -X/-X until end of turn.
Whenever a creature you control dies, each opponent loses 1 life.
Whenever a creature an opponent controls dies, you gain 1 life.".to_string(),
        abilities: vec![
            // TODO: DSL gap — X cost ETB: "each creature gets -X/-X" needs X mana value
            // at resolution + mass ApplyContinuousEffect to AllCreatures.
            // TODO: DSL gap — two death triggers with controller filters (you vs opponent).
            // WheneverCreatureDies has no controller filter.
        ],
        ..Default::default()
    }
}

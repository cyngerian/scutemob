// Reconnaissance Mission — {2}{U}{U}, Enchantment
// Whenever a creature you control deals combat damage to a player, you may draw a card.
// Cycling {2}
//
// TODO: "Whenever a creature you control deals combat damage to a player" —
//   needs per-creature combat damage trigger, not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reconnaissance-mission"),
        name: "Reconnaissance Mission".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control deals combat damage to a player, you may draw a card.\nCycling {2}".to_string(),
        abilities: vec![
            // TODO: Per-creature combat damage trigger not in DSL.
            // Cycling {2}
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { generic: 2, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}

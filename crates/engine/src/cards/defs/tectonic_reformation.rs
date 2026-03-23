// Tectonic Reformation — {1}{R}, Enchantment
// Each land card in your hand has cycling {R}.
// Cycling {2}
//
// TODO: "Grant cycling to cards in hand" — static ability grant to cards in hand not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tectonic-reformation"),
        name: "Tectonic Reformation".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Each land card in your hand has cycling {R}.\nCycling {2}".to_string(),
        abilities: vec![
            // TODO: Grant cycling to hand lands not expressible.
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { generic: 2, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}

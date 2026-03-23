// Xorn — {2}{R}, Creature — Elemental 3/2
// If you would create one or more Treasure tokens, instead create those tokens
// plus an additional Treasure token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("xorn"),
        name: "Xorn".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Elemental"]),
        oracle_text: "If you would create one or more Treasure tokens, instead create those tokens plus an additional Treasure token.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // TODO: Replacement effect for token creation quantity ("create one more Treasure")
            //   not expressible in DSL. Would need ReplacementModification::AddExtraToken or
            //   similar variant. W5 policy: no implementation produces wrong game state.
        ],
        ..Default::default()
    }
}

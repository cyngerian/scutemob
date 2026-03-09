// Strike It Rich
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("strike-it-rich"),
        name: "Strike It Rich".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Create a Treasure token. (It's an artifact with \"{T}, Sacrifice this token: Add one mana of any color.\")\nFlashback {2}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.)".to_string(),
        abilities: vec![
            // CR 702.34a: Flashback marker — enables casting from graveyard in casting.rs.
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            // CR 702.34a: The flashback cost itself ({2}{R}).
            AbilityDefinition::AltCastAbility { kind: AltCostKind::Flashback, details: None,
                cost: ManaCost { generic: 2, red: 1, ..Default::default() },
            },
            // The spell effect: create one Treasure token.
            AbilityDefinition::Spell {
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

// Moment's Peace — {1}{G}, Instant
// Prevent all combat damage that would be dealt this turn.
// Flashback {2}{G}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("moments-peace"),
        name: "Moment's Peace".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Prevent all combat damage that would be dealt this turn.\nFlashback {2}{G} (You may cast this card from your graveyard for its flashback cost. Then exile it.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Flashback,
                details: None,
                cost: ManaCost { generic: 2, green: 1, ..Default::default() },
            },
            AbilityDefinition::Spell {
                effect: Effect::PreventAllCombatDamage,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

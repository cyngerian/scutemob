// Toxic Deluge — {2}{B}, Sorcery
// As an additional cost to cast this spell, pay X life.
// All creatures get -X/-X until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("toxic-deluge"),
        name: "Toxic Deluge".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "As an additional cost to cast this spell, pay X life.\nAll creatures get -X/-X until end of turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "pay X life" additional cost + -X/-X scaled by life paid.
            // Needs XValue-based life payment and negative ModifyBoth.
            effect: Effect::Nothing,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

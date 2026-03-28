// Mana Confluence — Land, {T}, Pay 1 life: Add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-confluence"),
        name: "Mana Confluence".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Pay 1 life: Add one mana of any color.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)]),
            effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
            timing_restriction: None,
            targets: vec![],
                activation_condition: None,
                activation_zone: None,
        }],
        ..Default::default()
    }
}

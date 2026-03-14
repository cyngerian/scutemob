// 9. Command Tower — Land, tap: add one mana of any color in your commander's
// color identity.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("command-tower"),
        name: "Command Tower".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add one mana of any color in your commander's color identity.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
            timing_restriction: None,
            targets: vec![],
        }],
        ..Default::default()
    }
}

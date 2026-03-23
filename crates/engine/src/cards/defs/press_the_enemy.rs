// Press the Enemy — {2}{U}{U}, Instant
// Return target spell or nonland permanent an opponent controls to its owner's hand.
// You may cast an instant or sorcery spell with equal or lesser mana value from your
// hand without paying its mana cost.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("press-the-enemy"),
        name: "Press the Enemy".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Return target spell or nonland permanent an opponent controls to its owner's hand. You may cast an instant or sorcery spell with equal or lesser mana value from your hand without paying its mana cost.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Targets a spell or nonland permanent an opponent controls — dual zone
            // targeting (stack + battlefield). Bounce + free cast based on mana value comparison.
            effect: Effect::Nothing,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

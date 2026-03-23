// Archmage's Charm — {U}{U}{U}, Instant
// Choose one —
// • Counter target spell.
// • Target player draws two cards.
// • Gain control of target nonland permanent with mana value 1 or less.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archmages-charm"),
        name: "Archmage's Charm".to_string(),
        mana_cost: Some(ManaCost { blue: 3, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Counter target spell.\n• Target player draws two cards.\n• Gain control of target nonland permanent with mana value 1 or less.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Modal spell (choose one of three). Mode 1: counter target spell.
            // Mode 2: target player draws two cards. Mode 3: gain control of target
            // nonland permanent with mana value 1 or less (requires MV filter).
            effect: Effect::Nothing,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

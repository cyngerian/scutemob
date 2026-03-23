// Cryptic Command — {1}{U}{U}{U}, Instant
// Choose two —
// • Counter target spell.
// • Return target permanent to its owner's hand.
// • Tap all creatures your opponents control.
// • Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cryptic-command"),
        name: "Cryptic Command".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 3, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose two —\n• Counter target spell.\n• Return target permanent to its owner's hand.\n• Tap all creatures your opponents control.\n• Draw a card.".to_string(),
        abilities: vec![
            // TODO: Modal spell (choose two of four). Mode 1: counter target spell.
            // Mode 2: bounce target permanent. Mode 3: tap all opponent creatures.
            // Mode 4: draw a card. Requires multi-modal with per-mode targets.
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

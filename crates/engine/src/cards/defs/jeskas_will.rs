// Jeska's Will — {2}{R}, Sorcery
// Choose one. If you control a commander, you may choose both.
// • Add {R} for each card in target opponent's hand.
// • Exile the top three cards of your library. You may play them this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("jeskas-will"),
        name: "Jeska's Will".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one. If you control a commander as you cast this spell, you may choose both instead.\n• Add {R} for each card in target opponent's hand.\n• Exile the top three cards of your library. You may play them this turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Mode 1 needs mana-scaled-by-opponent-hand-count.
            // TODO: Mode 2 needs impulse-draw (exile top 3, play this turn).
            // TODO: "choose both if commander" conditional entwine.
            effect: Effect::Nothing,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// Silence — {W}, Instant
// Your opponents can't cast spells this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("silence"),
        name: "Silence".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Your opponents can't cast spells this turn.".to_string(),
        abilities: vec![
            // TODO: "Your opponents can't cast spells this turn" — needs a one-shot
            // effect that registers a turn-scoped restriction (not a static from a
            // permanent). No Effect variant for temporary game restrictions exists.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

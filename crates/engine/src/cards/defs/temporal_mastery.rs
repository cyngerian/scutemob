// Temporal Mastery — {5}{U}{U}, Sorcery
// Take an extra turn after this one. Exile Temporal Mastery.
// Miracle {1}{U} (You may cast this card for its miracle cost when you draw it
// if it's the first card you drew this turn.)
//
// CR 500.7: "Take an extra turn after this one."
// self_exile_on_resolution: "Exile Temporal Mastery." — the spell exiles itself
// after resolving instead of going to the graveyard.
// CR 702.94: Miracle — may cast for {1}{U} when drawn as the first card this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("temporal-mastery"),
        name: "Temporal Mastery".to_string(),
        mana_cost: Some(ManaCost { generic: 5, blue: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Take an extra turn after this one. Exile Temporal Mastery.\nMiracle {1}{U} (You may cast this card for its miracle cost when you draw it if it's the first card you drew this turn.)".to_string(),
        abilities: vec![
            // CR 702.94a: Miracle keyword marker — enables miracle casting in miracle.rs.
            AbilityDefinition::Keyword(KeywordAbility::Miracle),
            // CR 702.94a: The miracle alternative cost ({1}{U}).
            AbilityDefinition::Miracle {
                cost: ManaCost { generic: 1, blue: 1, ..Default::default() },
            },
            // CR 500.7: Take an extra turn after this one.
            AbilityDefinition::Spell {
                effect: Effect::ExtraTurn {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        // "Exile Temporal Mastery." — self-exile on successful resolution.
        self_exile_on_resolution: true,
        ..Default::default()
    }
}

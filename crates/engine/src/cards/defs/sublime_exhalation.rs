// 76. Sublime Exhalation — {6}{W}, Sorcery; Undaunted (This spell costs {1} less to
// cast for each of your opponents.)
// Destroy all creatures.
// In a 4-player Commander game (3 opponents), costs {3}{W} instead of {6}{W}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sublime-exhalation"),
        name: "Sublime Exhalation".to_string(),
        mana_cost: Some(ManaCost { generic: 6, white: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Undaunted (This spell costs {1} less to cast for each of your opponents.)\nDestroy all creatures.".to_string(),
        abilities: vec![
            // CR 702.125a: Undaunted keyword marker.
            AbilityDefinition::Keyword(KeywordAbility::Undaunted),
            // Spell effect: destroy all creatures.
            AbilityDefinition::Spell {
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::AllCreatures,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

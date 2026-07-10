// Despark — {W}{B}, Instant
// Exile target permanent with mana value 4 or greater.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("despark"),
        name: "Despark".to_string(),
        mana_cost: Some(ManaCost { white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Exile target permanent with mana value 4 or greater.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ExileObject {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                min_cmc: Some(4),
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

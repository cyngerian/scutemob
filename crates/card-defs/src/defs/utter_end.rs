// Utter End — {2}{W}{B}, Instant
// Exile target nonland permanent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("utter-end"),
        name: "Utter End".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Exile target nonland permanent.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ExileObject {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                non_land: true,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

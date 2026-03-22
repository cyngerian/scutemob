// Fracture — {W}{B}, Instant; destroy target artifact, enchantment, or planeswalker.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fracture"),
        name: "Fracture".to_string(),
        mana_cost: Some(ManaCost { white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target artifact, enchantment, or planeswalker.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                has_card_types: vec![
                    CardType::Artifact,
                    CardType::Enchantment,
                    CardType::Planeswalker,
                ],
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

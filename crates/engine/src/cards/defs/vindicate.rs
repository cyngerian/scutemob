// Vindicate — {1}{W}{B} Sorcery; destroy target permanent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vindicate"),
        name: "Vindicate".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Destroy target permanent.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetPermanent],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

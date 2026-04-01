// Go for the Throat — {1}{B}, Instant
// Destroy target nonartifact creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("go-for-the-throat"),
        name: "Go for the Throat".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target nonartifact creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "nonartifact creature" — no exclude_card_types on TargetFilter.
            // This targets any creature including artifact creatures.
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

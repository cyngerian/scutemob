// Fell the Profane // Fell Mire — {2}{B}{B} Instant // Land (MDFC)
// Oracle: "Destroy target creature or planeswalker."
// Note: TargetCreature approximation — no TargetCreatureOrPlaneswalker variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fell-the-profane"),
        name: "Fell the Profane // Fell Mire".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target creature or planeswalker.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
            },
            // TODO: Should be TargetCreatureOrPlaneswalker — no such variant; using TargetCreature.
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

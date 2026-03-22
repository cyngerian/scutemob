// Atomize — {2}{B}{G}, Instant
// Destroy target nonland permanent. Proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("atomize"),
        name: "Atomize".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target nonland permanent. Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // Destroy target nonland permanent.
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                // Proliferate.
                Effect::Proliferate,
            ]),
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

// Wasteland — Land
// {T}: Add {C}. {T}, Sacrifice this land: Destroy target nonbasic land.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wasteland"),
        name: "Wasteland".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}, Sacrifice this land: Destroy target nonbasic land.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // {T}, Sacrifice this land: Destroy target nonbasic land.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::SacrificeSelf]),
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                // TODO: Target should be "nonbasic land" — TargetFilter lacks non_basic exclusion field.
                // Using TargetLand as approximation (allows targeting basic lands too).
                targets: vec![TargetRequirement::TargetLand],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

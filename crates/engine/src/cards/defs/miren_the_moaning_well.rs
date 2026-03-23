// Miren, the Moaning Well — Legendary Land
// {T}: Add {C}.
// {3}, {T}, Sacrifice a creature: You gain life equal to the sacrificed creature's toughness.
//
// TODO: "gain life equal to the sacrificed creature's toughness" requires
// EffectAmount::SacrificedCreatureToughness (dynamic amount based on sacrificed creature stats).
// This DSL primitive does not exist. The {T}: Add {C} ability is implemented; the
// life-gain ability is omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("miren-the-moaning-well"),
        name: "Miren, the Moaning Well".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{3}, {T}, Sacrifice a creature: You gain life equal to the sacrificed creature's toughness.".to_string(),
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
            // TODO: {3}, {T}, Sacrifice a creature: Gain life = sacrificed creature's toughness.
            // Requires EffectAmount::SacrificedCreatureToughness which does not exist in the DSL.
        ],
        ..Default::default()
    }
}

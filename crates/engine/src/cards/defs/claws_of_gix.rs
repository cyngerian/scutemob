// Claws of Gix — {0}, Artifact
// {1}, Sacrifice a permanent: You gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("claws-of-gix"),
        name: "Claws of Gix".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{1}, Sacrifice a permanent: You gain 1 life.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter::default()),
                ]),
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

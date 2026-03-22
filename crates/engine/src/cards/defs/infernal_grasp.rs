// Infernal Grasp — {1}{B} Instant; destroy target creature. You lose 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("infernal-grasp"),
        name: "Infernal Grasp".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target creature. You lose 2 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                Effect::LoseLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
            ]),
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

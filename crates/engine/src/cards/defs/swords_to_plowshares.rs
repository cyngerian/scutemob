// 20. Swords to Plowshares — {W}, Instant
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("swords-to-plowshares"),
        name: "Swords to Plowshares".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Exile target creature. Its controller gains life equal to its power."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::ExileObject {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                Effect::GainLife {
                    player: PlayerTarget::ControllerOf(Box::new(
                        EffectTarget::DeclaredTarget { index: 0 },
                    )),
                    amount: EffectAmount::PowerOf(EffectTarget::DeclaredTarget { index: 0 }),
                },
            ]),
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

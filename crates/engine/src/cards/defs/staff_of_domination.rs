// Staff of Domination — {3} Artifact
// {1}: Untap this artifact.
// {2}, {T}: You gain 1 life.
// {3}, {T}: Untap target creature.
// {4}, {T}: Tap target creature.
// {5}, {T}: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("staff-of-domination"),
        name: "Staff of Domination".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{1}: Untap this artifact.\n{2}, {T}: You gain 1 life.\n{3}, {T}: Untap target creature.\n{4}, {T}: Tap target creature.\n{5}, {T}: Draw a card.".to_string(),
        abilities: vec![
            // {1}: Untap this artifact.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                effect: Effect::UntapPermanent { target: EffectTarget::Source },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {2}, {T}: You gain 1 life.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {3}, {T}: Untap target creature.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::UntapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {4}, {T}: Tap target creature.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 4, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::TapPermanent { target: EffectTarget::DeclaredTarget { index: 0 } },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {5}, {T}: Draw a card.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 5, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

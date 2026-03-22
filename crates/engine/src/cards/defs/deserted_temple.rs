// Deserted Temple — Land, {T}: Add {C}. {1}, {T}: Untap target land.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deserted-temple"),
        name: "Deserted Temple".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}, {T}: Untap target land.".to_string(),
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
            // {1}, {T}: Untap target land.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetLand],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

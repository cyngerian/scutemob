// Kor Haven — Legendary Land
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kor-haven"),
        name: "Kor Haven".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Legendary], &[CardType::Land], &[]),
        oracle_text: "{T}: Add {C}.\n{1}{W}, {T}: Prevent all combat damage that would be dealt \
                      by target attacking creature this turn."
            .to_string(),
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
                activation_zone: None,
                once_per_turn: false,
            },
            // CR 615.1: {1}{W},{T}: Prevent all combat damage dealt BY target attacking creature.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        white: 1,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::PreventCombatDamageFromOrTo {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    prevent_from: true,
                    prevent_to: false,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    is_attacking: true,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}

// Minamo, School at Water's Edge — Legendary Land
// {T}: Add {U}. {U}, {T}: Untap target legendary permanent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("minamo-school-at-waters-edge"),
        name: "Minamo, School at Water's Edge".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {U}.\n{U}, {T}: Untap target legendary permanent.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {U}, {T}: Untap target legendary permanent.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { blue: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    ..Default::default()
                })],
                // TODO: Target should be "legendary permanent" — TargetFilter lacks supertype constraint.
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

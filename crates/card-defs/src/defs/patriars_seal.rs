// Patriar's Seal — {3}, Artifact
// {T}: Add one mana of any color.
// {1}, {T}: Untap target legendary creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("patriars-seal"),
        name: "Patriar's Seal".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add one mana of any color.\n{1}, {T}: Untap target legendary creature \
                      you control."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {1}, {T}: Untap target legendary creature you control.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        ..Default::default()
                    }),
                    Cost::Tap,
                ]),
                effect: Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    legendary: true,
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        // PB-EF12 (EF-W-PB2-3): un-marked, see birds_of_paradise.rs for the fix.
        ..Default::default()
    }
}

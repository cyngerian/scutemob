// Aggravated Assault — {2}{R}, Enchantment
// {3}{R}{R}: Untap all creatures you control. After this main phase, there is an
// additional combat phase followed by an additional main phase. Activate only as a
// sorcery.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aggravated-assault"),
        name: "Aggravated Assault".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "{3}{R}{R}: Untap all creatures you control. After this main phase, there is \
                      an additional combat phase followed by an additional main phase. Activate \
                      only as a sorcery."
            .to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Mana(ManaCost {
                generic: 3,
                red: 2,
                ..Default::default()
            }),
            effect: Effect::Sequence(vec![
                Effect::UntapAll {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        ..Default::default()
                    },
                },
                // CR 500.8/505.1a: an additional combat phase, followed by an additional
                // (postcombat) main phase.
                Effect::AdditionalCombatPhase {
                    followed_by_main: true,
                },
            ]),
            timing_restriction: Some(TimingRestriction::SorcerySpeed),
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

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
            },
        ],
        completeness: Completeness::known_wrong(
            "Untap ability now correctly implemented (TargetCreatureWithFilter{legendary:true, \
             controller:You}). Real blocker: the mana ability uses Effect::AddManaAnyColor, which \
             is gated out of Complete by tests/core/effect_choose_gate.rs (SR-37/SF-11) — it \
             always adds ManaColor::Colorless, not a chosen color, so 'Add one mana of any color' \
             produces wrong game state. Needs the per-colour-ability rewire pattern \
             (tainted_field.rs) once that primitive lands for artifacts.",
        ),
        ..Default::default()
    }
}

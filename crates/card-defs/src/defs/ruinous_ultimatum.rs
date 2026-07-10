// Ruinous Ultimatum — {R}{R}{W}{W}{W}{B}{B} Sorcery
// Destroy all nonland permanents your opponents control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ruinous-ultimatum"),
        name: "Ruinous Ultimatum".to_string(),
        mana_cost: Some(ManaCost {
            red: 2,
            white: 3,
            black: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Destroy all nonland permanents your opponents control.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.8: Destroy all nonland permanents opponents control.
            // TargetController::Opponent enforces the "your opponents control" constraint.
            effect: Effect::DestroyAll {
                filter: TargetFilter {
                    non_land: true,
                    controller: TargetController::Opponent,
                    ..Default::default()
                },
                cant_be_regenerated: false,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

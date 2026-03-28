// Ravenous Chupacabra — {2}{B}{B} Creature — Beast Horror 2/2
// When this creature enters, destroy target creature an opponent controls.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ravenous-chupacabra"),
        name: "Ravenous Chupacabra".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: creature_types(&["Beast", "Horror"]),
        oracle_text: "When this creature enters, destroy target creature an opponent controls."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 603.1: ETB trigger — destroy target creature an opponent controls.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

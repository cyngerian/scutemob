// Goblin Lookout — {1}{R}, Creature — Goblin 1/2
// {T}, Sacrifice a Goblin: Goblin creatures get +2/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-lookout"),
        name: "Goblin Lookout".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin"]),
        oracle_text: "{T}, Sacrifice a Goblin: Goblin creatures get +2/+0 until end of turn."
            .to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![
                Cost::Tap,
                Cost::Sacrifice(TargetFilter {
                    has_subtype: Some(SubType("Goblin".to_string())),
                    ..Default::default()
                }),
            ]),
            // "Goblin creatures get +2/+0" is unrestricted by controller — ALL Goblin
            // creatures on the battlefield, not just yours (AllCreaturesWithSubtype).
            effect: Effect::ApplyContinuousEffect {
                effect_def: Box::new(ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(2),
                    filter: EffectFilter::AllCreaturesWithSubtype(SubType("Goblin".to_string())),
                    duration: EffectDuration::UntilEndOfTurn,
                    condition: None,
                }),
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    }
}

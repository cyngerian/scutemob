// Goblin Sledder — {R}, Creature — Goblin 1/1
// Sacrifice a Goblin: Target creature gets +1/+1 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-sledder"),
        name: "Goblin Sledder".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Sacrifice a Goblin: Target creature gets +1/+1 until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_subtype: Some(SubType("Goblin".to_string())),
                    ..Default::default()
                }),
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(1),
                        filter: EffectFilter::DeclaredTarget { index: 0 },
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreature],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

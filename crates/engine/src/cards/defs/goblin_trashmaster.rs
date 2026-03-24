// Goblin Trashmaster — {2}{R}{R}, Creature — Goblin Warrior 3/3
// Other Goblins you control get +1/+1.
// Sacrifice a Goblin: Destroy target artifact.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-trashmaster"),
        name: "Goblin Trashmaster".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Other Goblins you control get +1/+1.\nSacrifice a Goblin: Destroy target artifact.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // CR 613.1c / Layer 7c: "Other Goblins you control get +1/+1."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Goblin".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Sacrifice a Goblin: Destroy target artifact.
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    has_subtype: Some(SubType("Goblin".to_string())),
                    ..Default::default()
                }),
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetArtifact],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

// Rundvelt Hordemaster
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rundvelt-hordemaster"),
        name: "Rundvelt Hordemaster".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Other Goblins you control get +1/+1.
Whenever Rundvelt Hordemaster or another Goblin you control dies, exile the top card of your library. If it's a Goblin creature card, you may cast that card until the end of your next turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Goblin".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: DSL gap — death trigger with controller filter (Goblin you control)
            // + exile top card + conditional cast permission.
        ],
        ..Default::default()
    }
}

// Citanul Hierophants — {3}{G} Creature — Human Druid 3/2
// Creatures you control have "{T}: Add {G}."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("citanul-hierophants"),
        name: "Citanul Hierophants".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Human", "Druid"]),
        oracle_text: "Creatures you control have \"{T}: Add {G}.\"".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // CR 613.1f: Layer 6 static ability — grants {T}: Add {G} to each
            // creature you control (including itself) while it's on the battlefield.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddManaAbility(
                        ManaAbility::tap_for(ManaColor::Green),
                    ),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

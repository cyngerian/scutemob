// Bloodmark Mentor — {1}{R}, Creature — Goblin Warrior 1/1
// Red creatures you control have first strike.
//
// CR 613.1f (Layer 6): Static ability — grant filtered by color.
// Colors are layer-resolved before Layer 6 (Layers 4-5 precede ability grants).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodmark-mentor"),
        name: "Bloodmark Mentor".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Red creatures you control have first strike.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 613.1f (Layer 6): "Red creatures you control have first strike."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::FirstStrike),
                    filter: EffectFilter::CreaturesYouControlWithColor(Color::Red),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

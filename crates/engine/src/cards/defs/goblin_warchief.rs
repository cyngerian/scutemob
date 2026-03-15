// Goblin Warchief — {1}{R}{R}, Creature — Goblin Warrior 2/2
// Goblin spells you cast cost {1} less to cast.
// Goblins you control have haste.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-warchief"),
        name: "Goblin Warchief".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Goblin spells you cast cost {1} less to cast.\nGoblins you control have haste.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // "Goblins you control have haste" — static keyword grant.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Goblin".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
        ],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasSubtype(SubType("Goblin".to_string())),
            scope: CostModifierScope::Controller,
            eminence: false,
        }],
        ..Default::default()
    }
}

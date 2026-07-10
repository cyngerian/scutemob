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
            // Goblin Warchief itself is a Goblin and benefits from its own haste grant.
            // Since no CreaturesYouControlWithSubtype (self-inclusive) filter exists, we use
            // the workaround: intrinsic Haste for self + OtherCreaturesYouControlWithSubtype
            // for other Goblins. Functionally equivalent to "Goblins you control have haste."
            // CR 604.2 / CR 613.1f: Layer 6 ability-granting effect.
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // "Goblins you control have haste" — static keyword grant to other Goblins.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Goblin".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasSubtype(SubType("Goblin".to_string())),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
            colored_mana_reduction: None,
        }],
        ..Default::default()
    }
}

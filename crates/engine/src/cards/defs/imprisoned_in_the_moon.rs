// Imprisoned in the Moon — {2}{U}, Enchantment — Aura
// Enchant creature, land, or planeswalker
// Enchanted permanent is a colorless land with "{T}: Add {C}" and loses all other
//   card types and abilities.
//
// Layers 4/5/6: SetTypeLine(Land), SetColors(colorless), RemoveAllAbilities implemented.
// Note: "{T}: Add {C}" grant omitted (no LayerModification for adding mana abilities via static).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("imprisoned-in-the-moon"),
        name: "Imprisoned in the Moon".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature, land, or planeswalker\nEnchanted permanent is a colorless land with \"{T}: Add {C}\" and loses all other card types and abilities.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Permanent)),
            // CR 613.1b/d/f: Enchanted permanent is a colorless land with "{T}: Add {C}"
            // and loses all other card types and abilities.
            // Layer 4: SetTypeLine to Land only.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::SetTypeLine {
                        supertypes: im::OrdSet::new(),
                        card_types: [CardType::Land].into_iter().collect(),
                        subtypes: im::OrdSet::new(),
                    },
                    filter: EffectFilter::AttachedPermanent,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 5: BecomeColorless.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::ColorChange,
                    modification: LayerModification::SetColors(im::OrdSet::new()),
                    filter: EffectFilter::AttachedPermanent,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Layer 6: RemoveAllAbilities.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::RemoveAllAbilities,
                    filter: EffectFilter::AttachedPermanent,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

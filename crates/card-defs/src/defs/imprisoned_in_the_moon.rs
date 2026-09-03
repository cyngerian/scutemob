// Imprisoned in the Moon — {2}{U}, Enchantment — Aura
// Enchant creature, land, or planeswalker
// Enchanted permanent is a colorless land with "{T}: Add {C}" and loses all other
//   card types and abilities.
//
// CR 702.5a: the printed Enchant line is an OR over three CARD TYPES. Declared as
//   EnchantFilter { has_card_types: [Creature, Land, Planeswalker] } (PB-DX20b).
//   Before PB-DX20b `EnchantFilter` had no OR over card types, so this def declared
//   EnchantTarget::Permanent, which also admitted artifacts, enchantments and battles
//   (OOS-DX20-10). CR 303.4a (cast) and CR 704.5m (SBA) both enforce it.
//
// Layers 4/5/6: SetTypeLine(Land), SetColors(colorless), RemoveAllAbilities implemented.
// Note: "{T}: Add {C}" grant omitted (no LayerModification for adding mana abilities via static).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    // CR 702.5a — "Enchant creature, land, or planeswalker".
    let enchant_filter = EnchantFilter {
        has_card_types: vec![CardType::Creature, CardType::Land, CardType::Planeswalker],
        ..Default::default()
    };
    CardDefinition {
        card_id: cid("imprisoned-in-the-moon"),
        name: "Imprisoned in the Moon".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature, land, or planeswalker\nEnchanted permanent is a colorless \
                      land with \"{T}: Add {C}\" and loses all other card types and abilities."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(
                enchant_filter,
            ))),
            // CR 613.1b/d/f: Enchanted permanent is a colorless land with "{T}: Add {C}"
            // and loses all other card types and abilities.
            // Layer 4: SetTypeLine to Land only.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::TypeChange,
                    modification: LayerModification::SetTypeLine {
                        supertypes: imbl::OrdSet::new(),
                        card_types: [CardType::Land].into_iter().collect(),
                        subtypes: imbl::OrdSet::new(),
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
                    modification: LayerModification::SetColors(imbl::OrdSet::new()),
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

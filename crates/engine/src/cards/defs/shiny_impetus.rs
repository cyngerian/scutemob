// Shiny Impetus — {2}{R}, Enchantment — Aura
// Enchant creature. Enchanted creature gets +2/+2 and is goaded.
// Whenever enchanted creature attacks, you create a Treasure token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shiny-impetus"),
        name: "Shiny Impetus".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nEnchanted creature gets +2/+2 and is goaded. (It attacks each combat if able and attacks a player other than you if able.)\nWhenever enchanted creature attacks, you create a Treasure token. (It's an artifact with \"{T}, Sacrifice this token: Add one mana of any color.\")".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // Static: enchanted creature gets +2/+2
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: "Enchanted creature is goaded" — static goad applied to attached creature
            // not expressible in DSL. Goad is an Effect (not a keyword/static layer modifier).
            // TODO: "Whenever enchanted creature attacks, you create a Treasure token" —
            // WhenEnchantedCreatureAttacks trigger does not exist in the DSL.
        ],
        ..Default::default()
    }
}

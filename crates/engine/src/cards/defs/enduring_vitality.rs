// Enduring Vitality — {1}{G}{G}, Enchantment Creature — Elk Glimmer 3/3
// Vigilance
// Creatures you control have "{T}: Add one mana of any color."
// When Enduring Vitality dies, if it was a creature, return it to the battlefield
// under its owner's control. It's an enchantment. (It's not a creature.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("enduring-vitality"),
        name: "Enduring Vitality".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Elk", "Glimmer"]),
        oracle_text: "Vigilance\nCreatures you control have \"{T}: Add one mana of any color.\"\nWhen Enduring Vitality dies, if it was a creature, return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // CR 613.1f: Layer 6 static ability — grants tap-for-any-color mana ability
            // to each creature you control while this permanent is on the battlefield.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddManaAbility(ManaAbility {
                        produces: Default::default(),
                        requires_tap: true,
                        sacrifice_self: false,
                        any_color: true,
                        damage_to_controller: 0,
                    }),
                    filter: EffectFilter::CreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: die-return-as-enchantment (Enduring cycle mechanic — zone-change
            // replacement + type change not yet in DSL)
        ],
        ..Default::default()
    }
}

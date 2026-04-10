// Cryptolith Rite — {1}{G} Enchantment
// Creatures you control have "{T}: Add one mana of any color."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cryptolith-rite"),
        name: "Cryptolith Rite".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control have \"{T}: Add one mana of any color.\""
            .to_string(),
        abilities: vec![
            // CR 613.1f: Layer 6 static ability — grants the mana ability to each
            // creature you control for as long as this enchantment is on the battlefield.
            // CR 605.1a: The granted ability IS a mana ability (no target, adds mana).
            // CR 613.5: Multiple Rites each push one entry; tap cost prevents both firing.
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
        ],
        ..Default::default()
    }
}

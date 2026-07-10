// Paradise Mantle — {0}, Artifact — Equipment
// Equipped creature has "{T}: Add one mana of any color."
// Equip {1}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("paradise-mantle"),
        name: "Paradise Mantle".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has \"{T}: Add one mana of any color.\"\nEquip {1}"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // CR 613.1f: Layer 6 static ability — grants mana ability to the equipped
            // creature only (EffectFilter::AttachedCreature resolves via source.attached_to
            // at characteristic-calculation time). Unequipped = filter matches nothing.
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
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

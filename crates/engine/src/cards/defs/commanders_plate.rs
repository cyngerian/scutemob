// Commander's Plate — {1}, Artifact — Equipment
// Equipped creature gets +3/+3 and has protection from each color that's not in your
// commander's color identity.
// Equip commander {3}
// Equip {5}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("commanders-plate"),
        name: "Commander's Plate".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +3/+3 and has protection from each color that's not in your commander's color identity.\nEquip commander {3}\nEquip {5}".to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(3),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: DSL gap — dynamic protection from colors not in commander's color identity.
            // TODO: DSL gap — "Equip commander {3}" variant equip cost.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}

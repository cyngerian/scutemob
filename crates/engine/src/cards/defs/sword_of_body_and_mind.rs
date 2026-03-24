// Sword of Body and Mind — {3}, Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from green and from blue.
// Whenever equipped creature deals combat damage to a player, you create a 2/2 green
// Wolf creature token and that player mills ten cards.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-body-and-mind"),
        name: "Sword of Body and Mind".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+2 and has protection from green and from blue.\nWhenever equipped creature deals combat damage to a player, you create a 2/2 green Wolf creature token and that player mills ten cards.\nEquip {2}".to_string(),
        abilities: vec![
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: Protection from green and blue on equipped creature — multi-color
            //   protection grant not in LayerModification.
            // TODO: "Equipped creature deals combat damage" trigger — per-creature
            //   combat damage trigger not in DSL.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}

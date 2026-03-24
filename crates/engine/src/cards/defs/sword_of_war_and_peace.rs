// Sword of War and Peace — {3}, Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from red and from white.
// Whenever equipped creature deals combat damage to a player, this Equipment deals damage
// to that player equal to the number of cards in their hand and you gain 1 life for each
// card in your hand.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-war-and-peace"),
        name: "Sword of War and Peace".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +2/+2 and has protection from red and from white.\nWhenever equipped creature deals combat damage to a player, Sword of War and Peace deals damage to that player equal to the number of cards in their hand and you gain 1 life for each card in your hand.\nEquip {2}".to_string(),
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
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red))),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::White))),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: DSL gap — equipped creature combat damage trigger + hand-size damage + life gain.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}

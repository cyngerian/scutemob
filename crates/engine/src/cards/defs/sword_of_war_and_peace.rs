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
            // CR 510.3a: "Whenever equipped creature deals combat damage to a player,
            // deals damage to that player equal to cards in their hand; gain 1 life per card in
            // your hand." DamagedPlayer resolves from ctx.damaged_player at resolution.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::CardCount {
                            zone: ZoneTarget::Hand { owner: PlayerTarget::DeclaredTarget { index: 0 } },
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            filter: None,
                        },
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::CardCount {
                            zone: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                            player: PlayerTarget::Controller,
                            filter: None,
                        },
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPlayer],
            },
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}

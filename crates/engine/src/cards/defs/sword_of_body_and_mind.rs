// Sword of Body and Mind — {3}, Artifact — Equipment
// Equipped creature gets +2/+2 and has protection from green and from blue.
// Whenever equipped creature deals combat damage to a player, you create a 2/2 green
// Wolf creature token and that player mills ten cards.
// Equip {2}
//
// TODO: Protection from green and blue on equipped creature — multi-color
//   protection grant not in LayerModification.
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
            // TODO: Protection from green and blue on equipped creature.
            // CR 510.3a: "Whenever equipped creature deals combat damage to a player,
            // create a 2/2 green Wolf token and that player mills ten cards."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Wolf".to_string(),
                            power: 2,
                            toughness: 2,
                            colors: [Color::Green].into_iter().collect(),
                            supertypes: OrdSet::new(),
                            card_types: [CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Wolf".to_string())].into_iter().collect(),
                            keywords: OrdSet::new(),
                            count: 1,
                            tapped: false,
                            enters_attacking: false,
                            mana_color: None,
                            mana_abilities: vec![],
                            activated_abilities: vec![],
                            ..Default::default()
                        },
                    },
                    Effect::MillCards {
                        player: PlayerTarget::DamagedPlayer,
                        count: EffectAmount::Fixed(10),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
            },
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}

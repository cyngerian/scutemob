// Smoke Shroud — {1}{U}, Enchantment — Aura
// Enchant creature; enchanted creature gets +1/+1 and has flying.
// When a Ninja you control enters, you may return this from your graveyard to battlefield.
// TODO: Graveyard-return trigger (when a Ninja enters, return from GY to battlefield attached).
// DSL gap: return_from_graveyard pattern not supported; TriggerCondition has no subtype filter
// for WheneverCreatureEntersBattlefield that also triggers from graveyard zone. Deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("smoke-shroud"),
        name: "Smoke Shroud".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nEnchanted creature gets +1/+1 and has flying.\nWhen a Ninja you control enters, you may return this card from your graveyard to the battlefield attached to that creature.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // CR 613.4c: Enchanted creature gets +1/+1 (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // CR 702.9a: Enchanted creature has flying (layer 6).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // TODO: "When a Ninja you control enters, you may return this from your graveyard
            // to the battlefield attached to that creature."
            // DSL gap: triggered from graveyard zone with subtype filter, return-to-battlefield
            // attached mechanic. Deferred.
        ],
        ..Default::default()
    }
}

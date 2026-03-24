// 61. Rancor — {G}, Enchantment — Aura.
// "Enchant creature. Enchanted creature gets +2/+0 and has trample.
// When Rancor is put into a graveyard from the battlefield, return
// Rancor to its owner's hand."
//
// CR 702.5a: Enchant creature — restricts casting target and legal attachments.
// CR 613.4c: +2/+0 is a layer-7c P/T-modifying effect.
// CR 702.19a: Trample keyword granted in layer 6.
// CR 603.1: "When Rancor is put into a graveyard from the battlefield" is a
// triggered ability that fires on the zone-change event.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rancor"),
        name: "Rancor".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nEnchanted creature gets +2/+0 and has trample.\nWhen Rancor is put into a graveyard from the battlefield, return Rancor to its owner's hand.".to_string(),
        abilities: vec![
            // CR 702.5a: Enchant creature — defines legal targets and attachment restriction.
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // CR 613.4c: Enchanted creature gets +2/+0 (layer 7c, P/T modify).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 702.19a: Enchanted creature has trample (layer 6, ability grant).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Trample),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 603.1: When Rancor is put into a graveyard from the battlefield,
            // return it to its owner's hand. This trigger fires on the WhenDies
            // zone-change event (battlefield → graveyard).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

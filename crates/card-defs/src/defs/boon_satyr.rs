// 59. Boon Satyr — {1}{G}{G}, Enchantment Creature — Satyr 4/2.
// "Flash. Bestow {3}{G}{G}. Enchanted creature gets +4/+2."
//
// CARDS-2 repair (scutemob-181, playtest finding F1). Four defects, all in this file, all
// found by a human playing the game rather than by the suite: the cost was transposed to
// {2}{G} (so the card was castable off one green source), the bestow cost was {4}{G}{G},
// the Enchantment card type was missing, and the "+4/+2" clause was never authored at all —
// while the def still declared `Completeness::Complete`.
//
// CR 702.103a: Bestow is an alternative cost; the printed mana cost is unchanged (CR 118.9c).
// CR 702.103b: cast for bestow, the spell is an Aura with enchant creature.
// CR 613.4c: "+4/+2" is a layer-7c P/T-modifying effect, expressed the same way Rancor's
//            +2/+0 is — a `Static` with `EffectFilter::AttachedCreature`.
// CR 702.103f: unattached, it reverts to a creature; `attached_to` is then None and the
//            filter (`layers.rs`, `EffectFilter::AttachedCreature`) matches nothing, so the
//            bonus stops applying with no extra machinery. That is also why the same two
//            statics are correct on a card that spends most of its life as a plain creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boon-satyr"),
        name: "Boon Satyr".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Satyr"]),
        oracle_text: "Flash\nBestow {3}{G}{G} (If you cast this card for its bestow cost, it's an \
                      Aura spell with enchant creature. It becomes a creature again if it's not \
                      attached to a creature.)\nEnchanted creature gets +4/+2."
            .to_string(),
        power: Some(4),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::Bestow),
            AbilityDefinition::Bestow {
                cost: ManaCost {
                    generic: 3,
                    green: 2,
                    ..Default::default()
                },
            },
            // CR 613.4c: enchanted creature gets +4/+2 (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(4),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(2),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::Complete,
    }
}

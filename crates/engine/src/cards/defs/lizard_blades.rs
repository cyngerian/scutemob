// Lizard Blades — {1}{R}, Artifact Creature — Equipment Lizard 1/1
// Double strike
// Reconfigure {2} ({2}: Attach to target creature you control; or unattach from a creature.
// Reconfigure only as a sorcery. While attached, this isn't a creature.)
// Equipped creature gets +1/+1 and has double strike.
//
// CR 702.151a: Reconfigure is a sorcery-speed activated ability.
// CR 702.151b: While attached (is_reconfigured), loses all creature types; not a creature (Layer 4).
// CR 613.4c: +1/+1 to equipped creature is layer 7c.
// CR 702.4a: Double strike granted to equipped creature in layer 6.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lizard-blades"),
        name: "Lizard Blades".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Equipment", "Lizard"]),
        oracle_text: "Double strike\nReconfigure {2} ({2}: Attach to target creature you control; \
or unattach from a creature. Reconfigure only as a sorcery. While attached, this isn't a creature.)\n\
Equipped creature gets +1/+1 and has double strike."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 702.4a: Double strike on the creature itself.
            AbilityDefinition::Keyword(KeywordAbility::DoubleStrike),
            // CR 702.151a: Reconfigure {2} — generates attach + unattach activated abilities.
            AbilityDefinition::Keyword(KeywordAbility::Reconfigure),
            AbilityDefinition::Reconfigure { cost: ManaCost { generic: 2, ..Default::default() } },
            // CR 613.4c: Equipped creature gets +1 power (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // CR 613.4c: Equipped creature gets +1 toughness (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // CR 702.4a: Equipped creature has double strike (layer 6).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::DoubleStrike),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}

// Crown of Skemfar — {2}{G}{G}, Enchantment — Aura
// Enchant creature; enchanted creature gets +1/+1 for each Elf you control and has reach.
// {2}{G}: Return this card from your graveyard to your hand.
//
// TODO: DSL gap — static "+1/+1 for each Elf you control" count-based continuous effect
//   on the enchanted creature. No EffectAmount variant for counting permanents of a subtype.
// Graveyard return ability ({2}{G}: return to hand) is fully implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crown-of-skemfar"),
        name: "Crown of Skemfar".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature\nEnchanted creature gets +1/+1 for each Elf you control and has reach.\n{2}{G}: Return this card from your graveyard to your hand.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Creature)),
            // Static: enchanted creature has Reach (CR 613.1f, Layer 6)
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Reach),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: DSL gap — "+1/+1 for each Elf you control" count-based P/T modifier
            // (no EffectAmount variant for subtype count).
            // CR 602.2 / PB-35: "{2}{G}: Return this card from your graveyard to your hand."
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 2, green: 1, ..Default::default() }),
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: Some(ActivationZone::Graveyard),
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

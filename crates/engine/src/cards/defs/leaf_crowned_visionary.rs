// Leaf-Crowned Visionary — {G}{G}, Creature — Elf Druid 1/1
// Other Elves you control get +1/+1.
// Whenever you cast an Elf spell, you may pay {G}. If you do, draw a card.
//
// PB-AC7: unblocked by TriggerCondition::WheneverYouCastSpell.spell_subtype_filter
// (CR 205.1a) combined with PB-AC2's Effect::MayPayThenEffect (CR 118.12).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("leaf-crowned-visionary"),
        name: "Leaf-Crowned Visionary".to_string(),
        mana_cost: Some(ManaCost { green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "Other Elves you control get +1/+1.\nWhenever you cast an Elf spell, you may pay {G}. If you do, draw a card.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // Other Elves you control get +1/+1.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Elf".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Whenever you cast an Elf spell, you may pay {G}. If you do, draw a card.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: None,
                    noncreature_only: false,
                    chosen_subtype_filter: false,
                    spell_subtype_filter: Some(vec![SubType("Elf".to_string())]),
                },
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Mana(ManaCost { green: 1, ..Default::default() }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

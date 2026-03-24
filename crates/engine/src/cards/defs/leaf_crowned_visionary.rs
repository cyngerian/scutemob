// Leaf-Crowned Visionary — {G}{G}, Creature — Elf Druid 1/1
// Other Elves you control get +1/+1.
// Whenever you cast an Elf spell, you may pay {G}. If you do, draw a card.
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
            // Other Elves you control get +1/+1
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Elf".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Whenever you cast a creature spell, draw a card.
            // TODO: "Elf spell" subtype filter and "may pay {G}" not in DSL.
            // Using creature spell as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Creature]),
                    noncreature_only: false,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

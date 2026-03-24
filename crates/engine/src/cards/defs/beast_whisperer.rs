// Beast Whisperer — {2}{G}{G}, Creature — Elf Druid 2/3
// Whenever you cast a creature spell, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("beast-whisperer"),
        name: "Beast Whisperer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "Whenever you cast a creature spell, draw a card.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // Creature spell filter applied.
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

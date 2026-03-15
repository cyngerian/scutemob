// 50. Wall of Omens — {1W}, Creature — Wall 0/4; Defender; When Wall of Omens
// enters the battlefield, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wall-of-omens"),
        name: "Wall of Omens".to_string(),
        mana_cost: Some(ManaCost { white: 1, generic: 1, ..Default::default() }),
        types: creature_types(&["Wall"]),
        oracle_text: "Defender\nWhen Wall of Omens enters the battlefield, draw a card."
            .to_string(),
        power: Some(0),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Defender),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}

// Archmage of Runes — {3}{U}{U}, Creature — Giant Wizard 3/6
// Instant and sorcery spells you cast cost {1} less to cast.
// Whenever you cast an instant or sorcery spell, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archmage-of-runes"),
        name: "Archmage of Runes".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 2, ..Default::default() }),
        types: creature_types(&["Giant", "Wizard"]),
        oracle_text: "Instant and sorcery spells you cast cost {1} less to cast.\nWhenever you cast an instant or sorcery spell, draw a card.".to_string(),
        power: Some(3),
        toughness: Some(6),
        abilities: vec![
            // Instant and sorcery spells cost {1} less.
            // TODO: Cost reduction for instant/sorcery only — SelfCostReduction
            //   lacks spell-type filter. Using generic cost reduction.
            // Whenever you cast an instant or sorcery spell, draw a card.
            // TODO: WheneverYouCastSpell lacks spell-type filter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
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

// Hazoret's Monument — {3}, Legendary Artifact
// Red creature spells you cast cost {1} less to cast.
// Whenever you cast a creature spell, you may discard a card. If you do, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hazorets-monument"),
        name: "Hazoret's Monument".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "Red creature spells you cast cost {1} less to cast.\nWhenever you cast a creature spell, you may discard a card. If you do, draw a card.".to_string(),
        abilities: vec![
            // TODO: "Red creature spells cost {1} less" — color+type cost reduction not in DSL.
            // TODO: "may discard, if you do draw" — optional loot on cast trigger.
            //   WheneverYouCastSpell lacks spell-type filter. Using unfiltered cast trigger
            //   with draw only (no loot).
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

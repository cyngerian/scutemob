// Ripples of Undeath — {1}{B}, Enchantment
// At the beginning of your first main phase, mill three cards. Then you may pay
// 1 life. If you do, return a card from among those milled this way to your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ripples-of-undeath"),
        name: "Ripples of Undeath".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your first main phase, mill three cards. Then you may \
                      pay 1 life. If you do, return a card from among those milled this way to \
                      your hand."
            .to_string(),
        abilities: vec![
            // ENGINE-BLOCKED: "mill three cards. Then you may pay {1} and 3 life. If you do,
            // put a card from among those cards into your hand." The optional cost is {1} AND
            // 3 life, and the returned card must be chosen from among the cards milled by THIS
            // resolution. No milled-cards-this-resolution handle exists in the DSL to carry
            // that set forward as the selection pool.
            // (The "at the beginning of your first main phase" trigger itself is now
            // available as TriggerCondition::AtBeginningOfFirstMainPhase — PB-AC6.)
        ],
        completeness: Completeness::inert(
            "'mill three cards. Then you may pay {1} and 3 life. If you do, put a card from among \
             those cards into your hand.' The...",
        ),
        ..Default::default()
    }
}

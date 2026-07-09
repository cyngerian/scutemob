// Ripples of Undeath — {1}{B}, Enchantment
// At the beginning of your first main phase, mill three cards. Then you may pay
// 1 life. If you do, return a card from among those milled this way to your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ripples-of-undeath"),
        name: "Ripples of Undeath".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your first main phase, mill three cards. Then you may pay 1 life. If you do, return a card from among those milled this way to your hand.".to_string(),
        abilities: vec![
            // ENGINE-BLOCKED: the "return one of the milled cards" clause needs the set of
            // cards milled by THIS resolution to be carried forward as the target pool for
            // the pay-1-life optional return. No mill-tracking / milled-cards-this-resolution
            // handle exists in the DSL.
            // (The "at the beginning of your first main phase" trigger itself is now
            // available as TriggerCondition::AtBeginningOfFirstMainPhase — PB-AC6.)
        ],
        ..Default::default()
    }
}

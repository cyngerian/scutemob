// Raiders' Wake — {3}{B}, Enchantment
// Whenever an opponent discards a card, that player loses 2 life.
// Raid — At the beginning of your end step, if you attacked this turn, target opponent
// discards a card.
//
// TODO: First ability requires TriggerCondition::WheneverOpponentDiscards which does not
// exist in the DSL. The "that player" reference also needs discard trigger target passing.
// Second ability (Raid — end step discard) requires Condition::YouAttackedThisTurn as an
// intervening-if plus a targeted discard effect on an opponent; Condition::YouAttackedThisTurn
// is not confirmed to exist in the DSL. Both abilities omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("raiders-wake"),
        name: "Raiders' Wake".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent discards a card, that player loses 2 life.\nRaid — At the beginning of your end step, if you attacked this turn, target opponent discards a card.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

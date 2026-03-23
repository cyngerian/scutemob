// Tinybones, Trinket Thief — {1}{B}, Legendary Creature — Skeleton Rogue 1/2
// At the beginning of each end step, if an opponent discarded a card this turn,
// you draw a card and you lose 1 life.
// {4}{B}{B}: Each opponent with no cards in hand loses 10 life.
//
// TODO: First ability requires a game-state condition "an opponent discarded a card this turn"
// (TriggerCondition::AtBeginningOfEachEndStep with Condition::OpponentDiscardedThisTurn).
// Neither the trigger condition nor the condition variant exist in the DSL.
// Second ability requires Condition::OpponentHasNoCardsInHand (to filter which opponents
// lose life), which also does not exist. Both abilities omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tinybones-trinket-thief"),
        name: "Tinybones, Trinket Thief".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: TypeLine {
            supertypes: [SuperType::Legendary].into_iter().collect(),
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Skeleton".to_string()), SubType("Rogue".to_string())].into_iter().collect(),
        },
        oracle_text: "At the beginning of each end step, if an opponent discarded a card this turn, you draw a card and you lose 1 life.\n{4}{B}{B}: Each opponent with no cards in hand loses 10 life.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}

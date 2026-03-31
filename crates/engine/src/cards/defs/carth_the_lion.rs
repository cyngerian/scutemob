// Carth the Lion — {2}{B}{G}, Legendary Creature — Human Warrior 3/5
// Whenever Carth enters or a planeswalker you control dies, look at the top seven
// cards of your library. You may reveal a planeswalker card from among them and
// put it into your hand. Put the rest on the bottom of your library in a random order.
// Planeswalkers' loyalty abilities you activate cost an additional [+1] to activate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("carth-the-lion"),
        name: "Carth the Lion".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Warrior"]),
        oracle_text: "Whenever Carth enters or a planeswalker you control dies, look at the top seven cards of your library. You may reveal a planeswalker card from among them and put it into your hand. Put the rest on the bottom of your library in a random order.\nPlaneswalkers' loyalty abilities you activate cost an additional [+1] to activate.".to_string(),
        power: Some(3),
        toughness: Some(5),
        abilities: vec![
            // TODO: ETB trigger — "look at the top seven cards, may put a planeswalker card
            // into hand, rest on bottom in random order" — RevealAndRoute sends ALL matching
            // cards to matched_dest, but oracle says "you may reveal a planeswalker card"
            // (at most one, player chooses). DSL lacks single-card-choice-from-top-N-to-hand.
            // Also needs the alternative trigger "or a planeswalker you control dies" which
            // requires WheneverPlaneswalkerDies (not in TriggerCondition enum). Omitted per W5.

            // TODO: "Planeswalkers' loyalty abilities you activate cost an additional [+1]"
            // — requires a new LoyaltyCostModifier static effect. No equivalent in DSL.
            // Static ability omitted per W5 policy.
        ],
        ..Default::default()
    }
}

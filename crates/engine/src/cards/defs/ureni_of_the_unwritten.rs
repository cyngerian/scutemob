// Ureni of the Unwritten — {4}{G}{U}{R}, Legendary Creature — Spirit Dragon 7/7
// Flying, trample
// Whenever Ureni enters or attacks, look at the top eight cards of your library.
// You may put a Dragon creature card from among them onto the battlefield.
// Put the rest on the bottom of your library in a random order.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ureni-of-the-unwritten"),
        name: "Ureni of the Unwritten".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, blue: 1, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Spirit", "Dragon"]),
        oracle_text: "Flying, trample\nWhenever Ureni enters or attacks, look at the top eight cards of your library. You may put a Dragon creature card from among them onto the battlefield. Put the rest on the bottom of your library in a random order.".to_string(),
        power: Some(7),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: "look at top 8, you may put a Dragon creature card onto the battlefield,
            // rest on bottom in random order" — RevealAndRoute sends ALL matching cards to
            // matched_dest, but oracle says "you may put a Dragon" (at most one, player
            // chooses). DSL lacks a single-card-choice-from-top-N-to-battlefield pattern.
            // ETB trigger omitted per W5 policy.
            // TODO: Attack trigger ("whenever Ureni attacks") has the same pattern.
            // Both triggers omitted per W5 policy.
        ],
        ..Default::default()
    }
}

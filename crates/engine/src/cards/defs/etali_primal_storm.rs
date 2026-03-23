// Etali, Primal Storm — {4}{R}{R}, Legendary Creature — Elder Dinosaur 6/6
// Whenever Etali attacks, exile the top card of each player's library, then you may cast
// any number of spells from among those cards without paying their mana costs.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("etali-primal-storm"),
        name: "Etali, Primal Storm".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elder", "Dinosaur"]),
        oracle_text: "Whenever Etali attacks, exile the top card of each player's library, then you may cast any number of spells from among those cards without paying their mana costs.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            // TODO: DSL gap — the WhenAttacks trigger exiles the top card of EACH player's
            // library (ForEach over EachPlayer with ExileTopOfLibrary), then allows casting
            // any number of those exiled cards for free. The multi-player exile + conditional
            // free cast from among recently exiled cards is not expressible in the current DSL.
        ],
        ..Default::default()
    }
}

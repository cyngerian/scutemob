// Meren of Clan Nel Toth — {2}{B}{G}, Legendary Creature — Human Shaman 3/4
// Whenever another creature you control dies, you get an experience counter.
// At the beginning of your end step, choose target creature card in your graveyard. If
// that card's mana value is less than or equal to the number of experience counters you
// have, return it to the battlefield. Otherwise, put it into your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("meren-of-clan-nel-toth"),
        name: "Meren of Clan Nel Toth".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Shaman"],
        ),
        oracle_text: "Whenever another creature you control dies, you get an experience counter.\nAt the beginning of your end step, choose target creature card in your graveyard. If that card's mana value is less than or equal to the number of experience counters you have, return it to the battlefield. Otherwise, put it into your hand.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // TODO: DSL gap — "another creature you control dies" trigger with controller
            // filter + experience counter grant + end step trigger with MV comparison.
        ],
        ..Default::default()
    }
}

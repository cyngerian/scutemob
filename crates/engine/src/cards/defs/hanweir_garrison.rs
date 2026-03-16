// Hanweir Garrison — {2}{R} Creature — Human Soldier 2/3
// Whenever this creature attacks, create two 1/1 red Human creature tokens
// that are tapped and attacking.
// (Melds with Hanweir Battlements.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hanweir-garrison"),
        name: "Hanweir Garrison".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(
            &[],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "Whenever Hanweir Garrison attacks, create two 1/1 red Human creature tokens that are tapped and attacking.\n(Melds with Hanweir Battlements.)".to_string(),
        abilities: vec![
            // TODO: attack trigger — create two 1/1 red Human creature tokens tapped and attacking
            // Requires "tapped and attacking" token creation which is not yet in the DSL
        ],
        power: Some(2),
        toughness: Some(3),
        meld_pair: Some(MeldPair {
            pair_card_id: CardId("hanweir-battlements".to_string()),
            melded_card_id: CardId("hanweir-the-writhing-township".to_string()),
        }),
        ..Default::default()
    }
}

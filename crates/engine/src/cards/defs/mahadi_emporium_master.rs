// Mahadi, Emporium Master — {1}{B}{R}, Legendary Creature — Devil 3/3
// At the beginning of your end step, create a Treasure token for each creature that died
// this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mahadi-emporium-master"),
        name: "Mahadi, Emporium Master".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Devil"],
        ),
        oracle_text: "At the beginning of your end step, create a Treasure token for each creature that died this turn. (It's an artifact with \"{T}, Sacrifice this token: Add one mana of any color.\")".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: "Create a Treasure token for each creature that died this turn."
            // DSL gap: EffectAmount::CreaturesThatDiedThisTurn does not exist — no way
            // to express a count that scales with the number of creatures that died this
            // turn. A fixed count of 1 would produce wrong game state, so leaving empty.
        ],
        ..Default::default()
    }
}

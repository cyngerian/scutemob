// Tombstone Stairwell — {2}{B}{B}, World Enchantment
// Cumulative upkeep {1}{B}
// At the beginning of each upkeep, if this enchantment is on the battlefield, each player
// creates a 2/2 black Zombie creature token with haste named Tombspawn for each creature
// card in their graveyard.
// At the beginning of each end step and when this enchantment leaves the battlefield,
// destroy all tokens created with this enchantment. They can't be regenerated.
//
// CumulativeUpkeep keyword implemented.
// TODO: DSL gap — "each player's upkeep" trigger (not just controller's)
// TODO: DSL gap — "create N tokens for each creature card in graveyard" (count-based token creation)
// TODO: DSL gap — "destroy tokens created by this permanent" (token-origin tracking)
// TODO: DSL gap — "when this leaves the battlefield" combined with "begin of end step"
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tombstone-stairwell"),
        name: "Tombstone Stairwell".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: supertypes(&[SuperType::World], &[CardType::Enchantment]),
        oracle_text: "Cumulative upkeep {1}{B} (At the beginning of your upkeep, put an age counter on this permanent, then sacrifice it unless you pay its upkeep cost for each age counter on it.)\nAt the beginning of each upkeep, if this enchantment is on the battlefield, each player creates a 2/2 black Zombie creature token with haste named Tombspawn for each creature card in their graveyard.\nAt the beginning of each end step and when this enchantment leaves the battlefield, destroy all tokens created with this enchantment. They can't be regenerated.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CumulativeUpkeep(
                CumulativeUpkeepCost::Mana(ManaCost { generic: 1, black: 1, ..Default::default() }),
            )),
            AbilityDefinition::CumulativeUpkeep {
                cost: CumulativeUpkeepCost::Mana(ManaCost { generic: 1, black: 1, ..Default::default() }),
            },
            // TODO: Each player's upkeep trigger creating Tombspawn tokens per graveyard creature count
            // TODO: End step + LTB trigger destroying Tombspawn tokens
        ],
        ..Default::default()
    }
}

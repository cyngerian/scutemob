// Mardu Ascendancy — {R}{W}{B}, Enchantment
// Whenever a nontoken creature you control attacks, create a 1/1 red Goblin creature
// token that's tapped and attacking.
// Sacrifice this enchantment: Creatures you control get +0/+3 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mardu-ascendancy"),
        name: "Mardu Ascendancy".to_string(),
        mana_cost: Some(ManaCost { red: 1, white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a nontoken creature you control attacks, create a 1/1 red Goblin creature token that's tapped and attacking.\nSacrifice this enchantment: Creatures you control get +0/+3 until end of turn.".to_string(),
        abilities: vec![
            // TODO: "Whenever a nontoken creature you control attacks" — DSL lacks a
            //   WheneverAttacks trigger for any controlled creature with a nontoken filter.
            //   WhenAttacks only fires for self. W5 policy: no approximation.
            // TODO: Sacrifice activated ability with +0/+3 buff to all creatures you control —
            //   DSL ContinuousEffectDef.ModifyBoth(3) would be +3/+3, not +0/+3.
            //   Also Cost::SacrificeSelf in activated ability context needs further review.
        ],
        ..Default::default()
    }
}

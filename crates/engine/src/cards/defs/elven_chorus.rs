// Elven Chorus — {3}{G}, Enchantment
// You may look at the top card of your library any time.
// You may cast creature spells from the top of your library.
// Creatures you control have "{T}: Add one mana of any color."
//
// TODO: Three DSL gaps:
//   (1) "look at the top card of your library" — hidden info reveal not in DSL
//   (2) "cast creature spells from top of library" — cast-from-zone grant not in DSL
//   (3) "creatures you control have '{T}: Add any color'" — GrantActivatedAbility not in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elven-chorus"),
        name: "Elven Chorus".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "You may look at the top card of your library any time.\nYou may cast creature spells from the top of your library.\nCreatures you control have \"{T}: Add one mana of any color.\"".to_string(),
        abilities: vec![
            // TODO: look at top of library, cast creatures from top, grant mana ability
        ],
        ..Default::default()
    }
}

// Elven Chorus — {3}{G}, Enchantment
// You may look at the top card of your library any time.
// You may cast creature spells from the top of your library.
// Creatures you control have "{T}: Add one mana of any color."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elven-chorus"),
        name: "Elven Chorus".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "You may look at the top card of your library any time.\nYou may cast creature spells from the top of your library.\nCreatures you control have \"{T}: Add one mana of any color.\"".to_string(),
        abilities: vec![
            // CR 601.3 (PB-A): "You may look at the top card of your library any time.
            // You may cast creature spells from the top of your library."
            // look_at_top: true (controller sees top; not all players — distinguish from reveal_top).
            AbilityDefinition::StaticPlayFromTop {
                filter: PlayFromTopFilter::CreaturesOnly,
                look_at_top: true,
                reveal_top: false,
                pay_life_instead: false,
                condition: None,
                on_cast_effect: None,
            },
            // TODO: "Creatures you control have '{T}: Add one mana of any color.'"
            // Requires GrantActivatedAbility DSL support (separate gap). Deferred.
        ],
        ..Default::default()
    }
}

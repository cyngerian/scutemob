// Thundermane Dragon — {3}{R}, Creature — Dragon 4/4
// Flying
// You may look at the top card of your library any time.
// You may cast creature spells with power 4 or greater from the top of your library.
// If you cast a creature spell this way, it gains haste until end of turn.
// TODO: DSL gap — "look at top of library any time" is a static permission effect;
// casting from the top of library with a power filter has no DSL support.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thundermane-dragon"),
        name: "Thundermane Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nYou may look at the top card of your library any time.\nYou may cast creature spells with power 4 or greater from the top of your library. If you cast a creature spell this way, it gains haste until end of turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: look at top of library any time (static permission)
            // TODO: cast creature spells with power 4+ from top of library, grant haste if cast this way
        ],
        ..Default::default()
    }
}

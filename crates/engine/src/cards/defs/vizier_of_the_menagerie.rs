// Vizier of the Menagerie — {3}{G}, Creature — Snake Cleric 3/4
// You may look at the top card of your library any time.
// You may cast creature spells from the top of your library.
// You can spend mana as though it were mana of any type to cast creature spells.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vizier-of-the-menagerie"),
        name: "Vizier of the Menagerie".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Snake", "Cleric"]),
        oracle_text: "You may look at the top card of your library any time.\nYou may cast creature spells from the top of your library.\nYou can spend mana as though it were mana of any type to cast creature spells.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // CR 601.3 (PB-A): "You may look at the top card of your library any time.
            // You may cast creature spells from the top of your library."
            AbilityDefinition::StaticPlayFromTop {
                filter: PlayFromTopFilter::CreaturesOnly,
                look_at_top: true,
                reveal_top: false,
                pay_life_instead: false,
                condition: None,
                on_cast_effect: None,
            },
            // TODO: "You can spend mana as though it were mana of any type to cast creature spells."
            // Requires ManaRestriction relaxation DSL support (separate gap, mana spending rules).
            // Deferred — the play-from-top ability above is the primary implementation.
        ],
        ..Default::default()
    }
}

// Maelstrom of the Spirit Dragon — Land, {T}: Add {C}. {T}: Add any color (Dragon/Omen only, TODO). {4},{T}: Search for Dragon (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("maelstrom-of-the-spirit-dragon"),
        name: "Maelstrom of the Spirit Dragon".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast a Dragon spell or an Omen spell.\n{4}, {T}, Sacrifice this land: Search your library for a Dragon card, reveal it, put it into your hand, then shuffle.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}: Add one mana of any color. Spend this mana only to cast a Dragon spell
            // or an Omen spell. DSL gap: no mana-spending restriction on AddManaAnyColor.
            // TODO: {4}, {T}, Sacrifice this land: Search your library for a Dragon card, reveal
            // it, put it into your hand, then shuffle. DSL gap: SearchLibrary filter for creature
            // subtype (Dragon) not supported; only basic_land_filter() exists.
        ],
        ..Default::default()
    }
}

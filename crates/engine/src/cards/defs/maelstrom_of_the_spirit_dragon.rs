// Maelstrom of the Spirit Dragon — Land
// {T}: Add {C}.
// {T}: Add one mana of any color. Spend this mana only to cast a Dragon spell or an Omen spell.
// {4}, {T}, Sacrifice: Search library for Dragon card, reveal, put into hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("maelstrom-of-the-spirit-dragon"),
        name: "Maelstrom of the Spirit Dragon".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast a Dragon spell or an Omen spell.\n{4}, {T}, Sacrifice this land: Search your library for a Dragon card, reveal it, put it into your hand, then shuffle.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // {T}: Add one mana of any color. Spend this mana only to cast a Dragon spell
            // or an Omen spell.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColorRestricted {
                    player: PlayerTarget::Controller,
                    restriction: ManaRestriction::SubtypeOrSubtype(
                        SubType("Dragon".to_string()),
                        SubType("Omen".to_string()),
                    ),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {4}, {T}, Sacrifice this land: Search your library for a Dragon card,
            // reveal it, put it into your hand, then shuffle.
            // Blocked on: PB-17 SearchLibrary filter for creature subtype (Dragon).
        ],
        ..Default::default()
    }
}

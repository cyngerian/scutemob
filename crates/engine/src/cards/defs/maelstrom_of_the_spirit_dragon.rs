// Maelstrom of the Spirit Dragon — Land
// {T}: Add {C}.
// {T}: Add one mana of any color. Spend this mana only to cast a Dragon spell or an Omen spell.
// {4}, {T}, Sacrifice: Search library for Dragon card, reveal, put into hand, shuffle.
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
                activation_condition: None,
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
                activation_condition: None,
            },
            // {4}, {T}, Sacrifice: Search for a Dragon card, put into hand, shuffle.
            // CR 701.23
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 4,
                        ..Default::default()
                    }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            has_subtype: Some(SubType("Dragon".to_string())),
                            ..Default::default()
                        },
                        reveal: true,
                        destination: ZoneTarget::Hand {
                            owner: PlayerTarget::Controller,
                        },
                        shuffle_before_placing: false,
                    also_search_graveyard: false,
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::Controller,
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}

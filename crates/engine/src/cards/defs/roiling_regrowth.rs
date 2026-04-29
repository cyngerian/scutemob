// Roiling Regrowth — {2}{G} Instant
// Sacrifice a land. Search your library for up to two basic land cards,
// put them onto the battlefield tapped, then shuffle.
//
// Per ruling, "Sacrifice a land" happens at resolution (not as an additional cost —
// CR 601.2b does not apply). PB-SFT enables the land filter on SacrificePermanents.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("roiling-regrowth"),
        name: "Roiling Regrowth".to_string(),
        mana_cost: Some(ManaCost { green: 1, generic: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Sacrifice a land. Search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // "Sacrifice a land." — at resolution, the caster sacrifices one land
                    // they control. PB-SFT (CR 701.17a + CR 109.1c): land filter applied.
                    Effect::SacrificePermanents {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                        filter: Some(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        }),
                    },
                    // "Search your library for up to two basic land cards, put them onto
                    // the battlefield tapped, then shuffle." (two separate searches, same
                    // pattern as Explosive Vegetation.)
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

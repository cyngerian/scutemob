// Flare of Cultivation — {1}{G}{G}, Sorcery
// You may sacrifice a nontoken green creature rather than pay this spell's mana cost.
// Search your library for up to two basic land cards, reveal those cards, put one onto
// the battlefield tapped and the other into your hand, then shuffle.
//
// TODO: Alternative cost (sacrifice nontoken green creature) — AltCostKind lacks a
// SacrificeCreature variant with nontoken+color filter. See flare_of_fortitude.rs for
// the same pattern. Main effect is implemented below.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flare-of-cultivation"),
        name: "Flare of Cultivation".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "You may sacrifice a nontoken green creature rather than pay this spell's mana cost.\nSearch your library for up to two basic land cards, reveal those cards, put one onto the battlefield tapped and the other into your hand, then shuffle.".to_string(),
        abilities: vec![
            // TODO: Alt cost — sacrifice nontoken green creature. No AltCostKind::SacrificeCreature
            // variant with color/nontoken filter exists yet.

            // Main effect: search for up to two basic lands — one to battlefield tapped,
            // one to hand — then shuffle.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: true,
                        destination: ZoneTarget::Battlefield { tapped: true },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: basic_land_filter(),
                        reveal: true,
                        destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
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

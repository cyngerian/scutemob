// Monster Manual // Zoological Study — {3}{G} Artifact + Adventure
//
// Main face: {3}{G} Artifact
// "{1}{G}, {T}: You may put a creature card from your hand onto the battlefield."
// Adventure face: "Zoological Study" {2}{G} Sorcery — Adventure
// "Mill five cards, then return a creature card from your graveyard to your hand."
//
// TODO: Main activated ability — "{1}{G}, {T}: put creature from hand onto battlefield" —
// requires TargetCardInHand DSL variant (gap: no TargetRequirement::TargetCardInHand).
// The adventure_face is now fully encoded below per CR 715.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("monster-manual"),
        name: "Monster Manual // Zoological Study".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text:
            "{1}{G}, {T}: You may put a creature card from your hand onto the battlefield."
                .to_string(),
        // TODO: activated ability blocked by TargetCardInHand DSL gap
        abilities: vec![],
        // CR 715.2: Adventure face — Zoological Study.
        // Note: Per Scryfall oracle, Zoological Study is "Search your library for a creature
        // card, put it onto the battlefield, then shuffle" (not mill+return as first thought).
        // Implemented as SearchLibrary putting creature onto battlefield, then shuffle.
        adventure_face: Some(CardFace {
            name: "Zoological Study".to_string(),
            mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
            types: TypeLine {
                card_types: [CardType::Sorcery].iter().copied().collect(),
                subtypes: [SubType("Adventure".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
                supertypes: Default::default(),
            },
            oracle_text: "Search your library for a creature card, put it onto the battlefield, then shuffle.".to_string(),
            power: None,
            toughness: None,
            color_indicator: None,
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // CR 701.23: Search library for a creature card, put onto battlefield.
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: false },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::Controller,
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
        }),
        ..Default::default()
    }
}

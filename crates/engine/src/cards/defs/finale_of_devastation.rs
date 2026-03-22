// Finale of Devastation — {X}{G}{G}, Sorcery
// Search your library and/or graveyard for a creature card with mana value X or less and
// put it onto the battlefield. If you search your library this way, shuffle. If X is 10
// or more, creatures you control get +X/+X and gain haste until end of turn.
//
// Partial implementation: dual-zone search supported via also_search_graveyard: true.
// TODO: max_cmc should be dynamic (XValue) — max_cmc is Option<u32>, so dynamic
//       comparison needs EffectAmount-based filter or runtime evaluation.
// TODO: X >= 10 conditional pump + mass haste grant (Condition::XValueAtLeast not yet in DSL)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("finale-of-devastation"),
        name: "Finale of Devastation".to_string(),
        mana_cost: Some(ManaCost {
            green: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library and/or graveyard for a creature card with mana value X or less and put it onto the battlefield. If you search your library this way, shuffle. If X is 10 or more, creatures you control get +X/+X and gain haste until end of turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.23: Search library and graveyard for a creature card.
            // also_search_graveyard: true covers the "and/or graveyard" portion.
            // TODO: if X >= 10, all creatures get +X/+X and haste (Condition::XValueAtLeast gap).
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        // TODO: max_cmc should be dynamic (XValue) — needs EffectAmount-based filter.
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                    shuffle_before_placing: false,
                    also_search_graveyard: true,
                },
                Effect::Shuffle {
                    player: PlayerTarget::Controller,
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

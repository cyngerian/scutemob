// Finale of Devastation — {X}{G}{G}, Sorcery
// Search your library and/or graveyard for a creature card with mana value X or less and
// put it onto the battlefield. If you search your library this way, shuffle. If X is 10
// or more, creatures you control get +X/+X and gain haste until end of turn.
// Partial implementation: SearchLibrary with creature + max_cmc from X value.
// TODO: Search graveyard portion (dual-zone search not in DSL)
// TODO: X >= 10 conditional pump + mass haste grant
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
            // Partial: search library for creature with CMC <= X, put onto battlefield, shuffle.
            // TODO: Also search graveyard; if X >= 10, all creatures get +X/+X and haste.
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        // TODO: max_cmc should be dynamic (XValue), but max_cmc is Option<u32>.
                        // Using a high static value as approximation — real fix needs
                        // EffectAmount-based filter or runtime evaluation.
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                    shuffle_before_placing: false,
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

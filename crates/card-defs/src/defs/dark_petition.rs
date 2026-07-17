// Dark Petition — {3}{B}{B}, Sorcery; search for a card to hand, then shuffle.
// Spell mastery — if 2+ instant/sorcery cards in graveyard, add {B}{B}{B}.
//
// CR 207.2c: PB-AC6 added Condition::SpellMastery ("two or more instant and/or
// sorcery cards in your graveyard"), used here in an Effect::Conditional.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dark-petition"),
        name: "Dark Petition".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            black: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for a card, put that card into your hand, then \
                      shuffle.\nSpell mastery — If there are two or more instant and/or sorcery \
                      cards in your graveyard, add {B}{B}{B}."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter::default(),
                    reveal: false,
                    destination: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle {
                    player: PlayerTarget::Controller,
                },
                Effect::Conditional {
                    condition: Condition::SpellMastery,
                    if_true: Box::new(Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 3, 0, 0, 0),
                    }),
                    if_false: Box::new(Effect::Nothing),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

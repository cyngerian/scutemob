// Razaketh, the Foulblooded — {5}{B}{B}{B}, Legendary Creature — Demon 8/8
// Flying, trample
// Pay 2 life, Sacrifice another creature: Search your library for a card,
// put that card into your hand, then shuffle.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("razaketh-the-foulblooded"),
        name: "Razaketh, the Foulblooded".to_string(),
        mana_cost: Some(ManaCost { generic: 5, black: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Demon"],
        ),
        oracle_text:
            "Flying, trample\n\
             Pay 2 life, Sacrifice another creature: Search your library for a card, \
             put that card into your hand, then shuffle."
                .to_string(),
        power: Some(8),
        toughness: Some(8),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // Pay 2 life, Sacrifice another creature: search library for any card, to hand.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::PayLife(2),
                    Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter::default(),
                        reveal: false,
                        destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}

// Scion of the Ur-Dragon — {W}{U}{B}{R}{G}, Legendary Creature — Dragon Avatar 4/4
// Flying
// {2}: Search your library for a Dragon permanent card and put it into your graveyard.
//   If you do, Scion of the Ur-Dragon becomes a copy of that card until end of turn.
//   Then shuffle.
// TODO: "becomes a copy of that card until end of turn" — needs EffectTarget::LastSearchResult
//   to reference the card found by SearchLibrary. BecomeCopyOf infrastructure exists but
//   can't wire to the search result yet. Search-to-graveyard + shuffle works correctly.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scion-of-the-ur-dragon"),
        name: "Scion of the Ur-Dragon".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Avatar"],
        ),
        oracle_text: "Flying\n{2}: Search your library for a Dragon permanent card and put it into your graveyard. If you do, Scion of the Ur-Dragon becomes a copy of that card until end of turn. Then shuffle.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // {2}: Search for Dragon → graveyard, become copy until EOT, shuffle.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 2,
                    ..Default::default()
                }),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            has_subtype: Some(SubType("Dragon".to_string())),
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Graveyard {
                            owner: PlayerTarget::Controller,
                        },
                        shuffle_before_placing: false,
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

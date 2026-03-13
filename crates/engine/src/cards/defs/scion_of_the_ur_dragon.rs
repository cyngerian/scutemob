// Scion of the Ur-Dragon — {W}{U}{B}{R}{G}, Legendary Creature — Dragon Avatar 4/4
// Flying
// TODO: DSL gap — activated ability "{2}: Search your library for a Dragon permanent card and
//   put it into your graveyard. If you do, Scion of the Ur-Dragon becomes a copy of that card
//   until end of turn. Then shuffle."
//   (library search to graveyard with subtype filter, plus self-copy effect, not supported
//   in card DSL)
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
        ],
        ..Default::default()
    }
}

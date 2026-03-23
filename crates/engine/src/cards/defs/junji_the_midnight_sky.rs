// Junji, the Midnight Sky — {3}{B}{B}, Legendary Creature — Dragon Spirit 5/5
// Flying, menace
// When Junji dies, choose one —
// • Each opponent discards two cards and loses 2 life.
// • Put target non-Dragon creature card from a graveyard onto the battlefield under your
//   control. You lose 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("junji-the-midnight-sky"),
        name: "Junji, the Midnight Sky".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Spirit"],
        ),
        oracle_text: "Flying, menace\nWhen Junji, the Midnight Sky dies, choose one —\n• Each opponent discards two cards and loses 2 life.\n• Put target non-Dragon creature card from a graveyard onto the battlefield under your control. You lose 2 life.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // TODO: DSL gap — modal death trigger (WhenDies + choose one). Modal triggered
            // abilities not in DSL.
        ],
        ..Default::default()
    }
}

// Korvold, Fae-Cursed King — {2}{B}{R}{G}, Legendary Creature — Dragon Noble 4/4
// Flying
// Whenever Korvold enters or attacks, sacrifice another permanent.
// Whenever you sacrifice a permanent, put a +1/+1 counter on Korvold and draw a card.
//
// TODO: "Sacrifice another permanent" on ETB/attack — forced sacrifice not expressible.
// TODO: "Whenever you sacrifice a permanent" trigger not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("korvold-fae-cursed-king"),
        name: "Korvold, Fae-Cursed King".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, red: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Noble"],
        ),
        oracle_text: "Flying\nWhenever Korvold, Fae-Cursed King enters or attacks, sacrifice another permanent.\nWhenever you sacrifice a permanent, put a +1/+1 counter on Korvold and draw a card.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: Both sacrifice-related triggers not in DSL.
        ],
        ..Default::default()
    }
}

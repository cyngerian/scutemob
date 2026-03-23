// Adeline, Resplendent Cathar — {1}{W}{W}, Legendary Creature — Human Knight */4
// Vigilance
// Adeline's power is equal to the number of creatures you control.
// Whenever you attack, for each opponent, create a 1/1 white Human creature token
// that's tapped and attacking that player or a planeswalker they control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("adeline-resplendent-cathar"),
        name: "Adeline, Resplendent Cathar".to_string(),
        mana_cost: Some(ManaCost { white: 2, generic: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Knight"],
        ),
        oracle_text: "Vigilance\nAdeline's power is equal to the number of creatures you control.\nWhenever you attack, for each opponent, create a 1/1 white Human creature token that's tapped and attacking that player or a planeswalker they control.".to_string(),
        // CDA: power = number of creatures you control
        power: None,
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // TODO: CDA "power = number of creatures you control" not expressible in DSL.
            // TODO: Attack trigger creates tokens per-opponent (ForEach EachOpponent creating
            //   tokens entering attacking against that specific opponent) — DSL lacks
            //   per-target token creation. W5 policy: no approximation implemented.
        ],
        ..Default::default()
    }
}

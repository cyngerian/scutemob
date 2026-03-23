// Miirym, Sentinel Wyrm — {3}{G}{U}{R}, Legendary Creature — Dragon Spirit 6/6
// Flying, ward {2}
// Whenever another nontoken Dragon you control enters, create a token that's a copy of
// it, except the token isn't legendary.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("miirym-sentinel-wyrm"),
        name: "Miirym, Sentinel Wyrm".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, blue: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Spirit"],
        ),
        oracle_text: "Flying, ward {2}\nWhenever another nontoken Dragon you control enters, create a token that's a copy of it, except the token isn't legendary.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Ward(2)),
            // TODO: "Copy token of entering Dragon" — CreateCopyToken not in DSL.
        ],
        ..Default::default()
    }
}

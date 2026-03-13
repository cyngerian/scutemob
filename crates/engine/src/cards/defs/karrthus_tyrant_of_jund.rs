// Karrthus, Tyrant of Jund — {4}{B}{R}{G}, Legendary Creature — Dragon 7/7
// Flying, haste
// TODO: DSL gap — ETB triggered ability "gain control of all Dragons, then untap all Dragons"
//   (mass control change + untap for subtype-filtered permanents not supported in card DSL)
// TODO: DSL gap — static ability "Other Dragon creatures you control have haste."
//   (subtype-filtered keyword grant for "other" creatures not supported in card DSL)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("karrthus-tyrant-of-jund"),
        name: "Karrthus, Tyrant of Jund".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying, haste\nWhen Karrthus enters, gain control of all Dragons, then untap all Dragons.\nOther Dragon creatures you control have haste.".to_string(),
        power: Some(7),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
        ],
        ..Default::default()
    }
}

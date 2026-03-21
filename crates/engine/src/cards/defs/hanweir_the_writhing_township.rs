// Hanweir, the Writhing Township — Melded permanent definition
// CR 712.5b: Hanweir Garrison and Hanweir Battlements meld to form this.
//
// This CardDefinition exists solely to hold the melded back_face characteristics.
// It is never cast directly. Both Hanweir Garrison and Hanweir Battlements reference
// this card_id in their meld_pair.melded_card_id.
//
// Melded face: Legendary Creature — Eldrazi Ooze, 7/4, Trample, Haste
// "Whenever Hanweir, the Writhing Township attacks, create two 3/2 colorless
//  Eldrazi Horror creature tokens that are tapped and attacking."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hanweir-the-writhing-township"),
        name: "Hanweir, the Writhing Township".to_string(),
        mana_cost: None,
        types: TypeLine::default(),
        oracle_text: String::new(),
        abilities: vec![],
        power: None,
        toughness: None,
        back_face: Some(CardFace {
            name: "Hanweir, the Writhing Township".to_string(),
            mana_cost: None,
            types: full_types(
                &[SuperType::Legendary],
                &[CardType::Creature],
                &["Eldrazi", "Ooze"],
            ),
            oracle_text: "Trample, haste\nWhenever Hanweir, the Writhing Township attacks, create two 3/2 colorless Eldrazi Horror creature tokens that are tapped and attacking.".to_string(),
            abilities: vec![
                AbilityDefinition::Keyword(KeywordAbility::Trample),
                AbilityDefinition::Keyword(KeywordAbility::Haste),
                // CR 508.4: Attack trigger — create two 3/2 colorless Eldrazi Horror tokens
                // tapped and attacking. Tokens inherit the attack target (CR 508.4).
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WhenAttacks,
                    effect: Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Eldrazi Horror".to_string(),
                            power: 3,
                            toughness: 2,
                            colors: im::OrdSet::new(), // colorless
                            card_types: [CardType::Creature].iter().copied().collect(),
                            subtypes: [
                                SubType("Eldrazi".to_string()),
                                SubType("Horror".to_string()),
                            ]
                            .iter()
                            .cloned()
                            .collect(),
                            count: 2,
                            tapped: true,
                            enters_attacking: true,
                            ..Default::default()
                        },
                    },
                    intervening_if: None,
                    targets: vec![],
                },
            ],
            power: Some(7),
            toughness: Some(4),
            color_indicator: None,
        }),
        ..Default::default()
    }
}
